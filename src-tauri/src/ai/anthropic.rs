//! Anthropic Messages API adapter (raw HTTP; Rust has no official SDK).
//! `POST /v1/messages` with `stream: true`; tools are offered with
//! `eager_input_streaming`, so their arguments are parsed strictly here
//! before anything runs. Thinking blocks are kept verbatim and sent back
//! unchanged within a conversation, as the API requires. Refusal fallbacks
//! are on for the models that support them.
//!
//! Experimental: unit-tested against recorded streams, not yet run against
//! the live API (D29).

use blotter_core::store::providers::{Provider, ProviderKind};
use futures_util::StreamExt;
use serde_json::{Value, json};

use super::sse;
use super::{
    AiError, Block, ChatRequest, ChatResponse, Delta, Message, ModelInfo, Role, Stop, http_client,
    parse_tool_input,
};

const DEFAULT_BASE: &str = "https://api.anthropic.com";
const VERSION: &str = "2023-06-01";
const FALLBACK_BETA: &str = "server-side-fallback-2026-07-01";

fn base(provider: &Provider) -> String {
    provider
        .base_url
        .as_deref()
        .unwrap_or(DEFAULT_BASE)
        .trim_end_matches('/')
        .to_owned()
}

/// Whether the model takes server-side refusal fallbacks (`fallbacks:
/// "default"`); sending it to others is a 400.
fn supports_fallbacks(model: &str) -> bool {
    [
        "claude-sonnet-5-5",
        "claude-opus-5-5",
        "claude-opus-5",
        "claude-fable-5-1",
    ]
    .contains(&model)
}

fn block_json(b: &Block) -> Option<Value> {
    Some(match b {
        Block::Text { text } => json!({ "type": "text", "text": text }),
        Block::ToolUse {
            id, name, input, ..
        } => {
            let input = if input.is_object() {
                input.clone()
            } else {
                json!({})
            };
            json!({ "type": "tool_use", "id": id, "name": name, "input": input })
        }
        Block::ToolResult {
            tool_use_id,
            content,
            is_error,
        } => json!({
            "type": "tool_result",
            "tool_use_id": tool_use_id,
            "content": content,
            "is_error": is_error,
        }),
        Block::Raw {
            provider: ProviderKind::Anthropic,
            value,
        } => value.clone(),
        Block::Raw { .. } => return None,
    })
}

fn message_json(m: &Message) -> Option<Value> {
    let content: Vec<Value> = m.content.iter().filter_map(block_json).collect();
    if content.is_empty() {
        return None;
    }
    let role = match m.role {
        Role::User => "user",
        Role::Assistant => "assistant",
    };
    Some(json!({ "role": role, "content": content }))
}

/// The request body for a streamed turn.
pub fn request_body(req: &ChatRequest) -> Value {
    let mut body = json!({
        "model": req.model,
        "max_tokens": req.max_tokens,
        "stream": true,
        "messages": req.messages.iter().filter_map(message_json).collect::<Vec<_>>(),
    });
    if !req.system.is_empty() {
        body["system"] = json!(req.system);
    }
    if !req.tools.is_empty() {
        body["tools"] = req
            .tools
            .iter()
            .map(|t| {
                json!({
                    "name": t.name,
                    "description": t.description,
                    "input_schema": t.input_schema,
                    "eager_input_streaming": true,
                })
            })
            .collect();
    }
    if supports_fallbacks(&req.model) {
        body["fallbacks"] = json!("default");
    }
    body
}

/// A content block being assembled from the stream.
#[derive(Debug)]
enum Partial {
    Text(String),
    ToolUse {
        id: String,
        name: String,
        json: String,
    },
    Thinking {
        block: Value,
        thinking: String,
        signature: String,
    },
    /// Anything else (redacted thinking, fallback markers): kept as given.
    Other(Value),
}

/// Assembles a reply from Anthropic stream events.
#[derive(Debug, Default)]
pub struct StreamState {
    blocks: Vec<Option<Partial>>,
    stop: Option<String>,
    input_tokens: Option<u64>,
    output_tokens: Option<u64>,
}

impl StreamState {
    fn slot(&mut self, index: usize) -> &mut Option<Partial> {
        if self.blocks.len() <= index {
            self.blocks.resize_with(index + 1, || None);
        }
        &mut self.blocks[index]
    }

    /// Applies one SSE event's data.
    pub fn apply(&mut self, data: &str, on_delta: &mut impl FnMut(Delta)) -> Result<(), AiError> {
        let ev: Value = serde_json::from_str(data).map_err(|e| AiError::Protocol {
            message: format!("unreadable stream event ({e})"),
        })?;
        let index = ev["index"].as_u64().unwrap_or(0) as usize;
        match ev["type"].as_str().unwrap_or("") {
            "message_start" => {
                self.input_tokens = ev["message"]["usage"]["input_tokens"].as_u64();
            }
            "content_block_start" => {
                let b = &ev["content_block"];
                let partial = match b["type"].as_str().unwrap_or("") {
                    "text" => Partial::Text(b["text"].as_str().unwrap_or("").to_owned()),
                    "tool_use" => {
                        let name = b["name"].as_str().unwrap_or("").to_owned();
                        on_delta(Delta::ToolStart { name: name.clone() });
                        Partial::ToolUse {
                            id: b["id"].as_str().unwrap_or("").to_owned(),
                            name,
                            json: String::new(),
                        }
                    }
                    "thinking" => Partial::Thinking {
                        block: b.clone(),
                        thinking: b["thinking"].as_str().unwrap_or("").to_owned(),
                        signature: b["signature"].as_str().unwrap_or("").to_owned(),
                    },
                    _ => Partial::Other(b.clone()),
                };
                *self.slot(index) = Some(partial);
            }
            "content_block_delta" => {
                let d = &ev["delta"];
                match (self.slot(index).as_mut(), d["type"].as_str().unwrap_or("")) {
                    (Some(Partial::Text(t)), "text_delta") => {
                        let piece = d["text"].as_str().unwrap_or("");
                        t.push_str(piece);
                        on_delta(Delta::Text {
                            text: piece.to_owned(),
                        });
                    }
                    (Some(Partial::ToolUse { json, .. }), "input_json_delta") => {
                        json.push_str(d["partial_json"].as_str().unwrap_or(""));
                    }
                    (Some(Partial::Thinking { thinking, .. }), "thinking_delta") => {
                        thinking.push_str(d["thinking"].as_str().unwrap_or(""));
                    }
                    (Some(Partial::Thinking { signature, .. }), "signature_delta") => {
                        signature.push_str(d["signature"].as_str().unwrap_or(""));
                    }
                    _ => {} // citations and future delta types: not needed here
                }
            }
            "message_delta" => {
                if let Some(s) = ev["delta"]["stop_reason"].as_str() {
                    self.stop = Some(s.to_owned());
                }
                if let Some(n) = ev["usage"]["output_tokens"].as_u64() {
                    self.output_tokens = Some(n);
                }
            }
            "error" => {
                let e = &ev["error"];
                let message = e["message"].as_str().unwrap_or("unknown error").to_owned();
                return Err(match e["type"].as_str().unwrap_or("") {
                    "overloaded_error" | "rate_limit_error" => AiError::RateLimited { message },
                    "authentication_error" | "permission_error" => AiError::Auth { message },
                    "invalid_request_error" | "not_found_error" => AiError::Request { message },
                    _ => AiError::Unavailable { message },
                });
            }
            _ => {} // content_block_stop, message_stop, ping
        }
        Ok(())
    }

    pub fn finish(self) -> ChatResponse {
        let content = self
            .blocks
            .into_iter()
            .flatten()
            .map(|p| match p {
                Partial::Text(text) => Block::Text { text },
                Partial::ToolUse { id, name, json } => {
                    let (input, invalid) = parse_tool_input(&json);
                    Block::ToolUse {
                        id,
                        name,
                        input,
                        invalid,
                    }
                }
                Partial::Thinking {
                    mut block,
                    thinking,
                    signature,
                } => {
                    block["thinking"] = json!(thinking);
                    block["signature"] = json!(signature);
                    Block::Raw {
                        provider: ProviderKind::Anthropic,
                        value: block,
                    }
                }
                Partial::Other(value) => Block::Raw {
                    provider: ProviderKind::Anthropic,
                    value,
                },
            })
            .collect();
        let stop = match self.stop.as_deref() {
            Some("end_turn") | Some("stop_sequence") => Stop::EndTurn,
            Some("tool_use") => Stop::ToolUse,
            Some("max_tokens") => Stop::MaxTokens,
            Some("refusal") => Stop::Refusal,
            Some(other) => Stop::Other(other.to_owned()),
            None => Stop::Other("incomplete".to_owned()),
        };
        ChatResponse {
            content,
            stop,
            input_tokens: self.input_tokens,
            output_tokens: self.output_tokens,
        }
    }
}

/// The message from an Anthropic error body (`{"type":"error","error":{…}}`).
fn error_message(body: &str) -> String {
    serde_json::from_str::<Value>(body)
        .ok()
        .and_then(|v| v["error"]["message"].as_str().map(str::to_owned))
        .unwrap_or_else(|| body.chars().take(300).collect())
}

fn authed(rb: reqwest::RequestBuilder, key: &str) -> reqwest::RequestBuilder {
    rb.header("x-api-key", key)
        .header("anthropic-version", VERSION)
}

pub async fn chat(
    provider: &Provider,
    key: &str,
    req: &ChatRequest,
    mut on_delta: impl FnMut(Delta),
) -> Result<ChatResponse, AiError> {
    let client = http_client()?;
    let mut rb = authed(client.post(format!("{}/v1/messages", base(provider))), key)
        .json(&request_body(req));
    if supports_fallbacks(&req.model) {
        rb = rb.header("anthropic-beta", FALLBACK_BETA);
    }
    let res = rb.send().await.map_err(AiError::from_reqwest)?;
    let status = res.status();
    if !status.is_success() {
        let body = res.text().await.unwrap_or_default();
        return Err(AiError::from_status(status.as_u16(), error_message(&body)));
    }
    let mut parser = sse::Parser::default();
    let mut state = StreamState::default();
    let mut bytes = res.bytes_stream();
    while let Some(chunk) = bytes.next().await {
        let chunk = chunk.map_err(AiError::from_reqwest)?;
        for ev in parser.feed(&chunk) {
            state.apply(&ev.data, &mut on_delta)?;
        }
    }
    if let Some(ev) = parser.finish() {
        state.apply(&ev.data, &mut on_delta)?;
    }
    Ok(state.finish())
}

pub async fn list_models(provider: &Provider, key: &str) -> Result<Vec<ModelInfo>, AiError> {
    let client = http_client()?;
    let mut out = Vec::new();
    let mut after: Option<String> = None;
    loop {
        let mut url = format!("{}/v1/models?limit=100", base(provider));
        if let Some(a) = &after {
            url.push_str(&format!("&after_id={a}"));
        }
        let res = authed(client.get(url), key)
            .send()
            .await
            .map_err(AiError::from_reqwest)?;
        let status = res.status();
        let body = res.text().await.map_err(AiError::from_reqwest)?;
        if !status.is_success() {
            return Err(AiError::from_status(status.as_u16(), error_message(&body)));
        }
        let page: Value = serde_json::from_str(&body).map_err(|e| AiError::Protocol {
            message: format!("unreadable model list ({e})"),
        })?;
        for m in page["data"].as_array().into_iter().flatten() {
            if let Some(id) = m["id"].as_str() {
                out.push(ModelInfo {
                    id: id.to_owned(),
                    display_name: m["display_name"].as_str().map(str::to_owned),
                });
            }
        }
        match (page["has_more"].as_bool(), page["last_id"].as_str()) {
            (Some(true), Some(last)) => after = Some(last.to_owned()),
            _ => break,
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::ToolSpec;

    /// A recorded-shape stream: text, a thinking block, then a tool call
    /// whose arguments arrive in pieces.
    const STREAM: &str = "event: message_start\n\
data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_1\",\"usage\":{\"input_tokens\":42}}}\n\n\
event: content_block_start\n\
data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"thinking\",\"thinking\":\"\",\"signature\":\"\"}}\n\n\
event: content_block_delta\n\
data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"signature_delta\",\"signature\":\"sig==\"}}\n\n\
event: content_block_start\n\
data: {\"type\":\"content_block_start\",\"index\":1,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}\n\n\
event: content_block_delta\n\
data: {\"type\":\"content_block_delta\",\"index\":1,\"delta\":{\"type\":\"text_delta\",\"text\":\"Adding \"}}\n\n\
event: content_block_delta\n\
data: {\"type\":\"content_block_delta\",\"index\":1,\"delta\":{\"type\":\"text_delta\",\"text\":\"it.\"}}\n\n\
event: content_block_start\n\
data: {\"type\":\"content_block_start\",\"index\":2,\"content_block\":{\"type\":\"tool_use\",\"id\":\"toolu_1\",\"name\":\"create_item\",\"input\":{}}}\n\n\
event: content_block_delta\n\
data: {\"type\":\"content_block_delta\",\"index\":2,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"{\\\"title\\\": \\\"Gy\"}}\n\n\
event: content_block_delta\n\
data: {\"type\":\"content_block_delta\",\"index\":2,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"m\\\"}\"}}\n\n\
event: content_block_stop\n\
data: {\"type\":\"content_block_stop\",\"index\":2}\n\n\
event: message_delta\n\
data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"tool_use\"},\"usage\":{\"output_tokens\":17}}\n\n\
event: message_stop\n\
data: {\"type\":\"message_stop\"}\n\n";

    fn run(stream: &str) -> (Result<ChatResponse, AiError>, Vec<Delta>) {
        let mut deltas = Vec::new();
        let mut parser = sse::Parser::default();
        let mut state = StreamState::default();
        // Feed in awkward 7-byte chunks, as a network would.
        for chunk in stream.as_bytes().chunks(7) {
            for ev in parser.feed(chunk) {
                if let Err(e) = state.apply(&ev.data, &mut |d| deltas.push(d)) {
                    return (Err(e), deltas);
                }
            }
        }
        (Ok(state.finish()), deltas)
    }

    #[test]
    fn assembles_text_thinking_and_tool_calls() {
        let (res, deltas) = run(STREAM);
        let res = res.unwrap();
        assert_eq!(res.stop, Stop::ToolUse);
        assert_eq!((res.input_tokens, res.output_tokens), (Some(42), Some(17)));
        assert_eq!(
            deltas,
            [
                Delta::Text {
                    text: "Adding ".into()
                },
                Delta::Text { text: "it.".into() },
                Delta::ToolStart {
                    name: "create_item".into()
                },
            ]
        );
        let Block::Raw { value, .. } = &res.content[0] else {
            panic!("thinking kept raw")
        };
        assert_eq!(value["signature"], "sig==");
        assert_eq!(
            res.content[1],
            Block::Text {
                text: "Adding it.".into()
            }
        );
        assert_eq!(
            res.content[2],
            Block::ToolUse {
                id: "toolu_1".into(),
                name: "create_item".into(),
                input: json!({ "title": "Gym" }),
                invalid: None,
            }
        );
    }

    #[test]
    fn truncated_tool_input_is_flagged_not_run() {
        // Drop the closing quote and brace from the last fragment: {"title": "Gym
        let cut = STREAM.replace(r#"m\"}"#, "m");
        assert_ne!(cut, STREAM);
        let (res, _) = run(&cut);
        let res = res.unwrap();
        let Block::ToolUse { invalid, .. } = &res.content[2] else {
            panic!()
        };
        assert!(invalid.is_some());
    }

    #[test]
    fn stream_errors_are_classified() {
        let s = "event: error\ndata: {\"type\":\"error\",\"error\":{\"type\":\"overloaded_error\",\"message\":\"Overloaded\"}}\n\n";
        assert!(matches!(run(s).0, Err(AiError::RateLimited { .. })));
    }

    #[test]
    fn request_body_round_trips_the_conversation() {
        let req = ChatRequest {
            model: "claude-sonnet-5-5".into(),
            system: "You plan days.".into(),
            messages: vec![
                Message::user_text("Add gym"),
                Message {
                    role: Role::Assistant,
                    content: vec![
                        Block::Raw {
                            provider: ProviderKind::Anthropic,
                            value: json!({"type":"thinking","thinking":"","signature":"s"}),
                        },
                        Block::Raw {
                            provider: ProviderKind::Openai,
                            value: json!({"x":1}),
                        },
                        Block::ToolUse {
                            id: "t1".into(),
                            name: "create_item".into(),
                            input: json!({"title":"Gym"}),
                            invalid: None,
                        },
                    ],
                },
                Message {
                    role: Role::User,
                    content: vec![Block::ToolResult {
                        tool_use_id: "t1".into(),
                        content: "{\"id\":\"i1\"}".into(),
                        is_error: false,
                    }],
                },
            ],
            tools: vec![ToolSpec {
                name: "create_item".into(),
                description: "Create".into(),
                input_schema: json!({"type":"object"}),
            }],
            max_tokens: 64000,
        };
        let body = request_body(&req);
        assert_eq!(body["system"], "You plan days.");
        assert_eq!(body["stream"], true);
        assert_eq!(body["fallbacks"], "default");
        assert_eq!(body["tools"][0]["eager_input_streaming"], true);
        let assistant = &body["messages"][1]["content"];
        assert_eq!(
            assistant.as_array().unwrap().len(),
            2,
            "foreign raw blocks dropped"
        );
        assert_eq!(assistant[0]["signature"], "s", "thinking echoed unchanged");
        assert_eq!(body["messages"][2]["content"][0]["type"], "tool_result");

        let older = ChatRequest {
            model: "claude-haiku-4-5".into(),
            ..req
        };
        assert!(request_body(&older).get("fallbacks").is_none());
    }
}

//! OpenAI-style chat completions: OpenAI itself and compatible servers
//! (Ollama, LM Studio, llama.cpp server, OpenRouter, GitHub Models, …).
//! `POST {base}/chat/completions` with `stream: true`; tool calls arrive as
//! indexed fragments whose arguments are joined, then parsed strictly.
//!
//! Experimental: unit-tested against recorded streams, not yet run against
//! a live server (D29).

use blotter_core::store::providers::{Provider, ProviderKind};
use futures_util::StreamExt;
use serde_json::{Value, json};

use super::sse;
use super::{
    AiError, Block, ChatRequest, ChatResponse, Delta, ModelInfo, Role, Stop, http_client,
    parse_tool_input,
};

const OPENAI_BASE: &str = "https://api.openai.com/v1";

fn base(provider: &Provider) -> String {
    match (&provider.base_url, provider.kind) {
        (Some(u), _) if !u.trim().is_empty() => u.trim().trim_end_matches('/').to_owned(),
        _ => OPENAI_BASE.to_owned(),
    }
}

/// Converts the conversation to OpenAI messages: a tool call becomes an
/// assistant `tool_calls` entry, each result a `tool` message.
pub fn messages_json(req: &ChatRequest) -> Vec<Value> {
    let mut out = Vec::new();
    if !req.system.is_empty() {
        out.push(json!({ "role": "system", "content": req.system }));
    }
    for m in &req.messages {
        let text: String = m
            .content
            .iter()
            .filter_map(|b| match b {
                Block::Text { text } => Some(text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n");
        match m.role {
            Role::User => {
                for b in &m.content {
                    if let Block::ToolResult {
                        tool_use_id,
                        content,
                        is_error,
                    } = b
                    {
                        let content = if *is_error {
                            format!("Error: {content}")
                        } else {
                            content.clone()
                        };
                        out.push(json!({
                            "role": "tool",
                            "tool_call_id": tool_use_id,
                            "content": content,
                        }));
                    }
                }
                if !text.is_empty() {
                    out.push(json!({ "role": "user", "content": text }));
                }
            }
            Role::Assistant => {
                let calls: Vec<Value> = m
                    .content
                    .iter()
                    .filter_map(|b| match b {
                        Block::ToolUse { id, name, input, .. } => Some(json!({
                            "id": id,
                            "type": "function",
                            "function": {
                                "name": name,
                                "arguments": if input.is_object() { input.to_string() } else { "{}".to_owned() },
                            },
                        })),
                        _ => None,
                    })
                    .collect();
                let mut msg = json!({
                    "role": "assistant",
                    "content": if text.is_empty() { Value::Null } else { json!(text) },
                });
                if !calls.is_empty() {
                    msg["tool_calls"] = json!(calls);
                }
                out.push(msg);
            }
        }
    }
    out
}

pub fn request_body(req: &ChatRequest) -> Value {
    let mut body = json!({
        "model": req.model,
        "stream": true,
        "stream_options": { "include_usage": true },
        "messages": messages_json(req),
    });
    if !req.tools.is_empty() {
        body["tools"] = req
            .tools
            .iter()
            .map(|t| {
                json!({
                    "type": "function",
                    "function": {
                        "name": t.name,
                        "description": t.description,
                        "parameters": t.input_schema,
                    },
                })
            })
            .collect();
    }
    body
}

#[derive(Debug, Default)]
struct PartialCall {
    id: String,
    name: String,
    args: String,
}

/// Assembles a reply from OpenAI-style stream chunks.
#[derive(Debug, Default)]
pub struct StreamState {
    text: String,
    calls: Vec<PartialCall>,
    finish: Option<String>,
    input_tokens: Option<u64>,
    output_tokens: Option<u64>,
    done: bool,
}

impl StreamState {
    pub fn apply(&mut self, data: &str, on_delta: &mut impl FnMut(Delta)) -> Result<(), AiError> {
        if data.trim() == "[DONE]" {
            self.done = true;
            return Ok(());
        }
        let chunk: Value = serde_json::from_str(data).map_err(|e| AiError::Protocol {
            message: format!("unreadable stream chunk ({e})"),
        })?;
        if let Some(e) = chunk.get("error") {
            let message = e["message"].as_str().unwrap_or("unknown error").to_owned();
            return Err(AiError::Unavailable { message });
        }
        if let Some(u) = chunk.get("usage").filter(|u| !u.is_null()) {
            self.input_tokens = u["prompt_tokens"].as_u64();
            self.output_tokens = u["completion_tokens"].as_u64();
        }
        let Some(choice) = chunk["choices"].get(0) else {
            return Ok(());
        };
        let delta = &choice["delta"];
        if let Some(t) = delta["content"].as_str().filter(|t| !t.is_empty()) {
            self.text.push_str(t);
            on_delta(Delta::Text { text: t.to_owned() });
        }
        for tc in delta["tool_calls"].as_array().into_iter().flatten() {
            let i = tc["index"].as_u64().unwrap_or(0) as usize;
            if self.calls.len() <= i {
                self.calls.resize_with(i + 1, PartialCall::default);
            }
            let call = &mut self.calls[i];
            if let Some(id) = tc["id"].as_str() {
                call.id = id.to_owned();
            }
            if let Some(name) = tc["function"]["name"].as_str() {
                if call.name.is_empty() {
                    on_delta(Delta::ToolStart {
                        name: name.to_owned(),
                    });
                }
                call.name.push_str(name);
            }
            if let Some(a) = tc["function"]["arguments"].as_str() {
                call.args.push_str(a);
            }
        }
        if let Some(f) = choice["finish_reason"].as_str() {
            self.finish = Some(f.to_owned());
        }
        Ok(())
    }

    pub fn finish(self) -> ChatResponse {
        let mut content = Vec::new();
        if !self.text.is_empty() {
            content.push(Block::Text { text: self.text });
        }
        for (i, c) in self.calls.into_iter().enumerate() {
            let (input, invalid) = parse_tool_input(&c.args);
            content.push(Block::ToolUse {
                // Some local servers omit ids; give each call a stable one.
                id: if c.id.is_empty() {
                    format!("call_{i}")
                } else {
                    c.id
                },
                name: c.name,
                input,
                invalid,
            });
        }
        let stop = match self.finish.as_deref() {
            Some("stop") => Stop::EndTurn,
            Some("tool_calls") | Some("function_call") => Stop::ToolUse,
            Some("length") => Stop::MaxTokens,
            Some("content_filter") => Stop::Refusal,
            Some(other) => Stop::Other(other.to_owned()),
            None if self.done => Stop::EndTurn,
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

fn error_message(body: &str) -> String {
    serde_json::from_str::<Value>(body)
        .ok()
        .and_then(|v| {
            v["error"]["message"]
                .as_str()
                .or_else(|| v["error"].as_str())
                .map(str::to_owned)
        })
        .unwrap_or_else(|| body.chars().take(300).collect())
}

fn authed(rb: reqwest::RequestBuilder, key: Option<&str>) -> reqwest::RequestBuilder {
    match key.filter(|k| !k.is_empty()) {
        Some(k) => rb.bearer_auth(k),
        None => rb,
    }
}

pub async fn chat(
    provider: &Provider,
    key: Option<&str>,
    req: &ChatRequest,
    mut on_delta: impl FnMut(Delta),
) -> Result<ChatResponse, AiError> {
    let client = http_client()?;
    let res = authed(
        client.post(format!("{}/chat/completions", base(provider))),
        key,
    )
    .json(&request_body(req))
    .send()
    .await
    .map_err(AiError::from_reqwest)?;
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

pub async fn list_models(
    provider: &Provider,
    key: Option<&str>,
) -> Result<Vec<ModelInfo>, AiError> {
    let client = http_client()?;
    let res = authed(client.get(format!("{}/models", base(provider))), key)
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
    let mut out: Vec<ModelInfo> = page["data"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|m| m["id"].as_str())
        .map(|id| ModelInfo {
            id: id.to_owned(),
            display_name: None,
        })
        .collect();
    out.sort_by(|a, b| a.id.cmp(&b.id));
    // OpenAI's list includes embeddings, audio and image models; keep chat-capable ones.
    if provider.kind == ProviderKind::Openai {
        out.retain(|m| {
            ![
                "embedding",
                "tts",
                "whisper",
                "dall-e",
                "moderation",
                "audio",
                "image",
                "realtime",
            ]
            .iter()
            .any(|x| m.id.contains(x))
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::{Message, ToolSpec};

    const STREAM: &str = "data: {\"choices\":[{\"index\":0,\"delta\":{\"role\":\"assistant\",\"content\":\"On \"}}]}\n\n\
data: {\"choices\":[{\"index\":0,\"delta\":{\"content\":\"it.\"}}]}\n\n\
data: {\"choices\":[{\"index\":0,\"delta\":{\"tool_calls\":[{\"index\":0,\"id\":\"call_a\",\"type\":\"function\",\"function\":{\"name\":\"create_item\",\"arguments\":\"\"}}]}}]}\n\n\
data: {\"choices\":[{\"index\":0,\"delta\":{\"tool_calls\":[{\"index\":0,\"function\":{\"arguments\":\"{\\\"title\\\":\"}}]}}]}\n\n\
data: {\"choices\":[{\"index\":0,\"delta\":{\"tool_calls\":[{\"index\":0,\"function\":{\"arguments\":\"\\\"Gym\\\"}\"}}]}}]}\n\n\
data: {\"choices\":[{\"index\":0,\"delta\":{},\"finish_reason\":\"tool_calls\"}]}\n\n\
data: {\"choices\":[],\"usage\":{\"prompt_tokens\":30,\"completion_tokens\":9}}\n\n\
data: [DONE]\n\n";

    #[test]
    fn assembles_text_and_tool_calls() {
        let mut deltas = Vec::new();
        let mut parser = sse::Parser::default();
        let mut state = StreamState::default();
        for chunk in STREAM.as_bytes().chunks(11) {
            for ev in parser.feed(chunk) {
                state.apply(&ev.data, &mut |d| deltas.push(d)).unwrap();
            }
        }
        let res = state.finish();
        assert_eq!(res.stop, Stop::ToolUse);
        assert_eq!((res.input_tokens, res.output_tokens), (Some(30), Some(9)));
        assert_eq!(
            res.content[0],
            Block::Text {
                text: "On it.".into()
            }
        );
        assert_eq!(
            res.content[1],
            Block::ToolUse {
                id: "call_a".into(),
                name: "create_item".into(),
                input: json!({ "title": "Gym" }),
                invalid: None,
            }
        );
        assert_eq!(deltas.len(), 3);
    }

    #[test]
    fn converts_tool_calls_and_results_to_openai_messages() {
        let req = ChatRequest {
            model: "m".into(),
            system: "Sys".into(),
            messages: vec![
                Message::user_text("Add gym"),
                Message {
                    role: Role::Assistant,
                    content: vec![
                        Block::Raw {
                            provider: ProviderKind::Anthropic,
                            value: json!({}),
                        },
                        Block::ToolUse {
                            id: "c1".into(),
                            name: "create_item".into(),
                            input: json!({"title":"Gym"}),
                            invalid: None,
                        },
                    ],
                },
                Message {
                    role: Role::User,
                    content: vec![Block::ToolResult {
                        tool_use_id: "c1".into(),
                        content: "bad".into(),
                        is_error: true,
                    }],
                },
            ],
            tools: vec![ToolSpec {
                name: "create_item".into(),
                description: "Create".into(),
                input_schema: json!({"type":"object"}),
            }],
            max_tokens: 1000,
        };
        let body = request_body(&req);
        let m = body["messages"].as_array().unwrap();
        assert_eq!(m[0]["role"], "system");
        assert_eq!(
            m[2]["tool_calls"][0]["function"]["arguments"],
            "{\"title\":\"Gym\"}"
        );
        assert!(m[2]["content"].is_null());
        assert_eq!(
            m[3],
            json!({ "role": "tool", "tool_call_id": "c1", "content": "Error: bad" })
        );
        assert_eq!(body["tools"][0]["function"]["parameters"]["type"], "object");
    }
}

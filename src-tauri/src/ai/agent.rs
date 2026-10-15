//! The tool loop for API-key providers (roadmap §4.6). The model gets the
//! agent operations as tools; each call is validated against its schema
//! and run as the AI. Invalid arguments and planner errors go back as
//! tool errors so the model can retry, which matters for local models.
//! The loop stops after [`MAX_ROUNDS`] rounds of tool calls.
//!
//! Until proposals arrive (#32) changes apply straight away; a snapshot is
//! taken before the first change of each turn, and each change undoes.

use std::future::Future;
use std::path::Path;
use std::sync::{Arc, Mutex};

use blotter_core::snapshots::{self, Reason};
use blotter_core::{Actor, Planner};

use super::{AiError, Block, ChatRequest, ChatResponse, Delta, Message, Role, Stop, ToolSpec};

/// Where a model call streams its text as it arrives.
pub type DeltaSink = Box<dyn FnMut(Delta) + Send>;

/// Tool rounds per user message before the loop gives up.
pub const MAX_ROUNDS: usize = 8;

/// Something the chat shows as it happens.
#[derive(Debug, Clone, PartialEq)]
pub enum Step {
    Text(String),
    ToolStart {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    ToolResult {
        id: String,
        name: String,
        is_error: bool,
        text: String,
        batch_id: Option<String>,
    },
}

/// What a turn produced.
#[derive(Debug, Default)]
pub struct TurnOutput {
    /// The new messages, to append to the thread: the model's turns and the
    /// tool results sent back.
    pub messages: Vec<Message>,
    /// Change-log batches written, in order.
    pub batches: Vec<String>,
    /// Set when the turn ended for a reason the user should know.
    pub notice: Option<String>,
}

/// The agent operations as tools.
pub fn tools(planner: &Planner) -> Vec<ToolSpec> {
    planner
        .registry()
        .agent_ops()
        .map(|d| ToolSpec {
            name: d.name.to_owned(),
            description: d.description.to_owned(),
            input_schema: d.schema.clone(),
        })
        .collect()
}

/// Runs one user turn: model, tools, model … until it answers.
///
/// `call` sends one request to the model (streaming text through the
/// `Delta` callback). `db_path` locates the snapshots folder; `None`
/// skips the snapshot (tests on an in-memory database).
pub async fn run_turn<C, F>(
    planner: &Mutex<Planner>,
    db_path: Option<&Path>,
    actor: &Actor,
    mut request: ChatRequest,
    mut call: C,
    on_step: Arc<dyn Fn(Step) + Send + Sync>,
) -> Result<TurnOutput, AiError>
where
    C: FnMut(ChatRequest, DeltaSink) -> F,
    F: Future<Output = Result<ChatResponse, AiError>>,
{
    let mut out = TurnOutput::default();
    let mut snapshotted = false;
    for round in 0..=MAX_ROUNDS {
        let stream = Arc::clone(&on_step);
        let response = call(
            request.clone(),
            Box::new(move |d| {
                if let Delta::Text { text } = d {
                    stream(Step::Text(text));
                }
            }),
        )
        .await?;

        let reply = Message {
            role: Role::Assistant,
            content: response.content.clone(),
        };
        request.messages.push(reply.clone());
        out.messages.push(reply);

        match response.stop {
            Stop::ToolUse => {}
            Stop::MaxTokens => {
                out.notice = Some("The reply was cut off at the length limit.".to_owned());
                return Ok(out);
            }
            Stop::Refusal => {
                out.notice = Some("The provider declined to answer that.".to_owned());
                return Ok(out);
            }
            Stop::EndTurn | Stop::Other(_) => return Ok(out),
        }
        if round == MAX_ROUNDS {
            out.notice = Some(format!(
                "Stopped after {MAX_ROUNDS} rounds of tool calls without an answer."
            ));
            return Ok(out);
        }

        let mut results = Vec::new();
        for block in &response.content {
            let Block::ToolUse {
                id,
                name,
                input,
                invalid,
            } = block
            else {
                continue;
            };
            on_step(Step::ToolStart {
                id: id.clone(),
                name: name.clone(),
                input: input.clone(),
            });
            let (text, is_error, batch_id) = match invalid {
                Some(problem) => (
                    format!("Not run: {problem}. Send the call again."),
                    true,
                    None,
                ),
                None => run_tool(planner, db_path, actor, name, input, &mut snapshotted),
            };
            if let Some(b) = &batch_id {
                out.batches.push(b.clone());
            }
            on_step(Step::ToolResult {
                id: id.clone(),
                name: name.clone(),
                is_error,
                text: text.clone(),
                batch_id,
            });
            results.push(Block::ToolResult {
                tool_use_id: id.clone(),
                content: text,
                is_error,
            });
        }
        let back = Message {
            role: Role::User,
            content: results,
        };
        request.messages.push(back.clone());
        out.messages.push(back);
    }
    Ok(out)
}

/// Runs one tool call, returning the result text, whether it failed, and
/// the batch it wrote.
fn run_tool(
    planner: &Mutex<Planner>,
    db_path: Option<&Path>,
    actor: &Actor,
    name: &str,
    input: &serde_json::Value,
    snapshotted: &mut bool,
) -> (String, bool, Option<String>) {
    let Ok(mut planner) = planner.lock() else {
        return ("The planner is unavailable.".to_owned(), true, None);
    };
    let mutates = match planner.registry().get(name) {
        Some(def) if blotter_core::ops::agent_may_run(def.name) => def.mutates,
        _ => return (format!("There is no tool called '{name}'."), true, None),
    };
    if mutates && !*snapshotted {
        if let Some(path) = db_path {
            let conn = planner.db().conn();
            let taken = snapshots::dir(conn, path).and_then(|dir| {
                let note = format!("Before changes from the chat ({actor})");
                snapshots::create(conn, &dir, Reason::BeforeAiBatch, Some(note), planner.now())
            });
            if let Err(err) = taken {
                return (
                    format!(
                        "Nothing was changed: Blotter couldn't take its safety snapshot first ({err})."
                    ),
                    true,
                    None,
                );
            }
        }
        *snapshotted = true;
    }
    match planner.execute(name, input.clone(), actor) {
        Ok(outcome) => (outcome.result.to_string(), false, outcome.batch_id),
        Err(err) => (err.for_agent(), true, None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use blotter_core::Database;
    use serde_json::json;
    use std::collections::VecDeque;

    fn planner() -> Mutex<Planner> {
        Mutex::new(Planner::new(
            Database::open_in_memory().unwrap(),
            chrono_tz::UTC,
        ))
    }

    fn ai() -> Actor {
        Actor::Ai {
            provider: "anthropic".into(),
            model: "test".into(),
        }
    }

    fn request(p: &Mutex<Planner>) -> ChatRequest {
        ChatRequest {
            model: "test".into(),
            system: "sys".into(),
            messages: vec![Message::user_text("Add milk to my inbox")],
            tools: tools(&p.lock().unwrap()),
            max_tokens: 1024,
        }
    }

    fn tool_use(id: &str, name: &str, input: serde_json::Value) -> Block {
        Block::ToolUse {
            id: id.into(),
            name: name.into(),
            input,
            invalid: None,
        }
    }

    fn reply(content: Vec<Block>, stop: Stop) -> ChatResponse {
        ChatResponse {
            content,
            stop,
            input_tokens: None,
            output_tokens: None,
        }
    }

    type Seen = Arc<Mutex<Vec<ChatRequest>>>;

    /// A model that answers from a script and records what it was sent.
    fn scripted(
        script: Vec<ChatResponse>,
    ) -> (
        impl FnMut(ChatRequest, DeltaSink) -> std::future::Ready<Result<ChatResponse, AiError>>,
        Seen,
    ) {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let log = Arc::clone(&seen);
        let mut script: VecDeque<_> = script.into();
        let call = move |req: ChatRequest, mut delta: DeltaSink| {
            log.lock().unwrap().push(req);
            let next = script.pop_front().expect("model called more than scripted");
            for b in &next.content {
                if let Block::Text { text } = b {
                    delta(Delta::Text { text: text.clone() });
                }
            }
            std::future::ready(Ok(next))
        };
        (call, seen)
    }

    #[tokio::test]
    async fn runs_tools_and_retries_after_an_error() {
        let p = planner();
        let (call, seen) = scripted(vec![
            // Bad arguments first, then a fix, then the answer.
            reply(
                vec![tool_use("t1", "create_item", json!({ "priority": 9 }))],
                Stop::ToolUse,
            ),
            reply(
                vec![tool_use("t2", "create_item", json!({ "title": "Milk" }))],
                Stop::ToolUse,
            ),
            reply(
                vec![Block::Text {
                    text: "Added Milk.".into(),
                }],
                Stop::EndTurn,
            ),
        ]);
        let steps = Arc::new(Mutex::new(Vec::new()));
        let sink = Arc::clone(&steps);
        let out = run_turn(
            &p,
            None,
            &ai(),
            request(&p),
            call,
            Arc::new(move |s| {
                sink.lock().unwrap().push(s);
            }),
        )
        .await
        .unwrap();
        let steps = steps.lock().unwrap();

        assert_eq!(out.messages.len(), 5, "3 replies + 2 tool-result messages");
        assert_eq!(out.batches.len(), 1);
        assert!(out.notice.is_none());
        let errors: Vec<bool> = steps
            .iter()
            .filter_map(|s| match s {
                Step::ToolResult { is_error, .. } => Some(*is_error),
                _ => None,
            })
            .collect();
        assert_eq!(errors, [true, false]);
        assert!(steps.contains(&Step::Text("Added Milk.".into())));

        // The model saw the validation problems before retrying.
        let second = &seen.lock().unwrap()[1];
        match &second.messages.last().unwrap().content[0] {
            Block::ToolResult {
                is_error, content, ..
            } => {
                assert!(is_error);
                assert!(content.contains("title"), "{content}");
            }
            other => panic!("{other:?}"),
        }
        // The item exists, logged as the AI.
        let planner = p.lock().unwrap();
        let actor: String = planner
            .db()
            .conn()
            .query_row("SELECT actor FROM changes LIMIT 1", [], |r| r.get(0))
            .unwrap();
        assert_eq!(actor, "ai:anthropic/test");
    }

    #[tokio::test]
    async fn malformed_and_user_only_calls_never_run() {
        let p = planner();
        let (call, _) = scripted(vec![
            reply(
                vec![
                    Block::ToolUse {
                        id: "t1".into(),
                        name: "create_item".into(),
                        input: serde_json::Value::Null,
                        invalid: Some("arguments weren't valid JSON".into()),
                    },
                    tool_use("t2", "undo", json!({})),
                ],
                Stop::ToolUse,
            ),
            reply(
                vec![Block::Text {
                    text: "Sorry.".into(),
                }],
                Stop::EndTurn,
            ),
        ]);
        let out = run_turn(&p, None, &ai(), request(&p), call, Arc::new(|_| {}))
            .await
            .unwrap();
        assert!(out.batches.is_empty());
        let changes: i64 = p
            .lock()
            .unwrap()
            .db()
            .conn()
            .query_row("SELECT COUNT(*) FROM changes", [], |r| r.get(0))
            .unwrap();
        assert_eq!(changes, 0);
    }

    #[tokio::test]
    async fn stops_after_the_round_cap() {
        let p = planner();
        let looping: Vec<_> = (0..=MAX_ROUNDS)
            .map(|i| {
                reply(
                    vec![tool_use(&format!("t{i}"), "list_inbox", json!({}))],
                    Stop::ToolUse,
                )
            })
            .collect();
        let (call, seen) = scripted(looping);
        let out = run_turn(&p, None, &ai(), request(&p), call, Arc::new(|_| {}))
            .await
            .unwrap();
        assert_eq!(seen.lock().unwrap().len(), MAX_ROUNDS + 1);
        assert!(out.notice.unwrap().contains("Stopped after"));
    }

    #[tokio::test]
    async fn cut_off_replies_do_not_run_tools() {
        let p = planner();
        let (call, _) = scripted(vec![reply(
            vec![tool_use("t1", "create_item", json!({ "title": "Half" }))],
            Stop::MaxTokens,
        )]);
        let out = run_turn(&p, None, &ai(), request(&p), call, Arc::new(|_| {}))
            .await
            .unwrap();
        assert!(out.batches.is_empty());
        assert!(out.notice.is_some());
    }
}

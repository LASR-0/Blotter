//! The AI chat (roadmap §4.6): threads persisted in the database, one turn
//! at a time per thread, streamed to the UI as `chat:event`s. A turn runs
//! through Claude Code (the default, D29) or the default API provider.
//!
//! The chat panel arrives with #31; these commands are its backend.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use blotter_core::context;
use blotter_core::settings::{KEY_AI_ROUTE, KEY_CLAUDE_CODE_MODEL, KEY_CLAUDE_CODE_PATH};
use blotter_core::store::chat::{self, ChatMessage, ChatRole, Thread};
use blotter_core::store::{self, providers};
use blotter_core::{Actor, Planner};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tauri::async_runtime::JoinHandle;
use tauri::{AppHandle, Emitter, Manager, State};

use super::agent::{self, Step};
use super::claude_code::{self, Event};
use super::commands::ai_err;
use super::{Block, ChatRequest, Message, Role, keychain};
use crate::AppState;
use crate::commands::{CHANGED_EVENT, ChangedPayload, CommandError};

/// Streamed to the UI while a turn runs.
pub const CHAT_EVENT: &str = "chat:event";
/// Reply length cap for API providers.
const MAX_TOKENS: u32 = 8192;
/// Tool results shown in the chat are cut to this many characters.
const RESULT_PREVIEW: usize = 400;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Route {
    /// The user's signed-in Claude Code, on their subscription (D29).
    #[default]
    ClaudeCode,
    /// The default API-key provider (experimental).
    Api,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChatEvent {
    Text {
        text: String,
    },
    ToolStart {
        id: String,
        name: String,
        input: Value,
    },
    ToolResult {
        id: String,
        name: String,
        is_error: bool,
        preview: String,
    },
    /// The turn finished; its messages are saved.
    Done {
        notice: Option<String>,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize)]
struct ChatEventPayload {
    thread_id: String,
    #[serde(flatten)]
    event: ChatEvent,
}

/// Turns in flight, by thread, so they can be cancelled.
#[derive(Default)]
pub struct ChatRuns(Mutex<HashMap<String, JoinHandle<()>>>);

fn lock_err() -> CommandError {
    CommandError::internal("planner state is unavailable after an earlier crash")
}

fn emit(app: &AppHandle, thread_id: &str, event: ChatEvent) {
    let _ = app.emit(
        CHAT_EVENT,
        ChatEventPayload {
            thread_id: thread_id.to_owned(),
            event,
        },
    );
}

fn preview(text: &str) -> String {
    if text.chars().count() <= RESULT_PREVIEW {
        return text.to_owned();
    }
    let cut: String = text.chars().take(RESULT_PREVIEW).collect();
    format!("{cut}…")
}

// ---- Settings -------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiSettings {
    pub route: Route,
    pub claude_code_model: String,
    /// Empty: find `claude` automatically.
    pub claude_code_path: String,
}

fn read_settings(planner: &Planner) -> Result<AiSettings, CommandError> {
    let conn = planner.db().conn();
    Ok(AiSettings {
        route: store::settings::get(conn, KEY_AI_ROUTE)?.unwrap_or_default(),
        claude_code_model: store::settings::get::<String>(conn, KEY_CLAUDE_CODE_MODEL)?
            .filter(|m| !m.trim().is_empty())
            .unwrap_or_else(|| claude_code::DEFAULT_MODEL.to_owned()),
        claude_code_path: store::settings::get(conn, KEY_CLAUDE_CODE_PATH)?.unwrap_or_default(),
    })
}

#[tauri::command]
pub fn get_ai_settings(state: State<'_, AppState>) -> Result<AiSettings, CommandError> {
    let planner = state.planner.lock().map_err(|_| lock_err())?;
    read_settings(&planner)
}

#[derive(Debug, Deserialize)]
pub struct AiSettingsInput {
    pub route: Option<Route>,
    pub claude_code_model: Option<String>,
    pub claude_code_path: Option<String>,
}

#[tauri::command]
pub fn set_ai_settings(
    state: State<'_, AppState>,
    input: AiSettingsInput,
) -> Result<AiSettings, CommandError> {
    let planner = state.planner.lock().map_err(|_| lock_err())?;
    let conn = planner.db().conn();
    let now = Utc::now();
    if let Some(route) = input.route {
        store::settings::set(conn, KEY_AI_ROUTE, &route, now)?;
    }
    if let Some(model) = input.claude_code_model {
        store::settings::set(conn, KEY_CLAUDE_CODE_MODEL, &model.trim(), now)?;
    }
    if let Some(path) = input.claude_code_path {
        store::settings::set(conn, KEY_CLAUDE_CODE_PATH, &path.trim(), now)?;
    }
    read_settings(&planner)
}

/// Whether Claude Code is installed and signed in.
#[tauri::command]
pub async fn claude_code_status(
    state: State<'_, AppState>,
) -> Result<claude_code::Status, CommandError> {
    let path = {
        let planner = state.planner.lock().map_err(|_| lock_err())?;
        read_settings(&planner)?.claude_code_path
    };
    Ok(claude_code::status(Some(&path)).await)
}

// ---- Threads --------------------------------------------------------------

#[tauri::command]
pub fn chat_threads(state: State<'_, AppState>) -> Result<Vec<Thread>, CommandError> {
    let planner = state.planner.lock().map_err(|_| lock_err())?;
    Ok(chat::list_threads(planner.db().conn())?)
}

#[tauri::command]
pub fn chat_messages(
    state: State<'_, AppState>,
    thread_id: String,
) -> Result<Vec<ChatMessage>, CommandError> {
    let planner = state.planner.lock().map_err(|_| lock_err())?;
    Ok(chat::messages(planner.db().conn(), &thread_id)?)
}

#[tauri::command]
pub fn chat_delete_thread(
    state: State<'_, AppState>,
    runs: State<'_, ChatRuns>,
    thread_id: String,
) -> Result<(), CommandError> {
    cancel(&runs, &thread_id);
    let planner = state.planner.lock().map_err(|_| lock_err())?;
    chat::delete_thread(planner.db().conn(), &thread_id)?;
    Ok(())
}

fn cancel(runs: &ChatRuns, thread_id: &str) -> bool {
    let handle = runs.0.lock().ok().and_then(|mut r| r.remove(thread_id));
    handle.map(|h| h.abort()).is_some()
}

/// Stops the thread's turn. Claude Code is killed with it; changes already
/// made stay (and can be undone).
#[tauri::command]
pub fn chat_cancel(app: AppHandle, runs: State<'_, ChatRuns>, thread_id: String) -> bool {
    let stopped = cancel(&runs, &thread_id);
    if stopped {
        emit(
            &app,
            &thread_id,
            ChatEvent::Done {
                notice: Some("Stopped.".to_owned()),
            },
        );
    }
    stopped
}

// ---- Turns ----------------------------------------------------------------

/// Everything a turn needs, gathered under one lock.
struct Prepared {
    thread: Thread,
    text: String,
    system: String,
    settings: AiSettings,
    /// For the API route: the default provider and the thread so far.
    api: Option<(providers::Provider, Vec<Message>)>,
    data_dir: PathBuf,
    db_path: PathBuf,
}

fn prepare(
    state: &AppState,
    thread_id: Option<String>,
    text: String,
) -> Result<Prepared, CommandError> {
    let mut planner = state.planner.lock().map_err(|_| lock_err())?;
    let settings = read_settings(&planner)?;
    let provider = match settings.route {
        Route::Api => Some(
            providers::default_provider(planner.db().conn())?.ok_or_else(|| {
                CommandError::internal("Add an AI provider in Settings → AI providers first.")
            })?,
        ),
        Route::ClaudeCode => None,
    };
    let now = Utc::now();
    let thread = match &thread_id {
        Some(id) => chat::require_thread(planner.db().conn(), id)?,
        None => chat::create_thread(
            planner.db().conn(),
            &text,
            provider.as_ref().map(|p| p.id.as_str()),
            now,
        )?,
    };
    let history = match &provider {
        Some(_) => to_messages(&chat::messages(planner.db().conn(), &thread.id)?),
        None => Vec::new(),
    };
    chat::append(
        planner.db().conn(),
        &thread.id,
        ChatRole::User,
        &text,
        None,
        None,
        now,
    )?;
    let system = context::system_prompt(&mut planner)?;
    let data_dir = state
        .db_path
        .parent()
        .map(PathBuf::from)
        .unwrap_or_default();
    Ok(Prepared {
        thread,
        text: text.clone(),
        system,
        settings,
        api: provider.map(|p| {
            let mut messages = history;
            messages.push(Message::user_text(text));
            (p, messages)
        }),
        data_dir,
        db_path: state.db_path.clone(),
    })
}

/// A thread's saved messages as the conversation for an API provider.
/// Claude Code turns (saved with their tool activity in one message) are
/// reduced to their text.
fn to_messages(saved: &[ChatMessage]) -> Vec<Message> {
    saved
        .iter()
        .filter_map(|m| {
            let blocks: Option<Vec<Block>> = m
                .blocks
                .clone()
                .and_then(|b| serde_json::from_value(b).ok());
            match m.role {
                ChatRole::User => Some(Message::user_text(m.content.clone())),
                ChatRole::Tool => blocks.map(|content| Message {
                    role: Role::User,
                    content,
                }),
                ChatRole::Assistant => {
                    let content = match blocks {
                        Some(b) if !b.iter().any(|x| matches!(x, Block::ToolResult { .. })) => b,
                        _ if m.content.is_empty() => return None,
                        _ => vec![Block::Text {
                            text: m.content.clone(),
                        }],
                    };
                    Some(Message {
                        role: Role::Assistant,
                        content,
                    })
                }
            }
        })
        .collect()
}

fn text_of(blocks: &[Block]) -> String {
    blocks
        .iter()
        .filter_map(|b| match b {
            Block::Text { text } => Some(text.as_str()),
            _ => None,
        })
        .collect()
}

/// Sends a message, starting a thread when `thread_id` is empty. Returns
/// the thread id at once; the reply streams as `chat:event`s.
#[tauri::command]
pub fn chat_send(
    app: AppHandle,
    state: State<'_, AppState>,
    runs: State<'_, ChatRuns>,
    thread_id: Option<String>,
    text: String,
) -> Result<String, CommandError> {
    let text = text.trim().to_owned();
    if text.is_empty() {
        return Err(CommandError::internal("Type a message first."));
    }
    // Held until the turn is registered, so two sends can't race, and a
    // turn that ends at once can't unregister before it's registered.
    let mut in_flight = runs.0.lock().map_err(|_| lock_err())?;
    if thread_id
        .as_ref()
        .is_some_and(|id| in_flight.contains_key(id))
    {
        return Err(CommandError::internal("Still answering the last message."));
    }
    let prepared = prepare(&state, thread_id, text)?;
    let id = prepared.thread.id.clone();
    let task_app = app.clone();
    let handle = tauri::async_runtime::spawn(async move {
        let thread_id = prepared.thread.id.clone();
        let result = match prepared.settings.route {
            Route::ClaudeCode => run_claude_code(&task_app, prepared).await,
            Route::Api => run_api(&task_app, prepared).await,
        };
        match result {
            Ok(notice) => emit(&task_app, &thread_id, ChatEvent::Done { notice }),
            Err(message) => {
                save_error(&task_app, &thread_id, &message);
                emit(&task_app, &thread_id, ChatEvent::Error { message });
            }
        }
        if let Some(runs) = task_app.try_state::<ChatRuns>()
            && let Ok(mut r) = runs.0.lock()
        {
            r.remove(&thread_id);
        }
    });
    in_flight.insert(id.clone(), handle);
    Ok(id)
}

/// Runs `f` on the planner, if it's available.
fn with_planner<T>(app: &AppHandle, f: impl FnOnce(&Planner) -> T) -> Option<T> {
    let state = app.try_state::<AppState>()?;
    let planner = state.planner.lock().ok()?;
    Some(f(&planner))
}

/// Keeps a failed turn visible in the thread.
fn save_error(app: &AppHandle, thread_id: &str, message: &str) {
    let blocks = json!([{ "type": "error", "message": message }]);
    save(app, thread_id, ChatRole::Assistant, "", Some(&blocks), None);
}

fn save(
    app: &AppHandle,
    thread_id: &str,
    role: ChatRole,
    text: &str,
    blocks: Option<&Value>,
    batch: Option<&str>,
) {
    let saved = with_planner(app, |p| {
        chat::append(
            p.db().conn(),
            thread_id,
            role,
            text,
            blocks,
            batch,
            Utc::now(),
        )
    });
    if let Some(Err(err)) = saved {
        eprintln!("blotter: couldn't save a chat message: {err}");
    }
}

async fn run_api(app: &AppHandle, p: Prepared) -> Result<Option<String>, String> {
    let Some((provider, messages)) = p.api else {
        return Err("No AI provider is set up.".to_owned());
    };
    let key = keychain::get(&provider.id).map_err(|e| e.message)?;
    let state = app.state::<AppState>();
    let tools = {
        let planner = state
            .planner
            .lock()
            .map_err(|_| "The planner is unavailable.")?;
        agent::tools(&planner)
    };
    let actor = Actor::Ai {
        provider: serde_json::to_value(provider.kind)
            .ok()
            .and_then(|v| v.as_str().map(str::to_owned))
            .unwrap_or_default(),
        model: provider.model.clone(),
    };
    let request = ChatRequest {
        model: provider.model.clone(),
        system: p.system,
        messages,
        tools,
        max_tokens: MAX_TOKENS,
    };
    let thread_id = p.thread.id.clone();
    let step_app = app.clone();
    let on_step: Arc<dyn Fn(Step) + Send + Sync> = Arc::new(move |step| {
        let event = match step {
            Step::Text(text) => ChatEvent::Text { text },
            Step::ToolStart { id, name, input } => ChatEvent::ToolStart { id, name, input },
            Step::ToolResult {
                id,
                name,
                is_error,
                text,
                batch_id,
            } => {
                if let Some(batch_id) = batch_id {
                    let _ = step_app.emit(
                        CHANGED_EVENT,
                        ChangedPayload {
                            batch_id,
                            ops: vec![name.clone()],
                        },
                    );
                }
                ChatEvent::ToolResult {
                    id,
                    name,
                    is_error,
                    preview: preview(&text),
                }
            }
        };
        emit(&step_app, &thread_id, event);
    });
    let call = |req: ChatRequest, mut delta: agent::DeltaSink| {
        let provider = provider.clone();
        let key = key.clone();
        async move { super::chat(&provider, key.as_deref(), &req, &mut delta).await }
    };
    let out = agent::run_turn(
        &state.planner,
        Some(&p.db_path),
        &actor,
        request,
        call,
        on_step,
    )
    .await
    .map_err(|e| ai_err(e).message)?;
    let last_batch = out.batches.last().cloned();
    for m in &out.messages {
        let blocks = serde_json::to_value(&m.content).ok();
        match m.role {
            Role::Assistant => save(
                app,
                &p.thread.id,
                ChatRole::Assistant,
                &text_of(&m.content),
                blocks.as_ref(),
                last_batch.as_deref(),
            ),
            Role::User => save(app, &p.thread.id, ChatRole::Tool, "", blocks.as_ref(), None),
        }
    }
    Ok(out.notice)
}

async fn run_claude_code(app: &AppHandle, p: Prepared) -> Result<Option<String>, String> {
    let exe = claude_code::find(Some(&p.settings.claude_code_path)).ok_or_else(|| {
        "Claude Code isn't installed, or Blotter can't find it. Set it up in Settings → AI providers."
            .to_owned()
    })?;
    let blotter =
        std::env::current_exe().map_err(|e| format!("Couldn't find Blotter's own program: {e}"))?;
    let work = p.data_dir.join("claude-code");
    std::fs::create_dir_all(&work)
        .map_err(|e| format!("Couldn't prepare a folder for Claude Code: {e}"))?;
    let config_path = work.join("blotter-mcp.json");
    let config = claude_code::mcp_config(&blotter, &p.data_dir, &p.settings.claude_code_model);
    std::fs::write(&config_path, config.to_string())
        .map_err(|e| format!("Couldn't write Claude Code's tool settings: {e}"))?;

    let mut resume = p.thread.external_session.clone();
    loop {
        let turn = claude_code::Turn {
            prompt: &p.text,
            system_prompt: &p.system,
            model: &p.settings.claude_code_model,
            resume: resume.as_deref(),
            mcp_config: &config_path,
        };
        let mut blocks: Vec<Block> = Vec::new();
        let mut names: HashMap<String, String> = HashMap::new();
        let mut session = None;
        let mut final_text = String::new();
        let mut failed = false;
        let mut tools_ok = true;
        let thread_id = p.thread.id.clone();
        let ran = claude_code::run(&exe, &work, &turn, |event| match event {
            Event::Init {
                session_id,
                tools_ok: ok,
            } => {
                session = Some(session_id);
                tools_ok = ok;
            }
            Event::Text(text) => emit(app, &thread_id, ChatEvent::Text { text }),
            Event::ToolStart { id, name, input } => {
                names.insert(id.clone(), name.clone());
                emit(
                    app,
                    &thread_id,
                    ChatEvent::ToolStart {
                        id: id.clone(),
                        name: name.clone(),
                        input: input.clone(),
                    },
                );
                blocks.push(Block::ToolUse {
                    id,
                    name,
                    input,
                    invalid: None,
                });
            }
            Event::ToolResult { id, is_error, text } => {
                let name = names.get(&id).cloned().unwrap_or_default();
                emit(
                    app,
                    &thread_id,
                    ChatEvent::ToolResult {
                        id: id.clone(),
                        name,
                        is_error,
                        preview: preview(&text),
                    },
                );
                blocks.push(Block::ToolResult {
                    tool_use_id: id,
                    content: text,
                    is_error,
                });
            }
            Event::Done {
                session_id,
                text,
                is_error,
            } => {
                session = session_id.or(session.take());
                final_text = text;
                failed = is_error;
            }
        })
        .await;

        // A session Claude Code no longer has: start a fresh one once.
        let lost_session = resume.is_some()
            && match &ran {
                Err(msg) => msg.contains("No conversation found"),
                Ok(()) => failed && final_text.contains("No conversation found"),
            };
        if lost_session {
            resume = None;
            continue;
        }
        ran?;
        if failed {
            return Err(if final_text.is_empty() {
                "Claude Code couldn't answer.".to_owned()
            } else {
                final_text
            });
        }
        blocks.push(Block::Text {
            text: final_text.clone(),
        });
        let blocks = serde_json::to_value(&blocks).ok();
        save(
            app,
            &p.thread.id,
            ChatRole::Assistant,
            &final_text,
            blocks.as_ref(),
            None,
        );
        with_planner(app, |planner| {
            chat::set_external_session(planner.db().conn(), &p.thread.id, session.as_deref())
        });
        return Ok((!tools_ok).then(|| {
            "Blotter's tools didn't start for Claude Code, so it couldn't see your planner."
                .to_owned()
        }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use blotter_core::time::Instant;
    use chrono::TimeZone;

    fn saved(role: ChatRole, content: &str, blocks: Option<Value>) -> ChatMessage {
        ChatMessage {
            id: "m".into(),
            thread_id: "t".into(),
            role,
            content: content.into(),
            blocks,
            batch_id: None,
            created_at: Utc.with_ymd_and_hms(2026, 10, 15, 0, 0, 0).unwrap() as Instant,
        }
    }

    #[test]
    fn saved_threads_replay_to_api_providers() {
        let api_turn =
            json!([{ "type": "tool_use", "id": "t1", "name": "list_inbox", "input": {} }]);
        let results = json!([{ "type": "tool_result", "tool_use_id": "t1", "content": "[]", "is_error": false }]);
        let claude_turn = json!([
            { "type": "tool_use", "id": "c1", "name": "list_inbox", "input": {} },
            { "type": "tool_result", "tool_use_id": "c1", "content": "[]", "is_error": false },
            { "type": "text", "text": "Empty." }
        ]);
        let msgs = to_messages(&[
            saved(ChatRole::User, "What's in my inbox?", None),
            saved(ChatRole::Assistant, "", Some(api_turn)),
            saved(ChatRole::Tool, "", Some(results)),
            saved(
                ChatRole::Assistant,
                "Nothing.",
                Some(json!([{ "type": "text", "text": "Nothing." }])),
            ),
            saved(ChatRole::User, "And now?", None),
            saved(ChatRole::Assistant, "Empty.", Some(claude_turn)),
            saved(
                ChatRole::Assistant,
                "",
                Some(json!([{ "type": "error", "message": "x" }])),
            ),
        ]);
        let roles: Vec<Role> = msgs.iter().map(|m| m.role).collect();
        assert_eq!(
            roles,
            [
                Role::User,
                Role::Assistant,
                Role::User,
                Role::Assistant,
                Role::User,
                Role::Assistant
            ]
        );
        assert!(matches!(msgs[1].content[0], Block::ToolUse { .. }));
        assert!(matches!(msgs[2].content[0], Block::ToolResult { .. }));
        assert_eq!(
            msgs[5].content,
            [Block::Text {
                text: "Empty.".into()
            }],
            "Claude Code turn as text"
        );
    }

    #[test]
    fn previews_are_cut() {
        assert_eq!(preview("short"), "short");
        let long = "x".repeat(RESULT_PREVIEW + 10);
        assert!(preview(&long).ends_with('…'));
    }
}

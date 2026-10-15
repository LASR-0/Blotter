//! The Claude Code route (D29): Blotter drives the user's installed,
//! signed-in `claude` program headless, on their existing subscription,
//! with Blotter's own MCP server as its only tools. Each chat message is
//! one `claude -p` run streaming JSON events; a thread resumes its Claude
//! session so the conversation carries on.
//!
//! Claude Code gets no built-in tools (no shell, no files), only the
//! `blotter` MCP server, and runs in its own folder under the app data so
//! no project instructions are picked up. The server logs its changes as
//! `ai:claude-code/<model>` and snapshots before the first one.

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Duration;

use serde::Serialize;
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, Command};
use tokio::sync::mpsc;

use crate::{DATA_DIR_ENV, MCP_ACTOR_ENV, cli};

/// The model when the user hasn't picked one: the latest Sonnet.
pub const DEFAULT_MODEL: &str = "sonnet";
/// The name Claude Code gives Blotter's tools: `mcp__blotter__<op>`.
const SERVER: &str = "blotter";
const TOOL_PREFIX: &str = "mcp__blotter__";
/// The permission rule allowing every Blotter tool (and nothing else).
/// Without it, `dontAsk` mode refuses every tool that changes anything.
const ALLOW_RULE: &str = "mcp__blotter";
/// `--version` and `auth status` should answer quickly.
const PROBE_TIMEOUT: Duration = Duration::from_secs(20);

#[cfg(windows)]
const EXE: &str = "claude.exe";
#[cfg(not(windows))]
const EXE: &str = "claude";

/// Finds `claude`: the path the user set, then `PATH`, then the places
/// its installers put it (the app may have started before an install
/// changed `PATH`).
pub fn find(override_path: Option<&str>) -> Option<PathBuf> {
    if let Some(p) = override_path.map(str::trim).filter(|p| !p.is_empty()) {
        let p = PathBuf::from(p);
        return p.is_file().then_some(p);
    }
    let on_path = std::env::var_os("PATH")
        .map(|paths| std::env::split_paths(&paths).collect::<Vec<_>>())
        .unwrap_or_default();
    on_path
        .into_iter()
        .chain(install_dirs())
        .map(|dir| dir.join(EXE))
        .find(|p| p.is_file())
}

/// Where Claude Code's installers put the program.
fn install_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(home) = dirs::home_dir() {
        dirs.push(home.join(".local").join("bin"));
        dirs.push(home.join(".claude").join("local"));
    }
    #[cfg(windows)]
    if let Some(local) = dirs::data_local_dir() {
        let winget = local.join("Microsoft").join("WinGet");
        dirs.push(winget.join("Links"));
        if let Ok(entries) = std::fs::read_dir(winget.join("Packages")) {
            dirs.extend(
                entries
                    .flatten()
                    .filter(|e| {
                        e.file_name()
                            .to_string_lossy()
                            .starts_with("Anthropic.ClaudeCode")
                    })
                    .map(|e| e.path()),
            );
        }
    }
    #[cfg(not(windows))]
    {
        dirs.push(PathBuf::from("/usr/local/bin"));
        dirs.push(PathBuf::from("/opt/homebrew/bin"));
    }
    dirs
}

/// Variables a Claude Code session sets for the programs it runs. Blotter
/// started from inside one (a terminal in an editor with Claude Code)
/// inherits them, and passing them on would make Blotter's `claude` act as
/// a child of that session. Configuration such as `CLAUDE_CONFIG_DIR` and
/// `ANTHROPIC_*` is kept.
fn is_session_var(name: &str) -> bool {
    name == "CLAUDECODE"
        || name == "CLAUDE_PID"
        || name == "CLAUDE_EFFORT"
        || name == "CLAUDE_AGENT_SDK_VERSION"
        || name == "MCP_CONNECTION_NONBLOCKING"
        || (name.starts_with("CLAUDE_CODE_")
            && name != "CLAUDE_CODE_USE_BEDROCK"
            && name != "CLAUDE_CODE_USE_VERTEX")
}

fn command(exe: &Path) -> Command {
    let mut cmd = Command::new(exe);
    for (name, _) in std::env::vars_os() {
        if name.to_str().is_some_and(is_session_var) {
            cmd.env_remove(&name);
        }
    }
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    // Blotter is a windowed app: don't flash a console for the child.
    #[cfg(windows)]
    cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    cmd
}

/// What the settings page shows about Claude Code.
#[derive(Debug, Clone, Serialize, Default)]
pub struct Status {
    pub path: Option<String>,
    pub version: Option<String>,
    pub logged_in: Option<bool>,
    /// e.g. `claude.ai` (a subscription) or `api_key`.
    pub auth_method: Option<String>,
    pub error: Option<String>,
}

/// Checks that `claude` is installed and signed in.
pub async fn status(override_path: Option<&str>) -> Status {
    let Some(exe) = find(override_path) else {
        return Status {
            error: Some(if override_path.is_some_and(|p| !p.trim().is_empty()) {
                "There's no program at the path you set.".to_owned()
            } else {
                "Claude Code isn't installed, or Blotter can't find it.".to_owned()
            }),
            ..Status::default()
        };
    };
    let mut out = Status {
        path: Some(exe.display().to_string()),
        ..Status::default()
    };
    match probe(&exe, &["--version"]).await {
        Ok(v) => out.version = v.split_whitespace().next().map(str::to_owned),
        Err(e) => {
            out.error = Some(e);
            return out;
        }
    }
    match probe(&exe, &["auth", "status"]).await {
        Ok(text) => {
            let auth: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
            out.logged_in = auth["loggedIn"].as_bool();
            out.auth_method = auth["authMethod"].as_str().map(str::to_owned);
        }
        // `auth status` exits non-zero when signed out.
        Err(_) => out.logged_in = Some(false),
    }
    out
}

async fn probe(exe: &Path, args: &[&str]) -> Result<String, String> {
    let run = command(exe).args(args).output();
    let output = tokio::time::timeout(PROBE_TIMEOUT, run)
        .await
        .map_err(|_| "Claude Code didn't answer in time.".to_owned())?
        .map_err(|e| format!("Couldn't run Claude Code: {e}"))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_owned())
    }
}

/// The MCP config handed to Claude Code: Blotter's own program in server
/// mode, on this app's database, logging changes as the AI.
pub fn mcp_config(blotter_exe: &Path, data_dir: &Path, model: &str) -> Value {
    json!({
        "mcpServers": {
            SERVER: {
                "type": "stdio",
                "command": blotter_exe.display().to_string(),
                "args": [cli::MCP_SUBCOMMAND],
                "env": {
                    DATA_DIR_ENV: data_dir.display().to_string(),
                    MCP_ACTOR_ENV: actor(model),
                },
            }
        }
    })
}

/// How a turn's changes are logged.
pub fn actor(model: &str) -> String {
    format!("ai:claude-code/{model}")
}

/// How to start a Claude Code process.
pub struct SessionConfig<'a> {
    pub system_prompt: &'a str,
    pub model: &'a str,
    /// The thread's Claude session, to carry the conversation on.
    pub resume: Option<&'a str>,
    pub mcp_config: &'a Path,
}

/// The arguments for a process. Messages go on stdin as stream-json, not
/// here, so no quoting can mangle them.
pub fn args(turn: &SessionConfig<'_>) -> Vec<String> {
    let mut args: Vec<String> = [
        "-p",
        "--input-format",
        "stream-json",
        "--output-format",
        "stream-json",
        "--verbose",
        "--include-partial-messages",
        // No built-in tools at all: no shell, no files, no web.
        "--tools",
        "",
        "--strict-mcp-config",
        "--mcp-config",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    args.push(turn.mcp_config.display().to_string());
    args.extend(
        [
            "--allowedTools",
            ALLOW_RULE,
            "--permission-mode",
            "dontAsk",
            "--model",
        ]
        .into_iter()
        .map(str::to_owned),
    );
    args.push(turn.model.to_owned());
    args.push("--system-prompt".to_owned());
    args.push(turn.system_prompt.to_owned());
    if let Some(session) = turn.resume {
        args.push("--resume".to_owned());
        args.push(session.to_owned());
    }
    args
}

/// What a line of Claude Code's stream means for the chat.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// The session started; `tools_ok` is false if Blotter's server failed.
    Init {
        session_id: String,
        tools_ok: bool,
    },
    Text(String),
    ToolStart {
        id: String,
        name: String,
        input: Value,
    },
    ToolResult {
        id: String,
        is_error: bool,
        text: String,
    },
    Done {
        session_id: Option<String>,
        text: String,
        is_error: bool,
    },
}

/// Parses one stream-json line. Unknown or uninteresting lines (thinking,
/// rate-limit notices, partial tool arguments) yield nothing, as do events
/// from inside a sub-agent.
pub fn parse_line(line: &str) -> Vec<Event> {
    let Ok(m) = serde_json::from_str::<Value>(line) else {
        return Vec::new();
    };
    if !m["parent_tool_use_id"].is_null() {
        return Vec::new();
    }
    match (m["type"].as_str(), m["subtype"].as_str()) {
        (Some("system"), Some("init")) => {
            let tools_ok = m["mcp_servers"].as_array().is_some_and(|servers| {
                servers
                    .iter()
                    .any(|s| s["name"] == SERVER && s["status"] == "connected")
            });
            vec![Event::Init {
                session_id: m["session_id"].as_str().unwrap_or_default().to_owned(),
                tools_ok,
            }]
        }
        (Some("stream_event"), _) => {
            let e = &m["event"];
            match (e["type"].as_str(), e["delta"]["type"].as_str()) {
                (Some("content_block_delta"), Some("text_delta")) => e["delta"]["text"]
                    .as_str()
                    .map(|t| vec![Event::Text(t.to_owned())])
                    .unwrap_or_default(),
                _ => Vec::new(),
            }
        }
        (Some("assistant"), _) => m["message"]["content"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|b| b["type"] == "tool_use")
            .map(|b| Event::ToolStart {
                id: b["id"].as_str().unwrap_or_default().to_owned(),
                name: tool_name(b["name"].as_str().unwrap_or_default()),
                input: b["input"].clone(),
            })
            .collect(),
        (Some("user"), _) => m["message"]["content"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|b| b["type"] == "tool_result")
            .map(|b| Event::ToolResult {
                id: b["tool_use_id"].as_str().unwrap_or_default().to_owned(),
                is_error: b["is_error"].as_bool().unwrap_or(false),
                text: result_text(&b["content"]),
            })
            .collect(),
        (Some("result"), _) => vec![Event::Done {
            session_id: m["session_id"].as_str().map(str::to_owned),
            text: m["result"].as_str().unwrap_or_default().to_owned(),
            is_error: m["is_error"].as_bool().unwrap_or(false),
        }],
        _ => Vec::new(),
    }
}

/// `mcp__blotter__create_item` → `create_item`.
fn tool_name(raw: &str) -> String {
    raw.strip_prefix(TOOL_PREFIX).unwrap_or(raw).to_owned()
}

/// A tool result's content is a string or a list of text blocks.
fn result_text(content: &Value) -> String {
    match content {
        Value::String(s) => s.clone(),
        Value::Array(blocks) => blocks
            .iter()
            .filter_map(|b| b["text"].as_str())
            .collect::<Vec<_>>()
            .join("\n"),
        _ => String::new(),
    }
}

/// One user message as Claude Code's stream-json input reads it.
fn input_line(message: &str) -> String {
    let line = json!({ "type": "user", "message": { "role": "user", "content": message } });
    format!("{line}\n")
}

/// A running Claude Code process for one conversation. Starting `claude`
/// takes seconds (its own start-up and connecting to Blotter's server),
/// so the process stays up between messages: each message is a line on
/// its stdin, each reply streams back. Dropping the session kills it; if
/// Blotter itself dies, the closed stdin ends it too.
pub struct Session {
    child: Child,
    stdin: ChildStdin,
    events: mpsc::UnboundedReceiver<Event>,
    stderr: Arc<StdMutex<String>>,
    /// The model it was started with; another model needs a new process.
    pub model: String,
    pub last_used: std::time::Instant,
}

impl Session {
    /// Starts `claude` in the background. It boots while nothing is asked
    /// of it yet, so starting one ahead of time hides the wait; it makes no
    /// model call until a message arrives.
    pub fn start(exe: &Path, cwd: &Path, config: &SessionConfig<'_>) -> Result<Self, String> {
        std::fs::create_dir_all(cwd)
            .map_err(|e| format!("Couldn't prepare a folder for Claude Code: {e}"))?;
        let mut child = command(exe)
            .args(args(config))
            .current_dir(cwd)
            .stdin(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Couldn't start Claude Code: {e}"))?;
        let stdin = child
            .stdin
            .take()
            .ok_or("Couldn't talk to Claude Code (no input).")?;
        let stdout = child
            .stdout
            .take()
            .ok_or("Couldn't talk to Claude Code (no output).")?;

        let (tx, events) = mpsc::unbounded_channel();
        tokio::spawn(async move {
            let mut lines = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                for event in parse_line(&line) {
                    if tx.send(event).is_err() {
                        return;
                    }
                }
            }
            // The sender drops here, so a waiting turn learns the process ended.
        });
        let stderr = Arc::new(StdMutex::new(String::new()));
        if let Some(mut err) = child.stderr.take() {
            let sink = Arc::clone(&stderr);
            tokio::spawn(async move {
                let mut text = String::new();
                let _ = err.read_to_string(&mut text).await;
                if let Ok(mut s) = sink.lock() {
                    *s = text;
                }
            });
        }
        Ok(Self {
            child,
            stdin,
            events,
            stderr,
            model: config.model.to_owned(),
            last_used: std::time::Instant::now(),
        })
    }

    /// Whether the process is still running.
    pub fn is_alive(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }

    /// Sends one message and streams the reply to `on_event` until Claude
    /// Code reports the turn finished. Fails if the process ends first.
    pub async fn turn(
        &mut self,
        message: &str,
        mut on_event: impl FnMut(Event),
    ) -> Result<(), String> {
        self.last_used = std::time::Instant::now();
        self.stdin
            .write_all(input_line(message).as_bytes())
            .await
            .map_err(|_| self.ended())?;
        self.stdin.flush().await.map_err(|_| self.ended())?;
        while let Some(event) = self.events.recv().await {
            let done = matches!(event, Event::Done { .. });
            on_event(event);
            if done {
                self.last_used = std::time::Instant::now();
                return Ok(());
            }
        }
        // The output closed: give stderr a moment to arrive.
        let _ = tokio::time::timeout(Duration::from_secs(2), self.child.wait()).await;
        Err(self.ended())
    }

    fn ended(&self) -> String {
        let stderr = self
            .stderr
            .lock()
            .map(|s| s.trim().to_owned())
            .unwrap_or_default();
        if stderr.is_empty() {
            "Claude Code stopped without answering.".to_owned()
        } else {
            format!("Claude Code stopped: {}", last_lines(&stderr, 6))
        }
    }
}

fn last_lines(text: &str, n: usize) -> String {
    let lines: Vec<&str> = text.lines().collect();
    lines[lines.len().saturating_sub(n)..].join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    // Lines as Claude Code 2.1.292 writes them (trimmed to the fields read).
    const INIT: &str = r#"{"type":"system","subtype":"init","session_id":"s-1","tools":["mcp__blotter__list_inbox"],"mcp_servers":[{"name":"blotter","status":"connected","source":"dynamic"}],"model":"claude-sonnet-5-5"}"#;
    const INIT_FAILED: &str = r#"{"type":"system","subtype":"init","session_id":"s-1","mcp_servers":[{"name":"blotter","status":"failed","source":"dynamic"}]}"#;
    const TEXT: &str = r#"{"type":"stream_event","event":{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"You have "}},"session_id":"s-1","parent_tool_use_id":null}"#;
    const JSON_DELTA: &str = r#"{"type":"stream_event","event":{"type":"content_block_delta","index":0,"delta":{"type":"input_json_delta","partial_json":""}},"session_id":"s-1","parent_tool_use_id":null}"#;
    const TOOL_USE: &str = r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"thinking","thinking":""},{"type":"tool_use","id":"toolu_1","name":"mcp__blotter__list_inbox","input":{}}]},"parent_tool_use_id":null,"session_id":"s-1"}"#;
    const TOOL_RESULT: &str = r#"{"type":"user","message":{"role":"user","content":[{"tool_use_id":"toolu_1","type":"tool_result","content":[{"type":"text","text":"[]"}]}]},"parent_tool_use_id":null,"session_id":"s-1"}"#;
    const TOOL_ERROR: &str = r#"{"type":"user","message":{"role":"user","content":[{"tool_use_id":"toolu_2","type":"tool_result","content":"Invalid arguments for create_item:\n- \"title\" is a required property","is_error":true}]},"parent_tool_use_id":null}"#;
    const RESULT: &str = r#"{"type":"result","subtype":"success","is_error":false,"result":"You have 7 items in your inbox.","session_id":"s-1","num_turns":2}"#;
    const SUBAGENT: &str = r#"{"type":"stream_event","event":{"type":"content_block_delta","delta":{"type":"text_delta","text":"x"}},"parent_tool_use_id":"toolu_9"}"#;

    #[test]
    fn parses_a_recorded_turn() {
        assert_eq!(
            parse_line(INIT),
            [Event::Init {
                session_id: "s-1".into(),
                tools_ok: true
            }]
        );
        assert_eq!(parse_line(TEXT), [Event::Text("You have ".into())]);
        assert!(parse_line(JSON_DELTA).is_empty());
        assert_eq!(
            parse_line(TOOL_USE),
            [Event::ToolStart {
                id: "toolu_1".into(),
                name: "list_inbox".into(),
                input: json!({})
            }]
        );
        assert_eq!(
            parse_line(TOOL_RESULT),
            [Event::ToolResult {
                id: "toolu_1".into(),
                is_error: false,
                text: "[]".into()
            }]
        );
        assert_eq!(
            parse_line(RESULT),
            [Event::Done {
                session_id: Some("s-1".into()),
                text: "You have 7 items in your inbox.".into(),
                is_error: false
            }]
        );
    }

    #[test]
    fn flags_failed_tools_errors_and_ignores_noise() {
        assert!(matches!(
            parse_line(INIT_FAILED)[..],
            [Event::Init {
                tools_ok: false,
                ..
            }]
        ));
        match &parse_line(TOOL_ERROR)[..] {
            [Event::ToolResult { is_error, text, .. }] => {
                assert!(*is_error);
                assert!(text.starts_with("Invalid arguments"));
            }
            other => panic!("{other:?}"),
        }
        assert!(parse_line(SUBAGENT).is_empty());
        assert!(parse_line("not json").is_empty());
        assert!(parse_line(r#"{"type":"rate_limit_event"}"#).is_empty());
    }

    #[test]
    fn args_lock_claude_code_to_blotter_tools() {
        let cfg = PathBuf::from("mcp.json");
        let turn = SessionConfig {
            system_prompt: "You plan.",
            model: "sonnet",
            resume: Some("s-1"),
            mcp_config: &cfg,
        };
        let a = args(&turn);
        let after = |flag: &str| {
            let i = a.iter().position(|x| x == flag).unwrap();
            a[i + 1].clone()
        };
        assert_eq!(after("--tools"), "");
        assert_eq!(after("--allowedTools"), "mcp__blotter");
        assert_eq!(after("--permission-mode"), "dontAsk");
        assert_eq!(after("--mcp-config"), "mcp.json");
        assert_eq!(after("--resume"), "s-1");
        assert_eq!(after("--system-prompt"), "You plan.");
        assert!(a.contains(&"--strict-mcp-config".to_owned()));
        assert_eq!(
            after("--input-format"),
            "stream-json",
            "messages go on stdin"
        );
    }

    #[test]
    fn messages_are_stream_json_lines() {
        let line = input_line("Add \"milk\"\nplease");
        assert!(line.ends_with('\n') && line.matches('\n').count() == 1);
        let v: Value = serde_json::from_str(line.trim()).unwrap();
        assert_eq!(v["type"], "user");
        assert_eq!(v["message"]["content"], "Add \"milk\"\nplease");
    }

    #[test]
    fn inherited_session_variables_are_dropped() {
        for name in [
            "CLAUDECODE",
            "CLAUDE_CODE_SESSION_ID",
            "CLAUDE_CODE_ENTRYPOINT",
            "CLAUDE_PID",
        ] {
            assert!(is_session_var(name), "{name}");
        }
        for name in [
            "CLAUDE_CONFIG_DIR",
            "ANTHROPIC_API_KEY",
            "CLAUDE_CODE_USE_BEDROCK",
            "PATH",
        ] {
            assert!(!is_session_var(name), "{name}");
        }
    }

    #[test]
    fn mcp_config_runs_blotter_as_the_ai() {
        let cfg = mcp_config(Path::new("/opt/blotter"), Path::new("/data"), "opus");
        let server = &cfg["mcpServers"]["blotter"];
        assert_eq!(server["args"], json!(["mcp"]));
        assert_eq!(server["env"]["BLOTTER_DATA_DIR"], "/data");
        assert_eq!(server["env"]["BLOTTER_MCP_ACTOR"], "ai:claude-code/opus");
        assert!(actor("opus").parse::<blotter_core::Actor>().is_ok());
    }
}

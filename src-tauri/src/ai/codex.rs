//! The Codex route: Blotter drives the user's installed, signed-in OpenAI
//! Codex CLI headless, on their ChatGPT plan, with Blotter's own MCP server
//! as its only tools — the same shape as the Claude Code route (D29).
//!
//! Each chat message is one `codex exec --json` run; a thread's later
//! messages resume its Codex thread (`codex exec resume <id>`), so the
//! conversation carries on. Codex's own tools are switched off (shell,
//! exec, apps, plugins, browser, computer use, sub-agents, images), it runs
//! read-only in its own folder under the app data, and the user's Codex
//! config and rules aren't loaded (sign-in still is). The MCP server logs
//! its changes as `ai:codex/<model>`.

use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::Serialize;
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};

use super::claude_code::{Event, command};
use crate::{DATA_DIR_ENV, MCP_ACTOR_ENV, cli};

/// The MCP server's name, as Codex reports its tool calls.
const SERVER: &str = "blotter";
/// `--version` and `login status` should answer quickly.
const PROBE_TIMEOUT: Duration = Duration::from_secs(20);
/// Codex's own tools, all off: Blotter's MCP tools are the only ones.
const DISABLED_FEATURES: &[&str] = &[
    "shell_tool",
    "unified_exec",
    "apps",
    "plugins",
    "browser_use",
    "computer_use",
    "multi_agent",
    "image_generation",
    "view_image",
];

#[cfg(windows)]
const EXE: &str = "codex.exe";
#[cfg(not(windows))]
const EXE: &str = "codex";

/// Finds `codex`: the path the user set, then `PATH`, then where npm puts
/// the real program (on Windows `PATH` has only npm's `codex.cmd` shim,
/// which runs the bundled `codex.exe` through Node).
pub fn find(override_path: Option<&str>) -> Option<PathBuf> {
    let set = override_path.map(str::trim).filter(|p| !p.is_empty());
    if crate::flatpak::app_id().is_some() {
        return Some(PathBuf::from(set.unwrap_or(EXE)));
    }
    if let Some(p) = set {
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
        .or_else(npm_binary)
}

fn install_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(home) = dirs::home_dir() {
        dirs.push(home.join(".local").join("bin"));
        dirs.push(home.join(".codex").join("bin"));
    }
    #[cfg(not(windows))]
    {
        dirs.push(PathBuf::from("/usr/local/bin"));
        dirs.push(PathBuf::from("/opt/homebrew/bin"));
    }
    dirs
}

/// The program inside npm's global `@openai/codex` package:
/// `…/@openai/codex/node_modules/@openai/codex-<platform>/vendor/<target>/bin/codex(.exe)`.
fn npm_binary() -> Option<PathBuf> {
    let mut roots = Vec::new();
    #[cfg(windows)]
    if let Some(appdata) = dirs::data_dir() {
        roots.push(appdata.join("npm").join("node_modules"));
    }
    #[cfg(not(windows))]
    {
        if let Some(home) = dirs::home_dir() {
            roots.push(home.join(".npm-global").join("lib").join("node_modules"));
        }
        roots.push(PathBuf::from("/usr/local/lib/node_modules"));
        roots.push(PathBuf::from("/usr/lib/node_modules"));
    }
    for root in roots {
        let platforms = root
            .join("@openai")
            .join("codex")
            .join("node_modules")
            .join("@openai");
        let Ok(entries) = std::fs::read_dir(&platforms) else {
            continue;
        };
        for entry in entries.flatten() {
            if !entry.file_name().to_string_lossy().starts_with("codex-") {
                continue;
            }
            let Ok(targets) = std::fs::read_dir(entry.path().join("vendor")) else {
                continue;
            };
            for target in targets.flatten() {
                let exe = target.path().join("bin").join(EXE);
                if exe.is_file() {
                    return Some(exe);
                }
            }
        }
    }
    None
}

/// What the settings page shows about Codex.
#[derive(Debug, Clone, Serialize, Default)]
pub struct Status {
    pub path: Option<String>,
    pub version: Option<String>,
    pub logged_in: Option<bool>,
    /// `chatgpt` (a ChatGPT plan) or `api_key`.
    pub auth_method: Option<String>,
    pub error: Option<String>,
}

/// Checks that `codex` is installed and signed in.
pub async fn status(override_path: Option<&str>) -> Status {
    let Some(exe) = find(override_path) else {
        return Status {
            error: Some(if override_path.is_some_and(|p| !p.trim().is_empty()) {
                "There's no program at the path you set.".to_owned()
            } else {
                "Codex isn't installed, or Blotter can't find it.".to_owned()
            }),
            ..Status::default()
        };
    };
    let mut out = Status {
        path: Some(exe.display().to_string()),
        ..Status::default()
    };
    match probe(&exe, &["--version"]).await {
        // "codex-cli 0.163.0"
        Ok(v) => out.version = v.split_whitespace().last().map(str::to_owned),
        Err(e) => {
            out.error = Some(e);
            return out;
        }
    }
    match probe(&exe, &["login", "status"]).await {
        Ok(text) => {
            let (logged_in, method) = read_login(&text);
            out.logged_in = Some(logged_in);
            out.auth_method = method;
        }
        // `login status` exits non-zero when signed out.
        Err(_) => out.logged_in = Some(false),
    }
    out
}

/// `codex login status` → (signed in, how). It prints e.g. "Logged in using ChatGPT".
fn read_login(text: &str) -> (bool, Option<String>) {
    let lower = text.to_lowercase();
    if !lower.contains("logged in") || lower.contains("not logged in") {
        return (false, None);
    }
    let method = if lower.contains("chatgpt") {
        Some("chatgpt".to_owned())
    } else if lower.contains("api key") {
        Some("api_key".to_owned())
    } else {
        None
    };
    (true, method)
}

async fn probe(exe: &Path, args: &[&str]) -> Result<String, String> {
    let run = command(exe).args(args).output();
    let output = tokio::time::timeout(PROBE_TIMEOUT, run)
        .await
        .map_err(|_| "Codex didn't answer in time.".to_owned())?
        .map_err(|e| format!("Couldn't run Codex: {e}"))?;
    let text = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if output.status.success() {
        // Some versions print the login state on stderr.
        if text.is_empty() {
            return Ok(String::from_utf8_lossy(&output.stderr).trim().to_owned());
        }
        Ok(text)
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_owned())
    }
}

/// How a turn's changes are logged; `default` when Codex picks the model.
pub fn actor(model: &str) -> String {
    format!("ai:codex/{}", model_label(model))
}

pub fn model_label(model: &str) -> &str {
    let m = model.trim();
    if m.is_empty() { "default" } else { m }
}

/// A TOML string for a `-c key=value` override. JSON's escapes (`\"`,
/// `\\`, `\n`, `\uXXXX`) are all valid in a TOML basic string.
fn toml_str(s: &str) -> String {
    serde_json::to_string(s).unwrap_or_else(|_| "\"\"".to_owned())
}

/// How to run one Codex turn.
pub struct TurnConfig<'a> {
    pub instructions: &'a str,
    /// Empty: Codex's own default model.
    pub model: &'a str,
    /// The thread's Codex thread, to carry the conversation on.
    pub resume: Option<&'a str>,
    /// How Codex starts Blotter's MCP server (`flatpak::self_command`).
    pub blotter: &'a (String, Vec<String>),
    pub data_dir: &'a Path,
}

/// The arguments for a turn. The message goes on stdin, not here, so no
/// quoting can mangle it.
pub fn args(turn: &TurnConfig<'_>) -> Vec<String> {
    let (blotter, blotter_args) = turn.blotter;
    let mut server_args = blotter_args.clone();
    server_args.push(cli::MCP_SUBCOMMAND.to_owned());
    let server_args: Vec<String> = server_args.iter().map(|a| toml_str(a)).collect();
    let env = format!(
        "{{{DATA_DIR_ENV}={}, {MCP_ACTOR_ENV}={}}}",
        toml_str(&turn.data_dir.display().to_string()),
        toml_str(&actor(turn.model)),
    );

    let mut args: Vec<String> = [
        "exec",
        "--json",
        "--skip-git-repo-check",
        "--ignore-user-config",
        "--ignore-rules",
        "--sandbox",
        "read-only",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    for feature in DISABLED_FEATURES {
        args.push("--disable".to_owned());
        args.push((*feature).to_owned());
    }
    for (key, value) in [
        ("approval_policy", toml_str("never")),
        ("web_search", toml_str("disabled")),
        ("developer_instructions", toml_str(turn.instructions)),
        (&format!("mcp_servers.{SERVER}.command"), toml_str(blotter)),
        (
            &format!("mcp_servers.{SERVER}.args"),
            format!("[{}]", server_args.join(", ")),
        ),
        (&format!("mcp_servers.{SERVER}.env"), env),
    ] {
        args.push("-c".to_owned());
        args.push(format!("{key}={value}"));
    }
    if !turn.model.trim().is_empty() {
        args.push("--model".to_owned());
        args.push(turn.model.trim().to_owned());
    }
    if let Some(thread) = turn.resume {
        args.push("resume".to_owned());
        args.push(thread.to_owned());
    }
    // The message is read from stdin.
    args.push("-".to_owned());
    args
}

/// The text of an MCP tool result: its text blocks, joined.
fn result_text(result: &Value) -> String {
    result["content"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|b| b["text"].as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

/// What a line of `codex exec --json` means for the chat. Codex sends each
/// message whole rather than as it is typed; a turn may say something
/// before its tool calls and again after, and the reply is all of it.
pub fn parse_line(line: &str) -> Vec<Event> {
    let Ok(m) = serde_json::from_str::<Value>(line) else {
        return Vec::new();
    };
    match m["type"].as_str() {
        Some("thread.started") => vec![Event::Init {
            session_id: m["thread_id"].as_str().unwrap_or_default().to_owned(),
            tools_ok: true,
        }],
        Some(kind @ ("item.started" | "item.completed")) => {
            let item = &m["item"];
            let id = item["id"].as_str().unwrap_or_default().to_owned();
            match (item["type"].as_str(), kind) {
                (Some("agent_message"), "item.completed") => item["text"]
                    .as_str()
                    .map(|t| vec![Event::Text(t.to_owned())])
                    .unwrap_or_default(),
                (Some("mcp_tool_call"), "item.started") => vec![Event::ToolStart {
                    id,
                    name: item["tool"].as_str().unwrap_or_default().to_owned(),
                    input: item["arguments"].clone(),
                }],
                (Some("mcp_tool_call"), "item.completed") => {
                    let failed = !item["error"].is_null() || item["status"] == "failed";
                    let text = if failed {
                        item["error"]["message"]
                            .as_str()
                            .map(str::to_owned)
                            .unwrap_or_else(|| result_text(&item["result"]))
                    } else {
                        result_text(&item["result"])
                    };
                    vec![Event::ToolResult {
                        id,
                        is_error: failed,
                        text,
                    }]
                }
                _ => Vec::new(),
            }
        }
        Some("turn.completed") => vec![Event::Done {
            session_id: None,
            text: String::new(),
            is_error: false,
        }],
        Some("turn.failed") => vec![Event::Done {
            session_id: None,
            text: m["error"]["message"]
                .as_str()
                .unwrap_or("Codex couldn't answer.")
                .to_owned(),
            is_error: true,
        }],
        Some("error") => vec![Event::Done {
            session_id: None,
            text: m["message"]
                .as_str()
                .unwrap_or("Codex couldn't answer.")
                .to_owned(),
            is_error: true,
        }],
        _ => Vec::new(),
    }
}

/// Runs one turn: starts `codex exec` in `cwd`, writes `message` to its
/// stdin, and streams events to `on_event` until it exits. Dropping the
/// future (a cancelled turn) kills the process.
pub async fn run_turn(
    exe: &Path,
    cwd: &Path,
    config: &TurnConfig<'_>,
    message: &str,
    mut on_event: impl FnMut(Event),
) -> Result<(), String> {
    use std::process::Stdio;
    use tokio::io::AsyncWriteExt;

    std::fs::create_dir_all(cwd)
        .map_err(|e| format!("Couldn't prepare a folder for Codex: {e}"))?;
    let mut child = command(exe)
        .args(args(config))
        .current_dir(cwd)
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Couldn't start Codex: {e}"))?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(message.as_bytes())
            .await
            .map_err(|e| format!("Couldn't talk to Codex: {e}"))?;
        // Closing stdin tells Codex the message is complete.
        drop(stdin);
    }
    let mut stderr_task = child.stderr.take().map(|mut err| {
        tokio::spawn(async move {
            let mut text = String::new();
            let _ = err.read_to_string(&mut text).await;
            text
        })
    });
    let stdout = child
        .stdout
        .take()
        .ok_or("Couldn't talk to Codex (no output).")?;
    let mut lines = BufReader::new(stdout).lines();
    let mut finished = false;
    while let Ok(Some(line)) = lines.next_line().await {
        for event in parse_line(&line) {
            finished |= matches!(event, Event::Done { .. });
            on_event(event);
        }
    }
    let _ = child.wait().await;
    if finished {
        return Ok(());
    }
    let stderr = match stderr_task.take() {
        Some(t) => t.await.unwrap_or_default(),
        None => String::new(),
    };
    let stderr = stderr
        .lines()
        .filter(|l| !l.starts_with("Reading additional input from stdin"))
        .collect::<Vec<_>>();
    Err(if stderr.is_empty() {
        "Codex stopped without answering.".to_owned()
    } else {
        format!(
            "Codex stopped: {}",
            stderr[stderr.len().saturating_sub(6)..].join("\n")
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // Lines as Codex CLI 0.163.0 writes them (trimmed).
    const STARTED: &str =
        r#"{"type":"thread.started","thread_id":"01a15743-cda0-7d71-84ea-7ec22f66ff8e"}"#;
    const SAID: &str = r#"{"type":"item.completed","item":{"id":"item_0","type":"agent_message","text":"I'll check the Inbox."}}"#;
    const CALL: &str = r#"{"type":"item.started","item":{"id":"item_1","type":"mcp_tool_call","server":"blotter","tool":"list_inbox","arguments":{},"result":null,"error":null,"status":"in_progress"}}"#;
    const RESULT: &str = r#"{"type":"item.completed","item":{"id":"item_1","type":"mcp_tool_call","server":"blotter","tool":"list_inbox","arguments":{},"result":{"content":[{"type":"text","text":"[]"}]},"error":null,"status":"completed"}}"#;
    const FAILED_CALL: &str = r#"{"type":"item.completed","item":{"id":"item_2","type":"mcp_tool_call","server":"blotter","tool":"create_item","arguments":{},"result":null,"error":{"message":"title is a required property"},"status":"failed"}}"#;
    const DONE: &str =
        r#"{"type":"turn.completed","usage":{"input_tokens":79698,"output_tokens":146}}"#;
    const FAILED: &str =
        r#"{"type":"turn.failed","error":{"message":"You've hit your usage limit."}}"#;

    #[test]
    fn parses_a_recorded_turn() {
        let events: Vec<Event> = [STARTED, SAID, CALL, RESULT, FAILED_CALL, DONE]
            .iter()
            .flat_map(|l| parse_line(l))
            .collect();
        assert_eq!(
            events,
            [
                Event::Init {
                    session_id: "01a15743-cda0-7d71-84ea-7ec22f66ff8e".into(),
                    tools_ok: true
                },
                Event::Text("I'll check the Inbox.".into()),
                Event::ToolStart {
                    id: "item_1".into(),
                    name: "list_inbox".into(),
                    input: json!({})
                },
                Event::ToolResult {
                    id: "item_1".into(),
                    is_error: false,
                    text: "[]".into()
                },
                Event::ToolResult {
                    id: "item_2".into(),
                    is_error: true,
                    text: "title is a required property".into()
                },
                Event::Done {
                    session_id: None,
                    text: String::new(),
                    is_error: false
                },
            ]
        );
        assert_eq!(
            parse_line(FAILED),
            [Event::Done {
                session_id: None,
                text: "You've hit your usage limit.".into(),
                is_error: true
            }]
        );
        assert!(parse_line("not json").is_empty());
    }

    #[test]
    fn args_lock_codex_to_blotter_tools() {
        let blotter = ("C:\\Blotter\\blotter.exe".to_owned(), Vec::new());
        let a = args(&TurnConfig {
            instructions: "Plan \"well\".\nThanks.",
            model: "",
            resume: Some("t-1"),
            blotter: &blotter,
            data_dir: Path::new("C:\\data"),
        });
        let joined = a.join(" ");
        for f in DISABLED_FEATURES {
            assert!(joined.contains(&format!("--disable {f}")), "{f}");
        }
        assert!(joined.contains("--sandbox read-only"));
        assert!(joined.contains("--ignore-user-config"));
        assert!(a.contains(&r#"developer_instructions="Plan \"well\".\nThanks.""#.to_owned()));
        assert!(
            a.contains(&r#"mcp_servers.blotter.command="C:\\Blotter\\blotter.exe""#.to_owned())
        );
        assert!(a.contains(&r#"mcp_servers.blotter.args=["mcp"]"#.to_owned()));
        assert!(a.contains(
            &r#"mcp_servers.blotter.env={BLOTTER_DATA_DIR="C:\\data", BLOTTER_MCP_ACTOR="ai:codex/default"}"#
                .to_owned()
        ));
        assert!(
            !a.contains(&"--model".to_owned()),
            "Codex picks its default"
        );
        // Resume goes last, before the stdin marker.
        assert_eq!(a[a.len() - 3..], ["resume", "t-1", "-"]);
    }

    #[test]
    fn reads_the_login_state() {
        assert_eq!(
            read_login("Logged in using ChatGPT"),
            (true, Some("chatgpt".into()))
        );
        assert_eq!(
            read_login("Logged in using an API key - sk-***"),
            (true, Some("api_key".into()))
        );
        assert_eq!(read_login("Not logged in"), (false, None));
    }
}

/// A real turn against the installed Codex, on a scratch data folder:
/// `BLOTTER_LIVE_CODEX=<data dir> cargo test -p blotter live_codex -- --ignored`.
/// Uses the signed-in plan, so it never runs by default.
#[cfg(test)]
mod live {
    use super::*;

    #[tokio::test]
    #[ignore = "runs a real Codex turn"]
    async fn live_codex_turn() {
        let Ok(data) = std::env::var("BLOTTER_LIVE_CODEX") else {
            return;
        };
        let data = PathBuf::from(data);
        let exe = find(None).expect("codex installed");
        let blotter = (
            std::env::current_exe()
                .unwrap()
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .join(if cfg!(windows) {
                    "blotter.exe"
                } else {
                    "blotter"
                })
                .display()
                .to_string(),
            Vec::new(),
        );
        let config = TurnConfig {
            instructions: "You are Blotter's planning assistant. End every reply with the word DONE.",
            model: "",
            resume: None,
            blotter: &blotter,
            data_dir: &data,
        };
        let mut events = Vec::new();
        run_turn(
            &exe,
            &data.join("codex"),
            &config,
            "How many tasks are in my Inbox? Use the blotter tools. One sentence.",
            |e| events.push(e),
        )
        .await
        .unwrap();
        for e in &events {
            println!("{e:?}");
        }
        assert!(
            events
                .iter()
                .any(|e| matches!(e, Event::ToolStart { name, .. } if name == "list_inbox"))
        );
        assert!(matches!(
            events.last(),
            Some(Event::Done {
                is_error: false,
                ..
            })
        ));
    }
}

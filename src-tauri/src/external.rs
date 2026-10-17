//! Changes made outside the app, by `blotter mcp` (roadmap §4.8). The MCP
//! server writes to the same database from its own process, so the app
//! polls SQLite's `data_version`, which moves whenever another connection
//! commits, and then refreshes every window and wakes the reminder
//! engine, as for its own changes. Also the setup the settings page shows
//! for connecting an MCP client.

use std::thread;
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::commands::{CHANGED_EVENT, ChangedPayload, CommandError};
use crate::{AppState, DATA_DIR_ENV, cli, flatpak};

/// How often to look for outside writes.
const POLL: Duration = Duration::from_secs(1);

pub fn start(app: AppHandle) {
    let spawned = thread::Builder::new()
        .name("external-changes".into())
        .spawn(move || {
            let mut seen = data_version(&app);
            loop {
                thread::sleep(POLL);
                let now = data_version(&app);
                if now.is_some() && now != seen {
                    let payload = ChangedPayload {
                        batch_id: String::new(),
                        ops: vec!["external".into()],
                    };
                    let _ = app.emit(CHANGED_EVENT, payload);
                }
                seen = now.or(seen);
            }
        });
    if let Err(err) = spawned {
        eprintln!("blotter: could not watch for outside changes: {err}");
    }
}

fn data_version(app: &AppHandle) -> Option<i64> {
    let state = app.try_state::<AppState>()?;
    let planner = state.planner.lock().ok()?;
    planner.db().data_version().ok()
}

/// How an MCP client should launch Blotter's server.
#[derive(Debug, Serialize)]
pub struct McpSetup {
    pub command: String,
    pub args: Vec<String>,
    /// Set when this app runs on a non-default database, so the server
    /// opens the same one.
    pub data_dir: Option<String>,
    pub data_dir_env: &'static str,
}

#[tauri::command]
pub fn mcp_setup() -> Result<McpSetup, CommandError> {
    let (command, mut args) = flatpak::self_command()
        .map_err(|err| CommandError::internal(format!("couldn't find Blotter's program: {err}")))?;
    args.push(cli::MCP_SUBCOMMAND.to_owned());
    Ok(McpSetup {
        command,
        args,
        data_dir: std::env::var_os(DATA_DIR_ENV)
            .filter(|d| !d.is_empty())
            .map(|d| d.to_string_lossy().into_owned()),
        data_dir_env: DATA_DIR_ENV,
    })
}

//! The Blotter desktop app: windows, tray, notifications, keychain and AI
//! HTTP on top of `blotter-core`.

mod ai;
mod autostart;
mod backups;
mod capture;
mod cli;
mod commands;
mod external;
mod reminders;
mod rollover;
mod shell;

use std::path::PathBuf;
use std::sync::Mutex;

use blotter_core::{Database, Planner};
use chrono_tz::Tz;
use tauri::Manager;

/// Process-wide state shared by every command.
pub(crate) struct AppState {
    pub planner: Mutex<Planner>,
    pub db_path: PathBuf,
}

/// The OS timezone as an IANA zone, falling back to UTC if it can't be
/// read or isn't in the tz database.
fn system_timezone() -> Tz {
    iana_time_zone::get_timezone()
        .ok()
        .and_then(|name| name.parse::<Tz>().ok())
        .unwrap_or(chrono_tz::UTC)
}

/// Overrides the data directory, e.g. to try things on a scratch database.
pub(crate) const DATA_DIR_ENV: &str = "BLOTTER_DATA_DIR";

/// The app identifier from tauri.conf.json (checked by a test).
const APP_ID: &str = "io.github.lasr0.Blotter";

/// The database file: in the OS app-data directory, e.g.
/// `%APPDATA%\io.github.lasr0.Blotter\blotter.db` (the same place as
/// Tauri's `app_data_dir`), or in `$BLOTTER_DATA_DIR` when that is set.
/// The app and `blotter mcp` both use this, so they share one database.
fn db_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let data_dir = match std::env::var_os(DATA_DIR_ENV) {
        Some(dir) if !dir.is_empty() => PathBuf::from(dir),
        _ => dirs::data_dir()
            .ok_or("couldn't find the app-data directory")?
            .join(APP_ID),
    };
    Ok(data_dir.join(blotter_core::db::DB_FILE_NAME))
}

/// Opens (creating and migrating) the database.
fn open_state() -> Result<AppState, Box<dyn std::error::Error>> {
    let db_path = db_path()?;
    let db = Database::open(&db_path)?;
    Ok(AppState {
        planner: Mutex::new(Planner::new(db, system_timezone())),
        db_path,
    })
}

/// Set by Blotter's own chat when it runs Claude Code, so the server logs
/// changes as that AI (`ai:claude-code/<model>`) rather than `mcp`.
pub(crate) const MCP_ACTOR_ENV: &str = "BLOTTER_MCP_ACTOR";

/// `blotter mcp`: serves the MCP server on stdio instead of starting the
/// app, then exits. It never touches the single-instance lock, so it runs
/// beside an open Blotter.
fn run_mcp() -> ! {
    let actor = std::env::var(MCP_ACTOR_ENV)
        .ok()
        .and_then(|a| a.parse().ok())
        .unwrap_or(blotter_core::Actor::Mcp);
    let result = db_path().and_then(|path| blotter_mcp::run(&path, system_timezone(), actor));
    match result {
        Ok(()) => std::process::exit(0),
        Err(err) => {
            eprintln!("blotter mcp: {err}");
            std::process::exit(1);
        }
    }
}

/// Applies Linux-specific environment workarounds. Must run before the
/// webview is created, i.e. before the Tauri builder starts.
fn apply_platform_workarounds() {
    #[cfg(target_os = "linux")]
    {
        // WebKitGTK on NVIDIA + Wayland can render a blank or flickering
        // window with the DMA-BUF renderer. Respect a user override.
        const KEY: &str = "WEBKIT_DISABLE_DMABUF_RENDERER";
        if std::env::var_os(KEY).is_none() {
            // SAFETY: called at the top of `run`, before Tauri or any other
            // thread is started, so no other thread can read the environment
            // concurrently.
            #[allow(unsafe_code)]
            unsafe {
                std::env::set_var(KEY, "1")
            };
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    if cli::is_mcp(std::env::args()) {
        run_mcp();
    }
    apply_platform_workarounds();
    let launch = cli::parse(std::env::args());

    let result = tauri::Builder::default()
        // Must be the first plugin: a second launch hands its arguments to
        // this instance and exits before anything else initialises.
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            shell::handle_launch(app, &cli::parse(argv));
        }))
        .setup(move |app| {
            let state = open_state()?;
            app.manage(state);
            external::start(app.handle().clone());
            shell::build_tray(app.handle())?;
            let shortcut = capture::register_shortcut(app.handle());
            app.manage(shortcut);
            capture::prepare(app.handle());
            app.manage(rollover::RolloverNotice::default());
            app.manage(ai::chat::ChatRuns::default());
            rollover::start(app.handle().clone());
            reminders::start(app.handle().clone());
            // The main window starts hidden (tauri.conf.json) so `--hidden`
            // launches never flash it.
            shell::handle_launch(app.handle(), &launch);
            Ok(())
        })
        .plugin(tauri_plugin_notification::init())
        .plugin(autostart::plugin())
        .plugin(tauri_plugin_dialog::init())
        .on_window_event(shell::on_window_event)
        .invoke_handler(tauri::generate_handler![
            commands::app_version,
            commands::quit_app,
            commands::db_status,
            commands::list_ops,
            commands::run_op,
            commands::run_batch,
            external::mcp_setup,
            commands::get_setting,
            commands::set_setting,
            capture::show_capture,
            capture::hide_capture,
            capture::fit_capture,
            capture::capture_shortcut_status,
            rollover::take_rollover_notice,
            reminders::take_reminder_notices,
            reminders::send_test_notification,
            autostart::get_autostart,
            autostart::set_autostart,
            backups::list_snapshots,
            backups::create_snapshot,
            backups::restore_snapshot,
            backups::choose_snapshot_dir,
            backups::reset_snapshot_dir,
            backups::export_json,
            backups::pick_import,
            backups::apply_import,
            ai::commands::list_providers,
            ai::commands::save_provider,
            ai::commands::delete_provider,
            ai::commands::set_default_provider,
            ai::commands::fetch_models,
            ai::commands::test_provider,
            ai::chat::get_ai_settings,
            ai::chat::set_ai_settings,
            ai::chat::claude_code_status,
            ai::chat::chat_threads,
            ai::chat::chat_messages,
            ai::chat::chat_delete_thread,
            ai::chat::chat_cancel,
            ai::chat::chat_send,
        ])
        .run(tauri::generate_context!());

    if let Err(err) = result {
        eprintln!("Blotter failed to start: {err}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn app_id_matches_tauri_config() {
        let conf: serde_json::Value =
            serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
        assert_eq!(conf["identifier"], super::APP_ID);
    }
}

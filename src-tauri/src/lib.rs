//! The Blotter desktop app: windows, tray, notifications, keychain and AI
//! HTTP on top of `blotter-core`.

mod commands;

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

/// Opens (creating and migrating) the database in the OS app-data
/// directory, e.g. `%APPDATA%\io.github.lasr0.Blotter\blotter.db`.
fn open_state(app: &tauri::App) -> Result<AppState, Box<dyn std::error::Error>> {
    let db_path = app
        .path()
        .app_data_dir()?
        .join(blotter_core::db::DB_FILE_NAME);
    let db = Database::open(&db_path)?;
    Ok(AppState {
        planner: Mutex::new(Planner::new(db, system_timezone())),
        db_path,
    })
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
    apply_platform_workarounds();

    let result = tauri::Builder::default()
        .setup(|app| {
            let state = open_state(app)?;
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app_version,
            commands::db_status,
            commands::list_ops,
            commands::run_op,
        ])
        .run(tauri::generate_context!());

    if let Err(err) = result {
        eprintln!("Blotter failed to start: {err}");
        std::process::exit(1);
    }
}

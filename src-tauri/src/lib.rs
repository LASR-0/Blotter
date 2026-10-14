//! The Blotter desktop app: windows, tray, notifications, keychain and AI
//! HTTP on top of `blotter-core`.

use std::path::PathBuf;
use std::sync::Mutex;

use blotter_core::Database;
use serde::Serialize;
use tauri::{Manager, State};

/// Process-wide state shared by every command.
struct AppState {
    db: Mutex<Database>,
    db_path: PathBuf,
}

#[derive(Serialize)]
struct DbStatus {
    path: String,
    journal_mode: String,
    schema_version: i64,
}

#[tauri::command]
fn app_version() -> &'static str {
    blotter_core::VERSION
}

#[tauri::command]
fn db_status(state: State<'_, AppState>) -> Result<DbStatus, String> {
    let db = state
        .db
        .lock()
        .map_err(|_| "database lock poisoned".to_string())?;
    Ok(DbStatus {
        path: state.db_path.display().to_string(),
        journal_mode: db.journal_mode().map_err(|e| e.to_string())?,
        schema_version: db.schema_version().map_err(|e| e.to_string())?,
    })
}

/// Opens (creating and migrating) the database in the OS app-data
/// directory, e.g. `%APPDATA%\io.github.lasr0.Blotter\blotter.db`.
fn open_database(app: &tauri::App) -> Result<AppState, Box<dyn std::error::Error>> {
    let db_path = app
        .path()
        .app_data_dir()?
        .join(blotter_core::db::DB_FILE_NAME);
    let db = Database::open(&db_path)?;
    Ok(AppState {
        db: Mutex::new(db),
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
            let state = open_database(app)?;
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![app_version, db_status])
        .run(tauri::generate_context!());

    if let Err(err) = result {
        eprintln!("Blotter failed to start: {err}");
        std::process::exit(1);
    }
}

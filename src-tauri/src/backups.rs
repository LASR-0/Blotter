//! Settings → Backups (roadmap §4.9): snapshots, restore, and JSON export
//! and import. File and folder pickers are native dialogs; they block, so
//! these commands are async (off the main thread).

use std::path::PathBuf;

use blotter_core::export::{self, Summary};
use blotter_core::id::new_id;
use blotter_core::snapshots::{self, Reason, Snapshot};
use chrono::Utc;
use serde::Serialize;
use serde_json::json;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_dialog::DialogExt;

use crate::AppState;
use crate::commands::{CHANGED_EVENT, CommandError};

fn lock_err() -> CommandError {
    CommandError::internal("planner state is unavailable after an earlier crash")
}

/// Tells every window the data changed wholesale (after a restore or import).
fn replaced(app: &AppHandle, what: &str) {
    let _ = app.emit(
        CHANGED_EVENT,
        json!({ "batch_id": new_id(), "ops": [what] }),
    );
}

#[derive(Serialize)]
pub struct BackupState {
    /// Where snapshots are written.
    pub dir: String,
    /// Whether that's the default folder beside the database.
    pub default_dir: bool,
    pub snapshots: Vec<Snapshot>,
}

#[tauri::command]
pub fn list_snapshots(state: State<'_, AppState>) -> Result<BackupState, CommandError> {
    let planner = state.planner.lock().map_err(|_| lock_err())?;
    let conn = planner.db().conn();
    let dir = snapshots::dir(conn, &state.db_path)?;
    let default_dir = state
        .db_path
        .parent()
        .is_some_and(|p| p.join("snapshots") == dir);
    Ok(BackupState {
        dir: dir.to_string_lossy().into_owned(),
        default_dir,
        snapshots: snapshots::list(conn)?,
    })
}

#[tauri::command]
pub fn create_snapshot(
    state: State<'_, AppState>,
    note: Option<String>,
) -> Result<Snapshot, CommandError> {
    let planner = state.planner.lock().map_err(|_| lock_err())?;
    let conn = planner.db().conn();
    let dir = snapshots::dir(conn, &state.db_path)?;
    Ok(snapshots::create(
        conn,
        &dir,
        Reason::Manual,
        note,
        Utc::now(),
    )?)
}

/// Restores a snapshot (after snapshotting the current state). The UI
/// reloads afterwards, since everything may have changed.
#[tauri::command]
pub fn restore_snapshot(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<Snapshot, CommandError> {
    let before = {
        let mut planner = state.planner.lock().map_err(|_| lock_err())?;
        snapshots::restore(planner.db_mut(), &state.db_path, &id, Utc::now())?
    };
    replaced(&app, "restore");
    Ok(before)
}

/// Picks a new snapshot folder and moves the snapshots there. `None` if
/// the picker was cancelled.
#[tauri::command]
pub async fn choose_snapshot_dir(app: AppHandle) -> Result<Option<String>, CommandError> {
    let Some(picked) = app
        .dialog()
        .file()
        .set_title("Choose a folder for Blotter's snapshots")
        .blocking_pick_folder()
    else {
        return Ok(None);
    };
    let dir = picked
        .into_path()
        .map_err(|e| CommandError::internal(format!("couldn't use that folder: {e}")))?;
    move_snapshots(&app, Some(dir)).map(Some)
}

/// Moves the snapshots back to the default folder beside the database.
#[tauri::command]
pub fn reset_snapshot_dir(app: AppHandle) -> Result<String, CommandError> {
    move_snapshots(&app, None)
}

fn move_snapshots(app: &AppHandle, dir: Option<PathBuf>) -> Result<String, CommandError> {
    let state = app.state::<AppState>();
    let planner = state.planner.lock().map_err(|_| lock_err())?;
    let moved = snapshots::relocate(
        planner.db().conn(),
        &state.db_path,
        dir.as_deref(),
        Utc::now(),
    )?;
    Ok(moved.to_string_lossy().into_owned())
}

/// Saves a full JSON export where the user picks. Returns the path, or
/// `None` if cancelled.
#[tauri::command]
pub async fn export_json(app: AppHandle) -> Result<Option<String>, CommandError> {
    let name = format!("blotter-export-{}.json", Utc::now().format("%Y-%m-%d"));
    let Some(picked) = app
        .dialog()
        .file()
        .set_title("Export Blotter as JSON")
        .set_file_name(name)
        .add_filter("Blotter export", &["json"])
        .blocking_save_file()
    else {
        return Ok(None);
    };
    let path = picked
        .into_path()
        .map_err(|e| CommandError::internal(format!("couldn't save there: {e}")))?;
    let data = {
        let state = app.state::<AppState>();
        let planner = state.planner.lock().map_err(|_| lock_err())?;
        export::export(planner.db().conn(), Utc::now())?
    };
    let text =
        serde_json::to_string_pretty(&data).map_err(|e| CommandError::internal(e.to_string()))?;
    std::fs::write(&path, text)
        .map_err(|e| CommandError::internal(format!("couldn't write the file: {e}")))?;
    Ok(Some(path.to_string_lossy().into_owned()))
}

#[derive(Serialize)]
pub struct ImportPreview {
    pub path: String,
    pub summary: Summary,
}

/// Picks an export file and checks it, so the UI can confirm what it
/// holds before anything is replaced. `None` if cancelled.
#[tauri::command]
pub async fn pick_import(app: AppHandle) -> Result<Option<ImportPreview>, CommandError> {
    let Some(picked) = app
        .dialog()
        .file()
        .set_title("Import a Blotter export")
        .add_filter("Blotter export", &["json"])
        .blocking_pick_file()
    else {
        return Ok(None);
    };
    let path = picked
        .into_path()
        .map_err(|e| CommandError::internal(format!("couldn't open that file: {e}")))?;
    let data = read_export(&path)?;
    Ok(Some(ImportPreview {
        path: path.to_string_lossy().into_owned(),
        summary: export::summarize(&data),
    }))
}

fn read_export(path: &std::path::Path) -> Result<export::Export, CommandError> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| CommandError::internal(format!("couldn't read that file: {e}")))?;
    Ok(export::parse(&text)?)
}

/// Replaces everything with the export at `path`, after a snapshot.
#[tauri::command]
pub fn apply_import(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
) -> Result<Snapshot, CommandError> {
    let data = read_export(std::path::Path::new(&path))?;
    let before = {
        let mut planner = state.planner.lock().map_err(|_| lock_err())?;
        export::import(planner.db_mut(), &state.db_path, &data, Utc::now())?
    };
    replaced(&app, "import");
    Ok(before)
}

/// Takes the day's snapshot if it hasn't been taken (startup, then each
/// minute from the rollover check).
pub fn daily(app: &AppHandle) {
    let Some(state) = app.try_state::<AppState>() else {
        return;
    };
    let Ok(planner) = state.planner.lock() else {
        return;
    };
    let result = planner.settings().and_then(|s| {
        let conn = planner.db().conn();
        let dir = snapshots::dir(conn, &state.db_path)?;
        snapshots::daily_if_due(conn, &dir, s.tz, Utc::now())
    });
    if let Err(err) = result {
        eprintln!("blotter: daily snapshot failed: {err}");
    }
}

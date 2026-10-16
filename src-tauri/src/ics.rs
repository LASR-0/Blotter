//! Settings → Calendars (roadmap §5.8): ICS export and file import. File
//! pickers are native dialogs; they block, so those commands are async.

use std::path::{Path, PathBuf};

use blotter_core::CoreError;
use blotter_core::ics::{self, ImportSummary, Imported};
use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_dialog::DialogExt;

use crate::AppState;
use crate::commands::{CHANGED_EVENT, CommandError};

fn lock_err() -> CommandError {
    CommandError::internal("planner state is unavailable after an earlier crash")
}

/// Inclusive dates to export; none exports everything.
#[derive(Debug, Deserialize)]
pub struct DateRange {
    pub from: NaiveDate,
    pub to: NaiveDate,
}

#[derive(Serialize)]
pub struct ExportResult {
    pub path: String,
    pub events: usize,
    pub todos: usize,
}

/// Saves an .ics file where the user picks. `None` if cancelled.
#[tauri::command]
pub async fn export_ics(
    app: AppHandle,
    range: Option<DateRange>,
) -> Result<Option<ExportResult>, CommandError> {
    if let Some(r) = &range
        && r.to < r.from
    {
        return Err(CoreError::invalid("the range ends before it starts").into());
    }
    let name = match &range {
        Some(r) => format!("blotter-{}-to-{}.ics", r.from, r.to),
        None => format!("blotter-{}.ics", Utc::now().format("%Y-%m-%d")),
    };
    let Some(picked) = app
        .dialog()
        .file()
        .set_title("Export Blotter as an iCalendar file")
        .set_file_name(name)
        .add_filter("iCalendar", &["ics"])
        .blocking_save_file()
    else {
        return Ok(None);
    };
    let path = picked
        .into_path()
        .map_err(|e| CommandError::internal(format!("couldn't save there: {e}")))?;
    let out = {
        let state = app.state::<AppState>();
        let planner = state.planner.lock().map_err(|_| lock_err())?;
        let tz = planner.settings()?.tz;
        ics::export(
            planner.db().conn(),
            tz,
            range.map(|r| (r.from, r.to)),
            planner.now(),
        )?
    };
    std::fs::write(&path, &out.text)
        .map_err(|e| CommandError::internal(format!("couldn't write the file: {e}")))?;
    Ok(Some(ExportResult {
        path: path.to_string_lossy().into_owned(),
        events: out.events,
        todos: out.todos,
    }))
}

#[derive(Serialize)]
pub struct IcsPreview {
    pub path: String,
    pub summary: ImportSummary,
}

fn read_ics(path: &Path) -> Result<String, CommandError> {
    let bytes = std::fs::read(path)
        .map_err(|e| CommandError::internal(format!("couldn't read that file: {e}")))?;
    // Some exporters write Latin-1; keep what can be read.
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

/// Picks an .ics file and says what importing it would add. `None` if
/// cancelled.
#[tauri::command]
pub async fn pick_ics_import(app: AppHandle) -> Result<Option<IcsPreview>, CommandError> {
    let Some(picked) = app
        .dialog()
        .file()
        .set_title("Import an iCalendar file")
        .add_filter("iCalendar", &["ics", "ical", "ifb", "icalendar"])
        .blocking_pick_file()
    else {
        return Ok(None);
    };
    let path: PathBuf = picked
        .into_path()
        .map_err(|e| CommandError::internal(format!("couldn't open that file: {e}")))?;
    let text = read_ics(&path)?;
    let summary = {
        let state = app.state::<AppState>();
        let planner = state.planner.lock().map_err(|_| lock_err())?;
        let tz = planner.settings()?.tz;
        ics::preview(planner.db().conn(), tz, &text, planner.now())?
    };
    Ok(Some(IcsPreview {
        path: path.to_string_lossy().into_owned(),
        summary,
    }))
}

/// Imports the .ics file at `path` (after a snapshot) as one undoable batch.
#[tauri::command]
pub fn apply_ics_import(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
) -> Result<Imported, CommandError> {
    let text = read_ics(Path::new(&path))?;
    let imported = {
        let mut planner = state.planner.lock().map_err(|_| lock_err())?;
        ics::import(&mut planner, &state.db_path, &text)?
    };
    if let Some(batch_id) = &imported.batch_id {
        let _ = app.emit(
            CHANGED_EVENT,
            json!({ "batch_id": batch_id, "ops": ["import_ics"] }),
        );
    }
    Ok(imported)
}

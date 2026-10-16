//! Settings → Keybindings (roadmap §5.4): the user's overrides of the
//! default bindings, and moving them between installs as a JSON file.
//! The frontend owns the actions and checks bindings; this stores them.

use std::collections::BTreeMap;

use blotter_core::store;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;

use crate::AppState;
use crate::commands::CommandError;

/// The `format` of an exported keybindings file.
const FORMAT: &str = "blotter-keybindings";
const VERSION: u32 = 1;

fn lock_err() -> CommandError {
    CommandError::internal("planner state is unavailable after an earlier crash")
}

fn now() -> blotter_core::time::Instant {
    blotter_core::time::truncate(chrono::Utc::now())
}

#[derive(Debug, Serialize, Deserialize)]
struct KeybindingsFile {
    format: String,
    version: u32,
    /// Action id → binding (`""` for unbound).
    bindings: BTreeMap<String, String>,
}

/// Every override, by action id (`""` means unbound).
#[tauri::command]
pub fn list_keybindings(
    state: State<'_, AppState>,
) -> Result<BTreeMap<String, String>, CommandError> {
    let planner = state.planner.lock().map_err(|_| lock_err())?;
    Ok(store::keybindings::list(planner.db().conn())?)
}

/// Sets one action's binding (`""` unbinds it; `None` restores the default).
#[tauri::command]
pub fn set_keybinding(
    state: State<'_, AppState>,
    action_id: String,
    binding: Option<String>,
) -> Result<(), CommandError> {
    let planner = state.planner.lock().map_err(|_| lock_err())?;
    let conn = planner.db().conn();
    match binding {
        Some(b) => store::keybindings::set(conn, &action_id, &b, now())?,
        None => store::keybindings::reset(conn, &action_id)?,
    }
    Ok(())
}

/// Replaces every override at once (a swap, an import, or reset all with `{}`).
#[tauri::command]
pub fn replace_keybindings(
    state: State<'_, AppState>,
    bindings: BTreeMap<String, String>,
) -> Result<(), CommandError> {
    let mut planner = state.planner.lock().map_err(|_| lock_err())?;
    store::keybindings::replace_all(planner.db_mut().conn_mut(), &bindings, now())?;
    Ok(())
}

/// Saves the overrides to a file where the user picks. The path, or `None`
/// if cancelled.
#[tauri::command]
pub async fn export_keybindings(app: AppHandle) -> Result<Option<String>, CommandError> {
    let Some(picked) = app
        .dialog()
        .file()
        .set_title("Export keybindings")
        .set_file_name("blotter-keybindings.json")
        .add_filter("Blotter keybindings", &["json"])
        .blocking_save_file()
    else {
        return Ok(None);
    };
    let path = picked
        .into_path()
        .map_err(|e| CommandError::internal(format!("couldn't save there: {e}")))?;
    let bindings = {
        let state = app.state::<AppState>();
        let planner = state.planner.lock().map_err(|_| lock_err())?;
        store::keybindings::list(planner.db().conn())?
    };
    let file = KeybindingsFile {
        format: FORMAT.to_owned(),
        version: VERSION,
        bindings,
    };
    let text =
        serde_json::to_string_pretty(&file).map_err(|e| CommandError::internal(e.to_string()))?;
    std::fs::write(&path, text)
        .map_err(|e| CommandError::internal(format!("couldn't write the file: {e}")))?;
    Ok(Some(path.to_string_lossy().into_owned()))
}

/// Reads a keybindings file the user picks, without applying it (the
/// frontend checks it against the actions first). `None` if cancelled.
#[tauri::command]
pub async fn pick_keybindings_import(
    app: AppHandle,
) -> Result<Option<BTreeMap<String, String>>, CommandError> {
    let Some(picked) = app
        .dialog()
        .file()
        .set_title("Import keybindings")
        .add_filter("Blotter keybindings", &["json"])
        .blocking_pick_file()
    else {
        return Ok(None);
    };
    let path = picked
        .into_path()
        .map_err(|e| CommandError::internal(format!("couldn't open that file: {e}")))?;
    let text = std::fs::read_to_string(&path)
        .map_err(|e| CommandError::internal(format!("couldn't read that file: {e}")))?;
    parse_file(&text).map(Some)
}

fn parse_file(text: &str) -> Result<BTreeMap<String, String>, CommandError> {
    let not_ours = || {
        CommandError::from(blotter_core::CoreError::invalid(
            "that isn't a Blotter keybindings file",
        ))
    };
    let file: KeybindingsFile = serde_json::from_str(text).map_err(|_| not_ours())?;
    if file.format != FORMAT {
        return Err(not_ours());
    }
    if file.version > VERSION {
        return Err(blotter_core::CoreError::invalid(
            "that file is from a newer Blotter; update to import it",
        )
        .into());
    }
    Ok(file.bindings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_only_its_own_files() {
        let ok = r#"{ "format": "blotter-keybindings", "version": 1, "bindings": { "quick.add": "a" } }"#;
        assert_eq!(
            parse_file(ok).unwrap().get("quick.add").map(String::as_str),
            Some("a")
        );
        for bad in [
            r#"{ "format": "blotter", "version": 1, "bindings": {} }"#,
            r#"{ "format": "blotter-keybindings", "version": 9, "bindings": {} }"#,
            "not json",
        ] {
            assert!(parse_file(bad).is_err(), "{bad}");
        }
    }
}

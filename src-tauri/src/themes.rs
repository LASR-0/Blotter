//! Settings → Theme (roadmap §5.5): the user's own themes, and moving a
//! theme between installs as a JSON file.

use std::collections::BTreeMap;

use blotter_core::CoreError;
use blotter_core::store::themes::{self, UserTheme};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

use crate::AppState;
use crate::commands::CommandError;

/// The `format` of an exported theme file.
const FORMAT: &str = "blotter-theme";
const VERSION: u32 = 1;

fn lock_err() -> CommandError {
    CommandError::internal("planner state is unavailable after an earlier crash")
}

fn now() -> blotter_core::time::Instant {
    blotter_core::time::truncate(chrono::Utc::now())
}

#[derive(Debug, Serialize, Deserialize)]
struct ThemeFile {
    format: String,
    version: u32,
    name: String,
    tokens: BTreeMap<String, String>,
}

/// A theme read from a file, not yet saved.
#[derive(Debug, Serialize)]
pub struct ImportedTheme {
    pub name: String,
    pub tokens: BTreeMap<String, String>,
}

/// What the page sends to save: a new theme has no `id` yet.
#[derive(Debug, Deserialize)]
pub struct ThemeDraft {
    pub id: Option<String>,
    pub name: String,
    pub tokens: BTreeMap<String, String>,
}

#[tauri::command]
pub fn list_themes(state: State<'_, AppState>) -> Result<Vec<UserTheme>, CommandError> {
    let planner = state.planner.lock().map_err(|_| lock_err())?;
    Ok(themes::list(planner.db().conn())?)
}

/// Saves a user theme (new when it has no id) and returns it.
#[tauri::command]
pub fn save_theme(
    state: State<'_, AppState>,
    theme: ThemeDraft,
) -> Result<UserTheme, CommandError> {
    let planner = state.planner.lock().map_err(|_| lock_err())?;
    let conn = planner.db().conn();
    let now = now();
    let existing = match &theme.id {
        Some(id) => themes::list(conn)?.into_iter().find(|t| &t.id == id),
        None => None,
    };
    let saved = UserTheme {
        id: theme
            .id
            .unwrap_or_else(|| format!("user-{}", blotter_core::id::new_id().to_lowercase())),
        name: theme.name.trim().to_owned(),
        tokens: theme.tokens,
        created_at: existing.map_or(now, |t| t.created_at),
        updated_at: now,
    };
    themes::upsert(conn, &saved)?;
    Ok(saved)
}

#[tauri::command]
pub fn delete_theme(state: State<'_, AppState>, id: String) -> Result<(), CommandError> {
    let planner = state.planner.lock().map_err(|_| lock_err())?;
    themes::delete(planner.db().conn(), &id)?;
    Ok(())
}

/// Saves `name` and `tokens` to a file where the user picks (any theme,
/// built-in ones too). The path, or `None` if cancelled.
#[tauri::command]
pub async fn export_theme(
    app: AppHandle,
    name: String,
    tokens: BTreeMap<String, String>,
) -> Result<Option<String>, CommandError> {
    themes::check_tokens(&tokens)?;
    let file_name: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    let Some(picked) = app
        .dialog()
        .file()
        .set_title("Export theme")
        .set_file_name(format!(
            "{}.blotter-theme.json",
            file_name.trim_matches('-')
        ))
        .add_filter("Blotter theme", &["json"])
        .blocking_save_file()
    else {
        return Ok(None);
    };
    let path = picked
        .into_path()
        .map_err(|e| CommandError::internal(format!("couldn't save there: {e}")))?;
    let file = ThemeFile {
        format: FORMAT.to_owned(),
        version: VERSION,
        name,
        tokens,
    };
    let text =
        serde_json::to_string_pretty(&file).map_err(|e| CommandError::internal(e.to_string()))?;
    std::fs::write(&path, text)
        .map_err(|e| CommandError::internal(format!("couldn't write the file: {e}")))?;
    Ok(Some(path.to_string_lossy().into_owned()))
}

/// Reads a theme file the user picks, without saving it. `None` if cancelled.
#[tauri::command]
pub async fn pick_theme_import(app: AppHandle) -> Result<Option<ImportedTheme>, CommandError> {
    let Some(picked) = app
        .dialog()
        .file()
        .set_title("Import a theme")
        .add_filter("Blotter theme", &["json"])
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

fn parse_file(text: &str) -> Result<ImportedTheme, CommandError> {
    let not_ours = || CommandError::from(CoreError::invalid("that isn't a Blotter theme file"));
    let file: ThemeFile = serde_json::from_str(text).map_err(|_| not_ours())?;
    if file.format != FORMAT {
        return Err(not_ours());
    }
    if file.version > VERSION {
        return Err(
            CoreError::invalid("that theme is from a newer Blotter; update to import it").into(),
        );
    }
    themes::check_tokens(&file.tokens)?;
    Ok(ImportedTheme {
        name: file.name,
        tokens: file.tokens,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_only_safe_theme_files() {
        let ok = r##"{ "format": "blotter-theme", "version": 1, "name": "Night",
                      "tokens": { "--color-bg": "#000000" } }"##;
        assert_eq!(parse_file(ok).unwrap().name, "Night");
        for bad in [
            r#"{ "format": "blotter-keybindings", "version": 1, "name": "x", "tokens": {} }"#,
            r#"{ "format": "blotter-theme", "version": 2, "name": "x", "tokens": {} }"#,
            r#"{ "format": "blotter-theme", "version": 1, "name": "x",
                 "tokens": { "--color-bg": "url(https://evil.example/x)" } }"#,
        ] {
            assert!(parse_file(bad).is_err(), "{bad}");
        }
    }
}

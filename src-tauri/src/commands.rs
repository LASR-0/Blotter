//! Tauri commands. The UI reaches planner state only through `run_op`,
//! which goes through the same operation registry as the AI and MCP.

use blotter_core::{Actor, CoreError, OpInfo, OpOutcome};
use serde::Serialize;
use serde_json::Value;
use tauri::{AppHandle, Emitter, State};

use crate::AppState;

/// Event emitted after any committed mutation, so every window refreshes.
pub const CHANGED_EVENT: &str = "planner:changed";

/// Errors as the UI sees them: a stable code for branching, a message for
/// the toast, and per-field details for validation failures.
#[derive(Debug, Serialize)]
pub struct CommandError {
    pub code: &'static str,
    pub message: String,
    pub details: Vec<String>,
}

impl From<CoreError> for CommandError {
    fn from(err: CoreError) -> Self {
        let code = match &err {
            CoreError::NotFound { .. } => "not_found",
            CoreError::Invalid(_) => "invalid",
            CoreError::InvalidArgs { .. } => "invalid_args",
            CoreError::UnknownOp(_) => "unknown_op",
            CoreError::Db(_) | CoreError::Migration(_) => "database",
            CoreError::Corrupt { .. } => "corrupt",
            CoreError::Json(_) | CoreError::Io(_) => "internal",
        };
        let details = match &err {
            CoreError::InvalidArgs { errors, .. } => errors.clone(),
            _ => Vec::new(),
        };
        Self {
            code,
            message: err.to_string(),
            details,
        }
    }
}

fn lock_error() -> CommandError {
    CommandError {
        code: "internal",
        message: "planner state is unavailable after an earlier crash".into(),
        details: Vec::new(),
    }
}

#[derive(Debug, Clone, Serialize)]
struct ChangedPayload {
    batch_id: String,
    ops: Vec<String>,
}

#[derive(Serialize)]
pub struct DbStatus {
    path: String,
    journal_mode: String,
    schema_version: i64,
}

/// Exits the process (the tray keeps it alive otherwise).
#[tauri::command]
pub fn quit_app(app: AppHandle) {
    app.exit(0);
}

#[tauri::command]
pub fn app_version() -> &'static str {
    blotter_core::VERSION
}

#[tauri::command]
pub fn db_status(state: State<'_, AppState>) -> Result<DbStatus, CommandError> {
    let planner = state.planner.lock().map_err(|_| lock_error())?;
    Ok(DbStatus {
        path: state.db_path.display().to_string(),
        journal_mode: planner.db().journal_mode()?,
        schema_version: planner.db().schema_version()?,
    })
}

#[tauri::command]
pub fn list_ops(state: State<'_, AppState>) -> Result<Vec<OpInfo>, CommandError> {
    let planner = state.planner.lock().map_err(|_| lock_error())?;
    Ok(planner.registry().infos())
}

/// Settings the UI may read and write directly. Planner data never goes
/// through here; it uses `run_op` so it is validated and change-logged.
const UI_SETTING_KEYS: &[&str] = &["theme"];

fn check_setting_key(key: &str) -> Result<(), CommandError> {
    if UI_SETTING_KEYS.contains(&key) {
        Ok(())
    } else {
        Err(CommandError {
            code: "invalid",
            message: format!("'{key}' is not a UI setting"),
            details: Vec::new(),
        })
    }
}

#[tauri::command]
pub fn get_setting(state: State<'_, AppState>, key: String) -> Result<Option<Value>, CommandError> {
    check_setting_key(&key)?;
    let planner = state.planner.lock().map_err(|_| lock_error())?;
    Ok(blotter_core::store::settings::get(
        planner.db().conn(),
        &key,
    )?)
}

#[tauri::command]
pub fn set_setting(
    state: State<'_, AppState>,
    key: String,
    value: Value,
) -> Result<(), CommandError> {
    check_setting_key(&key)?;
    let planner = state.planner.lock().map_err(|_| lock_error())?;
    let now = blotter_core::time::truncate(chrono::Utc::now());
    blotter_core::store::settings::set(planner.db().conn(), &key, &value, now)?;
    Ok(())
}

/// Runs one operation as the user.
#[tauri::command]
pub fn run_op(
    app: AppHandle,
    state: State<'_, AppState>,
    name: String,
    args: Option<Value>,
) -> Result<OpOutcome, CommandError> {
    let outcome = {
        let mut planner = state.planner.lock().map_err(|_| lock_error())?;
        planner.execute(&name, args.unwrap_or(Value::Null), &Actor::User)?
    };
    if let Some(batch_id) = &outcome.batch_id {
        let payload = ChangedPayload {
            batch_id: batch_id.clone(),
            ops: vec![name],
        };
        // A failed emit only means no window is listening yet.
        let _ = app.emit(CHANGED_EVENT, payload);
    }
    Ok(outcome)
}

/// One call in a `run_batch` request.
#[derive(Debug, serde::Deserialize)]
pub struct BatchCall {
    pub name: String,
    #[serde(default)]
    pub args: Option<Value>,
}

/// Runs several operations as one atomic batch (one change-log batch, so
/// they undo together). If any fails, none take effect.
#[tauri::command]
pub fn run_batch(
    app: AppHandle,
    state: State<'_, AppState>,
    calls: Vec<BatchCall>,
) -> Result<OpOutcome, CommandError> {
    let names: Vec<String> = calls.iter().map(|c| c.name.clone()).collect();
    let calls: Vec<blotter_core::OpCall> = calls
        .into_iter()
        .map(|c| blotter_core::OpCall {
            name: c.name,
            args: c.args.unwrap_or(Value::Null),
        })
        .collect();
    let outcome = {
        let mut planner = state.planner.lock().map_err(|_| lock_error())?;
        planner.execute_batch(&calls, &Actor::User)?
    };
    if let Some(batch_id) = &outcome.batch_id {
        let _ = app.emit(
            CHANGED_EVENT,
            ChangedPayload {
                batch_id: batch_id.clone(),
                ops: names,
            },
        );
    }
    Ok(outcome)
}

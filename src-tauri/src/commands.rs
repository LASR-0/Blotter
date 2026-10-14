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

//! Launch at login (roadmap D11): keeps reminders firing after a restart.
//! The login launch passes `--hidden`, so Blotter starts in the tray.
//!
//! The OS keeps this state (a registry Run key on Windows, an autostart
//! .desktop file on Linux), so it isn't a planner setting.

use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

use crate::commands::CommandError;

/// The argument the login launch passes.
pub const LOGIN_ARGS: [&str; 1] = ["--hidden"];

pub fn plugin<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri_plugin_autostart::Builder::new()
        .args(LOGIN_ARGS)
        .build()
}

fn error(action: &str, err: impl std::fmt::Display) -> CommandError {
    CommandError::internal(format!("couldn't {action} launch at login: {err}"))
}

#[tauri::command]
pub fn get_autostart(app: AppHandle) -> Result<bool, CommandError> {
    app.autolaunch()
        .is_enabled()
        .map_err(|err| error("check", err))
}

#[tauri::command]
pub fn set_autostart(app: AppHandle, enabled: bool) -> Result<bool, CommandError> {
    let manager = app.autolaunch();
    if enabled {
        manager.enable().map_err(|err| error("turn on", err))?;
    } else {
        manager.disable().map_err(|err| error("turn off", err))?;
    }
    manager.is_enabled().map_err(|err| error("check", err))
}

//! The chat's pop-out window (roadmap §4.1, D10): a slim "sticky note"
//! that asks to stay on top and can't be maximised. It shows the same
//! chat as the panel docked in the main window; docking it back closes it
//! and reopens the panel there. Always-on-top isn't reliable on Wayland;
//! the README covers Hyprland window rules.

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

use crate::commands::CommandError;
use crate::shell::{MAIN_WINDOW, show_main};

pub const CHAT_WINDOW: &str = "chat";
/// Sent to the main window to open the docked panel on a thread.
pub(crate) const DOCK_EVENT: &str = "chat:dock";
/// Sent to the pop-out to switch it to a thread.
const SHOW_EVENT: &str = "chat:show-thread";

const WIDTH: f64 = 380.0;
const HEIGHT: f64 = 640.0;

#[derive(Debug, Clone, Serialize)]
struct ThreadPayload {
    thread_id: Option<String>,
}

fn url(thread_id: Option<&str>) -> WebviewUrl {
    // The frontend renders only the chat when its window label is "chat";
    // the thread to open rides along in the query string.
    let page = match thread_id {
        Some(id) if id.chars().all(|c| c.is_ascii_alphanumeric()) => {
            format!("index.html?thread={id}")
        }
        _ => "index.html".to_owned(),
    };
    WebviewUrl::App(page.into())
}

fn build(app: &AppHandle, thread_id: Option<&str>) -> tauri::Result<WebviewWindow> {
    WebviewWindowBuilder::new(app, CHAT_WINDOW, url(thread_id))
        .title("Blotter chat")
        .inner_size(WIDTH, HEIGHT)
        .min_inner_size(320.0, 420.0)
        .maximizable(false)
        .always_on_top(true)
        .build()
}

/// Pops the chat out (or brings the pop-out forward) on `thread_id`. Async
/// on purpose: building a webview inside a synchronous command deadlocks
/// on Windows (WebView2 needs the main thread the command would hold).
#[tauri::command]
pub async fn open_chat_window(
    app: AppHandle,
    thread_id: Option<String>,
) -> Result<(), CommandError> {
    if let Some(window) = app.get_webview_window(CHAT_WINDOW) {
        let _ = window.emit(SHOW_EVENT, ThreadPayload { thread_id });
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
        return Ok(());
    }
    build(&app, thread_id.as_deref())
        .map(|_| ())
        .map_err(|e| CommandError::internal(format!("couldn't open the chat window: {e}")))
}

/// Pins the pop-out on top, or lets other windows cover it.
#[tauri::command]
pub fn set_chat_on_top(app: AppHandle, on_top: bool) -> Result<(), CommandError> {
    if let Some(window) = app.get_webview_window(CHAT_WINDOW) {
        window
            .set_always_on_top(on_top)
            .map_err(|e| CommandError::internal(e.to_string()))?;
    }
    Ok(())
}

/// Closes the pop-out and reopens the chat docked in the main window.
#[tauri::command]
pub fn dock_chat(app: AppHandle, thread_id: Option<String>) {
    if let Some(window) = app.get_webview_window(CHAT_WINDOW) {
        let _ = window.close();
    }
    show_main(&app);
    if let Some(main) = app.get_webview_window(MAIN_WINDOW) {
        let _ = main.emit(DOCK_EVENT, ThreadPayload { thread_id });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_plain_thread_ids_reach_the_url() {
        let page = |id| match url(id) {
            WebviewUrl::App(p) => p.display().to_string(),
            _ => String::new(),
        };
        assert_eq!(
            page(Some("01M4ZJMVTFDKZ43KBJA2B14CWZ")),
            "index.html?thread=01M4ZJMVTFDKZ43KBJA2B14CWZ"
        );
        assert_eq!(page(Some("x&y=1")), "index.html");
        assert_eq!(page(None), "index.html");
    }
}

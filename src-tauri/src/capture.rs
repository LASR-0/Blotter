//! The quick-capture popup: a small undecorated window, centred and on top,
//! that writes a line straight to the planner and hides again. It opens from
//! the system-wide shortcut (Windows, X11), from `blotter --capture` (any
//! platform; how Wayland users bind it in their compositor), from the tray
//! menu and from the command palette.

use serde::Serialize;
use tauri::{
    AppHandle, Emitter, LogicalSize, Manager, Runtime, State, WebviewUrl, WebviewWindow,
    WebviewWindowBuilder,
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

pub const CAPTURE_WINDOW: &str = "capture";

/// Emitted to the capture window each time it is shown, so it refocuses its
/// input and picks up a theme changed while it was hidden.
const OPEN_EVENT: &str = "capture:open";

/// The design's default (Settings → Keybindings, "Quick capture (system-wide)").
pub const DEFAULT_SHORTCUT: &str = "Ctrl+Shift+Space";

const WIDTH: f64 = 600.0;
/// Close to the panel's usual height, so the first frame barely resizes.
const INITIAL_HEIGHT: f64 = 236.0;
const MIN_HEIGHT: f64 = 120.0;
const MAX_HEIGHT: f64 = 480.0;

/// Whether the system-wide shortcut is live, for the settings page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum ShortcutStatus {
    Registered {
        shortcut: String,
    },
    /// Global shortcuts don't work on Wayland; bind `blotter --capture` instead.
    Wayland,
    /// e.g. another app already owns the key combination.
    Failed {
        shortcut: String,
        reason: String,
    },
}

/// Whether this is a Wayland session, read through `env` so it is testable.
fn is_wayland_session(env: impl Fn(&str) -> Option<String>) -> bool {
    env("XDG_SESSION_TYPE").is_some_and(|v| v.eq_ignore_ascii_case("wayland"))
        || env("WAYLAND_DISPLAY").is_some_and(|v| !v.is_empty())
}

fn wayland_session() -> bool {
    cfg!(target_os = "linux") && is_wayland_session(|key| std::env::var(key).ok())
}

/// Keeps the window between a sensible minimum and maximum height.
fn clamp_height(height: f64) -> f64 {
    if height.is_finite() {
        height.clamp(MIN_HEIGHT, MAX_HEIGHT)
    } else {
        INITIAL_HEIGHT
    }
}

fn build<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<WebviewWindow<R>> {
    // The frontend renders the capture UI when its window label is "capture".
    WebviewWindowBuilder::new(app, CAPTURE_WINDOW, WebviewUrl::App("index.html".into()))
        .title("Blotter quick capture")
        .inner_size(WIDTH, INITIAL_HEIGHT)
        .resizable(false)
        .maximizable(false)
        .minimizable(false)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .center()
        .visible(false)
        .build()
}

/// Creates the hidden popup at startup so the shortcut shows it instantly.
pub fn prepare<R: Runtime>(app: &AppHandle<R>) {
    if app.get_webview_window(CAPTURE_WINDOW).is_none()
        && let Err(err) = build(app)
    {
        eprintln!("blotter: could not create the quick-capture window: {err}");
    }
}

/// Shows, centres and focuses the popup, creating it if needed.
pub fn show<R: Runtime>(app: &AppHandle<R>) {
    let window = match app.get_webview_window(CAPTURE_WINDOW) {
        Some(window) => window,
        None => match build(app) {
            Ok(window) => window,
            Err(err) => {
                eprintln!("blotter: could not open quick capture: {err}");
                return;
            }
        },
    };
    let _ = window.center();
    let _ = window.show();
    let _ = window.set_focus();
    let _ = app.emit_to(CAPTURE_WINDOW, OPEN_EVENT, ());
}

pub fn hide<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window(CAPTURE_WINDOW) {
        let _ = window.hide();
    }
}

/// The shortcut toggles: pressed again while the popup has focus, it hides.
fn toggle<R: Runtime>(app: &AppHandle<R>) {
    let focused = app
        .get_webview_window(CAPTURE_WINDOW)
        .and_then(|w| w.is_focused().ok())
        .unwrap_or(false);
    if focused {
        hide(app);
    } else {
        show(app);
    }
}

/// Registers the system-wide shortcut, except on Wayland where it can't work.
/// Never fails startup: problems are reported through the returned status.
pub fn register_shortcut<R: Runtime>(app: &AppHandle<R>) -> ShortcutStatus {
    if wayland_session() {
        return ShortcutStatus::Wayland;
    }
    let failed = |reason: String| ShortcutStatus::Failed {
        shortcut: DEFAULT_SHORTCUT.to_owned(),
        reason,
    };
    // Added here rather than on the builder: its X11 backend can fail to
    // start, and that must not stop the app.
    if let Err(err) = app.plugin(tauri_plugin_global_shortcut::Builder::new().build()) {
        return failed(err.to_string());
    }
    let registered =
        app.global_shortcut()
            .on_shortcut(DEFAULT_SHORTCUT, |app, _shortcut, event| {
                if event.state() == ShortcutState::Pressed {
                    toggle(app);
                }
            });
    match registered {
        Ok(()) => ShortcutStatus::Registered {
            shortcut: DEFAULT_SHORTCUT.to_owned(),
        },
        Err(err) => failed(err.to_string()),
    }
}

#[tauri::command]
pub fn show_capture(app: AppHandle) {
    show(&app);
}

#[tauri::command]
pub fn hide_capture(app: AppHandle) {
    hide(&app);
}

/// Resizes the popup to its content (chips can wrap onto a second line).
#[tauri::command]
pub fn fit_capture(app: AppHandle, height: f64) {
    if let Some(window) = app.get_webview_window(CAPTURE_WINDOW) {
        let _ = window.set_size(LogicalSize::new(WIDTH, clamp_height(height)));
    }
}

#[tauri::command]
pub fn capture_shortcut_status(status: State<'_, ShortcutStatus>) -> ShortcutStatus {
    status.inner().clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env<'a>(vars: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
        move |key| {
            vars.iter()
                .find(|(k, _)| *k == key)
                .map(|(_, v)| (*v).to_owned())
        }
    }

    #[test]
    fn detects_wayland_sessions() {
        assert!(is_wayland_session(env(&[("XDG_SESSION_TYPE", "wayland")])));
        assert!(is_wayland_session(env(&[("WAYLAND_DISPLAY", "wayland-1")])));
        assert!(!is_wayland_session(env(&[("XDG_SESSION_TYPE", "x11")])));
        assert!(!is_wayland_session(env(&[("WAYLAND_DISPLAY", "")])));
        assert!(!is_wayland_session(env(&[])));
    }

    #[test]
    fn height_is_clamped() {
        assert_eq!(clamp_height(214.0), 214.0);
        assert_eq!(clamp_height(10.0), MIN_HEIGHT);
        assert_eq!(clamp_height(5000.0), MAX_HEIGHT);
        assert_eq!(clamp_height(f64::NAN), INITIAL_HEIGHT);
    }

    #[test]
    fn status_serialises_for_the_ui() {
        let json = serde_json::to_value(ShortcutStatus::Wayland).unwrap();
        assert_eq!(json, serde_json::json!({ "state": "wayland" }));
        let json = serde_json::to_value(ShortcutStatus::Registered {
            shortcut: DEFAULT_SHORTCUT.into(),
        })
        .unwrap();
        assert_eq!(
            json,
            serde_json::json!({ "state": "registered", "shortcut": "Ctrl+Shift+Space" })
        );
    }
}

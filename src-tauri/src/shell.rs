//! Window lifecycle and the system tray: close hides to the tray so
//! reminders keep firing; Quit (tray menu) really exits.

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, Runtime, WindowEvent};

use crate::capture::{self, CAPTURE_WINDOW};
use crate::cli::{CliAction, LaunchArgs};

pub const MAIN_WINDOW: &str = "main";

/// Emitted with a [`CliAction`] when a launch asks for the chat.
pub const CLI_EVENT: &str = "app:cli";

const MENU_SHOW: &str = "show";
const MENU_CAPTURE: &str = "capture";
const MENU_QUIT: &str = "quit";

/// Shows, unminimizes and focuses the main window.
pub fn show_main<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW) {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// Acts on launch flags, from the first launch or forwarded from a second.
pub fn handle_launch<R: Runtime>(app: &AppHandle<R>, args: &LaunchArgs) {
    for flag in &args.unknown {
        eprintln!("blotter: ignoring unknown argument '{flag}'");
    }
    match args.action {
        CliAction::Show if args.hidden => {}
        CliAction::Show => show_main(app),
        // Only the popup: capturing shouldn't pull the whole app forward.
        CliAction::Capture => capture::show(app),
        CliAction::Chat => {
            show_main(app);
            let _ = app.emit(CLI_EVENT, CliAction::Chat);
        }
    }
}

/// Closing the main window hides it instead; the process stays in the tray.
/// The capture popup hides when closed or when it loses focus.
pub fn on_window_event<R: Runtime>(window: &tauri::Window<R>, event: &WindowEvent) {
    match (window.label(), event) {
        (MAIN_WINDOW, WindowEvent::CloseRequested { .. }) if !close_to_tray(window) => {
            window.app_handle().exit(0);
        }
        (MAIN_WINDOW | CAPTURE_WINDOW, WindowEvent::CloseRequested { api, .. }) => {
            api.prevent_close();
            let _ = window.hide();
        }
        // Clicking elsewhere dismisses it; the typed text stays for next time.
        (CAPTURE_WINDOW, WindowEvent::Focused(false)) => {
            let _ = window.hide();
        }
        _ => {}
    }
}

/// Builds the tray icon: left-click opens Blotter; the menu has Open, Quick
/// capture and Quit.
pub fn build_tray<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, MENU_SHOW, "Open Blotter", true, None::<&str>)?;
    let capture = MenuItem::with_id(app, MENU_CAPTURE, "Quick capture", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, MENU_QUIT, "Quit Blotter", true, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[&show, &capture, &PredefinedMenuItem::separator(app)?, &quit],
    )?;

    let mut builder = TrayIconBuilder::with_id("blotter")
        .tooltip("Blotter")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            MENU_SHOW => show_main(app),
            MENU_CAPTURE => capture::show(app),
            MENU_QUIT => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    Ok(())
}

/// Whether closing the main window hides it (Settings → General, on by
/// default) rather than quitting.
fn close_to_tray<R: Runtime>(window: &tauri::Window<R>) -> bool {
    let Some(state) = window.app_handle().try_state::<crate::AppState>() else {
        return true;
    };
    let Ok(planner) = state.planner.lock() else {
        return true;
    };
    blotter_core::store::settings::get::<bool>(
        planner.db().conn(),
        blotter_core::settings::KEY_CLOSE_TO_TRAY,
    )
    .ok()
    .flatten()
    .unwrap_or(true)
}

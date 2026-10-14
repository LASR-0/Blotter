//! Window lifecycle and the system tray: close hides to the tray so
//! reminders keep firing; Quit (tray menu) really exits.

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, Runtime, WindowEvent};

use crate::cli::{CliAction, LaunchArgs};

pub const MAIN_WINDOW: &str = "main";

/// Emitted with a [`CliAction`] when a launch asks for capture or chat.
pub const CLI_EVENT: &str = "app:cli";

const MENU_SHOW: &str = "show";
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
        action @ (CliAction::Capture | CliAction::Chat) => {
            show_main(app);
            let _ = app.emit(CLI_EVENT, action);
        }
    }
}

/// Closing the main window hides it instead; the process stays in the tray.
pub fn on_window_event<R: Runtime>(window: &tauri::Window<R>, event: &WindowEvent) {
    if let WindowEvent::CloseRequested { api, .. } = event {
        if window.label() == MAIN_WINDOW {
            api.prevent_close();
            let _ = window.hide();
        }
    }
}

/// Builds the tray icon: left-click opens Blotter, the menu has Open and Quit.
pub fn build_tray<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, MENU_SHOW, "Open Blotter", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, MENU_QUIT, "Quit Blotter", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &PredefinedMenuItem::separator(app)?, &quit])?;

    let mut builder = TrayIconBuilder::with_id("blotter")
        .tooltip("Blotter")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            MENU_SHOW => show_main(app),
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

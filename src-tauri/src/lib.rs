//! The Blotter desktop app: windows, tray, notifications, keychain and AI
//! HTTP on top of `blotter-core`.

#[tauri::command]
fn app_version() -> &'static str {
    blotter_core::VERSION
}

/// Applies Linux-specific environment workarounds. Must run before the
/// webview is created, i.e. before the Tauri builder starts.
fn apply_platform_workarounds() {
    #[cfg(target_os = "linux")]
    {
        // WebKitGTK on NVIDIA + Wayland can render a blank or flickering
        // window with the DMA-BUF renderer. Respect a user override.
        const KEY: &str = "WEBKIT_DISABLE_DMABUF_RENDERER";
        if std::env::var_os(KEY).is_none() {
            // SAFETY: called at the top of `run`, before Tauri or any other
            // thread is started, so no other thread can read the environment
            // concurrently.
            #[allow(unsafe_code)]
            unsafe {
                std::env::set_var(KEY, "1")
            };
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    apply_platform_workarounds();

    let result = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![app_version])
        .run(tauri::generate_context!());

    if let Err(err) = result {
        eprintln!("Blotter failed to start: {err}");
        std::process::exit(1);
    }
}

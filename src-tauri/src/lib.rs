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
            unsafe { std::env::set_var(KEY, "1") };
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    apply_platform_workarounds();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![app_version])
        .run(tauri::generate_context!())
        .expect("error while running Blotter");
}

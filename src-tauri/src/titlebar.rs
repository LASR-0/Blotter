//! The window title bar in the theme's colours (Settings → Theme, Window
//! group). Windows 11 takes any caption and text colour; Windows 10 only
//! light or dark, which `set_theme` gives it. Elsewhere the desktop draws
//! the title bar (Hyprland has none), so only the light/dark hint applies.
//!
//! On Windows the bar draws neither the icon nor the title (Luke): the
//! window keeps its title, so the taskbar, Alt+Tab and screen readers
//! still name it.

use tauri::{Theme, WebviewWindow};

use crate::commands::CommandError;

/// `#rrggbb` as a Win32 COLORREF (0x00bbggrr).
#[cfg_attr(not(windows), allow(dead_code))]
fn colorref(hex: &str) -> Option<u32> {
    let h = hex.trim().strip_prefix('#')?;
    if h.len() != 6 {
        return None;
    }
    let rgb = u32::from_str_radix(h, 16).ok()?;
    let (r, g, b) = (rgb >> 16, (rgb >> 8) & 0xff, rgb & 0xff);
    Some(b << 16 | g << 8 | r)
}

/// Colours the calling window's title bar.
#[tauri::command]
pub fn set_titlebar(
    window: WebviewWindow,
    caption: String,
    text: String,
    dark: bool,
) -> Result<(), CommandError> {
    window
        .set_theme(Some(if dark { Theme::Dark } else { Theme::Light }))
        .map_err(|e| CommandError::internal(e.to_string()))?;
    #[cfg(windows)]
    if let Ok(hwnd) = window.hwnd() {
        dwm::hide_icon_and_title(hwnd.0);
        if let (Some(caption), Some(text)) = (colorref(&caption), colorref(&text)) {
            dwm::set_colors(hwnd.0, caption, text);
        }
    }
    #[cfg(not(windows))]
    let _ = (caption, text);
    Ok(())
}

#[cfg(windows)]
#[allow(unsafe_code)]
mod dwm {
    use std::ffi::c_void;

    // Windows 11 (build 22000+) window attributes; older builds reject them.
    const DWMWA_BORDER_COLOR: u32 = 34;
    const DWMWA_CAPTION_COLOR: u32 = 35;
    const DWMWA_TEXT_COLOR: u32 = 36;

    // SetWindowThemeAttribute's non-client options: draw no caption text, no icon.
    const WTA_NONCLIENT: i32 = 1;
    const WTNCA_NODRAWCAPTION: u32 = 0x1;
    const WTNCA_NODRAWICON: u32 = 0x2;

    #[repr(C)]
    struct WtaOptions {
        flags: u32,
        mask: u32,
    }

    #[link(name = "uxtheme")]
    unsafe extern "system" {
        fn SetWindowThemeAttribute(
            hwnd: *mut c_void,
            attr: i32,
            value: *const c_void,
            size: u32,
        ) -> i32;
    }

    pub fn hide_icon_and_title(hwnd: *mut c_void) {
        let flags = WTNCA_NODRAWCAPTION | WTNCA_NODRAWICON;
        let options = WtaOptions { flags, mask: flags };
        // SAFETY: `hwnd` is this live window's handle from Tauri, and `options`
        // is a WTA_OPTIONS that outlives the call, its size passed with it.
        // A failure leaves the default caption; nothing to undo.
        unsafe {
            SetWindowThemeAttribute(
                hwnd,
                WTA_NONCLIENT,
                (&raw const options).cast(),
                size_of::<WtaOptions>() as u32,
            );
        }
    }

    #[link(name = "dwmapi")]
    unsafe extern "system" {
        fn DwmSetWindowAttribute(
            hwnd: *mut c_void,
            attr: u32,
            value: *const c_void,
            size: u32,
        ) -> i32;
    }

    pub fn set_colors(hwnd: *mut c_void, caption: u32, text: u32) {
        for (attr, value) in [
            (DWMWA_CAPTION_COLOR, caption),
            (DWMWA_BORDER_COLOR, caption),
            (DWMWA_TEXT_COLOR, text),
        ] {
            // SAFETY: `hwnd` is this live window's handle from Tauri, and `value`
            // is a u32 COLORREF that outlives the call, its size passed with it.
            // A failure (an older Windows) leaves the default colours; nothing to undo.
            unsafe {
                DwmSetWindowAttribute(hwnd, attr, (&raw const value).cast(), 4);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::colorref;

    #[test]
    fn hex_becomes_colorref() {
        assert_eq!(colorref("#fe5858"), Some(0x0058_58fe));
        assert_eq!(colorref("#123"), None);
        assert_eq!(colorref("red"), None);
    }
}

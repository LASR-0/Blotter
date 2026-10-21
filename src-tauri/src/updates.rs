//! In-app updates (roadmap §13, D31): once a day, and when asked, Blotter
//! reads the latest release's `latest.json` (signed with the updater key,
//! not an OS code signature). Installs it owns (NSIS, MSI, AppImage) can
//! update in place and restart; ones a package manager owns (.deb, Flatpak,
//! AUR, a source build) only hear that a new version is out.

use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::utils::config::BundleType;
use tauri::utils::platform::bundle_type;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_opener::OpenerExt;
use tauri_plugin_updater::UpdaterExt;

use blotter_core::settings::KEY_UPDATE_CHECK;
use blotter_core::store;

use crate::AppState;
use crate::commands::CommandError;

/// Tells the UI a new version is out (payload: `UpdateInfo`).
pub const UPDATE_EVENT: &str = "planner:update";

const RELEASES: &str = "https://github.com/LASR-0/Blotter/releases/tag/v";
const CHECK_EVERY: Duration = Duration::from_secs(24 * 60 * 60);
/// The first automatic check waits, so it doesn't slow the start.
const FIRST_CHECK_AFTER: Duration = Duration::from_secs(60);

/// A newer release.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct UpdateInfo {
    pub version: String,
    pub current: String,
    /// Its release notes on GitHub.
    pub notes_url: String,
    /// Blotter can install it itself; otherwise `how` says what to do.
    pub can_install: bool,
    pub how: Option<String>,
}

pub struct Updates {
    started: Instant,
    last_check: Mutex<Option<Instant>>,
    available: Mutex<Option<UpdateInfo>>,
}

impl Default for Updates {
    fn default() -> Self {
        Self {
            started: Instant::now(),
            last_check: Mutex::new(None),
            available: Mutex::new(None),
        }
    }
}

/// `None` when this install can update itself, else how the user updates it.
fn managed_by() -> Option<&'static str> {
    if crate::flatpak::app_id().is_some() {
        return Some("Update it with your software centre, or flatpak update.");
    }
    match bundle_type() {
        Some(BundleType::Nsis | BundleType::Msi | BundleType::AppImage) => None,
        Some(BundleType::Deb) => Some("Install the new .deb from the release page."),
        // The AUR package and other source builds.
        _ if cfg!(target_os = "linux") => {
            Some("Update it with your package manager (e.g. the AUR).")
        }
        _ => Some("Download it from the release page."),
    }
}

fn lock_err() -> CommandError {
    CommandError::internal("update state is unavailable after an earlier crash")
}

/// A failed check, in words: the plugin's own say "release JSON".
fn check_err(e: tauri_plugin_updater::Error) -> CommandError {
    let text = e.to_string();
    CommandError::internal(if text.contains("release JSON") {
        "Couldn't check for updates: GitHub has no update information for the latest release \
         (or Blotter couldn't reach it)."
            .to_owned()
    } else {
        format!("Couldn't check for updates: {text}")
    })
}

async fn check(app: &AppHandle) -> Result<Option<UpdateInfo>, CommandError> {
    let updater = app
        .updater()
        .map_err(|e| CommandError::internal(format!("updates are unavailable: {e}")))?;
    let found = updater.check().await.map_err(check_err)?;
    let info = found.map(|u| {
        let how = managed_by();
        UpdateInfo {
            notes_url: format!("{RELEASES}{}", u.version),
            version: u.version,
            current: u.current_version,
            can_install: how.is_none(),
            how: how.map(str::to_owned),
        }
    });
    let updates = app.state::<Updates>();
    *updates.last_check.lock().map_err(|_| lock_err())? = Some(Instant::now());
    *updates.available.lock().map_err(|_| lock_err())? = info.clone();
    Ok(info)
}

/// Looks for a newer release now.
#[tauri::command]
pub async fn check_for_update(app: AppHandle) -> Result<Option<UpdateInfo>, CommandError> {
    check(&app).await
}

/// The newer release the last check found, if any.
#[tauri::command]
pub fn available_update(updates: State<'_, Updates>) -> Result<Option<UpdateInfo>, CommandError> {
    Ok(updates.available.lock().map_err(|_| lock_err())?.clone())
}

/// Opens a release's notes on GitHub in the browser.
#[tauri::command]
pub fn open_release_notes(app: AppHandle, version: String) -> Result<(), CommandError> {
    if !version
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
    {
        return Err(CommandError::internal(format!(
            "'{version}' isn't a version"
        )));
    }
    app.opener()
        .open_url(format!("{RELEASES}{version}"), None::<&str>)
        .map_err(|e| CommandError::internal(format!("couldn't open the browser: {e}")))
}

/// Downloads and installs the newer release, then restarts into it. On
/// Windows the installer closes Blotter itself and starts the new version.
#[tauri::command]
pub async fn install_update(app: AppHandle) -> Result<(), CommandError> {
    if let Some(how) = managed_by() {
        return Err(CommandError::internal(how.to_owned()));
    }
    let updater = app
        .updater()
        .map_err(|e| CommandError::internal(format!("updates are unavailable: {e}")))?;
    let Some(update) = updater.check().await.map_err(check_err)? else {
        return Err(CommandError::internal("Blotter is already up to date."));
    };
    update
        .download_and_install(|_, _| {}, || {})
        .await
        .map_err(|e| CommandError::internal(format!("the update didn't install: {e}")))?;
    app.restart();
}

/// From the minute check: a check a minute after start, then once a day,
/// unless turned off. Tells the UI when it finds a version it hadn't.
pub fn check_if_due(app: &AppHandle) {
    let Some(updates) = app.try_state::<Updates>() else {
        return;
    };
    let due = match updates.last_check.lock() {
        Ok(last) => match *last {
            Some(at) => at.elapsed() >= CHECK_EVERY,
            None => updates.started.elapsed() >= FIRST_CHECK_AFTER,
        },
        Err(_) => false,
    };
    if !due || !enabled(app) {
        return;
    }
    // Mark it now, so a slow or failed check isn't retried every minute.
    if let Ok(mut last) = updates.last_check.lock() {
        *last = Some(Instant::now());
    }
    let before = updates.available.lock().ok().and_then(|a| a.clone());
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        match check(&app).await {
            Ok(Some(info)) if before.as_ref() != Some(&info) => {
                let _ = app.emit(UPDATE_EVENT, info);
            }
            Ok(_) => {}
            Err(err) => eprintln!("blotter: {}", err.message),
        }
    });
}

fn enabled(app: &AppHandle) -> bool {
    let Some(state) = app.try_state::<AppState>() else {
        return false;
    };
    let Ok(planner) = state.planner.lock() else {
        return false;
    };
    store::settings::get(planner.db().conn(), KEY_UPDATE_CHECK)
        .ok()
        .flatten()
        .unwrap_or(true)
}

#[cfg(test)]
mod tests {
    /// The updater's settings in tauri.conf.json parse the way the plugin
    /// reads them at start-up, with a minisign public key and an https
    /// endpoint (a bad one would stop Blotter starting).
    #[test]
    fn updater_config_is_valid() {
        let conf: serde_json::Value =
            serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
        let updater = conf["plugins"]["updater"].clone();
        let parsed: tauri_plugin_updater::Config = serde_json::from_value(updater).unwrap();
        assert!(
            parsed.endpoints[0]
                .as_str()
                .starts_with("https://github.com/")
        );
        let key =
            String::from_utf8(base64_decode(&parsed.pubkey).expect("the public key is base64"))
                .unwrap();
        assert!(
            key.starts_with("untrusted comment: minisign public key"),
            "{key}"
        );
    }

    fn base64_decode(s: &str) -> Option<Vec<u8>> {
        const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut bits = 0u32;
        let mut n = 0;
        let mut out = Vec::new();
        for c in s.bytes().filter(|c| *c != b'=') {
            let v = ALPHABET.iter().position(|a| *a == c)? as u32;
            bits = (bits << 6) | v;
            n += 6;
            if n >= 8 {
                n -= 8;
                out.push((bits >> n) as u8);
            }
        }
        Some(out)
    }
}

//! Sync with a home server running Blotter Sync (roadmap §14.6).
//!
//! In Local only mode (no server paired) this does nothing. Once paired, a
//! background task syncs: after local writes (debounced), when the server's
//! change stream announces a new seq, every minute as a fallback, and with
//! backoff and jitter while the server can't be reached. Each sync pushes
//! the outbox and pulls from the cursor ([`engine`]); remote changes reach
//! the windows through the usual changed event. The UI reads one status,
//! never a toast per attempt. Settings → Sync pairs, edits the address
//! list and disconnects through the commands at the end ([`pairing`]).

pub mod address;
pub mod client;
pub mod engine;
#[cfg(test)]
mod fake;
pub mod pairing;
pub mod token;

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use blotter_core::sync::api::Notice;
use blotter_core::sync::{capture, link};
use futures_util::StreamExt;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Listener, Manager, State};
use tokio::sync::Notify;
use tokio::task::JoinHandle;

use crate::AppState;
use crate::ai::sse;
use crate::commands::{CHANGED_EVENT, ChangedPayload, CommandError};
use client::Http;
use engine::{Attention, Engine, Problem};

/// Tells the UI the sync status changed; it then calls `sync_status`.
pub const SYNC_STATUS_EVENT: &str = "sync:status";

/// Tells the UI a sync notice is waiting; it then calls `take_sync_notice`.
pub const SYNC_NOTICE_EVENT: &str = "sync:notice";

/// The changed event's op for rows a sync merged, so they don't wake the
/// engine again as if they were local writes.
const SYNC_OP: &str = "sync";

/// Local writes within this long of each other go out in one push.
const DEBOUNCE: Duration = Duration::from_millis(1500);
/// How often to sync while connected, in case a notice was missed.
const POLL: Duration = Duration::from_secs(60);
/// How long to wait before trying again after something only the user
/// can fix (they may have fixed it without telling the app).
const ATTENTION_RETRY: Duration = Duration::from_secs(15 * 60);
/// In Local only mode the task sleeps until woken by pairing.
const IDLE: Duration = Duration::from_secs(60 * 60);
const BACKOFF_FIRST: Duration = Duration::from_secs(2);
const BACKOFF_MAX: Duration = Duration::from_secs(5 * 60);

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum SyncState {
    LocalOnly,
    Syncing,
    Synced {
        address: String,
        tailscale: bool,
        /// Blotter Sync's version.
        server_version: Option<String>,
    },
    Offline {
        /// Every address failed and at least one is a Tailscale address:
        /// the UI asks whether Tailscale is connected on this device.
        tailscale: bool,
        /// Why the last attempt failed, per address.
        tried: Vec<(String, String)>,
        retry_in_s: u64,
    },
    Attention {
        #[serde(flatten)]
        attention: Attention,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SyncStatus {
    #[serde(flatten)]
    pub state: SyncState,
    /// Local changes the server doesn't have yet.
    pub unsent: u64,
    /// When the last sync finished, as stored (RFC 3339, UTC).
    pub last_sync_at: Option<String>,
}

/// The engine's handle: what wakes it and what it last reported.
pub struct SyncEngine {
    wake: Notify,
    /// The link changed (paired, disconnected, addresses edited): forget
    /// the route and the change stream before the next sync.
    relinked: AtomicBool,
    status: Mutex<SyncStatus>,
    /// A pairing waiting on Use the server's planner or Merge.
    joining: Mutex<Option<pairing::PendingJoin>>,
    /// The server's planner was replaced and fetched again: held until a
    /// window shows it, like the rollover notice.
    notice: Mutex<Option<engine::Replaced>>,
}

impl Default for SyncEngine {
    fn default() -> Self {
        Self {
            wake: Notify::new(),
            relinked: AtomicBool::new(false),
            joining: Mutex::new(None),
            notice: Mutex::new(None),
            status: Mutex::new(SyncStatus {
                state: SyncState::LocalOnly,
                unsent: 0,
                last_sync_at: None,
            }),
        }
    }
}

/// Asks the engine to sync now (after pairing, or a change of addresses).
pub fn wake(app: &AppHandle) {
    if let Some(sync) = app.try_state::<SyncEngine>() {
        sync.wake.notify_one();
    }
}

/// Asks the engine to start over with the link as it now stands.
fn relink(app: &AppHandle) {
    if let Some(sync) = app.try_state::<SyncEngine>() {
        sync.relinked.store(true, Ordering::SeqCst);
        sync.wake.notify_one();
    }
}

#[tauri::command]
pub fn sync_status(sync: State<'_, SyncEngine>) -> Option<SyncStatus> {
    sync.status.lock().ok().map(|s| s.clone())
}

/// Starts the engine and wakes it on local writes.
pub fn start(app: AppHandle) {
    let listener = app.clone();
    app.listen_any(CHANGED_EVENT, move |event| {
        let ops = serde_json::from_str::<serde_json::Value>(event.payload())
            .ok()
            .and_then(|v| v.get("ops").cloned());
        let merged = ops.is_some_and(|ops| ops == serde_json::json!([SYNC_OP]));
        if !merged {
            wake(&listener);
        }
    });
    tauri::async_runtime::spawn(run(app));
}

/// Exponential backoff with ±20% jitter, so devices don't retry in step.
#[derive(Default)]
struct Backoff(u32);

impl Backoff {
    fn next(&mut self) -> Duration {
        let base = BACKOFF_FIRST
            .saturating_mul(1 << self.0.min(16))
            .min(BACKOFF_MAX);
        self.0 = self.0.saturating_add(1);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.subsec_nanos());
        let jitter = f64::from(nanos % 400) / 1000.0 + 0.8; // 0.8..1.2
        base.mul_f64(jitter)
    }

    fn reset(&mut self) {
        self.0 = 0;
    }
}

/// Where the engine stands, read from the database.
fn snapshot(app: &AppHandle) -> Option<(Option<link::Link>, u64)> {
    let state = app.try_state::<AppState>()?;
    let planner = state.planner.lock().ok()?;
    let conn = planner.db().conn();
    Some((
        link::get(conn).ok()?,
        capture::pending_count(conn).unwrap_or(0),
    ))
}

fn report(app: &AppHandle, state: SyncState) {
    let Some(sync) = app.try_state::<SyncEngine>() else {
        return;
    };
    let (l, unsent) = snapshot(app).unwrap_or((None, 0));
    let status = SyncStatus {
        state,
        unsent,
        last_sync_at: l
            .and_then(|l| l.last_sync_at)
            .map(blotter_core::time::fmt_instant),
    };
    let changed = match sync.status.lock() {
        Ok(mut slot) if *slot != status => {
            *slot = status;
            true
        }
        _ => false,
    };
    if changed {
        let _ = app.emit(SYNC_STATUS_EVENT, ());
    }
}

/// The change stream for one address: wakes the engine on every notice,
/// and once more when the stream ends, so the engine reconnects.
async fn listen(app: AppHandle, http: Http, address: String, token: String) {
    if let Ok(resp) = http.events(&address, &token).await {
        let mut parser = sse::Parser::default();
        let mut body = resp.bytes_stream();
        while let Some(Ok(chunk)) = body.next().await {
            for event in parser.feed(&chunk) {
                if event.name.as_deref() == Some("seq")
                    && serde_json::from_str::<Notice>(&event.data).is_ok()
                {
                    wake(&app);
                }
            }
        }
    }
    wake(&app);
}

/// The open change stream: its address and the task reading it.
type Stream = Option<(String, JoinHandle<()>)>;

fn stop_stream(stream: &mut Stream) {
    if let Some((_, task)) = stream.take() {
        task.abort();
    }
}

async fn run(app: AppHandle) {
    let http = match Http::new() {
        Ok(http) => http,
        Err(err) => {
            eprintln!("blotter: sync is unavailable: {err}");
            return;
        }
    };
    let mut engine = Engine::new(http);
    if let Some(state) = app.try_state::<AppState>() {
        engine = engine.snapshots_beside(state.db_path.clone());
    }
    let mut backoff = Backoff::default();
    let mut stream: Stream = None;
    let mut wait = Duration::ZERO;

    loop {
        if let Some(sync) = app.try_state::<SyncEngine>() {
            tokio::select! {
                () = sync.wake.notified() => tokio::time::sleep(DEBOUNCE).await,
                () = tokio::time::sleep(wait) => {}
            }
        } else {
            tokio::time::sleep(wait.max(DEBOUNCE)).await;
        }
        if app
            .try_state::<SyncEngine>()
            .is_some_and(|sync| sync.relinked.swap(false, Ordering::SeqCst))
        {
            stop_stream(&mut stream);
            engine.reset_route();
            backoff.reset();
        }

        let Some((l, unsent)) = snapshot(&app) else {
            wait = BACKOFF_FIRST;
            continue;
        };
        let Some(l) = l else {
            stop_stream(&mut stream);
            engine.reset_route();
            report(&app, SyncState::LocalOnly);
            wait = IDLE;
            continue;
        };
        let token = match token::get(&l.server_id) {
            Ok(t) => t.unwrap_or_default(),
            Err(err) => {
                eprintln!("blotter: {err}");
                String::new()
            }
        };
        if unsent > 0 {
            report(&app, SyncState::Syncing);
        }

        let outcome = {
            let Some(state) = app.try_state::<AppState>() else {
                continue;
            };
            engine
                .sync_once(&state.planner, &token, Instant::now())
                .await
        };
        match outcome {
            Ok(done) => {
                backoff.reset();
                if done.pulled > 0 || done.replaced.is_some() {
                    merged(&app);
                }
                if let Some(replaced) = done.replaced {
                    if let Some(sync) = app.try_state::<SyncEngine>()
                        && let Ok(mut slot) = sync.notice.lock()
                    {
                        *slot = Some(replaced);
                    }
                    let _ = app.emit(SYNC_NOTICE_EVENT, ());
                }
                let current = stream
                    .as_ref()
                    .is_some_and(|(a, task)| *a == done.address && !task.is_finished());
                if !current {
                    stop_stream(&mut stream);
                    // On Tauri's runtime already: tokio's handle can abort.
                    let task = tokio::spawn(listen(
                        app.clone(),
                        engine.transport().clone(),
                        done.address.clone(),
                        token.clone(),
                    ));
                    stream = Some((done.address.clone(), task));
                }
                report(
                    &app,
                    SyncState::Synced {
                        tailscale: address::is_tailscale(&done.address),
                        address: done.address,
                        server_version: done.server_version,
                    },
                );
                wait = POLL;
            }
            Err(Problem::LocalOnly) => {
                stop_stream(&mut stream);
                report(&app, SyncState::LocalOnly);
                wait = IDLE;
            }
            Err(Problem::Offline { tried, tailscale }) => {
                stop_stream(&mut stream);
                wait = backoff.next();
                report(
                    &app,
                    SyncState::Offline {
                        tailscale,
                        tried,
                        retry_in_s: wait.as_secs(),
                    },
                );
            }
            Err(Problem::Retry(message)) => {
                eprintln!("blotter: sync will retry: {message}");
                wait = backoff.next();
            }
            Err(Problem::Local(message)) => {
                eprintln!("blotter: sync failed here: {message}");
                wait = backoff.next();
            }
            Err(Problem::Attention(attention)) => {
                stop_stream(&mut stream);
                engine.reset_route();
                report(&app, SyncState::Attention { attention });
                wait = ATTENTION_RETRY;
            }
        }
    }
}

/// Settings → Sync's refusals, as sentences the page shows.
fn refused(message: String) -> CommandError {
    CommandError {
        code: "invalid",
        message,
        details: Vec::new(),
    }
}

fn http() -> Result<Http, CommandError> {
    Http::new().map_err(|e| CommandError::internal(format!("sync is unavailable: {e}")))
}

/// Rows came from the server: the windows refresh, and the engine isn't
/// woken as if they were local writes.
fn merged(app: &AppHandle) {
    let _ = app.emit(
        CHANGED_EVENT,
        ChangedPayload {
            batch_id: String::new(),
            ops: vec![SYNC_OP.into()],
        },
    );
}

#[tauri::command]
pub fn sync_info(
    state: State<'_, AppState>,
    sync: State<'_, SyncEngine>,
) -> Result<pairing::SyncInfo, CommandError> {
    let mut info = pairing::info(&state.planner).map_err(CommandError::internal)?;
    info.joining = sync
        .joining
        .lock()
        .ok()
        .and_then(|j| j.as_ref().map(|p| p.address.clone()));
    Ok(info)
}

/// Pairs with the server at `address` using the code `blotter-sync pair`
/// printed; the token goes to the OS keychain. Answers what happened to
/// the planners: `seeded`, `fetched`, `again`, or `choose`, when the page
/// asks Use the server's planner or Merge and calls `sync_join`.
#[tauri::command]
pub async fn sync_pair(
    app: AppHandle,
    state: State<'_, AppState>,
    sync: State<'_, SyncEngine>,
    address: String,
    code: String,
    name: String,
) -> Result<&'static str, CommandError> {
    let http = http()?;
    let joined = pairing::pair(&http, &state.planner, &address, &code, &name, token::set)
        .await
        .map_err(refused)?;
    let answer = joined.as_str();
    match joined {
        pairing::Joined::Choose(pending) => {
            if let Ok(mut slot) = sync.joining.lock() {
                *slot = Some(pending);
            }
        }
        pairing::Joined::Fetched | pairing::Joined::Seeded => merged(&app),
        pairing::Joined::Again => {}
    }
    relink(&app);
    Ok(answer)
}

fn take_pending(sync: &SyncEngine) -> Result<pairing::PendingJoin, CommandError> {
    sync.joining
        .lock()
        .ok()
        .and_then(|mut j| j.take())
        .ok_or_else(|| refused("Nothing is waiting to join; pair again.".into()))
}

/// Joins the server paired last as the user chose: `server` or `merge`.
#[tauri::command]
pub fn sync_join(
    app: AppHandle,
    state: State<'_, AppState>,
    sync: State<'_, SyncEngine>,
    choice: pairing::Choice,
) -> Result<(), CommandError> {
    let pending = take_pending(&sync)?;
    pairing::finish_join(&state.planner, pending, choice, &state.db_path).map_err(refused)?;
    merged(&app);
    relink(&app);
    Ok(())
}

/// Gives up on the pairing waiting to join: the server forgets this device
/// and the token goes. The planner is as it was.
#[tauri::command]
pub async fn sync_join_cancel(sync: State<'_, SyncEngine>) -> Result<(), CommandError> {
    let pending = take_pending(&sync)?;
    let token = token::get(&pending.server_id).ok().flatten();
    pairing::abandon(&http()?, &pending, token.as_deref()).await;
    if let Err(err) = token::delete(&pending.server_id) {
        eprintln!("blotter: {err}");
    }
    Ok(())
}

/// The waiting notice that the server's planner was replaced, once.
#[tauri::command]
pub fn take_sync_notice(sync: State<'_, SyncEngine>) -> Option<engine::Replaced> {
    sync.notice.lock().ok().and_then(|mut slot| slot.take())
}

#[tauri::command]
pub fn sync_set_addresses(
    app: AppHandle,
    state: State<'_, AppState>,
    addresses: Vec<String>,
) -> Result<Vec<String>, CommandError> {
    let saved = pairing::set_addresses(&state.planner, &addresses).map_err(refused)?;
    relink(&app);
    Ok(saved)
}

#[tauri::command]
pub async fn sync_test_address(
    state: State<'_, AppState>,
    address: String,
) -> Result<pairing::AddressCheck, CommandError> {
    let server_id = pairing::linked_server(&state.planner).map_err(CommandError::internal)?;
    pairing::test_address(&http()?, &address, server_id.as_deref())
        .await
        .map_err(refused)
}

/// Back to Local only: revokes this device on the server when it can be
/// reached, forgets the link and removes the token. The planner stays.
#[tauri::command]
pub async fn sync_disconnect(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<pairing::Disconnected, CommandError> {
    let server_id = pairing::linked_server(&state.planner).map_err(CommandError::internal)?;
    let token = match server_id.as_deref().map(token::get) {
        Some(Ok(t)) => t,
        Some(Err(err)) => {
            eprintln!("blotter: {err}");
            None
        }
        None => None,
    };
    let done = pairing::disconnect(&http()?, &state.planner, token.as_deref())
        .await
        .map_err(refused)?;
    if let Some(id) = &done.server_id
        && let Err(err) = token::delete(id)
    {
        eprintln!("blotter: {err}");
    }
    relink(&app);
    Ok(done)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backoff_grows_to_a_cap_with_jitter() {
        let mut b = Backoff::default();
        let first = b.next();
        assert!(first >= Duration::from_millis(1600) && first <= Duration::from_millis(2400));
        for _ in 0..20 {
            b.next();
        }
        assert!(b.next() <= BACKOFF_MAX.mul_f64(1.2));
        b.reset();
        assert!(b.next() <= Duration::from_millis(2400));
    }

    #[test]
    fn statuses_read_as_the_ui_expects() {
        let offline = SyncStatus {
            state: SyncState::Offline {
                tailscale: true,
                tried: vec![("http://192.168.1.10:7458".into(), "timed out".into())],
                retry_in_s: 30,
            },
            unsent: 3,
            last_sync_at: None,
        };
        let v = serde_json::to_value(&offline).unwrap();
        assert_eq!(v["state"], "offline");
        assert_eq!(v["unsent"], 3);
        assert_eq!(v["tried"][0][1], "timed out");

        let attention = SyncStatus {
            state: SyncState::Attention {
                attention: Attention::UpdateServer {
                    server: blotter_core::sync::api::Versions {
                        protocol: 1,
                        schema: 0,
                    },
                },
            },
            unsent: 0,
            last_sync_at: Some("2026-10-23T09:00:00Z".into()),
        };
        let v = serde_json::to_value(&attention).unwrap();
        assert_eq!(v["state"], "attention");
        assert_eq!(v["reason"], "update_server");
        assert_eq!(v["server"]["schema"], 0);
    }
}

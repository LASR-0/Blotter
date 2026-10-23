//! Sync with a home server running Blotter Sync (roadmap §14.6).
//!
//! In Local only mode (no server paired) this does nothing. Once paired, a
//! background task syncs: after local writes (debounced), when the server's
//! change stream announces a new seq, every minute as a fallback, and with
//! backoff and jitter while the server can't be reached. Each sync pushes
//! the outbox and pulls from the cursor ([`engine`]); remote changes reach
//! the windows through the usual changed event. The UI reads one status,
//! never a toast per attempt.

pub mod address;
pub mod client;
pub mod engine;
pub mod token;

use std::sync::Mutex;
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
use crate::commands::{CHANGED_EVENT, ChangedPayload};
use client::Http;
use engine::{Attention, Engine, Problem};

/// Tells the UI the sync status changed; it then calls `sync_status`.
pub const SYNC_STATUS_EVENT: &str = "sync:status";

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
    status: Mutex<SyncStatus>,
}

impl Default for SyncEngine {
    fn default() -> Self {
        Self {
            wake: Notify::new(),
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
                if done.pulled > 0 {
                    let _ = app.emit(
                        CHANGED_EVENT,
                        ChangedPayload {
                            batch_id: String::new(),
                            ops: vec![SYNC_OP.into()],
                        },
                    );
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

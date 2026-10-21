//! Settings → Calendars: ICS subscriptions (roadmap §5.8). The core keeps
//! the subscriptions and their cached events; this fetches them, on a
//! minute check for the ones due and whenever the user asks.

use std::collections::HashSet;
use std::sync::Mutex;
use std::time::Duration;

use blotter_core::subscriptions::{self, NewSubscription, SubscriptionInfo, SubscriptionPatch};
use blotter_core::{CoreError, store};
use futures_util::StreamExt;
use serde_json::json;
use tauri::{AppHandle, Emitter, Manager};

use crate::AppState;
use crate::commands::{CHANGED_EVENT, CommandError};

/// How often due subscriptions are looked for.
const CHECK_EVERY: Duration = Duration::from_secs(60);
/// The first check waits a little, so it doesn't slow the start.
const FIRST_CHECK_AFTER: Duration = Duration::from_secs(5);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(60);
/// Largest calendar accepted.
const MAX_BYTES: usize = 20 * 1024 * 1024;

/// Subscriptions being fetched right now, so the minute check and a
/// Refresh click don't fetch the same one twice.
#[derive(Default)]
pub struct Fetching(Mutex<HashSet<String>>);

fn lock_err() -> CommandError {
    CommandError::internal("planner state is unavailable after an earlier crash")
}

/// Tells every window that calendar events or block-outs may have changed.
fn announce(app: &AppHandle) {
    let _ = app.emit(
        CHANGED_EVENT,
        json!({ "batch_id": "", "ops": ["subscriptions"] }),
    );
}

/// Why a fetch failed, in a few words for the status line.
fn describe(err: &reqwest::Error) -> String {
    if err.is_timeout() {
        "the calendar took too long to answer".into()
    } else if err.is_connect() {
        "couldn't connect (check the address and your connection)".into()
    } else {
        let mut text = err.to_string();
        if let Some(url) = err.url() {
            text = text.replace(&format!(" for url ({url})"), "");
        }
        text
    }
}

/// Fetches `url` (an http(s) URL) and returns the body as text.
pub(crate) async fn fetch_body(url: &str) -> Result<String, String> {
    fetch_text(url, "text/calendar, */*;q=0.5").await
}

/// Fetches `url`, asking for `accept`, and returns the body as text.
pub(crate) async fn fetch_text(url: &str, accept: &str) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .connect_timeout(CONNECT_TIMEOUT)
        .timeout(REQUEST_TIMEOUT)
        .user_agent(concat!("Blotter/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client
        .get(url)
        .header(reqwest::header::ACCEPT, accept)
        .send()
        .await
        .map_err(|e| describe(&e))?;
    let status = resp.status();
    if !status.is_success() {
        return Err(match status.canonical_reason() {
            Some(reason) => format!("HTTP {} {reason}", status.as_u16()),
            None => format!("HTTP {}", status.as_u16()),
        });
    }
    let too_big = || {
        format!(
            "the calendar is larger than {} MB",
            MAX_BYTES / (1024 * 1024)
        )
    };
    if resp.content_length().is_some_and(|n| n > MAX_BYTES as u64) {
        return Err(too_big());
    }
    let mut body = Vec::new();
    let mut stream = resp.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| describe(&e))?;
        if body.len() + chunk.len() > MAX_BYTES {
            return Err(too_big());
        }
        body.extend_from_slice(&chunk);
    }
    // Some servers send Latin-1; keep what can be read.
    Ok(String::from_utf8_lossy(&body).into_owned())
}

/// Fetches subscription `id` now and records the outcome. Returns `None`
/// if it is already being fetched.
async fn fetch_now(app: &AppHandle, id: &str) -> Result<Option<SubscriptionInfo>, CommandError> {
    let fetching = app.state::<Fetching>();
    {
        let mut busy = fetching.0.lock().map_err(|_| lock_err())?;
        if !busy.insert(id.to_owned()) {
            return Ok(None);
        }
    }
    let result = fetch_and_record(app, id).await;
    if let Ok(mut busy) = fetching.0.lock() {
        busy.remove(id);
    }
    announce(app);
    result.map(Some)
}

async fn fetch_and_record(app: &AppHandle, id: &str) -> Result<SubscriptionInfo, CommandError> {
    let state = app.state::<AppState>();
    let url = {
        let planner = state.planner.lock().map_err(|_| lock_err())?;
        store::subscriptions::require(planner.db().conn(), id)?.url
    };
    let body = match subscriptions::fetch_url(&url) {
        Ok(fetch) => fetch_body(&fetch).await,
        Err(e) => Err(e.to_string()),
    };
    let mut planner = state.planner.lock().map_err(|_| lock_err())?;
    let tz = planner.settings()?.tz;
    let now = planner.now();
    let conn = planner.db_mut().conn_mut();
    let subscription =
        subscriptions::record_fetch(conn, tz, id, body.as_deref().map_err(Clone::clone), now)?;
    Ok(SubscriptionInfo {
        events: store::subscriptions::count_events(conn, id)?,
        subscription,
    })
}

/// Starts the minute check that refreshes subscriptions when they're due.
pub fn start(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(FIRST_CHECK_AFTER).await;
        loop {
            refresh_due(&app).await;
            tokio::time::sleep(CHECK_EVERY).await;
        }
    });
}

async fn refresh_due(app: &AppHandle) {
    let due = {
        let Some(state) = app.try_state::<AppState>() else {
            return;
        };
        let Ok(planner) = state.planner.lock() else {
            return;
        };
        let now = planner.now();
        subscriptions::due(planner.db().conn(), now).unwrap_or_default()
    };
    for s in due {
        if let Err(err) = fetch_now(app, &s.id).await {
            eprintln!("blotter: couldn't refresh '{}': {}", s.name, err.message);
        }
    }
}

fn info(app: &AppHandle, id: &str) -> Result<SubscriptionInfo, CommandError> {
    let state = app.state::<AppState>();
    let planner = state.planner.lock().map_err(|_| lock_err())?;
    subscriptions::list(planner.db().conn())?
        .into_iter()
        .find(|s| s.subscription.id == id)
        .ok_or_else(|| CoreError::not_found("subscription", id).into())
}

#[tauri::command]
pub fn list_subscriptions(app: AppHandle) -> Result<Vec<SubscriptionInfo>, CommandError> {
    let state = app.state::<AppState>();
    let planner = state.planner.lock().map_err(|_| lock_err())?;
    Ok(subscriptions::list(planner.db().conn())?)
}

/// Subscribes to a calendar and fetches it straight away. A first fetch
/// that fails still leaves the subscription, showing why.
#[tauri::command]
pub async fn add_subscription(
    app: AppHandle,
    subscription: NewSubscription,
) -> Result<SubscriptionInfo, CommandError> {
    let id = {
        let state = app.state::<AppState>();
        let planner = state.planner.lock().map_err(|_| lock_err())?;
        let now = planner.now();
        subscriptions::add(planner.db().conn(), subscription, now)?.id
    };
    announce(&app);
    match fetch_now(&app, &id).await? {
        Some(done) => Ok(done),
        None => info(&app, &id),
    }
}

/// Changes a subscription; a new URL is fetched straight away.
#[tauri::command]
pub async fn update_subscription(
    app: AppHandle,
    id: String,
    patch: SubscriptionPatch,
) -> Result<SubscriptionInfo, CommandError> {
    let refetch = {
        let state = app.state::<AppState>();
        let planner = state.planner.lock().map_err(|_| lock_err())?;
        let now = planner.now();
        let s = subscriptions::update(planner.db().conn(), &id, patch, now)?;
        s.enabled && s.last_fetch_at.is_none()
    };
    announce(&app);
    if refetch && let Some(done) = fetch_now(&app, &id).await? {
        return Ok(done);
    }
    info(&app, &id)
}

#[tauri::command]
pub fn remove_subscription(app: AppHandle, id: String) -> Result<(), CommandError> {
    {
        let state = app.state::<AppState>();
        let planner = state.planner.lock().map_err(|_| lock_err())?;
        subscriptions::remove(planner.db().conn(), &id)?;
    }
    announce(&app);
    Ok(())
}

/// Fetches a subscription now (the Refresh / Retry button).
#[tauri::command]
pub async fn refresh_subscription(
    app: AppHandle,
    id: String,
) -> Result<SubscriptionInfo, CommandError> {
    match fetch_now(&app, &id).await? {
        Some(done) => Ok(done),
        None => info(&app, &id),
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::net::TcpListener;

    use super::*;

    /// Serves one request with `response` on a free local port.
    fn serve_once(response: Vec<u8>) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        std::thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buf = [0u8; 4096];
                let _ = stream.read(&mut buf);
                let _ = stream.write_all(&response);
            }
        });
        format!("http://{addr}/calendar.ics")
    }

    fn http(status: &str, body: &str) -> Vec<u8> {
        format!(
            "HTTP/1.1 {status}\r\nContent-Type: text/calendar\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
        .into_bytes()
    }

    #[tokio::test]
    async fn fetches_a_calendar() {
        let url = serve_once(http("200 OK", "BEGIN:VCALENDAR\r\nEND:VCALENDAR\r\n"));
        assert_eq!(
            fetch_body(&url).await.unwrap(),
            "BEGIN:VCALENDAR\r\nEND:VCALENDAR\r\n"
        );
    }

    #[tokio::test]
    async fn http_errors_are_described() {
        let url = serve_once(http("404 Not Found", "nope"));
        assert_eq!(fetch_body(&url).await.unwrap_err(), "HTTP 404 Not Found");
    }

    #[tokio::test]
    async fn oversized_calendars_are_refused() {
        let head = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            MAX_BYTES + 1
        );
        let url = serve_once(head.into_bytes());
        assert!(
            fetch_body(&url)
                .await
                .unwrap_err()
                .contains("larger than 20 MB")
        );
    }

    #[tokio::test]
    async fn nothing_listening_is_a_connect_error() {
        let port = TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        let err = fetch_body(&format!("http://127.0.0.1:{port}/c.ics"))
            .await
            .unwrap_err();
        assert!(err.starts_with("couldn't connect"), "{err}");
    }

    /// Fetch, cache and slot, end to end: a served calendar marked as a
    /// block-out refuses a flexible task at its busy time.
    #[tokio::test]
    async fn a_fetched_blockout_calendar_blocks_time() {
        use blotter_core::{Actor, Database, Planner};
        use chrono::{Duration, Timelike, Utc};

        let start = (Utc::now() + Duration::days(1))
            .with_hour(10)
            .and_then(|t| t.with_minute(0))
            .and_then(|t| t.with_second(0))
            .unwrap();
        let stamp = |t: chrono::DateTime<Utc>| t.format("%Y%m%dT%H%M%SZ").to_string();
        let ics = [
            "BEGIN:VCALENDAR",
            "VERSION:2.0",
            "X-WR-CALNAME:Work",
            "BEGIN:VEVENT",
            "UID:m1",
            "SUMMARY:Board meeting",
            &format!("DTSTART:{}", stamp(start)),
            &format!("DTEND:{}", stamp(start + Duration::hours(1))),
            "END:VEVENT",
            "END:VCALENDAR",
            "",
        ]
        .join("\r\n");
        let url = serve_once(http("200 OK", &ics));

        let mut planner = Planner::new(Database::open_in_memory().unwrap(), chrono_tz::UTC);
        let now = planner.now();
        let new = NewSubscription {
            url,
            as_blockout: true,
            ..Default::default()
        };
        let s = subscriptions::add(planner.db().conn(), new, now).unwrap();
        let body = fetch_body(&subscriptions::fetch_url(&s.url).unwrap()).await;
        let conn = planner.db_mut().conn_mut();
        let s = subscriptions::record_fetch(
            conn,
            chrono_tz::UTC,
            &s.id,
            body.as_deref().map_err(Clone::clone),
            now,
        )
        .unwrap();
        assert_eq!(
            (s.name.as_str(), s.last_status.as_deref()),
            ("Work", Some("ok"))
        );

        let task = serde_json::json!({
            "title": "Deep work",
            "start": (start + Duration::minutes(30)).format("%Y-%m-%dT%H:%M").to_string(),
            "duration_min": 30,
        });
        let err = planner
            .execute("create_item", task, &Actor::User)
            .unwrap_err();
        assert!(err.to_string().contains("Board meeting"), "{err}");
    }
}

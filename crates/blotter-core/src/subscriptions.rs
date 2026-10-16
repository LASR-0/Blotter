//! ICS subscriptions (roadmap §5.8): read-only calendars fetched from a
//! URL, cached in `ics_events`, optionally acting as block-outs.
//!
//! These are the user's settings, like AI providers: they are managed
//! here, not through the operation registry, so agents can't add URLs for
//! the app to fetch, and they sit outside undo. The app does the HTTP and
//! hands the body (or the failure) to [`record_fetch`].

use chrono_tz::Tz;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::error::{CoreError, Result};
use crate::ics::feed;
use crate::id::new_id;
use crate::model::Subscription;
use crate::store;
use crate::time::Instant;

/// Refresh intervals offered, in minutes. The schema allows any value ≥ 5.
pub const MIN_REFRESH_MIN: u32 = 5;
pub const DEFAULT_REFRESH_MIN: u32 = 60;
/// Longest refresh interval: a week.
pub const MAX_REFRESH_MIN: u32 = 7 * 24 * 60;

/// A subscription as the settings page shows it.
#[derive(Debug, Clone, Serialize)]
pub struct SubscriptionInfo {
    #[serde(flatten)]
    pub subscription: Subscription,
    /// Events in the cache.
    pub events: usize,
}

/// The URL to fetch for what the user pasted: `webcal://` (and `webcals://`)
/// mean https; anything but http(s) is refused.
pub fn fetch_url(url: &str) -> Result<String> {
    let url = url.trim();
    let lower = url.to_ascii_lowercase();
    let rest = ["webcals://", "webcal://"]
        .iter()
        .find_map(|p| lower.starts_with(p).then(|| &url[p.len()..]));
    let fetch = match rest {
        Some(rest) => format!("https://{rest}"),
        None if lower.starts_with("https://") || lower.starts_with("http://") => url.to_owned(),
        None => {
            return Err(CoreError::invalid(
                "a calendar URL starts with webcal://, https:// or http://",
            ));
        }
    };
    let host = fetch
        .split_once("://")
        .map(|(_, r)| r.split(['/', '?', '#']).next().unwrap_or_default())
        .unwrap_or_default();
    if host.is_empty() || url.chars().any(char::is_whitespace) {
        return Err(CoreError::invalid("that doesn't look like a calendar URL"));
    }
    Ok(fetch)
}

fn check_refresh(min: u32) -> Result<u32> {
    if (MIN_REFRESH_MIN..=MAX_REFRESH_MIN).contains(&min) {
        Ok(min)
    } else {
        Err(CoreError::invalid(format!(
            "refresh every {MIN_REFRESH_MIN} minutes to 7 days"
        )))
    }
}

fn check_color(color: Option<String>) -> Result<Option<String>> {
    match color.map(|c| c.trim().to_owned()).filter(|c| !c.is_empty()) {
        Some(c)
            if c.len() == 7
                && c.starts_with('#')
                && c[1..].chars().all(|h| h.is_ascii_hexdigit()) =>
        {
            Ok(Some(c.to_ascii_lowercase()))
        }
        Some(c) => Err(CoreError::invalid(format!("'{c}' isn't a #rrggbb colour"))),
        None => Ok(None),
    }
}

/// A name from the URL until the calendar says its own: the host, e.g.
/// `outlook.office365.com`.
fn name_from_url(url: &str) -> String {
    url.split_once("://")
        .map(|(_, r)| r.split(['/', '?', '#']).next().unwrap_or(r))
        .unwrap_or(url)
        .to_owned()
}

pub fn list(conn: &Connection) -> Result<Vec<SubscriptionInfo>> {
    store::subscriptions::list(conn)?
        .into_iter()
        .map(|s| {
            Ok(SubscriptionInfo {
                events: store::subscriptions::count_events(conn, &s.id)?,
                subscription: s,
            })
        })
        .collect()
}

/// What a new subscription starts with.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct NewSubscription {
    pub url: String,
    pub name: Option<String>,
    pub refresh_min: Option<u32>,
    pub color: Option<String>,
    #[serde(default)]
    pub as_blockout: bool,
}

/// Adds a subscription. It has no events until its first fetch.
pub fn add(conn: &Connection, new: NewSubscription, now: Instant) -> Result<Subscription> {
    fetch_url(&new.url)?;
    let url = new.url.trim().to_owned();
    if store::subscriptions::list(conn)?
        .iter()
        .any(|s| s.url == url)
    {
        return Err(CoreError::invalid(
            "you're already subscribed to that calendar",
        ));
    }
    let name = new
        .name
        .map(|n| n.trim().to_owned())
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| name_from_url(&url));
    let s = Subscription {
        id: new_id(),
        name,
        url,
        refresh_min: check_refresh(new.refresh_min.unwrap_or(DEFAULT_REFRESH_MIN))?,
        color: check_color(new.color)?,
        as_blockout: new.as_blockout,
        enabled: true,
        last_fetch_at: None,
        last_ok_at: None,
        last_status: None,
        last_error: None,
        created_at: now,
        updated_at: now,
    };
    store::subscriptions::upsert(conn, &s)?;
    Ok(s)
}

/// Changes to a subscription; fields left out stay as they are. A new URL
/// clears the cache and the fetch status, so it is fetched again.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct SubscriptionPatch {
    pub name: Option<String>,
    pub url: Option<String>,
    pub refresh_min: Option<u32>,
    /// `Some(None)` clears it.
    #[serde(default, with = "double_option")]
    pub color: Option<Option<String>>,
    pub as_blockout: Option<bool>,
    pub enabled: Option<bool>,
}

mod double_option {
    use serde::{Deserialize, Deserializer};

    pub fn deserialize<'de, D: Deserializer<'de>>(
        d: D,
    ) -> Result<Option<Option<String>>, D::Error> {
        Option::<String>::deserialize(d).map(Some)
    }
}

pub fn update(
    conn: &Connection,
    id: &str,
    patch: SubscriptionPatch,
    now: Instant,
) -> Result<Subscription> {
    let mut s = store::subscriptions::require(conn, id)?;
    if let Some(name) = patch.name {
        let name = name.trim();
        if name.is_empty() {
            return Err(CoreError::invalid("give the calendar a name"));
        }
        s.name = name.to_owned();
    }
    if let Some(url) = patch
        .url
        .map(|u| u.trim().to_owned())
        .filter(|u| *u != s.url)
    {
        fetch_url(&url)?;
        s.url = url;
        s.last_fetch_at = None;
        s.last_ok_at = None;
        s.last_status = None;
        s.last_error = None;
        store::subscriptions::replace_events(conn, &s.id, &[])?;
    }
    if let Some(min) = patch.refresh_min {
        s.refresh_min = check_refresh(min)?;
    }
    if let Some(color) = patch.color {
        s.color = check_color(color)?;
    }
    if let Some(b) = patch.as_blockout {
        s.as_blockout = b;
    }
    if let Some(e) = patch.enabled {
        s.enabled = e;
    }
    s.updated_at = now;
    store::subscriptions::upsert(conn, &s)?;
    Ok(s)
}

/// Removes a subscription and its cached events.
pub fn remove(conn: &Connection, id: &str) -> Result<()> {
    if store::subscriptions::delete(conn, id)? {
        Ok(())
    } else {
        Err(CoreError::not_found("subscription", id))
    }
}

/// Subscriptions due a fetch at `now`.
pub fn due(conn: &Connection, now: Instant) -> Result<Vec<Subscription>> {
    Ok(store::subscriptions::list(conn)?
        .into_iter()
        .filter(|s| s.is_due(now))
        .collect())
}

/// Records a fetch of `id` made at `now`: the body on success, or why it
/// failed. A body that isn't a calendar counts as a failure. On failure the
/// previous events are kept, so a calendar that is briefly down still
/// blocks time. Returns the updated subscription.
pub fn record_fetch(
    conn: &mut Connection,
    tz: Tz,
    id: &str,
    body: std::result::Result<&str, String>,
    now: Instant,
) -> Result<Subscription> {
    let mut s = store::subscriptions::require(conn, id)?;
    s.last_fetch_at = Some(now);
    let parsed = body.and_then(|text| {
        feed::read(&s.id, tz, text, now).map_err(|e| match e {
            CoreError::Invalid(_) => "that address didn't return a calendar".to_owned(),
            other => other.to_string(),
        })
    });
    let tx = conn.transaction()?;
    match parsed {
        Ok(feed) => {
            store::subscriptions::replace_events(&tx, &s.id, &feed.events)?;
            s.last_ok_at = Some(now);
            s.last_status = Some("ok".into());
            s.last_error = (!feed.warnings.is_empty()).then(|| feed.warnings.join(" "));
            // A name taken from the URL gives way to the calendar's own.
            if let Some(name) = feed.name
                && s.name == name_from_url(&s.url)
            {
                s.name = name;
            }
        }
        Err(why) => {
            s.last_status = Some("error".into());
            s.last_error = Some(why);
        }
    }
    store::subscriptions::upsert(&tx, &s)?;
    tx.commit()?;
    Ok(s)
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, TimeZone, Utc};
    use chrono_tz::Australia::Melbourne;

    use super::*;
    use crate::Database;

    fn now() -> Instant {
        Utc.with_ymd_and_hms(2026, 10, 14, 0, 0, 0).unwrap()
    }

    fn new(url: &str) -> NewSubscription {
        NewSubscription {
            url: url.into(),
            ..Default::default()
        }
    }

    #[test]
    fn urls() {
        assert_eq!(
            fetch_url(" webcal://outlook.office365.com/owa/calendar/x/reachcalendar.ics ").unwrap(),
            "https://outlook.office365.com/owa/calendar/x/reachcalendar.ics"
        );
        assert_eq!(
            fetch_url("WEBCALS://a.example/c.ics").unwrap(),
            "https://a.example/c.ics"
        );
        assert_eq!(
            fetch_url("http://localhost:8080/c.ics").unwrap(),
            "http://localhost:8080/c.ics"
        );
        for bad in [
            "file:///etc/passwd",
            "ftp://x/y.ics",
            "outlook.com/c.ics",
            "https://",
            "https://a b/c",
        ] {
            assert!(fetch_url(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn add_update_remove() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.conn();
        let s = add(conn, new("webcal://outlook.office365.com/owa/c.ics"), now()).unwrap();
        assert_eq!(s.name, "outlook.office365.com");
        assert_eq!(s.refresh_min, DEFAULT_REFRESH_MIN);
        assert!(s.is_due(now()), "never fetched");
        let again = add(conn, new("webcal://outlook.office365.com/owa/c.ics"), now());
        assert!(
            again
                .unwrap_err()
                .to_string()
                .contains("already subscribed")
        );

        let patch = SubscriptionPatch {
            refresh_min: Some(30),
            as_blockout: Some(true),
            color: Some(Some("#AA00CC".into())),
            ..Default::default()
        };
        let s = update(conn, &s.id, patch, now()).unwrap();
        assert_eq!(
            (s.refresh_min, s.as_blockout, s.color.as_deref()),
            (30, true, Some("#aa00cc"))
        );
        let bad = SubscriptionPatch {
            refresh_min: Some(1),
            ..Default::default()
        };
        assert!(update(conn, &s.id, bad, now()).is_err());

        remove(conn, &s.id).unwrap();
        assert!(list(conn).unwrap().is_empty());
        assert!(remove(conn, &s.id).is_err());
    }

    #[test]
    fn fetches_fill_the_cache_and_failures_keep_it() {
        let mut db = Database::open_in_memory().unwrap();
        let s = add(
            db.conn(),
            new("https://outlook.office365.com/owa/c.ics"),
            now(),
        )
        .unwrap();
        let ok = record_fetch(
            db.conn_mut(),
            Melbourne,
            &s.id,
            Ok(feed::tests::OUTLOOK),
            now(),
        )
        .unwrap();
        assert_eq!(ok.last_status.as_deref(), Some("ok"));
        assert_eq!(
            ok.name, "Calendar",
            "the calendar's own name replaces the host"
        );
        assert_eq!(ok.last_ok_at, Some(now()));
        let cached = store::subscriptions::count_events(db.conn(), &s.id).unwrap();
        assert!(cached > 20);
        assert!(!ok.is_due(now() + Duration::minutes(59)));
        assert!(ok.is_due(now() + Duration::minutes(60)));

        let later = now() + Duration::hours(1);
        let failed = record_fetch(
            db.conn_mut(),
            Melbourne,
            &s.id,
            Err("HTTP 404".into()),
            later,
        )
        .unwrap();
        assert_eq!(failed.last_status.as_deref(), Some("error"));
        assert_eq!(failed.last_error.as_deref(), Some("HTTP 404"));
        assert_eq!(failed.last_ok_at, Some(now()));
        assert_eq!(
            store::subscriptions::count_events(db.conn(), &s.id).unwrap(),
            cached
        );

        let html =
            record_fetch(db.conn_mut(), Melbourne, &s.id, Ok("<html></html>"), later).unwrap();
        assert_eq!(
            html.last_error.as_deref(),
            Some("that address didn't return a calendar")
        );

        // A user-given name stays; a new URL empties the cache.
        let patch = SubscriptionPatch {
            name: Some("Work".into()),
            url: Some("https://example.com/other.ics".into()),
            ..Default::default()
        };
        let moved = update(db.conn(), &s.id, patch, later).unwrap();
        assert_eq!(
            (moved.name.as_str(), moved.last_status.as_deref()),
            ("Work", None)
        );
        assert_eq!(
            store::subscriptions::count_events(db.conn(), &s.id).unwrap(),
            0
        );
        let ok = record_fetch(
            db.conn_mut(),
            Melbourne,
            &s.id,
            Ok(feed::tests::OUTLOOK),
            later,
        )
        .unwrap();
        assert_eq!(ok.name, "Work");
    }

    #[test]
    fn a_blockout_calendar_keeps_flexible_tasks_out() {
        use serde_json::{Value, json};

        use crate::Actor;

        // The planner's clock is 09:00 Wednesday 14 Oct in Melbourne; the
        // feed moves that day's Team sync to 14:00–15:00.
        let mut p = crate::planner::tests::planner();
        let now = p.now();
        let s = add(
            p.db().conn(),
            new("https://outlook.office365.com/c.ics"),
            now,
        )
        .unwrap();
        record_fetch(
            p.db_mut().conn_mut(),
            Melbourne,
            &s.id,
            Ok(feed::tests::OUTLOOK),
            now,
        )
        .unwrap();
        let run = |p: &mut crate::Planner, op: &str, args: Value| {
            p.execute(op, args, &Actor::User).map(|o| o.result)
        };
        let first_slot = |p: &mut crate::Planner| {
            let out = p
                .execute(
                    "find_free_slots",
                    json!({ "duration_min": 30, "from": "2026-10-14T14:00", "until": "2026-10-14T16:00" }),
                    &Actor::User,
                )
                .unwrap()
                .result;
            out["slots"][0]["start"].as_str().unwrap().to_owned()
        };
        let task =
            json!({ "title": "Write report", "start": "2026-10-14T14:15", "duration_min": 30 });

        // Shown, but not blocking: the calendar isn't a block-out yet.
        let events = run(
            &mut p,
            "list_calendar_events",
            json!({ "start": "2026-10-14" }),
        )
        .unwrap();
        assert_eq!(events["events"].as_array().unwrap().len(), 1);
        assert_eq!(events["calendars"][0]["name"], "Calendar");
        assert_eq!(first_slot(&mut p), "2026-10-14T03:00:00Z", "14:00");

        let on = SubscriptionPatch {
            as_blockout: Some(true),
            ..Default::default()
        };
        update(p.db().conn(), &s.id, on, now).unwrap();
        assert_eq!(
            first_slot(&mut p),
            "2026-10-14T04:00:00Z",
            "15:00, after the meeting"
        );
        let err = run(&mut p, "create_item", task.clone())
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("'Team sync (moved)'") && err.contains("from Calendar"),
            "{err}"
        );
        let windows = run(
            &mut p,
            "list_blockout_windows",
            json!({ "start": "2026-10-14" }),
        )
        .unwrap();
        let w = &windows["windows"][0];
        assert_eq!(
            (w["label"].as_str(), w["calendar"].as_str()),
            (Some("Team sync (moved)"), Some("Calendar"))
        );
        assert_eq!(w["date"], "2026-10-14");

        // Fixed items may sit in it, without a conflict.
        let mut fixed = task.clone();
        fixed["anchoring"] = json!("fixed");
        run(&mut p, "create_item", fixed).unwrap();
        let agenda = run(&mut p, "list_agenda", json!({ "start": "2026-10-14" })).unwrap();
        assert!(agenda["conflicts"].as_array().unwrap().is_empty());

        // Free events never block, and a disabled calendar doesn't either.
        assert!(
            run(
                &mut p,
                "list_blockout_windows",
                json!({ "start": "2026-10-15" })
            )
            .unwrap()["windows"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        let off = SubscriptionPatch {
            enabled: Some(false),
            ..Default::default()
        };
        update(p.db().conn(), &s.id, off, now).unwrap();
        run(&mut p, "create_item", task).unwrap();
    }
}

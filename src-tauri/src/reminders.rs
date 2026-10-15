//! The reminder engine (roadmap §4.7): one thread that sleeps until the
//! next reminder is due, or until the planner changes, then shows an OS
//! notification. The process stays in the tray, so reminders fire with
//! the window closed. Fired state lives in the database, so a restart
//! never fires one twice.
//!
//! Each check covers the time since the previous one, which is saved. A
//! reminder found more than [`GRACE`] late was missed (Blotter was closed,
//! or the computer asleep); those within the catch-up lookback are
//! grouped into one notification instead of firing one by one.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration as StdDuration;

use blotter_core::alerts::{self, Due};
use blotter_core::settings::{KEY_LAST_REMINDER_CHECK, NotifySettings};
use blotter_core::store;
use blotter_core::time::{self, Instant};
use chrono::{Duration, Utc};
use chrono_tz::Tz;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Listener, Manager, State};
use tauri_plugin_notification::NotificationExt;

use crate::AppState;
use crate::commands::{CHANGED_EVENT, CommandError};

/// Sent to the UI with each [`Due`] that fires on time.
pub const FIRED_EVENT: &str = "reminder:fired";
/// Sent to the UI with a [`Missed`] group.
pub const MISSED_EVENT: &str = "reminder:missed";

/// Reminders this far overdue still fire normally (e.g. the app was busy
/// or just starting). Older ones are missed and caught up as a group.
const GRACE: Duration = Duration::minutes(2);
/// How far ahead to look for the next reminder.
const HORIZON: Duration = Duration::days(1);
/// Longest sleep, so clock changes and sleep/resume are noticed.
const MAX_SLEEP: StdDuration = StdDuration::from_secs(60);
/// The furthest back a check ever looks, whatever was saved.
const MAX_LOOKBACK: Duration = Duration::days(7);

/// Reminders that came due while nothing could show them.
#[derive(Debug, Clone, Serialize)]
pub struct Missed {
    /// The last check before the gap.
    pub since: Instant,
    /// Closed (the first check after starting) rather than asleep.
    pub while_closed: bool,
    pub items: Vec<Due>,
}

/// What the UI hasn't shown yet. Firings at startup happen before the
/// window loads, so they wait here until the UI says it's listening.
#[derive(Default)]
pub struct ReminderNotices {
    ui_ready: AtomicBool,
    pending: Mutex<Pending>,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct Pending {
    pub fired: Vec<Due>,
    pub missed: Vec<Missed>,
}

impl ReminderNotices {
    fn deliver_fired(&self, app: &AppHandle, due: &Due) {
        if self.ui_ready.load(Ordering::Acquire) {
            let _ = app.emit(FIRED_EVENT, due);
        } else if let Ok(mut p) = self.pending.lock() {
            p.fired.push(due.clone());
        }
    }

    fn deliver_missed(&self, app: &AppHandle, missed: &Missed) {
        if self.ui_ready.load(Ordering::Acquire) {
            let _ = app.emit(MISSED_EVENT, missed);
        } else if let Ok(mut p) = self.pending.lock() {
            p.missed.push(missed.clone());
        }
    }
}

/// Called once the UI listens for reminder events: returns (and clears)
/// what fired before then; later firings arrive as events.
#[tauri::command]
pub fn take_reminder_notices(notices: State<'_, ReminderNotices>) -> Pending {
    notices.ui_ready.store(true, Ordering::Release);
    notices
        .pending
        .lock()
        .map(|mut p| std::mem::take(&mut *p))
        .unwrap_or_default()
}

/// Shows a sample notification, so the user can check their OS settings.
#[tauri::command]
pub fn send_test_notification(app: AppHandle) -> Result<(), CommandError> {
    app.notification()
        .builder()
        .title("Blotter")
        .body("Notifications are working. Reminders will look like this.")
        .show()
        .map_err(|err| CommandError::internal(format!("couldn't show a notification: {err}")))
}

/// Lets planner changes cut the engine's sleep short.
#[derive(Default)]
struct Wake {
    rung: Mutex<bool>,
    cv: Condvar,
}

impl Wake {
    fn ring(&self) {
        if let Ok(mut rung) = self.rung.lock() {
            *rung = true;
            self.cv.notify_one();
        }
    }

    fn wait(&self, timeout: StdDuration) {
        let Ok(rung) = self.rung.lock() else {
            thread::sleep(timeout);
            return;
        };
        if let Ok((mut rung, _)) = self.cv.wait_timeout_while(rung, timeout, |r| !*r) {
            *rung = false;
        }
    }
}

/// Where the engine is up to.
struct Cursor {
    /// The end of the last window checked.
    last: Instant,
    /// Whether `last` was saved by an earlier run (so a gap means closed).
    from_previous_run: bool,
}

pub fn start(app: AppHandle) {
    app.manage(ReminderNotices::default());
    let wake = Arc::new(Wake::default());
    let ringer = Arc::clone(&wake);
    app.listen(CHANGED_EVENT, move |_| ringer.ring());
    let spawned = thread::Builder::new()
        .name("reminders".into())
        .spawn(move || {
            let mut cursor = initial_cursor(&app);
            loop {
                let sleep = tick(&app, &mut cursor);
                wake.wait(sleep);
            }
        });
    if let Err(err) = spawned {
        eprintln!("blotter: could not start the reminder engine: {err}");
    }
}

/// Resumes from the saved check, or starts fresh (no catch-up) the first
/// time Blotter runs.
fn initial_cursor(app: &AppHandle) -> Cursor {
    let now = Utc::now();
    let saved = app.try_state::<AppState>().and_then(|state| {
        let planner = state.planner.lock().ok()?;
        let raw: String = store::settings::get(planner.db().conn(), KEY_LAST_REMINDER_CHECK)
            .ok()
            .flatten()?;
        time::parse_instant(KEY_LAST_REMINDER_CHECK, &raw).ok()
    });
    match saved {
        Some(at) => Cursor {
            last: at.max(now - MAX_LOOKBACK).min(now),
            from_previous_run: true,
        },
        None => Cursor {
            last: now - GRACE,
            from_previous_run: false,
        },
    }
}

/// Splits what came due in a check into on-time firings and missed ones
/// worth catching up (older ones are dropped quietly).
fn partition(due: Vec<Due>, now: Instant, notify: &NotifySettings) -> (Vec<Due>, Vec<Due>) {
    let late = now - GRACE;
    let oldest = now - Duration::hours(i64::from(notify.catch_up_hours));
    let (on_time, late): (Vec<Due>, Vec<Due>) = due.into_iter().partition(|d| d.due > late);
    let missed = if notify.catch_up_enabled {
        late.into_iter().filter(|d| d.due > oldest).collect()
    } else {
        Vec::new()
    };
    (on_time, missed)
}

/// Fires what's due and returns how long to sleep until the next one.
fn tick(app: &AppHandle, cursor: &mut Cursor) -> StdDuration {
    let Some(state) = app.try_state::<AppState>() else {
        return MAX_SLEEP;
    };
    let Some(notices) = app.try_state::<ReminderNotices>() else {
        return MAX_SLEEP;
    };
    let Ok(planner) = state.planner.lock() else {
        return MAX_SLEEP;
    };
    let tz = match planner.settings() {
        Ok(s) => s.tz,
        Err(err) => {
            eprintln!("blotter: reminders can't read settings: {err}");
            return MAX_SLEEP;
        }
    };
    let conn = planner.db().conn();
    let notify = NotifySettings::load(conn).unwrap_or_else(|err| {
        eprintln!("blotter: reminders can't read notification settings: {err}");
        NotifySettings {
            enabled: true,
            snooze_options_min: Vec::new(),
            catch_up_enabled: false,
            catch_up_hours: 0,
        }
    });
    let now = Utc::now();
    match alerts::due_between(conn, tz, cursor.last, now) {
        Ok(due) => {
            for d in &due {
                if let Err(err) = alerts::mark_fired(conn, d) {
                    eprintln!("blotter: couldn't record a fired reminder: {err}");
                }
            }
            let (on_time, missed) = partition(due, now, &notify);
            for d in &on_time {
                if notify.enabled {
                    show(app, &d.title, &body(d, tz, now));
                }
                notices.deliver_fired(app, d);
            }
            if !missed.is_empty() {
                let group = Missed {
                    since: cursor.last,
                    while_closed: cursor.from_previous_run,
                    items: missed,
                };
                if notify.enabled {
                    let (title, text) = missed_text(&group, tz);
                    show(app, &title, &text);
                }
                notices.deliver_missed(app, &group);
            }
            cursor.last = now;
            cursor.from_previous_run = false;
            let saved = time::fmt_instant(time::truncate(now));
            if let Err(err) = store::settings::set(conn, KEY_LAST_REMINDER_CHECK, &saved, now) {
                eprintln!("blotter: couldn't save the reminder check: {err}");
            }
        }
        Err(err) => eprintln!("blotter: reminder check failed: {err}"),
    }
    let next = alerts::due_between(conn, tz, now, now + HORIZON)
        .ok()
        .and_then(|due| due.first().map(|d| d.due));
    next.and_then(|at| (at - Utc::now()).to_std().ok())
        // Wake a moment after it's due so it falls inside the window.
        .map(|d| d + StdDuration::from_millis(250))
        .unwrap_or(MAX_SLEEP)
        .min(MAX_SLEEP)
}

/// "15:00–15:45 · in 10 min", "All day", "Now"; snoozed ones say so.
pub(crate) fn body(d: &Due, tz: Tz, now: Instant) -> String {
    let text = when(d, tz, now);
    if d.snoozed {
        format!("{text} · snoozed")
    } else {
        text
    }
}

fn when(d: &Due, tz: Tz, now: Instant) -> String {
    let Some(start) = d.start else {
        return "Reminder".to_owned();
    };
    if d.all_day {
        return "All day".to_owned();
    }
    let local = |t: Instant| t.with_timezone(&tz).format("%H:%M").to_string();
    let when = match d.end {
        Some(end) if end > start => format!("{}–{}", local(start), local(end)),
        _ => local(start),
    };
    let minutes = (start - now).num_minutes();
    let rel = match minutes {
        m if m > 1 => format!("in {m} min"),
        m if m >= -1 => "now".to_owned(),
        m => format!("started {} min ago", -m),
    };
    format!("{when} · {rel}")
}

/// Title and body of the grouped catch-up notification.
pub(crate) fn missed_text(m: &Missed, tz: Tz) -> (String, String) {
    let n = m.items.len();
    let what = if n == 1 {
        "1 reminder".to_owned()
    } else {
        format!("{n} reminders")
    };
    let why = if m.while_closed {
        "while Blotter was closed"
    } else {
        "while your computer was asleep"
    };
    const LISTED: usize = 3;
    let mut list: Vec<String> = m
        .items
        .iter()
        .take(LISTED)
        .map(|d| {
            let at = d.due.with_timezone(&tz).format("%H:%M");
            format!("{} {at}", d.title)
        })
        .collect();
    if n > LISTED {
        list.push(format!("+{} more", n - LISTED));
    }
    (format!("{what} missed {why}"), list.join(" · "))
}

fn show(app: &AppHandle, title: &str, body: &str) {
    let shown = app.notification().builder().title(title).body(body).show();
    if let Err(err) = shown {
        eprintln!("blotter: couldn't show a notification: {err}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    const TZ: Tz = chrono_tz::Australia::Melbourne;

    fn t(h: u32, m: u32) -> Instant {
        Utc.with_ymd_and_hms(2026, 10, 15, h, m, 0).unwrap()
    }

    fn due_at(title: &str, at: Instant) -> Due {
        Due {
            reminder_id: "r".into(),
            item_id: "i".into(),
            title: title.into(),
            start: Some(at),
            end: None,
            all_day: false,
            due: at,
            snoozed: false,
        }
    }

    fn due(start: Option<(u32, u32)>, end: Option<(u32, u32)>, all_day: bool) -> Due {
        Due {
            start: start.map(|(h, m)| t(h, m)),
            end: end.map(|(h, m)| t(h, m)),
            all_day,
            ..due_at("Dentist", t(3, 50))
        }
    }

    #[test]
    fn notification_body() {
        let now = t(3, 50);
        assert_eq!(
            body(&due(Some((4, 0)), Some((4, 45)), false), TZ, now),
            "15:00–15:45 · in 10 min"
        );
        assert_eq!(
            body(&due(Some((3, 50)), None, false), TZ, now),
            "14:50 · now"
        );
        assert_eq!(
            body(&due(Some((3, 0)), None, false), TZ, now),
            "14:00 · started 50 min ago"
        );
        assert_eq!(body(&due(Some((0, 0)), None, true), TZ, now), "All day");
        assert_eq!(body(&due(None, None, false), TZ, now), "Reminder");
        let snoozed = Due {
            snoozed: true,
            ..due(Some((3, 50)), None, false)
        };
        assert_eq!(body(&snoozed, TZ, now), "14:50 · now · snoozed");
    }

    #[test]
    fn late_reminders_are_caught_up_within_the_lookback() {
        let now = t(10, 0);
        let notify = NotifySettings {
            enabled: true,
            snooze_options_min: vec![5],
            catch_up_enabled: true,
            catch_up_hours: 6,
        };
        let all = vec![
            due_at("Too old", t(3, 0)),
            due_at("Missed", t(8, 0)),
            due_at("Just late", t(9, 59)),
        ];
        let (on_time, missed) = partition(all.clone(), now, &notify);
        assert_eq!(on_time.len(), 1);
        assert_eq!(on_time[0].title, "Just late", "within the grace period");
        assert_eq!(missed.len(), 1);
        assert_eq!(
            missed[0].title, "Missed",
            "older than the lookback is dropped"
        );

        let off = NotifySettings {
            catch_up_enabled: false,
            ..notify
        };
        assert!(partition(all, now, &off).1.is_empty());
    }

    #[test]
    fn missed_notification_text() {
        let group = |n: usize, while_closed| Missed {
            since: t(0, 0),
            while_closed,
            items: (0..n)
                .map(|i| due_at(&format!("R{i}"), t(1, i as u32)))
                .collect(),
        };
        assert_eq!(
            missed_text(&group(1, true), TZ),
            (
                "1 reminder missed while Blotter was closed".to_owned(),
                "R0 12:00".to_owned()
            )
        );
        let (title, text) = missed_text(&group(5, false), TZ);
        assert_eq!(title, "5 reminders missed while your computer was asleep");
        assert_eq!(text, "R0 12:00 · R1 12:01 · R2 12:02 · +2 more");
    }
}

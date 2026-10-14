//! The reminder engine (roadmap §4.7): one thread that sleeps until the
//! next reminder is due, or until the planner changes, then shows an OS
//! notification. The process stays in the tray, so reminders fire with
//! the window closed. Fired state lives in the database, so a restart
//! never fires one twice.

use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration as StdDuration;

use blotter_core::alerts::{self, Due};
use chrono::{Duration, Utc};
use chrono_tz::Tz;
use tauri::{AppHandle, Emitter, Listener, Manager};
use tauri_plugin_notification::NotificationExt;

use crate::AppState;
use crate::commands::CHANGED_EVENT;

/// Sent to the UI with each [`Due`] that fires.
pub const FIRED_EVENT: &str = "reminder:fired";

/// Reminders this far overdue still fire normally (e.g. the app was busy
/// or just starting). Older ones missed while closed are caught up
/// separately.
const GRACE: Duration = Duration::minutes(2);
/// How far ahead to look for the next reminder.
const HORIZON: Duration = Duration::days(1);
/// Longest sleep, so clock changes and sleep/resume are noticed.
const MAX_SLEEP: StdDuration = StdDuration::from_secs(60);

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

pub fn start(app: AppHandle) {
    let wake = Arc::new(Wake::default());
    let ringer = Arc::clone(&wake);
    app.listen(CHANGED_EVENT, move |_| ringer.ring());
    let spawned = thread::Builder::new()
        .name("reminders".into())
        .spawn(move || {
            loop {
                let sleep = tick(&app);
                wake.wait(sleep);
            }
        });
    if let Err(err) = spawned {
        eprintln!("blotter: could not start the reminder engine: {err}");
    }
}

/// Fires what's due and returns how long to sleep until the next one.
fn tick(app: &AppHandle) -> StdDuration {
    let Some(state) = app.try_state::<AppState>() else {
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
    let now = Utc::now();
    match alerts::due_between(conn, tz, now - GRACE, now) {
        Ok(due) => {
            for d in due {
                notify(app, &d, tz);
                if let Err(err) = alerts::mark_fired(conn, &d) {
                    eprintln!("blotter: couldn't record a fired reminder: {err}");
                }
                let _ = app.emit(FIRED_EVENT, &d);
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

/// "15:00–15:45 · in 10 min", "All day", "Now".
pub(crate) fn body(d: &Due, tz: Tz, now: chrono::DateTime<Utc>) -> String {
    let Some(start) = d.start else {
        return "Reminder".to_owned();
    };
    if d.all_day {
        return "All day".to_owned();
    }
    let local = |t: chrono::DateTime<Utc>| t.with_timezone(&tz).format("%H:%M").to_string();
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

fn notify(app: &AppHandle, d: &Due, tz: Tz) {
    let shown = app
        .notification()
        .builder()
        .title(&d.title)
        .body(body(d, tz, Utc::now()))
        .show();
    if let Err(err) = shown {
        eprintln!("blotter: couldn't show a notification: {err}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn due(start: Option<(u32, u32)>, end: Option<(u32, u32)>, all_day: bool) -> Due {
        let t = |(h, m): (u32, u32)| Utc.with_ymd_and_hms(2026, 10, 15, h, m, 0).unwrap();
        Due {
            reminder_id: "r".into(),
            item_id: "i".into(),
            title: "Dentist".into(),
            start: start.map(t),
            end: end.map(t),
            all_day,
            due: Utc.with_ymd_and_hms(2026, 10, 15, 3, 50, 0).unwrap(),
            snoozed: false,
        }
    }

    #[test]
    fn notification_body() {
        let tz = chrono_tz::Australia::Melbourne;
        let now = Utc.with_ymd_and_hms(2026, 10, 15, 3, 50, 0).unwrap();
        assert_eq!(
            body(&due(Some((4, 0)), Some((4, 45)), false), tz, now),
            "15:00–15:45 · in 10 min"
        );
        assert_eq!(
            body(&due(Some((3, 50)), None, false), tz, now),
            "14:50 · now"
        );
        assert_eq!(
            body(&due(Some((3, 0)), None, false), tz, now),
            "14:00 · started 50 min ago"
        );
        assert_eq!(body(&due(Some((0, 0)), None, true), tz, now), "All day");
        assert_eq!(body(&due(None, None, false), tz, now), "Reminder");
    }
}

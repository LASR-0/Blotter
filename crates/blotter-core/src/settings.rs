//! Planner settings the core needs to interpret operations. Stored as JSON
//! in the `settings` table; missing keys fall back to defaults.

use chrono_tz::Tz;
use rusqlite::Connection;
use serde::Serialize;

use crate::error::Result;
use crate::slots::PlanningHours;
use crate::store;

pub const KEY_TIMEZONE: &str = "timezone";
pub const KEY_DEFAULT_TASK_DURATION: &str = "default_task_duration_min";
pub const KEY_DEFAULT_EVENT_DURATION: &str = "default_event_duration_min";
pub const KEY_PLANNING_HOURS: &str = "planning_hours";
pub const KEY_SLOT_BUFFER: &str = "slot_buffer_min";
/// Minutes before an event its default reminder fires (0 = none; default 10).
pub const KEY_DEFAULT_EVENT_REMINDER: &str = "default_event_reminder_min";
/// `"inbox"` (default) or `"reslot"`: where unfinished tasks from earlier days go.
pub const KEY_ROLLOVER_MODE: &str = "rollover_mode";
/// Local `HH:MM` after which the day's rollover may run (default 04:00).
pub const KEY_ROLLOVER_TIME: &str = "rollover_time";
/// The local date the last automatic rollover ran on.
pub const KEY_LAST_ROLLOVER: &str = "last_rollover_date";
/// Whether reminders show OS notifications (default on; in-app toasts always show).
pub const KEY_NOTIFICATIONS: &str = "notifications_enabled";
/// The snooze lengths offered on a reminder, in minutes (default 5, 15, 60).
pub const KEY_SNOOZE_OPTIONS: &str = "snooze_options_min";
/// Whether reminders missed while closed or asleep are caught up (default on).
pub const KEY_CATCH_UP: &str = "catch_up_enabled";
/// How far back the catch-up looks, in hours (default 24).
pub const KEY_CATCH_UP_HOURS: &str = "catch_up_hours";
/// When the reminder engine last checked (internal; the catch-up starts here).
pub const KEY_LAST_REMINDER_CHECK: &str = "last_reminder_check";
/// Automatic snapshots kept (default 20); manual ones are always kept.
pub const KEY_SNAPSHOT_KEEP: &str = "snapshot_keep";
/// Where snapshots are written (default: a `snapshots` folder by the database).
pub const KEY_SNAPSHOT_DIR: &str = "snapshot_dir";
/// `"propose"` (default, D8) or `"auto"`: whether AI and MCP changes wait for approval.
pub const KEY_AI_MODE: &str = "ai_mode";
/// Which AI the chat uses: `"claude_code"` (default, D29) or `"api"` (the default provider).
pub const KEY_AI_ROUTE: &str = "ai_route";
/// The model Claude Code runs, as an alias (`sonnet`, `opus`, …) or full name.
pub const KEY_CLAUDE_CODE_MODEL: &str = "claude_code_model";
/// Where the `claude` program is, when it isn't found on its own.
pub const KEY_CLAUDE_CODE_PATH: &str = "claude_code_path";

/// Settings that describe this install rather than the planner: they stay
/// put through a restore or import, and aren't exported.
pub const DEVICE_KEYS: [&str; 8] = [
    KEY_AI_MODE,
    KEY_LAST_REMINDER_CHECK,
    KEY_LAST_ROLLOVER,
    KEY_SNAPSHOT_DIR,
    KEY_AI_ROUTE,
    KEY_CLAUDE_CODE_MODEL,
    KEY_CLAUDE_CODE_PATH,
    "theme",
];
pub const DEFAULT_SNAPSHOT_KEEP: u32 = 20;

#[derive(Debug, Clone)]
pub struct PlannerSettings {
    /// The user's zone. Naive times in operation arguments and day
    /// boundaries are interpreted here.
    pub tz: Tz,
    /// Applied when a task is scheduled without a duration.
    pub default_task_duration_min: u32,
    /// Applied when an event is created without an end or duration.
    pub default_event_duration_min: u32,
    /// When flexible tasks may be placed (default 07:00–22:00 every day).
    pub planning_hours: PlanningHours,
    /// Minutes kept free before and after scheduled items when slotting.
    pub slot_buffer_min: u32,
    /// The reminder new events get unless told otherwise (`None` = none).
    pub default_event_reminder_min: Option<u32>,
}

impl PlannerSettings {
    /// Loads settings, falling back to `default_tz` (the OS zone, supplied
    /// by the app) when the user hasn't chosen one or it no longer parses.
    pub fn load(conn: &Connection, default_tz: Tz) -> Result<Self> {
        let tz = store::settings::get::<String>(conn, KEY_TIMEZONE)?
            .and_then(|name| name.parse::<Tz>().ok())
            .unwrap_or(default_tz);
        Ok(Self {
            tz,
            default_task_duration_min: store::settings::get(conn, KEY_DEFAULT_TASK_DURATION)?
                .unwrap_or(30),
            default_event_duration_min: store::settings::get(conn, KEY_DEFAULT_EVENT_DURATION)?
                .unwrap_or(60),
            planning_hours: store::settings::get(conn, KEY_PLANNING_HOURS)?.unwrap_or_default(),
            slot_buffer_min: store::settings::get(conn, KEY_SLOT_BUFFER)?.unwrap_or(0),
            default_event_reminder_min: Some(
                store::settings::get(conn, KEY_DEFAULT_EVENT_REMINDER)?.unwrap_or(10u32),
            )
            .filter(|m| *m > 0),
        })
    }
}

pub const DEFAULT_SNOOZE_OPTIONS: [u32; 3] = [5, 15, 60];
pub const DEFAULT_CATCH_UP_HOURS: u32 = 24;

/// How reminders are delivered (roadmap §4.7, §5.6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NotifySettings {
    /// OS notifications on or off.
    pub enabled: bool,
    pub snooze_options_min: Vec<u32>,
    pub catch_up_enabled: bool,
    pub catch_up_hours: u32,
}

impl NotifySettings {
    pub fn load(conn: &Connection) -> Result<Self> {
        Ok(Self {
            enabled: store::settings::get(conn, KEY_NOTIFICATIONS)?.unwrap_or(true),
            snooze_options_min: store::settings::get(conn, KEY_SNOOZE_OPTIONS)?
                .unwrap_or_else(|| DEFAULT_SNOOZE_OPTIONS.to_vec()),
            catch_up_enabled: store::settings::get(conn, KEY_CATCH_UP)?.unwrap_or(true),
            catch_up_hours: store::settings::get(conn, KEY_CATCH_UP_HOURS)?
                .unwrap_or(DEFAULT_CATCH_UP_HOURS),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use chrono::{TimeZone, Utc};

    #[test]
    fn defaults_and_overrides() {
        let db = Database::open_in_memory().unwrap();
        let s = PlannerSettings::load(db.conn(), chrono_tz::Australia::Melbourne).unwrap();
        assert_eq!(s.tz, chrono_tz::Australia::Melbourne);
        assert_eq!(s.default_task_duration_min, 30);

        let now = Utc.with_ymd_and_hms(2026, 10, 14, 0, 0, 0).unwrap();
        store::settings::set(db.conn(), KEY_TIMEZONE, &"Europe/Berlin", now).unwrap();
        store::settings::set(db.conn(), KEY_DEFAULT_TASK_DURATION, &45u32, now).unwrap();
        let s = PlannerSettings::load(db.conn(), chrono_tz::Australia::Melbourne).unwrap();
        assert_eq!(s.tz, chrono_tz::Europe::Berlin);
        assert_eq!(s.default_task_duration_min, 45);

        store::settings::set(db.conn(), KEY_TIMEZONE, &"Mars/Olympus", now).unwrap();
        let s = PlannerSettings::load(db.conn(), chrono_tz::UTC).unwrap();
        assert_eq!(s.tz, chrono_tz::UTC, "unparseable zone falls back");
    }
}

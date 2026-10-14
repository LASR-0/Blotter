//! Planner settings the core needs to interpret operations. Stored as JSON
//! in the `settings` table; missing keys fall back to defaults.

use chrono_tz::Tz;
use rusqlite::Connection;

use crate::error::Result;
use crate::slots::PlanningHours;
use crate::store;

pub const KEY_TIMEZONE: &str = "timezone";
pub const KEY_DEFAULT_TASK_DURATION: &str = "default_task_duration_min";
pub const KEY_DEFAULT_EVENT_DURATION: &str = "default_event_duration_min";
pub const KEY_PLANNING_HOURS: &str = "planning_hours";
pub const KEY_SLOT_BUFFER: &str = "slot_buffer_min";

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

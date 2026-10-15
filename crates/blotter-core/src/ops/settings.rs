//! Reading and changing the user's settings (roadmap §5.6, §5.9). Each
//! changed key is change-logged as a `setting` with its old and new value,
//! so settings changes can be undone like any other edit.

use chrono::NaiveTime;
use rusqlite::Connection;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::rollover::RolloverMode;
use super::{Ctx, OpDef};
use crate::error::{CoreError, Result};
use crate::settings::{self as keys, NotifySettings, PlannerSettings};
use crate::store;

pub(crate) const ENTITY: &str = "setting";

/// Default rollover time, mirrored from the planner.
const DEFAULT_ROLLOVER_TIME: &str = "04:00";
/// Longest snooze or default reminder: one week.
const MAX_MINUTES: u32 = 7 * 24 * 60;

pub(super) fn defs() -> Vec<OpDef> {
    vec![
        OpDef::new(
            "get_settings",
            "Read the user's settings: timezone, default durations and reminder, rollover, \
             how reminders are delivered (notifications, snooze options, catch-up) and how \
             many automatic snapshots are kept.",
            false,
            get_settings,
        ),
        OpDef::new(
            "update_settings",
            "Change one or more settings. Only the fields given are changed. \
             `default_event_reminder_min` 0 means new events get no reminder.",
            true,
            update_settings,
        ),
    ]
}

#[derive(Debug, Default, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GetSettingsArgs {}

/// Settings as the user sees them.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Settings {
    pub timezone: String,
    pub default_task_duration_min: u32,
    pub default_event_duration_min: u32,
    pub slot_buffer_min: u32,
    /// 0 = new events get no reminder.
    pub default_event_reminder_min: u32,
    pub rollover_mode: RolloverMode,
    /// Local `HH:MM`.
    pub rollover_time: String,
    pub notifications_enabled: bool,
    pub snooze_options_min: Vec<u32>,
    pub catch_up_enabled: bool,
    pub catch_up_hours: u32,
    /// Automatic snapshots kept; manual ones are always kept.
    pub snapshot_keep: u32,
}

#[derive(Debug, Default, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct UpdateSettingsArgs {
    /// Minutes before an event its default reminder fires; 0 = none.
    #[schemars(range(max = 10080))]
    pub default_event_reminder_min: Option<u32>,
    /// Local time the day's rollover may run, `HH:MM`.
    #[schemars(regex(pattern = r"^([01]\d|2[0-3]):[0-5]\d$"))]
    pub rollover_time: Option<String>,
    /// OS notifications for reminders.
    pub notifications_enabled: Option<bool>,
    /// Snooze lengths offered on a reminder, in minutes (1 to 6 of them).
    #[schemars(length(min = 1, max = 6))]
    pub snooze_options_min: Option<Vec<u32>>,
    /// Catch up on reminders missed while closed or asleep.
    pub catch_up_enabled: Option<bool>,
    /// How far back the catch-up looks, in hours (1 to 168).
    #[schemars(range(min = 1, max = 168))]
    pub catch_up_hours: Option<u32>,
    /// Automatic snapshots to keep (5 to 200).
    #[schemars(range(min = 5, max = 200))]
    pub snapshot_keep: Option<u32>,
}

fn read(conn: &Connection, planner: &PlannerSettings) -> Result<Settings> {
    let notify = NotifySettings::load(conn)?;
    Ok(Settings {
        timezone: planner.tz.name().to_owned(),
        default_task_duration_min: planner.default_task_duration_min,
        default_event_duration_min: planner.default_event_duration_min,
        slot_buffer_min: planner.slot_buffer_min,
        default_event_reminder_min: planner.default_event_reminder_min.unwrap_or(0),
        rollover_mode: store::settings::get(conn, keys::KEY_ROLLOVER_MODE)?.unwrap_or_default(),
        rollover_time: store::settings::get(conn, keys::KEY_ROLLOVER_TIME)?
            .unwrap_or_else(|| DEFAULT_ROLLOVER_TIME.to_owned()),
        notifications_enabled: notify.enabled,
        snooze_options_min: notify.snooze_options_min,
        catch_up_enabled: notify.catch_up_enabled,
        catch_up_hours: notify.catch_up_hours,
        snapshot_keep: crate::snapshots::keep(conn)?,
    })
}

fn get_settings(ctx: &mut Ctx<'_>, _: GetSettingsArgs) -> Result<Settings> {
    read(ctx.conn, ctx.settings)
}

/// Writes one key and logs the change; unchanged values are skipped.
fn write<T: Serialize>(ctx: &mut Ctx<'_>, key: &str, value: &T) -> Result<()> {
    let before: Option<Value> = store::settings::get(ctx.conn, key)?;
    let after = serde_json::to_value(value)?;
    if before.as_ref() == Some(&after) {
        return Ok(());
    }
    store::settings::set(ctx.conn, key, &after, ctx.now)?;
    ctx.record(ENTITY, key, before.as_ref(), Some(&after))
}

fn update_settings(ctx: &mut Ctx<'_>, a: UpdateSettingsArgs) -> Result<Settings> {
    if let Some(m) = a.default_event_reminder_min {
        write(ctx, keys::KEY_DEFAULT_EVENT_REMINDER, &m)?;
    }
    if let Some(t) = &a.rollover_time {
        NaiveTime::parse_from_str(t, "%H:%M")
            .map_err(|_| CoreError::invalid(format!("rollover_time \"{t}\" isn't HH:MM")))?;
        write(ctx, keys::KEY_ROLLOVER_TIME, t)?;
    }
    if let Some(on) = a.notifications_enabled {
        write(ctx, keys::KEY_NOTIFICATIONS, &on)?;
    }
    if let Some(options) = a.snooze_options_min {
        let mut options: Vec<u32> = options;
        if options.iter().any(|m| *m == 0 || *m > MAX_MINUTES) {
            return Err(CoreError::invalid(
                "each snooze option must be 1 minute to 1 week",
            ));
        }
        options.sort_unstable();
        options.dedup();
        write(ctx, keys::KEY_SNOOZE_OPTIONS, &options)?;
    }
    if let Some(on) = a.catch_up_enabled {
        write(ctx, keys::KEY_CATCH_UP, &on)?;
    }
    if let Some(h) = a.catch_up_hours {
        write(ctx, keys::KEY_CATCH_UP_HOURS, &h)?;
    }
    if let Some(n) = a.snapshot_keep {
        write(ctx, keys::KEY_SNAPSHOT_KEEP, &n)?;
    }
    // Re-read so a changed default reminder shows; the zone is unchanged.
    let planner = PlannerSettings::load(ctx.conn, ctx.settings.tz)?;
    read(ctx.conn, &planner)
}

#[cfg(test)]
mod tests {
    use crate::model::Actor;
    use serde_json::{Value, json};

    fn run(p: &mut crate::Planner, op: &str, args: Value) -> crate::Result<crate::ops::OpOutcome> {
        p.execute(op, args, &Actor::User)
    }

    #[test]
    fn defaults_update_and_change_log() {
        let mut p = crate::planner::tests::planner();
        let s = run(&mut p, "get_settings", json!({})).unwrap().result;
        assert_eq!(s["default_event_reminder_min"], 10);
        assert_eq!(s["snooze_options_min"], json!([5, 15, 60]));
        assert_eq!(s["catch_up_hours"], 24);
        assert_eq!(s["snapshot_keep"], 20);
        assert_eq!(s["notifications_enabled"], true);
        assert_eq!(s["rollover_time"], "04:00");

        let out = run(
            &mut p,
            "update_settings",
            json!({ "snooze_options_min": [60, 10, 10], "notifications_enabled": false,
                    "rollover_time": "23:30", "default_event_reminder_min": 0 }),
        )
        .unwrap();
        assert_eq!(
            out.result["snooze_options_min"],
            json!([10, 60]),
            "sorted, deduped"
        );
        assert_eq!(out.result["notifications_enabled"], false);
        assert_eq!(out.result["rollover_time"], "23:30");
        assert_eq!(out.result["default_event_reminder_min"], 0);
        assert_eq!(out.changes.len(), 4);
        assert!(out.changes.iter().all(|c| c.entity == "setting"));

        // Setting the same value again logs nothing.
        let same = run(
            &mut p,
            "update_settings",
            json!({ "rollover_time": "23:30" }),
        )
        .unwrap();
        assert!(same.changes.is_empty());

        // New events now get no reminder.
        let e = run(
            &mut p,
            "create_item",
            json!({ "title": "Gig", "kind": "event", "start": "2026-10-20T20:00" }),
        )
        .unwrap();
        let id = e.result["id"].as_str().unwrap();
        let reminders = crate::store::reminders::list_for_item(p.db().conn(), id).unwrap();
        assert!(reminders.is_empty());
    }

    #[test]
    fn rejects_bad_values() {
        let mut p = crate::planner::tests::planner();
        assert!(
            run(
                &mut p,
                "update_settings",
                json!({ "rollover_time": "25:00" })
            )
            .is_err()
        );
        assert!(
            run(
                &mut p,
                "update_settings",
                json!({ "snooze_options_min": [] })
            )
            .is_err()
        );
        assert!(
            run(
                &mut p,
                "update_settings",
                json!({ "snooze_options_min": [0] })
            )
            .is_err()
        );
        assert!(run(&mut p, "update_settings", json!({ "catch_up_hours": 0 })).is_err());
        assert!(run(&mut p, "update_settings", json!({ "snapshot_keep": 1 })).is_err());
        assert!(run(&mut p, "update_settings", json!({ "volume": 11 })).is_err());
    }
}

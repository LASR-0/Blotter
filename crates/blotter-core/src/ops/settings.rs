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
use crate::settings::{self as keys, ClockStyle, NotifySettings, PlannerSettings, WeekStart};
use crate::slots::PlanningHours;
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
            "Read the user's settings: timezone, default durations and reminder, planning \
             hours (when flexible tasks may be placed), slot buffer, rollover, week start and \
             clock, how reminders are delivered (notifications, snooze options, catch-up) and how \
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
    /// The morning briefing (D28: off by default).
    pub briefing_enabled: bool,
    /// Local `HH:MM` it runs at.
    pub briefing_time: String,
    /// When flexible tasks may be placed, per weekday (null = not that day).
    pub planning_hours: PlanningHours,
    pub week_start: WeekStart,
    pub clock: ClockStyle,
    /// Closing the main window hides it to the tray (true) or quits.
    pub close_to_tray: bool,
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
    /// The AI briefs the user each morning.
    pub briefing_enabled: Option<bool>,
    /// Local time of the morning briefing, `HH:MM`.
    #[schemars(regex(pattern = r"^([01]\d|2[0-3]):[0-5]\d$"))]
    pub briefing_time: Option<String>,
    /// Minutes a task without a duration takes when it's slotted (5 to 480).
    #[schemars(range(min = 5, max = 480))]
    pub default_task_duration_min: Option<u32>,
    /// Minutes a new event lasts when no end is given (5 to 1440).
    #[schemars(range(min = 5, max = 1440))]
    pub default_event_duration_min: Option<u32>,
    /// Gap kept between slotted items, in minutes (0 to 60).
    #[schemars(range(max = 60))]
    pub slot_buffer_min: Option<u32>,
    /// Where unfinished flexible tasks go at rollover.
    pub rollover_mode: Option<RolloverMode>,
    /// When flexible tasks may be placed, per weekday; a day set to null gets none.
    pub planning_hours: Option<PlanningHours>,
    pub week_start: Option<WeekStart>,
    pub clock: Option<ClockStyle>,
    /// Closing the main window hides it to the tray (true) or quits Blotter.
    pub close_to_tray: Option<bool>,
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
        briefing_enabled: crate::briefing::load(conn)?.enabled,
        briefing_time: crate::briefing::load(conn)?
            .time
            .format("%H:%M")
            .to_string(),
        planning_hours: planner.planning_hours.clone(),
        week_start: store::settings::get(conn, keys::KEY_WEEK_START)?.unwrap_or_default(),
        clock: store::settings::get(conn, keys::KEY_CLOCK)?.unwrap_or_default(),
        close_to_tray: store::settings::get(conn, keys::KEY_CLOSE_TO_TRAY)?.unwrap_or(true),
    })
}

fn get_settings(ctx: &mut Ctx<'_>, _: GetSettingsArgs) -> Result<Settings> {
    read(ctx.conn, ctx.settings)
}

/// Writes one key and logs the change; unchanged values are skipped.
pub(crate) fn write<T: Serialize>(ctx: &mut Ctx<'_>, key: &str, value: &T) -> Result<()> {
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
    if let Some(on) = a.briefing_enabled {
        write(ctx, keys::KEY_BRIEFING_ENABLED, &on)?;
    }
    if let Some(t) = &a.briefing_time {
        NaiveTime::parse_from_str(t, "%H:%M")
            .map_err(|_| CoreError::invalid(format!("briefing_time \"{t}\" isn't HH:MM")))?;
        write(ctx, keys::KEY_BRIEFING_TIME, t)?;
    }
    if let Some(h) = a.catch_up_hours {
        write(ctx, keys::KEY_CATCH_UP_HOURS, &h)?;
    }
    if let Some(n) = a.snapshot_keep {
        write(ctx, keys::KEY_SNAPSHOT_KEEP, &n)?;
    }
    if let Some(m) = a.default_task_duration_min {
        write(ctx, keys::KEY_DEFAULT_TASK_DURATION, &m)?;
    }
    if let Some(m) = a.default_event_duration_min {
        write(ctx, keys::KEY_DEFAULT_EVENT_DURATION, &m)?;
    }
    if let Some(m) = a.slot_buffer_min {
        write(ctx, keys::KEY_SLOT_BUFFER, &m)?;
    }
    if let Some(mode) = a.rollover_mode {
        write(ctx, keys::KEY_ROLLOVER_MODE, &mode)?;
    }
    if let Some(hours) = &a.planning_hours {
        for day in [
            hours.mon, hours.tue, hours.wed, hours.thu, hours.fri, hours.sat, hours.sun,
        ]
        .into_iter()
        .flatten()
        {
            if day.end <= day.start {
                return Err(CoreError::invalid(
                    "planning hours must end after they start, on the same day",
                ));
            }
        }
        write(ctx, keys::KEY_PLANNING_HOURS, hours)?;
    }
    if let Some(w) = a.week_start {
        write(ctx, keys::KEY_WEEK_START, &w)?;
    }
    if let Some(c) = a.clock {
        write(ctx, keys::KEY_CLOCK, &c)?;
    }
    if let Some(on) = a.close_to_tray {
        write(ctx, keys::KEY_CLOSE_TO_TRAY, &on)?;
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
    fn general_settings() {
        let mut p = crate::planner::tests::planner();
        let s = run(&mut p, "get_settings", json!({})).unwrap().result;
        assert_eq!(
            (&s["week_start"], &s["clock"], &s["close_to_tray"]),
            (&json!("mon"), &json!("24h"), &json!(true))
        );
        assert_eq!(
            s["planning_hours"]["mon"],
            json!({ "start": "07:00", "end": "22:00" })
        );
        assert_eq!(s["rollover_mode"], "inbox");

        let hours = json!({
            "mon": { "start": "09:00", "end": "17:00" }, "tue": { "start": "09:00", "end": "17:00" },
            "wed": { "start": "09:00", "end": "17:00" }, "thu": { "start": "09:00", "end": "17:00" },
            "fri": { "start": "09:00", "end": "17:00" }, "sat": null, "sun": null
        });
        let out = run(
            &mut p,
            "update_settings",
            json!({ "week_start": "sun", "clock": "12h", "close_to_tray": false,
                    "default_task_duration_min": 45, "slot_buffer_min": 10,
                    "rollover_mode": "reslot", "planning_hours": hours }),
        )
        .unwrap()
        .result;
        assert_eq!(
            (&out["week_start"], &out["clock"]),
            (&json!("sun"), &json!("12h"))
        );
        assert_eq!(out["default_task_duration_min"], 45);
        assert_eq!(out["slot_buffer_min"], 10);
        assert_eq!(out["rollover_mode"], "reslot");
        assert_eq!(out["planning_hours"]["sat"], Value::Null);

        // The slot engine uses the new hours: nothing on a Saturday.
        let slots = run(
            &mut p,
            "find_free_slots",
            json!({ "duration_min": 30, "from": "2026-10-17T00:00", "until": "2026-10-18T00:00" }),
        )
        .unwrap()
        .result;
        assert!(slots["slots"].as_array().unwrap().is_empty());

        let backwards = json!({ "mon": { "start": "17:00", "end": "09:00" } });
        assert!(
            run(
                &mut p,
                "update_settings",
                json!({ "planning_hours": backwards })
            )
            .is_err()
        );
        assert!(run(&mut p, "update_settings", json!({ "week_start": "wed" })).is_err());
        assert!(run(&mut p, "update_settings", json!({ "slot_buffer_min": 90 })).is_err());
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
        assert!(
            run(
                &mut p,
                "update_settings",
                json!({ "briefing_time": "7:30" })
            )
            .is_err()
        );
        let s = run(
            &mut p,
            "update_settings",
            json!({ "briefing_enabled": true, "briefing_time": "06:45" }),
        )
        .unwrap();
        assert_eq!(
            (
                s.result["briefing_enabled"].clone(),
                s.result["briefing_time"].clone()
            ),
            (json!(true), json!("06:45"))
        );
    }
}

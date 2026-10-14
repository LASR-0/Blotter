//! Scheduling operations over the slot engine, and the rule every path that
//! places an item must obey: a flexible task never sits in a block-out.

use chrono::Duration;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::input::DateTimeInput;
use super::{Ctx, OpDef};
use crate::error::{CoreError, Result};
use crate::model::{Anchoring, Item, ItemStatus};
use crate::slots::{Interval, Slot, SlotEngine};
use crate::store;
use crate::store::items::ItemRange;
use crate::time::Instant;
use crate::windows;

/// How far ahead `find_free_slots` looks when no `until` is given.
const DEFAULT_SEARCH_DAYS: i64 = 14;
/// Longest span one `find_free_slots` call searches.
const MAX_SEARCH_DAYS: i64 = 62;

pub(super) fn defs() -> Vec<OpDef> {
    vec![OpDef::new(
        "find_free_slots",
        "Find free time for a flexible task of `duration_min`: within planning hours, outside \
         block-outs and clear of scheduled items (plus the buffer). Returns the earliest fit in \
         each free gap, soonest first, never in the past. Defaults to the next 14 days; pass \
         `until` (e.g. a due date) to search less.",
        false,
        find_free_slots,
    )]
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FindFreeSlotsArgs {
    #[schemars(range(min = 5, max = 1440))]
    pub duration_min: u32,
    /// Earliest start; defaults to now and is never earlier than now.
    pub from: Option<DateTimeInput>,
    /// Latest end, e.g. a deadline. Defaults to 14 days after `from`.
    pub until: Option<DateTimeInput>,
    /// Defaults to 10.
    #[schemars(range(min = 1, max = 100))]
    pub limit: Option<u32>,
    /// Ignore this item's own time, when looking for somewhere to move it.
    pub exclude_item: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct FreeSlots {
    pub timezone: String,
    pub slots: Vec<Slot>,
}

fn find_free_slots(ctx: &mut Ctx<'_>, a: FindFreeSlotsArgs) -> Result<FreeSlots> {
    let tz = ctx.settings.tz;
    let from = a
        .from
        .map(|f| f.resolve(tz))
        .unwrap_or(ctx.now)
        .max(ctx.now);
    let until = a
        .until
        .map(|u| u.resolve(tz))
        .unwrap_or(from + Duration::days(DEFAULT_SEARCH_DAYS));
    if until <= from {
        return Err(CoreError::invalid("`until` must be after `from` and now"));
    }
    if until - from > Duration::days(MAX_SEARCH_DAYS) {
        return Err(CoreError::invalid(format!(
            "search at most {MAX_SEARCH_DAYS} days at a time"
        )));
    }
    let buffer = Duration::minutes(i64::from(ctx.settings.slot_buffer_min));
    let blockouts = windows::load(ctx.conn, from, until)?;
    let busy = busy_time(
        ctx,
        from - buffer,
        until + buffer,
        a.exclude_item.as_deref(),
    )?;
    let engine = SlotEngine {
        tz,
        planning: &ctx.settings.planning_hours,
        blockouts: &blockouts,
        busy: &busy,
        buffer,
    };
    let limit = a.limit.unwrap_or(10) as usize;
    Ok(FreeSlots {
        timezone: tz.name().to_owned(),
        slots: engine.slots(
            Duration::minutes(i64::from(a.duration_min)),
            from,
            until,
            limit,
        ),
    })
}

/// Time taken by open, timed items overlapping `[from, to)`.
pub(crate) fn busy_time(
    ctx: &Ctx<'_>,
    from: Instant,
    to: Instant,
    exclude: Option<&str>,
) -> Result<Vec<Interval>> {
    let tz = ctx.settings.tz;
    let range = ItemRange {
        start: from,
        end: to,
        start_date: from.with_timezone(&tz).date_naive(),
        end_date: to.with_timezone(&tz).date_naive(),
    };
    Ok(store::items::list_in_range(ctx.conn, range)?
        .into_iter()
        .filter(|i| i.status == ItemStatus::Open && Some(i.id.as_str()) != exclude)
        .filter_map(|i| Some((i.start_at?, i.end_at?)))
        .collect())
}

/// Refuses to put a flexible, open item inside a block-out (roadmap §4.5
/// rule 1). Runs on every create, update and schedule, but only when the
/// item's time or anchoring changes, so an item caught inside a block-out
/// created later can still be edited, completed or moved out.
pub(crate) fn check_placement(ctx: &Ctx<'_>, before: Option<&Item>, item: &Item) -> Result<()> {
    if item.anchoring != Anchoring::Flexible || item.status != ItemStatus::Open {
        return Ok(());
    }
    let (Some(start), Some(end)) = (item.start_at, item.end_at) else {
        return Ok(());
    };
    let unchanged = before.is_some_and(|b| {
        b.start_at == item.start_at && b.end_at == item.end_at && b.anchoring == item.anchoring
    });
    if unchanged {
        return Ok(());
    }
    // A zero-length item still occupies its start minute.
    let end = end.max(start + Duration::minutes(1));
    let blocking = windows::load(ctx.conn, start, end)?;
    let Some(w) = blocking.iter().find(|w| w.overlaps(start, end)) else {
        return Ok(());
    };
    let tz = ctx.settings.tz;
    let (ws, we) = (w.start.with_timezone(&tz), w.end.with_timezone(&tz));
    Err(CoreError::invalid(format!(
        "'{}' would be inside the {} block-out ({} {}–{}). Flexible tasks can't go in \
         block-outs: pick another time, or make it fixed.",
        item.title,
        w.label,
        ws.format("%a %-d %b"),
        ws.format("%H:%M"),
        we.format("%H:%M"),
    )))
}

#[cfg(test)]
mod tests {
    use crate::model::Actor;
    use serde_json::{Value, json};

    fn run(p: &mut crate::Planner, op: &str, args: Value) -> Value {
        p.execute(op, args, &Actor::User).unwrap().result
    }

    fn fails(p: &mut crate::Planner, op: &str, args: Value) -> String {
        p.execute(op, args, &Actor::User).unwrap_err().to_string()
    }

    /// The test clock is Wed 14 Oct 2026 09:00 Melbourne.
    fn with_work() -> crate::Planner {
        let mut p = crate::planner::tests::planner();
        run(
            &mut p,
            "create_blockout",
            json!({ "label": "Work", "weekdays": ["mon","tue","wed","thu","fri"],
                    "start_time": "08:30", "end_time": "17:00" }),
        );
        p
    }

    fn starts(out: &Value) -> Vec<String> {
        out["slots"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s["start"].as_str().unwrap().to_owned())
            .collect()
    }

    #[test]
    fn free_slots_skip_blockouts_and_busy_time() {
        let mut p = with_work();
        run(
            &mut p,
            "create_item",
            json!({ "title": "Gym", "start": "2026-10-14T17:30", "duration_min": 60 }),
        );
        let out = run(
            &mut p,
            "find_free_slots",
            json!({ "duration_min": 45, "until": "2026-10-15T09:00", "limit": 5 }),
        );
        // Now (Wed 09:00) is inside Work and 17:00–17:30 is too short before Gym,
        // so: 18:30 Wednesday, then 07:00 Thursday.
        assert_eq!(
            starts(&out),
            ["2026-10-14T07:30:00Z", "2026-10-14T20:00:00Z"],
            "18:30 Wed and 07:00 Thu, Melbourne"
        );
        assert_eq!(out["timezone"], "Australia/Melbourne");
    }

    #[test]
    fn free_slots_validate_their_range() {
        let mut p = with_work();
        assert!(
            fails(
                &mut p,
                "find_free_slots",
                json!({ "duration_min": 30, "until": "2026-10-01T00:00" })
            )
            .contains("after")
        );
        assert!(
            fails(
                &mut p,
                "find_free_slots",
                json!({ "duration_min": 30, "until": "2027-10-01T00:00" })
            )
            .contains("62 days")
        );
        assert!(
            fails(&mut p, "find_free_slots", json!({ "duration_min": 2 })).contains("duration_min")
        );
    }

    #[test]
    fn flexible_tasks_never_land_in_blockouts() {
        let mut p = with_work();
        let err = fails(
            &mut p,
            "create_item",
            json!({ "title": "Meal prep", "start": "2026-10-14T16:30" }),
        );
        assert!(
            err.contains("Work block-out (Wed 14 Oct 08:30–17:00)"),
            "{err}"
        );

        // Fixed items may go inside.
        run(
            &mut p,
            "create_item",
            json!({ "title": "Dentist", "kind": "event", "start": "2026-10-14T15:00" }),
        );
        run(
            &mut p,
            "create_item",
            json!({ "title": "Telehealth", "anchoring": "fixed", "start": "2026-10-14T10:00" }),
        );

        // Moving a flexible task in by update or schedule is refused too.
        let task = run(
            &mut p,
            "create_item",
            json!({ "title": "Run", "start": "2026-10-14T07:00" }),
        );
        assert!(
            fails(
                &mut p,
                "update_item",
                json!({ "id": task["id"], "start": "2026-10-14T08:15" })
            )
            .contains("Work")
        );
        assert!(
            fails(
                &mut p,
                "schedule_item",
                json!({ "id": task["id"], "start": "2026-10-15T12:00" })
            )
            .contains("Thu 15 Oct")
        );
        // Ending exactly when Work starts is fine.
        run(
            &mut p,
            "update_item",
            json!({ "id": task["id"], "start": "2026-10-14T08:00" }),
        );
        // Day-only (all-day) tasks have no time, so they're fine on a workday.
        run(
            &mut p,
            "schedule_item",
            json!({ "id": task["id"], "date": "2026-10-15" }),
        );
    }

    #[test]
    fn items_caught_by_a_new_blockout_can_still_be_edited() {
        let mut p = crate::planner::tests::planner();
        let task = run(
            &mut p,
            "create_item",
            json!({ "title": "Errand", "start": "2026-10-14T12:00" }),
        );
        run(
            &mut p,
            "create_blockout",
            json!({ "label": "Work", "weekdays": ["wed"], "start_time": "08:30", "end_time": "17:00" }),
        );
        run(
            &mut p,
            "update_item",
            json!({ "id": task["id"], "title": "Errand (renamed)" }),
        );
        run(&mut p, "complete_item", json!({ "id": task["id"] }));
        run(&mut p, "uncomplete_item", json!({ "id": task["id"] }));
        // But moving it within the block-out is refused.
        assert!(
            fails(
                &mut p,
                "update_item",
                json!({ "id": task["id"], "start": "2026-10-14T13:00" })
            )
            .contains("Work")
        );
        // Making it flexible again while inside is a change of anchoring, so it's checked.
        run(
            &mut p,
            "update_item",
            json!({ "id": task["id"], "anchoring": "fixed" }),
        );
        assert!(
            fails(
                &mut p,
                "update_item",
                json!({ "id": task["id"], "anchoring": "flexible" })
            )
            .contains("Work")
        );
    }

    #[test]
    fn lifted_dates_allow_flexible_tasks() {
        let mut p = with_work();
        let work = run(&mut p, "list_blockouts", json!({}))[0]["id"].clone();
        run(
            &mut p,
            "add_blockout_exception",
            json!({ "blockout_id": work, "date": "2026-10-15" }),
        );
        run(
            &mut p,
            "create_item",
            json!({ "title": "Day off errand", "start": "2026-10-15T11:00" }),
        );
    }
}

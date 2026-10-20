//! End-of-day rollover (roadmap §4.5): unfinished flexible tasks left on
//! earlier days go back to the Inbox, or are re-slotted into free time.
//! Fixed items are never moved automatically.

use chrono::Duration;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::input::start_of_day;
use super::items::{ENTITY as ITEM_ENTITY, finalize};
use super::occurrences;
use super::reads::{day_range, today};
use super::slots::{PlacedTask, UnplacedTask, place};
use super::{Ctx, OpDef};
use crate::error::Result;
use crate::model::{Anchoring, Item, ItemKind, ItemStatus};
use crate::recurrence::{self, Slot};
use crate::store;
use crate::store::items::ItemRange;

/// Where rolled-over tasks go. Stored as the `rollover_mode` setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RolloverMode {
    /// Back to the Inbox, to be planned again.
    #[default]
    Inbox,
    /// Into the next free slots; anything that doesn't fit goes to the Inbox.
    Reslot,
}

/// How far ahead re-slotting looks.
const RESLOT_DAYS: i64 = 14;
/// Furthest ahead a windowed task is re-planned (its window may run longer).
const WINDOW_DAYS: i64 = 62;

pub(super) fn defs() -> Vec<OpDef> {
    vec![OpDef::new(
        "rollover",
        "Move unfinished flexible tasks left on earlier days: back to the Inbox, or into the next \
         free slots (`mode`, else the user's rollover setting; tasks that don't fit go to the \
         Inbox). Fixed items and recurring tasks never move. Runs automatically once a day; \
         running it again does nothing until more tasks are left behind.",
        true,
        rollover,
    )]
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RolloverArgs {
    /// Defaults to the `rollover_mode` setting (`inbox` unless changed).
    pub mode: Option<RolloverMode>,
}

#[derive(Debug, Serialize)]
pub struct RolloverResult {
    pub mode: RolloverMode,
    /// Tasks returned to the Inbox (all of them in `inbox` mode; in
    /// `reslot` mode, those that didn't fit).
    pub to_inbox: Vec<Item>,
    pub reslotted: Vec<PlacedTask>,
    /// Why each task in `to_inbox` couldn't be re-slotted (`reslot` mode).
    pub not_reslotted: Vec<UnplacedTask>,
}

/// Open flexible tasks whose scheduled day is before today.
pub(crate) fn left_behind(ctx: &Ctx<'_>) -> Result<Vec<Item>> {
    let today = today(ctx);
    let today_start = start_of_day(ctx.settings.tz, today);
    Ok(
        store::items::list_overdue(ctx.conn, ctx.now, today_start, today)?
            .into_iter()
            .filter(|i| {
                i.kind == ItemKind::Task
                    && i.anchoring == Anchoring::Flexible
                    && i.rrule.is_none()
                    // An occurrence of a repeating task is re-planned in its own week.
                    && i.parent_id.is_none()
            })
            .filter(|i| match (i.end_at, i.start_date) {
                (Some(end), _) => end <= today_start,
                (None, Some(start)) => i.end_date.unwrap_or(start) < today,
                // Unscheduled tasks merely past their deadline stay in the Inbox.
                (None, None) => false,
            })
            .collect(),
    )
}

fn to_inbox(ctx: &mut Ctx<'_>, item: Item) -> Result<Item> {
    let before = item.clone();
    let mut item = item;
    item.all_day = false;
    item.start_at = None;
    item.end_at = None;
    item.start_date = None;
    item.end_date = None;
    item.updated_at = ctx.now;
    finalize(ctx, &mut item)?;
    store::items::update(ctx.conn, &item)?;
    ctx.record(ITEM_ENTITY, &item.id, Some(&before), Some(&item))?;
    Ok(item)
}

/// How far back missed occurrences of repeating tasks are looked for.
const REPEAT_LOOKBACK_DAYS: i64 = 7;

/// Repeating flexible tasks (weekly groceries, laundry): an occurrence whose
/// day has passed undone is moved to the next free slot before the next
/// occurrence (its usual time of day first), so it still gets done that week.
/// One that can't fit stays where it was; one whose next occurrence has
/// already come is left alone.
fn replan_repeats(ctx: &mut Ctx<'_>, result: &mut RolloverResult) -> Result<()> {
    let tz = ctx.settings.tz;
    let today = today(ctx);
    let today_start = start_of_day(tz, today);
    let range = day_range(
        ctx,
        today - Duration::days(REPEAT_LOOKBACK_DAYS),
        today - Duration::days(1),
    );
    let series_list: Vec<Item> = store::items::list_series(ctx.conn, range)?
        .into_iter()
        .filter(|s| {
            s.kind == ItemKind::Task
                && s.anchoring == Anchoring::Flexible
                && s.status == ItemStatus::Open
        })
        .collect();
    for series in series_list {
        let Ok(slots) = recurrence::occurrences(&series, tz, range) else {
            continue;
        };
        let children = store::items::list_children(ctx.conn, &series.id)?;
        for slot in slots {
            let (slot_start, slot_end) = match slot {
                Slot::Timed { start, end } => (start, end),
                Slot::AllDay { date, end_date } => (
                    start_of_day(tz, date),
                    start_of_day(tz, end_date.unwrap_or(date) + Duration::days(1)),
                ),
            };
            if slot_end > today_start {
                continue;
            }
            let rid = slot.recurrence_id();
            let existing = children
                .iter()
                .find(|c| c.recurrence_id.as_deref() == Some(rid.as_str()))
                .cloned();
            if let Some(o) = &existing {
                // Done, cancelled, or already moved to today or later.
                let moved_ahead = o.end_at.is_some_and(|e| e > today_start)
                    || o.start_date.is_some_and(|d| d >= today);
                if o.status != ItemStatus::Open || moved_ahead {
                    continue;
                }
            }
            // It has until the next occurrence starts.
            let ahead = ItemRange {
                start: slot_start + Duration::minutes(1),
                end: slot_start + Duration::days(400),
                start_date: slot_start.with_timezone(&tz).date_naive() + Duration::days(1),
                end_date: today + Duration::days(400),
            };
            let next = recurrence::occurrences(&series, tz, ahead)?
                .into_iter()
                .map(|s| match s {
                    Slot::Timed { start, .. } => start,
                    Slot::AllDay { date, .. } => start_of_day(tz, date),
                })
                .find(|s| *s > slot_start);
            let until = next.unwrap_or(ctx.now + Duration::days(7));
            if until <= ctx.now {
                continue;
            }
            let child = occurrences::override_for(ctx, &series, slot, existing)?;
            let (placed, missed) = place(ctx, vec![child], ctx.now, until, true)?;
            result.reslotted.extend(placed);
            for (_, mut why) in missed {
                why.reason = why
                    .reason
                    .replace("It goes back to the Inbox.", "It stays on its day.");
                result.not_reslotted.push(why);
            }
        }
    }
    Ok(())
}

fn rollover(ctx: &mut Ctx<'_>, a: RolloverArgs) -> Result<RolloverResult> {
    let mode = match a.mode {
        Some(mode) => mode,
        None => {
            store::settings::get(ctx.conn, crate::settings::KEY_ROLLOVER_MODE)?.unwrap_or_default()
        }
    };
    // Tasks with a do-window are always re-planned inside it, whatever the
    // mode; any that no longer fit (or whose window has closed) go to the Inbox.
    let (windowed, tasks): (Vec<Item>, Vec<Item>) = left_behind(ctx)?
        .into_iter()
        .partition(|i| i.not_before.is_some() || i.window_start.is_some());
    let mut result = RolloverResult {
        mode,
        to_inbox: Vec::new(),
        reslotted: Vec::new(),
        not_reslotted: Vec::new(),
    };
    replan_repeats(ctx, &mut result)?;
    let mut leftovers = Vec::new();
    if !windowed.is_empty() {
        let until = windowed
            .iter()
            .filter_map(|i| i.due_at)
            .max()
            .unwrap_or(ctx.now)
            .clamp(
                ctx.now + Duration::days(RESLOT_DAYS),
                ctx.now + Duration::days(WINDOW_DAYS),
            );
        let (placed, missed) = place(ctx, windowed, ctx.now, until, true)?;
        result.reslotted = placed;
        for (item, reason) in missed {
            leftovers.push(item);
            result.not_reslotted.push(reason);
        }
    }
    leftovers.extend(match mode {
        RolloverMode::Inbox => tasks,
        RolloverMode::Reslot => {
            let (from, until) = (ctx.now, ctx.now + Duration::days(RESLOT_DAYS));
            let (placed, missed) = place(ctx, tasks, from, until, true)?;
            result.reslotted.extend(placed);
            let (items, reasons): (Vec<_>, Vec<_>) = missed.into_iter().unzip();
            result.not_reslotted.extend(reasons);
            items
        }
    });
    for item in leftovers {
        let item = to_inbox(ctx, item)?;
        result.to_inbox.push(item);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::model::Actor;
    use serde_json::{Value, json};

    fn run(p: &mut crate::Planner, op: &str, args: Value) -> Value {
        p.execute(op, args, &Actor::User).unwrap().result
    }

    /// The test clock is Wed 14 Oct 2026 09:00 Melbourne.
    fn setup() -> (crate::Planner, Value) {
        let mut p = crate::planner::tests::planner();
        let ids = json!({
            "timed": run(&mut p, "create_item", json!({ "title": "Timed yesterday", "start": "2026-10-13T19:00" }))["id"],
            "day": run(&mut p, "create_item", json!({ "title": "Day task Monday", "date": "2026-10-12" }))["id"],
            "fixed": run(&mut p, "create_item", json!({ "title": "Fixed yesterday", "anchoring": "fixed", "start": "2026-10-13T10:00" }))["id"],
            "event": run(&mut p, "create_item", json!({ "title": "Event yesterday", "kind": "event", "start": "2026-10-13T12:00" }))["id"],
            "today": run(&mut p, "create_item", json!({ "title": "Earlier today", "start": "2026-10-14T07:00" }))["id"],
            "due": run(&mut p, "create_item", json!({ "title": "Inbox, past due", "due": "2026-10-13T17:00" }))["id"],
        });
        let done = run(
            &mut p,
            "create_item",
            json!({ "title": "Done yesterday", "start": "2026-10-13T08:00" }),
        );
        run(&mut p, "complete_item", json!({ "id": done["id"] }));
        (p, ids)
    }

    fn titles(items: &Value) -> Vec<&str> {
        items
            .as_array()
            .unwrap()
            .iter()
            .map(|i| i["title"].as_str().unwrap())
            .collect()
    }

    #[test]
    fn inbox_mode_returns_only_flexible_tasks_from_earlier_days() {
        let (mut p, ids) = setup();
        let out = p.execute("rollover", json!({}), &Actor::System).unwrap();
        let mut moved = titles(&out.result["to_inbox"]);
        moved.sort_unstable();
        assert_eq!(moved, ["Day task Monday", "Timed yesterday"]);
        assert!(out.changes.iter().all(|c| c.actor == Actor::System));

        let item = run(&mut p, "get_item", json!({ "id": ids["timed"] }));
        assert!(item["start_at"].is_null() && item["start_date"].is_null());
        for kept in ["fixed", "event", "today"] {
            let item = run(&mut p, "get_item", json!({ "id": ids[kept] }));
            assert!(!item["start_at"].is_null(), "{kept} must not move");
        }

        // Nothing left behind any more: a second run is a no-op.
        let again = p.execute("rollover", json!({}), &Actor::System).unwrap();
        assert!(again.changes.is_empty());
    }

    #[test]
    fn reslot_mode_places_what_fits_and_inboxes_the_rest() {
        let (mut p, ids) = setup();
        run(
            &mut p,
            "update_item",
            json!({ "id": ids["day"], "duration_min": 1020 }),
        );
        let out = run(&mut p, "rollover", json!({ "mode": "reslot" }));
        assert_eq!(out["mode"], "reslot");
        // The 30m task gets the next free slot (now, 09:00, with no block-outs).
        assert_eq!(out["reslotted"][0]["item"]["title"], "Timed yesterday");
        assert_eq!(out["reslotted"][0]["start"], "2026-10-13T22:00:00Z");
        // 17 hours is longer than any day's planning hours, so it goes to the Inbox.
        assert_eq!(titles(&out["to_inbox"]), ["Day task Monday"]);
        assert!(
            out["not_reslotted"][0]["reason"]
                .as_str()
                .unwrap()
                .ends_with("It goes back to the Inbox.")
        );
    }

    #[test]
    fn the_setting_picks_the_default_mode() {
        let (mut p, _) = setup();
        let now = crate::planner::tests::now();
        crate::store::settings::set(
            p.db().conn(),
            crate::settings::KEY_ROLLOVER_MODE,
            &"reslot",
            now,
        )
        .unwrap();
        let out = run(&mut p, "rollover", json!({}));
        assert_eq!(out["mode"], "reslot");
    }

    #[test]
    fn a_windowed_task_is_replanned_inside_its_window_or_goes_to_the_inbox() {
        let mut p = crate::planner::tests::planner();
        run(
            &mut p,
            "create_item",
            json!({ "title": "Book trip", "start": "2026-10-13T19:00", "duration_min": 30,
                    "not_before": "2026-10-16", "due": "2026-10-25T17:00" }),
        );
        run(
            &mut p,
            "create_item",
            json!({ "title": "Closed", "start": "2026-10-12T19:00", "duration_min": 30,
                    "not_before": "2026-10-10", "due": "2026-10-13T17:00" }),
        );
        // Inbox mode, but the windowed task is re-planned: first thing on its window's first day.
        let out = run(&mut p, "rollover", json!({}));
        assert_eq!(out["mode"], "inbox");
        assert_eq!(out["reslotted"][0]["item"]["title"], "Book trip");
        assert_eq!(out["reslotted"][0]["start"], "2026-10-15T20:00:00Z");
        assert_eq!(titles(&out["to_inbox"]), ["Closed"]);
    }

    #[test]
    fn the_usual_time_of_day_is_tried_first() {
        let mut p = crate::planner::tests::planner();
        run(
            &mut p,
            "create_item",
            json!({ "title": "Laundry", "duration_min": 60,
                    "window_start": "18:00", "window_end": "20:00" }),
        );
        let out = run(&mut p, "auto_slot", json!({}));
        // 18:00 Melbourne today, not 09:00 now.
        assert_eq!(out["placed"][0]["start"], "2026-10-14T07:00:00Z");
        assert!(
            p.execute(
                "create_item",
                json!({ "title": "Bad", "window_start": "20:00", "window_end": "18:00" }),
                &Actor::User,
            )
            .is_err()
        );
    }

    #[test]
    fn a_missed_weekly_task_moves_to_the_next_free_slot_that_week() {
        let mut p = crate::planner::tests::planner();
        // Saturdays 09:00, usually 09:00–12:00; Sat 10 Oct was missed.
        let series = run(
            &mut p,
            "create_item",
            json!({ "title": "Groceries", "start": "2026-10-10T09:00", "duration_min": 60,
                    "rrule": "FREQ=WEEKLY;BYDAY=SA", "window_start": "09:00", "window_end": "12:00" }),
        );
        let out = run(&mut p, "rollover", json!({}));
        assert_eq!(out["reslotted"].as_array().unwrap().len(), 1);
        let moved = &out["reslotted"][0];
        // Wed 14 Oct 09:00 Melbourne: now, inside its usual time, before next Saturday.
        assert_eq!(moved["start"], "2026-10-13T22:00:00Z");
        assert_eq!(moved["item"]["parent_id"], series["id"]);
        assert!(out["to_inbox"].as_array().unwrap().is_empty());
        // Running again moves nothing more.
        let again = run(&mut p, "rollover", json!({}));
        assert!(again["reslotted"].as_array().unwrap().is_empty());
    }

    #[test]
    fn a_done_weekly_task_stays_put() {
        let mut p = crate::planner::tests::planner();
        run(
            &mut p,
            "create_item",
            json!({ "title": "Laundry", "start": "2026-10-10T09:00", "duration_min": 60,
                    "rrule": "FREQ=WEEKLY;BYDAY=SA" }),
        );
        let agenda = run(
            &mut p,
            "list_agenda",
            json!({ "start": "2026-10-10", "end": "2026-10-10" }),
        );
        run(
            &mut p,
            "complete_item",
            json!({ "id": agenda["items"][0]["id"] }),
        );
        let out = run(&mut p, "rollover", json!({}));
        assert!(out["reslotted"].as_array().unwrap().is_empty());
    }
}

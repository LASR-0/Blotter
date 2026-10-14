//! How operations reach one occurrence of a recurring series: resolving
//! occurrence ids, creating overrides, and splitting a series for "this
//! and following" changes.

use chrono::Duration;
use schemars::JsonSchema;
use serde::Deserialize;

use super::Ctx;
use super::items::ENTITY as ITEM_ENTITY;
use super::reminders::ENTITY as REMINDER_ENTITY;
use crate::error::{CoreError, Result};
use crate::id::new_id;
use crate::model::Item;
use crate::recurrence::{self, Slot};
use crate::store;

/// Which occurrences of a recurring item a change applies to. Ignored for
/// items that don't repeat.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    /// Only this occurrence (the default).
    #[default]
    This,
    /// This occurrence and every later one.
    Following,
    /// The whole series.
    All,
}

/// What an item id names.
pub(crate) enum Target {
    /// A stored item that isn't part of a series.
    Item(Item),
    /// One occurrence of `series`: either an occurrence id
    /// (`<series>@<recurrence id>`), or the id of its stored override.
    Occurrence {
        series: Box<Item>,
        slot: Slot,
        existing: Option<Item>,
    },
}

pub(crate) fn resolve(ctx: &Ctx<'_>, id: &str) -> Result<Target> {
    let tz = ctx.settings.tz;
    let (series, rid, existing) = match recurrence::split_occurrence_id(id) {
        Some((series_id, rid)) => {
            let series = store::items::require(ctx.conn, series_id)?;
            let existing = store::items::list_children(ctx.conn, series_id)?
                .into_iter()
                .find(|c| c.recurrence_id.as_deref() == Some(rid));
            (series, rid.to_owned(), existing)
        }
        None => {
            let item = store::items::require(ctx.conn, id)?;
            let (Some(parent), Some(rid)) = (item.parent_id.clone(), item.recurrence_id.clone())
            else {
                return Ok(Target::Item(item));
            };
            (store::items::require(ctx.conn, &parent)?, rid, Some(item))
        }
    };
    if series.rrule.is_none() {
        return Err(CoreError::not_found("item", id));
    }
    let slot = match recurrence::find_occurrence(&series, tz, &rid)? {
        Some(slot) => slot,
        None => match &existing {
            // An override of an occurrence the rule no longer has: use where it was.
            Some(o) => match (o.start_at, o.end_at, o.start_date) {
                (Some(start), end, _) => Slot::Timed {
                    start,
                    end: end.unwrap_or(start),
                },
                (None, _, Some(date)) => Slot::AllDay {
                    date,
                    end_date: o.end_date,
                },
                _ => return Err(CoreError::not_found("item", id)),
            },
            None => return Err(CoreError::not_found("item", id)),
        },
    };
    Ok(Target::Occurrence {
        series: Box::new(series),
        slot,
        existing,
    })
}

/// The stored override for one occurrence, created from the occurrence as
/// it is now if there isn't one yet.
pub(crate) fn override_for(
    ctx: &mut Ctx<'_>,
    series: &Item,
    slot: Slot,
    existing: Option<Item>,
) -> Result<Item> {
    if let Some(o) = existing {
        return Ok(o);
    }
    let mut child = recurrence::occurrence_item(series, slot);
    child.id = new_id();
    child.rrule = None;
    child.created_at = ctx.now;
    child.updated_at = ctx.now;
    store::items::insert(ctx.conn, &child)?;
    ctx.record(ITEM_ENTITY, &child.id, None, Some(&child))?;
    Ok(child)
}

/// The instant just before `slot`, for ending a series there.
fn until_before(slot: Slot) -> chrono::DateTime<chrono::Utc> {
    match slot {
        Slot::Timed { start, .. } => start - Duration::seconds(1),
        Slot::AllDay { date, .. } => {
            let day_before = date.pred_opt().unwrap_or(date);
            day_before.and_time(chrono::NaiveTime::MIN).and_utc()
        }
    }
}

/// Ends `series` just before `slot`. Overrides from `slot` on are returned
/// (still attached) so the caller can move or delete them. Returns false
/// when `slot` is the first occurrence, i.e. nothing would remain.
pub(crate) fn end_before(ctx: &mut Ctx<'_>, series: &Item, slot: Slot) -> Result<bool> {
    let tz = ctx.settings.tz;
    if recurrence::count_before(series, tz, slot)? == 0 {
        return Ok(false);
    }
    let rule = series.rrule.clone().unwrap_or_default();
    let mut ended = series.clone();
    ended.rrule = Some(recurrence::rule_until(&rule, until_before(slot)));
    ended.updated_at = ctx.now;
    store::items::update(ctx.conn, &ended)?;
    ctx.record(ITEM_ENTITY, &ended.id, Some(series), Some(&ended))?;
    Ok(true)
}

/// Overrides of `series` at or after `slot`.
pub(crate) fn overrides_from(ctx: &Ctx<'_>, series: &Item, slot: Slot) -> Result<Vec<Item>> {
    let rid = slot.recurrence_id();
    Ok(store::items::list_children(ctx.conn, &series.id)?
        .into_iter()
        .filter(|c| {
            c.recurrence_id
                .as_deref()
                .is_some_and(|r| r >= rid.as_str())
        })
        .collect())
}

/// Splits `series` at `slot` for a "this and following" change: the old
/// series ends just before it, and a new series (returned) starts at it
/// with the same rule (COUNT reduced by the occurrences already past),
/// reminders and later overrides. At the first occurrence there is
/// nothing to split, so the series itself is returned.
pub(crate) fn split(ctx: &mut Ctx<'_>, series: &Item, slot: Slot) -> Result<Item> {
    let tz = ctx.settings.tz;
    let before = recurrence::count_before(series, tz, slot)?;
    if !end_before(ctx, series, slot)? {
        return Ok(series.clone());
    }
    let rule = series.rrule.clone().unwrap_or_default();
    let mut next = recurrence::occurrence_item(series, slot);
    next.id = new_id();
    next.parent_id = None;
    next.recurrence_id = None;
    next.rrule = Some(match recurrence::rule_count(&rule) {
        Some(count) => recurrence::rule_with_count(&rule, count.saturating_sub(before).max(1)),
        None => rule,
    });
    next.created_at = ctx.now;
    next.updated_at = ctx.now;
    store::items::insert(ctx.conn, &next)?;
    ctx.record(ITEM_ENTITY, &next.id, None, Some(&next))?;

    for child in overrides_from(ctx, series, slot)? {
        let mut moved = child.clone();
        moved.parent_id = Some(next.id.clone());
        moved.updated_at = ctx.now;
        store::items::update(ctx.conn, &moved)?;
        ctx.record(ITEM_ENTITY, &moved.id, Some(&child), Some(&moved))?;
    }
    for r in store::reminders::list_for_item(ctx.conn, &series.id)? {
        let mut copy = r.clone();
        copy.id = new_id();
        copy.item_id = next.id.clone();
        copy.fired_at = None;
        copy.snoozed_until = None;
        copy.created_at = ctx.now;
        store::reminders::upsert(ctx.conn, &copy)?;
        ctx.record(REMINDER_ENTITY, &copy.id, None, Some(&copy))?;
    }
    Ok(next)
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

    /// Standup at 09:00–09:15 Melbourne from Mon 12 Oct 2026.
    fn standup(rule: &str) -> (crate::Planner, String) {
        let mut p = crate::planner::tests::planner();
        let s = run(
            &mut p,
            "create_item",
            json!({ "title": "Standup", "kind": "event", "start": "2026-10-12T09:00",
                    "duration_min": 15, "rrule": rule }),
        );
        (p, s["id"].as_str().unwrap().to_owned())
    }

    /// (title, start) of every item in the week of 12 Oct.
    fn week(p: &mut crate::Planner) -> Vec<(String, String)> {
        run(
            p,
            "list_agenda",
            json!({ "start": "2026-10-12", "end": "2026-10-18" }),
        )["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|i| {
                (
                    i["title"].as_str().unwrap().to_owned(),
                    i["start_at"].as_str().unwrap_or("").to_owned(),
                )
            })
            .collect()
    }

    fn occ(series: &str, utc: &str) -> String {
        format!("{series}@{utc}")
    }

    #[test]
    fn a_series_expands_into_occurrences() {
        let (mut p, s) = standup("freq=weekly;byday=mo,tu,we,th,fr");
        let agenda = run(
            &mut p,
            "list_agenda",
            json!({ "start": "2026-10-12", "end": "2026-10-18" }),
        );
        let items = agenda["items"].as_array().unwrap();
        assert_eq!(items.len(), 5);
        assert_eq!(items[1]["id"], occ(&s, "2026-10-12T22:00:00Z"));
        assert_eq!(items[1]["parent_id"], s.as_str());
        assert_eq!(
            items[1]["rrule"], "FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR",
            "normalised"
        );
        let one = run(
            &mut p,
            "get_item",
            json!({ "id": occ(&s, "2026-10-13T22:00:00Z") }),
        );
        assert_eq!(one["start_at"], "2026-10-13T22:00:00Z");
        // Saturday has no occurrence.
        let err = fails(
            &mut p,
            "get_item",
            json!({ "id": occ(&s, "2026-10-16T22:00:00Z") }),
        );
        assert!(err.contains("not found"), "{err}");
    }

    #[test]
    fn this_occurrence_becomes_an_override() {
        let (mut p, s) = standup("FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR");
        let wed = occ(&s, "2026-10-13T22:00:00Z");
        let moved = run(
            &mut p,
            "update_item",
            json!({ "id": wed, "start": "2026-10-14T10:00" }),
        );
        assert_ne!(moved["id"], wed.as_str(), "stored as its own row");
        assert_eq!(moved["parent_id"], s.as_str());
        let starts: Vec<String> = week(&mut p).into_iter().map(|(_, s)| s).collect();
        assert_eq!(
            starts,
            [
                "2026-10-11T22:00:00Z",
                "2026-10-12T22:00:00Z",
                "2026-10-13T23:00:00Z", // Wednesday, now 10:00
                "2026-10-14T22:00:00Z",
                "2026-10-15T22:00:00Z",
            ]
        );
        assert_eq!(
            run(&mut p, "get_item", json!({ "id": wed }))["id"],
            moved["id"]
        );
        let err = fails(
            &mut p,
            "update_item",
            json!({ "id": wed, "rrule": "FREQ=DAILY" }),
        );
        assert!(err.contains("whole series"), "{err}");
    }

    #[test]
    fn all_occurrences_move_by_the_same_amount() {
        let (mut p, s) = standup("FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR");
        run(
            &mut p,
            "update_item",
            json!({ "id": occ(&s, "2026-10-14T22:00:00Z"), "start": "2026-10-15T09:30",
                    "title": "Daily sync", "scope": "all" }),
        );
        let week = week(&mut p);
        assert_eq!(week.len(), 5);
        assert!(week.iter().all(|(t, _)| t == "Daily sync"));
        assert_eq!(week[0].1, "2026-10-11T22:30:00Z", "Monday 09:30");
    }

    #[test]
    fn this_and_following_splits_the_series() {
        let (mut p, s) = standup("FREQ=DAILY;COUNT=10");
        let thu = occ(&s, "2026-10-14T22:00:00Z");
        let out = p
            .execute(
                "update_item",
                json!({ "id": thu, "title": "New standup", "scope": "following" }),
                &Actor::User,
            )
            .unwrap();
        assert_eq!(
            out.result["rrule"], "FREQ=DAILY;COUNT=7",
            "3 already happened"
        );
        let titles: Vec<String> = week(&mut p).into_iter().map(|(t, _)| t).collect();
        assert_eq!(
            titles,
            [
                "Standup",
                "Standup",
                "Standup",
                "New standup",
                "New standup",
                "New standup",
                "New standup"
            ]
        );
        let old = run(&mut p, "get_item", json!({ "id": s }));
        assert!(
            old["rrule"]
                .as_str()
                .unwrap()
                .contains("UNTIL=20261014T215959Z")
        );
        let batch = &out.changes[0].batch_id;
        assert!(
            out.changes.iter().all(|c| &c.batch_id == batch),
            "one batch"
        );
    }

    #[test]
    fn deleting_occurrences() {
        let (mut p, s) = standup("FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR");
        run(
            &mut p,
            "delete_item",
            json!({ "id": occ(&s, "2026-10-12T22:00:00Z") }),
        );
        assert_eq!(week(&mut p).len(), 4, "Tuesday is an exception now");

        run(
            &mut p,
            "delete_item",
            json!({ "id": occ(&s, "2026-10-14T22:00:00Z"), "scope": "following" }),
        );
        let starts: Vec<String> = week(&mut p).into_iter().map(|(_, s)| s).collect();
        assert_eq!(
            starts,
            ["2026-10-11T22:00:00Z", "2026-10-13T22:00:00Z"],
            "Mon and Wed"
        );

        run(
            &mut p,
            "delete_item",
            json!({ "id": occ(&s, "2026-10-11T22:00:00Z"), "scope": "all" }),
        );
        assert!(week(&mut p).is_empty());
        assert!(fails(&mut p, "get_item", json!({ "id": s })).contains("not found"));
    }

    #[test]
    fn completing_an_occurrence_leaves_the_series_open() {
        let mut p = crate::planner::tests::planner();
        let s = run(
            &mut p,
            "create_item",
            json!({ "title": "Water plants", "date": "2026-10-12", "rrule": "FREQ=DAILY" }),
        );
        let s = s["id"].as_str().unwrap().to_owned();
        let done = run(
            &mut p,
            "complete_item",
            json!({ "id": format!("{s}@2026-10-13") }),
        );
        assert_eq!(done["status"], "done");
        assert_eq!(done["recurrence_id"], "2026-10-13");
        let agenda = run(
            &mut p,
            "list_agenda",
            json!({ "start": "2026-10-12", "end": "2026-10-14" }),
        );
        let statuses: Vec<&str> = agenda["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|i| i["status"].as_str().unwrap())
            .collect();
        assert_eq!(statuses, ["open", "done", "open"]);
        assert!(fails(&mut p, "complete_item", json!({ "id": s })).contains("repeats"));
        let err = fails(
            &mut p,
            "unschedule_item",
            json!({ "id": format!("{s}@2026-10-14") }),
        );
        assert!(err.contains("Inbox"), "{err}");
    }

    #[test]
    fn reminders_and_busy_time_follow_the_series() {
        let (mut p, s) = standup("FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR");
        let r = run(
            &mut p,
            "set_reminder",
            json!({ "item_id": occ(&s, "2026-10-14T22:00:00Z"), "offset_min": 5 }),
        );
        assert_eq!(r["item_id"], s.as_str());
        // Thursday 09:00–09:15 is taken by the standup.
        let slots = run(
            &mut p,
            "find_free_slots",
            json!({ "duration_min": 30, "from": "2026-10-15T08:30", "until": "2026-10-15T11:00" }),
        );
        assert_eq!(slots["slots"][0]["start"], "2026-10-14T21:30:00Z");
        assert_eq!(slots["slots"][1]["start"], "2026-10-14T22:15:00Z");
    }

    #[test]
    fn bad_rules_are_refused() {
        let mut p = crate::planner::tests::planner();
        for rule in ["every monday", "FREQ=HOURLY"] {
            let err = fails(
                &mut p,
                "create_item",
                json!({ "title": "X", "start": "2026-10-12T09:00", "rrule": rule }),
            );
            assert!(
                err.contains("rrule") || err.contains("daily"),
                "{rule}: {err}"
            );
        }
    }
}

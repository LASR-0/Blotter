//! The reminder engine's core (roadmap §4.7): which reminders fall due in
//! a window of time. The app keeps one timer for the next one and asks
//! again whenever the planner changes.
//!
//! A reminder is either `offset_min` before its item's start or at an
//! absolute instant. On a recurring series an offset reminder fires for
//! every occurrence (an override's own time wins; cancelled and done
//! occurrences are skipped). An all-day item's start counts as 09:00 local.
//! `fired_at` holds the due time of the last firing, so a restart never
//! fires the same one twice, and a snooze replaces the next firing.

use chrono::{Duration, NaiveTime};
use chrono_tz::Tz;
use rusqlite::Connection;
use serde::Serialize;

use crate::error::Result;
use crate::model::{Item, ItemStatus, Reminder};
use crate::ops::input::resolve_local;
use crate::recurrence;
use crate::store;
use crate::store::items::ItemRange;
use crate::time::Instant;

/// When an all-day item's offset reminders count back from.
const ALL_DAY_START: NaiveTime = match NaiveTime::from_hms_opt(9, 0, 0) {
    Some(t) => t,
    None => NaiveTime::MIN,
};

/// One reminder firing.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Due {
    pub reminder_id: String,
    /// The item to open: an occurrence id for a recurring series.
    pub item_id: String,
    pub title: String,
    /// The item's (or occurrence's) start, if it has one.
    pub start: Option<Instant>,
    pub end: Option<Instant>,
    pub all_day: bool,
    /// When it fires.
    pub due: Instant,
    /// Whether this firing is a snoozed one coming back.
    pub snoozed: bool,
}

/// Where an item's offset reminders count back from.
fn start_of(item: &Item, tz: Tz) -> Option<Instant> {
    match (item.start_at, item.start_date) {
        (Some(start), _) if !item.all_day => Some(start),
        (_, Some(date)) => Some(resolve_local(tz, date.and_time(ALL_DAY_START))),
        _ => None,
    }
}

/// The open instances of `item` (itself, or a series' occurrences and
/// overrides) starting in `(from, to]`.
fn instances(
    conn: &Connection,
    item: &Item,
    tz: Tz,
    from: Instant,
    to: Instant,
) -> Result<Vec<Item>> {
    if item.rrule.is_none() {
        let starts_in = start_of(item, tz).is_some_and(|s| from < s && s <= to);
        return Ok(if item.status == ItemStatus::Open && starts_in {
            vec![item.clone()]
        } else {
            Vec::new()
        });
    }
    if item.status == ItemStatus::Cancelled {
        return Ok(Vec::new());
    }
    // A day either side covers all-day occurrences counted from 09:00.
    let range = ItemRange {
        start: from - Duration::days(1),
        end: to + Duration::days(1),
        start_date: (from - Duration::days(1)).with_timezone(&tz).date_naive(),
        end_date: (to + Duration::days(1)).with_timezone(&tz).date_naive(),
    };
    let children = store::items::list_children(conn, &item.id)?;
    let overridden: Vec<&str> = children
        .iter()
        .filter_map(|c| c.recurrence_id.as_deref())
        .collect();
    let mut out: Vec<Item> = recurrence::occurrences(item, tz, range)?
        .into_iter()
        .filter(|s| !overridden.contains(&s.recurrence_id().as_str()))
        .map(|s| recurrence::occurrence_item(item, s))
        .collect();
    out.extend(
        children
            .into_iter()
            .filter(|c| c.status == ItemStatus::Open),
    );
    out.retain(|i| start_of(i, tz).is_some_and(|s| from < s && s <= to));
    Ok(out)
}

fn due_for(
    conn: &Connection,
    r: &Reminder,
    tz: Tz,
    from: Instant,
    to: Instant,
) -> Result<Vec<Due>> {
    let Some(item) = store::items::get(conn, &r.item_id)? else {
        return Ok(Vec::new());
    };
    let due = |instance: &Item, at: Instant, snoozed: bool| Due {
        reminder_id: r.id.clone(),
        item_id: instance.id.clone(),
        title: instance.title.clone(),
        start: if instance.all_day {
            start_of(instance, tz)
        } else {
            instance.start_at
        },
        end: instance.end_at,
        all_day: instance.all_day,
        due: at,
        snoozed,
    };
    if let Some(at) = r.snoozed_until {
        return Ok(if from < at && at <= to {
            vec![due(&item, at, true)]
        } else {
            Vec::new()
        });
    }
    let after_last = |at: Instant| r.fired_at.is_none_or(|f| at > f);
    if let Some(at) = r.at {
        let open = item.status == ItemStatus::Open || item.rrule.is_some();
        return Ok(if open && from < at && at <= to && after_last(at) {
            vec![due(&item, at, false)]
        } else {
            Vec::new()
        });
    }
    let Some(offset) = r.offset_min else {
        return Ok(Vec::new());
    };
    let offset = Duration::minutes(i64::from(offset));
    Ok(instances(conn, &item, tz, from + offset, to + offset)?
        .iter()
        .filter_map(|i| {
            let at = start_of(i, tz)? - offset;
            after_last(at).then(|| due(i, at, false))
        })
        .collect())
}

/// Every reminder firing due in `(from, to]`, soonest first.
pub fn due_between(conn: &Connection, tz: Tz, from: Instant, to: Instant) -> Result<Vec<Due>> {
    let mut out = Vec::new();
    for r in store::reminders::list_all(conn)? {
        out.extend(due_for(conn, &r, tz, from, to)?);
    }
    out.sort_by_key(|d| d.due);
    Ok(out)
}

/// Records that `due` fired.
pub fn mark_fired(conn: &Connection, due: &Due) -> Result<()> {
    store::reminders::set_fired(conn, &due.reminder_id, due.due)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Actor;
    use chrono::{TimeZone, Utc};
    use chrono_tz::Australia::Melbourne;
    use serde_json::{Value, json};

    fn run(p: &mut crate::Planner, op: &str, args: Value) -> Value {
        p.execute(op, args, &Actor::User).unwrap().result
    }

    fn at(d: u32, h: u32, m: u32) -> Instant {
        Utc.with_ymd_and_hms(2026, 10, d, h, m, 0).unwrap()
    }

    fn titles(due: &[Due]) -> Vec<(&str, Instant)> {
        due.iter().map(|d| (d.title.as_str(), d.due)).collect()
    }

    #[test]
    fn offset_and_absolute_reminders() {
        let mut p = crate::planner::tests::planner();
        // Dentist Thu 15 Oct 15:00 Melbourne = 04:00Z; the event's default 10m reminder.
        run(
            &mut p,
            "create_item",
            json!({ "title": "Dentist", "kind": "event", "start": "2026-10-15T15:00" }),
        );
        let task = run(&mut p, "create_item", json!({ "title": "Call bank" }));
        run(
            &mut p,
            "set_reminder",
            json!({ "item_id": task["id"], "at": "2026-10-15T12:00" }),
        );
        let conn = p.db().conn();
        let due = due_between(conn, Melbourne, at(14, 0, 0), at(16, 0, 0)).unwrap();
        assert_eq!(
            titles(&due),
            [("Call bank", at(15, 1, 0)), ("Dentist", at(15, 3, 50))]
        );

        // Once fired, it's gone; a window ending before it misses it.
        mark_fired(conn, &due[1]).unwrap();
        let again = due_between(conn, Melbourne, at(14, 0, 0), at(16, 0, 0)).unwrap();
        assert_eq!(titles(&again), [("Call bank", at(15, 1, 0))]);
        assert!(
            due_between(conn, Melbourne, at(15, 1, 0), at(15, 3, 0))
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn every_occurrence_of_a_series_fires_once() {
        let mut p = crate::planner::tests::planner();
        let s = run(
            &mut p,
            "create_item",
            json!({ "title": "Standup", "kind": "event", "start": "2026-10-12T09:00",
                    "rrule": "FREQ=DAILY", "reminders": [5] }),
        );
        let s = s["id"].as_str().unwrap().to_owned();
        // Tuesday is cancelled; Wednesday moved to 10:00.
        run(
            &mut p,
            "delete_item",
            json!({ "id": format!("{s}@2026-10-12T22:00:00Z") }),
        );
        run(
            &mut p,
            "update_item",
            json!({ "id": format!("{s}@2026-10-13T22:00:00Z"), "start": "2026-10-14T10:00" }),
        );
        let conn = p.db().conn();
        // Mon 12 – Thu 15 (Melbourne).
        let due = due_between(conn, Melbourne, at(11, 13, 0), at(15, 13, 0)).unwrap();
        let times: Vec<Instant> = due.iter().map(|d| d.due).collect();
        assert_eq!(
            times,
            [at(11, 21, 55), at(13, 22, 55), at(14, 21, 55)],
            "Mon, Wed (moved), Thu"
        );
        assert!(due[0].item_id.starts_with(&format!("{s}@")));
        mark_fired(conn, &due[1]).unwrap();
        let later = due_between(conn, Melbourne, at(11, 13, 0), at(15, 13, 0)).unwrap();
        assert_eq!(later.len(), 1, "Mon and Wed are behind the last firing");
    }

    #[test]
    fn all_day_counts_from_nine_and_done_items_are_quiet() {
        let mut p = crate::planner::tests::planner();
        let bins = run(
            &mut p,
            "create_item",
            json!({ "title": "Bin night", "date": "2026-10-14", "reminders": [60] }),
        );
        let conn = p.db().conn();
        let due = due_between(conn, Melbourne, at(13, 0, 0), at(14, 0, 0)).unwrap();
        assert_eq!(titles(&due), [("Bin night", at(13, 21, 0))], "08:00 local");
        run(&mut p, "complete_item", json!({ "id": bins["id"] }));
        assert!(
            due_between(p.db().conn(), Melbourne, at(13, 0, 0), at(14, 0, 0))
                .unwrap()
                .is_empty()
        );
    }
}

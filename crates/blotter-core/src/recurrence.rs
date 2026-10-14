//! Recurring items (RFC 5545 RRULE via the `rrule` crate).
//!
//! A series is one stored item with an `rrule`; its own start is the first
//! occurrence (DTSTART). Occurrences are expanded on read for whatever range
//! is asked for, and keep their wall-clock time across DST: a fixed item in
//! its own timezone, a flexible one in the user's.
//!
//! Each occurrence has a stable id, `<series id>@<recurrence id>`, where the
//! recurrence id is the occurrence's original start (`2026-10-15T09:00:00Z`
//! for timed series, `2026-10-15` for all-day ones). Changing one occurrence
//! stores an override: a child item with `parent_id` = the series and that
//! `recurrence_id`. A deleted occurrence is a cancelled override.

use std::collections::HashSet;

use chrono::{Duration, NaiveDate, TimeZone};
use chrono_tz::Tz;
use rrule::{RRule, RRuleSet, Unvalidated};
use rusqlite::Connection;

use crate::error::{CoreError, Result};
use crate::model::{Item, ItemStatus};
use crate::store;
use crate::store::items::ItemRange;
use crate::time::{self, Instant};

/// Separates the series id from the recurrence id in an occurrence id.
pub const OCCURRENCE_SEP: char = '@';

/// Most occurrences one series expands to in one read.
const MAX_OCCURRENCES: u16 = 4000;

pub fn occurrence_id(series_id: &str, recurrence_id: &str) -> String {
    format!("{series_id}{OCCURRENCE_SEP}{recurrence_id}")
}

/// `(series id, recurrence id)` if `id` names an occurrence.
pub fn split_occurrence_id(id: &str) -> Option<(&str, &str)> {
    id.split_once(OCCURRENCE_SEP)
        .filter(|(s, r)| !s.is_empty() && !r.is_empty())
}

/// Normalises a rule (`rrule:freq=weekly` → `FREQ=WEEKLY`) and checks it
/// parses. Sub-daily frequencies are refused: a planner has no use for
/// "every 5 minutes", and they would flood every view.
pub fn normalize_rule(rule: &str) -> Result<String> {
    let mut s = rule.trim().to_ascii_uppercase();
    if let Some(rest) = s.strip_prefix("RRULE:") {
        s = rest.trim().to_owned();
    }
    let parsed: RRule<Unvalidated> = s.parse().map_err(|e| {
        CoreError::invalid(format!(
            "rrule '{rule}' isn't a valid RFC 5545 rule like FREQ=WEEKLY;BYDAY=MO ({e})"
        ))
    })?;
    if s.contains("FREQ=SECONDLY") || s.contains("FREQ=MINUTELY") || s.contains("FREQ=HOURLY") {
        return Err(CoreError::invalid(
            "repeat at most daily (FREQ=DAILY, WEEKLY, MONTHLY or YEARLY)",
        ));
    }
    drop(parsed);
    Ok(s)
}

/// The rule with any COUNT/UNTIL replaced by `UNTIL=<until>` (UTC), so the
/// series ends before a given occurrence.
pub fn rule_until(rule: &str, until: Instant) -> String {
    let mut parts: Vec<String> = rule
        .split(';')
        .filter(|p| {
            let p = p.trim().to_ascii_uppercase();
            !p.is_empty() && !p.starts_with("COUNT=") && !p.starts_with("UNTIL=")
        })
        .map(str::to_owned)
        .collect();
    parts.push(format!("UNTIL={}", until.format("%Y%m%dT%H%M%SZ")));
    parts.join(";")
}

/// The rule's COUNT, if it has one.
pub fn rule_count(rule: &str) -> Option<u32> {
    rule.split(';').find_map(|p| {
        p.trim()
            .to_ascii_uppercase()
            .strip_prefix("COUNT=")?
            .parse()
            .ok()
    })
}

/// The rule with COUNT set to `count` (UNTIL dropped).
pub fn rule_with_count(rule: &str, count: u32) -> String {
    let mut parts: Vec<String> = rule
        .split(';')
        .filter(|p| {
            let p = p.trim().to_ascii_uppercase();
            !p.is_empty() && !p.starts_with("COUNT=") && !p.starts_with("UNTIL=")
        })
        .map(str::to_owned)
        .collect();
    parts.push(format!("COUNT={count}"));
    parts.join(";")
}

/// One occurrence's place in time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slot {
    Timed {
        start: Instant,
        end: Instant,
    },
    AllDay {
        date: NaiveDate,
        end_date: Option<NaiveDate>,
    },
}

impl Slot {
    /// The recurrence id naming this occurrence.
    pub fn recurrence_id(&self) -> String {
        match self {
            Slot::Timed { start, .. } => time::fmt_instant(*start),
            Slot::AllDay { date, .. } => time::fmt_date(*date),
        }
    }
}

fn rule_set(series: &Item, user_tz: Tz) -> Result<RRuleSet> {
    let rule = series
        .rrule
        .as_deref()
        .ok_or_else(|| CoreError::invalid(format!("'{}' doesn't repeat", series.title)))?;
    let rule: RRule<Unvalidated> = normalize_rule(rule)?
        .parse()
        .map_err(|e| CoreError::invalid(format!("bad rrule: {e}")))?;
    let start = if let (Some(start), false) = (series.start_at, series.all_day) {
        let tz: Tz = series
            .tz
            .as_deref()
            .and_then(|t| t.parse().ok())
            .unwrap_or(user_tz);
        let local = start.with_timezone(&tz).naive_local();
        rrule::Tz::from(tz)
            .from_local_datetime(&local)
            .earliest()
            .ok_or_else(|| CoreError::invalid("series start falls in a DST gap"))?
    } else if let Some(date) = series.start_date {
        // All-day series repeat by date; midnight UTC keeps the date maths exact.
        rrule::Tz::UTC.from_utc_datetime(&date.and_time(chrono::NaiveTime::MIN))
    } else {
        return Err(CoreError::invalid(format!(
            "a recurring item needs a start or date ('{}')",
            series.title
        )));
    };
    rule.build(start)
        .map_err(|e| CoreError::invalid(format!("bad rrule: {e}")))
}

/// Occurrences of `series` overlapping `range`, in order.
pub fn occurrences(series: &Item, user_tz: Tz, range: ItemRange) -> Result<Vec<Slot>> {
    let set = rule_set(series, user_tz)?;
    let utc = |t: Instant| t.with_timezone(&rrule::Tz::UTC);
    if let (Some(start), false) = (series.start_at, series.all_day) {
        let length = series
            .end_at
            .map(|e| e - start)
            .unwrap_or_else(Duration::zero);
        let dates = set
            .after(utc(range.start - length))
            .before(utc(range.end))
            .all(MAX_OCCURRENCES)
            .dates;
        Ok(dates
            .into_iter()
            .map(|d| d.with_timezone(&chrono::Utc))
            .filter(|s| *s < range.end && (*s + length > range.start || *s >= range.start))
            .map(|s| Slot::Timed {
                start: s,
                end: s + length,
            })
            .collect())
    } else {
        let first = series.start_date.unwrap_or(range.start_date);
        let span = series
            .end_date
            .map(|e| e - first)
            .unwrap_or_else(Duration::zero);
        let midnight =
            |d: NaiveDate| rrule::Tz::UTC.from_utc_datetime(&d.and_time(chrono::NaiveTime::MIN));
        let dates = set
            .after(midnight(range.start_date - span))
            .before(midnight(range.end_date))
            .all(MAX_OCCURRENCES)
            .dates;
        Ok(dates
            .into_iter()
            .map(|d| d.date_naive())
            .filter(|d| *d <= range.end_date && *d + span >= range.start_date)
            .map(|d| Slot::AllDay {
                date: d,
                end_date: (span > Duration::zero()).then(|| d + span),
            })
            .collect())
    }
}

/// The occurrence of `series` at `slot`, as an item (not stored). It keeps
/// the series' `rrule` so readers can tell it repeats.
pub fn occurrence_item(series: &Item, slot: Slot) -> Item {
    let mut item = series.clone();
    let rid = slot.recurrence_id();
    item.id = occurrence_id(&series.id, &rid);
    item.parent_id = Some(series.id.clone());
    item.recurrence_id = Some(rid);
    item.status = ItemStatus::Open;
    item.completed_at = None;
    match slot {
        Slot::Timed { start, end } => {
            item.start_at = Some(start);
            item.end_at = Some(end);
        }
        Slot::AllDay { date, end_date } => {
            item.start_date = Some(date);
            item.end_date = end_date;
        }
    }
    item
}

/// The slot an occurrence id names, if the series really has an occurrence
/// there.
pub fn find_occurrence(series: &Item, user_tz: Tz, recurrence_id: &str) -> Result<Option<Slot>> {
    let range = if let Ok(start) = time::parse_instant("recurrence_id", recurrence_id) {
        let date = start.date_naive();
        ItemRange {
            start,
            end: start + Duration::seconds(1),
            start_date: date,
            end_date: date,
        }
    } else {
        let date = time::parse_date("recurrence_id", recurrence_id)?;
        let start = date.and_time(chrono::NaiveTime::MIN).and_utc();
        ItemRange {
            start,
            end: start + Duration::days(1),
            start_date: date,
            end_date: date,
        }
    };
    Ok(occurrences(series, user_tz, range)?
        .into_iter()
        .find(|s| s.recurrence_id() == recurrence_id))
}

/// Occurrences of `series` strictly before `slot` (to carry a COUNT over
/// when a series is split).
pub fn count_before(series: &Item, user_tz: Tz, slot: Slot) -> Result<u32> {
    let first = series.start_at.unwrap_or_else(|| {
        series
            .start_date
            .map(|d| d.and_time(chrono::NaiveTime::MIN).and_utc())
            .unwrap_or_default()
    });
    let (end, end_date) = match slot {
        Slot::Timed { start, .. } => (start, start.date_naive()),
        Slot::AllDay { date, .. } => (date.and_time(chrono::NaiveTime::MIN).and_utc(), date),
    };
    let range = ItemRange {
        start: first,
        end,
        start_date: series.start_date.unwrap_or(first.date_naive()),
        end_date: end_date.pred_opt().unwrap_or(end_date),
    };
    let before = occurrences(series, user_tz, range)?
        .into_iter()
        .filter(|s| s.recurrence_id() < slot.recurrence_id())
        .count();
    Ok(u32::try_from(before).unwrap_or(u32::MAX))
}

/// Every item in `range` with recurring series expanded into their
/// occurrences: plain items, overrides (except cancelled ones) and the
/// occurrences that have no override. In display order.
pub fn agenda(conn: &Connection, range: ItemRange, user_tz: Tz) -> Result<Vec<Item>> {
    let mut items = store::items::list_in_range(conn, range)?;
    for series in store::items::list_series(conn, range)? {
        let overridden: HashSet<String> = store::items::list_children(conn, &series.id)?
            .into_iter()
            .filter_map(|c| c.recurrence_id)
            .collect();
        // A bad stored rule must not take the whole agenda down.
        let Ok(slots) = occurrences(&series, user_tz, range) else {
            continue;
        };
        items.extend(
            slots
                .into_iter()
                .filter(|s| !overridden.contains(&s.recurrence_id()))
                .map(|s| occurrence_item(&series, s)),
        );
    }
    items.sort_by(|a, b| {
        let key = |i: &Item| {
            i.start_at
                .map(time::fmt_instant)
                .or_else(|| i.start_date.map(time::fmt_date))
                .unwrap_or_default()
        };
        key(a)
            .cmp(&key(b))
            .then(a.sort_order.total_cmp(&b.sort_order))
            .then(a.created_at.cmp(&b.created_at))
    });
    Ok(items)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ItemKind;
    use crate::ops::input::start_of_day;
    use chrono::Utc;
    use chrono_tz::Australia::Melbourne;

    fn at(y: i32, m: u32, d: u32, h: u32, min: u32) -> Instant {
        Utc.with_ymd_and_hms(y, m, d, h, min, 0).unwrap()
    }

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    fn days(first: NaiveDate, last: NaiveDate) -> ItemRange {
        ItemRange {
            start: start_of_day(Melbourne, first),
            end: start_of_day(Melbourne, last.succ_opt().unwrap()),
            start_date: first,
            end_date: last,
        }
    }

    /// Standup at 09:00 Melbourne (22:00Z the day before), 15 minutes.
    fn standup(rule: &str) -> Item {
        let mut i = Item::new("s".into(), ItemKind::Event, "Standup", at(2026, 1, 1, 0, 0));
        i.start_at = Some(at(2026, 10, 11, 22, 0)); // Mon 12 Oct 09:00
        i.end_at = Some(at(2026, 10, 11, 22, 15));
        i.tz = Some("Australia/Melbourne".into());
        i.rrule = Some(rule.into());
        i
    }

    #[test]
    fn rules_are_normalised_and_checked() {
        assert_eq!(
            normalize_rule(" rrule:freq=weekly;byday=mo ").unwrap(),
            "FREQ=WEEKLY;BYDAY=MO"
        );
        assert!(normalize_rule("FREQ=FORTNIGHTLY").is_err());
        assert!(normalize_rule("every monday").is_err());
        assert!(
            normalize_rule("FREQ=HOURLY")
                .unwrap_err()
                .to_string()
                .contains("at most daily")
        );
    }

    #[test]
    fn weekday_series_expands_in_a_week() {
        let s = standup("FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR");
        let slots =
            occurrences(&s, Melbourne, days(date(2026, 10, 12), date(2026, 10, 18))).unwrap();
        assert_eq!(slots.len(), 5);
        assert_eq!(
            slots[2],
            Slot::Timed {
                start: at(2026, 10, 13, 22, 0),
                end: at(2026, 10, 13, 22, 15)
            }
        );
        let item = occurrence_item(&s, slots[2]);
        assert_eq!(item.id, "s@2026-10-13T22:00:00Z");
        assert_eq!(item.parent_id.as_deref(), Some("s"));
        assert_eq!(
            split_occurrence_id(&item.id),
            Some(("s", "2026-10-13T22:00:00Z"))
        );
    }

    #[test]
    fn wall_clock_is_kept_across_dst() {
        // Melbourne falls back on Sunday 4 April 2027: 09:00 AEDT is 22:00Z, 09:00 AEST is 23:00Z.
        let mut s = standup("FREQ=DAILY");
        s.start_at = Some(at(2027, 4, 2, 22, 0)); // Sat 3 Apr 09:00 AEDT
        s.end_at = Some(at(2027, 4, 2, 22, 15));
        let slots = occurrences(&s, Melbourne, days(date(2027, 4, 3), date(2027, 4, 5))).unwrap();
        let starts: Vec<_> = slots
            .iter()
            .map(|s| match s {
                Slot::Timed { start, .. } => *start,
                Slot::AllDay { .. } => unreachable!(),
            })
            .collect();
        // Sat 3 (AEDT), then Sun 4 and Mon 5 (AEST): still 09:00 on the wall clock.
        assert_eq!(
            starts,
            [
                at(2027, 4, 2, 22, 0),
                at(2027, 4, 3, 23, 0),
                at(2027, 4, 4, 23, 0)
            ]
        );
    }

    #[test]
    fn all_day_and_count() {
        let mut bins = Item::new(
            "b".into(),
            ItemKind::Event,
            "Bin night",
            at(2026, 1, 1, 0, 0),
        );
        bins.all_day = true;
        bins.start_date = Some(date(2026, 10, 14));
        bins.rrule = Some("FREQ=WEEKLY;COUNT=3".into());
        let slots = occurrences(
            &bins,
            Melbourne,
            days(date(2026, 10, 1), date(2026, 11, 30)),
        )
        .unwrap();
        let ids: Vec<_> = slots.iter().map(Slot::recurrence_id).collect();
        assert_eq!(ids, ["2026-10-14", "2026-10-21", "2026-10-28"]);
        assert_eq!(count_before(&bins, Melbourne, slots[2]).unwrap(), 2);
        assert_eq!(
            find_occurrence(&bins, Melbourne, "2026-10-21").unwrap(),
            Some(slots[1])
        );
        assert_eq!(
            find_occurrence(&bins, Melbourne, "2026-10-22").unwrap(),
            None
        );
    }

    #[test]
    fn rules_can_be_cut_and_recounted() {
        assert_eq!(
            rule_until("FREQ=WEEKLY;COUNT=10;BYDAY=MO", at(2026, 10, 19, 21, 59)),
            "FREQ=WEEKLY;BYDAY=MO;UNTIL=20261019T215900Z"
        );
        assert_eq!(rule_count("FREQ=DAILY;COUNT=10"), Some(10));
        assert_eq!(rule_count("FREQ=DAILY"), None);
        assert_eq!(
            rule_with_count("FREQ=DAILY;UNTIL=20270101T000000Z", 4),
            "FREQ=DAILY;COUNT=4"
        );
        let mut s = standup("FREQ=DAILY;UNTIL=20261013T230000Z");
        s.rrule = Some(rule_until("FREQ=DAILY", at(2026, 10, 13, 21, 59)));
        let slots =
            occurrences(&s, Melbourne, days(date(2026, 10, 12), date(2026, 10, 18))).unwrap();
        assert_eq!(slots.len(), 2, "Mon and Tue only");
    }
}

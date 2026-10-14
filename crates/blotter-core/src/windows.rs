//! Block-out windows: the concrete time ranges a recurring block-out covers.
//!
//! A block-out is a rule: weekdays, a wall-clock window in its own timezone,
//! an optional active date range and lifted dates. [`expand`] turns rules
//! into UTC windows for a span of time; the slot engine, the calendar
//! shading and the AI all read block-outs through it.
//!
//! Each occurrence belongs to the local date it starts on. That date is what
//! the weekdays, the active range and lifted dates are checked against, so
//! Sleep 23:00–06:30 on a Friday runs into Saturday morning, and lifting
//! Friday frees that whole night.
//!
//! Times are wall-clock, so a window keeps its local times across DST and
//! its real length changes: Sleep is an hour shorter on the spring-forward
//! night. A time that doesn't exist (inside the spring-forward gap) moves
//! forward by the gap, like everywhere else in the core.

use std::collections::HashSet;

use chrono::NaiveDate;
use chrono_tz::Tz;
use rusqlite::Connection;
use serde::Serialize;

use crate::error::{CoreError, Result};
use crate::model::{Blockout, BlockoutException};
use crate::ops::input::resolve_local;
use crate::store;
use crate::time::Instant;

/// One occurrence of a block-out, as a UTC range `[start, end)`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BlockoutWindow {
    pub blockout_id: String,
    pub label: String,
    /// The local date (in the block-out's zone) this occurrence starts on;
    /// lift it with `add_blockout_exception` on this date.
    pub date: NaiveDate,
    pub start: Instant,
    pub end: Instant,
    /// Whether fixed items may sit inside without a warning.
    pub permits_fixed: bool,
    pub color: Option<String>,
}

impl BlockoutWindow {
    /// Whether `[start, end)` overlaps this window. Touching ends don't count,
    /// so a task may end exactly when Work starts.
    pub fn overlaps(&self, start: Instant, end: Instant) -> bool {
        self.start < end && start < self.end
    }
}

/// Every stored block-out's windows overlapping `[from, to)`.
pub fn load(conn: &Connection, from: Instant, to: Instant) -> Result<Vec<BlockoutWindow>> {
    let blockouts = store::blockouts::list(conn)?;
    let exceptions = store::blockouts::list_exceptions(conn, None, None)?;
    expand(&blockouts, &exceptions, from, to)
}

/// Every window of `blockouts` overlapping `[from, to)`, sorted by start.
/// `exceptions` may hold lifted dates for any of them.
pub fn expand(
    blockouts: &[Blockout],
    exceptions: &[BlockoutException],
    from: Instant,
    to: Instant,
) -> Result<Vec<BlockoutWindow>> {
    let mut out = Vec::new();
    if to <= from {
        return Ok(out);
    }
    for b in blockouts {
        let tz: Tz = b.tz.parse().map_err(|_| CoreError::Corrupt {
            column: "tz",
            value: b.tz.clone(),
        })?;
        let lifted: HashSet<NaiveDate> = exceptions
            .iter()
            .filter(|e| e.blockout_id == b.id)
            .map(|e| e.date)
            .collect();
        // An occurrence that starts the day before `from` can still run into it.
        let first = from.with_timezone(&tz).date_naive();
        let first = first.pred_opt().unwrap_or(first);
        let last = to.with_timezone(&tz).date_naive();
        for date in first.iter_days().take_while(|d| *d <= last) {
            if !b.applies_on(date) || lifted.contains(&date) {
                continue;
            }
            let end_date = if b.wraps_midnight() {
                match date.succ_opt() {
                    Some(next) => next,
                    None => continue,
                }
            } else {
                date
            };
            let start = resolve_local(tz, date.and_time(b.start_time));
            let end = resolve_local(tz, end_date.and_time(b.end_time));
            // A window entirely inside a DST gap collapses to nothing.
            if end <= start || end <= from || start >= to {
                continue;
            }
            out.push(BlockoutWindow {
                blockout_id: b.id.clone(),
                label: b.label.clone(),
                date,
                start,
                end,
                permits_fixed: b.permits_fixed,
                color: b.color.clone(),
            });
        }
    }
    // Stable, so block-outs starting together keep their sort order.
    out.sort_by_key(|w| w.start);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Weekdays;
    use crate::ops::input::start_of_day;
    use chrono::{Duration, NaiveTime, TimeZone, Utc};
    use chrono_tz::Australia::Melbourne;

    fn at(y: i32, m: u32, d: u32, h: u32, min: u32) -> Instant {
        Utc.with_ymd_and_hms(y, m, d, h, min, 0).unwrap()
    }

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    fn blockout(id: &str, weekdays: Weekdays, start: &str, end: &str) -> Blockout {
        let t = |s: &str| NaiveTime::parse_from_str(s, "%H:%M").unwrap();
        Blockout {
            id: id.into(),
            label: id.into(),
            weekdays,
            start_time: t(start),
            end_time: t(end),
            tz: "Australia/Melbourne".into(),
            active_from: None,
            active_until: None,
            permits_fixed: true,
            color: None,
            icon: None,
            sort_order: 0,
            created_at: at(2026, 1, 1, 0, 0),
            updated_at: at(2026, 1, 1, 0, 0),
        }
    }

    fn lifted(blockout_id: &str, on: NaiveDate) -> BlockoutException {
        BlockoutException {
            id: format!("{blockout_id}-{on}"),
            blockout_id: blockout_id.into(),
            date: on,
            note: String::new(),
            created_at: at(2026, 1, 1, 0, 0),
        }
    }

    /// `[start of first, start of the day after last)` in Melbourne.
    fn days(first: NaiveDate, last: NaiveDate) -> (Instant, Instant) {
        (
            start_of_day(Melbourne, first),
            start_of_day(Melbourne, last.succ_opt().unwrap()),
        )
    }

    fn work() -> Blockout {
        blockout("work", Weekdays::WEEKDAYS, "08:30", "17:00")
    }

    fn sleep() -> Blockout {
        blockout("sleep", Weekdays::EVERY_DAY, "23:00", "06:30")
    }

    #[test]
    fn weekday_windows_in_local_time() {
        let (from, to) = days(date(2026, 10, 12), date(2026, 10, 18));
        let w = expand(&[work()], &[], from, to).unwrap();
        let dates: Vec<_> = w.iter().map(|w| w.date.to_string()).collect();
        assert_eq!(
            dates,
            [
                "2026-10-12",
                "2026-10-13",
                "2026-10-14",
                "2026-10-15",
                "2026-10-16"
            ]
        );
        // Melbourne is UTC+11 in October: 08:30–17:00 local.
        assert_eq!(w[0].start, at(2026, 10, 11, 21, 30));
        assert_eq!(w[0].end, at(2026, 10, 12, 6, 0));
    }

    #[test]
    fn midnight_wrap_covers_both_days() {
        let (from, to) = days(date(2026, 10, 14), date(2026, 10, 14));
        let w = expand(&[sleep()], &[], from, to).unwrap();
        // Tuesday night runs into Wednesday morning; Wednesday night starts at 23:00.
        let dates: Vec<_> = w.iter().map(|w| w.date.to_string()).collect();
        assert_eq!(dates, ["2026-10-13", "2026-10-14"]);
        assert_eq!(w[0].end, at(2026, 10, 13, 19, 30), "06:30 Wednesday");
        assert_eq!(w[1].start, at(2026, 10, 14, 12, 0), "23:00 Wednesday");
    }

    #[test]
    fn lifted_date_frees_the_whole_occurrence() {
        let (from, to) = days(date(2026, 10, 14), date(2026, 10, 14));
        let w = expand(
            &[sleep(), work()],
            &[
                lifted("sleep", date(2026, 10, 13)),
                lifted("work", date(2026, 10, 14)),
            ],
            from,
            to,
        )
        .unwrap();
        // Tuesday night is lifted, so Wednesday morning is free too; Work is off.
        assert_eq!(w.len(), 1);
        assert_eq!(
            (w[0].blockout_id.as_str(), w[0].date),
            ("sleep", date(2026, 10, 14))
        );
    }

    #[test]
    fn active_range_bounds_occurrences() {
        let (from, to) = days(date(2026, 10, 12), date(2026, 10, 18));
        let mut b = work();
        b.active_from = Some(date(2026, 10, 13));
        b.active_until = Some(date(2026, 10, 15));
        let dates: Vec<_> = expand(&[b], &[], from, to)
            .unwrap()
            .iter()
            .map(|w| w.date.to_string())
            .collect();
        assert_eq!(dates, ["2026-10-13", "2026-10-14", "2026-10-15"]);
    }

    #[test]
    fn dst_keeps_wall_clock_times() {
        // Melbourne springs forward 02:00 → 03:00 on Sunday 4 October 2026.
        let (from, to) = days(date(2026, 10, 3), date(2026, 10, 3));
        let w = expand(&[sleep()], &[], from, to).unwrap();
        let night = w.iter().find(|w| w.date == date(2026, 10, 3)).unwrap();
        assert_eq!(night.start, at(2026, 10, 3, 13, 0), "23:00 AEST");
        assert_eq!(night.end, at(2026, 10, 3, 19, 30), "06:30 AEDT");
        assert_eq!(night.end - night.start, Duration::minutes(6 * 60 + 30));

        // And falls back 03:00 → 02:00 on Sunday 4 April 2027.
        let (from, to) = days(date(2027, 4, 3), date(2027, 4, 3));
        let w = expand(&[sleep()], &[], from, to).unwrap();
        let night = w.iter().find(|w| w.date == date(2027, 4, 3)).unwrap();
        assert_eq!(night.end - night.start, Duration::minutes(8 * 60 + 30));
    }

    #[test]
    fn times_in_a_dst_gap_move_forward() {
        let b = blockout("gap", Weekdays::EVERY_DAY, "02:00", "02:30");
        let (from, to) = days(date(2026, 10, 4), date(2026, 10, 4));
        let w = expand(&[b], &[], from, to).unwrap();
        // 02:00 and 02:30 don't exist that night; both become 03:xx AEDT.
        assert_eq!(w.len(), 1);
        assert_eq!(w[0].start, at(2026, 10, 3, 16, 0));
        assert_eq!(w[0].end - w[0].start, Duration::minutes(30));
    }

    #[test]
    fn each_blockout_uses_its_own_zone() {
        let mut b = blockout("calls", Weekdays::EVERY_DAY, "09:00", "10:00");
        b.tz = "Europe/London".into();
        let w = expand(&[b], &[], at(2026, 10, 14, 0, 0), at(2026, 10, 15, 0, 0)).unwrap();
        // 09:00 BST is 08:00 UTC. (The 13 Oct window, 08:00Z the day before, is out of range.)
        assert_eq!(w.len(), 1);
        assert_eq!(w[0].start, at(2026, 10, 14, 8, 0));
    }

    #[test]
    fn range_edges_and_overlap_are_half_open() {
        let w = &expand(
            &[work()],
            &[],
            at(2026, 10, 11, 21, 30),
            at(2026, 10, 11, 21, 31),
        )
        .unwrap()[0];
        assert!(w.overlaps(at(2026, 10, 12, 5, 0), at(2026, 10, 12, 7, 0)));
        assert!(
            !w.overlaps(at(2026, 10, 11, 20, 30), at(2026, 10, 11, 21, 30)),
            "may end as Work starts"
        );
        assert!(!w.overlaps(at(2026, 10, 12, 6, 0), at(2026, 10, 12, 7, 0)));
        // A range ending exactly at a window's start excludes it.
        assert!(
            expand(
                &[work()],
                &[],
                at(2026, 10, 11, 13, 0),
                at(2026, 10, 11, 21, 30)
            )
            .unwrap()
            .is_empty()
        );
        let (from, to) = days(date(2026, 10, 14), date(2026, 10, 14));
        assert!(
            expand(&[work()], &[], to, from).unwrap().is_empty(),
            "inverted range"
        );
    }

    #[test]
    fn corrupt_zone_is_reported() {
        let mut b = work();
        b.tz = "Mars/Olympus".into();
        let err = expand(&[b], &[], at(2026, 10, 12, 0, 0), at(2026, 10, 13, 0, 0)).unwrap_err();
        assert!(matches!(err, CoreError::Corrupt { column: "tz", .. }));
    }
}

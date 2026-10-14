//! The slot engine (roadmap §4.5): where flexible tasks may go.
//!
//! Pure and deterministic. Free time is planning hours (the part of each day
//! flexible tasks may use at all) minus block-out windows and busy time
//! (scheduled items, padded by a buffer on both sides). Fixed items aren't
//! placed by the engine; they go exactly where they're put.

use std::collections::BTreeMap;

use chrono::{Datelike, Duration, NaiveTime, Weekday};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};

use crate::ops::input::resolve_local;
use crate::time::Instant;
use crate::windows::BlockoutWindow;

/// A `[start, end)` range of time.
pub type Interval = (Instant, Instant);

/// Slot starts are rounded up to this many minutes, so a slot after a
/// meeting ending 10:07 starts at 10:10.
const STEP_MIN: i64 = 5;

/// A wall-clock window within one day.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    #[serde(with = "crate::time::hhmm")]
    pub start: NaiveTime,
    #[serde(with = "crate::time::hhmm")]
    pub end: NaiveTime,
}

impl Span {
    fn new(start: (u32, u32), end: (u32, u32)) -> Self {
        let t = |(h, m)| NaiveTime::from_hms_opt(h, m, 0).unwrap_or(NaiveTime::MIN);
        Self {
            start: t(start),
            end: t(end),
        }
    }
}

/// When flexible tasks may be placed, per weekday; `None` means not at all
/// that day. Stored as JSON like `{"mon": {"start": "07:00", "end": "22:00"},
/// …, "sun": null}`; days left out keep the default 07:00–22:00.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PlanningHours {
    pub mon: Option<Span>,
    pub tue: Option<Span>,
    pub wed: Option<Span>,
    pub thu: Option<Span>,
    pub fri: Option<Span>,
    pub sat: Option<Span>,
    pub sun: Option<Span>,
}

impl Default for PlanningHours {
    fn default() -> Self {
        let day = Some(Span::new((7, 0), (22, 0)));
        Self {
            mon: day,
            tue: day,
            wed: day,
            thu: day,
            fri: day,
            sat: day,
            sun: day,
        }
    }
}

impl PlanningHours {
    pub fn for_day(&self, day: Weekday) -> Option<Span> {
        match day {
            Weekday::Mon => self.mon,
            Weekday::Tue => self.tue,
            Weekday::Wed => self.wed,
            Weekday::Thu => self.thu,
            Weekday::Fri => self.fri,
            Weekday::Sat => self.sat,
            Weekday::Sun => self.sun,
        }
    }
}

/// A place a task of the requested length fits.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Slot {
    pub start: Instant,
    pub end: Instant,
    /// Where the free gap this slot sits in ends, so callers can see how
    /// much room is around it.
    pub free_until: Instant,
}

/// Everything the engine needs to know about one stretch of time.
pub struct SlotEngine<'a> {
    /// The user's zone; planning hours are read in it.
    pub tz: Tz,
    pub planning: &'a PlanningHours,
    pub blockouts: &'a [BlockoutWindow],
    /// Scheduled items' time ranges.
    pub busy: &'a [Interval],
    /// Kept free before and after each busy range.
    pub buffer: Duration,
}

impl SlotEngine<'_> {
    /// Free time within `[from, to)`, in time order.
    pub fn free(&self, from: Instant, to: Instant) -> Vec<Interval> {
        if to <= from {
            return Vec::new();
        }
        let blocked = self.blocked();
        let mut out = Vec::new();
        let first = from.with_timezone(&self.tz).date_naive();
        let last = to.with_timezone(&self.tz).date_naive();
        for date in first.iter_days().take_while(|d| *d <= last) {
            let Some(span) = self.planning.for_day(date.weekday()) else {
                continue;
            };
            let open_start = resolve_local(self.tz, date.and_time(span.start)).max(from);
            let open_end = resolve_local(self.tz, date.and_time(span.end)).min(to);
            if open_end > open_start {
                subtract(&mut out, (open_start, open_end), &blocked);
            }
        }
        out
    }

    /// The earliest start in each free gap where `duration` fits, in time
    /// order, at most `limit`. Starts are rounded up to 5 minutes.
    pub fn slots(&self, duration: Duration, from: Instant, to: Instant, limit: usize) -> Vec<Slot> {
        if duration <= Duration::zero() {
            return Vec::new();
        }
        self.free(from, to)
            .into_iter()
            .filter_map(|(gap_start, gap_end)| {
                let start = round_up(gap_start);
                let end = start + duration;
                (end <= gap_end).then_some(Slot {
                    start,
                    end,
                    free_until: gap_end,
                })
            })
            .take(limit)
            .collect()
    }

    /// Block-outs and padded busy time, sorted and merged.
    fn blocked(&self) -> Vec<Interval> {
        let mut all: Vec<Interval> = self
            .blockouts
            .iter()
            .map(|w| (w.start, w.end))
            .chain(
                self.busy
                    .iter()
                    .map(|(s, e)| (*s - self.buffer, *e + self.buffer)),
            )
            .filter(|(s, e)| e > s)
            .collect();
        all.sort();
        let mut merged: Vec<Interval> = Vec::with_capacity(all.len());
        for (s, e) in all {
            match merged.last_mut() {
                Some(last) if s <= last.1 => last.1 = last.1.max(e),
                _ => merged.push((s, e)),
            }
        }
        merged
    }
}

/// For each id in `items` (id, start, end) that overlaps another, the ids
/// it overlaps, in time order. Touching ends don't overlap.
pub fn overlapping(items: &[(String, Instant, Instant)]) -> BTreeMap<String, Vec<String>> {
    let mut sorted: Vec<&(String, Instant, Instant)> = items.iter().collect();
    sorted.sort_by_key(|(_, s, _)| *s);
    let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (i, (a, _, a_end)) in sorted.iter().enumerate() {
        for (b, b_start, _) in sorted.iter().skip(i + 1) {
            if b_start >= a_end {
                break;
            }
            out.entry(a.clone()).or_default().push(b.clone());
            out.entry(b.clone()).or_default().push(a.clone());
        }
    }
    out
}

/// Appends the parts of `open` not covered by `blocked` (sorted, merged).
fn subtract(out: &mut Vec<Interval>, open: Interval, blocked: &[Interval]) {
    let (mut cursor, end) = open;
    for &(bs, be) in blocked {
        if be <= cursor {
            continue;
        }
        if bs >= end {
            break;
        }
        if bs > cursor {
            out.push((cursor, bs));
        }
        cursor = cursor.max(be);
        if cursor >= end {
            return;
        }
    }
    out.push((cursor, end));
}

fn round_up(t: Instant) -> Instant {
    let step = STEP_MIN * 60;
    let rem = t.timestamp().rem_euclid(step);
    if rem == 0 {
        t
    } else {
        t + Duration::seconds(step - rem)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::input::start_of_day;
    use chrono::{NaiveDate, TimeZone, Utc};
    use chrono_tz::Australia::Melbourne;

    /// A wall-clock time in Melbourne.
    fn mel(y: i32, m: u32, d: u32, h: u32, min: u32) -> Instant {
        Melbourne
            .with_ymd_and_hms(y, m, d, h, min, 0)
            .earliest()
            .unwrap()
            .to_utc()
    }

    fn day(y: i32, m: u32, d: u32) -> (Instant, Instant) {
        let date = NaiveDate::from_ymd_opt(y, m, d).unwrap();
        (
            start_of_day(Melbourne, date),
            start_of_day(Melbourne, date.succ_opt().unwrap()),
        )
    }

    fn window(start: Instant, end: Instant) -> BlockoutWindow {
        BlockoutWindow {
            blockout_id: "work".into(),
            label: "Work".into(),
            date: start.with_timezone(&Melbourne).date_naive(),
            start,
            end,
            permits_fixed: true,
            color: None,
        }
    }

    fn engine<'a>(
        planning: &'a PlanningHours,
        blockouts: &'a [BlockoutWindow],
        busy: &'a [Interval],
        buffer_min: i64,
    ) -> SlotEngine<'a> {
        SlotEngine {
            tz: Melbourne,
            planning,
            blockouts,
            busy,
            buffer: Duration::minutes(buffer_min),
        }
    }

    #[test]
    fn free_time_is_planning_hours_by_default() {
        let planning = PlanningHours::default();
        let (from, to) = day(2026, 10, 14);
        let free = engine(&planning, &[], &[], 0).free(from, to);
        assert_eq!(free, [(mel(2026, 10, 14, 7, 0), mel(2026, 10, 14, 22, 0))]);
    }

    #[test]
    fn blockouts_and_padded_busy_time_are_cut_out() {
        let planning = PlanningHours::default();
        let work = [window(mel(2026, 10, 14, 8, 30), mel(2026, 10, 14, 17, 0))];
        let busy = [(mel(2026, 10, 14, 18, 0), mel(2026, 10, 14, 19, 0))];
        let (from, to) = day(2026, 10, 14);
        let free = engine(&planning, &work, &busy, 10).free(from, to);
        assert_eq!(
            free,
            [
                (mel(2026, 10, 14, 7, 0), mel(2026, 10, 14, 8, 30)),
                (mel(2026, 10, 14, 17, 0), mel(2026, 10, 14, 17, 50)),
                (mel(2026, 10, 14, 19, 10), mel(2026, 10, 14, 22, 0)),
            ]
        );
    }

    #[test]
    fn slots_take_the_earliest_fit_in_each_gap() {
        let planning = PlanningHours::default();
        let work = [window(mel(2026, 10, 14, 8, 30), mel(2026, 10, 14, 17, 0))];
        let busy = [(mel(2026, 10, 14, 17, 0), mel(2026, 10, 14, 17, 47))];
        let (from, to) = day(2026, 10, 14);
        let e = engine(&planning, &work, &busy, 0);

        let starts: Vec<_> = e
            .slots(Duration::minutes(45), from, to, 10)
            .iter()
            .map(|s| s.start)
            .collect();
        // 17:47 rounds up to 17:50.
        assert_eq!(starts, [mel(2026, 10, 14, 7, 0), mel(2026, 10, 14, 17, 50)]);

        // Two hours doesn't fit before work, only in the evening.
        let long = e.slots(Duration::hours(2), from, to, 10);
        assert_eq!(long.len(), 1);
        assert_eq!(long[0].start, mel(2026, 10, 14, 17, 50));
        assert_eq!(long[0].free_until, mel(2026, 10, 14, 22, 0));

        assert_eq!(
            e.slots(Duration::minutes(45), from, to, 1).len(),
            1,
            "limit"
        );
        assert!(e.slots(Duration::zero(), from, to, 10).is_empty());
    }

    #[test]
    fn slots_respect_the_requested_range() {
        let planning = PlanningHours::default();
        let e = engine(&planning, &[], &[], 0);
        // From 13:02 (e.g. now) until a 15:00 deadline.
        let slots = e.slots(
            Duration::minutes(30),
            mel(2026, 10, 14, 13, 2),
            mel(2026, 10, 14, 15, 0),
            10,
        );
        assert_eq!(slots.len(), 1);
        assert_eq!(slots[0].start, mel(2026, 10, 14, 13, 5));
        assert_eq!(slots[0].free_until, mel(2026, 10, 14, 15, 0));
        // Nothing fits between 14:50 and a 15:00 deadline.
        assert!(
            e.slots(
                Duration::minutes(30),
                mel(2026, 10, 14, 14, 50),
                mel(2026, 10, 14, 15, 0),
                10
            )
            .is_empty()
        );
    }

    #[test]
    fn planning_hours_per_weekday() {
        let planning = PlanningHours {
            sun: None,
            sat: Some(Span::new((9, 0), (12, 0))),
            ..PlanningHours::default()
        };
        let e = engine(&planning, &[], &[], 0);
        // Sat 17 – Sun 18 October 2026.
        let (from, _) = day(2026, 10, 17);
        let (_, to) = day(2026, 10, 18);
        assert_eq!(
            e.free(from, to),
            [(mel(2026, 10, 17, 9, 0), mel(2026, 10, 17, 12, 0))]
        );
    }

    #[test]
    fn dst_day_uses_real_time() {
        // Melbourne skips 02:00–03:00 on Sunday 4 October 2026.
        let planning = PlanningHours {
            sun: Some(Span::new((1, 0), (5, 0))),
            ..PlanningHours::default()
        };
        let (from, to) = day(2026, 10, 4);
        let free = engine(&planning, &[], &[], 0).free(from, to);
        assert_eq!(free.len(), 1);
        assert_eq!(free[0].1 - free[0].0, Duration::hours(3));
        assert_eq!(
            free[0].0,
            Utc.with_ymd_and_hms(2026, 10, 3, 15, 0, 0).unwrap()
        );
    }

    #[test]
    fn overlapping_blocks_merge() {
        let planning = PlanningHours::default();
        let blocks = [
            window(mel(2026, 10, 14, 9, 0), mel(2026, 10, 14, 12, 0)),
            window(mel(2026, 10, 14, 11, 0), mel(2026, 10, 14, 13, 0)),
        ];
        let busy = [(mel(2026, 10, 14, 12, 30), mel(2026, 10, 14, 14, 0))];
        let (from, to) = day(2026, 10, 14);
        let free = engine(&planning, &blocks, &busy, 0).free(from, to);
        assert_eq!(
            free,
            [
                (mel(2026, 10, 14, 7, 0), mel(2026, 10, 14, 9, 0)),
                (mel(2026, 10, 14, 14, 0), mel(2026, 10, 14, 22, 0)),
            ]
        );
    }

    #[test]
    fn overlaps_are_reported_both_ways() {
        let t = |h, m| mel(2026, 10, 15, h, m);
        let items = vec![
            ("dentist".to_owned(), t(15, 0), t(15, 45)),
            ("pt".to_owned(), t(15, 30), t(15, 50)),
            ("call".to_owned(), t(15, 45), t(16, 0)),
            ("run".to_owned(), t(7, 0), t(7, 30)),
        ];
        let o = overlapping(&items);
        assert_eq!(o["dentist"], ["pt"]);
        assert_eq!(o["pt"], ["dentist", "call"]);
        assert_eq!(o["call"], ["pt"], "touching Dentist's end is fine");
        assert!(!o.contains_key("run"));
    }

    #[test]
    fn planning_hours_json() {
        let json = serde_json::json!({ "sun": null, "sat": { "start": "09:00", "end": "12:00" } });
        let p: PlanningHours = serde_json::from_value(json).unwrap();
        assert_eq!(p.sun, None);
        assert_eq!(p.sat, Some(Span::new((9, 0), (12, 0))));
        assert_eq!(
            p.mon,
            PlanningHours::default().mon,
            "missing days keep the default"
        );
    }
}

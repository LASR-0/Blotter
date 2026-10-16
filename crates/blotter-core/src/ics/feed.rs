//! Reading a subscribed calendar into cached events (roadmap §5.8).
//!
//! A feed is read like an import, then each series is expanded into its
//! occurrences between a year ago and two years ahead, with moved and
//! deleted occurrences applied. The cache is rebuilt on every fetch, so
//! the window moves along with the refreshes.

use std::collections::{HashMap, HashSet};

use chrono::Duration;
use chrono_tz::Tz;
use serde::Serialize;

use super::import::{self, Plan};
use super::text;
use crate::error::Result;
use crate::id::new_id;
use crate::model::{Item, SubscriptionEvent};
use crate::recurrence;
use crate::store::items::ItemRange;
use crate::time::Instant;

/// How far back and ahead of the fetch series are expanded.
const PAST_DAYS: i64 = 365;
const AHEAD_DAYS: i64 = 2 * 365;
/// Most events one subscription keeps.
pub const MAX_EVENTS: usize = 20_000;

/// A fetched calendar, ready to cache.
#[derive(Debug, Clone, Serialize)]
pub struct Feed {
    /// The calendar's own name (X-WR-CALNAME), if it gives one.
    pub name: Option<String>,
    pub events: Vec<SubscriptionEvent>,
    /// Things left out or read approximately, for the status line.
    pub warnings: Vec<String>,
}

/// The days a fetch at `now` caches, in `tz`.
pub fn window(tz: Tz, now: Instant) -> ItemRange {
    let start = now - Duration::days(PAST_DAYS);
    let end = now + Duration::days(AHEAD_DAYS);
    ItemRange {
        start,
        end,
        start_date: start.with_timezone(&tz).date_naive(),
        end_date: end.with_timezone(&tz).date_naive(),
    }
}

fn overlaps(item: &Item, range: ItemRange) -> bool {
    match (item.all_day, item.start_at, item.end_at, item.start_date) {
        (false, Some(s), e, _) => {
            let e = e.unwrap_or(s);
            s < range.end && (e > range.start || s >= range.start)
        }
        (true, _, _, Some(d)) => {
            d <= range.end_date && item.end_date.unwrap_or(d) >= range.start_date
        }
        _ => false,
    }
}

fn event(
    subscription_id: &str,
    uid: &str,
    item: &Item,
    busy: bool,
    location: &str,
) -> SubscriptionEvent {
    SubscriptionEvent {
        id: new_id(),
        subscription_id: subscription_id.to_owned(),
        uid: uid.to_owned(),
        title: item.title.clone(),
        location: location.to_owned(),
        all_day: item.all_day,
        start_at: item.start_at.filter(|_| !item.all_day),
        end_at: item.end_at.filter(|_| !item.all_day),
        start_date: item.start_date.filter(|_| item.all_day),
        end_date: item.end_date.filter(|_| item.all_day),
        busy,
    }
}

fn calendar_name(text_in: &str) -> Option<String> {
    let roots = text::parse(text_in).ok()?;
    roots
        .iter()
        .find(|r| r.name == "VCALENDAR")?
        .text("X-WR-CALNAME")
        .map(|n| n.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|n| !n.is_empty())
        .map(|n| n.chars().take(100).collect())
}

/// Reads `text` (an iCalendar feed) into events for `subscription_id`.
pub fn read(subscription_id: &str, tz: Tz, text_in: &str, now: Instant) -> Result<Feed> {
    let Plan {
        entries,
        changes,
        summary,
    } = import::plan_feed(tz, text_in, now)?;
    let range = window(tz, now);
    let mut warnings = summary.warnings;
    let mut by_uid: HashMap<&str, Vec<&import::Change>> = HashMap::new();
    for c in &changes {
        by_uid.entry(c.uid.as_str()).or_default().push(c);
    }

    let mut events = Vec::new();
    let mut broken = Vec::new();
    for e in &entries {
        // Tasks without a date have nowhere to show.
        if !e.item.is_scheduled() {
            continue;
        }
        let uid = e.item.ics_uid.as_deref().unwrap_or_default();
        if e.item.rrule.is_none() {
            if overlaps(&e.item, range) {
                events.push(event(subscription_id, uid, &e.item, e.busy, &e.location));
            }
            continue;
        }
        let Ok(slots) = recurrence::occurrences(&e.item, tz, range) else {
            broken.push(e.item.title.clone());
            continue;
        };
        let own = by_uid.get(uid).map(Vec::as_slice).unwrap_or_default();
        let replaced: HashSet<&str> = e
            .exdates
            .iter()
            .map(String::as_str)
            .chain(own.iter().map(|c| c.recurrence_id.as_str()))
            .collect();
        for slot in slots {
            if replaced.contains(slot.recurrence_id().as_str()) {
                continue;
            }
            let occurrence = recurrence::occurrence_item(&e.item, slot);
            events.push(event(
                subscription_id,
                uid,
                &occurrence,
                e.busy,
                &e.location,
            ));
        }
        for c in own {
            if let Some(item) = c.item.as_ref().filter(|i| overlaps(i, range)) {
                events.push(event(subscription_id, uid, item, c.busy, &c.location));
            }
        }
    }
    if !broken.is_empty() {
        warnings.push(format!(
            "Left out, as their repeat rule couldn't be read: {}.",
            broken.join(", ")
        ));
    }
    if events.len() > MAX_EVENTS {
        events.sort_by_key(|e| e.start_at.map(|s| (s - now).abs()).unwrap_or_default());
        events.truncate(MAX_EVENTS);
        warnings.push(format!(
            "Only the {MAX_EVENTS} events nearest today are kept."
        ));
    }
    events.sort_by(|a, b| {
        let key = |e: &SubscriptionEvent| {
            e.start_at
                .map(crate::time::fmt_instant)
                .or_else(|| e.start_date.map(crate::time::fmt_date))
                .unwrap_or_default()
        };
        key(a).cmp(&key(b))
    });
    Ok(Feed {
        name: calendar_name(text_in),
        events,
        warnings,
    })
}

#[cfg(test)]
pub(crate) mod tests {
    use chrono::{NaiveDate, TimeZone, Utc};
    use chrono_tz::Australia::Melbourne;

    use super::*;

    fn now() -> Instant {
        Utc.with_ymd_and_hms(2026, 10, 14, 0, 0, 0).unwrap()
    }

    fn at(d: u32, h: u32, m: u32) -> Instant {
        Utc.with_ymd_and_hms(2026, 10, d, h, m, 0).unwrap()
    }

    /// What a published Outlook calendar looks like: Windows zone names, a
    /// weekly meeting with one moved and one cancelled occurrence, a free
    /// event, an all-day event and a past one-off.
    pub const OUTLOOK: &str = "BEGIN:VCALENDAR\r\n\
METHOD:PUBLISH\r\n\
PRODID:Microsoft Exchange Server 2010\r\n\
VERSION:2.0\r\n\
X-WR-CALNAME:Calendar\r\n\
BEGIN:VTIMEZONE\r\n\
TZID:AUS Eastern Standard Time\r\n\
BEGIN:STANDARD\r\n\
DTSTART:16010101T030000\r\n\
TZOFFSETFROM:+1100\r\n\
TZOFFSETTO:+1000\r\n\
RRULE:FREQ=YEARLY;INTERVAL=1;BYDAY=1SU;BYMONTH=4\r\n\
END:STANDARD\r\n\
BEGIN:DAYLIGHT\r\n\
DTSTART:16010101T020000\r\n\
TZOFFSETFROM:+1000\r\n\
TZOFFSETTO:+1100\r\n\
RRULE:FREQ=YEARLY;INTERVAL=1;BYDAY=1SU;BYMONTH=10\r\n\
END:DAYLIGHT\r\n\
END:VTIMEZONE\r\n\
BEGIN:VEVENT\r\n\
RRULE:FREQ=WEEKLY;UNTIL=20261231T000000Z;INTERVAL=1;BYDAY=MO,WE;WKST=SU\r\n\
EXDATE;TZID=AUS Eastern Standard Time:20261019T100000\r\n\
UID:040000008200E00074C5B7101A82E00800000000A1B2\r\n\
SUMMARY:Team sync\r\n\
DTSTART;TZID=AUS Eastern Standard Time:20261012T100000\r\n\
DTEND;TZID=AUS Eastern Standard Time:20261012T110000\r\n\
CLASS:PUBLIC\r\n\
TRANSP:OPAQUE\r\n\
X-MICROSOFT-CDO-BUSYSTATUS:BUSY\r\n\
LOCATION:Room 4\r\n\
END:VEVENT\r\n\
BEGIN:VEVENT\r\n\
UID:040000008200E00074C5B7101A82E00800000000A1B2\r\n\
RECURRENCE-ID;TZID=AUS Eastern Standard Time:20261014T100000\r\n\
SUMMARY:Team sync (moved)\r\n\
DTSTART;TZID=AUS Eastern Standard Time:20261014T140000\r\n\
DTEND;TZID=AUS Eastern Standard Time:20261014T150000\r\n\
TRANSP:OPAQUE\r\n\
END:VEVENT\r\n\
BEGIN:VEVENT\r\n\
UID:free-1\r\n\
SUMMARY:Lunch (tentative hold)\r\n\
DTSTART:20261015T020000Z\r\n\
DTEND:20261015T030000Z\r\n\
TRANSP:TRANSPARENT\r\n\
X-MICROSOFT-CDO-BUSYSTATUS:FREE\r\n\
END:VEVENT\r\n\
BEGIN:VEVENT\r\n\
UID:allday-1\r\n\
SUMMARY:Offsite\r\n\
DTSTART;VALUE=DATE:20261016\r\n\
DTEND;VALUE=DATE:20261017\r\n\
END:VEVENT\r\n\
BEGIN:VEVENT\r\n\
UID:old-1\r\n\
SUMMARY:Long gone\r\n\
DTSTART:20200101T000000Z\r\n\
DTEND:20200101T010000Z\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

    #[test]
    fn outlook_feed_expands_with_changes_applied() {
        let feed = read("sub", Melbourne, OUTLOOK, now()).unwrap();
        assert_eq!(feed.name.as_deref(), Some("Calendar"));
        assert!(feed.warnings.is_empty(), "{:?}", feed.warnings);
        let sync: Vec<_> = feed
            .events
            .iter()
            .filter(|e| e.title.starts_with("Team sync"))
            .collect();
        // Mon/Wed from 12 Oct to the end of December: 24 slots, less the
        // cancelled 19 Oct; the 14th is moved to 14:00.
        assert_eq!(sync.len(), 23);
        assert_eq!(sync[0].start_at, Some(at(11, 23, 0)), "10:00 AEDT");
        assert_eq!(sync[0].location, "Room 4");
        assert_eq!(sync[1].title, "Team sync (moved)");
        assert_eq!(sync[1].start_at, Some(at(14, 3, 0)), "14:00 AEDT");
        assert!(!sync.iter().any(|e| e.start_at == Some(at(18, 23, 0))));
        assert!(sync.iter().all(|e| e.busy && e.uid.starts_with("0400")));

        let lunch = feed.events.iter().find(|e| e.uid == "free-1").unwrap();
        assert!(!lunch.busy);
        let offsite = feed.events.iter().find(|e| e.uid == "allday-1").unwrap();
        assert!(offsite.all_day && offsite.start_at.is_none());
        assert_eq!(offsite.start_date, NaiveDate::from_ymd_opt(2026, 10, 16));
        assert_eq!(offsite.end_date, None, "one day");
        assert!(
            !feed.events.iter().any(|e| e.uid == "old-1"),
            "outside the window"
        );
    }

    #[test]
    fn a_blotter_export_reads_as_a_feed() {
        // Subscribing to your own export must not count it as "already there".
        let mut p = crate::planner::tests::planner();
        p.execute(
            "create_item",
            serde_json::json!({ "title": "Dentist", "kind": "event", "start": "2026-10-15T15:00", "duration_min": 45 }),
            &crate::Actor::User,
        )
        .unwrap();
        let out = super::super::export(p.db().conn(), Melbourne, None, now()).unwrap();
        let feed = read("sub", Melbourne, &out.text, now()).unwrap();
        assert_eq!(feed.events.len(), 1);
        assert_eq!(feed.events[0].title, "Dentist");
    }

    #[test]
    fn garbage_is_an_error() {
        assert!(read("sub", Melbourne, "<html>Not found</html>", now()).is_err());
    }
}

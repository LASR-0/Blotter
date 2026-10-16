//! Items to an iCalendar file (roadmap §5.8): everything, or the items in a
//! date range.
//!
//! Scheduled items become VEVENTs, so tasks show in Google Calendar and
//! Apple Calendar too; Inbox tasks (whole exports only) become VTODOs. Plain
//! items are written in UTC. A recurring series is written in its own zone
//! with a VTIMEZONE, so other calendars keep its wall-clock time across DST;
//! its changed occurrences follow with RECURRENCE-ID and deleted ones become
//! EXDATEs. What iCalendar has no word for (kind, anchoring, completion…)
//! goes in X-BLOTTER-* properties, so a Blotter export imports back whole.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use chrono::{Datelike, Duration, NaiveDate};
use chrono_tz::Tz;
use rusqlite::Connection;
use serde::Serialize;

use super::text::Writer;
use super::zones;
use crate::error::Result;
use crate::model::{Item, ItemStatus, Reminder};
use crate::ops::input::start_of_day;
use crate::recurrence;
use crate::store;
use crate::store::items::ItemRange;
use crate::time::{self, Instant};

/// The UID suffix of items that didn't come with one.
pub const UID_DOMAIN: &str = "blotter";

/// What an export wrote.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Exported {
    pub text: String,
    /// VEVENTs for items and series (not counting changed occurrences).
    pub events: usize,
    /// VTODOs for Inbox tasks.
    pub todos: usize,
}

fn utc(t: Instant) -> String {
    t.format("%Y%m%dT%H%M%SZ").to_string()
}

fn date(d: NaiveDate) -> String {
    d.format("%Y%m%d").to_string()
}

/// Blotter's 3 (highest) … 1 as iCalendar's 1 (highest) … 9.
fn priority(p: u8) -> Option<&'static str> {
    match p {
        3 => Some("1"),
        2 => Some("5"),
        1 => Some("9"),
        _ => None,
    }
}

/// How one item's times are written.
enum Times {
    /// UTC, for plain timed items and changed occurrences.
    Utc,
    /// Wall-clock in this zone, for a timed series.
    Zoned(Tz),
}

fn when(w: &mut Writer, head: &str, t: Instant, times: &Times) {
    match times {
        Times::Utc => w.line(head, &utc(t)),
        Times::Zoned(tz) => w.line(
            &format!("{head};TZID={}", tz.name()),
            &t.with_timezone(tz).format("%Y%m%dT%H%M%S").to_string(),
        ),
    }
}

/// The zone a series repeats in: its own, or the user's.
fn series_tz(item: &Item, user_tz: Tz) -> Tz {
    item.tz
        .as_deref()
        .and_then(|t| t.parse().ok())
        .unwrap_or(user_tz)
}

/// The properties every component shares.
fn common(w: &mut Writer, item: &Item) {
    w.text("SUMMARY", &item.title);
    if !item.notes.trim().is_empty() {
        w.text("DESCRIPTION", &item.notes);
    }
    if !item.tags.is_empty() {
        let tags: Vec<String> = item.tags.iter().map(|t| super::text::escape(t)).collect();
        w.line("CATEGORIES", &tags.join(","));
    }
    if let Some(p) = priority(item.priority) {
        w.line("PRIORITY", p);
    }
    w.line("CREATED", &utc(item.created_at));
    w.line("LAST-MODIFIED", &utc(item.updated_at));
    w.line("X-BLOTTER-KIND", item.kind.as_str());
    w.line("X-BLOTTER-ANCHORING", item.anchoring.as_str());
    if let Some(c) = &item.color {
        w.line("X-BLOTTER-COLOR", c);
    }
    if let (ItemStatus::Done, Some(at)) = (item.status, item.completed_at) {
        w.line("X-BLOTTER-COMPLETED", &utc(at));
    }
}

/// DTSTART/DTEND (or the all-day dates) of a scheduled item.
fn schedule(w: &mut Writer, item: &Item, times: &Times) {
    if let (true, Some(d)) = (item.all_day, item.start_date) {
        let last = item.end_date.unwrap_or(d);
        w.line("DTSTART;VALUE=DATE", &date(d));
        // DTEND is exclusive for dates.
        w.line("DTEND;VALUE=DATE", &date(last + Duration::days(1)));
    } else if let Some(start) = item.start_at {
        when(w, "DTSTART", start, times);
        when(w, "DTEND", item.end_at.unwrap_or(start), times);
    }
}

fn alarms(w: &mut Writer, item: &Item, reminders: &[&Reminder]) {
    for r in reminders {
        w.begin("VALARM");
        w.line("ACTION", "DISPLAY");
        w.text("DESCRIPTION", &item.title);
        match (r.offset_min, r.at) {
            (Some(m), _) => w.line("TRIGGER", &format!("-PT{m}M")),
            (None, Some(at)) => w.line("TRIGGER;VALUE=DATE-TIME", &utc(at)),
            (None, None) => {}
        }
        w.end("VALARM");
    }
}

/// The recurrence id of an override, written as RECURRENCE-ID/EXDATE wants it.
fn rid_value(rid: &str, series: &Item, tz: Tz) -> Option<(String, String)> {
    if series.all_day {
        let d = time::parse_date("recurrence_id", rid).ok()?;
        Some(("VALUE=DATE".into(), date(d)))
    } else {
        let t = time::parse_instant("recurrence_id", rid).ok()?;
        Some((
            format!("TZID={}", tz.name()),
            t.with_timezone(&tz).format("%Y%m%dT%H%M%S").to_string(),
        ))
    }
}

/// An all-day series' UNTIL as a DATE, as RFC 5545 asks when DTSTART is one.
fn rule_for(item: &Item, rule: &str) -> String {
    if !item.all_day {
        return rule.to_owned();
    }
    rule.split(';')
        .map(|part| match part.strip_prefix("UNTIL=") {
            Some(v) if v.len() >= 8 => format!("UNTIL={}", &v[..8]),
            _ => part.to_owned(),
        })
        .collect::<Vec<_>>()
        .join(";")
}

fn in_range(item: &Item, r: &ItemRange) -> bool {
    match (item.start_at, item.start_date) {
        (Some(s), _) if !item.all_day => {
            s < r.end && (item.end_at.unwrap_or(s) > r.start || s >= r.start)
        }
        (_, Some(d)) => d <= r.end_date && item.end_date.unwrap_or(d) >= r.start_date,
        _ => false,
    }
}

/// Writes the iCalendar file: every item, or (with `range`, inclusive
/// dates in `tz`) the scheduled items and series that fall in it.
pub fn export(
    conn: &Connection,
    tz: Tz,
    range: Option<(NaiveDate, NaiveDate)>,
    now: Instant,
) -> Result<Exported> {
    let range = range.map(|(from, to)| ItemRange {
        start: start_of_day(tz, from),
        end: start_of_day(tz, to + Duration::days(1)),
        start_date: from,
        end_date: to,
    });
    let all = store::items::list_all(conn)?;
    let reminders = store::reminders::list_all(conn)?;
    let mut by_item: HashMap<&str, Vec<&Reminder>> = HashMap::new();
    for r in &reminders {
        by_item.entry(r.item_id.as_str()).or_default().push(r);
    }
    let mut children: HashMap<&str, Vec<&Item>> = HashMap::new();
    for c in all.iter().filter(|i| i.parent_id.is_some()) {
        if let Some(p) = &c.parent_id {
            children.entry(p.as_str()).or_default().push(c);
        }
    }

    let masters: Vec<&Item> = all
        .iter()
        .filter(|i| i.parent_id.is_none() && i.status != ItemStatus::Cancelled)
        .filter(|i| match &range {
            None => true,
            Some(r) if i.rrule.is_some() => {
                recurrence::occurrences(i, tz, *r).is_ok_and(|o| !o.is_empty())
                    || children
                        .get(i.id.as_str())
                        .is_some_and(|cs| cs.iter().any(|c| in_range(c, r)))
            }
            Some(r) => in_range(i, r),
        })
        .collect();

    // The zones timed series repeat in, each written once.
    let mut zones_used: BTreeMap<String, Tz> = BTreeMap::new();
    for m in masters.iter().filter(|m| m.rrule.is_some() && !m.all_day) {
        let z = series_tz(m, tz);
        zones_used.insert(z.name().to_owned(), z);
    }

    let mut w = Writer::default();
    w.begin("VCALENDAR");
    w.line("VERSION", "2.0");
    w.line(
        "PRODID",
        &format!("-//Blotter//Blotter {}//EN", crate::VERSION),
    );
    w.line("CALSCALE", "GREGORIAN");
    w.line("METHOD", "PUBLISH");
    w.text("X-WR-CALNAME", "Blotter");
    w.line("X-WR-TIMEZONE", tz.name());
    for z in zones_used.values() {
        zones::write_vtimezone(&mut w, *z, now.year());
    }

    let stamp = utc(now);
    let mut uids: BTreeSet<String> = BTreeSet::new();
    let (mut events, mut todos) = (0, 0);
    for item in masters {
        // A split series can share its UID with the original; keep UIDs unique.
        let uid = item
            .ics_uid
            .clone()
            .filter(|u| !uids.contains(u))
            .unwrap_or_else(|| format!("{}@{UID_DOMAIN}", item.id));
        uids.insert(uid.clone());
        let mine = by_item.get(item.id.as_str()).map_or(&[][..], Vec::as_slice);

        if !item.is_scheduled() {
            todos += 1;
            w.begin("VTODO");
            w.line("UID", &uid);
            w.line("DTSTAMP", &stamp);
            common(&mut w, item);
            if let Some(due) = item.due_at {
                w.line("DUE", &utc(due));
            }
            if let Some(m) = item.duration_min {
                w.line("X-BLOTTER-DURATION", &m.to_string());
            }
            match (item.status, item.completed_at) {
                (ItemStatus::Done, Some(at)) => {
                    w.line("STATUS", "COMPLETED");
                    w.line("COMPLETED", &utc(at));
                }
                _ => w.line("STATUS", "NEEDS-ACTION"),
            }
            w.end("VTODO");
            continue;
        }

        events += 1;
        let zone = series_tz(item, tz);
        let times = if item.rrule.is_some() {
            Times::Zoned(zone)
        } else {
            Times::Utc
        };
        w.begin("VEVENT");
        w.line("UID", &uid);
        w.line("DTSTAMP", &stamp);
        schedule(&mut w, item, &times);
        common(&mut w, item);
        if let Some(due) = item.due_at {
            w.line("X-BLOTTER-DUE", &utc(due));
        }
        let kids = children
            .get(item.id.as_str())
            .map_or(&[][..], Vec::as_slice);
        if let Some(rule) = &item.rrule {
            w.line("RRULE", &rule_for(item, rule));
            let cancelled: Vec<(String, String)> = kids
                .iter()
                .filter(|c| c.status == ItemStatus::Cancelled)
                .filter_map(|c| rid_value(c.recurrence_id.as_deref()?, item, zone))
                .collect();
            if let Some((param, _)) = cancelled.first() {
                let values: Vec<&str> = cancelled.iter().map(|(_, v)| v.as_str()).collect();
                w.line(&format!("EXDATE;{param}"), &values.join(","));
            }
        }
        alarms(&mut w, item, mine);
        w.end("VEVENT");

        if item.rrule.is_none() {
            continue;
        }
        for c in kids.iter().filter(|c| c.status != ItemStatus::Cancelled) {
            let Some((param, value)) = c
                .recurrence_id
                .as_deref()
                .and_then(|rid| rid_value(rid, item, zone))
            else {
                continue;
            };
            w.begin("VEVENT");
            w.line("UID", &uid);
            w.line("DTSTAMP", &stamp);
            w.line(&format!("RECURRENCE-ID;{param}"), &value);
            schedule(&mut w, c, &Times::Utc);
            common(&mut w, c);
            w.end("VEVENT");
        }
    }
    w.end("VCALENDAR");
    Ok(Exported {
        text: w.finish(),
        events,
        todos,
    })
}

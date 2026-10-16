//! An iCalendar file to items (roadmap §5.8). Reading is separate from
//! writing: [`plan`] works out what the file holds and what's already in
//! Blotter (by UID), so the user sees a summary before anything changes;
//! the `import_ics` operation then writes the plan as one undoable batch.
//!
//! VEVENTs become events (or, from a Blotter export, whatever kind they
//! were), VTODOs tasks: dated ones on their day, the rest in the Inbox.
//! RRULE, EXDATE and RECURRENCE-ID become a series with its deleted and
//! changed occurrences; VALARMs before the start become reminders;
//! CATEGORIES become tags. Times keep the zone they were written in.

use std::collections::{BTreeSet, HashMap, HashSet};

use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use chrono_tz::Tz;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use super::export::UID_DOMAIN;
use super::text::{self, Component, Property};
use super::zones;
use crate::error::Result;
use crate::model::{Anchoring, Item, ItemKind, ItemStatus};
use crate::ops::input::resolve_local;
use crate::recurrence;
use crate::store::items::ItemRange;
use crate::time::{self, Instant};

/// What an import adds, shown before the user confirms and after it ran.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ImportSummary {
    /// New items with a time or date.
    pub scheduled: usize,
    /// New tasks without one, for the Inbox.
    pub inbox: usize,
    /// How many of the scheduled ones repeat.
    pub recurring: usize,
    /// Changed or deleted occurrences of the new series.
    pub occurrences: usize,
    pub reminders: usize,
    /// Left out because an earlier import (or the item itself) is there.
    pub already: usize,
    /// Left out: cancelled, or without a start.
    pub skipped: usize,
    /// The first and last day the new items are on.
    pub first: Option<NaiveDate>,
    pub last: Option<NaiveDate>,
    /// Things read differently than written, in words.
    pub warnings: Vec<String>,
}

impl ImportSummary {
    /// Whether the import would add nothing.
    pub fn is_empty(&self) -> bool {
        self.scheduled + self.inbox + self.occurrences == 0
    }
}

/// A reminder from a VALARM.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Alarm {
    Before(u32),
    At(Instant),
}

/// One new item (or series) to create. Its id is given when it's written.
#[derive(Debug, Clone)]
pub(crate) struct Entry {
    pub item: Item,
    pub alarms: Vec<Alarm>,
    /// Recurrence ids of deleted occurrences.
    pub exdates: Vec<String>,
    /// Not marked free (only subscriptions use it).
    pub busy: bool,
    pub location: String,
}

/// A changed (`Some`) or deleted (`None`) occurrence of a new series.
#[derive(Debug, Clone)]
pub(crate) struct Change {
    pub uid: String,
    pub recurrence_id: String,
    pub item: Option<Item>,
    pub busy: bool,
    pub location: String,
}

#[derive(Debug, Clone)]
pub(crate) struct Plan {
    pub entries: Vec<Entry>,
    pub changes: Vec<Change>,
    pub summary: ImportSummary,
}

/// A DATE or DATE-TIME value.
#[derive(Debug, Clone, Copy, PartialEq)]
enum When {
    Date(NaiveDate),
    /// The instant, and the zone it was written in (`None` for UTC or floating).
    Time(Instant, Option<Tz>),
}

impl When {
    fn date_in(self, tz: Tz) -> NaiveDate {
        match self {
            When::Date(d) => d,
            When::Time(t, _) => t.with_timezone(&tz).date_naive(),
        }
    }
}

/// Reads values, remembering zones it didn't know.
struct Reader {
    tz: Tz,
    unknown_zones: BTreeSet<String>,
}

impl Reader {
    fn values(&mut self, p: &Property) -> Vec<When> {
        p.value
            .split(',')
            .filter_map(|v| self.value(p, v.trim()))
            .collect()
    }

    fn when(&mut self, p: Option<&Property>) -> Option<When> {
        let p = p?;
        self.value(p, p.value.trim())
    }

    fn value(&mut self, p: &Property, v: &str) -> Option<When> {
        if p.param("VALUE")
            .is_some_and(|t| t.eq_ignore_ascii_case("DATE"))
            || v.len() == 8
        {
            return NaiveDate::parse_from_str(v.get(..8)?, "%Y%m%d")
                .ok()
                .map(When::Date);
        }
        let (local, utc) = match v.strip_suffix(['Z', 'z']) {
            Some(rest) => (rest, true),
            None => (v, false),
        };
        let ndt = ["%Y%m%dT%H%M%S", "%Y%m%dT%H%M"]
            .iter()
            .find_map(|f| NaiveDateTime::parse_from_str(local, f).ok())?;
        if utc {
            return Some(When::Time(Utc.from_utc_datetime(&ndt), None));
        }
        let zone = p.param("TZID").and_then(|id| {
            let z = zones::resolve_tzid(id);
            if z.is_none() {
                self.unknown_zones.insert(id.to_owned());
            }
            z
        });
        Some(When::Time(
            resolve_local(zone.unwrap_or(self.tz), ndt),
            zone,
        ))
    }
}

/// `P1DT2H`, `-PT15M`, `P1W`.
fn parse_duration(s: &str) -> Option<Duration> {
    let s = s.trim();
    let (neg, s) = match s.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, s.strip_prefix('+').unwrap_or(s)),
    };
    let s = s.strip_prefix(['P', 'p'])?;
    let mut total = Duration::zero();
    let mut num = String::new();
    let mut in_time = false;
    for c in s.chars() {
        match c.to_ascii_uppercase() {
            'T' => in_time = true,
            d if d.is_ascii_digit() => num.push(d),
            unit => {
                let n: i64 = std::mem::take(&mut num).parse().ok()?;
                total += match (unit, in_time) {
                    ('W', false) => Duration::weeks(n),
                    ('D', false) => Duration::days(n),
                    ('H', true) => Duration::hours(n),
                    ('M', true) => Duration::minutes(n),
                    ('S', true) => Duration::seconds(n),
                    _ => return None,
                };
            }
        }
    }
    if !num.is_empty() {
        return None;
    }
    Some(if neg { -total } else { total })
}

/// iCalendar's 1 (highest) … 9 as Blotter's 3 … 1.
fn priority(p: Option<&Property>) -> u8 {
    match p.and_then(|p| p.value.trim().parse::<u8>().ok()) {
        Some(1..=4) => 3,
        Some(5) => 2,
        Some(6..=9) => 1,
        _ => 0,
    }
}

/// CATEGORIES as Blotter tags: lower case, no spaces, at most 32 characters.
fn tags(c: &Component) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for p in c.props_named("CATEGORIES") {
        for raw in p.texts() {
            let tag: String = raw
                .trim()
                .trim_start_matches('#')
                .to_lowercase()
                .split_whitespace()
                .collect::<Vec<_>>()
                .join("-")
                .chars()
                .take(32)
                .collect();
            if !tag.is_empty() && !out.contains(&tag) {
                out.push(tag);
            }
        }
    }
    out
}

fn title(c: &Component) -> String {
    let t = c.text("SUMMARY").unwrap_or_else(|| "(no title)".into());
    let t = t.split_whitespace().collect::<Vec<_>>().join(" ");
    t.chars().take(500).collect()
}

fn notes(c: &Component) -> String {
    let notes = c
        .text("DESCRIPTION")
        .map(|d| d.trim().replace("\r\n", "\n"))
        .unwrap_or_default();
    match c.text("LOCATION") {
        Some(loc) if notes.is_empty() => format!("Location: {}", loc.trim()),
        Some(loc) => format!("{notes}\n\nLocation: {}", loc.trim()),
        None => notes,
    }
}

fn utc_prop(c: &Component, name: &str) -> Option<Instant> {
    let v = c.prop(name)?.value.trim().trim_end_matches(['Z', 'z']);
    NaiveDateTime::parse_from_str(v, "%Y%m%dT%H%M%S")
        .ok()
        .map(|t| Utc.from_utc_datetime(&t))
}

fn alarms(c: &Component, reader: &mut Reader) -> Vec<Alarm> {
    let mut out = Vec::new();
    for a in c.children_named("VALARM") {
        let Some(trigger) = a.prop("TRIGGER") else {
            continue;
        };
        let alarm = if trigger
            .param("VALUE")
            .is_some_and(|v| v.eq_ignore_ascii_case("DATE-TIME"))
        {
            match reader.when(Some(trigger)) {
                Some(When::Time(t, _)) => Some(Alarm::At(t)),
                _ => None,
            }
        } else if trigger
            .param("RELATED")
            .is_some_and(|r| r.eq_ignore_ascii_case("END"))
        {
            None
        } else {
            // Only alarms at or before the start: Blotter's are "N minutes before".
            parse_duration(&trigger.value)
                .filter(|d| *d <= Duration::zero())
                .and_then(|d| u32::try_from(-d.num_minutes()).ok())
                .map(Alarm::Before)
        };
        if let Some(alarm) = alarm.filter(|a| !out.contains(a)) {
            out.push(alarm);
        }
    }
    out
}

/// The RRULE with any UNTIL in UTC, as Blotter's recurrence engine needs.
fn rule(value: &str, start: When, tz: Tz) -> String {
    value
        .trim()
        .split(';')
        .filter(|p| !p.trim().is_empty())
        .map(|part| {
            let part = part.trim();
            let upper = part.to_ascii_uppercase();
            let Some(until) = upper.strip_prefix("UNTIL=").filter(|u| !u.ends_with('Z')) else {
                return part.to_owned();
            };
            let zone = match start {
                When::Time(_, Some(z)) => z,
                _ => tz,
            };
            let at = match NaiveDate::parse_from_str(until, "%Y%m%d") {
                // A date: the whole day counts. All-day series repeat at
                // midnight UTC, timed ones in their zone.
                Ok(d) => d.and_hms_opt(23, 59, 59).map(|t| match start {
                    When::Date(_) => t.and_utc(),
                    When::Time(..) => resolve_local(zone, t),
                }),
                Err(_) => NaiveDateTime::parse_from_str(until, "%Y%m%dT%H%M%S")
                    .ok()
                    .map(|t| resolve_local(zone, t)),
            };
            match at {
                Some(t) => format!("UNTIL={}", t.format("%Y%m%dT%H%M%SZ")),
                None => part.to_owned(),
            }
        })
        .collect::<Vec<_>>()
        .join(";")
}

/// Whether Blotter can expand `item`'s rule (checked on its first weeks).
fn rule_works(item: &Item, tz: Tz) -> bool {
    let Ok(rule) = recurrence::normalize_rule(item.rrule.as_deref().unwrap_or_default()) else {
        return false;
    };
    let mut probe = item.clone();
    probe.rrule = Some(rule);
    let start = item.start_at.or_else(|| {
        item.start_date
            .map(|d| d.and_time(NaiveTime::MIN).and_utc())
    });
    let Some(start) = start else {
        return false;
    };
    let range = ItemRange {
        start,
        end: start + Duration::days(400),
        start_date: start.date_naive(),
        end_date: start.date_naive() + Duration::days(400),
    };
    recurrence::occurrences(&probe, tz, range).is_ok()
}

/// The recurrence id Blotter gives the occurrence `w` of `series`.
fn recurrence_id(series: &Item, w: When, tz: Tz) -> String {
    match (series.all_day, w) {
        (true, w) => time::fmt_date(w.date_in(tz)),
        (false, When::Time(t, _)) => time::fmt_instant(t),
        (false, When::Date(d)) => {
            // A date-only RECURRENCE-ID on a timed series: that day at the series' time.
            let zone = series_zone(series, tz);
            let at = series
                .start_at
                .map_or(NaiveTime::MIN, |s| s.with_timezone(&zone).time());
            time::fmt_instant(resolve_local(zone, d.and_time(at)))
        }
    }
}

fn series_zone(series: &Item, tz: Tz) -> Tz {
    series
        .tz
        .as_deref()
        .and_then(|t| t.parse().ok())
        .unwrap_or(tz)
}

/// Reads one VEVENT or VTODO into an item. `None` if it can't be placed.
fn read_item(c: &Component, reader: &mut Reader, now: Instant) -> Option<(Item, Option<When>)> {
    let todo = c.name == "VTODO";
    let kind = c
        .prop("X-BLOTTER-KIND")
        .and_then(|p| p.value.trim().parse::<ItemKind>().ok())
        .unwrap_or(if todo {
            ItemKind::Task
        } else {
            ItemKind::Event
        });
    let mut item = Item::new(String::new(), kind, title(c), now);
    if let Some(a) = c
        .prop("X-BLOTTER-ANCHORING")
        .and_then(|p| p.value.trim().parse::<Anchoring>().ok())
    {
        item.anchoring = a;
    }
    item.notes = notes(c);
    item.tags = tags(c);
    item.priority = priority(c.prop("PRIORITY"));
    item.color = c
        .prop("X-BLOTTER-COLOR")
        .map(|p| p.value.trim().to_owned())
        .filter(|v| {
            v.len() == 7 && v.starts_with('#') && v[1..].chars().all(|c| c.is_ascii_hexdigit())
        });

    let completed = utc_prop(c, "X-BLOTTER-COMPLETED").or_else(|| {
        let done = c
            .prop("STATUS")
            .is_some_and(|s| s.value.trim().eq_ignore_ascii_case("COMPLETED"));
        utc_prop(c, "COMPLETED").or(done.then_some(now))
    });
    if let Some(at) = completed {
        item.status = ItemStatus::Done;
        item.completed_at = Some(at);
    }

    let start = reader.when(c.prop("DTSTART"));
    let due = reader.when(c.prop(if todo { "DUE" } else { "X-BLOTTER-DUE" }));
    item.due_at = due.map(|d| match d {
        When::Time(t, _) => t,
        // A due date: the end of that day.
        When::Date(d) => resolve_local(reader.tz, d.and_hms_opt(23, 59, 0).unwrap_or_default()),
    });
    item.duration_min = c
        .prop("X-BLOTTER-DURATION")
        .and_then(|p| p.value.trim().parse().ok());

    let Some(start) = start else {
        // Only a task can wait in the Inbox.
        return todo.then_some((item, None));
    };
    // A VTODO's DUE is its deadline, not its end.
    let dtend = if todo { None } else { c.prop("DTEND") };
    let end = reader.when(dtend).or_else(|| {
        let d = parse_duration(&c.prop("DURATION")?.value)?;
        Some(match start {
            When::Date(s) => When::Date(s + Duration::days(d.num_days().max(1))),
            When::Time(s, z) => When::Time(s + d, z),
        })
    });
    match start {
        When::Date(d) => {
            item.all_day = true;
            item.start_date = Some(d);
            // DTEND is exclusive for dates.
            let last = end.map(|e| e.date_in(reader.tz) - Duration::days(1));
            item.end_date = last.filter(|l| *l > d);
        }
        When::Time(s, zone) => {
            let e = match end {
                Some(When::Time(e, _)) if e >= s => e,
                _ => s,
            };
            item.start_at = Some(s);
            item.end_at = Some(e);
            item.duration_min = u32::try_from((e - s).num_minutes())
                .ok()
                .filter(|m| *m > 0)
                .or(item.duration_min);
            item.tz = zone.map(|z| z.name().to_owned());
        }
    }
    Some((item, Some(start)))
}

/// UIDs already in Blotter: imported ones, and Blotter's own `<id>@blotter`.
fn known(conn: &Connection, uid: &str, imported: &HashSet<String>) -> Result<bool> {
    if imported.contains(uid) {
        return Ok(true);
    }
    match uid.strip_suffix(&format!("@{UID_DOMAIN}")) {
        Some(id) => Ok(crate::store::items::get(conn, id)?.is_some()),
        None => Ok(false),
    }
}

fn plural(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

/// Works out what importing `text` would add, without writing anything.
pub(crate) fn plan(conn: &Connection, tz: Tz, text_in: &str, now: Instant) -> Result<Plan> {
    let mut imported = HashSet::new();
    {
        let mut stmt = conn
            .prepare("SELECT ics_uid FROM items WHERE ics_uid IS NOT NULL AND parent_id IS NULL")?;
        for uid in stmt.query_map([], |r| r.get::<_, String>(0))? {
            imported.insert(uid?);
        }
    }
    plan_with(tz, text_in, now, &mut |uid| known(conn, uid, &imported))
}

/// Reads a subscribed calendar: like [`plan`], but nothing counts as
/// already imported, since the feed is cached apart from the items.
pub(crate) fn plan_feed(tz: Tz, text_in: &str, now: Instant) -> Result<Plan> {
    plan_with(tz, text_in, now, &mut |_| Ok(false))
}

/// Whether an event shows as busy: not marked free (TRANSP, or Outlook's
/// busy status).
fn busy(c: &Component) -> bool {
    let is = |name: &str, value: &str| {
        c.prop(name)
            .is_some_and(|p| p.value.trim().eq_ignore_ascii_case(value))
    };
    !is("TRANSP", "TRANSPARENT") && !is("X-MICROSOFT-CDO-BUSYSTATUS", "FREE")
}

fn location(c: &Component) -> String {
    c.text("LOCATION")
        .map(|l| l.trim().to_owned())
        .unwrap_or_default()
}

fn plan_with(
    tz: Tz,
    text_in: &str,
    now: Instant,
    is_known: &mut dyn FnMut(&str) -> Result<bool>,
) -> Result<Plan> {
    let roots = text::parse(text_in)?;
    let mut reader = Reader {
        tz,
        unknown_zones: BTreeSet::new(),
    };
    let mut summary = ImportSummary::default();
    let mut entries: Vec<Entry> = Vec::new();
    let mut changes: Vec<Change> = Vec::new();
    // UID → (index in entries, the series' DTSTART) for this file's series.
    let mut series: HashMap<String, (usize, When)> = HashMap::new();
    let mut seen: HashSet<String> = HashSet::new();
    let mut known_skipped: HashSet<String> = HashSet::new();
    let (mut dropped_rules, mut extra_dates) = (Vec::new(), 0);
    let mut overrides: Vec<&Component> = Vec::new();

    let components = roots
        .iter()
        .filter(|r| r.name == "VCALENDAR")
        .flat_map(|cal| cal.children.iter())
        .filter(|c| c.name == "VEVENT" || c.name == "VTODO");
    for c in components {
        if c.prop("RECURRENCE-ID").is_some() {
            overrides.push(c);
            continue;
        }
        let uid = c.text("UID").map(|u| u.trim().to_owned());
        if let Some(uid) = &uid {
            if is_known(uid)? {
                summary.already += 1;
                known_skipped.insert(uid.clone());
                continue;
            }
            if !seen.insert(uid.clone()) {
                summary.skipped += 1;
                continue;
            }
        }
        if c.prop("STATUS")
            .is_some_and(|s| s.value.trim().eq_ignore_ascii_case("CANCELLED"))
        {
            summary.skipped += 1;
            continue;
        }
        let Some((mut item, start)) = read_item(c, &mut reader, now) else {
            summary.skipped += 1;
            continue;
        };
        item.ics_uid = uid.clone();
        let mut exdates = Vec::new();
        if let (Some(rule_prop), Some(start)) = (c.prop("RRULE"), start) {
            item.rrule = Some(rule(&rule_prop.value, start, tz));
            if rule_works(&item, tz) {
                for p in c.props_named("EXDATE") {
                    for w in reader.values(p) {
                        exdates.push(recurrence_id(&item, w, tz));
                    }
                }
                if c.prop("RDATE").is_some() {
                    extra_dates += 1;
                }
                if let Some(uid) = &uid {
                    series.insert(uid.clone(), (entries.len(), start));
                }
            } else {
                dropped_rules.push(item.title.clone());
                item.rrule = None;
            }
        }
        let alarms = if item.is_scheduled() {
            alarms(c, &mut reader)
        } else {
            Vec::new()
        };
        entries.push(Entry {
            busy: busy(c),
            location: location(c),
            item,
            alarms,
            exdates,
        });
    }

    for c in overrides {
        let Some(uid) = c.text("UID").map(|u| u.trim().to_owned()) else {
            summary.skipped += 1;
            continue;
        };
        if known_skipped.contains(&uid) {
            continue;
        }
        let cancelled = c
            .prop("STATUS")
            .is_some_and(|s| s.value.trim().eq_ignore_ascii_case("CANCELLED"));
        let Some((index, _)) = series.get(&uid).copied() else {
            // No series for it here: a lone occurrence is just an event.
            if cancelled {
                summary.skipped += 1;
            } else if let Some((item, _)) = read_item(c, &mut reader, now) {
                let alarms = alarms(c, &mut reader);
                entries.push(Entry {
                    busy: busy(c),
                    location: location(c),
                    item,
                    alarms,
                    exdates: Vec::new(),
                });
            } else {
                summary.skipped += 1;
            }
            continue;
        };
        let Some(rid_when) = reader.when(c.prop("RECURRENCE-ID")) else {
            summary.skipped += 1;
            continue;
        };
        let rid = recurrence_id(&entries[index].item, rid_when, tz);
        let item = if cancelled {
            None
        } else {
            match read_item(c, &mut reader, now) {
                Some((item, _)) => Some(item),
                None => {
                    summary.skipped += 1;
                    continue;
                }
            }
        };
        changes.push(Change {
            busy: busy(c),
            location: location(c),
            uid,
            recurrence_id: rid,
            item,
        });
    }

    for e in &entries {
        if e.item.is_scheduled() {
            summary.scheduled += 1;
        } else {
            summary.inbox += 1;
        }
        if e.item.rrule.is_some() {
            summary.recurring += 1;
        }
        summary.reminders += e.alarms.len();
        summary.occurrences += e.exdates.len();
        let day = e
            .item
            .start_date
            .or_else(|| e.item.start_at.map(|s| s.with_timezone(&tz).date_naive()));
        if let Some(d) = day {
            summary.first = Some(summary.first.map_or(d, |f| f.min(d)));
            summary.last = Some(summary.last.map_or(d, |l| l.max(d)));
        }
    }
    summary.occurrences += changes.len();

    if !reader.unknown_zones.is_empty() {
        let names: Vec<String> = reader.unknown_zones.into_iter().collect();
        summary.warnings.push(format!(
            "Unknown time zone {}: those times were read in yours ({}).",
            names.join(", "),
            tz.name()
        ));
    }
    if !dropped_rules.is_empty() {
        summary.warnings.push(format!(
            "Imported once only, as Blotter can't follow how {} repeat{}: {}.",
            if dropped_rules.len() == 1 {
                "it"
            } else {
                "they"
            },
            if dropped_rules.len() == 1 { "s" } else { "" },
            dropped_rules.join(", ")
        ));
    }
    if extra_dates > 0 {
        summary.warnings.push(format!(
            "{} extra dates (RDATE) that were left out.",
            plural(extra_dates, "series has", "series have")
        ));
    }
    Ok(Plan {
        entries,
        changes,
        summary,
    })
}

/// What importing `text` would add, for the user to confirm.
pub fn preview(conn: &Connection, tz: Tz, text: &str, now: Instant) -> Result<ImportSummary> {
    Ok(plan(conn, tz, text, now)?.summary)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn durations() {
        assert_eq!(parse_duration("PT15M"), Some(Duration::minutes(15)));
        assert_eq!(parse_duration("-P1DT2H"), Some(-Duration::hours(26)));
        assert_eq!(parse_duration("P2W"), Some(Duration::weeks(2)));
        assert_eq!(parse_duration("PT0S"), Some(Duration::zero()));
        assert_eq!(parse_duration("15M"), None);
        assert_eq!(parse_duration("PT15"), None);
    }

    #[test]
    fn until_becomes_utc() {
        let mel = chrono_tz::Australia::Melbourne;
        let start = When::Time(Utc::now(), Some(mel));
        // 23:59:59 on the 30th in Melbourne (AEDT, +11) is 12:59:59Z.
        assert_eq!(
            rule("FREQ=WEEKLY;UNTIL=20261030;BYDAY=MO", start, mel),
            "FREQ=WEEKLY;UNTIL=20261030T125959Z;BYDAY=MO"
        );
        assert_eq!(
            rule("FREQ=DAILY;UNTIL=20261030T090000", start, mel),
            "FREQ=DAILY;UNTIL=20261029T220000Z"
        );
        assert_eq!(
            rule("FREQ=DAILY;UNTIL=20261030T090000Z", start, mel),
            "FREQ=DAILY;UNTIL=20261030T090000Z"
        );
        let day = When::Date(NaiveDate::from_ymd_opt(2026, 10, 1).unwrap());
        assert_eq!(
            rule("FREQ=WEEKLY;UNTIL=20261029", day, mel),
            "FREQ=WEEKLY;UNTIL=20261029T235959Z"
        );
    }
}

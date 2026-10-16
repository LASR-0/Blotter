//! iCalendar (RFC 5545) export and file import (roadmap §5.8). The text
//! layer, zones, export and the import plan live here; the import is
//! written by the `import_ics` operation, so it is one undoable batch.

pub mod export;
pub mod feed;
pub mod import;
pub mod text;
pub mod zones;

use std::path::Path;

use serde::Serialize;
use serde_json::json;

pub use export::{Exported, export};
pub use import::{ImportSummary, preview};

use crate::Planner;
use crate::error::{CoreError, Result};
use crate::model::Actor;
use crate::snapshots::{self, Reason, Snapshot};

/// What [`import`] did.
#[derive(Debug, Clone, Serialize)]
pub struct Imported {
    pub summary: ImportSummary,
    /// The "before ICS import" snapshot; none if nothing was new.
    pub snapshot: Option<Snapshot>,
    /// The change-log batch, for undo.
    pub batch_id: Option<String>,
}

/// Imports `text` as the user: a "before ICS import" snapshot, then the
/// `import_ics` operation. A file that can't be read leaves no snapshot.
pub fn import(planner: &mut Planner, db_path: &Path, text: &str) -> Result<Imported> {
    let now = planner.now();
    let tz = planner.settings()?.tz;
    let preview = import::preview(planner.db().conn(), tz, text, now)?;
    let snapshot = if preview.is_empty() {
        None
    } else {
        let conn = planner.db().conn();
        let dir = snapshots::dir(conn, db_path)?;
        Some(snapshots::create(
            conn,
            &dir,
            Reason::BeforeIcsImport,
            None,
            now,
        )?)
    };
    let out = planner.execute("import_ics", json!({ "ics": text }), &Actor::User)?;
    let summary = serde_json::from_value(out.result)
        .map_err(|e| CoreError::invalid(format!("import summary: {e}")))?;
    Ok(Imported {
        summary,
        snapshot,
        batch_id: out.batch_id,
    })
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveDate, TimeZone, Utc};
    use chrono_tz::Australia::Melbourne;
    use serde_json::{Value, json};

    use super::*;
    use crate::model::{Item, ItemStatus, Source};
    use crate::store;
    use crate::time::Instant;

    fn now() -> Instant {
        crate::planner::tests::now()
    }

    fn run(p: &mut Planner, op: &str, args: Value) -> Value {
        p.execute(op, args, &Actor::User).unwrap().result
    }

    fn items(p: &Planner) -> Vec<Item> {
        store::items::list_all(p.db().conn()).unwrap()
    }

    fn find<'a>(items: &'a [Item], title: &str) -> &'a Item {
        items
            .iter()
            .find(|i| i.title == title && i.parent_id.is_none())
            .unwrap_or_else(|| panic!("no {title}"))
    }

    /// A week of everything: plain, all-day, Inbox, recurring with a moved
    /// and a deleted occurrence, reminders, tags, a done task.
    fn seed(p: &mut Planner) {
        run(
            p,
            "create_item",
            json!({ "title": "Dentist, then shops; maybe", "kind": "event", "start": "2026-10-15T15:00",
                    "duration_min": 45, "reminders": [30, 5], "tags": ["health"], "priority": 3,
                    "notes": "Bring card\nAsk about crown" }),
        );
        run(
            p,
            "create_item",
            json!({ "title": "Mum visiting", "kind": "event", "date": "2026-10-17",
                    "end_date": "2026-10-18", "reminders": [] }),
        );
        run(
            p,
            "create_item",
            json!({ "title": "Call plumber", "tags": ["home"], "duration_min": 20 }),
        );
        let done = run(
            p,
            "create_item",
            json!({ "title": "Gym", "start": "2026-10-14T07:00", "duration_min": 45 }),
        );
        run(p, "complete_item", json!({ "id": done["id"] }));
        let s = run(
            p,
            "create_item",
            json!({ "title": "Standup", "kind": "event", "start": "2026-10-12T09:00",
                    "duration_min": 15, "rrule": "FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR", "reminders": [] }),
        );
        let sid = s["id"].as_str().unwrap();
        // Tue 13 moved to 10:00; Wed 14 deleted.
        run(
            p,
            "update_item",
            json!({ "id": format!("{sid}@2026-10-12T22:00:00Z"), "start": "2026-10-13T10:00" }),
        );
        run(
            p,
            "delete_item",
            json!({ "id": format!("{sid}@2026-10-13T22:00:00Z") }),
        );
        run(
            p,
            "create_item",
            json!({ "title": "Bin night", "kind": "event", "date": "2026-10-14",
                    "rrule": "FREQ=WEEKLY;COUNT=4", "reminders": [] }),
        );
    }

    #[test]
    fn export_reimports_to_the_same_items() {
        let mut a = crate::planner::tests::planner();
        seed(&mut a);
        let out = export(a.db().conn(), Melbourne, None, now()).unwrap();
        assert_eq!((out.events, out.todos), (5, 1), "{}", out.text);
        assert!(
            out.text
                .contains("BEGIN:VTIMEZONE\r\nTZID:Australia/Melbourne")
        );
        assert!(
            out.text
                .contains("DTSTART;TZID=Australia/Melbourne:20261012T090000")
        );
        assert!(
            out.text
                .contains("RECURRENCE-ID;TZID=Australia/Melbourne:20261013T090000")
        );
        assert!(
            out.text
                .contains("EXDATE;TZID=Australia/Melbourne:20261014T090000")
        );
        assert!(
            out.text
                .contains("DTSTART;VALUE=DATE:20261017\r\nDTEND;VALUE=DATE:20261019")
        );
        assert!(out.text.contains("SUMMARY:Dentist\\, then shops\\; maybe"));
        assert!(out.text.contains("TRIGGER:-PT30M"));

        let mut b = crate::planner::tests::planner();
        let preview = import::preview(b.db().conn(), Melbourne, &out.text, now()).unwrap();
        assert_eq!((preview.scheduled, preview.inbox), (5, 1));
        assert_eq!(
            (preview.recurring, preview.occurrences, preview.reminders),
            (2, 2, 2)
        );
        assert_eq!(preview.first, NaiveDate::from_ymd_opt(2026, 10, 12));
        assert!(preview.warnings.is_empty(), "{:?}", preview.warnings);
        let ran = run(&mut b, "import_ics", json!({ "ics": out.text }));
        assert_eq!(ran["scheduled"], 5);

        let (old, new) = (items(&a), items(&b));
        assert_eq!(old.len(), new.len());
        for o in old.iter().filter(|i| i.parent_id.is_none()) {
            let n = find(&new, &o.title);
            assert_eq!(
                (n.kind, n.anchoring, n.status, &n.notes, &n.tags, n.priority),
                (o.kind, o.anchoring, o.status, &o.notes, &o.tags, o.priority),
                "{}",
                o.title
            );
            assert_eq!(
                (
                    n.start_at,
                    n.end_at,
                    n.start_date,
                    n.end_date,
                    &n.rrule,
                    n.completed_at
                ),
                (
                    o.start_at,
                    o.end_at,
                    o.start_date,
                    o.end_date,
                    &o.rrule,
                    o.completed_at
                ),
                "{}",
                o.title
            );
            assert_eq!(n.duration_min, o.duration_min, "{}", o.title);
            assert_eq!(n.source, Source::Import);
            assert_eq!(
                n.ics_uid.as_deref(),
                Some(format!("{}@blotter", o.id).as_str())
            );
        }
        // The moved and the deleted standup came across as overrides.
        let standup = find(&new, "Standup");
        let mut kids: Vec<_> = new
            .iter()
            .filter(|i| i.parent_id.as_deref() == Some(&standup.id))
            .map(|i| (i.recurrence_id.clone().unwrap(), i.status, i.start_at))
            .collect();
        kids.sort_by(|a, b| a.0.cmp(&b.0));
        assert_eq!(
            kids,
            [
                (
                    "2026-10-12T22:00:00Z".to_owned(),
                    ItemStatus::Open,
                    Utc.with_ymd_and_hms(2026, 10, 12, 23, 0, 0).single()
                ),
                (
                    "2026-10-13T22:00:00Z".to_owned(),
                    ItemStatus::Cancelled,
                    Utc.with_ymd_and_hms(2026, 10, 13, 22, 0, 0).single()
                ),
            ]
        );
        let dentist = find(&new, "Dentist, then shops; maybe");
        let mut offsets: Vec<_> = store::reminders::list_for_item(b.db().conn(), &dentist.id)
            .unwrap()
            .into_iter()
            .filter_map(|r| r.offset_min)
            .collect();
        offsets.sort_unstable();
        assert_eq!(offsets, [5, 30]);

        // Importing the same file again adds nothing.
        let again = import::preview(b.db().conn(), Melbourne, &out.text, now()).unwrap();
        assert!(again.is_empty());
        assert_eq!(again.already, 6);
        // Nor into the planner it came from.
        let home = import::preview(a.db().conn(), Melbourne, &out.text, now()).unwrap();
        assert_eq!(home.already, 6);
    }

    #[test]
    fn the_import_is_one_undoable_batch() {
        let mut a = crate::planner::tests::planner();
        seed(&mut a);
        let text = export(a.db().conn(), Melbourne, None, now()).unwrap().text;
        let mut b = crate::planner::tests::planner();
        let out = b
            .execute("import_ics", json!({ "ics": text }), &Actor::User)
            .unwrap();
        assert!(!items(&b).is_empty());
        let state = run(&mut b, "get_undo_state", json!({}));
        assert_eq!(state["undo"]["summary"], "Import ICS (6 items)");
        assert_eq!(state["undo"]["batch_id"], json!(out.batch_id));
        run(&mut b, "undo", json!({}));
        assert!(items(&b).is_empty());
        // Agents can't import.
        assert!(
            b.execute("import_ics", json!({ "ics": "" }), &Actor::Mcp)
                .is_err()
        );
    }

    #[test]
    fn a_date_range_keeps_what_falls_in_it() {
        let mut p = crate::planner::tests::planner();
        seed(&mut p);
        let d = |day| NaiveDate::from_ymd_opt(2026, 10, day).unwrap();
        // Sat 17 – Sun 18: Mum visiting, and no series (weekday standups, Wednesday bins).
        let out = export(p.db().conn(), Melbourne, Some((d(17), d(18))), now()).unwrap();
        assert_eq!((out.events, out.todos), (1, 0), "{}", out.text);
        assert!(out.text.contains("SUMMARY:Mum visiting"));
        assert!(!out.text.contains("VTIMEZONE"));
        // Wed 21: the standup series and the bins, which repeat into it.
        let out = export(p.db().conn(), Melbourne, Some((d(21), d(21))), now()).unwrap();
        assert_eq!(out.events, 2, "{}", out.text);
    }

    /// The kind of file Outlook and Google write.
    const FOREIGN: &str = "BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
PRODID:-//Microsoft Corporation//Outlook 16.0 MIMEDIR//EN\r\n\
BEGIN:VTIMEZONE\r\n\
TZID:AUS Eastern Standard Time\r\n\
END:VTIMEZONE\r\n\
BEGIN:VEVENT\r\n\
UID:040000008200E00074C5B7101A82E008\r\n\
SUMMARY:Quarterly review\r\n\
LOCATION:Room 4\r\n\
DTSTART;TZID=AUS Eastern Standard Time:20261020T140000\r\n\
DTEND;TZID=AUS Eastern Standard Time:20261020T153000\r\n\
CATEGORIES:Work Stuff,#Planning\r\n\
PRIORITY:5\r\n\
BEGIN:VALARM\r\n\
TRIGGER:-PT15M\r\n\
ACTION:DISPLAY\r\n\
END:VALARM\r\n\
BEGIN:VALARM\r\n\
TRIGGER;RELATED=END:PT0M\r\n\
ACTION:DISPLAY\r\n\
END:VALARM\r\n\
END:VEVENT\r\n\
BEGIN:VEVENT\r\n\
UID:weekly-1@google.com\r\n\
SUMMARY:Swim\r\n\
DTSTART;TZID=Europe/London:20261005T183000\r\n\
DURATION:PT1H\r\n\
RRULE:FREQ=WEEKLY;UNTIL=20261102;BYDAY=MO\r\n\
EXDATE;TZID=Europe/London:20261012T183000\r\n\
END:VEVENT\r\n\
BEGIN:VEVENT\r\n\
UID:weekly-1@google.com\r\n\
RECURRENCE-ID;TZID=Europe/London:20261019T183000\r\n\
SUMMARY:Swim (late)\r\n\
DTSTART;TZID=Europe/London:20261019T193000\r\n\
DTEND;TZID=Europe/London:20261019T203000\r\n\
END:VEVENT\r\n\
BEGIN:VEVENT\r\n\
UID:hourly@x\r\n\
SUMMARY:Ping\r\n\
DTSTART:20261020T000000Z\r\n\
RRULE:FREQ=HOURLY\r\n\
END:VEVENT\r\n\
BEGIN:VEVENT\r\n\
UID:off@x\r\n\
SUMMARY:Called off\r\n\
STATUS:CANCELLED\r\n\
DTSTART;VALUE=DATE:20261021\r\n\
END:VEVENT\r\n\
BEGIN:VEVENT\r\n\
UID:odd@x\r\n\
SUMMARY:Odd zone\r\n\
DTSTART;TZID=Customized Time Zone:20261022T090000\r\n\
END:VEVENT\r\n\
BEGIN:VTODO\r\n\
UID:todo@x\r\n\
SUMMARY:Renew passport\r\n\
DUE;VALUE=DATE:20261130\r\n\
END:VTODO\r\n\
END:VCALENDAR\r\n";

    #[test]
    fn reads_outlook_and_google_files() {
        let mut p = crate::planner::tests::planner();
        let preview = import::preview(p.db().conn(), Melbourne, FOREIGN, now()).unwrap();
        assert_eq!((preview.scheduled, preview.inbox), (4, 1), "{preview:?}");
        assert_eq!((preview.recurring, preview.occurrences), (1, 2));
        assert_eq!(preview.skipped, 1, "the cancelled one");
        assert_eq!(preview.warnings.len(), 2, "{:?}", preview.warnings);
        assert!(preview.warnings[0].contains("Customized Time Zone"));
        assert!(preview.warnings[1].contains("Ping"));

        run(&mut p, "import_ics", json!({ "ics": FOREIGN }));
        let all = items(&p);
        let review = find(&all, "Quarterly review");
        // 14:00 in Sydney (AEDT, +11) is 03:00Z.
        assert_eq!(
            review.start_at,
            Utc.with_ymd_and_hms(2026, 10, 20, 3, 0, 0).single()
        );
        assert_eq!(review.duration_min, Some(90));
        assert_eq!(review.tz.as_deref(), Some("Australia/Sydney"));
        assert_eq!(review.tags, ["work-stuff", "planning"]);
        assert_eq!(review.notes, "Location: Room 4");
        assert_eq!(review.priority, 2);
        let reminders = store::reminders::list_for_item(p.db().conn(), &review.id).unwrap();
        assert_eq!(reminders.len(), 1, "the after-the-end alarm is left out");

        let swim = find(&all, "Swim");
        assert_eq!(swim.tz.as_deref(), Some("Europe/London"));
        assert_eq!(
            swim.rrule.as_deref(),
            Some("FREQ=WEEKLY;UNTIL=20261102T235959Z;BYDAY=MO")
        );
        let mut kids: Vec<_> = all
            .iter()
            .filter(|i| i.parent_id.as_deref() == Some(&swim.id))
            .map(|i| (i.recurrence_id.clone(), i.title.as_str(), i.status))
            .collect();
        kids.sort_by(|a, b| a.0.cmp(&b.0));
        let kids: Vec<_> = kids.into_iter().map(|(_, t, s)| (t, s)).collect();
        assert_eq!(
            kids,
            [
                ("Swim", ItemStatus::Cancelled),
                ("Swim (late)", ItemStatus::Open)
            ]
        );
        assert!(find(&all, "Ping").rrule.is_none(), "hourly isn't allowed");
        let passport = find(&all, "Renew passport");
        assert!(passport.is_inbox());
        assert!(passport.due_at.is_some());
    }

    #[test]
    fn import_snapshots_first_only_when_something_is_new() {
        let root = std::env::temp_dir().join(format!("blotter-ics-{}", crate::id::new_id()));
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("blotter.db");
        let mut p = Planner::new(crate::Database::open(&path).unwrap(), Melbourne);
        let first = import(&mut p, &path, FOREIGN).unwrap();
        let snap = first.snapshot.expect("snapshot");
        assert_eq!(snap.reason, Reason::BeforeIcsImport);
        assert!(first.batch_id.is_some());
        let second = import(&mut p, &path, FOREIGN).unwrap();
        assert!(second.snapshot.is_none() && second.batch_id.is_none());
        assert!(import(&mut p, &path, "not a calendar").is_err());
        assert_eq!(snapshots::list(p.db().conn()).unwrap().len(), 1);
        drop(p);
        let _ = std::fs::remove_dir_all(root);
    }
}

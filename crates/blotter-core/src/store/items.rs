use chrono::NaiveDate;
use rusqlite::{Connection, OptionalExtension, Row, named_params};

use super::{fmt_opt_date, fmt_opt_instant, opt_date, opt_instant, parse_enum};
use crate::error::{CoreError, Result};
use crate::model::Item;
use crate::time::{self, Instant};

const COLUMNS: &str = "id, kind, anchoring, status, title, notes, all_day, start_at, end_at, \
     start_date, end_date, tz, duration_min, due_at, priority, tags, color, sort_order, rrule, \
     parent_id, recurrence_id, completed_at, source, ics_uid, created_at, updated_at";

/// A window to list items in. Timed items overlapping `[start, end)` and
/// all-day items overlapping `[start_date, end_date]` are returned. The
/// caller derives the dates from the user's timezone.
#[derive(Debug, Clone, Copy)]
pub struct ItemRange {
    pub start: Instant,
    pub end: Instant,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}

fn read(row: &Row<'_>) -> Result<Item> {
    let tags: String = row.get("tags")?;
    Ok(Item {
        id: row.get("id")?,
        kind: parse_enum("kind", row.get("kind")?)?,
        anchoring: parse_enum("anchoring", row.get("anchoring")?)?,
        status: parse_enum("status", row.get("status")?)?,
        title: row.get("title")?,
        notes: row.get("notes")?,
        all_day: row.get("all_day")?,
        start_at: opt_instant("start_at", row.get("start_at")?)?,
        end_at: opt_instant("end_at", row.get("end_at")?)?,
        start_date: opt_date("start_date", row.get("start_date")?)?,
        end_date: opt_date("end_date", row.get("end_date")?)?,
        tz: row.get("tz")?,
        duration_min: row.get("duration_min")?,
        due_at: opt_instant("due_at", row.get("due_at")?)?,
        priority: row.get("priority")?,
        tags: serde_json::from_str(&tags).map_err(|_| CoreError::Corrupt {
            column: "tags",
            value: tags.clone(),
        })?,
        color: row.get("color")?,
        sort_order: row.get("sort_order")?,
        rrule: row.get("rrule")?,
        parent_id: row.get("parent_id")?,
        recurrence_id: row.get("recurrence_id")?,
        completed_at: opt_instant("completed_at", row.get("completed_at")?)?,
        source: parse_enum("source", row.get("source")?)?,
        ics_uid: row.get("ics_uid")?,
        created_at: time::parse_instant("created_at", &row.get::<_, String>("created_at")?)?,
        updated_at: time::parse_instant("updated_at", &row.get::<_, String>("updated_at")?)?,
    })
}

fn collect(conn: &Connection, sql: &str, params: impl rusqlite::Params) -> Result<Vec<Item>> {
    let mut stmt = conn.prepare(sql)?;
    let mut rows = stmt.query(params)?;
    let mut out = Vec::new();
    while let Some(row) = rows.next()? {
        out.push(read(row)?);
    }
    Ok(out)
}

/// Writes every column of `item`. On an existing id, updates in place
/// (never delete-and-reinsert, which would cascade to reminders).
fn write(conn: &Connection, item: &Item, upsert: bool) -> Result<usize> {
    let tail = if upsert {
        " ON CONFLICT (id) DO UPDATE SET kind = excluded.kind, anchoring = excluded.anchoring, \
         status = excluded.status, title = excluded.title, notes = excluded.notes, \
         all_day = excluded.all_day, start_at = excluded.start_at, end_at = excluded.end_at, \
         start_date = excluded.start_date, end_date = excluded.end_date, tz = excluded.tz, \
         duration_min = excluded.duration_min, due_at = excluded.due_at, \
         priority = excluded.priority, tags = excluded.tags, color = excluded.color, \
         sort_order = excluded.sort_order, rrule = excluded.rrule, parent_id = excluded.parent_id, \
         recurrence_id = excluded.recurrence_id, completed_at = excluded.completed_at, \
         source = excluded.source, ics_uid = excluded.ics_uid, \
         created_at = excluded.created_at, \
         updated_at = excluded.updated_at"
    } else {
        ""
    };
    let sql = format!(
        "INSERT INTO items ({COLUMNS}) VALUES (:id, :kind, :anchoring, :status, :title, :notes, \
         :all_day, :start_at, :end_at, :start_date, :end_date, :tz, :duration_min, :due_at, \
         :priority, :tags, :color, :sort_order, :rrule, :parent_id, :recurrence_id, \
         :completed_at, :source, :ics_uid, :created_at, :updated_at){tail}"
    );
    Ok(conn.execute(
        &sql,
        named_params! {
            ":id": item.id,
            ":kind": item.kind.as_str(),
            ":anchoring": item.anchoring.as_str(),
            ":status": item.status.as_str(),
            ":title": item.title,
            ":notes": item.notes,
            ":all_day": item.all_day,
            ":start_at": fmt_opt_instant(item.start_at),
            ":end_at": fmt_opt_instant(item.end_at),
            ":start_date": fmt_opt_date(item.start_date),
            ":end_date": fmt_opt_date(item.end_date),
            ":tz": item.tz,
            ":duration_min": item.duration_min,
            ":due_at": fmt_opt_instant(item.due_at),
            ":priority": item.priority,
            ":tags": serde_json::to_string(&item.tags)?,
            ":color": item.color,
            ":sort_order": item.sort_order,
            ":rrule": item.rrule,
            ":parent_id": item.parent_id,
            ":recurrence_id": item.recurrence_id,
            ":completed_at": fmt_opt_instant(item.completed_at),
            ":source": item.source.as_str(),
            ":ics_uid": item.ics_uid,
            ":created_at": time::fmt_instant(item.created_at),
            ":updated_at": time::fmt_instant(item.updated_at),
        },
    )?)
}

/// Inserts a new item; fails if the id already exists.
pub fn insert(conn: &Connection, item: &Item) -> Result<()> {
    write(conn, item, false).map(|_| ())
}

/// Inserts or fully overwrites an item. Used to restore state on undo.
pub fn upsert(conn: &Connection, item: &Item) -> Result<()> {
    write(conn, item, true).map(|_| ())
}

/// Overwrites an existing item; `NotFound` if it doesn't exist.
pub fn update(conn: &Connection, item: &Item) -> Result<()> {
    if get(conn, &item.id)?.is_none() {
        return Err(CoreError::not_found("item", &item.id));
    }
    upsert(conn, item)
}

pub fn get(conn: &Connection, id: &str) -> Result<Option<Item>> {
    let sql = format!("SELECT {COLUMNS} FROM items WHERE id = ?1");
    let mut stmt = conn.prepare(&sql)?;
    let row = stmt.query_row([id], |row| Ok(read(row))).optional()?;
    row.transpose()
}

/// Like [`get`] but `NotFound` when missing.
pub fn require(conn: &Connection, id: &str) -> Result<Item> {
    get(conn, id)?.ok_or_else(|| CoreError::not_found("item", id))
}

/// Deletes an item (reminders and occurrence overrides cascade). Returns
/// whether a row was removed.
pub fn delete(conn: &Connection, id: &str) -> Result<bool> {
    Ok(conn.execute("DELETE FROM items WHERE id = ?1", [id])? > 0)
}

/// Items scheduled within `range`, in display order. Recurring series and
/// cancelled occurrences are left out; `recurrence::agenda` expands series.
pub fn list_in_range(conn: &Connection, range: ItemRange) -> Result<Vec<Item>> {
    let sql = format!(
        "SELECT {COLUMNS} FROM items WHERE rrule IS NULL \
           AND NOT (parent_id IS NOT NULL AND status = 'cancelled') AND ( \
           (all_day = 0 AND start_at IS NOT NULL AND start_at < :end \
             AND (COALESCE(end_at, start_at) > :start OR start_at >= :start)) \
           OR (all_day = 1 AND start_date <= :end_date \
             AND COALESCE(end_date, start_date) >= :start_date)) \
         ORDER BY COALESCE(start_at, start_date), sort_order, created_at"
    );
    collect(
        conn,
        &sql,
        named_params! {
            ":start": time::fmt_instant(range.start),
            ":end": time::fmt_instant(range.end),
            ":start_date": time::fmt_date(range.start_date),
            ":end_date": time::fmt_date(range.end_date),
        },
    )
}

/// Recurring series that start before `range` ends (they may have
/// occurrences in it).
pub fn list_series(conn: &Connection, range: ItemRange) -> Result<Vec<Item>> {
    let sql = format!(
        "SELECT {COLUMNS} FROM items WHERE rrule IS NOT NULL AND parent_id IS NULL \n           AND status <> 'cancelled' \n           AND ((all_day = 0 AND start_at < :end) OR (all_day = 1 AND start_date <= :end_date)) \n         ORDER BY created_at"
    );
    collect(
        conn,
        &sql,
        named_params! {
            ":end": time::fmt_instant(range.end),
            ":end_date": time::fmt_date(range.end_date),
        },
    )
}

/// Open and done Inbox tasks (flexible, unscheduled), in manual order.
pub fn list_inbox(conn: &Connection) -> Result<Vec<Item>> {
    let sql = format!(
        "SELECT {COLUMNS} FROM items WHERE kind = 'task' AND anchoring = 'flexible' \
           AND start_at IS NULL AND start_date IS NULL AND status <> 'cancelled' \
         ORDER BY status = 'done', sort_order, created_at"
    );
    collect(conn, &sql, [])
}

/// Case-insensitive substring match on title, notes and tags.
pub fn search(conn: &Connection, query: &str, limit: u32) -> Result<Vec<Item>> {
    let escaped = query
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");
    let pattern = format!("%{escaped}%");
    let sql = format!(
        "SELECT {COLUMNS} FROM items WHERE title LIKE :p ESCAPE '\\' \
           OR notes LIKE :p ESCAPE '\\' OR tags LIKE :p ESCAPE '\\' \
         ORDER BY updated_at DESC LIMIT :limit"
    );
    collect(conn, &sql, named_params! { ":p": pattern, ":limit": limit })
}

/// Open tasks left behind: scheduled on a day before `today` (timed items
/// ending at or before `today_start`, all-day items ending before `today`),
/// or past their deadline at `now`. Oldest first.
pub fn list_overdue(
    conn: &Connection,
    now: Instant,
    today_start: Instant,
    today: NaiveDate,
) -> Result<Vec<Item>> {
    let sql = format!(
        "SELECT {COLUMNS} FROM items WHERE status = 'open' AND kind = 'task' AND rrule IS NULL AND (            (all_day = 0 AND end_at IS NOT NULL AND end_at <= :today_start)            OR (all_day = 1 AND COALESCE(end_date, start_date) < :today)            OR (due_at IS NOT NULL AND due_at < :now))          ORDER BY COALESCE(end_at, end_date, start_date, due_at), sort_order"
    );
    collect(
        conn,
        &sql,
        named_params! {
            ":now": time::fmt_instant(now),
            ":today_start": time::fmt_instant(today_start),
            ":today": time::fmt_date(today),
        },
    )
}

/// Every item: series and plain items first, then occurrence overrides
/// (so a restore can insert parents before children).
pub fn list_all(conn: &Connection) -> Result<Vec<Item>> {
    let sql = format!("SELECT {COLUMNS} FROM items ORDER BY parent_id IS NOT NULL, created_at, id");
    collect(conn, &sql, [])
}

/// Occurrence overrides of a recurring series.
pub fn list_children(conn: &Connection, parent_id: &str) -> Result<Vec<Item>> {
    let sql = format!("SELECT {COLUMNS} FROM items WHERE parent_id = ?1 ORDER BY recurrence_id");
    collect(conn, &sql, [parent_id])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::model::{Anchoring, ItemKind, ItemStatus};
    use chrono::{Duration, TimeZone, Utc};

    fn at(h: u32, m: u32) -> Instant {
        Utc.with_ymd_and_hms(2026, 10, 14, h, m, 0).unwrap()
    }

    fn task(id: &str, title: &str) -> Item {
        Item::new(id.into(), ItemKind::Task, title, at(0, 0))
    }

    fn day_range() -> ItemRange {
        ItemRange {
            start: at(0, 0),
            end: at(0, 0) + Duration::days(1),
            start_date: NaiveDate::from_ymd_opt(2026, 10, 14).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2026, 10, 14).unwrap(),
        }
    }

    #[test]
    fn insert_get_round_trip_preserves_every_field() {
        let db = Database::open_in_memory().unwrap();
        let mut item = Item::new("01A".into(), ItemKind::Event, "Dentist", at(8, 0));
        item.notes = "Bring referral".into();
        item.start_at = Some(at(15, 0));
        item.end_at = Some(at(15, 45));
        item.tz = Some("Australia/Melbourne".into());
        item.duration_min = Some(45);
        item.priority = 2;
        item.tags = vec!["health".into()];
        item.color = Some("#56c49b".into());
        item.sort_order = 1.5;
        insert(db.conn(), &item).unwrap();
        assert_eq!(get(db.conn(), "01A").unwrap().unwrap(), item);
    }

    #[test]
    fn insert_rejects_duplicate_and_update_requires_existing() {
        let db = Database::open_in_memory().unwrap();
        let t = task("a", "Call plumber");
        insert(db.conn(), &t).unwrap();
        assert!(insert(db.conn(), &t).is_err());
        assert!(matches!(
            update(db.conn(), &task("missing", "x")),
            Err(CoreError::NotFound { entity: "item", .. })
        ));
    }

    #[test]
    fn upsert_does_not_cascade_reminders() {
        let db = Database::open_in_memory().unwrap();
        let mut t = task("a", "Call plumber");
        insert(db.conn(), &t).unwrap();
        db.conn()
            .execute(
                "INSERT INTO reminders (id, item_id, offset_min, created_at) \
                 VALUES ('r', 'a', 10, '2026-10-14T00:00:00Z')",
                [],
            )
            .unwrap();
        t.title = "Call the plumber".into();
        upsert(db.conn(), &t).unwrap();
        let n: i64 = db
            .conn()
            .query_row("SELECT COUNT(*) FROM reminders", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 1);
    }

    #[test]
    fn schema_rejects_inconsistent_rows() {
        let db = Database::open_in_memory().unwrap();
        let mut done_without_time = task("a", "x");
        done_without_time.status = ItemStatus::Done;
        assert!(insert(db.conn(), &done_without_time).is_err());

        let mut backwards = task("b", "x");
        backwards.start_at = Some(at(10, 0));
        backwards.end_at = Some(at(9, 0));
        assert!(insert(db.conn(), &backwards).is_err());

        let mut bad_priority = task("c", "x");
        bad_priority.priority = 4;
        assert!(insert(db.conn(), &bad_priority).is_err());
    }

    #[test]
    fn range_includes_overlaps_and_all_day_but_not_inbox() {
        let db = Database::open_in_memory().unwrap();
        let mut morning = task("m", "Run");
        morning.start_at = Some(at(7, 15));
        morning.end_at = Some(at(7, 45));
        let mut overnight = task("o", "Flight");
        overnight.start_at = Some(at(0, 0) - Duration::hours(2));
        overnight.end_at = Some(at(1, 0));
        let mut ended_yesterday = task("y", "Yesterday");
        ended_yesterday.start_at = Some(at(0, 0) - Duration::hours(3));
        ended_yesterday.end_at = Some(at(0, 0));
        let mut all_day = task("d", "Leave");
        all_day.all_day = true;
        all_day.start_date = NaiveDate::from_ymd_opt(2026, 10, 13);
        all_day.end_date = NaiveDate::from_ymd_opt(2026, 10, 16);
        let inbox = task("i", "Someday");
        for i in [&morning, &overnight, &ended_yesterday, &all_day, &inbox] {
            insert(db.conn(), i).unwrap();
        }
        let ids: Vec<_> = list_in_range(db.conn(), day_range())
            .unwrap()
            .into_iter()
            .map(|i| i.id)
            .collect();
        assert!(ids.contains(&"m".to_string()));
        assert!(ids.contains(&"o".to_string()));
        assert!(ids.contains(&"d".to_string()));
        assert!(
            !ids.contains(&"y".to_string()),
            "ends exactly at range start"
        );
        assert!(!ids.contains(&"i".to_string()));
    }

    #[test]
    fn inbox_lists_only_unscheduled_flexible_tasks() {
        let db = Database::open_in_memory().unwrap();
        let mut a = task("a", "Fix tap");
        a.sort_order = 2.0;
        let mut b = task("b", "Book flu shot");
        b.sort_order = 1.0;
        let mut fixed = task("f", "Telehealth");
        fixed.anchoring = Anchoring::Fixed;
        let mut scheduled = task("s", "Gym");
        scheduled.start_at = Some(at(17, 30));
        let event = Item::new("e".into(), ItemKind::Event, "Party", at(0, 0));
        for i in [&a, &b, &fixed, &scheduled, &event] {
            insert(db.conn(), i).unwrap();
        }
        let ids: Vec<_> = list_inbox(db.conn())
            .unwrap()
            .into_iter()
            .map(|i| i.id)
            .collect();
        assert_eq!(ids, ["b", "a"]);
    }

    #[test]
    fn search_matches_title_notes_tags_and_escapes_wildcards() {
        let db = Database::open_in_memory().unwrap();
        let mut a = task("a", "Weekend shopping");
        a.tags = vec!["shopping".into()];
        let mut b = task("b", "Insurance");
        b.notes = "Renewal is 100% due".into();
        insert(db.conn(), &a).unwrap();
        insert(db.conn(), &b).unwrap();
        assert_eq!(search(db.conn(), "SHOP", 10).unwrap().len(), 1);
        assert_eq!(search(db.conn(), "100%", 10).unwrap()[0].id, "b");
        assert!(
            search(db.conn(), "%", 10)
                .unwrap()
                .iter()
                .all(|i| i.id == "b")
        );
    }

    #[test]
    fn delete_cascades_to_occurrence_overrides() {
        let db = Database::open_in_memory().unwrap();
        let mut series = task("s", "Standup");
        series.rrule = Some("FREQ=DAILY".into());
        let mut child = task("c", "Standup (moved)");
        child.parent_id = Some("s".into());
        child.recurrence_id = Some("2026-10-15T09:00:00Z".into());
        insert(db.conn(), &series).unwrap();
        insert(db.conn(), &child).unwrap();
        assert_eq!(list_children(db.conn(), "s").unwrap().len(), 1);
        assert!(delete(db.conn(), "s").unwrap());
        assert!(get(db.conn(), "c").unwrap().is_none());
        assert!(!delete(db.conn(), "s").unwrap());
    }
}

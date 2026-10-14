use rusqlite::{Connection, OptionalExtension, Row, named_params};

use super::{fmt_opt_instant, opt_instant};
use crate::error::{CoreError, Result};
use crate::model::Reminder;
use crate::time;

const COLUMNS: &str = "id, item_id, offset_min, at, fired_at, snoozed_until, created_at";

fn read(row: &Row<'_>) -> Result<Reminder> {
    Ok(Reminder {
        id: row.get("id")?,
        item_id: row.get("item_id")?,
        offset_min: row.get("offset_min")?,
        at: opt_instant("at", row.get("at")?)?,
        fired_at: opt_instant("fired_at", row.get("fired_at")?)?,
        snoozed_until: opt_instant("snoozed_until", row.get("snoozed_until")?)?,
        created_at: time::parse_instant("created_at", &row.get::<_, String>("created_at")?)?,
    })
}

/// Inserts or fully overwrites a reminder.
pub fn upsert(conn: &Connection, r: &Reminder) -> Result<()> {
    conn.execute(
        &format!(
            "INSERT INTO reminders ({COLUMNS}) VALUES (:id, :item_id, :offset_min, :at, \
             :fired_at, :snoozed_until, :created_at) \
             ON CONFLICT (id) DO UPDATE SET item_id = excluded.item_id, \
             offset_min = excluded.offset_min, at = excluded.at, fired_at = excluded.fired_at, \
             snoozed_until = excluded.snoozed_until, created_at = excluded.created_at"
        ),
        named_params! {
            ":id": r.id,
            ":item_id": r.item_id,
            ":offset_min": r.offset_min,
            ":at": fmt_opt_instant(r.at),
            ":fired_at": fmt_opt_instant(r.fired_at),
            ":snoozed_until": fmt_opt_instant(r.snoozed_until),
            ":created_at": time::fmt_instant(r.created_at),
        },
    )?;
    Ok(())
}

pub fn get(conn: &Connection, id: &str) -> Result<Option<Reminder>> {
    let mut stmt = conn.prepare(&format!("SELECT {COLUMNS} FROM reminders WHERE id = ?1"))?;
    stmt.query_row([id], |row| Ok(read(row)))
        .optional()?
        .transpose()
}

pub fn require(conn: &Connection, id: &str) -> Result<Reminder> {
    get(conn, id)?.ok_or_else(|| CoreError::not_found("reminder", id))
}

pub fn delete(conn: &Connection, id: &str) -> Result<bool> {
    Ok(conn.execute("DELETE FROM reminders WHERE id = ?1", [id])? > 0)
}

pub fn list_for_item(conn: &Connection, item_id: &str) -> Result<Vec<Reminder>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {COLUMNS} FROM reminders WHERE item_id = ?1 ORDER BY created_at, id"
    ))?;
    let mut rows = stmt.query([item_id])?;
    let mut out = Vec::new();
    while let Some(row) = rows.next()? {
        out.push(read(row)?);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::model::{Item, ItemKind};
    use chrono::{TimeZone, Utc};

    #[test]
    fn reminder_round_trip_and_schema_rules() {
        let db = Database::open_in_memory().unwrap();
        let now = Utc.with_ymd_and_hms(2026, 10, 14, 9, 0, 0).unwrap();
        crate::store::items::insert(
            db.conn(),
            &Item::new("i".into(), ItemKind::Event, "Dentist", now),
        )
        .unwrap();
        let r = Reminder {
            id: "r".into(),
            item_id: "i".into(),
            offset_min: Some(10),
            at: None,
            fired_at: None,
            snoozed_until: None,
            created_at: now,
        };
        upsert(db.conn(), &r).unwrap();
        assert_eq!(require(db.conn(), "r").unwrap(), r);
        assert_eq!(list_for_item(db.conn(), "i").unwrap(), vec![r.clone()]);

        let both = Reminder {
            id: "bad".into(),
            at: Some(now),
            ..r.clone()
        };
        assert!(
            upsert(db.conn(), &both).is_err(),
            "offset and at are exclusive"
        );

        let orphan = Reminder {
            id: "orphan".into(),
            item_id: "nope".into(),
            ..r
        };
        assert!(upsert(db.conn(), &orphan).is_err(), "foreign key enforced");
    }
}

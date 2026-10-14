use chrono::NaiveDate;
use rusqlite::{Connection, OptionalExtension, Row, named_params};

use super::{fmt_opt_date, opt_date, req_time};
use crate::error::{CoreError, Result};
use crate::model::{Blockout, BlockoutException, Weekdays};
use crate::time;

const COLUMNS: &str = "id, label, weekdays, start_time, end_time, tz, active_from, active_until, \
     permits_fixed, color, icon, sort_order, created_at, updated_at";

fn read(row: &Row<'_>) -> Result<Blockout> {
    let bits: u8 = row.get("weekdays")?;
    Ok(Blockout {
        id: row.get("id")?,
        label: row.get("label")?,
        weekdays: Weekdays::from_bits(bits).ok_or(CoreError::Corrupt {
            column: "weekdays",
            value: bits.to_string(),
        })?,
        start_time: req_time("start_time", row.get("start_time")?)?,
        end_time: req_time("end_time", row.get("end_time")?)?,
        tz: row.get("tz")?,
        active_from: opt_date("active_from", row.get("active_from")?)?,
        active_until: opt_date("active_until", row.get("active_until")?)?,
        permits_fixed: row.get("permits_fixed")?,
        color: row.get("color")?,
        icon: row.get("icon")?,
        sort_order: row.get("sort_order")?,
        created_at: time::parse_instant("created_at", &row.get::<_, String>("created_at")?)?,
        updated_at: time::parse_instant("updated_at", &row.get::<_, String>("updated_at")?)?,
    })
}

/// Inserts or fully overwrites a block-out (exceptions are kept).
pub fn upsert(conn: &Connection, b: &Blockout) -> Result<()> {
    conn.execute(
        &format!(
            "INSERT INTO blockouts ({COLUMNS}) VALUES (:id, :label, :weekdays, :start_time, \
             :end_time, :tz, :active_from, :active_until, :permits_fixed, :color, :icon, \
             :sort_order, :created_at, :updated_at) \
             ON CONFLICT (id) DO UPDATE SET label = excluded.label, \
             weekdays = excluded.weekdays, start_time = excluded.start_time, \
             end_time = excluded.end_time, tz = excluded.tz, active_from = excluded.active_from, \
             active_until = excluded.active_until, permits_fixed = excluded.permits_fixed, \
             color = excluded.color, icon = excluded.icon, sort_order = excluded.sort_order, \
             created_at = excluded.created_at, updated_at = excluded.updated_at"
        ),
        named_params! {
            ":id": b.id,
            ":label": b.label,
            ":weekdays": b.weekdays.bits(),
            ":start_time": time::fmt_time(b.start_time),
            ":end_time": time::fmt_time(b.end_time),
            ":tz": b.tz,
            ":active_from": fmt_opt_date(b.active_from),
            ":active_until": fmt_opt_date(b.active_until),
            ":permits_fixed": b.permits_fixed,
            ":color": b.color,
            ":icon": b.icon,
            ":sort_order": b.sort_order,
            ":created_at": time::fmt_instant(b.created_at),
            ":updated_at": time::fmt_instant(b.updated_at),
        },
    )?;
    Ok(())
}

pub fn get(conn: &Connection, id: &str) -> Result<Option<Blockout>> {
    let mut stmt = conn.prepare(&format!("SELECT {COLUMNS} FROM blockouts WHERE id = ?1"))?;
    stmt.query_row([id], |row| Ok(read(row)))
        .optional()?
        .transpose()
}

pub fn require(conn: &Connection, id: &str) -> Result<Blockout> {
    get(conn, id)?.ok_or_else(|| CoreError::not_found("blockout", id))
}

/// Deletes a block-out and its exceptions.
pub fn delete(conn: &Connection, id: &str) -> Result<bool> {
    Ok(conn.execute("DELETE FROM blockouts WHERE id = ?1", [id])? > 0)
}

pub fn list(conn: &Connection) -> Result<Vec<Blockout>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {COLUMNS} FROM blockouts ORDER BY sort_order, created_at"
    ))?;
    let mut rows = stmt.query([])?;
    let mut out = Vec::new();
    while let Some(row) = rows.next()? {
        out.push(read(row)?);
    }
    Ok(out)
}

fn read_exception(row: &Row<'_>) -> Result<BlockoutException> {
    Ok(BlockoutException {
        id: row.get("id")?,
        blockout_id: row.get("blockout_id")?,
        date: time::parse_date("date", &row.get::<_, String>("date")?)?,
        note: row.get("note")?,
        created_at: time::parse_instant("created_at", &row.get::<_, String>("created_at")?)?,
    })
}

/// Inserts or overwrites an exception. A second exception for the same
/// block-out and date violates the unique constraint.
pub fn upsert_exception(conn: &Connection, e: &BlockoutException) -> Result<()> {
    conn.execute(
        "INSERT INTO blockout_exceptions (id, blockout_id, date, note, created_at) \
         VALUES (:id, :blockout_id, :date, :note, :created_at) \
         ON CONFLICT (id) DO UPDATE SET blockout_id = excluded.blockout_id, \
         date = excluded.date, note = excluded.note, created_at = excluded.created_at",
        named_params! {
            ":id": e.id,
            ":blockout_id": e.blockout_id,
            ":date": time::fmt_date(e.date),
            ":note": e.note,
            ":created_at": time::fmt_instant(e.created_at),
        },
    )?;
    Ok(())
}

pub fn get_exception(conn: &Connection, id: &str) -> Result<Option<BlockoutException>> {
    let mut stmt = conn.prepare(
        "SELECT id, blockout_id, date, note, created_at FROM blockout_exceptions WHERE id = ?1",
    )?;
    stmt.query_row([id], |row| Ok(read_exception(row)))
        .optional()?
        .transpose()
}

pub fn delete_exception(conn: &Connection, id: &str) -> Result<bool> {
    Ok(conn.execute("DELETE FROM blockout_exceptions WHERE id = ?1", [id])? > 0)
}

/// Exceptions for one block-out, or for all block-outs when `blockout_id`
/// is `None`, optionally limited to dates in `[from, to]`.
pub fn list_exceptions(
    conn: &Connection,
    blockout_id: Option<&str>,
    dates: Option<(NaiveDate, NaiveDate)>,
) -> Result<Vec<BlockoutException>> {
    let (from, to) = dates
        .map(|(f, t)| (Some(time::fmt_date(f)), Some(time::fmt_date(t))))
        .unwrap_or((None, None));
    let mut stmt = conn.prepare(
        "SELECT id, blockout_id, date, note, created_at FROM blockout_exceptions \
         WHERE (:bid IS NULL OR blockout_id = :bid) \
           AND (:from IS NULL OR date >= :from) AND (:to IS NULL OR date <= :to) \
         ORDER BY date, blockout_id",
    )?;
    let mut rows = stmt.query(named_params! { ":bid": blockout_id, ":from": from, ":to": to })?;
    let mut out = Vec::new();
    while let Some(row) = rows.next()? {
        out.push(read_exception(row)?);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use chrono::{NaiveTime, TimeZone, Utc};

    fn work() -> Blockout {
        let now = Utc.with_ymd_and_hms(2026, 10, 14, 9, 0, 0).unwrap();
        Blockout {
            id: "work".into(),
            label: "Work".into(),
            weekdays: Weekdays::WEEKDAYS,
            start_time: NaiveTime::from_hms_opt(8, 30, 0).unwrap(),
            end_time: NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
            tz: "Australia/Melbourne".into(),
            active_from: None,
            active_until: None,
            permits_fixed: true,
            color: None,
            icon: Some("briefcase".into()),
            sort_order: 0,
            created_at: now,
            updated_at: now,
        }
    }

    fn d(day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 10, day).unwrap()
    }

    #[test]
    fn blockout_round_trip_and_exceptions() {
        let db = Database::open_in_memory().unwrap();
        let b = work();
        upsert(db.conn(), &b).unwrap();
        assert_eq!(require(db.conn(), "work").unwrap(), b);
        assert_eq!(list(db.conn()).unwrap().len(), 1);

        let e = BlockoutException {
            id: "e1".into(),
            blockout_id: "work".into(),
            date: d(20),
            note: "Leave".into(),
            created_at: b.created_at,
        };
        upsert_exception(db.conn(), &e).unwrap();
        let dup = BlockoutException {
            id: "e2".into(),
            ..e.clone()
        };
        assert!(
            upsert_exception(db.conn(), &dup).is_err(),
            "one exception per date"
        );

        assert_eq!(
            list_exceptions(db.conn(), Some("work"), None).unwrap(),
            vec![e.clone()]
        );
        assert!(
            list_exceptions(db.conn(), None, Some((d(1), d(19))))
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            list_exceptions(db.conn(), None, Some((d(20), d(20))))
                .unwrap()
                .len(),
            1
        );

        assert!(delete(db.conn(), "work").unwrap());
        assert!(
            get_exception(db.conn(), "e1").unwrap().is_none(),
            "exceptions cascade"
        );
    }

    #[test]
    fn overnight_blockout_is_allowed_but_zero_length_is_not() {
        let db = Database::open_in_memory().unwrap();
        let mut sleep = work();
        sleep.id = "sleep".into();
        sleep.start_time = NaiveTime::from_hms_opt(22, 30, 0).unwrap();
        sleep.end_time = NaiveTime::from_hms_opt(6, 30, 0).unwrap();
        sleep.permits_fixed = false;
        assert!(sleep.wraps_midnight());
        upsert(db.conn(), &sleep).unwrap();

        let mut empty = work();
        empty.id = "empty".into();
        empty.end_time = empty.start_time;
        assert!(upsert(db.conn(), &empty).is_err());
    }
}

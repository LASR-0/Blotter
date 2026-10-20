use chrono::NaiveDate;
use rusqlite::{Connection, OptionalExtension, Row, named_params};

use crate::error::{CoreError, Result};
use crate::model::{BlockoutException, Holiday};
use crate::time;

const COLUMNS: &str = "id, label, start_date, end_date, color, created_at, updated_at";

fn read(row: &Row<'_>) -> Result<Holiday> {
    Ok(Holiday {
        id: row.get("id")?,
        label: row.get("label")?,
        start_date: time::parse_date("start_date", &row.get::<_, String>("start_date")?)?,
        end_date: time::parse_date("end_date", &row.get::<_, String>("end_date")?)?,
        color: row.get("color")?,
        created_at: time::parse_instant("created_at", &row.get::<_, String>("created_at")?)?,
        updated_at: time::parse_instant("updated_at", &row.get::<_, String>("updated_at")?)?,
    })
}

fn collect(conn: &Connection, sql: &str, params: impl rusqlite::Params) -> Result<Vec<Holiday>> {
    let mut stmt = conn.prepare(sql)?;
    let mut rows = stmt.query(params)?;
    let mut out = Vec::new();
    while let Some(row) = rows.next()? {
        out.push(read(row)?);
    }
    Ok(out)
}

pub fn upsert(conn: &Connection, h: &Holiday) -> Result<()> {
    conn.execute(
        &format!(
            "INSERT INTO holidays ({COLUMNS}) VALUES (:id, :label, :start_date, :end_date, \
             :color, :created_at, :updated_at) \
             ON CONFLICT (id) DO UPDATE SET label = excluded.label, \
             start_date = excluded.start_date, end_date = excluded.end_date, \
             color = excluded.color, created_at = excluded.created_at, \
             updated_at = excluded.updated_at"
        ),
        named_params! {
            ":id": h.id,
            ":label": h.label,
            ":start_date": time::fmt_date(h.start_date),
            ":end_date": time::fmt_date(h.end_date),
            ":color": h.color,
            ":created_at": time::fmt_instant(h.created_at),
            ":updated_at": time::fmt_instant(h.updated_at),
        },
    )?;
    Ok(())
}

pub fn get(conn: &Connection, id: &str) -> Result<Option<Holiday>> {
    let sql = format!("SELECT {COLUMNS} FROM holidays WHERE id = ?1");
    conn.prepare(&sql)?
        .query_row([id], |row| Ok(read(row)))
        .optional()?
        .transpose()
}

pub fn require(conn: &Connection, id: &str) -> Result<Holiday> {
    get(conn, id)?.ok_or_else(|| CoreError::not_found("holiday", id))
}

pub fn delete(conn: &Connection, id: &str) -> Result<bool> {
    Ok(conn.execute("DELETE FROM holidays WHERE id = ?1", [id])? > 0)
}

/// Every holiday, soonest first.
pub fn list(conn: &Connection) -> Result<Vec<Holiday>> {
    collect(
        conn,
        &format!("SELECT {COLUMNS} FROM holidays ORDER BY start_date, label"),
        [],
    )
}

/// Holidays with a day in `[from, to]`.
pub fn overlapping(conn: &Connection, from: NaiveDate, to: NaiveDate) -> Result<Vec<Holiday>> {
    collect(
        conn,
        &format!(
            "SELECT {COLUMNS} FROM holidays WHERE start_date <= :to AND end_date >= :from \
             ORDER BY start_date, label"
        ),
        named_params! { ":from": time::fmt_date(from), ":to": time::fmt_date(to) },
    )
}

/// Holidays whose last day is before `today`.
pub fn ended_before(conn: &Connection, today: NaiveDate) -> Result<Vec<Holiday>> {
    collect(
        conn,
        &format!("SELECT {COLUMNS} FROM holidays WHERE end_date < ?1 ORDER BY start_date"),
        [time::fmt_date(today)],
    )
}

/// The block-out exceptions a holiday made.
pub fn exceptions_of(conn: &Connection, holiday_id: &str) -> Result<Vec<BlockoutException>> {
    Ok(super::blockouts::list_exceptions(conn, None, None)?
        .into_iter()
        .filter(|e| e.holiday_id.as_deref() == Some(holiday_id))
        .collect())
}

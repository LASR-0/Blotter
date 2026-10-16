//! ICS subscriptions and their cached, read-only events.

use rusqlite::{Connection, OptionalExtension, Row, named_params};

use super::{fmt_opt_date, fmt_opt_instant, opt_date, opt_instant};
use crate::error::{CoreError, Result};
use crate::model::{Subscription, SubscriptionEvent};
use crate::store::items::ItemRange;
use crate::time::{self, Instant};

const COLUMNS: &str = "id, name, url, refresh_min, color, as_blockout, enabled, last_fetch_at, \
     last_ok_at, last_status, last_error, created_at, updated_at";

const EVENT_COLUMNS: &str = "id, subscription_id, uid, title, location, all_day, start_at, end_at, start_date, end_date, busy";

fn read(row: &Row<'_>) -> Result<Subscription> {
    Ok(Subscription {
        id: row.get("id")?,
        name: row.get("name")?,
        url: row.get("url")?,
        refresh_min: row.get("refresh_min")?,
        color: row.get("color")?,
        as_blockout: row.get("as_blockout")?,
        enabled: row.get("enabled")?,
        last_fetch_at: opt_instant("last_fetch_at", row.get("last_fetch_at")?)?,
        last_ok_at: opt_instant("last_ok_at", row.get("last_ok_at")?)?,
        last_status: row.get("last_status")?,
        last_error: row.get("last_error")?,
        created_at: time::parse_instant("created_at", &row.get::<_, String>("created_at")?)?,
        updated_at: time::parse_instant("updated_at", &row.get::<_, String>("updated_at")?)?,
    })
}

fn read_event(row: &Row<'_>) -> Result<SubscriptionEvent> {
    Ok(SubscriptionEvent {
        id: row.get("id")?,
        subscription_id: row.get("subscription_id")?,
        uid: row.get("uid")?,
        title: row.get("title")?,
        location: row.get("location")?,
        all_day: row.get("all_day")?,
        start_at: opt_instant("start_at", row.get("start_at")?)?,
        end_at: opt_instant("end_at", row.get("end_at")?)?,
        start_date: opt_date("start_date", row.get("start_date")?)?,
        end_date: opt_date("end_date", row.get("end_date")?)?,
        busy: row.get("busy")?,
    })
}

/// Inserts or fully overwrites a subscription (its events are kept).
pub fn upsert(conn: &Connection, s: &Subscription) -> Result<()> {
    conn.execute(
        &format!(
            "INSERT INTO ics_subscriptions ({COLUMNS}) VALUES (:id, :name, :url, :refresh_min, \
             :color, :as_blockout, :enabled, :last_fetch_at, :last_ok_at, :last_status, \
             :last_error, :created_at, :updated_at) \
             ON CONFLICT (id) DO UPDATE SET name = excluded.name, url = excluded.url, \
             refresh_min = excluded.refresh_min, color = excluded.color, \
             as_blockout = excluded.as_blockout, enabled = excluded.enabled, \
             last_fetch_at = excluded.last_fetch_at, last_ok_at = excluded.last_ok_at, \
             last_status = excluded.last_status, last_error = excluded.last_error, \
             created_at = excluded.created_at, updated_at = excluded.updated_at"
        ),
        named_params! {
            ":id": s.id,
            ":name": s.name,
            ":url": s.url,
            ":refresh_min": s.refresh_min,
            ":color": s.color,
            ":as_blockout": s.as_blockout,
            ":enabled": s.enabled,
            ":last_fetch_at": fmt_opt_instant(s.last_fetch_at),
            ":last_ok_at": fmt_opt_instant(s.last_ok_at),
            ":last_status": s.last_status,
            ":last_error": s.last_error,
            ":created_at": time::fmt_instant(s.created_at),
            ":updated_at": time::fmt_instant(s.updated_at),
        },
    )?;
    Ok(())
}

pub fn get(conn: &Connection, id: &str) -> Result<Option<Subscription>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {COLUMNS} FROM ics_subscriptions WHERE id = ?1"
    ))?;
    stmt.query_row([id], |row| Ok(read(row)))
        .optional()?
        .transpose()
}

pub fn require(conn: &Connection, id: &str) -> Result<Subscription> {
    get(conn, id)?.ok_or_else(|| CoreError::not_found("subscription", id))
}

/// Every subscription, oldest first.
pub fn list(conn: &Connection) -> Result<Vec<Subscription>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {COLUMNS} FROM ics_subscriptions ORDER BY created_at, id"
    ))?;
    let rows = stmt.query_map([], |row| Ok(read(row)))?;
    rows.map(|r| r?).collect()
}

/// Deletes a subscription and its cached events.
pub fn delete(conn: &Connection, id: &str) -> Result<bool> {
    Ok(conn.execute("DELETE FROM ics_subscriptions WHERE id = ?1", [id])? > 0)
}

/// Replaces every cached event of `subscription_id` with `events`.
pub fn replace_events(
    conn: &Connection,
    subscription_id: &str,
    events: &[SubscriptionEvent],
) -> Result<()> {
    conn.execute(
        "DELETE FROM ics_events WHERE subscription_id = ?1",
        [subscription_id],
    )?;
    let mut stmt = conn.prepare(&format!(
        "INSERT INTO ics_events ({EVENT_COLUMNS}) VALUES (:id, :subscription_id, :uid, :title, \
         :location, :all_day, :start_at, :end_at, :start_date, :end_date, :busy)"
    ))?;
    for e in events {
        stmt.execute(named_params! {
            ":id": e.id,
            ":subscription_id": subscription_id,
            ":uid": e.uid,
            ":title": e.title,
            ":location": e.location,
            ":all_day": e.all_day,
            ":start_at": fmt_opt_instant(e.start_at),
            ":end_at": fmt_opt_instant(e.end_at),
            ":start_date": fmt_opt_date(e.start_date),
            ":end_date": fmt_opt_date(e.end_date),
            ":busy": e.busy,
        })?;
    }
    Ok(())
}

/// How many events are cached for `subscription_id`.
pub fn count_events(conn: &Connection, subscription_id: &str) -> Result<usize> {
    let n: i64 = conn.query_row(
        "SELECT COUNT(*) FROM ics_events WHERE subscription_id = ?1",
        [subscription_id],
        |r| r.get(0),
    )?;
    Ok(usize::try_from(n).unwrap_or(0))
}

/// Cached events of enabled subscriptions overlapping `range`, by start.
pub fn events_in_range(conn: &Connection, range: ItemRange) -> Result<Vec<SubscriptionEvent>> {
    let sql = format!(
        "SELECT {} FROM ics_events e JOIN ics_subscriptions s ON s.id = e.subscription_id \
         WHERE s.enabled = 1 AND ( \
           (e.all_day = 0 AND e.start_at < :end \
             AND (COALESCE(e.end_at, e.start_at) > :start OR e.start_at >= :start)) \
           OR (e.all_day = 1 AND e.start_date <= :end_date \
             AND COALESCE(e.end_date, e.start_date) >= :start_date)) \
         ORDER BY COALESCE(e.start_at, e.start_date), e.title",
        prefixed("e")
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(
        named_params! {
            ":start": time::fmt_instant(range.start),
            ":end": time::fmt_instant(range.end),
            ":start_date": time::fmt_date(range.start_date),
            ":end_date": time::fmt_date(range.end_date),
        },
        |row| Ok(read_event(row)),
    )?;
    rows.map(|r| r?).collect()
}

/// Busy, timed events of enabled block-out subscriptions overlapping
/// `[from, to)`, with their subscription's name and colour.
pub fn blocking_events(
    conn: &Connection,
    from: Instant,
    to: Instant,
) -> Result<Vec<(SubscriptionEvent, String, Option<String>)>> {
    let sql = format!(
        "SELECT {}, s.name AS sub_name, s.color AS sub_color \
         FROM ics_events e JOIN ics_subscriptions s ON s.id = e.subscription_id \
         WHERE s.enabled = 1 AND s.as_blockout = 1 AND e.busy = 1 AND e.all_day = 0 \
           AND e.start_at < :to AND e.end_at > :from \
         ORDER BY e.start_at",
        prefixed("e")
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(
        named_params! {
            ":from": time::fmt_instant(from),
            ":to": time::fmt_instant(to),
        },
        |row| {
            Ok(read_event(row).and_then(|e| {
                Ok((
                    e,
                    row.get::<_, String>("sub_name")?,
                    row.get::<_, Option<String>>("sub_color")?,
                ))
            }))
        },
    )?;
    rows.map(|r| r?).collect()
}

fn prefixed(alias: &str) -> String {
    EVENT_COLUMNS
        .split(", ")
        .map(|c| format!("{alias}.{c}"))
        .collect::<Vec<_>>()
        .join(", ")
}

use rusqlite::{Connection, OptionalExtension, Row, named_params};

use super::{fmt_opt_instant, json_value, opt_instant, parse_enum};
use crate::error::Result;
use crate::model::Change;
use crate::time;

const COLUMNS: &str = "id, batch_id, op, entity, entity_id, before_json, after_json, actor, created_at, undone_at, \
     reverts";

fn read(row: &Row<'_>) -> Result<Change> {
    Ok(Change {
        id: row.get("id")?,
        batch_id: row.get("batch_id")?,
        op: row.get("op")?,
        entity: row.get("entity")?,
        entity_id: row.get("entity_id")?,
        before: json_value("before_json", row.get("before_json")?)?,
        after: json_value("after_json", row.get("after_json")?)?,
        actor: parse_enum("actor", row.get("actor")?)?,
        created_at: time::parse_instant("created_at", &row.get::<_, String>("created_at")?)?,
        undone_at: opt_instant("undone_at", row.get("undone_at")?)?,
        reverts: row.get("reverts")?,
    })
}

fn collect(conn: &Connection, sql: &str, params: impl rusqlite::Params) -> Result<Vec<Change>> {
    let mut stmt = conn.prepare(sql)?;
    let mut rows = stmt.query(params)?;
    let mut out = Vec::new();
    while let Some(row) = rows.next()? {
        out.push(read(row)?);
    }
    Ok(out)
}

pub fn insert(conn: &Connection, c: &Change) -> Result<()> {
    conn.execute(
        &format!(
            "INSERT INTO changes ({COLUMNS}) VALUES (:id, :batch_id, :op, :entity, :entity_id, \
             :before, :after, :actor, :created_at, :undone_at, :reverts)"
        ),
        named_params! {
            ":id": c.id,
            ":batch_id": c.batch_id,
            ":op": c.op,
            ":entity": c.entity,
            ":entity_id": c.entity_id,
            ":before": c.before.as_ref().map(serde_json::to_string).transpose()?,
            ":after": c.after.as_ref().map(serde_json::to_string).transpose()?,
            ":actor": c.actor.to_string(),
            ":created_at": time::fmt_instant(c.created_at),
            ":undone_at": fmt_opt_instant(c.undone_at),
            ":reverts": c.reverts,
        },
    )?;
    Ok(())
}

/// Every row of one batch, in the order they were written (rowid order: ULIDs
/// made in the same millisecond don't sort by time).
pub fn list_batch(conn: &Connection, batch_id: &str) -> Result<Vec<Change>> {
    collect(
        conn,
        &format!("SELECT {COLUMNS} FROM changes WHERE batch_id = ?1 ORDER BY rowid"),
        [batch_id],
    )
}

/// Most recent rows first.
pub fn list_recent(conn: &Connection, limit: u32) -> Result<Vec<Change>> {
    collect(
        conn,
        &format!("SELECT {COLUMNS} FROM changes ORDER BY rowid DESC LIMIT ?1"),
        [limit],
    )
}

/// History of one entity, oldest first.
pub fn list_for_entity(conn: &Connection, entity: &str, entity_id: &str) -> Result<Vec<Change>> {
    collect(
        conn,
        &format!(
            "SELECT {COLUMNS} FROM changes WHERE entity = ?1 AND entity_id = ?2 ORDER BY rowid"
        ),
        [entity, entity_id],
    )
}

/// Ops whose rows are undo bookkeeping rather than edits.
const REVERSAL_OPS: &str = "('undo', 'redo')";

/// The newest batch of edits that hasn't been undone: what undo reverses.
pub fn newest_undoable(conn: &Connection) -> Result<Option<String>> {
    Ok(conn
        .query_row(
            &format!(
                "SELECT batch_id FROM changes WHERE op NOT IN {REVERSAL_OPS} \
                 AND undone_at IS NULL ORDER BY rowid DESC LIMIT 1"
            ),
            [],
            |r| r.get(0),
        )
        .optional()?)
}

/// The batch redo re-applies: the one the most recent undo reversed, if
/// it's still undone and no edit has been made since that undo.
pub fn newest_redoable(conn: &Connection) -> Result<Option<String>> {
    let last_undo: Option<(i64, String)> = conn
        .query_row(
            "SELECT c.rowid, c.reverts FROM changes c WHERE c.op = 'undo' \
             AND EXISTS (SELECT 1 FROM changes o WHERE o.batch_id = c.reverts \
                         AND o.undone_at IS NOT NULL) \
             ORDER BY c.rowid DESC LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    let Some((undo_row, batch)) = last_undo else {
        return Ok(None);
    };
    let edited_since: bool = conn.query_row(
        &format!(
            "SELECT EXISTS (SELECT 1 FROM changes WHERE op NOT IN {REVERSAL_OPS} \
             AND rowid > ?1)"
        ),
        [undo_row],
        |r| r.get(0),
    )?;
    Ok((!edited_since).then_some(batch))
}

/// Marks a batch undone (`Some`) or redone (`None`).
pub fn set_undone(conn: &Connection, batch_id: &str, at: Option<time::Instant>) -> Result<()> {
    conn.execute(
        "UPDATE changes SET undone_at = ?2 WHERE batch_id = ?1",
        rusqlite::params![batch_id, fmt_opt_instant(at)],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::model::Actor;
    use chrono::{TimeZone, Utc};
    use serde_json::json;

    #[test]
    fn change_round_trip_and_batch_listing() {
        let db = Database::open_in_memory().unwrap();
        let now = Utc.with_ymd_and_hms(2026, 10, 14, 9, 0, 0).unwrap();
        let mk = |id: &str, batch: &str, after: serde_json::Value| Change {
            id: id.into(),
            batch_id: batch.into(),
            op: "create_item".into(),
            entity: "item".into(),
            entity_id: "i1".into(),
            before: None,
            after: Some(after),
            actor: Actor::Ai {
                provider: "anthropic".into(),
                model: "claude-sonnet-5-5".into(),
            },
            created_at: now,
            undone_at: None,
            reverts: None,
        };
        let a = mk("01", "b1", json!({ "title": "Gym" }));
        let b = mk("02", "b1", json!({ "title": "Gym", "priority": 1 }));
        let c = mk("03", "b2", json!({}));
        for ch in [&a, &b, &c] {
            insert(db.conn(), ch).unwrap();
        }
        assert_eq!(
            list_batch(db.conn(), "b1").unwrap(),
            vec![a.clone(), b.clone()]
        );
        assert_eq!(list_recent(db.conn(), 1).unwrap(), vec![c]);
        assert_eq!(list_for_entity(db.conn(), "item", "i1").unwrap().len(), 3);
    }
}

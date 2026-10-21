//! Tag records: a colour for a tag, or a tag made ahead of any item using it.
//! Tags in use live on the items themselves; names here are lowercase.

use std::collections::HashSet;

use rusqlite::{Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};

use crate::error::Result;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tag {
    pub name: String,
    /// A CSS colour: `#rrggbb` or a theme tag token like `var(--tag-work)`.
    pub color: Option<String>,
    /// Its tasks can be done at the same time as other tasks.
    #[serde(default)]
    pub shares_time: bool,
}

fn from_row(r: &Row<'_>) -> rusqlite::Result<Tag> {
    Ok(Tag {
        name: r.get(0)?,
        color: r.get(1)?,
        shares_time: r.get(2)?,
    })
}

pub fn get(conn: &Connection, name: &str) -> Result<Option<Tag>> {
    Ok(conn
        .query_row(
            "SELECT name, color, shares_time FROM tags WHERE name = ?1",
            [name],
            from_row,
        )
        .optional()?)
}

pub fn list(conn: &Connection) -> Result<Vec<Tag>> {
    let mut stmt = conn.prepare("SELECT name, color, shares_time FROM tags ORDER BY name")?;
    let rows = stmt.query_map([], from_row)?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

/// Names of the tags whose tasks can share time.
pub fn sharing(conn: &Connection) -> Result<HashSet<String>> {
    let mut stmt = conn.prepare("SELECT name FROM tags WHERE shares_time")?;
    let rows = stmt.query_map([], |r| r.get(0))?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

pub fn upsert(conn: &Connection, tag: &Tag) -> Result<()> {
    conn.execute(
        "INSERT INTO tags (name, color, shares_time) VALUES (?1, ?2, ?3) \
         ON CONFLICT (name) DO UPDATE SET color = excluded.color, \
         shares_time = excluded.shares_time",
        (&tag.name, &tag.color, tag.shares_time),
    )?;
    Ok(())
}

pub fn delete(conn: &Connection, name: &str) -> Result<bool> {
    Ok(conn.execute("DELETE FROM tags WHERE name = ?1", [name])? > 0)
}

/// Ids of items carrying `name` (case-insensitive).
pub fn items_with(conn: &Connection, name: &str) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT id FROM items WHERE EXISTS \
         (SELECT 1 FROM json_each(items.tags) AS t WHERE lower(t.value) = ?1) ORDER BY id",
    )?;
    let rows = stmt.query_map([name], |r| r.get(0))?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

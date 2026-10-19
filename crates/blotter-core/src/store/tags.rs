//! Tag records: a colour for a tag, or a tag made ahead of any item using it.
//! Tags in use live on the items themselves; names here are lowercase.

use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::error::Result;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tag {
    pub name: String,
    /// A CSS colour: `#rrggbb` or a theme tag token like `var(--tag-work)`.
    pub color: Option<String>,
}

pub fn get(conn: &Connection, name: &str) -> Result<Option<Tag>> {
    Ok(conn
        .query_row(
            "SELECT name, color FROM tags WHERE name = ?1",
            [name],
            |r| {
                Ok(Tag {
                    name: r.get(0)?,
                    color: r.get(1)?,
                })
            },
        )
        .optional()?)
}

pub fn list(conn: &Connection) -> Result<Vec<Tag>> {
    let mut stmt = conn.prepare("SELECT name, color FROM tags ORDER BY name")?;
    let rows = stmt.query_map([], |r| {
        Ok(Tag {
            name: r.get(0)?,
            color: r.get(1)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

pub fn upsert(conn: &Connection, tag: &Tag) -> Result<()> {
    conn.execute(
        "INSERT INTO tags (name, color) VALUES (?1, ?2) \
         ON CONFLICT (name) DO UPDATE SET color = excluded.color",
        (&tag.name, &tag.color),
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

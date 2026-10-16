//! User themes (roadmap §5.5). The built-in presets ship with the
//! frontend; these are the user's own, as a name and a set of CSS tokens.

use std::collections::BTreeMap;

use rusqlite::{Connection, Row};
use serde::{Deserialize, Serialize};

use crate::error::{CoreError, Result};
use crate::time::{self, Instant};

/// Most tokens one theme may set, and the longest name or value.
const MAX_TOKENS: usize = 200;
const MAX_VALUE: usize = 300;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserTheme {
    pub id: String,
    pub name: String,
    /// CSS custom property (`--color-bg`) → value.
    pub tokens: BTreeMap<String, String>,
    pub created_at: Instant,
    pub updated_at: Instant,
}

/// Refuses tokens that aren't plain custom properties with plain values:
/// a theme must not be able to load anything (`url(`, `@import`).
pub fn check_tokens(tokens: &BTreeMap<String, String>) -> Result<()> {
    if tokens.len() > MAX_TOKENS {
        return Err(CoreError::invalid("a theme has at most 200 tokens"));
    }
    for (name, value) in tokens {
        let name_ok = name.starts_with("--")
            && name.len() <= 64
            && name[2..]
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-');
        if !name_ok {
            return Err(CoreError::invalid(format!("'{name}' isn't a theme token")));
        }
        let lower = value.to_ascii_lowercase();
        let value_ok = value.len() <= MAX_VALUE
            && !value.contains([';', '{', '}', '<', '>', '\\'])
            && !lower.contains("url(")
            && !lower.contains("@import")
            && !lower.contains("expression(");
        if !value_ok {
            return Err(CoreError::invalid(format!(
                "the value of {name} can't be used in a theme"
            )));
        }
    }
    Ok(())
}

fn read(row: &Row<'_>) -> Result<UserTheme> {
    let raw: String = row.get("tokens_json")?;
    let tokens = serde_json::from_str(&raw).map_err(|_| CoreError::Corrupt {
        column: "themes.tokens_json",
        value: raw,
    })?;
    Ok(UserTheme {
        id: row.get("id")?,
        name: row.get("name")?,
        tokens,
        created_at: time::parse_instant("created_at", &row.get::<_, String>("created_at")?)?,
        updated_at: time::parse_instant("updated_at", &row.get::<_, String>("updated_at")?)?,
    })
}

pub fn list(conn: &Connection) -> Result<Vec<UserTheme>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, tokens_json, created_at, updated_at FROM themes ORDER BY created_at, id",
    )?;
    let rows = stmt.query_map([], |row| Ok(read(row)))?;
    rows.map(|r| r?).collect()
}

/// Inserts or overwrites a theme (its `created_at` is kept on overwrite).
pub fn upsert(conn: &Connection, t: &UserTheme) -> Result<()> {
    let name = t.name.trim();
    if name.is_empty() || name.chars().count() > 60 {
        return Err(CoreError::invalid("a theme name is 1 to 60 characters"));
    }
    if t.id.is_empty() || t.id.len() > 64 {
        return Err(CoreError::invalid("bad theme id"));
    }
    check_tokens(&t.tokens)?;
    conn.execute(
        "INSERT INTO themes (id, name, tokens_json, created_at, updated_at) \
         VALUES (?1, ?2, ?3, ?4, ?5) \
         ON CONFLICT (id) DO UPDATE SET name = excluded.name, \
         tokens_json = excluded.tokens_json, updated_at = excluded.updated_at",
        (
            &t.id,
            name,
            serde_json::to_string(&t.tokens)?,
            time::fmt_instant(t.created_at),
            time::fmt_instant(t.updated_at),
        ),
    )?;
    Ok(())
}

pub fn delete(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM themes WHERE id = ?1", [id])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::*;
    use crate::Database;

    fn theme(tokens: &[(&str, &str)]) -> UserTheme {
        let now = Utc.with_ymd_and_hms(2026, 10, 16, 0, 0, 0).unwrap();
        UserTheme {
            id: "t1".into(),
            name: "Night".into(),
            tokens: tokens
                .iter()
                .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
                .collect(),
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn themes_round_trip() {
        let db = Database::open_in_memory().unwrap();
        let t = theme(&[
            ("--color-bg", "#101010"),
            ("--font-ui", "\"Geist\", system-ui"),
        ]);
        upsert(db.conn(), &t).unwrap();
        assert_eq!(list(db.conn()).unwrap(), std::slice::from_ref(&t));
        let mut renamed = t;
        renamed.name = "Midnight".into();
        upsert(db.conn(), &renamed).unwrap();
        assert_eq!(list(db.conn()).unwrap()[0].name, "Midnight");
        delete(db.conn(), "t1").unwrap();
        assert!(list(db.conn()).unwrap().is_empty());
    }

    #[test]
    fn unsafe_tokens_are_refused() {
        for bad in [
            ("color-bg", "#000"),
            ("--x", "red; background: url(x)"),
            ("--x", "url(https://example.com/x.png)"),
            ("--x", "}"),
            ("--x y", "#000"),
        ] {
            assert!(check_tokens(&theme(&[bad]).tokens).is_err(), "{bad:?}");
        }
        let ok = theme(&[(
            "--shadow-overlay",
            "0 0 0 1px #5a524c, 0 12px 32px rgba(0,0,0,.45)",
        )]);
        check_tokens(&ok.tokens).unwrap();
    }
}

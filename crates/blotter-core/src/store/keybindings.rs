//! The user's keybinding overrides (roadmap §5.4). Defaults live with the
//! actions in the frontend; a row here replaces an action's default
//! bindings with one binding, or with none when `binding` is empty.

use std::collections::BTreeMap;

use rusqlite::Connection;

use crate::error::{CoreError, Result};
use crate::time::{self, Instant};

/// Longest action id or binding accepted.
const MAX_LEN: usize = 100;

fn check(action_id: &str, binding: &str) -> Result<()> {
    if action_id.trim().is_empty() || action_id.len() > MAX_LEN {
        return Err(CoreError::invalid("an action id is 1 to 100 characters"));
    }
    if binding.len() > MAX_LEN {
        return Err(CoreError::invalid("a binding is at most 100 characters"));
    }
    Ok(())
}

/// Every override, by action id. An empty binding means "unbound".
pub fn list(conn: &Connection) -> Result<BTreeMap<String, String>> {
    let mut stmt = conn.prepare("SELECT action_id, binding FROM keybindings ORDER BY action_id")?;
    let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

/// Overrides one action's bindings (`""` unbinds it).
pub fn set(conn: &Connection, action_id: &str, binding: &str, now: Instant) -> Result<()> {
    check(action_id, binding)?;
    conn.execute(
        "INSERT INTO keybindings (action_id, binding, updated_at) VALUES (?1, ?2, ?3) \
         ON CONFLICT (action_id) DO UPDATE SET binding = excluded.binding, \
         updated_at = excluded.updated_at",
        (action_id, binding.trim(), time::fmt_instant(now)),
    )?;
    Ok(())
}

/// Puts one action back to its defaults.
pub fn reset(conn: &Connection, action_id: &str) -> Result<()> {
    conn.execute("DELETE FROM keybindings WHERE action_id = ?1", [action_id])?;
    Ok(())
}

/// Replaces every override with `overrides` (empty puts all back to defaults).
pub fn replace_all(
    conn: &mut Connection,
    overrides: &BTreeMap<String, String>,
    now: Instant,
) -> Result<()> {
    for (id, binding) in overrides {
        check(id, binding)?;
    }
    let tx = conn.transaction()?;
    tx.execute("DELETE FROM keybindings", [])?;
    for (id, binding) in overrides {
        set(&tx, id, binding, now)?;
    }
    tx.commit()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::*;
    use crate::Database;

    #[test]
    fn overrides_round_trip() {
        let mut db = Database::open_in_memory().unwrap();
        let now = Utc.with_ymd_and_hms(2026, 10, 16, 0, 0, 0).unwrap();
        set(db.conn(), "palette.toggle", "ctrl+p", now).unwrap();
        set(db.conn(), "quick.add", "", now).unwrap();
        set(db.conn(), "palette.toggle", "g p", now).unwrap();
        let got = list(db.conn()).unwrap();
        assert_eq!(got.get("palette.toggle").map(String::as_str), Some("g p"));
        assert_eq!(got.get("quick.add").map(String::as_str), Some(""));

        reset(db.conn(), "quick.add").unwrap();
        assert_eq!(list(db.conn()).unwrap().len(), 1);

        let mut all = BTreeMap::new();
        all.insert("go.week".to_owned(), "w".to_owned());
        replace_all(db.conn_mut(), &all, now).unwrap();
        assert_eq!(list(db.conn()).unwrap(), all);
        assert!(set(db.conn(), "", "x", now).is_err());
        assert!(set(db.conn(), "x", &"a".repeat(101), now).is_err());
    }
}

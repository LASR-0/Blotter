//! Reading a replica's rows as [`RowState`]s: what it pushes, serves in a
//! bootstrap, and reports after a merge.

use std::collections::BTreeSet;

use rusqlite::types::{Value as SqlValue, ValueRef};
use rusqlite::{Connection, OptionalExtension};
use serde_json::{Map, Number, Value};

use crate::error::{CoreError, Result};
use crate::sync::capture::{self, Pending, ROW, Role};
use crate::sync::registry::{self, Group, TABLES, Table};
use crate::sync::wire::{GroupState, RowState};

pub(crate) fn to_json(v: ValueRef<'_>) -> Result<Value> {
    Ok(match v {
        ValueRef::Null => Value::Null,
        ValueRef::Integer(i) => Value::from(i),
        ValueRef::Real(f) => Number::from_f64(f).map_or(Value::Null, Value::Number),
        ValueRef::Text(t) => Value::String(String::from_utf8_lossy(t).into_owned()),
        ValueRef::Blob(_) => return Err(CoreError::invalid("synced columns never hold blobs")),
    })
}

pub(crate) fn to_sql(v: &Value) -> SqlValue {
    match v {
        Value::Null => SqlValue::Null,
        Value::Bool(b) => SqlValue::Integer(i64::from(*b)),
        Value::Number(n) => match (n.as_i64(), n.as_f64()) {
            (Some(i), _) => SqlValue::Integer(i),
            (None, Some(f)) => SqlValue::Real(f),
            (None, None) => SqlValue::Null,
        },
        Value::String(s) => SqlValue::Text(s.clone()),
        other => SqlValue::Text(other.to_string()),
    }
}

pub(crate) fn quoted_columns(columns: &[&str]) -> String {
    columns
        .iter()
        .map(|c| format!("\"{c}\""))
        .collect::<Vec<_>>()
        .join(", ")
}

/// The current values of `group`'s columns in a row, if the row exists.
pub(crate) fn values(
    conn: &Connection,
    table: &Table,
    key: &str,
    group: &Group,
) -> Result<Option<Map<String, Value>>> {
    let sql = format!(
        "SELECT {} FROM \"{}\" WHERE \"{}\" = ?1",
        quoted_columns(group.columns),
        table.name,
        table.key[0],
    );
    let row = conn
        .query_row(&sql, [key], |r| {
            let mut out = Vec::with_capacity(group.columns.len());
            for i in 0..group.columns.len() {
                out.push(to_json(r.get_ref(i)?));
            }
            Ok(out)
        })
        .optional()?;
    let Some(row) = row else { return Ok(None) };
    let mut map = Map::new();
    for (column, value) in group.columns.iter().zip(row) {
        map.insert((*column).to_owned(), value?);
    }
    Ok(Some(map))
}

/// One row as this replica holds it, or none if it has never seen it.
pub fn row(conn: &Connection, table: &str, key: &str) -> Result<Option<RowState>> {
    let Some(t) = registry::table(table) else {
        return Err(CoreError::invalid(format!("{table} isn't a synced table")));
    };
    let role = capture::role(conn)?;
    row_of(conn, t, key, role)
}

pub(crate) fn row_of(
    conn: &Connection,
    table: &Table,
    key: &str,
    role: Role,
) -> Result<Option<RowState>> {
    let Some((stamp, deleted)) = capture::stamp(conn, table.name, key, ROW)? else {
        return Ok(None);
    };
    let mut state = RowState {
        table: table.name.to_owned(),
        key: key.to_owned(),
        row: stamp,
        deleted,
        groups: Default::default(),
    };
    if deleted {
        return Ok(Some(state));
    }
    for group in capture::captured_groups(table, role) {
        let Some((hlc, _)) = capture::stamp(conn, table.name, key, group.name)? else {
            continue;
        };
        let Some(values) = values(conn, table, key, group)? else {
            // Stamped as live but not in the table: nothing to send.
            return Ok(None);
        };
        state
            .groups
            .insert(group.name.to_owned(), GroupState { hlc, values });
    }
    Ok(Some(state))
}

/// What to push: every row with a pending change, whole, and the pending
/// entries to acknowledge once the server has them.
pub fn outbox(conn: &Connection) -> Result<(Vec<RowState>, Vec<Pending>)> {
    let pending = capture::pending(conn)?;
    let role = capture::role(conn)?;
    let mut seen = BTreeSet::new();
    let mut rows = Vec::new();
    for p in &pending {
        if !seen.insert((p.table.clone(), p.row_key.clone())) {
            continue;
        }
        let Some(table) = registry::table(&p.table) else {
            continue;
        };
        if let Some(state) = row_of(conn, table, &p.row_key, role)? {
            rows.push(state);
        }
    }
    Ok((rows, pending))
}

/// Every row this replica knows, tombstones included: a bootstrap.
pub fn all(conn: &Connection) -> Result<Vec<RowState>> {
    let role = capture::role(conn)?;
    let mut out = Vec::new();
    for table in TABLES {
        if capture::captured_groups(table, role).is_empty() {
            continue;
        }
        let keys: Vec<String> = conn
            .prepare(
                "SELECT row_key FROM sync_clocks WHERE tbl = ?1 AND grp = '' ORDER BY row_key",
            )?
            .query_map([table.name], |r| r.get(0))?
            .collect::<rusqlite::Result<_>>()?;
        for key in keys {
            if let Some(state) = row_of(conn, table, &key, role)? {
                out.push(state);
            }
        }
    }
    Ok(out)
}

//! Change capture: every write to synced data, from any code path and any
//! process, is stamped and queued for the server (D-S7).
//!
//! Capture happens in SQLite triggers, generated from the sync registry and
//! installed when the database opens, so writes outside the operation layer
//! (settings, keybindings, themes, subscriptions, reminders firing, imports)
//! and from the MCP server's connection are all caught. The triggers tick the
//! hybrid logical clock in SQL, in the same format as [`Hlc`], and record:
//!
//! - per synced row and field group, the stamp that last set it;
//! - per synced row, under the group `""`, when it was last created or
//!   deleted (a tombstone when deleted);
//! - a pending flag on each of those: the outbox, cleared when the server
//!   acknowledges the stamp.
//!
//! Writes made while applying remote changes aren't captured. Server-owned
//! data is captured only when the replica's role is [`Role::Server`].

use std::fmt::Write as _;

use rusqlite::{Connection, OptionalExtension, TransactionBehavior};
use serde::Serialize;

use crate::error::{CoreError, Result};
use crate::sync::Hlc;
use crate::sync::registry::{self, Class, SERVER_OWNED_SETTINGS, SYNCED_SETTINGS, TABLES, Table};

/// The group name under which a row's own creation or deletion is stamped.
pub const ROW: &str = "";

/// Which kind of replica this database is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    /// A Blotter desktop: captures synced data only.
    Device,
    /// Blotter Sync's canonical planner: also captures server-owned data.
    Server,
}

impl Role {
    fn as_str(self) -> &'static str {
        match self {
            Role::Device => "device",
            Role::Server => "server",
        }
    }

    fn captures(self, class: Class) -> bool {
        match class {
            Class::Synced => true,
            Class::ServerOwned => self == Role::Server,
            Class::DeviceLocal => false,
        }
    }
}

/// One local change waiting for the server.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Pending {
    pub table: String,
    pub row_key: String,
    /// A field group, or [`ROW`] for the row's creation or deletion.
    pub group: String,
    pub hlc: Hlc,
    pub deleted: bool,
}

/// Physical time in milliseconds, inside SQL.
const NOW_MS: &str = "CAST(unixepoch('subsec') * 1000 AS INTEGER)";

/// Advances `sync_clock` for a local write, as `hlc::tick` does.
fn tick_sql() -> String {
    format!(
        "UPDATE sync_clock SET \
         wall_ms = CASE WHEN {NOW_MS} > wall_ms THEN {NOW_MS} \
                        WHEN counter >= 999999 THEN wall_ms + 1 ELSE wall_ms END, \
         counter = CASE WHEN {NOW_MS} > wall_ms THEN 0 \
                        WHEN counter >= 999999 THEN 0 ELSE counter + 1 END \
         WHERE id = 1;"
    )
}

const NOT_APPLYING: &str = "(SELECT applying FROM sync_state WHERE id = 1) = 0";

fn quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "''"))
}

fn quoted_list(items: &[&str]) -> String {
    items
        .iter()
        .map(|s| quote(s))
        .collect::<Vec<_>>()
        .join(", ")
}

/// A condition on the row (`alias` is NEW or OLD) for tables whose rows
/// don't all sync alike: settings sync per key.
fn row_filter(table: &Table, role: Role, alias: &str) -> Option<String> {
    if table.name != "settings" {
        return None;
    }
    let mut keys: Vec<&str> = SYNCED_SETTINGS.to_vec();
    if role == Role::Server {
        keys.extend_from_slice(SERVER_OWNED_SETTINGS);
    }
    Some(format!("{alias}.key IN ({})", quoted_list(&keys)))
}

/// The groups of `table` this role captures.
fn captured_groups(table: &Table, role: Role) -> Vec<&'static registry::Group> {
    if !role.captures(table.class) {
        return Vec::new();
    }
    table
        .groups
        .iter()
        .filter(|g| role.captures(g.class))
        .collect()
}

/// `INSERT … ON CONFLICT` of one stamp per group in `groups_select` (a
/// SELECT yielding a `grp` column), for the row `key_expr`.
fn stamp_sql(table: &str, key_expr: &str, groups_select: &str, deleted: bool) -> String {
    let deleted = i32::from(deleted);
    format!(
        "INSERT INTO sync_clocks (tbl, row_key, grp, hlc, deleted, pending) \
         SELECT {table}, CAST({key_expr} AS TEXT), g.grp, n.hlc, {deleted}, 1 \
         FROM sync_now n, ({groups_select}) g WHERE 1 \
         ON CONFLICT (tbl, row_key, grp) DO UPDATE SET \
         hlc = excluded.hlc, deleted = excluded.deleted, pending = 1;",
        table = quote(table),
    )
}

/// `SELECT '' AS grp UNION ALL SELECT 'timing' …`: the row and every group.
fn all_groups_select(groups: &[&registry::Group]) -> String {
    std::iter::once(ROW)
        .chain(groups.iter().map(|g| g.name))
        .map(|name| format!("SELECT {} AS grp", quote(name)))
        .collect::<Vec<_>>()
        .join(" UNION ALL ")
}

fn changed(columns: &[&str]) -> String {
    columns
        .iter()
        .map(|c| format!("NEW.\"{c}\" IS NOT OLD.\"{c}\""))
        .collect::<Vec<_>>()
        .join(" OR ")
}

fn when(conditions: &[Option<String>]) -> String {
    conditions
        .iter()
        .flatten()
        .map(|c| format!("({c})"))
        .collect::<Vec<_>>()
        .join(" AND ")
}

/// The capture triggers for `role`, in a fixed order.
pub(crate) fn triggers(role: Role) -> Vec<String> {
    let mut out = Vec::new();
    for table in TABLES {
        let groups = captured_groups(table, role);
        if groups.is_empty() {
            continue;
        }
        let [key] = table.key else {
            // Every shared table has a one-column key (registry tests).
            continue;
        };
        let name = table.name;
        let tick = tick_sql();
        let not_applying = Some(NOT_APPLYING.to_owned());
        let all = all_groups_select(&groups);
        let new_key = format!("NEW.\"{key}\"");
        let old_key = format!("OLD.\"{key}\"");
        let forget_groups = format!(
            "DELETE FROM sync_clocks WHERE tbl = {} AND row_key = CAST({old_key} AS TEXT) \
             AND grp <> '';",
            quote(name)
        );

        out.push(format!(
            "CREATE TRIGGER sync_capture_{name}_insert AFTER INSERT ON \"{name}\" \
             WHEN {when} BEGIN {tick} {stamp} END",
            when = when(&[not_applying.clone(), row_filter(table, role, "NEW")]),
            stamp = stamp_sql(name, &new_key, &all, false),
        ));

        let columns: Vec<&str> = groups
            .iter()
            .flat_map(|g| g.columns.iter().copied())
            .collect();
        let per_group = groups
            .iter()
            .map(|g| {
                format!(
                    "SELECT {} AS grp WHERE {}",
                    quote(g.name),
                    changed(g.columns)
                )
            })
            .collect::<Vec<_>>()
            .join(" UNION ALL ");
        out.push(format!(
            "CREATE TRIGGER sync_capture_{name}_update AFTER UPDATE ON \"{name}\" \
             WHEN {when} BEGIN {tick} {stamp} END",
            when = when(&[
                not_applying.clone(),
                Some(format!("{new_key} IS {old_key}")),
                row_filter(table, role, "NEW"),
                Some(changed(&columns)),
            ]),
            stamp = stamp_sql(name, &new_key, &per_group, false),
        ));

        // A changed key is the old row gone and a new one made.
        out.push(format!(
            "CREATE TRIGGER sync_capture_{name}_rekey AFTER UPDATE ON \"{name}\" \
             WHEN {when} BEGIN {tick} {forget_groups} {tombstone} {stamp} END",
            when = when(&[
                not_applying.clone(),
                Some(format!("{new_key} IS NOT {old_key}")),
            ]),
            tombstone = stamp_sql(
                name,
                &old_key,
                &format!("SELECT {} AS grp", quote(ROW)),
                true
            ),
            stamp = stamp_sql(name, &new_key, &all, false),
        ));

        out.push(format!(
            "CREATE TRIGGER sync_capture_{name}_delete AFTER DELETE ON \"{name}\" \
             WHEN {when} BEGIN {tick} {forget_groups} {tombstone} END",
            when = when(&[not_applying, row_filter(table, role, "OLD")]),
            tombstone = stamp_sql(
                name,
                &old_key,
                &format!("SELECT {} AS grp", quote(ROW)),
                true
            ),
        ));
    }
    out
}

fn fingerprint(sql: &[String]) -> String {
    let hash = sql
        .iter()
        .flat_map(|s| s.bytes().chain([0]))
        .fold(0xcbf2_9ce4_8422_2325_u64, |h, b| {
            (h ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3)
        });
    format!("{hash:016x}")
}

/// This database's role.
pub fn role(conn: &Connection) -> Result<Role> {
    let role: Option<String> = conn
        .query_row("SELECT role FROM sync_state WHERE id = 1", [], |r| r.get(0))
        .optional()?;
    match role.as_deref() {
        Some("server") => Ok(Role::Server),
        Some("device") | None => Ok(Role::Device),
        Some(other) => Err(CoreError::Corrupt {
            column: "sync_state.role",
            value: other.to_owned(),
        }),
    }
}

/// Makes this database a device's or the server's replica, reinstalling the
/// triggers to match. Only Blotter Sync sets [`Role::Server`].
pub fn set_role(conn: &mut Connection, role: Role) -> Result<()> {
    conn.execute(
        "UPDATE sync_state SET role = ?1 WHERE id = 1",
        [role.as_str()],
    )?;
    install(conn)
}

/// Installs the capture triggers if the installed ones don't match the
/// registry and role, and stamps rows that have no stamps yet (data from
/// before capture existed: an upgrade, or a pre-1.2.1 snapshot restored).
/// Safe to run from two connections at once.
pub(crate) fn install(conn: &mut Connection) -> Result<()> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let role = role(&tx)?;
    let sql = triggers(role);
    let wanted = fingerprint(&sql);
    let installed: Option<String> = tx
        .query_row("SELECT triggers FROM sync_state WHERE id = 1", [], |r| {
            r.get(0)
        })
        .optional()?
        .flatten();
    if installed.as_deref() == Some(wanted.as_str()) {
        return Ok(tx.commit()?);
    }

    let old: Vec<String> = {
        let mut stmt = tx.prepare(
            "SELECT name FROM sqlite_master WHERE type = 'trigger' AND name LIKE 'sync_capture_%'",
        )?;
        stmt.query_map([], |r| r.get(0))?
            .collect::<rusqlite::Result<_>>()?
    };
    for name in old {
        tx.execute_batch(&format!("DROP TRIGGER \"{name}\";"))?;
    }
    for trigger in &sql {
        tx.execute_batch(trigger)?;
    }
    backfill(&tx, role)?;
    tx.execute(
        "UPDATE sync_state SET triggers = ?1 WHERE id = 1",
        [&wanted],
    )?;
    Ok(tx.commit()?)
}

/// Stamps (and queues) every captured row and group that has no stamp,
/// with one tick of the clock. Existing stamps are left alone.
fn backfill(conn: &Connection, role: Role) -> Result<()> {
    let stamp = super::hlc::tick(conn, chrono::Utc::now())?.to_string();
    for table in TABLES {
        let groups = captured_groups(table, role);
        let [key] = table.key else { continue };
        if groups.is_empty() {
            continue;
        }
        let mut sql = format!(
            "INSERT OR IGNORE INTO sync_clocks (tbl, row_key, grp, hlc, deleted, pending) \
             SELECT {name}, CAST(t.\"{key}\" AS TEXT), g.grp, ?1, 0, 1 FROM \"{table}\" t, ({all}) g",
            name = quote(table.name),
            table = table.name,
            all = all_groups_select(&groups),
        );
        if let Some(filter) = row_filter(table, role, "t") {
            let _ = write!(sql, " WHERE {filter}");
        }
        conn.execute(&sql, [&stamp])?;
    }
    Ok(())
}

/// Turns capture off for the writes that apply remote changes. Always pair
/// it with `applying(conn, false)` in the same transaction.
// The merge (#76) is its first caller outside the tests.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn applying(conn: &Connection, on: bool) -> Result<()> {
    conn.execute(
        "UPDATE sync_state SET applying = ?1 WHERE id = 1",
        [i32::from(on)],
    )?;
    Ok(())
}

/// Local changes the server hasn't acknowledged, oldest first.
pub fn pending(conn: &Connection) -> Result<Vec<Pending>> {
    let mut stmt = conn.prepare(
        "SELECT tbl, row_key, grp, hlc, deleted FROM sync_clocks WHERE pending = 1 ORDER BY hlc",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, String>(3)?,
            r.get::<_, bool>(4)?,
        ))
    })?;
    rows.map(|row| {
        let (table, row_key, group, hlc, deleted) = row?;
        Ok(Pending {
            table,
            row_key,
            group,
            hlc: hlc.parse()?,
            deleted,
        })
    })
    .collect()
}

/// How many local changes are waiting for the server.
pub fn pending_count(conn: &Connection) -> Result<u64> {
    let n: i64 = conn.query_row(
        "SELECT count(*) FROM sync_clocks WHERE pending = 1",
        [],
        |r| r.get(0),
    )?;
    Ok(n.unsigned_abs())
}

/// Clears the given changes from the outbox, unless a newer local write has
/// stamped the same group since they were read.
pub fn acknowledge(conn: &Connection, changes: &[Pending]) -> Result<()> {
    let mut stmt = conn.prepare(
        "UPDATE sync_clocks SET pending = 0 \
         WHERE tbl = ?1 AND row_key = ?2 AND grp = ?3 AND hlc = ?4",
    )?;
    for c in changes {
        stmt.execute((&c.table, &c.row_key, &c.group, c.hlc.to_string()))?;
    }
    Ok(())
}

/// The stamp that last set a group of a row ([`ROW`] for the row itself),
/// and whether it is a tombstone.
pub fn stamp(
    conn: &Connection,
    table: &str,
    row_key: &str,
    group: &str,
) -> Result<Option<(Hlc, bool)>> {
    let row: Option<(String, bool)> = conn
        .query_row(
            "SELECT hlc, deleted FROM sync_clocks WHERE tbl = ?1 AND row_key = ?2 AND grp = ?3",
            (table, row_key, group),
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    row.map(|(hlc, deleted)| Ok((hlc.parse()?, deleted)))
        .transpose()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use chrono::Utc;
    use serde_json::json;

    use super::*;
    use crate::model::Actor;
    use crate::sync::{device, hlc};
    use crate::{Database, store};

    const T0: &str = "2026-10-22T00:00:00Z";

    fn add_item(conn: &Connection, id: &str) {
        conn.execute(
            "INSERT INTO items (id, kind, anchoring, title, created_at, updated_at) \
             VALUES (?1, 'task', 'flexible', 'Call plumber', ?2, ?2)",
            (id, T0),
        )
        .unwrap();
    }

    fn add_reminder(conn: &Connection, id: &str, item: &str) {
        conn.execute(
            "INSERT INTO reminders (id, item_id, offset_min, created_at) VALUES (?1, ?2, 10, ?3)",
            (id, item, T0),
        )
        .unwrap();
    }

    fn groups(conn: &Connection, table: &str, row: &str) -> BTreeSet<String> {
        pending(conn)
            .unwrap()
            .into_iter()
            .filter(|p| p.table == table && p.row_key == row)
            .map(|p| p.group)
            .collect()
    }

    fn just(group: &str) -> BTreeSet<String> {
        BTreeSet::from([group.to_owned()])
    }

    fn stamp_of(conn: &Connection, table: &str, row: &str, group: &str) -> Hlc {
        stamp(conn, table, row, group).unwrap().unwrap().0
    }

    fn clear(conn: &Connection) {
        acknowledge(conn, &pending(conn).unwrap()).unwrap();
    }

    #[test]
    fn a_new_row_stamps_itself_and_every_group() {
        let db = Database::open_in_memory().unwrap();
        add_item(db.conn(), "I1");
        let items = registry::table("items").unwrap();
        let mut want: BTreeSet<String> = items.groups.iter().map(|g| g.name.to_owned()).collect();
        want.insert(ROW.to_owned());
        assert_eq!(groups(db.conn(), "items", "I1"), want);
        let (row, deleted) = stamp(db.conn(), "items", "I1", ROW).unwrap().unwrap();
        assert!(!deleted);
        assert_eq!(row.device, device::id(db.conn()).unwrap());
        assert_eq!(row, stamp_of(db.conn(), "items", "I1", "title"));
    }

    #[test]
    fn an_update_stamps_only_the_groups_it_changed() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.conn();
        add_item(conn, "I1");
        let before = stamp_of(conn, "items", "I1", "timing");
        clear(conn);

        conn.execute(
            "UPDATE items SET title = 'Call the plumber', end_at = NULL WHERE id = 'I1'",
            [],
        )
        .unwrap();
        // end_at was already null: timing didn't change.
        assert_eq!(groups(conn, "items", "I1"), just("title"));
        assert_eq!(stamp_of(conn, "items", "I1", "timing"), before);
        assert!(stamp_of(conn, "items", "I1", "title") > before);

        clear(conn);
        conn.execute(
            "UPDATE items SET start_at = '2026-10-23T01:00:00Z', end_at = '2026-10-23T02:00:00Z' \
             WHERE id = 'I1'",
            [],
        )
        .unwrap();
        assert_eq!(groups(conn, "items", "I1"), just("timing"));
    }

    #[test]
    fn a_write_that_changes_nothing_isnt_captured() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.conn();
        add_item(conn, "I1");
        clear(conn);
        let clock = hlc::current(conn).unwrap();
        conn.execute("UPDATE items SET title = title WHERE id = 'I1'", [])
            .unwrap();
        assert_eq!(pending_count(conn).unwrap(), 0);
        assert_eq!(hlc::current(conn).unwrap(), clock, "no tick either");
    }

    #[test]
    fn device_local_columns_and_tables_arent_captured() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.conn();
        add_item(conn, "I1");
        add_reminder(conn, "R1", "I1");
        assert!(!groups(conn, "reminders", "R1").contains("fired_at"));
        clear(conn);

        store::reminders::set_fired(conn, "R1", Utc::now()).unwrap();
        assert_eq!(pending_count(conn).unwrap(), 0, "fired_at is per device");

        conn.execute(
            "INSERT INTO snapshots (id, reason, path, created_at) VALUES ('S1', 'manual', 'x', ?1)",
            [T0],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO changes (id, batch_id, op, entity, entity_id, actor, created_at) \
             VALUES ('C1', 'B1', 'x', 'item', 'I1', 'user', ?1)",
            [T0],
        )
        .unwrap();
        assert_eq!(pending_count(conn).unwrap(), 0);
    }

    #[test]
    fn a_snooze_is_captured() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.conn();
        add_item(conn, "I1");
        add_reminder(conn, "R1", "I1");
        clear(conn);
        conn.execute(
            "UPDATE reminders SET snoozed_until = '2026-10-22T00:15:00Z' WHERE id = 'R1'",
            [],
        )
        .unwrap();
        assert_eq!(groups(conn, "reminders", "R1"), just("snoozed_until"));
    }

    #[test]
    fn a_delete_leaves_a_tombstone_and_cascades_leave_theirs() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.conn();
        add_item(conn, "I1");
        add_reminder(conn, "R1", "I1");
        clear(conn);

        conn.execute("DELETE FROM items WHERE id = 'I1'", [])
            .unwrap();
        for (table, row) in [("items", "I1"), ("reminders", "R1")] {
            assert_eq!(groups(conn, table, row), just(ROW));
            let (_, deleted) = stamp(conn, table, row, ROW).unwrap().unwrap();
            assert!(deleted, "{table} {row} is a tombstone");
            assert_eq!(stamp(conn, table, row, "created_at").unwrap(), None);
        }

        // Made again (an undo): live again, stamped after the tombstone.
        let tombstone = stamp_of(conn, "items", "I1", ROW);
        add_item(conn, "I1");
        let (row, deleted) = stamp(conn, "items", "I1", ROW).unwrap().unwrap();
        assert!(!deleted && row > tombstone);
    }

    #[test]
    fn settings_sync_per_key() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.conn();
        clear(conn);
        let now = Utc::now();
        store::settings::set(conn, "timezone", &"Australia/Brisbane", now).unwrap();
        store::settings::set(conn, "theme", &"blotter-light", now).unwrap();
        store::settings::set(conn, "last_rollover_date", &"2026-10-22", now).unwrap();
        let keys: BTreeSet<String> = pending(conn)
            .unwrap()
            .into_iter()
            .map(|p| p.row_key)
            .collect();
        assert_eq!(keys, just("timezone"));

        store::settings::delete(conn, "timezone").unwrap();
        store::settings::delete(conn, "theme").unwrap();
        assert!(stamp(conn, "settings", "timezone", ROW).unwrap().unwrap().1);
        assert_eq!(stamp(conn, "settings", "theme", ROW).unwrap(), None);
    }

    #[test]
    fn writes_outside_the_op_layer_are_captured() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.conn();
        clear(conn);
        store::keybindings::set(conn, "view.week", "g w", Utc::now()).unwrap();
        conn.execute(
            "INSERT INTO themes (id, name, tokens_json, created_at, updated_at) \
             VALUES ('TH1', 'Mine', '{}', ?1, ?1)",
            [T0],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO ics_subscriptions (id, name, url, created_at, updated_at) \
             VALUES ('SUB1', 'Work', 'https://example.com/a.ics', ?1, ?1)",
            [T0],
        )
        .unwrap();
        let tables: BTreeSet<String> = pending(conn)
            .unwrap()
            .into_iter()
            .map(|p| p.table)
            .collect();
        assert_eq!(
            tables,
            BTreeSet::from(["keybindings", "themes", "ics_subscriptions"].map(String::from))
        );
    }

    #[test]
    fn operations_are_captured() {
        let mut p = crate::planner::tests::planner();
        clear(p.db().conn());
        let out = p
            .execute("create_item", json!({ "title": "Buy milk" }), &Actor::User)
            .unwrap();
        let id = out.result["id"].as_str().unwrap().to_owned();
        assert!(groups(p.db().conn(), "items", &id).contains(ROW));
        // The undo log itself is device-local.
        assert!(
            pending(p.db().conn())
                .unwrap()
                .iter()
                .all(|c| c.table != "changes")
        );
    }

    #[test]
    fn server_owned_data_is_captured_only_by_the_server() {
        let mut db = Database::open_in_memory().unwrap();
        db.conn()
            .execute(
                "INSERT INTO ics_subscriptions (id, name, url, created_at, updated_at) \
                 VALUES ('SUB1', 'Work', 'https://example.com/a.ics', ?1, ?1)",
                [T0],
            )
            .unwrap();
        clear(db.conn());
        db.conn()
            .execute(
                "INSERT INTO ics_events (id, subscription_id, uid, title) \
                 VALUES ('E1', 'SUB1', 'u1', 'Standup')",
                [],
            )
            .unwrap();
        db.conn()
            .execute(
                "UPDATE ics_subscriptions SET last_status = 'ok' WHERE id = 'SUB1'",
                [],
            )
            .unwrap();
        assert_eq!(pending_count(db.conn()).unwrap(), 0);

        set_role(db.conn_mut(), Role::Server).unwrap();
        assert_eq!(role(db.conn()).unwrap(), Role::Server);
        // Becoming the server stamps what it now owns.
        assert!(groups(db.conn(), "ics_events", "E1").contains("event"));
        assert!(groups(db.conn(), "ics_subscriptions", "SUB1").contains("fetch_status"));
        clear(db.conn());
        db.conn()
            .execute(
                "UPDATE ics_subscriptions SET last_status = 'error' WHERE id = 'SUB1'",
                [],
            )
            .unwrap();
        store::settings::set(db.conn(), "last_rollover_date", &"2026-10-22", Utc::now()).unwrap();
        assert_eq!(
            groups(db.conn(), "ics_subscriptions", "SUB1"),
            just("fetch_status")
        );
        assert!(groups(db.conn(), "settings", "last_rollover_date").contains("value"));
    }

    #[test]
    fn applying_remote_changes_isnt_captured() {
        let mut db = Database::open_in_memory().unwrap();
        clear(db.conn());
        let tx = db.conn_mut().transaction().unwrap();
        applying(&tx, true).unwrap();
        add_item(&tx, "I1");
        tx.execute("DELETE FROM items WHERE id = 'I1'", []).unwrap();
        applying(&tx, false).unwrap();
        tx.commit().unwrap();
        assert_eq!(pending_count(db.conn()).unwrap(), 0);
        add_item(db.conn(), "I2");
        assert!(pending_count(db.conn()).unwrap() > 0, "capture is back on");
    }

    #[test]
    fn another_connection_shares_the_clock_and_the_outbox() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(crate::db::DB_FILE_NAME);
        let app = Database::open(&path).unwrap();
        let mcp = Database::open(&path).unwrap();
        add_item(app.conn(), "I1");
        add_item(mcp.conn(), "I2");
        add_item(app.conn(), "I3");
        let [a, b, c] = ["I1", "I2", "I3"].map(|id| stamp_of(app.conn(), "items", id, ROW));
        assert!(a < b && b < c, "{a} {b} {c}");
        assert_eq!(b.device, device::id(app.conn()).unwrap());
    }

    #[test]
    fn sql_and_rust_ticks_interleave_in_order() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.conn();
        let rust = hlc::tick(conn, Utc::now()).unwrap();
        add_item(conn, "I1");
        assert!(stamp_of(conn, "items", "I1", ROW) > rust);
        // A Rust tick far ahead of the wall clock carries the SQL ticks past it.
        let ahead = hlc::tick(conn, Utc::now() + chrono::Duration::hours(1)).unwrap();
        conn.execute("UPDATE items SET title = 'b' WHERE id = 'I1'", [])
            .unwrap();
        let after = stamp_of(conn, "items", "I1", "title");
        assert!(after > ahead);
        assert_eq!(
            (after.wall_ms, after.counter),
            (ahead.wall_ms, ahead.counter + 1)
        );
    }

    #[test]
    fn acknowledging_keeps_newer_writes_pending() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.conn();
        add_item(conn, "I1");
        let sent = pending(conn).unwrap();
        conn.execute("UPDATE items SET title = 'b' WHERE id = 'I1'", [])
            .unwrap();
        acknowledge(conn, &sent).unwrap();
        assert_eq!(groups(conn, "items", "I1"), just("title"));
    }

    #[test]
    fn a_rekey_is_a_delete_and_a_create() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.conn();
        conn.execute("INSERT INTO tags (name, color) VALUES ('home', '#fff')", [])
            .unwrap();
        clear(conn);
        conn.execute("UPDATE tags SET name = 'house' WHERE name = 'home'", [])
            .unwrap();
        assert!(stamp(conn, "tags", "home", ROW).unwrap().unwrap().1);
        assert!(!stamp(conn, "tags", "house", ROW).unwrap().unwrap().1);
        assert!(groups(conn, "tags", "house").contains("color"));
        assert_eq!(stamp(conn, "tags", "home", "color").unwrap(), None);
    }

    #[test]
    fn upgrading_stamps_existing_rows_once() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(crate::db::DB_FILE_NAME);
        {
            let db = Database::open(&path).unwrap();
            crate::sync::downgrade_to_1_2_0(db.conn());
            add_item(db.conn(), "OLD");
            store::settings::set(db.conn(), "week_start", &"sun", Utc::now()).unwrap();
        }
        let db = Database::open(&path).unwrap();
        assert!(groups(db.conn(), "items", "OLD").contains(ROW));
        assert!(groups(db.conn(), "settings", "week_start").contains("value"));
        let first = stamp_of(db.conn(), "items", "OLD", "title");
        drop(db);
        let db = Database::open(&path).unwrap();
        assert_eq!(
            stamp_of(db.conn(), "items", "OLD", "title"),
            first,
            "not re-stamped"
        );
    }

    #[test]
    fn every_synced_table_has_its_four_triggers() {
        let db = Database::open_in_memory().unwrap();
        let names: BTreeSet<String> = db
            .conn()
            .prepare("SELECT name FROM sqlite_master WHERE type = 'trigger'")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap();
        let mut want = BTreeSet::new();
        for t in TABLES.iter().filter(|t| t.class == Class::Synced) {
            for kind in ["insert", "update", "rekey", "delete"] {
                want.insert(format!("sync_capture_{}_{kind}", t.name));
            }
        }
        assert_eq!(names, want);
    }

    #[test]
    fn a_wholesale_replace_is_deletes_and_creates() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.conn();
        add_item(conn, "I1");
        clear(conn);
        conn.execute("DELETE FROM items", []).unwrap();
        add_item(conn, "I1");
        let (_, deleted) = stamp(conn, "items", "I1", ROW).unwrap().unwrap();
        assert!(!deleted);
        assert!(groups(conn, "items", "I1").contains("title"));
    }
}

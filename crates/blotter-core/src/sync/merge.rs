//! Merging rows from another replica (roadmap §14.5). Every replica runs
//! this same function on the same rows, in any order, and ends up in the
//! same state:
//!
//! - **Field groups** are last-writer-wins: a group takes the incoming
//!   values when their stamp is newer than the one that set it here.
//! - **Rows** exist or not by the newer of their creation and deletion
//!   stamps, with a delete winning a tie. Edits only stamp groups, so an
//!   edit never brings back a row deleted elsewhere: delete wins over
//!   concurrent edits (D-S10). Making the row again (an undo) stamps it
//!   anew, after the delete.
//! - **Children follow their parent** (`ON DELETE CASCADE`): when a parent
//!   goes, its children are tombstoned too, at the later of the parent's
//!   tombstone and their own creation, so every replica stamps them alike.
//! - **Natural-unique collisions** (two block-out exceptions for the same
//!   day) keep the row created first and tombstone the other at its own
//!   creation stamp.
//!
//! A batch applies in one transaction with capture off, so remote changes
//! never go back out as local ones. A row that can't apply (an unknown
//! table, a group that doesn't fit, a constraint) is rejected on its own and
//! the rest still apply.

use std::collections::BTreeSet;

use rusqlite::{Connection, TransactionBehavior};
use serde_json::{Map, Value};

use crate::error::{CoreError, Result};
use crate::sync::capture::{self, ROW, Role};
use crate::sync::registry::{self, Class, Group, TABLES, Table};
use crate::sync::state::{self, quoted_columns, to_sql};
use crate::sync::wire::{GroupState, Merged, Rejection, RowState};
use crate::sync::{Hlc, hlc};
use crate::time::Instant;

type Touched = BTreeSet<(&'static str, String)>;

/// Merges `rows` from another replica into this one. Returns what changed
/// here (Blotter Sync logs it for every other device) and what was refused.
pub fn merge(conn: &mut Connection, rows: &[RowState], now: Instant) -> Result<Merged> {
    let mut tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let role = capture::role(&tx)?;
    capture::applying(&tx, true)?;
    tx.pragma_update(None, "defer_foreign_keys", "ON")?;

    if let Some(newest) = rows.iter().map(RowState::newest).max() {
        hlc::observe(&tx, newest, now)?;
    }

    let mut touched = Touched::new();
    let mut rejected = Vec::new();
    for row in rows {
        let sp = tx.savepoint()?;
        let mut here = Touched::new();
        match apply(&sp, role, row, &mut here) {
            Ok(()) => {
                sp.commit()?;
                touched.extend(here);
            }
            Err(err) => {
                drop(sp);
                rejected.push(Rejection {
                    table: row.table.clone(),
                    key: row.key.clone(),
                    reason: err.to_string(),
                });
            }
        }
    }
    orphans(&tx, &mut touched, &mut rejected)?;

    capture::applying(&tx, false)?;
    let mut changed = Vec::new();
    for (table, key) in &touched {
        let Some(t) = registry::table(table) else {
            continue;
        };
        if let Some(state) = state::row_of(&tx, t, key, role)? {
            changed.push(state);
        }
    }
    tx.commit()?;
    Ok(Merged { changed, rejected })
}

/// Whether an incoming group of this class may change this replica: devices
/// take synced and server-owned data (from the server), the server only
/// synced data (from devices).
fn accepts(role: Role, class: Class) -> bool {
    match class {
        Class::Synced => true,
        Class::ServerOwned => role == Role::Device,
        Class::DeviceLocal => false,
    }
}

fn apply(conn: &Connection, role: Role, row: &RowState, touched: &mut Touched) -> Result<()> {
    let table = registry::table(&row.table)
        .filter(|t| t.class != Class::DeviceLocal)
        .ok_or_else(|| CoreError::invalid(format!("{} isn't a synced table", row.table)))?;
    if !accepts(role, table.class) {
        return Err(CoreError::invalid(format!(
            "{} is the server's to write",
            table.name
        )));
    }
    if table.name == "settings" {
        let class = registry::setting_class(&row.key);
        if !accepts(role, class) {
            return Err(CoreError::invalid(format!(
                "the {} setting doesn't come from another replica",
                row.key
            )));
        }
    }
    let groups = incoming_groups(table, role, row)?;
    let key = row.key.as_str();

    let local = capture::stamp(conn, table.name, key, ROW)?;
    let incoming_wins = local
        .as_ref()
        .is_none_or(|(h, d)| (&row.row, row.deleted) > (h, *d));
    let exists = exists(conn, table, key)?;

    if row.deleted {
        if incoming_wins {
            tombstone(conn, table, key, &row.row, Derived::No, touched)?;
        }
        return Ok(());
    }
    // A delete here outranks this row: it saw the creation and removed it.
    if matches!(&local, Some((_, true))) && !incoming_wins {
        return Ok(());
    }

    if !exists {
        let final_row = row.row.clone();
        if collide(conn, table, key, &final_row, &groups, touched)? == Some(true) {
            return record_tombstone(conn, table, key, &final_row, Derived::Yes, touched);
        }
        insert(conn, table, key, &groups)?;
        for (group, g) in &groups {
            set_stamp(conn, table.name, key, group.name, &g.hlc, false)?;
        }
        set_stamp(conn, table.name, key, ROW, &row.row, false)?;
        touched.insert((table.name, key.to_owned()));
        return Ok(());
    }

    let (local_row, _) = local.unwrap_or((row.row.clone(), false));
    let final_row = local_row.clone().max(row.row.clone());
    let mut winners = Vec::new();
    for (group, g) in &groups {
        let here = capture::stamp(conn, table.name, key, group.name)?;
        if here.is_none_or(|(h, _)| g.hlc > h) {
            winners.push((*group, *g));
        }
    }
    if collide(conn, table, key, &final_row, &winners, touched)? == Some(true) {
        return tombstone(conn, table, key, &final_row, Derived::Yes, touched);
    }
    for (group, g) in &winners {
        update(conn, table, key, group, &g.values)?;
        set_stamp(conn, table.name, key, group.name, &g.hlc, false)?;
    }
    if row.row > local_row {
        set_stamp(conn, table.name, key, ROW, &row.row, false)?;
    }
    if !winners.is_empty() || row.row > local_row {
        touched.insert((table.name, key.to_owned()));
    }
    Ok(())
}

/// The incoming groups this replica takes, checked against the registry.
fn incoming_groups<'a>(
    table: &'static Table,
    role: Role,
    row: &'a RowState,
) -> Result<Vec<(&'static Group, &'a GroupState)>> {
    let mut out = Vec::new();
    for (name, g) in &row.groups {
        let group = table
            .groups
            .iter()
            .find(|x| x.name == name)
            .ok_or_else(|| CoreError::invalid(format!("{} has no group {name}", table.name)))?;
        let columns: BTreeSet<&str> = g.values.keys().map(String::as_str).collect();
        let wanted: BTreeSet<&str> = group.columns.iter().copied().collect();
        if columns != wanted {
            return Err(CoreError::invalid(format!(
                "{}.{name} carries {columns:?}, not {wanted:?}",
                table.name
            )));
        }
        if accepts(role, group.class) {
            out.push((group, g));
        }
    }
    Ok(out)
}

fn exists(conn: &Connection, table: &Table, key: &str) -> Result<bool> {
    Ok(conn
        .prepare_cached(&format!(
            "SELECT 1 FROM \"{}\" WHERE \"{}\" = ?1",
            table.name, table.key[0]
        ))?
        .exists([key])?)
}

/// Stamps a group (or the row) as the incoming change set it.
fn set_stamp(
    conn: &Connection,
    table: &str,
    key: &str,
    group: &str,
    hlc: &Hlc,
    deleted: bool,
) -> Result<()> {
    stamp_as(conn, table, key, group, hlc, deleted, Derived::No)
}

/// Whether this merge worked a tombstone out itself (a cascade, a lost
/// collision) rather than receiving it. Derived tombstones go out with the
/// next push, so replicas that never held the row learn of it too.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Derived {
    Yes,
    No,
}

fn stamp_as(
    conn: &Connection,
    table: &str,
    key: &str,
    group: &str,
    hlc: &Hlc,
    deleted: bool,
    derived: Derived,
) -> Result<()> {
    conn.execute(
        "INSERT INTO sync_clocks (tbl, row_key, grp, hlc, deleted, pending) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6) \
         ON CONFLICT (tbl, row_key, grp) DO UPDATE SET \
         hlc = excluded.hlc, deleted = excluded.deleted, pending = excluded.pending",
        (
            table,
            key,
            group,
            hlc.to_string(),
            deleted,
            derived == Derived::Yes,
        ),
    )?;
    Ok(())
}

fn insert(
    conn: &Connection,
    table: &Table,
    key: &str,
    groups: &[(&'static Group, &GroupState)],
) -> Result<()> {
    let mut columns = vec![table.key[0]];
    let mut values = vec![rusqlite::types::Value::Text(key.to_owned())];
    for (group, g) in groups {
        for column in group.columns {
            columns.push(column);
            values.push(to_sql(&g.values[*column]));
        }
    }
    let marks = (1..=columns.len())
        .map(|i| format!("?{i}"))
        .collect::<Vec<_>>()
        .join(", ");
    conn.execute(
        &format!(
            "INSERT INTO \"{}\" ({}) VALUES ({marks})",
            table.name,
            quoted_columns(&columns)
        ),
        rusqlite::params_from_iter(values),
    )?;
    Ok(())
}

fn update(
    conn: &Connection,
    table: &Table,
    key: &str,
    group: &Group,
    values: &Map<String, Value>,
) -> Result<()> {
    let sets = group
        .columns
        .iter()
        .enumerate()
        .map(|(i, c)| format!("\"{c}\" = ?{}", i + 2))
        .collect::<Vec<_>>()
        .join(", ");
    let mut params = vec![rusqlite::types::Value::Text(key.to_owned())];
    params.extend(group.columns.iter().map(|c| to_sql(&values[*c])));
    conn.execute(
        &format!(
            "UPDATE \"{}\" SET {sets} WHERE \"{}\" = ?1",
            table.name, table.key[0]
        ),
        rusqlite::params_from_iter(params),
    )?;
    Ok(())
}

/// Before `key` takes the incoming groups, settles a natural-unique
/// collision with another row: the row created first stays. Returns
/// `Some(true)` when `key` itself loses (the caller tombstones it),
/// `Some(false)` when the other row lost, `None` with no collision.
fn collide(
    conn: &Connection,
    table: &'static Table,
    key: &str,
    final_row: &Hlc,
    groups: &[(&'static Group, &GroupState)],
    touched: &mut Touched,
) -> Result<Option<bool>> {
    let mut outcome = None;
    for unique in table.unique {
        let Some((_, g)) = groups
            .iter()
            .find(|(group, _)| group.columns.contains(&unique[0]))
        else {
            continue;
        };
        let wheres = unique
            .iter()
            .enumerate()
            .map(|(i, c)| format!("\"{c}\" IS ?{}", i + 2))
            .collect::<Vec<_>>()
            .join(" AND ");
        let mut params = vec![rusqlite::types::Value::Text(key.to_owned())];
        params.extend(unique.iter().map(|c| to_sql(&g.values[*c])));
        let others: Vec<String> = conn
            .prepare(&format!(
                "SELECT \"{k}\" FROM \"{t}\" WHERE \"{k}\" <> ?1 AND {wheres}",
                k = table.key[0],
                t = table.name
            ))?
            .query_map(rusqlite::params_from_iter(params), |r| r.get(0))?
            .collect::<rusqlite::Result<_>>()?;
        for other in others {
            let theirs = capture::stamp(conn, table.name, &other, ROW)?
                .map(|(h, _)| h)
                .unwrap_or_else(|| final_row.clone());
            // Earlier creation stays; the same stamp falls back to the key.
            if (&theirs, other.as_str()) < (final_row, key) {
                return Ok(Some(true));
            }
            tombstone(conn, table, &other, &theirs, Derived::Yes, touched)?;
            outcome = Some(false);
        }
    }
    Ok(outcome)
}

/// Deletes a row and, first, its children, leaving tombstones. The
/// children's are always derived.
fn tombstone(
    conn: &Connection,
    table: &'static Table,
    key: &str,
    stamp: &Hlc,
    derived: Derived,
    touched: &mut Touched,
) -> Result<()> {
    tombstone_in(
        conn,
        table,
        key,
        stamp,
        derived,
        touched,
        &mut BTreeSet::new(),
    )
}

fn tombstone_in(
    conn: &Connection,
    table: &'static Table,
    key: &str,
    stamp: &Hlc,
    derived: Derived,
    touched: &mut Touched,
    visiting: &mut BTreeSet<(&'static str, String)>,
) -> Result<()> {
    if !visiting.insert((table.name, key.to_owned())) {
        return Ok(());
    }
    for child in TABLES {
        for parent in child.parents.iter().filter(|p| p.table == table.name) {
            let keys: Vec<String> = conn
                .prepare(&format!(
                    "SELECT \"{k}\" FROM \"{t}\" WHERE \"{c}\" = ?1",
                    k = child.key[0],
                    t = child.name,
                    c = parent.column
                ))?
                .query_map([key], |r| r.get(0))?
                .collect::<rusqlite::Result<_>>()?;
            for k in keys {
                let theirs = capture::stamp(conn, child.name, &k, ROW)?.map(|(h, _)| h);
                let at = theirs.map_or_else(|| stamp.clone(), |h| h.max(stamp.clone()));
                tombstone_in(conn, child, &k, &at, Derived::Yes, touched, visiting)?;
            }
        }
    }
    conn.execute(
        &format!(
            "DELETE FROM \"{}\" WHERE \"{}\" = ?1",
            table.name, table.key[0]
        ),
        [key],
    )?;
    record_tombstone(conn, table, key, stamp, derived, touched)
}

fn record_tombstone(
    conn: &Connection,
    table: &'static Table,
    key: &str,
    stamp: &Hlc,
    derived: Derived,
    touched: &mut Touched,
) -> Result<()> {
    conn.execute(
        "DELETE FROM sync_clocks WHERE tbl = ?1 AND row_key = ?2 AND grp <> ''",
        (table.name, key),
    )?;
    stamp_as(conn, table.name, key, ROW, stamp, true, derived)?;
    touched.insert((table.name, key.to_owned()));
    Ok(())
}

/// Rows whose parent is gone once the batch is in: tombstoned after the
/// parent's tombstone, or refused if this replica never knew the parent.
fn orphans(conn: &Connection, touched: &mut Touched, rejected: &mut Vec<Rejection>) -> Result<()> {
    loop {
        let mut found = false;
        for child in TABLES {
            for parent in child.parents {
                let Some(p) = registry::table(parent.table) else {
                    continue;
                };
                let rows: Vec<(String, String)> = conn
                    .prepare(&format!(
                        "SELECT c.\"{ck}\", c.\"{col}\" FROM \"{ct}\" c \
                         WHERE c.\"{col}\" IS NOT NULL AND NOT EXISTS \
                         (SELECT 1 FROM \"{pt}\" p WHERE p.\"{pk}\" = c.\"{col}\")",
                        ck = child.key[0],
                        col = parent.column,
                        ct = child.name,
                        pt = p.name,
                        pk = p.key[0],
                    ))?
                    .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
                    .collect::<rusqlite::Result<_>>()?;
                for (key, parent_key) in rows {
                    found = true;
                    match capture::stamp(conn, p.name, &parent_key, ROW)? {
                        Some((gone, true)) => {
                            let theirs =
                                capture::stamp(conn, child.name, &key, ROW)?.map(|(h, _)| h);
                            let at = theirs.map_or_else(|| gone.clone(), |h| h.max(gone.clone()));
                            tombstone(conn, child, &key, &at, Derived::Yes, touched)?;
                        }
                        _ => {
                            conn.execute(
                                &format!(
                                    "DELETE FROM \"{}\" WHERE \"{}\" = ?1",
                                    child.name, child.key[0]
                                ),
                                [&key],
                            )?;
                            conn.execute(
                                "DELETE FROM sync_clocks WHERE tbl = ?1 AND row_key = ?2",
                                (child.name, &key),
                            )?;
                            touched.remove(&(child.name, key.clone()));
                            rejected.push(Rejection {
                                table: child.name.to_owned(),
                                key,
                                reason: format!("its {} {parent_key} isn't known here", p.name),
                            });
                        }
                    }
                }
            }
        }
        if !found {
            return Ok(());
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use chrono::Utc;
    use serde_json::json;

    use super::*;
    use crate::Database;
    use crate::sync::capture::pending_count;
    use crate::sync::state::{all, outbox};

    const T0: &str = "2026-10-22T00:00:00Z";

    fn replica() -> Database {
        Database::open_in_memory().unwrap()
    }

    /// Sends `from`'s outbox to `to`, as a push or a pull would.
    fn send(from: &Database, to: &mut Database) -> Merged {
        let (rows, pending) = outbox(from.conn()).unwrap();
        let merged = merge(to.conn_mut(), &rows, Utc::now()).unwrap();
        capture::acknowledge(from.conn(), &pending).unwrap();
        merged
    }

    /// Both ways until neither has anything left to send.
    fn settle(a: &mut Database, b: &mut Database) {
        for _ in 0..4 {
            send(a, b);
            send(b, a);
        }
    }

    fn same(a: &Database, b: &Database) {
        assert_eq!(all(a.conn()).unwrap(), all(b.conn()).unwrap());
    }

    fn add_item(db: &Database, id: &str, title: &str) {
        db.conn()
            .execute(
                "INSERT INTO items (id, kind, anchoring, title, created_at, updated_at) \
                 VALUES (?1, 'task', 'flexible', ?2, ?3, ?3)",
                (id, title, T0),
            )
            .unwrap();
    }

    fn exec(db: &Database, sql: &str) {
        db.conn().execute(sql, []).unwrap();
    }

    fn title(db: &Database, id: &str) -> Option<String> {
        db.conn()
            .query_row("SELECT title FROM items WHERE id = ?1", [id], |r| r.get(0))
            .ok()
    }

    fn count(db: &Database, table: &str) -> i64 {
        db.conn()
            .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
            .unwrap()
    }

    #[test]
    fn a_new_row_arrives_whole_and_isnt_sent_back() {
        let (a, mut b) = (replica(), replica());
        add_item(&a, "I1", "Buy milk");
        let merged = send(&a, &mut b);
        assert_eq!(title(&b, "I1").as_deref(), Some("Buy milk"));
        assert_eq!(merged.changed.len(), 1);
        assert!(merged.rejected.is_empty());
        same(&a, &b);
        assert_eq!(
            pending_count(b.conn()).unwrap(),
            0,
            "remote changes aren't captured"
        );
    }

    #[test]
    fn edits_to_different_groups_both_survive() {
        let (mut a, mut b) = (replica(), replica());
        add_item(&a, "I1", "Buy milk");
        settle(&mut a, &mut b);
        // Both offline.
        exec(
            &a,
            "UPDATE items SET title = 'Buy oat milk' WHERE id = 'I1'",
        );
        exec(&b, "UPDATE items SET priority = 2 WHERE id = 'I1'");
        settle(&mut a, &mut b);
        same(&a, &b);
        assert_eq!(title(&b, "I1").as_deref(), Some("Buy oat milk"));
        let priority: i64 = a
            .conn()
            .query_row("SELECT priority FROM items WHERE id = 'I1'", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(priority, 2);
    }

    #[test]
    fn the_newer_edit_to_one_group_wins_everywhere() {
        let (mut a, mut b) = (replica(), replica());
        add_item(&a, "I1", "Buy milk");
        settle(&mut a, &mut b);
        exec(&a, "UPDATE items SET title = 'first' WHERE id = 'I1'");
        exec(&b, "UPDATE items SET title = 'second' WHERE id = 'I1'");
        settle(&mut a, &mut b);
        same(&a, &b);
        assert_eq!(title(&a, "I1").as_deref(), Some("second"));
    }

    #[test]
    fn coupled_columns_move_together() {
        let (mut a, mut b) = (replica(), replica());
        add_item(&a, "I1", "Meeting");
        exec(
            &a,
            "UPDATE items SET start_at = '2026-10-23T01:00:00Z', end_at = '2026-10-23T02:00:00Z' \
             WHERE id = 'I1'",
        );
        settle(&mut a, &mut b);
        // A moves the start later, B the end earlier: per column, end < start.
        exec(
            &a,
            "UPDATE items SET start_at = '2026-10-23T01:30:00Z', end_at = '2026-10-23T02:00:00Z' \
             WHERE id = 'I1'",
        );
        exec(
            &b,
            "UPDATE items SET start_at = '2026-10-23T01:00:00Z', end_at = '2026-10-23T01:15:00Z' \
             WHERE id = 'I1'",
        );
        settle(&mut a, &mut b);
        same(&a, &b);
        let (start, end): (String, String) = a
            .conn()
            .query_row(
                "SELECT start_at, end_at FROM items WHERE id = 'I1'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(
            (start.as_str(), end.as_str()),
            ("2026-10-23T01:00:00Z", "2026-10-23T01:15:00Z")
        );
    }

    #[test]
    fn a_delete_wins_over_a_concurrent_edit_either_way() {
        for deleter_first in [true, false] {
            let (mut a, mut b) = (replica(), replica());
            add_item(&a, "I1", "Buy milk");
            settle(&mut a, &mut b);
            if deleter_first {
                exec(&a, "DELETE FROM items WHERE id = 'I1'");
                exec(&b, "UPDATE items SET title = 'later edit' WHERE id = 'I1'");
            } else {
                exec(
                    &b,
                    "UPDATE items SET title = 'earlier edit' WHERE id = 'I1'",
                );
                exec(&a, "DELETE FROM items WHERE id = 'I1'");
            }
            settle(&mut a, &mut b);
            same(&a, &b);
            assert_eq!(title(&a, "I1"), None);
            assert_eq!(title(&b, "I1"), None);
        }
    }

    #[test]
    fn making_a_deleted_row_again_brings_it_back() {
        let (mut a, mut b) = (replica(), replica());
        add_item(&a, "I1", "Buy milk");
        settle(&mut a, &mut b);
        exec(&b, "DELETE FROM items WHERE id = 'I1'");
        settle(&mut a, &mut b);
        // An undo on B puts the row back as a new write.
        add_item(&b, "I1", "Buy milk");
        settle(&mut a, &mut b);
        same(&a, &b);
        assert_eq!(title(&a, "I1").as_deref(), Some("Buy milk"));
    }

    #[test]
    fn children_follow_a_parent_deleted_elsewhere() {
        let (mut a, mut b) = (replica(), replica());
        add_item(&a, "I1", "Dentist");
        settle(&mut a, &mut b);
        // A deletes the item while B adds a reminder to it.
        exec(&a, "DELETE FROM items WHERE id = 'I1'");
        b.conn()
            .execute(
                "INSERT INTO reminders (id, item_id, offset_min, created_at) VALUES ('R1', 'I1', 10, ?1)",
                [T0],
            )
            .unwrap();
        settle(&mut a, &mut b);
        same(&a, &b);
        assert_eq!(count(&a, "reminders"), 0);
        assert_eq!(count(&b, "reminders"), 0);
        let (_, gone) = capture::stamp(b.conn(), "reminders", "R1", ROW)
            .unwrap()
            .unwrap();
        assert!(gone);
    }

    #[test]
    fn a_unique_collision_keeps_the_row_made_first() {
        let (mut a, mut b) = (replica(), replica());
        a.conn()
            .execute(
                "INSERT INTO blockouts (id, label, weekdays, start_time, end_time, tz, created_at, \
                 updated_at) VALUES ('W', 'Work', 31, '09:00', '17:00', 'UTC', ?1, ?1)",
                [T0],
            )
            .unwrap();
        settle(&mut a, &mut b);
        // Both lift the same day offline.
        let lift = "INSERT INTO blockout_exceptions (id, blockout_id, date, created_at) \
                    VALUES (?1, 'W', '2026-10-30', ?2)";
        a.conn().execute(lift, ("XA", T0)).unwrap();
        b.conn().execute(lift, ("XB", T0)).unwrap();
        settle(&mut a, &mut b);
        same(&a, &b);
        let kept: Vec<String> = a
            .conn()
            .prepare("SELECT id FROM blockout_exceptions")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap();
        assert_eq!(kept, ["XA"], "A lifted it first");
    }

    #[test]
    fn a_newer_remote_edit_clears_a_pending_local_one() {
        let (mut a, mut b) = (replica(), replica());
        add_item(&a, "I1", "Buy milk");
        settle(&mut a, &mut b);
        exec(&a, "UPDATE items SET title = 'mine' WHERE id = 'I1'");
        exec(
            &b,
            "UPDATE items SET title = 'theirs, later' WHERE id = 'I1'",
        );
        send(&b, &mut a);
        assert_eq!(
            pending_count(a.conn()).unwrap(),
            0,
            "A's edit lost, nothing to push"
        );
        assert_eq!(title(&a, "I1").as_deref(), Some("theirs, later"));
    }

    #[test]
    fn merging_twice_changes_nothing() {
        let (a, mut b) = (replica(), replica());
        add_item(&a, "I1", "Buy milk");
        let (rows, _) = outbox(a.conn()).unwrap();
        merge(b.conn_mut(), &rows, Utc::now()).unwrap();
        let again = merge(b.conn_mut(), &rows, Utc::now()).unwrap();
        assert!(again.changed.is_empty());
        assert!(again.rejected.is_empty());
    }

    #[test]
    fn the_clock_moves_past_what_it_merged() {
        let (a, mut b) = (replica(), replica());
        hlc::tick(a.conn(), Utc::now() + chrono::Duration::hours(2)).unwrap();
        add_item(&a, "I1", "From the future");
        let stamp = capture::stamp(a.conn(), "items", "I1", ROW)
            .unwrap()
            .unwrap()
            .0;
        send(&a, &mut b);
        assert!(hlc::tick(b.conn(), Utc::now()).unwrap() > stamp);
    }

    #[test]
    fn bad_rows_are_refused_one_by_one() {
        let (a, mut b) = (replica(), replica());
        add_item(&a, "GOOD", "Fine");
        let (mut rows, _) = outbox(a.conn()).unwrap();
        let good = rows[0].clone();

        let mut unknown = good.clone();
        unknown.table = "nope".into();
        let mut local = good.clone();
        local.table = "chat_threads".into();
        let mut wrong_columns = good.clone();
        wrong_columns.key = "W".into();
        wrong_columns
            .groups
            .get_mut("title")
            .unwrap()
            .values
            .insert("extra".into(), json!(1));
        let mut broken = good.clone();
        broken.key = "B".into();
        let timing = broken.groups.get_mut("timing").unwrap();
        timing
            .values
            .insert("start_at".into(), json!("2026-10-23T02:00:00Z"));
        timing
            .values
            .insert("end_at".into(), json!("2026-10-23T01:00:00Z"));
        let mut setting = good.clone();
        setting.table = "settings".into();
        setting.key = "theme".into();
        setting.groups = BTreeMap::new();

        rows.extend([unknown, local, wrong_columns, broken, setting]);
        let merged = merge(b.conn_mut(), &rows, Utc::now()).unwrap();
        let refused: Vec<(&str, &str)> = merged
            .rejected
            .iter()
            .map(|r| (r.table.as_str(), r.key.as_str()))
            .collect();
        assert_eq!(
            refused,
            [
                ("nope", "GOOD"),
                ("chat_threads", "GOOD"),
                ("items", "W"),
                ("items", "B"),
                ("settings", "theme")
            ]
        );
        assert_eq!(title(&b, "GOOD").as_deref(), Some("Fine"));
        assert_eq!(count(&b, "items"), 1);
    }

    #[test]
    fn the_server_ignores_server_owned_data_from_devices() {
        let device = replica();
        let mut server = replica();
        capture::set_role(server.conn_mut(), Role::Server).unwrap();
        device
            .conn()
            .execute(
                "INSERT INTO ics_subscriptions (id, name, url, created_at, updated_at) \
                 VALUES ('S1', 'Work', 'https://example.com/a.ics', ?1, ?1)",
                [T0],
            )
            .unwrap();
        let (mut rows, _) = outbox(device.conn()).unwrap();
        // A misbehaving device sends fetch status too.
        let hlc = rows[0].row.clone();
        rows[0].groups.insert(
            "fetch_status".into(),
            GroupState {
                hlc,
                values: json!({ "last_fetch_at": T0, "last_ok_at": T0, "last_status": "ok", "last_error": null })
                    .as_object()
                    .unwrap()
                    .clone(),
            },
        );
        let merged = merge(server.conn_mut(), &rows, Utc::now()).unwrap();
        assert!(merged.rejected.is_empty());
        let status: Option<String> = server
            .conn()
            .query_row(
                "SELECT last_status FROM ics_subscriptions WHERE id = 'S1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(status, None);
    }

    #[test]
    fn devices_take_server_owned_data_from_the_server() {
        let mut server = replica();
        capture::set_role(server.conn_mut(), Role::Server).unwrap();
        let mut device = replica();
        server
            .conn()
            .execute(
                "INSERT INTO ics_subscriptions (id, name, url, created_at, updated_at, last_status) \
                 VALUES ('S1', 'Work', 'https://example.com/a.ics', ?1, ?1, 'ok')",
                [T0],
            )
            .unwrap();
        server
            .conn()
            .execute(
                "INSERT INTO ics_events (id, subscription_id, uid, title) VALUES ('E1', 'S1', 'u', 'Standup')",
                [],
            )
            .unwrap();
        let merged = send(&server, &mut device);
        assert!(merged.rejected.is_empty(), "{:?}", merged.rejected);
        assert_eq!(count(&device, "ics_events"), 1);
        let status: String = device
            .conn()
            .query_row(
                "SELECT last_status FROM ics_subscriptions WHERE id = 'S1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(status, "ok");
    }

    #[test]
    fn a_child_without_a_known_parent_is_refused() {
        let (a, mut b) = (replica(), replica());
        add_item(&a, "I1", "Dentist");
        a.conn()
            .execute(
                "INSERT INTO reminders (id, item_id, offset_min, created_at) VALUES ('R1', 'I1', 10, ?1)",
                [T0],
            )
            .unwrap();
        let (rows, _) = outbox(a.conn()).unwrap();
        let reminders: Vec<RowState> = rows
            .into_iter()
            .filter(|r| r.table == "reminders")
            .collect();
        let merged = merge(b.conn_mut(), &reminders, Utc::now()).unwrap();
        assert_eq!(merged.rejected.len(), 1);
        assert_eq!(count(&b, "reminders"), 0);
    }

    #[test]
    fn natural_keys_made_on_two_devices_merge_into_one() {
        let (mut a, mut b) = (replica(), replica());
        exec(
            &a,
            "INSERT INTO tags (name, color) VALUES ('home', '#111111')",
        );
        exec(
            &b,
            "INSERT INTO tags (name, color) VALUES ('home', '#222222')",
        );
        settle(&mut a, &mut b);
        same(&a, &b);
        assert_eq!(count(&a, "tags"), 1);
    }
}

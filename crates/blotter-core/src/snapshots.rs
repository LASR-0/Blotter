//! Snapshots (roadmap §4.9): full copies of the database, the coarse
//! safety net beside undo. Taken daily, before every restore and import,
//! before auto-applied AI batches (Phase 8), and by hand. Automatic ones
//! beyond the retention count are deleted; manual ones are kept.
//!
//! Each snapshot is a `VACUUM INTO` copy on disk; the `snapshots` table
//! holds the list. A restore replaces the whole database, that table
//! included, so the current list (and this install's device settings)
//! are carried across rather than taken from the restored copy.

use std::path::{Path, PathBuf};

use chrono_tz::Tz;
use rusqlite::{Connection, OptionalExtension, Row, named_params};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::db::Database;
use crate::error::{CoreError, Result};
use crate::id::new_id;
use crate::settings::{
    DEFAULT_SNAPSHOT_KEEP, DEVICE_KEYS, KEY_LAST_REMINDER_CHECK, KEY_SNAPSHOT_DIR,
    KEY_SNAPSHOT_KEEP,
};
use crate::store;
use crate::time::{self, Instant};

/// Why a snapshot was taken.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Reason {
    Manual,
    Daily,
    BeforeRestore,
    BeforeImport,
    BeforeAiBatch,
    BeforeIcsImport,
    /// Before this device took a home server's planner in place of its own:
    /// joining one, or the server's planner being restored or imported.
    BeforeSync,
}

impl Reason {
    fn as_str(self) -> &'static str {
        match self {
            Reason::Manual => "manual",
            Reason::Daily => "daily",
            Reason::BeforeRestore => "before_restore",
            Reason::BeforeImport => "before_import",
            Reason::BeforeAiBatch => "before_ai_batch",
            Reason::BeforeIcsImport => "before_ics_import",
            Reason::BeforeSync => "before_sync",
        }
    }

    fn parse(s: &str) -> Result<Self> {
        serde_json::from_value(Value::String(s.to_owned())).map_err(|_| CoreError::Corrupt {
            column: "snapshots.reason",
            value: s.to_owned(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Snapshot {
    pub id: String,
    pub reason: Reason,
    /// Manual snapshots are never pruned.
    pub manual: bool,
    pub note: Option<String>,
    pub path: String,
    pub size_bytes: Option<i64>,
    pub created_at: Instant,
}

const COLUMNS: &str = "id, reason, manual, note, path, size_bytes, created_at";

fn read(row: &Row<'_>) -> Result<Snapshot> {
    Ok(Snapshot {
        id: row.get("id")?,
        reason: Reason::parse(&row.get::<_, String>("reason")?)?,
        manual: row.get("manual")?,
        note: row.get("note")?,
        path: row.get("path")?,
        size_bytes: row.get("size_bytes")?,
        created_at: time::parse_instant("created_at", &row.get::<_, String>("created_at")?)?,
    })
}

fn insert(conn: &Connection, s: &Snapshot) -> Result<()> {
    conn.execute(
        &format!(
            "INSERT INTO snapshots ({COLUMNS}) VALUES \
             (:id, :reason, :manual, :note, :path, :size_bytes, :created_at)"
        ),
        named_params! {
            ":id": s.id,
            ":reason": s.reason.as_str(),
            ":manual": s.manual,
            ":note": s.note,
            ":path": s.path,
            ":size_bytes": s.size_bytes,
            ":created_at": time::fmt_instant(s.created_at),
        },
    )?;
    Ok(())
}

fn all_rows(conn: &Connection) -> Result<Vec<Snapshot>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {COLUMNS} FROM snapshots ORDER BY created_at DESC, rowid DESC"
    ))?;
    let mut rows = stmt.query([])?;
    let mut out = Vec::new();
    while let Some(row) = rows.next()? {
        out.push(read(row)?);
    }
    Ok(out)
}

/// Snapshots newest first. Rows whose file has gone (deleted by hand) are
/// dropped from the list.
pub fn list(conn: &Connection) -> Result<Vec<Snapshot>> {
    let (present, missing): (Vec<_>, Vec<_>) = all_rows(conn)?
        .into_iter()
        .partition(|s| Path::new(&s.path).is_file());
    for s in missing {
        conn.execute("DELETE FROM snapshots WHERE id = ?1", [&s.id])?;
    }
    Ok(present)
}

pub fn get(conn: &Connection, id: &str) -> Result<Snapshot> {
    conn.query_row(
        &format!("SELECT {COLUMNS} FROM snapshots WHERE id = ?1"),
        [id],
        |r| Ok(read(r)),
    )
    .optional()?
    .transpose()?
    .ok_or_else(|| CoreError::not_found("snapshot", id))
}

/// The configured folder, or `snapshots` beside the database file.
pub fn dir(conn: &Connection, db_path: &Path) -> Result<PathBuf> {
    Ok(
        match store::settings::get::<String>(conn, KEY_SNAPSHOT_DIR)? {
            Some(d) if !d.trim().is_empty() => PathBuf::from(d),
            _ => db_path
                .parent()
                .map_or_else(|| PathBuf::from("snapshots"), |p| p.join("snapshots")),
        },
    )
}

pub fn keep(conn: &Connection) -> Result<u32> {
    Ok(store::settings::get(conn, KEY_SNAPSHOT_KEEP)?.unwrap_or(DEFAULT_SNAPSHOT_KEEP))
}

/// Takes a snapshot into `dir` and prunes old automatic ones.
pub fn create(
    conn: &Connection,
    dir: &Path,
    reason: Reason,
    note: Option<String>,
    now: Instant,
) -> Result<Snapshot> {
    std::fs::create_dir_all(dir)?;
    let stamp = now.format("%Y%m%d-%H%M%S");
    let mut path = dir.join(format!("blotter-{stamp}-{}.db", reason.as_str()));
    if path.exists() {
        path = dir.join(format!(
            "blotter-{stamp}-{}-{}.db",
            reason.as_str(),
            new_id()
        ));
    }
    let path_str = path.to_string_lossy().into_owned();
    conn.execute("VACUUM INTO ?1", [&path_str])?;
    let snap = Snapshot {
        id: new_id(),
        reason,
        manual: reason == Reason::Manual,
        note: note.map(|n| n.trim().to_owned()).filter(|n| !n.is_empty()),
        size_bytes: std::fs::metadata(&path).ok().map(|m| m.len() as i64),
        path: path_str,
        created_at: time::truncate(now),
    };
    insert(conn, &snap)?;
    prune(conn, keep(conn)?)?;
    Ok(snap)
}

/// Deletes automatic snapshots beyond the newest `keep`.
pub fn prune(conn: &Connection, keep: u32) -> Result<Vec<Snapshot>> {
    let old: Vec<Snapshot> = all_rows(conn)?
        .into_iter()
        .filter(|s| !s.manual)
        .skip(keep as usize)
        .collect();
    for s in &old {
        let _ = std::fs::remove_file(&s.path);
        conn.execute("DELETE FROM snapshots WHERE id = ?1", [&s.id])?;
    }
    Ok(old)
}

/// Takes the daily snapshot if none has been taken today (local time).
pub fn daily_if_due(
    conn: &Connection,
    dir: &Path,
    tz: Tz,
    now: Instant,
) -> Result<Option<Snapshot>> {
    let today = now.with_timezone(&tz).date_naive();
    let taken_today = all_rows(conn)?.iter().any(|s| {
        s.reason == Reason::Daily && s.created_at.with_timezone(&tz).date_naive() == today
    });
    if taken_today {
        return Ok(None);
    }
    create(conn, dir, Reason::Daily, None, now).map(Some)
}

/// Device settings and the snapshot list, read before a restore or import
/// and written back after it.
struct Carried {
    snapshots: Vec<Snapshot>,
    device: Vec<(&'static str, Value)>,
}

fn carry(conn: &Connection) -> Result<Carried> {
    let mut device = Vec::new();
    for key in DEVICE_KEYS {
        if let Some(v) = store::settings::get::<Value>(conn, key)? {
            device.push((key, v));
        }
    }
    Ok(Carried {
        snapshots: all_rows(conn)?,
        device,
    })
}

fn put_back(conn: &Connection, c: &Carried, now: Instant) -> Result<()> {
    conn.execute("DELETE FROM snapshots", [])?;
    for s in &c.snapshots {
        insert(conn, s)?;
    }
    for key in DEVICE_KEYS {
        store::settings::delete(conn, key)?;
    }
    for (key, v) in &c.device {
        store::settings::set(conn, key, v, now)?;
    }
    // Reminders that came due between the copy and now belong to the
    // restored data's past; don't catch them all up at once.
    store::settings::set(conn, KEY_LAST_REMINDER_CHECK, &time::fmt_instant(now), now)?;
    Ok(())
}

/// Restores snapshot `id`, taking a "before restore" snapshot first so the
/// restore can itself be undone. Returns that new snapshot.
pub fn restore(db: &mut Database, db_path: &Path, id: &str, now: Instant) -> Result<Snapshot> {
    let target = get(db.conn(), id)?;
    let source = PathBuf::from(&target.path);
    if !source.is_file() {
        return Err(CoreError::invalid(format!(
            "the snapshot file is missing: {}",
            source.display()
        )));
    }
    let dir = dir(db.conn(), db_path)?;
    let before = create(db.conn(), &dir, Reason::BeforeRestore, None, now)?;
    let carried = carry(db.conn())?;
    db.restore_from(&source)?;
    put_back(db.conn(), &carried, now)?;
    Ok(before)
}

/// Runs `replace` (an import) with the snapshot list and device settings
/// kept, after taking a "before import" snapshot.
pub(crate) fn around_import<T>(
    db: &mut Database,
    db_path: &Path,
    now: Instant,
    replace: impl FnOnce(&mut Database) -> Result<T>,
) -> Result<(Snapshot, T)> {
    let dir = dir(db.conn(), db_path)?;
    let before = create(db.conn(), &dir, Reason::BeforeImport, None, now)?;
    let carried = carry(db.conn())?;
    let out = replace(db)?;
    put_back(db.conn(), &carried, now)?;
    Ok((before, out))
}

/// Moves every snapshot into `new_dir` (or the default folder, for `None`)
/// and makes it the folder new snapshots go to.
pub fn relocate(
    conn: &Connection,
    db_path: &Path,
    new_dir: Option<&Path>,
    now: Instant,
) -> Result<PathBuf> {
    match new_dir {
        Some(d) => store::settings::set(conn, KEY_SNAPSHOT_DIR, &d.to_string_lossy(), now)?,
        None => store::settings::delete(conn, KEY_SNAPSHOT_DIR)?,
    }
    let target = dir(conn, db_path)?;
    std::fs::create_dir_all(&target)?;
    for s in list(conn)? {
        let from = PathBuf::from(&s.path);
        let Some(name) = from.file_name() else {
            continue;
        };
        let to = target.join(name);
        if to == from {
            continue;
        }
        // Rename within a drive; copy and delete across drives.
        if std::fs::rename(&from, &to).is_err() {
            std::fs::copy(&from, &to)?;
            std::fs::remove_file(&from)?;
        }
        conn.execute(
            "UPDATE snapshots SET path = ?2 WHERE id = ?1",
            [&s.id, &to.to_string_lossy().into_owned()],
        )?;
    }
    Ok(target)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Actor;
    use chrono::{TimeZone, Utc};
    use serde_json::json;

    fn at(d: u32, h: u32) -> Instant {
        Utc.with_ymd_and_hms(2026, 10, d, h, 0, 0).unwrap()
    }

    fn temp_dir(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("blotter-test-{name}-{}", new_id()));
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    /// A file-backed planner (VACUUM INTO and restore need real files).
    fn planner(dir: &Path) -> (crate::Planner, PathBuf) {
        let path = dir.join("blotter.db");
        let db = Database::open(&path).unwrap();
        (
            crate::Planner::new(db, chrono_tz::Australia::Melbourne),
            path,
        )
    }

    fn titles(p: &crate::Planner) -> Vec<String> {
        let mut t: Vec<String> = p
            .db()
            .conn()
            .prepare("SELECT title FROM items")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        t.sort();
        t
    }

    #[test]
    fn snapshot_restore_round_trip_keeps_the_list() {
        let root = temp_dir("restore");
        let (mut p, db_path) = planner(&root);
        p.execute("create_item", json!({ "title": "Gym" }), &Actor::User)
            .unwrap();
        let dir = dir(p.db().conn(), &db_path).unwrap();
        assert_eq!(dir, root.join("snapshots"));
        let snap = create(
            p.db().conn(),
            &dir,
            Reason::Manual,
            Some(" before Q4 ".into()),
            at(15, 1),
        )
        .unwrap();
        assert_eq!(snap.note.as_deref(), Some("before Q4"));
        assert!(Path::new(&snap.path).is_file());

        p.execute("create_item", json!({ "title": "Later" }), &Actor::User)
            .unwrap();
        assert_eq!(titles(&p), ["Gym", "Later"]);

        let before = restore(p.db_mut(), &db_path, &snap.id, at(15, 2)).unwrap();
        assert_eq!(before.reason, Reason::BeforeRestore);
        assert_eq!(titles(&p), ["Gym"], "back to the snapshot");

        // Both snapshots are listed, though the restored copy only knew one.
        let listed = list(p.db().conn()).unwrap();
        assert_eq!(listed.len(), 2);
        assert_eq!(listed[0].id, before.id);

        // And the restore itself can be undone.
        restore(p.db_mut(), &db_path, &before.id, at(15, 3)).unwrap();
        assert_eq!(titles(&p), ["Gym", "Later"]);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn prunes_automatic_snapshots_but_keeps_manual() {
        let root = temp_dir("prune");
        let (p, db_path) = planner(&root);
        let conn = p.db().conn();
        let dir = dir(conn, &db_path).unwrap();
        store::settings::set(conn, KEY_SNAPSHOT_KEEP, &2u32, at(1, 0)).unwrap();
        create(conn, &dir, Reason::Manual, None, at(10, 0)).unwrap();
        for d in 11..15 {
            create(conn, &dir, Reason::Daily, None, at(d, 0)).unwrap();
        }
        let listed = list(conn).unwrap();
        let reasons: Vec<_> = listed.iter().map(|s| (s.reason, s.created_at)).collect();
        assert_eq!(
            reasons,
            [
                (Reason::Daily, at(14, 0)),
                (Reason::Daily, at(13, 0)),
                (Reason::Manual, at(10, 0))
            ]
        );
        let files = std::fs::read_dir(&dir).unwrap().count();
        assert_eq!(files, 3, "pruned files are deleted too");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn daily_once_per_local_day_and_relocation() {
        let root = temp_dir("daily");
        let (p, db_path) = planner(&root);
        let conn = p.db().conn();
        let dir = dir(conn, &db_path).unwrap();
        let tz = chrono_tz::Australia::Melbourne;
        // 22:00Z on the 14th is 09:00 on the 15th in Melbourne.
        assert!(daily_if_due(conn, &dir, tz, at(14, 22)).unwrap().is_some());
        assert!(
            daily_if_due(conn, &dir, tz, at(15, 2)).unwrap().is_none(),
            "same local day"
        );
        assert!(
            daily_if_due(conn, &dir, tz, at(15, 14)).unwrap().is_some(),
            "next local day"
        );

        let elsewhere = root.join("elsewhere");
        let moved = relocate(conn, &db_path, Some(&elsewhere), at(15, 15)).unwrap();
        assert_eq!(moved, elsewhere);
        for s in list(conn).unwrap() {
            assert!(
                s.path.starts_with(&*elsewhere.to_string_lossy()),
                "{}",
                s.path
            );
        }
        assert_eq!(
            std::fs::read_dir(&dir).unwrap().count(),
            0,
            "old folder emptied"
        );
        let _ = std::fs::remove_dir_all(root);
    }
}

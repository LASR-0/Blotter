//! This install's identity for sync.
//!
//! Every database gets a device id (a ULID) the first time it is opened, and
//! keeps it for good: restoring a snapshot or importing JSON leaves it alone,
//! so the server always sees one device per install.

use rusqlite::{Connection, OptionalExtension};
use serde::Serialize;

use crate::error::{CoreError, Result};
use crate::id::new_id;
use crate::sync::hlc;
use crate::time::{self, Instant};

/// Longest device name, in characters.
pub const NAME_MAX: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Device {
    pub id: String,
    /// What the user calls this device ("Desktop", "Laptop"); none until set.
    pub name: Option<String>,
    pub created_at: Instant,
}

/// Gives the database a device id if it has none yet. Safe to call from two
/// connections at once: only the first insert lands.
pub(crate) fn ensure(conn: &Connection, now: Instant) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO sync_device (id, device_id, created_at) VALUES (1, ?1, ?2)",
        (new_id(), time::fmt_instant(now)),
    )?;
    Ok(())
}

/// Gives a copied database a new identity (for in-memory test copies).
pub(crate) fn renew(conn: &Connection, now: Instant) -> Result<()> {
    conn.execute(
        "UPDATE sync_device SET device_id = ?1, name = NULL, created_at = ?2 WHERE id = 1",
        (new_id(), time::fmt_instant(now)),
    )?;
    Ok(())
}

pub fn get(conn: &Connection) -> Result<Device> {
    let (id, name, created_at): (String, Option<String>, String) = conn
        .query_row(
            "SELECT device_id, name, created_at FROM sync_device WHERE id = 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?
        .ok_or_else(|| CoreError::not_found("device", "this device"))?;
    Ok(Device {
        id,
        name,
        created_at: time::parse_instant("sync_device.created_at", &created_at)?,
    })
}

/// This device's id.
pub fn id(conn: &Connection) -> Result<String> {
    conn.query_row("SELECT device_id FROM sync_device WHERE id = 1", [], |r| {
        r.get(0)
    })
    .optional()?
    .ok_or_else(|| CoreError::not_found("device", "this device"))
}

/// Names this device. Surrounding whitespace is dropped; the name can't be
/// empty or longer than [`NAME_MAX`] characters.
pub fn set_name(conn: &Connection, name: &str) -> Result<Device> {
    let name = name.trim();
    if name.is_empty() {
        return Err(CoreError::invalid("the device name can't be empty"));
    }
    if name.chars().count() > NAME_MAX {
        return Err(CoreError::invalid(format!(
            "the device name can be at most {NAME_MAX} characters"
        )));
    }
    let changed = conn.execute("UPDATE sync_device SET name = ?1 WHERE id = 1", [name])?;
    if changed == 0 {
        return Err(CoreError::not_found("device", "this device"));
    }
    get(conn)
}

/// Replaces what this device knows of the others on its server: id → name.
pub fn set_peers(conn: &Connection, peers: &[(String, String)]) -> Result<()> {
    conn.execute("DELETE FROM sync_peers", [])?;
    let mut stmt =
        conn.prepare("INSERT OR REPLACE INTO sync_peers (device_id, name) VALUES (?1, ?2)")?;
    for (id, name) in peers {
        stmt.execute((id, name))?;
    }
    Ok(())
}

/// What a change's `origin` reads as: none for this device's own changes,
/// the device's name when known, "another device" otherwise.
pub fn origin_name(conn: &Connection, origin: Option<&str>) -> Result<Option<String>> {
    let Some(origin) = origin else {
        return Ok(None);
    };
    if origin == id(conn)? {
        return Ok(None);
    }
    let name: Option<String> = conn
        .query_row(
            "SELECT name FROM sync_peers WHERE device_id = ?1",
            [origin],
            |r| r.get(0),
        )
        .optional()?;
    Ok(Some(name.unwrap_or_else(|| "another device".to_owned())))
}

/// This device's identity and clock, held across a wholesale replace of the
/// database (snapshot restore), which would otherwise bring back an older
/// clock, another install's id, or none at all for a pre-1.2.1 snapshot.
#[derive(Debug)]
pub(crate) struct Kept {
    device: Device,
    clock: hlc::Clock,
}

pub(crate) fn keep(conn: &Connection) -> Result<Kept> {
    Ok(Kept {
        device: get(conn)?,
        clock: hlc::load(conn)?,
    })
}

/// Puts back what [`keep`] held. The clock keeps the later of the two, so it
/// never goes backwards.
pub(crate) fn put_back(conn: &Connection, kept: &Kept) -> Result<()> {
    conn.execute(
        "INSERT INTO sync_device (id, device_id, name, created_at) VALUES (1, ?1, ?2, ?3) \
         ON CONFLICT (id) DO UPDATE SET device_id = excluded.device_id, name = excluded.name, \
         created_at = excluded.created_at",
        (
            &kept.device.id,
            &kept.device.name,
            time::fmt_instant(kept.device.created_at),
        ),
    )?;
    let restored = hlc::load(conn)?;
    hlc::save(conn, restored.max(kept.clock))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Database;
    use chrono::Utc;

    #[test]
    fn a_new_database_has_a_device_id() {
        let db = Database::open_in_memory().unwrap();
        let device = get(db.conn()).unwrap();
        assert_eq!(device.id.len(), 26);
        assert_eq!(device.name, None);
        assert_eq!(id(db.conn()).unwrap(), device.id);
    }

    #[test]
    fn the_id_is_kept_across_opens_and_connections() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(crate::db::DB_FILE_NAME);
        let app = Database::open(&path).unwrap();
        let mcp = Database::open(&path).unwrap();
        let first = id(app.conn()).unwrap();
        assert_eq!(id(mcp.conn()).unwrap(), first);
        drop((app, mcp));
        let again = Database::open(&path).unwrap();
        assert_eq!(id(again.conn()).unwrap(), first);
    }

    #[test]
    fn two_databases_get_different_ids() {
        let a = Database::open_in_memory().unwrap();
        let b = Database::open_in_memory().unwrap();
        assert_ne!(id(a.conn()).unwrap(), id(b.conn()).unwrap());
    }

    #[test]
    fn ensure_never_replaces_an_existing_id() {
        let db = Database::open_in_memory().unwrap();
        let first = id(db.conn()).unwrap();
        ensure(db.conn(), Utc::now()).unwrap();
        assert_eq!(id(db.conn()).unwrap(), first);
    }

    fn far_future() -> Instant {
        Utc::now() + chrono::Duration::days(365)
    }

    #[test]
    fn restoring_another_installs_file_keeps_this_identity_and_clock() {
        let dir = tempfile::tempdir().unwrap();
        let other_path = dir.path().join("other.db");
        let other_id = {
            let other = Database::open(&other_path).unwrap();
            id(other.conn()).unwrap()
        };

        let mut db = Database::open(&dir.path().join("mine.db")).unwrap();
        set_name(db.conn(), "Desktop").unwrap();
        let mine = get(db.conn()).unwrap();
        let stamp = hlc::tick(db.conn(), far_future()).unwrap();

        db.restore_from(&other_path).unwrap();
        assert_ne!(mine.id, other_id);
        assert_eq!(get(db.conn()).unwrap(), mine);
        assert_eq!(hlc::current(db.conn()).unwrap(), stamp);
    }

    #[test]
    fn restoring_a_pre_sync_file_keeps_this_identity() {
        let dir = tempfile::tempdir().unwrap();
        let old_path = dir.path().join("1.2.0.db");
        {
            let old = Database::open(&old_path).unwrap();
            crate::sync::downgrade_to_1_2_0(old.conn());
        }

        let mut db = Database::open(&dir.path().join("mine.db")).unwrap();
        let mine = get(db.conn()).unwrap();
        let stamp = hlc::tick(db.conn(), far_future()).unwrap();

        db.restore_from(&old_path).unwrap();
        assert_eq!(db.schema_version().unwrap(), 14);
        assert_eq!(get(db.conn()).unwrap(), mine);
        assert!(hlc::tick(db.conn(), Utc::now()).unwrap() > stamp);
    }

    #[test]
    fn naming_trims_and_checks_length() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.conn();
        assert_eq!(
            set_name(conn, "  Laptop ").unwrap().name.as_deref(),
            Some("Laptop")
        );
        assert!(set_name(conn, "   ").is_err());
        assert!(set_name(conn, &"x".repeat(NAME_MAX + 1)).is_err());
        let longest = "é".repeat(NAME_MAX);
        assert_eq!(set_name(conn, &longest).unwrap().name, Some(longest));
    }
}

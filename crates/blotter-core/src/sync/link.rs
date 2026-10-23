//! The home server this device syncs with: its id, the addresses to try in
//! order, and how far this device has pulled (roadmap §14.6). Device-local,
//! and kept across a snapshot restore like the device's identity. No link
//! means Local only. The token lives in the OS keychain, never here.

use rusqlite::{Connection, OptionalExtension};

use crate::error::{CoreError, Result};
use crate::time::{self, Instant};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    pub server_id: String,
    /// Base addresses (`http://192.168.1.10:7458`), tried in this order.
    pub addresses: Vec<String>,
    /// The epoch last bootstrapped from.
    pub epoch: String,
    /// The last seq applied from that epoch.
    pub seq: i64,
    pub paired_at: Instant,
    pub last_sync_at: Option<Instant>,
}

/// The link, or `None` in Local only mode.
pub fn get(conn: &Connection) -> Result<Option<Link>> {
    let row = conn
        .query_row(
            "SELECT server_id, addresses, epoch, seq, paired_at, last_sync_at \
             FROM sync_server WHERE id = 1",
            [],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, i64>(3)?,
                    r.get::<_, String>(4)?,
                    r.get::<_, Option<String>>(5)?,
                ))
            },
        )
        .optional()?;
    let Some((server_id, addresses, epoch, seq, paired_at, last_sync_at)) = row else {
        return Ok(None);
    };
    let addresses = serde_json::from_str(&addresses).map_err(|_| CoreError::Corrupt {
        column: "sync_server.addresses",
        value: addresses.clone(),
    })?;
    Ok(Some(Link {
        server_id,
        addresses,
        epoch,
        seq,
        paired_at: time::parse_instant("sync_server.paired_at", &paired_at)?,
        last_sync_at: last_sync_at
            .as_deref()
            .map(|s| time::parse_instant("sync_server.last_sync_at", s))
            .transpose()?,
    }))
}

/// Pairs this device with a server, replacing any earlier link.
pub fn set(conn: &Connection, link: &Link) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO sync_server \
         (id, server_id, addresses, epoch, seq, paired_at, last_sync_at) \
         VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6)",
        (
            &link.server_id,
            serde_json::to_string(&link.addresses)?,
            &link.epoch,
            link.seq,
            time::fmt_instant(link.paired_at),
            link.last_sync_at.map(time::fmt_instant),
        ),
    )?;
    Ok(())
}

/// Back to Local only.
pub fn clear(conn: &Connection) -> Result<()> {
    conn.execute("DELETE FROM sync_server", [])?;
    Ok(())
}

/// Records a sync: the epoch and seq now applied, and when.
pub fn record_sync(conn: &Connection, epoch: &str, seq: i64, now: Instant) -> Result<()> {
    conn.execute(
        "UPDATE sync_server SET epoch = ?1, seq = ?2, last_sync_at = ?3 WHERE id = 1",
        (epoch, seq, time::fmt_instant(now)),
    )?;
    Ok(())
}

/// Replaces the address list (Settings → Sync).
pub fn set_addresses(conn: &Connection, addresses: &[String]) -> Result<()> {
    conn.execute(
        "UPDATE sync_server SET addresses = ?1 WHERE id = 1",
        [serde_json::to_string(addresses)?],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Database;
    use chrono::{TimeZone, Utc};

    fn link() -> Link {
        Link {
            server_id: "01JBSERVER0000000000000000".into(),
            addresses: vec![
                "http://192.168.1.10:7458".into(),
                "https://blotter.tail1234.ts.net".into(),
            ],
            epoch: "01JBEPOCH00000000000000000".into(),
            seq: 0,
            paired_at: Utc.with_ymd_and_hms(2026, 10, 23, 9, 0, 0).unwrap(),
            last_sync_at: None,
        }
    }

    #[test]
    fn local_only_until_paired_and_after_clearing() {
        let db = Database::open_in_memory().unwrap();
        assert_eq!(get(db.conn()).unwrap(), None);
        set(db.conn(), &link()).unwrap();
        assert_eq!(get(db.conn()).unwrap(), Some(link()));

        let now = Utc.with_ymd_and_hms(2026, 10, 23, 9, 5, 0).unwrap();
        record_sync(db.conn(), "01JBEPOCH00000000000000000", 42, now).unwrap();
        set_addresses(db.conn(), &["http://10.0.0.2:7458".into()]).unwrap();
        let got = get(db.conn()).unwrap().unwrap();
        assert_eq!((got.seq, got.last_sync_at), (42, Some(now)));
        assert_eq!(got.addresses, ["http://10.0.0.2:7458"]);

        clear(db.conn()).unwrap();
        assert_eq!(get(db.conn()).unwrap(), None);
    }

    #[test]
    fn a_snapshot_restore_keeps_the_link() {
        let dir = tempfile::tempdir().unwrap();
        let old_path = dir.path().join("old.db");
        // A snapshot from before pairing.
        drop(Database::open(&old_path).unwrap());

        let mut db = Database::open(&dir.path().join("mine.db")).unwrap();
        set(db.conn(), &link()).unwrap();
        record_sync(db.conn(), &link().epoch, 7, link().paired_at).unwrap();
        db.restore_from(&old_path).unwrap();
        assert_eq!(get(db.conn()).unwrap().unwrap().seq, 7);
    }
}

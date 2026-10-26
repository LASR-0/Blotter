//! Sync groundwork, shared with Blotter Sync (the home server).
//!
//! Everything a replica needs to take part in sync without any networking:
//! this device's identity, its hybrid logical clock, and the registry of
//! what syncs. Change capture, the merge function and the wire types join
//! them (roadmap §14), with the HTTP API's request and response types
//! (`api`). Networking stays out of the core: the app and the
//! server each bring their own.

pub mod api;
pub mod capture;
#[cfg(test)]
mod convergence;
pub mod device;
pub mod hlc;
pub mod join;
pub mod link;
pub mod merge;
pub mod registry;
pub mod state;
pub mod wire;

pub use device::Device;
pub use hlc::Hlc;
pub use merge::merge;
pub use wire::{GroupState, Merged, Rejection, RowState, SYNC_PROTOCOL};

/// Turns a database back into Blotter 1.2.0's shape (schema 11, nothing of
/// sync), as a pre-sync file to restore or upgrade from.
#[cfg(test)]
pub(crate) fn downgrade_to_1_2_0(conn: &rusqlite::Connection) {
    let triggers: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type = 'trigger' AND name LIKE 'sync_%'")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap();
    for t in triggers {
        conn.execute_batch(&format!("DROP TRIGGER \"{t}\";"))
            .unwrap();
    }
    conn.execute_batch(
        "DROP VIEW sync_now; DROP TABLE sync_clocks; DROP TABLE sync_state; \
         DROP TABLE sync_server; DROP TABLE sync_peers; ALTER TABLE changes DROP COLUMN origin; \
         DROP TABLE sync_device; DROP TABLE sync_clock; PRAGMA user_version = 11;",
    )
    .unwrap();
}

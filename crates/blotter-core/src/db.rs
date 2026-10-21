//! Opening the SQLite database, connection pragmas and schema migrations.

use std::path::Path;

use rusqlite::Connection;
use rusqlite_migration::{M, Migrations};

use crate::error::Result;

/// Ordered schema migrations. Append new ones; never edit a shipped one.
const MIGRATION_LIST: &[M<'static>] = &[
    M::up(include_str!("../migrations/0001_initial.sql")),
    M::up(include_str!("../migrations/0002_change_reverts.sql")),
    M::up(include_str!("../migrations/0003_snapshot_note.sql")),
    M::up(include_str!("../migrations/0004_chat_session.sql")),
    M::up(include_str!("../migrations/0005_proposal_ids.sql")),
    M::up(include_str!("../migrations/0006_item_ics_uid.sql")),
    M::up(include_str!("../migrations/0007_holidays.sql")),
    M::up(include_str!("../migrations/0008_item_windows.sql")),
    M::up(include_str!("../migrations/0009_tag_shares_time.sql")),
    M::up(include_str!("../migrations/0010_holidays_keep_sleep.sql")),
    M::up(include_str!("../migrations/0011_holiday_source.sql")),
];
const MIGRATIONS: Migrations<'static> = Migrations::from_slice(MIGRATION_LIST);

/// File name of the database inside the app-data directory.
pub const DB_FILE_NAME: &str = "blotter.db";

/// An open, migrated Blotter database.
#[derive(Debug)]
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Opens (creating if needed) the database at `path` in WAL mode and
    /// migrates it to the latest schema. Parent directories are created.
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let conn = Connection::open(path)?;
        // WAL lets the MCP server and the app share the file: readers never
        // block the writer and vice versa.
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        Self::init(conn)
    }

    /// A private in-memory database, for tests.
    pub fn open_in_memory() -> Result<Self> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(mut conn: Connection) -> Result<Self> {
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.busy_timeout(std::time::Duration::from_secs(5))?;
        MIGRATIONS.to_latest(&mut conn)?;
        Ok(Self { conn })
    }

    /// Replaces this database's contents with the database file at `path`
    /// (SQLite's backup API, so the open connection stays valid), then
    /// migrates it, in case it came from an older version of Blotter.
    pub fn restore_from(&mut self, path: &Path) -> Result<()> {
        self.conn.restore(
            rusqlite::MAIN_DB,
            path,
            None::<fn(rusqlite::backup::Progress)>,
        )?;
        self.conn.pragma_update(None, "foreign_keys", "ON")?;
        MIGRATIONS.to_latest(&mut self.conn)?;
        Ok(())
    }

    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    pub fn conn_mut(&mut self) -> &mut Connection {
        &mut self.conn
    }

    /// Current journal mode, e.g. `wal`.
    pub fn journal_mode(&self) -> Result<String> {
        Ok(self
            .conn
            .pragma_query_value(None, "journal_mode", |r| r.get(0))?)
    }

    /// Changes whenever another connection (e.g. the MCP server) commits.
    /// The app polls this to refresh after external writes.
    pub fn data_version(&self) -> Result<i64> {
        Ok(self
            .conn
            .pragma_query_value(None, "data_version", |r| r.get(0))?)
    }

    /// Schema version after migrations (SQLite `user_version`).
    pub fn schema_version(&self) -> Result<i64> {
        Ok(self
            .conn
            .pragma_query_value(None, "user_version", |r| r.get(0))?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_are_valid() {
        MIGRATIONS.validate().unwrap();
    }

    #[test]
    fn in_memory_database_is_migrated() {
        let db = Database::open_in_memory().unwrap();
        assert_eq!(db.schema_version().unwrap(), 11);
        let fk: i64 = db
            .conn()
            .pragma_query_value(None, "foreign_keys", |r| r.get(0))
            .unwrap();
        assert_eq!(fk, 1);
    }

    #[test]
    fn file_database_uses_wal_and_reopens() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested").join(DB_FILE_NAME);
        {
            let db = Database::open(&path).unwrap();
            assert_eq!(db.journal_mode().unwrap(), "wal");
        }
        let db = Database::open(&path).unwrap();
        assert_eq!(db.schema_version().unwrap(), 11);
    }

    #[test]
    fn data_version_moves_on_external_commit() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(DB_FILE_NAME);
        let app = Database::open(&path).unwrap();
        let mcp = Database::open(&path).unwrap();
        let before = app.data_version().unwrap();
        mcp.conn()
            .execute(
                "INSERT INTO tags (name, color) VALUES ('work', '#6c9cf2')",
                [],
            )
            .unwrap();
        assert_ne!(app.data_version().unwrap(), before);
    }
}

use rusqlite::{Connection, OptionalExtension};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::error::{CoreError, Result};
use crate::time::{self, Instant};

/// Reads a setting, or `None` if it has never been written.
pub fn get<T: DeserializeOwned>(conn: &Connection, key: &str) -> Result<Option<T>> {
    let raw: Option<String> = conn
        .query_row(
            "SELECT value_json FROM settings WHERE key = ?1",
            [key],
            |r| r.get(0),
        )
        .optional()?;
    raw.map(|s| {
        serde_json::from_str(&s).map_err(|_| CoreError::Corrupt {
            column: "settings.value_json",
            value: s,
        })
    })
    .transpose()
}

pub fn set<T: Serialize>(conn: &Connection, key: &str, value: &T, now: Instant) -> Result<()> {
    conn.execute(
        "INSERT INTO settings (key, value_json, updated_at) VALUES (?1, ?2, ?3) \
         ON CONFLICT (key) DO UPDATE SET value_json = excluded.value_json, \
         updated_at = excluded.updated_at",
        (key, serde_json::to_string(value)?, time::fmt_instant(now)),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use chrono::{TimeZone, Utc};

    #[test]
    fn settings_round_trip_and_overwrite() {
        let db = Database::open_in_memory().unwrap();
        let now = Utc.with_ymd_and_hms(2026, 10, 14, 9, 0, 0).unwrap();
        assert_eq!(get::<u32>(db.conn(), "default_duration_min").unwrap(), None);
        set(db.conn(), "default_duration_min", &30u32, now).unwrap();
        set(db.conn(), "default_duration_min", &45u32, now).unwrap();
        assert_eq!(
            get::<u32>(db.conn(), "default_duration_min").unwrap(),
            Some(45)
        );
        assert!(get::<String>(db.conn(), "default_duration_min").is_err());
    }
}

//! Full JSON export and import (roadmap §4.9): the planner's data in one
//! portable file. Import replaces everything, after a "before import"
//! snapshot (D26). The change log isn't exported, and is cleared on
//! import: its history describes the data being replaced, which the
//! snapshot keeps.

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::db::Database;
use crate::error::{CoreError, Result};
use crate::model::{Blockout, BlockoutException, Item, Reminder};
use crate::settings::DEVICE_KEYS;
use crate::snapshots::{self, Snapshot};
use crate::store;
use crate::time::Instant;

pub const FORMAT: &str = "blotter";
pub const VERSION: u32 = 1;

/// The export file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Export {
    /// Always "blotter", so other JSON is rejected early.
    pub format: String,
    pub version: u32,
    pub exported_at: Instant,
    pub items: Vec<Item>,
    pub reminders: Vec<Reminder>,
    pub blockouts: Vec<Blockout>,
    pub blockout_exceptions: Vec<BlockoutException>,
    /// User settings; this install's own (theme, snapshot folder…) are left out.
    pub settings: BTreeMap<String, Value>,
}

/// What an import file holds, shown before the user confirms.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Summary {
    pub exported_at: Instant,
    pub items: usize,
    pub reminders: usize,
    pub blockouts: usize,
    pub blockout_exceptions: usize,
    pub settings: usize,
}

pub fn export(conn: &rusqlite::Connection, now: Instant) -> Result<Export> {
    let mut settings = BTreeMap::new();
    let mut stmt = conn.prepare("SELECT key FROM settings ORDER BY key")?;
    let keys: Vec<String> = stmt
        .query_map([], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    for key in keys
        .into_iter()
        .filter(|k| !DEVICE_KEYS.contains(&k.as_str()))
    {
        if let Some(v) = store::settings::get::<Value>(conn, &key)? {
            settings.insert(key, v);
        }
    }
    Ok(Export {
        format: FORMAT.to_owned(),
        version: VERSION,
        exported_at: now,
        items: store::items::list_all(conn)?,
        reminders: store::reminders::list_all(conn)?,
        blockouts: store::blockouts::list(conn)?,
        blockout_exceptions: store::blockouts::list_exceptions(conn, None, None)?,
        settings,
    })
}

/// Reads an export file, refusing anything that isn't one this version
/// understands.
pub fn parse(json: &str) -> Result<Export> {
    let raw: Value = serde_json::from_str(json)
        .map_err(|e| CoreError::invalid(format!("that file isn't valid JSON ({e})")))?;
    if raw.get("format").and_then(Value::as_str) != Some(FORMAT) {
        return Err(CoreError::invalid("that file isn't a Blotter export"));
    }
    match raw.get("version").and_then(Value::as_u64) {
        Some(v) if v == u64::from(VERSION) => {}
        Some(v) => {
            return Err(CoreError::invalid(format!(
                "that export is version {v}; this Blotter reads version {VERSION}"
            )));
        }
        None => return Err(CoreError::invalid("that export has no version")),
    }
    serde_json::from_value(raw)
        .map_err(|e| CoreError::invalid(format!("that export is damaged: {e}")))
}

pub fn summarize(e: &Export) -> Summary {
    Summary {
        exported_at: e.exported_at,
        items: e.items.len(),
        reminders: e.reminders.len(),
        blockouts: e.blockouts.len(),
        blockout_exceptions: e.blockout_exceptions.len(),
        settings: e.settings.len(),
    }
}

fn replace_all(db: &mut Database, e: &Export) -> Result<()> {
    let tx = db.conn_mut().transaction()?;
    tx.pragma_update(None, "defer_foreign_keys", "ON")?;
    for table in [
        "reminders",
        "blockout_exceptions",
        "items",
        "blockouts",
        "changes",
    ] {
        tx.execute(&format!("DELETE FROM {table}"), [])?;
    }
    let keep: Vec<String> = DEVICE_KEYS.iter().map(|k| format!("'{k}'")).collect();
    tx.execute(
        &format!(
            "DELETE FROM settings WHERE key NOT IN ({})",
            keep.join(", ")
        ),
        [],
    )?;
    for b in &e.blockouts {
        store::blockouts::upsert(&tx, b)?;
    }
    for x in &e.blockout_exceptions {
        store::blockouts::upsert_exception(&tx, x)?;
    }
    for i in &e.items {
        store::items::upsert(&tx, i)?;
    }
    for r in &e.reminders {
        store::reminders::upsert(&tx, r)?;
    }
    for (key, v) in e
        .settings
        .iter()
        .filter(|(k, _)| !DEVICE_KEYS.contains(&k.as_str()))
    {
        store::settings::set(&tx, key, v, e.exported_at)?;
    }
    tx.commit()?;
    Ok(())
}

/// Replaces everything with `e`, after a "before import" snapshot, which
/// it returns.
pub fn import(db: &mut Database, db_path: &Path, e: &Export, now: Instant) -> Result<Snapshot> {
    let (before, ()) = snapshots::around_import(db, db_path, now, |db| replace_all(db, e))?;
    Ok(before)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::new_id;
    use crate::model::Actor;
    use chrono::{TimeZone, Utc};
    use serde_json::json;

    fn now() -> Instant {
        Utc.with_ymd_and_hms(2026, 10, 15, 6, 0, 0).unwrap()
    }

    fn file_planner(name: &str) -> (crate::Planner, std::path::PathBuf, std::path::PathBuf) {
        let root = std::env::temp_dir().join(format!("blotter-export-{name}-{}", new_id()));
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("blotter.db");
        let p = crate::Planner::new(
            Database::open(&path).unwrap(),
            chrono_tz::Australia::Melbourne,
        );
        (p, path, root)
    }

    fn seed(p: &mut crate::Planner) {
        let run = |p: &mut crate::Planner, op: &str, a: Value| {
            p.execute(op, a, &Actor::User).unwrap().result
        };
        run(
            p,
            "create_item",
            json!({ "title": "Inbox task", "tags": ["home"] }),
        );
        let s = run(
            p,
            "create_item",
            json!({ "title": "Standup", "kind": "event", "start": "2026-10-12T09:00",
                    "rrule": "FREQ=DAILY", "reminders": [5] }),
        );
        let sid = s["id"].as_str().unwrap().to_owned();
        run(
            p,
            "update_item",
            json!({ "id": format!("{sid}@2026-10-13T22:00:00Z"), "start": "2026-10-14T10:00" }),
        );
        let b = run(
            p,
            "create_blockout",
            json!({ "label": "Work", "weekdays": ["mon", "tue"], "start_time": "09:00",
                    "end_time": "17:00" }),
        );
        run(
            p,
            "add_blockout_exception",
            json!({ "blockout_id": b["id"], "date": "2026-10-20" }),
        );
        run(
            p,
            "update_settings",
            json!({ "snooze_options_min": [10, 30] }),
        );
    }

    #[test]
    fn export_reimports_to_an_identical_state() {
        let (mut a, _, root_a) = file_planner("a");
        seed(&mut a);
        store::settings::set(a.db().conn(), "theme", &"paper", now()).unwrap();
        let exported = export(a.db().conn(), now()).unwrap();
        assert!(
            !exported.settings.contains_key("theme"),
            "device settings stay out"
        );
        let json = serde_json::to_string_pretty(&exported).unwrap();

        let (mut b, b_path, root_b) = file_planner("b");
        b.execute(
            "create_item",
            json!({ "title": "Will be replaced" }),
            &Actor::User,
        )
        .unwrap();
        let parsed = parse(&json).unwrap();
        assert_eq!(summarize(&parsed).items, 3, "task, series, override");
        let before = import(b.db_mut(), &b_path, &parsed, now()).unwrap();
        assert_eq!(before.reason, snapshots::Reason::BeforeImport);

        let again = export(b.db().conn(), now()).unwrap();
        assert_eq!(again, exported, "identical after a round trip");
        let changes: i64 = b
            .db()
            .conn()
            .query_row("SELECT count(*) FROM changes", [], |r| r.get(0))
            .unwrap();
        assert_eq!(changes, 0, "the old history went with the old data");
        assert_eq!(snapshots::list(b.db().conn()).unwrap().len(), 1);
        let _ = std::fs::remove_dir_all(root_a);
        let _ = std::fs::remove_dir_all(root_b);
    }

    #[test]
    fn rejects_other_files() {
        assert!(parse("not json").is_err());
        assert!(
            parse(r#"{ "hello": 1 }"#)
                .unwrap_err()
                .to_string()
                .contains("isn't a Blotter")
        );
        let future = r#"{ "format": "blotter", "version": 99 }"#;
        assert!(
            parse(future)
                .unwrap_err()
                .to_string()
                .contains("version 99")
        );
    }
}

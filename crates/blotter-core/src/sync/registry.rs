//! The sync registry: what every table, column and setting is to sync.
//!
//! - **Synced** data is shared by every replica and merged last-writer-wins
//!   per field group.
//! - **Device-local** data never leaves this install.
//! - **Server-owned** data is written only by Blotter Sync (subscription
//!   fetches, rollover bookkeeping) and flows from the server to devices.
//!
//! A field group is a set of columns that always merge together, because a
//! CHECK constraint or an operation's invariant ties them (roadmap §14.5):
//! merging them one by one could build a row no replica ever wrote.
//!
//! Tests hold the registry to the migrated schema: a table, column, setting
//! or constraint it doesn't account for fails them, and any change to what
//! syncs must bump [`SYNC_SCHEMA`].

use serde::Serialize;

use crate::settings::*;

/// The version of the synced tables and columns. Every sync request carries
/// it, and Blotter Sync refuses a device whose version differs, so it moves
/// only when synced data changes shape; app releases that touch only
/// device-local data keep working with the same server. The tests pin each
/// version's fingerprint in `FINGERPRINTS`.
pub const SYNC_SCHEMA: u32 = 1;

/// Each sync schema version's fingerprint, oldest first. Version 1 is not
/// frozen until Blotter 1.2.1 ships; after that, a change appends a new
/// fingerprint and bumps [`SYNC_SCHEMA`].
#[cfg(test)]
const FINGERPRINTS: &[u64] = &[0x0b92_413b_bac7_6b53];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Class {
    Synced,
    DeviceLocal,
    ServerOwned,
}

impl Class {
    pub fn as_str(self) -> &'static str {
        match self {
            Class::Synced => "synced",
            Class::DeviceLocal => "device_local",
            Class::ServerOwned => "server_owned",
        }
    }
}

/// Columns that merge together.
#[derive(Debug, PartialEq, Eq)]
pub struct Group {
    pub name: &'static str,
    pub columns: &'static [&'static str],
    pub class: Class,
}

/// A column whose row goes when its parent row goes (`ON DELETE CASCADE`).
#[derive(Debug, PartialEq, Eq)]
pub struct Parent {
    pub column: &'static str,
    pub table: &'static str,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Table {
    pub name: &'static str,
    pub class: Class,
    /// The primary key: a ULID, or a natural key (tag name, setting key,
    /// keybinding action).
    pub key: &'static [&'static str],
    /// Every non-key column, in exactly one group. Empty for device-local tables.
    pub groups: &'static [Group],
    /// UNIQUE constraints besides the key, which merging must keep (§14.5).
    pub unique: &'static [&'static [&'static str]],
    pub parents: &'static [Parent],
}

impl Table {
    /// The group a column merges in, if the table syncs it.
    pub fn group_of(&self, column: &str) -> Option<&'static Group> {
        self.groups.iter().find(|g| g.columns.contains(&column))
    }
}

const fn synced(name: &'static str, columns: &'static [&'static str]) -> Group {
    Group {
        name,
        columns,
        class: Class::Synced,
    }
}

const fn local_table(name: &'static str, key: &'static [&'static str]) -> Table {
    Table {
        name,
        class: Class::DeviceLocal,
        key,
        groups: &[],
        unique: &[],
        parents: &[],
    }
}

/// Every table in the schema.
pub const TABLES: &[Table] = &[
    Table {
        name: "items",
        class: Class::Synced,
        key: &["id"],
        groups: &[
            synced(
                "timing",
                &[
                    "all_day",
                    "start_at",
                    "end_at",
                    "start_date",
                    "end_date",
                    "tz",
                    "duration_min",
                ],
            ),
            synced("status", &["status", "completed_at"]),
            synced("kind", &["kind", "anchoring"]),
            synced(
                "window",
                &["not_before", "due_at", "window_start", "window_end"],
            ),
            synced("series", &["parent_id", "recurrence_id", "rrule"]),
            synced("title", &["title"]),
            synced("notes", &["notes"]),
            synced("priority", &["priority"]),
            synced("tags", &["tags"]),
            synced("color", &["color"]),
            synced("sort_order", &["sort_order"]),
            synced("ics_uid", &["ics_uid"]),
            synced("source", &["source"]),
            synced("created_at", &["created_at"]),
            synced("updated_at", &["updated_at"]),
        ],
        unique: &[],
        parents: &[Parent {
            column: "parent_id",
            table: "items",
        }],
    },
    Table {
        name: "reminders",
        class: Class::Synced,
        key: &["id"],
        groups: &[
            synced("item_id", &["item_id"]),
            synced("trigger", &["offset_min", "at"]),
            synced("snoozed_until", &["snoozed_until"]),
            synced("created_at", &["created_at"]),
            // Every device fires its own reminders (D-S9).
            Group {
                name: "fired_at",
                columns: &["fired_at"],
                class: Class::DeviceLocal,
            },
        ],
        unique: &[],
        parents: &[Parent {
            column: "item_id",
            table: "items",
        }],
    },
    Table {
        name: "tags",
        class: Class::Synced,
        key: &["name"],
        groups: &[
            synced("color", &["color"]),
            synced("shares_time", &["shares_time"]),
        ],
        unique: &[],
        parents: &[],
    },
    Table {
        name: "blockouts",
        class: Class::Synced,
        key: &["id"],
        groups: &[
            synced("hours", &["start_time", "end_time", "tz"]),
            synced("active", &["active_from", "active_until"]),
            synced("label", &["label"]),
            synced("weekdays", &["weekdays"]),
            synced("permits_fixed", &["permits_fixed"]),
            synced("color", &["color"]),
            synced("icon", &["icon"]),
            synced("sort_order", &["sort_order"]),
            synced("created_at", &["created_at"]),
            synced("updated_at", &["updated_at"]),
        ],
        unique: &[],
        parents: &[],
    },
    Table {
        name: "blockout_exceptions",
        class: Class::Synced,
        key: &["id"],
        groups: &[
            // The UNIQUE pair moves as one.
            synced("placement", &["blockout_id", "date"]),
            synced("note", &["note"]),
            synced("holiday_id", &["holiday_id"]),
            synced("created_at", &["created_at"]),
        ],
        unique: &[&["blockout_id", "date"]],
        parents: &[Parent {
            column: "blockout_id",
            table: "blockouts",
        }],
    },
    Table {
        name: "holidays",
        class: Class::Synced,
        key: &["id"],
        groups: &[
            synced("range", &["start_date", "end_date"]),
            synced("label", &["label"]),
            synced("color", &["color"]),
            synced("source", &["source"]),
            synced("created_at", &["created_at"]),
            synced("updated_at", &["updated_at"]),
        ],
        unique: &[],
        parents: &[],
    },
    Table {
        name: "themes",
        class: Class::Synced,
        key: &["id"],
        groups: &[
            synced("name", &["name"]),
            synced("tokens_json", &["tokens_json"]),
            synced("created_at", &["created_at"]),
            synced("updated_at", &["updated_at"]),
        ],
        unique: &[],
        parents: &[],
    },
    Table {
        name: "keybindings",
        class: Class::Synced,
        key: &["action_id"],
        groups: &[synced("binding", &["binding", "updated_at"])],
        unique: &[],
        parents: &[],
    },
    // Per key: see `setting_class`.
    Table {
        name: "settings",
        class: Class::Synced,
        key: &["key"],
        groups: &[synced("value", &["value_json", "updated_at"])],
        unique: &[],
        parents: &[],
    },
    Table {
        name: "ics_subscriptions",
        class: Class::Synced,
        key: &["id"],
        groups: &[
            synced("name", &["name"]),
            synced("url", &["url"]),
            synced("refresh_min", &["refresh_min"]),
            synced("color", &["color"]),
            synced("as_blockout", &["as_blockout"]),
            synced("enabled", &["enabled"]),
            synced("created_at", &["created_at"]),
            synced("updated_at", &["updated_at"]),
            // The server fetches; devices show its results.
            Group {
                name: "fetch_status",
                columns: &["last_fetch_at", "last_ok_at", "last_status", "last_error"],
                class: Class::ServerOwned,
            },
        ],
        unique: &[],
        parents: &[],
    },
    // The fetched cache: written by the server, never edited on a device.
    Table {
        name: "ics_events",
        class: Class::ServerOwned,
        key: &["id"],
        groups: &[Group {
            name: "event",
            columns: &[
                "subscription_id",
                "uid",
                "title",
                "location",
                "all_day",
                "start_at",
                "end_at",
                "start_date",
                "end_date",
                "busy",
            ],
            class: Class::ServerOwned,
        }],
        unique: &[],
        parents: &[Parent {
            column: "subscription_id",
            table: "ics_subscriptions",
        }],
    },
    local_table("changes", &["id"]),
    local_table("proposals", &["id"]),
    local_table("proposal_ops", &["id"]),
    local_table("chat_threads", &["id"]),
    local_table("chat_messages", &["id"]),
    local_table("providers", &["id"]),
    local_table("snapshots", &["id"]),
    local_table("sync_device", &["id"]),
    local_table("sync_clock", &["id"]),
    local_table("sync_state", &["id"]),
    local_table("sync_clocks", &["tbl", "row_key", "grp"]),
];

/// Settings shared by every device: how the planner itself behaves.
pub const SYNCED_SETTINGS: &[&str] = &[
    KEY_PLANNING_HOURS,
    KEY_WEEK_START,
    KEY_CLOCK,
    KEY_TIMEZONE,
    KEY_ROLLOVER_MODE,
    KEY_ROLLOVER_TIME,
    KEY_SLOT_BUFFER,
    KEY_DEFAULT_TASK_DURATION,
    KEY_DEFAULT_EVENT_DURATION,
    KEY_DEFAULT_EVENT_REMINDER,
    KEY_SNOOZE_OPTIONS,
    KEY_PUBLIC_HOLIDAYS,
];

/// Settings only Blotter Sync writes once a device is paired.
pub const SERVER_OWNED_SETTINGS: &[&str] = &[KEY_LAST_ROLLOVER];

/// Settings that stay on this install: its theme, its AI and how it runs.
/// `ai_mode` is here, not synced: turning on auto-apply for one device's AI
/// shouldn't turn it on for every device's.
pub const DEVICE_LOCAL_SETTINGS: &[&str] = &[
    KEY_THEME,
    KEY_AI_MODE,
    KEY_AI_ROUTE,
    KEY_CLAUDE_CODE_MODEL,
    KEY_CLAUDE_CODE_PATH,
    KEY_CODEX_MODEL,
    KEY_CODEX_PATH,
    KEY_BRIEFING_ENABLED,
    KEY_BRIEFING_TIME,
    KEY_LAST_BRIEFING,
    KEY_CLOSE_TO_TRAY,
    KEY_NOTIFICATIONS,
    KEY_CATCH_UP,
    KEY_CATCH_UP_HOURS,
    KEY_SNAPSHOT_DIR,
    KEY_SNAPSHOT_KEEP,
    KEY_UPDATE_CHECK,
    KEY_SETUP_DONE,
    KEY_LAST_REMINDER_CHECK,
];

pub fn table(name: &str) -> Option<&'static Table> {
    TABLES.iter().find(|t| t.name == name)
}

/// How a setting syncs. A key the registry doesn't know (written by a newer
/// or older Blotter) stays on this device: never sync what isn't understood.
pub fn setting_class(key: &str) -> Class {
    if SYNCED_SETTINGS.contains(&key) {
        Class::Synced
    } else if SERVER_OWNED_SETTINGS.contains(&key) {
        Class::ServerOwned
    } else {
        Class::DeviceLocal
    }
}

/// A canonical description of everything that isn't device-local, hashed
/// with 64-bit FNV-1a: it changes exactly when the sync schema does.
#[cfg(test)]
fn fingerprint() -> u64 {
    let mut text = String::new();
    let mut tables: Vec<&Table> = TABLES
        .iter()
        .filter(|t| t.class != Class::DeviceLocal)
        .collect();
    tables.sort_by_key(|t| t.name);
    for t in tables {
        text += &format!(
            "table {} {} key {}\n",
            t.name,
            t.class.as_str(),
            t.key.join(",")
        );
        let mut groups: Vec<&Group> = t.groups.iter().collect();
        groups.sort_by_key(|g| g.name);
        for g in groups {
            text += &format!(
                "  group {} {} {}\n",
                g.name,
                g.class.as_str(),
                g.columns.join(",")
            );
        }
        for u in t.unique {
            text += &format!("  unique {}\n", u.join(","));
        }
        for p in t.parents {
            text += &format!("  parent {} {}\n", p.column, p.table);
        }
    }
    for (class, keys) in [
        (Class::Synced, SYNCED_SETTINGS),
        (Class::ServerOwned, SERVER_OWNED_SETTINGS),
    ] {
        let mut keys = keys.to_vec();
        keys.sort_unstable();
        text += &format!("settings {} {}\n", class.as_str(), keys.join(","));
    }
    text.bytes().fold(0xcbf2_9ce4_8422_2325, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use rusqlite::Connection;

    use super::*;
    use crate::Database;

    fn db() -> Database {
        Database::open_in_memory().unwrap()
    }

    fn schema_tables(conn: &Connection) -> BTreeMap<String, String> {
        let mut stmt = conn
            .prepare(
                "SELECT name, sql FROM sqlite_master WHERE type = 'table' \
                 AND name NOT LIKE 'sqlite_%'",
            )
            .unwrap();
        stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap()
    }

    /// Columns in declaration order, with their primary-key position (0 = not in the key).
    fn columns(conn: &Connection, table: &str) -> Vec<(String, i64)> {
        let mut stmt = conn
            .prepare(&format!(
                "SELECT name, pk FROM pragma_table_info('{table}')"
            ))
            .unwrap();
        stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap()
    }

    #[test]
    fn every_table_is_classified_once() {
        let db = db();
        let in_schema: BTreeSet<String> = schema_tables(db.conn()).into_keys().collect();
        let mut in_registry = BTreeSet::new();
        for t in TABLES {
            assert!(
                in_registry.insert(t.name.to_owned()),
                "{} listed twice",
                t.name
            );
        }
        assert_eq!(
            in_registry, in_schema,
            "the registry and the schema list different tables"
        );
    }

    #[test]
    fn keys_match_primary_keys() {
        let db = db();
        for t in TABLES {
            let mut pk: Vec<(i64, String)> = columns(db.conn(), t.name)
                .into_iter()
                .filter(|(_, pk)| *pk > 0)
                .map(|(name, pk)| (pk, name))
                .collect();
            pk.sort();
            let pk: Vec<String> = pk.into_iter().map(|(_, name)| name).collect();
            assert_eq!(pk, t.key, "{}'s key", t.name);
        }
    }

    #[test]
    fn every_column_of_a_shared_table_is_in_exactly_one_group() {
        let db = db();
        for t in TABLES.iter().filter(|t| t.class != Class::DeviceLocal) {
            let mut seen = BTreeSet::new();
            for g in t.groups {
                assert!(!g.columns.is_empty(), "{}.{} is empty", t.name, g.name);
                for c in g.columns {
                    assert!(seen.insert(*c), "{}.{c} is in two groups", t.name);
                    assert!(
                        !t.key.contains(c),
                        "{}.{c} is the key and in a group",
                        t.name
                    );
                }
            }
            for (column, pk) in columns(db.conn(), t.name) {
                if pk == 0 {
                    assert!(
                        seen.remove(column.as_str()),
                        "{}.{column} isn't classified: add it to a group in sync::registry \
                         (and bump SYNC_SCHEMA)",
                        t.name
                    );
                }
            }
            assert!(
                seen.is_empty(),
                "{} lists columns the table doesn't have: {seen:?}",
                t.name
            );
            let groups: BTreeSet<&str> = t.groups.iter().map(|g| g.name).collect();
            assert_eq!(
                groups.len(),
                t.groups.len(),
                "{} repeats a group name",
                t.name
            );
        }
    }

    #[test]
    fn device_local_tables_have_no_groups() {
        for t in TABLES.iter().filter(|t| t.class == Class::DeviceLocal) {
            assert!(t.groups.is_empty() && t.unique.is_empty() && t.parents.is_empty());
        }
    }

    #[test]
    fn server_owned_tables_hold_only_server_owned_groups() {
        for t in TABLES.iter().filter(|t| t.class == Class::ServerOwned) {
            assert!(
                t.groups.iter().all(|g| g.class == Class::ServerOwned),
                "{}",
                t.name
            );
        }
    }

    /// The text inside each `CHECK ( … )`, with string literals removed.
    fn checks(sql: &str) -> Vec<String> {
        let mut out = Vec::new();
        let bytes = sql.as_bytes();
        let mut from = 0;
        while let Some(at) = sql[from..].find("CHECK") {
            let start = from + at + "CHECK".len();
            let Some(open) = sql[start..].find('(').map(|i| start + i) else {
                break;
            };
            let (mut depth, mut end, mut quoted) = (0, open, false);
            for (i, &b) in bytes.iter().enumerate().skip(open) {
                match b {
                    b'\'' => quoted = !quoted,
                    b'(' if !quoted => depth += 1,
                    b')' if !quoted => {
                        depth -= 1;
                        if depth == 0 {
                            end = i;
                            break;
                        }
                    }
                    _ => {}
                }
            }
            let body = &sql[open + 1..end];
            let mut plain = String::new();
            let mut quoted = false;
            for ch in body.chars() {
                if ch == '\'' {
                    quoted = !quoted;
                } else if !quoted {
                    plain.push(ch);
                }
            }
            out.push(plain);
            from = end;
        }
        out
    }

    #[test]
    fn columns_tied_by_a_check_merge_together() {
        let db = db();
        let tables = schema_tables(db.conn());
        let mut tied = 0;
        for t in TABLES.iter().filter(|t| t.class != Class::DeviceLocal) {
            let names: BTreeSet<String> = columns(db.conn(), t.name)
                .into_iter()
                .map(|(n, _)| n)
                .collect();
            for check in checks(&tables[t.name]) {
                let used: BTreeSet<&str> = check
                    .split(|c: char| !(c.is_ascii_lowercase() || c == '_'))
                    .filter(|w| names.contains(*w))
                    .collect();
                if used.len() < 2 {
                    continue;
                }
                tied += 1;
                let groups: BTreeSet<&str> = used
                    .iter()
                    .map(|c| t.group_of(c).map_or("<key>", |g| g.name))
                    .collect();
                assert_eq!(
                    groups.len(),
                    1,
                    "{}: CHECK ({check}) ties {used:?} across groups {groups:?}",
                    t.name
                );
            }
        }
        // items (5), reminders, blockouts (2), holidays: the parser found them all.
        assert_eq!(tied, 9);
    }

    #[test]
    fn unique_constraints_are_listed() {
        let db = db();
        for t in TABLES.iter().filter(|t| t.class != Class::DeviceLocal) {
            let mut stmt = db
                .conn()
                .prepare(&format!(
                    "SELECT name FROM pragma_index_list('{}') WHERE origin = 'u'",
                    t.name
                ))
                .unwrap();
            let indexes: Vec<String> = stmt
                .query_map([], |r| r.get(0))
                .unwrap()
                .collect::<rusqlite::Result<_>>()
                .unwrap();
            let mut found: Vec<Vec<String>> = indexes
                .iter()
                .map(|idx| {
                    let mut stmt = db
                        .conn()
                        .prepare(&format!(
                            "SELECT name FROM pragma_index_info('{idx}') ORDER BY seqno"
                        ))
                        .unwrap();
                    stmt.query_map([], |r| r.get(0))
                        .unwrap()
                        .collect::<rusqlite::Result<_>>()
                        .unwrap()
                })
                .collect();
            found.sort();
            let mut listed: Vec<Vec<String>> = t
                .unique
                .iter()
                .map(|u| u.iter().map(|c| (*c).to_owned()).collect())
                .collect();
            listed.sort();
            assert_eq!(found, listed, "{}'s UNIQUE constraints", t.name);
            for u in t.unique {
                let groups: BTreeSet<_> = u.iter().map(|c| t.group_of(c).map(|g| g.name)).collect();
                assert_eq!(groups.len(), 1, "{}: UNIQUE {u:?} spans groups", t.name);
            }
        }
    }

    #[test]
    fn cascading_parents_are_listed() {
        let db = db();
        for t in TABLES.iter().filter(|t| t.class != Class::DeviceLocal) {
            let mut stmt = db
                .conn()
                .prepare(&format!(
                    "SELECT \"from\", \"table\", on_delete FROM pragma_foreign_key_list('{}')",
                    t.name
                ))
                .unwrap();
            let mut found: Vec<(String, String)> = stmt
                .query_map([], |r| {
                    let on_delete: String = r.get(2)?;
                    assert_eq!(
                        on_delete, "CASCADE",
                        "{}: only cascades are handled",
                        t.name
                    );
                    Ok((r.get(0)?, r.get(1)?))
                })
                .unwrap()
                .collect::<rusqlite::Result<_>>()
                .unwrap();
            found.sort();
            let mut listed: Vec<(String, String)> = t
                .parents
                .iter()
                .map(|p| (p.column.to_owned(), p.table.to_owned()))
                .collect();
            listed.sort();
            assert_eq!(found, listed, "{}'s parents", t.name);
        }
    }

    #[test]
    fn every_setting_key_is_classified_once() {
        // Every `pub const KEY_…: &str = "…"` in settings.rs.
        let source = include_str!("../settings.rs");
        let keys: Vec<&str> = source
            .lines()
            .filter_map(|l| l.trim().strip_prefix("pub const KEY_"))
            .filter_map(|l| l.split('"').nth(1))
            .collect();
        assert!(keys.len() >= 30, "found only {keys:?}");
        let lists = [
            SYNCED_SETTINGS,
            SERVER_OWNED_SETTINGS,
            DEVICE_LOCAL_SETTINGS,
        ];
        for key in &keys {
            let homes = lists.iter().filter(|l| l.contains(key)).count();
            assert_eq!(homes, 1, "setting {key} must be in exactly one class list");
        }
        let total: usize = lists.iter().map(|l| l.len()).sum();
        assert_eq!(
            total,
            keys.len(),
            "a class list names a key settings.rs doesn't define"
        );
    }

    #[test]
    fn device_keys_never_sync() {
        // Kept through restore and import, and not exported: never shared either.
        for key in DEVICE_KEYS {
            assert_ne!(setting_class(key), Class::Synced, "{key}");
        }
    }

    #[test]
    fn unknown_settings_stay_on_the_device() {
        assert_eq!(setting_class("from_a_newer_blotter"), Class::DeviceLocal);
        assert_eq!(setting_class(KEY_TIMEZONE), Class::Synced);
        assert_eq!(setting_class(KEY_LAST_ROLLOVER), Class::ServerOwned);
    }

    #[test]
    fn group_lookup() {
        let items = table("items").unwrap();
        assert_eq!(items.group_of("end_at").unwrap().name, "timing");
        assert_eq!(items.group_of("id"), None);
        let reminders = table("reminders").unwrap();
        assert_eq!(
            reminders.group_of("fired_at").unwrap().class,
            Class::DeviceLocal
        );
        assert!(table("nope").is_none());
    }

    #[test]
    fn the_sync_schema_version_is_pinned() {
        assert_eq!(
            FINGERPRINTS.len(),
            SYNC_SCHEMA as usize,
            "one fingerprint per sync schema version"
        );
        assert_eq!(
            fingerprint(),
            *FINGERPRINTS.last().unwrap(),
            "what syncs has changed (fingerprint {:#018x}). Once a Blotter release has shipped \
             this version, append the new fingerprint to FINGERPRINTS and bump SYNC_SCHEMA \
             (Blotter Sync then needs a release built against it); before that, replace the last \
             one.",
            fingerprint()
        );
    }
}

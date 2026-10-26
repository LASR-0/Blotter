//! Joining a server (roadmap §14.7): whether a planner is there, and the two
//! ways in when both sides have one. **Use the server's planner** replaces
//! everything synced here with a bootstrap; **Merge** unions ULID rows and
//! lets the server's values win for rows with a natural key, and drops this
//! device's copy of a public holiday the server already has. The caller
//! takes a snapshot first; device-local tables and settings are never
//! touched.

use rusqlite::{Connection, TransactionBehavior};

use crate::error::Result;
use crate::model::PUBLIC_SOURCE;
use crate::sync::registry::{self, Class, TABLES};
use crate::sync::{Hlc, Merged, RowState, capture, device, merge};
use crate::time::Instant;

/// Tables keyed by a name rather than a ULID, plus themes by id: a row the
/// server also has takes the server's values when merging.
const SERVER_WINS: &[&str] = &["tags", "settings", "keybindings", "themes"];

/// Whether a row is part of someone's planner: live, synced, and not a
/// server-owned setting (rollover bookkeeping) or fetched event.
fn counts(row: &RowState) -> bool {
    if row.deleted {
        return false;
    }
    match registry::table(&row.table).map(|t| t.class) {
        Some(Class::Synced) => {
            row.table != "settings" || registry::setting_class(&row.key) == Class::Synced
        }
        _ => false,
    }
}

/// Whether `rows` (a bootstrap, or [`super::state::all`] here) hold a
/// planner, rather than nothing or only what a server makes by itself.
pub fn has_planner(rows: &[RowState]) -> bool {
    rows.iter().any(counts)
}

/// Use the server's planner: every synced row here goes, and `rows` (the
/// server's bootstrap) take their place, in one transaction. Nothing is
/// left to push.
pub fn replace_with(conn: &mut Connection, rows: &[RowState], now: Instant) -> Result<Merged> {
    let mut tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    capture::applying(&tx, true)?;
    tx.pragma_update(None, "defer_foreign_keys", "ON")?;
    for table in TABLES.iter().filter(|t| t.class != Class::DeviceLocal) {
        if table.name == "settings" {
            let keys: Vec<String> = tx
                .prepare("SELECT key FROM settings")?
                .query_map([], |r| r.get(0))?
                .collect::<rusqlite::Result<_>>()?;
            for key in keys
                .iter()
                .filter(|k| registry::setting_class(k) != Class::DeviceLocal)
            {
                tx.execute("DELETE FROM settings WHERE key = ?1", [key])?;
            }
        } else {
            tx.execute(&format!("DELETE FROM \"{}\"", table.name), [])?;
        }
    }
    tx.execute("DELETE FROM sync_clocks", [])?;
    capture::applying(&tx, false)?;
    let merged = merge::merge_in(&mut tx, rows, now)?;
    tx.commit()?;
    Ok(merged)
}

fn text<'a>(row: &'a RowState, group: &str, column: &str) -> Option<&'a str> {
    row.groups.get(group)?.values.get(column)?.as_str()
}

/// Merge: this device's planner joins the server's. Everything here is
/// queued to push; rows the server also has under a natural key (and
/// themes by id) take the server's values; a public holiday the server has
/// for the same source and dates replaces this device's copy, which hands
/// it its block-out lifts. Then `rows` (the bootstrap) merge as usual.
pub fn merge_into(conn: &mut Connection, rows: &[RowState], now: Instant) -> Result<Merged> {
    capture::queue_all(conn)?;
    // Older than any stamp a replica writes, so the server's always wins.
    let oldest = Hlc {
        wall_ms: 0,
        counter: 0,
        device: device::id(conn)?,
    }
    .to_string();
    let mut tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    for row in rows
        .iter()
        .filter(|r| SERVER_WINS.contains(&r.table.as_str()))
    {
        tx.execute(
            "UPDATE sync_clocks SET hlc = ?1, pending = 0 WHERE tbl = ?2 AND row_key = ?3",
            (&oldest, &row.table, &row.key),
        )?;
    }
    for row in rows.iter().filter(|r| r.table == "holidays" && !r.deleted) {
        let (Some(source), Some(start), Some(end)) = (
            text(row, "source", "source"),
            text(row, "range", "start_date"),
            text(row, "range", "end_date"),
        ) else {
            continue;
        };
        if !source.starts_with(PUBLIC_SOURCE) {
            continue;
        }
        let copies: Vec<String> = tx
            .prepare(
                "SELECT id FROM holidays \
                 WHERE source = ?1 AND start_date = ?2 AND end_date = ?3 AND id <> ?4",
            )?
            .query_map((source, start, end, &row.key), |r| r.get(0))?
            .collect::<rusqlite::Result<_>>()?;
        for copy in copies {
            // Captured, so the server hears of both.
            tx.execute(
                "UPDATE blockout_exceptions SET holiday_id = ?1 WHERE holiday_id = ?2",
                (&row.key, &copy),
            )?;
            tx.execute("DELETE FROM holidays WHERE id = ?1", [&copy])?;
        }
    }
    let merged = merge::merge_in(&mut tx, rows, now)?;
    tx.commit()?;
    Ok(merged)
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use serde_json::json;

    use super::*;
    use crate::sync::capture::Role;
    use crate::sync::state;
    use crate::{Actor, Database, Planner, store};

    const T0: &str = "2026-10-26T00:00:00Z";

    fn planner() -> Planner {
        Planner::new(Database::open_in_memory().unwrap(), chrono_tz::UTC)
    }

    fn server() -> Planner {
        let mut p = planner();
        capture::set_role(p.db_mut().conn_mut(), Role::Server).unwrap();
        p
    }

    fn item(p: &mut Planner, title: &str) -> String {
        p.execute("create_item", json!({ "title": title }), &Actor::User)
            .unwrap()
            .result["id"]
            .as_str()
            .unwrap()
            .to_owned()
    }

    fn titles(p: &Planner) -> Vec<String> {
        let mut t: Vec<String> = p
            .db()
            .conn()
            .prepare("SELECT title FROM items")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap();
        t.sort();
        t
    }

    fn tag(p: &Planner, name: &str, color: &str) {
        p.db()
            .conn()
            .execute(
                "INSERT INTO tags (name, color) VALUES (?1, ?2) \
                 ON CONFLICT (name) DO UPDATE SET color = excluded.color",
                (name, color),
            )
            .unwrap();
    }

    fn color(p: &Planner, name: &str) -> String {
        p.db()
            .conn()
            .query_row("SELECT color FROM tags WHERE name = ?1", [name], |r| {
                r.get(0)
            })
            .unwrap()
    }

    fn holiday(p: &Planner, id: &str, source: &str) {
        p.db()
            .conn()
            .execute(
                "INSERT INTO holidays (id, label, start_date, end_date, source, created_at, \
                 updated_at) VALUES (?1, 'Melbourne Cup', '2026-11-03', '2026-11-03', ?2, ?3, ?3)",
                (id, source, T0),
            )
            .unwrap();
    }

    fn boot(p: &Planner) -> Vec<RowState> {
        state::all(p.db().conn()).unwrap()
    }

    #[test]
    fn a_planner_is_live_synced_rows() {
        let mut s = server();
        assert!(!has_planner(&boot(&s)), "a fresh server");
        store::settings::set(
            s.db().conn(),
            crate::settings::KEY_LAST_ROLLOVER,
            &"2026-10-25",
            Utc::now(),
        )
        .unwrap();
        assert!(!has_planner(&boot(&s)), "rollover bookkeeping isn't one");
        let id = item(&mut s, "Dentist");
        assert!(has_planner(&boot(&s)));
        s.execute("delete_item", json!({ "id": id }), &Actor::User)
            .unwrap();
        assert!(!has_planner(&boot(&s)), "tombstones aren't one");
        tag(&s, "home", "#fff");
        assert!(has_planner(&boot(&s)));
    }

    #[test]
    fn using_the_servers_planner_replaces_everything_synced() {
        let mut s = server();
        item(&mut s, "From the server");
        tag(&s, "home", "#00f");

        let mut d = planner();
        item(&mut d, "Only here");
        tag(&d, "home", "#f00");
        tag(&d, "garden", "#0f0");
        let conn = d.db().conn();
        store::settings::set(conn, "theme", &"paper", Utc::now()).unwrap();
        store::settings::set(conn, "week_start", &"sun", Utc::now()).unwrap();

        let merged = replace_with(d.db_mut().conn_mut(), &boot(&s), Utc::now()).unwrap();
        assert!(merged.rejected.is_empty(), "{:?}", merged.rejected);
        assert_eq!(titles(&d), ["From the server"]);
        assert_eq!(color(&d, "home"), "#00f");
        let conn = d.db().conn();
        let gone: i64 = conn
            .query_row("SELECT COUNT(*) FROM tags WHERE name = 'garden'", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(gone, 0);
        assert_eq!(
            store::settings::get::<String>(conn, "week_start").unwrap(),
            None
        );
        assert_eq!(
            store::settings::get::<String>(conn, "theme")
                .unwrap()
                .as_deref(),
            Some("paper"),
            "device-local settings stay"
        );
        assert_eq!(capture::pending_count(conn).unwrap(), 0);
        assert_eq!(boot(&d).len(), boot(&s).len());
    }

    #[test]
    fn merging_unions_items_and_the_server_wins_natural_keys() {
        let mut s = server();
        item(&mut s, "From the server");
        tag(&s, "home", "#00f");
        holiday(&s, "HSERVER", "public:AU-VIC");

        let mut d = planner();
        item(&mut d, "Only here");
        tag(&d, "home", "#f00");
        tag(&d, "garden", "#0f0");
        holiday(&d, "HDEVICE", "public:AU-VIC");
        holiday(&d, "HMINE", "manual");
        let blockout = d
            .execute(
                "create_blockout",
                json!({ "label": "Work", "weekdays": ["tue"], "start_time": "09:00",
                        "end_time": "17:00" }),
                &Actor::User,
            )
            .unwrap()
            .result["id"]
            .as_str()
            .unwrap()
            .to_owned();
        d.db()
            .conn()
            .execute(
                "INSERT INTO blockout_exceptions (id, blockout_id, date, created_at, holiday_id) \
                 VALUES ('X1', ?1, '2026-11-03', ?2, 'HDEVICE')",
                (&blockout, T0),
            )
            .unwrap();

        let merged = merge_into(d.db_mut().conn_mut(), &boot(&s), Utc::now()).unwrap();
        assert!(merged.rejected.is_empty(), "{:?}", merged.rejected);
        assert_eq!(titles(&d), ["From the server", "Only here"]);
        assert_eq!(color(&d, "home"), "#00f", "the server's value");
        assert_eq!(color(&d, "garden"), "#0f0");
        let conn = d.db().conn();
        let holidays: Vec<String> = conn
            .prepare("SELECT id FROM holidays ORDER BY id")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap();
        assert_eq!(holidays, ["HMINE", "HSERVER"]);
        let lift: String = conn
            .query_row(
                "SELECT holiday_id FROM blockout_exceptions WHERE id = 'X1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(lift, "HSERVER");

        // What goes up brings the server level with the device.
        let (rows, _) = state::outbox(conn).unwrap();
        merge(s.db_mut().conn_mut(), &rows, Utc::now()).unwrap();
        assert_eq!(titles(&s), ["From the server", "Only here"]);
        assert_eq!(color(&s, "home"), "#00f");
        assert_eq!(color(&s, "garden"), "#0f0");
        let on_server: i64 = s
            .db()
            .conn()
            .query_row("SELECT COUNT(*) FROM holidays", [], |r| r.get(0))
            .unwrap();
        assert_eq!(on_server, 2);
    }
}

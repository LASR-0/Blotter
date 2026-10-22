//! Convergence: replicas that make any edits, offline, and sync in any order
//! end up identical (roadmap §14.10).
//!
//! Three devices and a server hub run random interleavings of edits,
//! deletes, re-creations, reminders, block-out lifts, tags and settings,
//! syncing now and then with deliveries shuffled and repeated. After a final
//! round of syncs every replica holds the same rows, stamps and values.
//! Proptest keeps failing cases in `proptest-regressions/`, so they replay.

use chrono::Utc;
use proptest::prelude::*;
use rusqlite::Connection;

use crate::Database;
use crate::sync::capture::{self, Role};
use crate::sync::merge;
use crate::sync::state::{all, outbox};
use crate::sync::wire::RowState;

const DEVICES: usize = 3;
const ITEMS: usize = 4;
const REMINDERS: usize = 3;
const LIFTS: usize = 3;
const T0: &str = "2026-10-22T00:00:00Z";

#[derive(Debug, Clone)]
enum Op {
    CreateItem(usize),
    Retitle(usize, u8),
    Reschedule(usize, u8, u8),
    Complete(usize, bool),
    DeleteItem(usize),
    AddReminder(usize, usize),
    Snooze(usize, u8),
    DeleteReminder(usize),
    Lift(usize, u8),
    Unlift(usize),
    TagColor(u8, u8),
    DeleteTag(u8),
    WeekStart(u8),
}

#[derive(Debug, Clone)]
enum Step {
    Edit(usize, Op),
    /// Push, then pull with the batch shuffled (by the seed) and, when
    /// `repeat`, some earlier log entries delivered again.
    Sync {
        device: usize,
        seed: u64,
        repeat: bool,
    },
}

fn op() -> impl Strategy<Value = Op> {
    prop_oneof![
        3 => (0..ITEMS).prop_map(Op::CreateItem),
        3 => (0..ITEMS, any::<u8>()).prop_map(|(i, t)| Op::Retitle(i, t)),
        2 => (0..ITEMS, 0..48u8, 1..8u8).prop_map(|(i, s, d)| Op::Reschedule(i, s, d)),
        2 => (0..ITEMS, any::<bool>()).prop_map(|(i, d)| Op::Complete(i, d)),
        2 => (0..ITEMS).prop_map(Op::DeleteItem),
        2 => (0..REMINDERS, 0..ITEMS).prop_map(|(r, i)| Op::AddReminder(r, i)),
        1 => (0..REMINDERS, any::<u8>()).prop_map(|(r, m)| Op::Snooze(r, m)),
        1 => (0..REMINDERS).prop_map(Op::DeleteReminder),
        2 => (0..LIFTS, 0..2u8).prop_map(|(x, d)| Op::Lift(x, d)),
        1 => (0..LIFTS).prop_map(Op::Unlift),
        1 => (0..2u8, any::<u8>()).prop_map(|(t, c)| Op::TagColor(t, c)),
        1 => (0..2u8).prop_map(Op::DeleteTag),
        1 => (0..3u8).prop_map(Op::WeekStart),
    ]
}

fn step() -> impl Strategy<Value = Step> {
    prop_oneof![
        4 => (0..DEVICES, op()).prop_map(|(d, o)| Step::Edit(d, o)),
        1 => (0..DEVICES, any::<u64>(), any::<bool>())
            .prop_map(|(device, seed, repeat)| Step::Sync { device, seed, repeat }),
    ]
}

/// Runs one edit as the app would, ignoring the ones this replica's state
/// refuses (a reminder on an item it doesn't have, a second lift of a day).
fn edit(conn: &Connection, op: &Op) {
    let item = |i: &usize| format!("I{i}");
    let _ = match op {
        Op::CreateItem(i) => conn.execute(
            "INSERT OR IGNORE INTO items (id, kind, anchoring, title, created_at, updated_at) \
             VALUES (?1, 'task', 'flexible', 'New', ?2, ?2)",
            (item(i), T0),
        ),
        Op::Retitle(i, t) => conn.execute(
            "UPDATE items SET title = ?2 WHERE id = ?1",
            (item(i), format!("Title {t}")),
        ),
        Op::Reschedule(i, start, hours) => conn.execute(
            "UPDATE items SET start_at = ?2, end_at = ?3 WHERE id = ?1",
            (
                item(i),
                format!("2026-10-{:02}T{:02}:00:00Z", 23 + start / 24, start % 24),
                format!(
                    "2026-10-{:02}T{:02}:00:00Z",
                    23 + (start + hours) / 24,
                    (start + hours) % 24
                ),
            ),
        ),
        Op::Complete(i, done) => conn.execute(
            "UPDATE items SET status = ?2, completed_at = ?3 WHERE id = ?1",
            (
                item(i),
                if *done { "done" } else { "open" },
                done.then_some(T0),
            ),
        ),
        Op::DeleteItem(i) => conn.execute("DELETE FROM items WHERE id = ?1", [item(i)]),
        Op::AddReminder(r, i) => conn.execute(
            "INSERT OR IGNORE INTO reminders (id, item_id, offset_min, created_at) \
             VALUES (?1, ?2, 10, ?3)",
            (format!("R{r}"), item(i), T0),
        ),
        Op::Snooze(r, m) => conn.execute(
            "UPDATE reminders SET snoozed_until = ?2 WHERE id = ?1",
            (format!("R{r}"), format!("2026-10-22T01:{:02}:00Z", m % 60)),
        ),
        Op::DeleteReminder(r) => {
            conn.execute("DELETE FROM reminders WHERE id = ?1", [format!("R{r}")])
        }
        Op::Lift(x, day) => conn.execute(
            "INSERT OR IGNORE INTO blockout_exceptions (id, blockout_id, date, created_at) \
             VALUES (?1, 'W', ?2, ?3)",
            (format!("X{x}"), format!("2026-10-{:02}", 27 + day), T0),
        ),
        Op::Unlift(x) => conn.execute(
            "DELETE FROM blockout_exceptions WHERE id = ?1",
            [format!("X{x}")],
        ),
        Op::TagColor(t, c) => conn.execute(
            "INSERT INTO tags (name, color) VALUES (?1, ?2) \
             ON CONFLICT (name) DO UPDATE SET color = excluded.color",
            (format!("tag{t}"), format!("#{c:02x}{c:02x}{c:02x}")),
        ),
        Op::DeleteTag(t) => conn.execute("DELETE FROM tags WHERE name = ?1", [format!("tag{t}")]),
        Op::WeekStart(w) => conn.execute(
            "INSERT INTO settings (key, value_json, updated_at) VALUES ('week_start', ?1, ?2) \
             ON CONFLICT (key) DO UPDATE SET value_json = excluded.value_json",
            (
                format!("\"{}\"", ["mon", "sun", "sat"][usize::from(*w)]),
                T0,
            ),
        ),
    };
}

struct World {
    server: Database,
    /// Everything the server accepted, in order: what devices pull.
    log: Vec<RowState>,
    devices: Vec<Database>,
    cursors: Vec<usize>,
}

impl World {
    fn new() -> Self {
        let mut server = Database::open_in_memory().unwrap();
        capture::set_role(server.conn_mut(), Role::Server).unwrap();
        let mut world = Self {
            server,
            log: Vec::new(),
            devices: (0..DEVICES)
                .map(|_| Database::open_in_memory().unwrap())
                .collect(),
            cursors: vec![0; DEVICES],
        };
        // A shared block-out to lift days of.
        world.devices[0]
            .conn()
            .execute(
                "INSERT INTO blockouts (id, label, weekdays, start_time, end_time, tz, created_at, \
                 updated_at) VALUES ('W', 'Work', 31, '09:00', '17:00', 'UTC', ?1, ?1)",
                [T0],
            )
            .unwrap();
        for d in 0..DEVICES {
            world.sync(d, 0, false);
        }
        world
    }

    fn sync(&mut self, d: usize, seed: u64, repeat: bool) {
        let (rows, pending) = outbox(self.devices[d].conn()).unwrap();
        let merged = merge(self.server.conn_mut(), &rows, Utc::now()).unwrap();
        assert!(
            merged.rejected.is_empty(),
            "server refused {:?}",
            merged.rejected
        );
        self.log.extend(merged.changed);
        capture::acknowledge(self.devices[d].conn(), &pending).unwrap();

        let from = if repeat {
            self.cursors[d] / 2
        } else {
            self.cursors[d]
        };
        let mut batch = self.log[from..].to_vec();
        shuffle(&mut batch, seed);
        let merged = merge(self.devices[d].conn_mut(), &batch, Utc::now()).unwrap();
        assert!(
            merged.rejected.is_empty(),
            "device {d} refused {:?}",
            merged.rejected
        );
        self.cursors[d] = self.log.len();
    }

    /// Syncs every device until none has anything left to push and all have
    /// pulled the whole log. A tombstone one device works out in a merge
    /// reaches the others a round later.
    fn settle(&mut self) {
        for _ in 0..10 {
            for d in 0..DEVICES {
                self.sync(d, 0, false);
            }
            let quiet = self.devices.iter().zip(&self.cursors).all(|(db, c)| {
                capture::pending_count(db.conn()).unwrap() == 0 && *c == self.log.len()
            });
            if quiet {
                return;
            }
        }
        panic!("still syncing after 10 rounds");
    }
}

/// A small deterministic shuffle, so a seed replays the same order.
fn shuffle<T>(items: &mut [T], mut seed: u64) {
    for i in (1..items.len()).rev() {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        let j = usize::try_from(seed % (i as u64 + 1)).unwrap();
        items.swap(i, j);
    }
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 64, ..ProptestConfig::default() })]

    #[test]
    fn replicas_converge(steps in prop::collection::vec(step(), 1..40)) {
        let mut world = World::new();
        for s in &steps {
            match s {
                Step::Edit(d, op) => edit(world.devices[*d].conn(), op),
                Step::Sync { device, seed, repeat } => world.sync(*device, *seed, *repeat),
            }
        }
        world.settle();
        let server = all(world.server.conn()).unwrap();
        for (d, device) in world.devices.iter().enumerate() {
            let here = all(device.conn()).unwrap();
            prop_assert_eq!(&here, &server, "device {} differs from the server", d);
            prop_assert_eq!(capture::pending_count(device.conn()).unwrap(), 0);
        }
        // The data itself is valid: constraints held, and no orphans.
        for device in &world.devices {
            let orphans: i64 = device.conn().query_row(
                "SELECT count(*) FROM reminders r WHERE NOT EXISTS (SELECT 1 FROM items i WHERE i.id = r.item_id)",
                [], |r| r.get(0)).unwrap();
            prop_assert_eq!(orphans, 0);
            let fk: i64 = device.conn().query_row("SELECT count(*) FROM pragma_foreign_key_check", [], |r| r.get(0)).unwrap();
            prop_assert_eq!(fk, 0);
        }
    }
}

/// Found by the property test: a tombstone one device works out mid-merge
/// (a lift that collided before another moved away) must still reach a
/// device that had already synced that round.
#[test]
fn a_tombstone_worked_out_in_a_merge_reaches_everyone() {
    let mut world = World::new();
    world.sync(2, 4_460_857_184_196_525_934, true);
    edit(world.devices[2].conn(), &Op::Lift(0, 1));
    edit(world.devices[1].conn(), &Op::Lift(0, 0));
    edit(world.devices[2].conn(), &Op::Lift(1, 0));
    world.settle();
    let server = all(world.server.conn()).unwrap();
    for device in &world.devices {
        assert_eq!(all(device.conn()).unwrap(), server);
    }
}

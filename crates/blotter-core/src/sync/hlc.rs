//! Hybrid logical clocks.
//!
//! Every synced write is stamped with an [`Hlc`]: wall-clock milliseconds, a
//! counter and the device id. Stamps from one device never go backwards,
//! even if its wall clock does, and every stamp is unique across devices, so
//! last-writer-wins is a plain comparison on every replica.
//!
//! Stamps are stored and sent as fixed-width text
//! (`001792656000000.000000.<device ULID>`), so comparing the text gives
//! the same order as comparing the stamps, in Rust and in SQL alike.

use std::fmt;
use std::str::FromStr;

use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::error::{CoreError, Result};
use crate::sync::device;
use crate::time::Instant;

/// Digits of the wall-clock part (milliseconds since the Unix epoch).
const WALL_DIGITS: usize = 15;
/// Digits of the counter part.
const COUNTER_DIGITS: usize = 6;
/// The counter's ceiling. Past it the stamp moves to the next millisecond,
/// which keeps stamps unique and ordered without widening the text.
const COUNTER_MAX: u32 = 999_999;
const WALL_MAX: i64 = 999_999_999_999_999;

/// One hybrid logical clock stamp. Ordering is by wall time, then counter,
/// then device id, which breaks ties the same way on every replica.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Hlc {
    pub wall_ms: i64,
    pub counter: u32,
    pub device: String,
}

impl fmt::Display for Hlc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:0wall$}.{:0counter$}.{}",
            self.wall_ms,
            self.counter,
            self.device,
            wall = WALL_DIGITS,
            counter = COUNTER_DIGITS,
        )
    }
}

impl FromStr for Hlc {
    type Err = CoreError;

    fn from_str(s: &str) -> Result<Self> {
        let corrupt = || CoreError::Corrupt {
            column: "hlc",
            value: s.to_owned(),
        };
        let mut parts = s.splitn(3, '.');
        let (Some(wall), Some(counter), Some(device)) = (parts.next(), parts.next(), parts.next())
        else {
            return Err(corrupt());
        };
        let digits = |p: &str, n: usize| p.len() == n && p.bytes().all(|b| b.is_ascii_digit());
        if !digits(wall, WALL_DIGITS) || !digits(counter, COUNTER_DIGITS) || device.is_empty() {
            return Err(corrupt());
        }
        Ok(Self {
            wall_ms: wall.parse().map_err(|_| corrupt())?,
            counter: counter.parse().map_err(|_| corrupt())?,
            device: device.to_owned(),
        })
    }
}

impl Serialize for Hlc {
    fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        s.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Hlc {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

/// The clock's state: the wall time and counter of the last stamp this
/// device handed out or saw.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Clock {
    pub wall_ms: i64,
    pub counter: u32,
}

impl Clock {
    /// The clock after a local write at physical time `now_ms`.
    fn send(self, now_ms: i64) -> Self {
        if now_ms > self.wall_ms {
            Self {
                wall_ms: now_ms,
                counter: 0,
            }
        } else {
            self.bump()
        }
    }

    /// The clock after seeing `remote` at physical time `now_ms`.
    fn receive(self, remote: Self, now_ms: i64) -> Self {
        let wall_ms = self.wall_ms.max(remote.wall_ms).max(now_ms);
        let local = wall_ms == self.wall_ms;
        let theirs = wall_ms == remote.wall_ms;
        match (local, theirs) {
            (true, true) => Self {
                wall_ms,
                counter: self.counter.max(remote.counter),
            }
            .bump(),
            (true, false) => self.bump(),
            (false, true) => remote.bump(),
            (false, false) => Self {
                wall_ms,
                counter: 0,
            },
        }
    }

    fn bump(self) -> Self {
        if self.counter < COUNTER_MAX {
            Self {
                wall_ms: self.wall_ms,
                counter: self.counter + 1,
            }
        } else {
            Self {
                wall_ms: (self.wall_ms + 1).min(WALL_MAX),
                counter: 0,
            }
        }
    }
}

fn now_ms(now: Instant) -> i64 {
    now.timestamp_millis().clamp(0, WALL_MAX)
}

pub(crate) fn load(conn: &Connection) -> Result<Clock> {
    let row = conn
        .query_row(
            "SELECT wall_ms, counter FROM sync_clock WHERE id = 1",
            [],
            |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?)),
        )
        .optional()?;
    let (wall_ms, counter) = row.unwrap_or((0, 0));
    Ok(Clock {
        wall_ms,
        counter: u32::try_from(counter).map_err(|_| CoreError::Corrupt {
            column: "sync_clock.counter",
            value: counter.to_string(),
        })?,
    })
}

pub(crate) fn save(conn: &Connection, clock: Clock) -> Result<()> {
    conn.execute(
        "INSERT INTO sync_clock (id, wall_ms, counter) VALUES (1, ?1, ?2) \
         ON CONFLICT (id) DO UPDATE SET wall_ms = excluded.wall_ms, counter = excluded.counter",
        (clock.wall_ms, i64::from(clock.counter)),
    )?;
    Ok(())
}

/// The newest stamp this device has handed out or seen, without advancing.
pub fn current(conn: &Connection) -> Result<Hlc> {
    let clock = load(conn)?;
    Ok(Hlc {
        wall_ms: clock.wall_ms,
        counter: clock.counter,
        device: device::id(conn)?,
    })
}

/// A new stamp for a local write at `now`, later than every stamp this
/// device has handed out or seen. Run it in the same transaction as the
/// write, so a crash can't keep one without the other.
pub fn tick(conn: &Connection, now: Instant) -> Result<Hlc> {
    let clock = load(conn)?.send(now_ms(now));
    save(conn, clock)?;
    Ok(Hlc {
        wall_ms: clock.wall_ms,
        counter: clock.counter,
        device: device::id(conn)?,
    })
}

/// Moves the clock past a stamp received from another replica, so this
/// device's next write is ordered after it.
pub fn observe(conn: &Connection, remote: &Hlc, now: Instant) -> Result<()> {
    let theirs = Clock {
        wall_ms: remote.wall_ms.clamp(0, WALL_MAX),
        counter: remote.counter.min(COUNTER_MAX),
    };
    save(conn, load(conn)?.receive(theirs, now_ms(now)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Database;
    use chrono::{Duration, TimeZone, Utc};

    fn at(ms: i64) -> Instant {
        Utc.timestamp_millis_opt(ms).single().unwrap()
    }

    fn clock(wall_ms: i64, counter: u32) -> Clock {
        Clock { wall_ms, counter }
    }

    #[test]
    fn text_is_fixed_width_and_round_trips() {
        let h = Hlc {
            wall_ms: 1_792_656_000_123,
            counter: 7,
            device: "01JBDEVICE0000000000000000".into(),
        };
        let s = h.to_string();
        assert_eq!(s, "001792656000123.000007.01JBDEVICE0000000000000000");
        assert_eq!(s.parse::<Hlc>().unwrap(), h);
        let json = serde_json::to_string(&h).unwrap();
        assert_eq!(json, format!("\"{s}\""));
        assert_eq!(serde_json::from_str::<Hlc>(&json).unwrap(), h);
    }

    #[test]
    fn text_order_matches_stamp_order() {
        let mk = |wall_ms, counter, device: &str| Hlc {
            wall_ms,
            counter,
            device: device.into(),
        };
        let mut stamps = [
            mk(2, 0, "A"),
            mk(1, 999_999, "B"),
            mk(1, 10, "A"),
            mk(1, 9, "B"),
            mk(10, 0, "A"),
            mk(1, 10, "B"),
        ];
        let mut texts: Vec<String> = stamps.iter().map(ToString::to_string).collect();
        stamps.sort();
        texts.sort();
        let sorted: Vec<String> = stamps.iter().map(ToString::to_string).collect();
        assert_eq!(texts, sorted);
    }

    #[test]
    fn malformed_text_is_refused() {
        for bad in [
            "",
            "1.2.A",
            "001792656000123.000007",
            "001792656000123.000007.",
            "00179265600012x.000007.A",
            "001792656000123.0000070.A",
        ] {
            assert!(bad.parse::<Hlc>().is_err(), "{bad:?} parsed");
        }
    }

    #[test]
    fn send_follows_the_wall_clock_and_never_goes_back() {
        assert_eq!(clock(100, 3).send(200), clock(200, 0));
        // The wall clock stepped back: the counter carries order instead.
        assert_eq!(clock(200, 0).send(150), clock(200, 1));
        assert_eq!(clock(200, 1).send(200), clock(200, 2));
    }

    #[test]
    fn counter_overflow_moves_to_the_next_millisecond() {
        assert_eq!(clock(200, COUNTER_MAX).send(200), clock(201, 0));
        assert!(clock(201, 0) > clock(200, COUNTER_MAX));
    }

    #[test]
    fn receive_lands_after_both_clocks() {
        // Remote ahead of us and of the wall clock.
        assert_eq!(clock(100, 5).receive(clock(300, 2), 200), clock(300, 3));
        // We're ahead.
        assert_eq!(clock(300, 5).receive(clock(100, 9), 200), clock(300, 6));
        // Same wall time: past both counters.
        assert_eq!(clock(300, 5).receive(clock(300, 9), 200), clock(300, 10));
        // Wall clock ahead of both.
        assert_eq!(clock(100, 5).receive(clock(150, 9), 400), clock(400, 0));
    }

    #[test]
    fn ticks_are_strictly_increasing_and_carry_the_device() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.conn();
        let device = device::id(conn).unwrap();
        let t = at(1_792_656_000_000);
        let a = tick(conn, t).unwrap();
        let b = tick(conn, t).unwrap();
        let c = tick(conn, t - Duration::seconds(30)).unwrap();
        let d = tick(conn, t + Duration::milliseconds(1)).unwrap();
        assert!(a < b && b < c && c < d);
        assert_eq!(a.device, device);
        assert_eq!(current(conn).unwrap(), d);
    }

    #[test]
    fn observing_a_later_stamp_orders_the_next_tick_after_it() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.conn();
        let t = at(1_792_656_000_000);
        let remote = Hlc {
            wall_ms: 1_792_656_600_000,
            counter: 4,
            device: "01JBOTHERDEVICE00000000000".into(),
        };
        observe(conn, &remote, t).unwrap();
        let next = tick(conn, t).unwrap();
        assert!(next > remote);
        assert_eq!((next.wall_ms, next.counter), (remote.wall_ms, 6));
    }

    #[test]
    fn the_clock_survives_reopening() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(crate::db::DB_FILE_NAME);
        let first = {
            let db = Database::open(&path).unwrap();
            tick(db.conn(), at(1_792_656_000_000)).unwrap()
        };
        let db = Database::open(&path).unwrap();
        let next = tick(db.conn(), at(1_792_656_000_000)).unwrap();
        assert!(next > first);
        assert_eq!(next.device, first.device);
    }

    #[test]
    fn a_rolled_back_write_rolls_back_its_tick() {
        let mut db = Database::open_in_memory().unwrap();
        let t = at(1_792_656_000_000);
        let before = current(db.conn()).unwrap();
        {
            let tx = db.conn_mut().transaction().unwrap();
            tick(&tx, t).unwrap();
            // Dropped without commit.
        }
        assert_eq!(current(db.conn()).unwrap(), before);
    }
}

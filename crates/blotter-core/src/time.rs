//! Canonical text encodings for instants, dates and times of day.
//!
//! The database stores instants as RFC 3339 UTC with whole seconds and a
//! trailing `Z` (`2026-10-14T08:30:00Z`). A single fixed-width format means
//! SQL text comparison matches chronological order.

use chrono::{DateTime, NaiveDate, NaiveTime, SecondsFormat, Timelike, Utc};

use crate::error::{CoreError, Result};

pub type Instant = DateTime<Utc>;

/// Drops sub-second precision so stored and in-memory values compare equal.
pub fn truncate(t: Instant) -> Instant {
    t.with_nanosecond(0).unwrap_or(t)
}

pub fn fmt_instant(t: Instant) -> String {
    truncate(t).to_rfc3339_opts(SecondsFormat::Secs, true)
}

pub fn parse_instant(column: &'static str, s: &str) -> Result<Instant> {
    DateTime::parse_from_rfc3339(s)
        .map(|t| t.with_timezone(&Utc))
        .map_err(|_| CoreError::Corrupt {
            column,
            value: s.to_owned(),
        })
}

pub fn fmt_date(d: NaiveDate) -> String {
    d.format("%Y-%m-%d").to_string()
}

pub fn parse_date(column: &'static str, s: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").map_err(|_| CoreError::Corrupt {
        column,
        value: s.to_owned(),
    })
}

pub fn fmt_time(t: NaiveTime) -> String {
    t.format("%H:%M").to_string()
}

pub fn parse_time(column: &'static str, s: &str) -> Result<NaiveTime> {
    NaiveTime::parse_from_str(s, "%H:%M").map_err(|_| CoreError::Corrupt {
        column,
        value: s.to_owned(),
    })
}

/// Source of "now", injectable so tests are deterministic.
pub trait Clock: Send + Sync {
    fn now(&self) -> Instant;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Instant {
        truncate(Utc::now())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn instants_round_trip_at_second_precision() {
        let t = Utc.with_ymd_and_hms(2026, 10, 14, 8, 30, 0).unwrap()
            + chrono::Duration::milliseconds(750);
        let s = fmt_instant(t);
        assert_eq!(s, "2026-10-14T08:30:00Z");
        assert_eq!(parse_instant("x", &s).unwrap(), truncate(t));
    }

    #[test]
    fn instant_text_order_matches_time_order() {
        let a = fmt_instant(Utc.with_ymd_and_hms(2026, 1, 2, 9, 0, 0).unwrap());
        let b = fmt_instant(Utc.with_ymd_and_hms(2026, 1, 2, 10, 0, 0).unwrap());
        let c = fmt_instant(Utc.with_ymd_and_hms(2026, 11, 1, 0, 0, 0).unwrap());
        assert!(a < b && b < c);
    }

    #[test]
    fn parse_rejects_garbage() {
        assert!(matches!(
            parse_instant("start_at", "tuesday"),
            Err(CoreError::Corrupt {
                column: "start_at",
                ..
            })
        ));
        assert!(parse_time("t", "25:00").is_err());
        assert!(parse_date("d", "2026-13-01").is_err());
    }
}

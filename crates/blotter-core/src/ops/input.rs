//! Argument value types shared by operations, with JSON Schemas written for
//! humans and models alike.

use std::borrow::Cow;

use chrono::{DateTime, Duration, LocalResult, NaiveDate, NaiveDateTime, NaiveTime, TimeZone};
use chrono_tz::Tz;
use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use serde::{Deserialize, Deserializer};

use crate::time::{Instant, truncate};

/// A point in time given either with an offset (`2026-10-14T15:00:00+11:00`,
/// `…Z`) or as a local wall-clock time (`2026-10-14T15:00`) that is read in
/// the user's timezone.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DateTimeInput {
    Absolute(Instant),
    Local(NaiveDateTime),
}

impl DateTimeInput {
    pub fn resolve(self, tz: Tz) -> Instant {
        match self {
            DateTimeInput::Absolute(t) => truncate(t),
            DateTimeInput::Local(ndt) => resolve_local(tz, ndt),
        }
    }
}

/// Maps a wall-clock time in `tz` to an instant. In a DST overlap the
/// earlier instant wins; in a DST gap the time moves forward by the gap
/// (02:30 on a spring-forward night becomes 03:30).
pub fn resolve_local(tz: Tz, ndt: NaiveDateTime) -> Instant {
    match tz.from_local_datetime(&ndt) {
        LocalResult::Single(t) => t.to_utc(),
        LocalResult::Ambiguous(earliest, _) => earliest.to_utc(),
        LocalResult::None => {
            // Gaps are at most a few hours; step forward until valid.
            (1..=4)
                .find_map(|h| {
                    tz.from_local_datetime(&(ndt + Duration::hours(h)))
                        .earliest()
                })
                .map(|t| t.to_utc())
                .unwrap_or_else(|| ndt.and_utc())
        }
    }
}

/// Start of `date` in `tz`, as an instant.
pub fn start_of_day(tz: Tz, date: NaiveDate) -> Instant {
    resolve_local(tz, date.and_time(NaiveTime::MIN))
}

fn parse_datetime(s: &str) -> Option<DateTimeInput> {
    let s = s.trim();
    if let Ok(t) = DateTime::parse_from_rfc3339(s) {
        return Some(DateTimeInput::Absolute(t.to_utc()));
    }
    let normalized = s.replacen(' ', "T", 1);
    ["%Y-%m-%dT%H:%M:%S", "%Y-%m-%dT%H:%M"]
        .iter()
        .find_map(|f| NaiveDateTime::parse_from_str(&normalized, f).ok())
        .map(DateTimeInput::Local)
}

impl<'de> Deserialize<'de> for DateTimeInput {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        parse_datetime(&s).ok_or_else(|| {
            serde::de::Error::custom(format!(
                "'{s}' is not a date-time; use 2026-10-14T15:00 (local) or RFC 3339 with offset"
            ))
        })
    }
}

impl JsonSchema for DateTimeInput {
    fn schema_name() -> Cow<'static, str> {
        "DateTime".into()
    }

    fn inline_schema() -> bool {
        true
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "type": "string",
            "pattern": r"^\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}",
            "description": "Local time in the user's timezone like 2026-10-14T15:00, or RFC 3339 with an offset like 2026-10-14T15:00:00+11:00."
        })
    }
}

/// A calendar date, `YYYY-MM-DD`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateInput(pub NaiveDate);

impl<'de> Deserialize<'de> for DateInput {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        NaiveDate::parse_from_str(s.trim(), "%Y-%m-%d")
            .map(DateInput)
            .map_err(|_| serde::de::Error::custom(format!("'{s}' is not a date; use YYYY-MM-DD")))
    }
}

impl JsonSchema for DateInput {
    fn schema_name() -> Cow<'static, str> {
        "Date".into()
    }

    fn inline_schema() -> bool {
        true
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "type": "string",
            "pattern": r"^\d{4}-\d{2}-\d{2}$",
            "description": "Calendar date, YYYY-MM-DD."
        })
    }
}

/// A time of day, `HH:MM` (24-hour).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeInput(pub NaiveTime);

impl<'de> Deserialize<'de> for TimeInput {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        NaiveTime::parse_from_str(s.trim(), "%H:%M")
            .map(TimeInput)
            .map_err(|_| {
                serde::de::Error::custom(format!("'{s}' is not a time; use HH:MM (24-hour)"))
            })
    }
}

impl JsonSchema for TimeInput {
    fn schema_name() -> Cow<'static, str> {
        "TimeOfDay".into()
    }

    fn inline_schema() -> bool {
        true
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "type": "string",
            "pattern": r"^([01]\d|2[0-3]):[0-5]\d$",
            "description": "Time of day, HH:MM in 24-hour format."
        })
    }
}

/// For patch fields: absent → `None` (leave unchanged), `null` →
/// `Some(None)` (clear), value → `Some(Some(v))` (set).
pub fn patch<'de, D, T>(d: D) -> Result<Option<Option<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(d).map(Some)
}

/// Arguments for operations that take none.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct NoArgs {}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use chrono_tz::Australia::Melbourne;

    fn local(s: &str) -> NaiveDateTime {
        NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M").unwrap()
    }

    #[test]
    fn parses_absolute_and_local_forms() {
        let abs: DateTimeInput = serde_json::from_str(r#""2026-10-14T15:00:00+11:00""#).unwrap();
        assert_eq!(
            abs.resolve(chrono_tz::UTC),
            Utc.with_ymd_and_hms(2026, 10, 14, 4, 0, 0).unwrap()
        );
        let loc: DateTimeInput = serde_json::from_str(r#""2026-10-14 15:00""#).unwrap();
        assert_eq!(loc, DateTimeInput::Local(local("2026-10-14T15:00")));
        assert_eq!(
            loc.resolve(Melbourne),
            Utc.with_ymd_and_hms(2026, 10, 14, 4, 0, 0).unwrap()
        );
        assert!(serde_json::from_str::<DateTimeInput>(r#""tuesday 3pm""#).is_err());
    }

    #[test]
    fn dst_gap_moves_forward_and_overlap_takes_earlier() {
        // Melbourne springs forward 2026-10-04 02:00 → 03:00.
        let gap = resolve_local(Melbourne, local("2026-10-04T02:30"));
        assert_eq!(gap, resolve_local(Melbourne, local("2026-10-04T03:30")));
        // Falls back 2026-04-05 03:00 → 02:00; 02:30 happens twice.
        let overlap = resolve_local(Melbourne, local("2026-04-05T02:30"));
        assert_eq!(
            overlap,
            Utc.with_ymd_and_hms(2026, 4, 4, 15, 30, 0).unwrap()
        );
    }

    #[test]
    fn date_and_time_inputs_validate() {
        assert!(serde_json::from_str::<DateInput>(r#""2026-02-30""#).is_err());
        assert!(serde_json::from_str::<TimeInput>(r#""24:00""#).is_err());
        let t: TimeInput = serde_json::from_str(r#""08:30""#).unwrap();
        assert_eq!(t.0, NaiveTime::from_hms_opt(8, 30, 0).unwrap());
    }
}

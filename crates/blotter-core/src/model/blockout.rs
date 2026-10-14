use chrono::{NaiveDate, NaiveTime, Weekday};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::time::Instant;

/// A recurring window flexible tasks may never be placed in, such as Work
/// 08:30–17:00 on weekdays. An `end_time` earlier than `start_time` wraps
/// past midnight.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Blockout {
    pub id: String,
    pub label: String,
    pub weekdays: Weekdays,
    #[serde(with = "hhmm")]
    pub start_time: NaiveTime,
    #[serde(with = "hhmm")]
    pub end_time: NaiveTime,
    pub tz: String,
    pub active_from: Option<NaiveDate>,
    pub active_until: Option<NaiveDate>,
    /// Whether fixed items may sit inside without a warning (false for e.g. Sleep).
    pub permits_fixed: bool,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub sort_order: i64,
    pub created_at: Instant,
    pub updated_at: Instant,
}

impl Blockout {
    pub fn wraps_midnight(&self) -> bool {
        self.end_time < self.start_time
    }
}

/// A date on which a block-out is lifted (leave, public holidays).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockoutException {
    pub id: String,
    pub blockout_id: String,
    pub date: NaiveDate,
    pub note: String,
    pub created_at: Instant,
}

/// A set of weekdays, stored as a bitmask (Monday = 1 … Sunday = 64) and
/// serialized as a list like `["mon", "tue"]` so it reads well in JSON.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Weekdays(u8);

const NAMES: [&str; 7] = ["mon", "tue", "wed", "thu", "fri", "sat", "sun"];

impl Weekdays {
    pub const WEEKDAYS: Weekdays = Weekdays(0b001_1111);
    pub const EVERY_DAY: Weekdays = Weekdays(0b111_1111);

    pub fn from_bits(bits: u8) -> Option<Self> {
        (1..=127).contains(&bits).then_some(Self(bits))
    }

    pub fn bits(self) -> u8 {
        self.0
    }

    pub fn contains(self, day: Weekday) -> bool {
        self.0 & (1 << day.num_days_from_monday()) != 0
    }

    pub fn from_days(days: impl IntoIterator<Item = Weekday>) -> Self {
        Self(
            days.into_iter()
                .fold(0, |acc, d| acc | (1 << d.num_days_from_monday())),
        )
    }

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }
}

impl Serialize for Weekdays {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let names: Vec<&str> = (0..7)
            .filter(|i| self.0 & (1 << i) != 0)
            .map(|i| NAMES[i])
            .collect();
        names.serialize(s)
    }
}

impl<'de> Deserialize<'de> for Weekdays {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let names = Vec::<String>::deserialize(d)?;
        let mut bits = 0u8;
        for name in names {
            let i = NAMES
                .iter()
                .position(|n| n.eq_ignore_ascii_case(&name))
                .ok_or_else(|| serde::de::Error::custom(format!("unknown weekday '{name}'")))?;
            bits |= 1 << i;
        }
        if bits == 0 {
            return Err(serde::de::Error::custom("weekdays must not be empty"));
        }
        Ok(Self(bits))
    }
}

/// Serde for `NaiveTime` as `HH:MM`, matching the database encoding.
mod hhmm {
    use chrono::NaiveTime;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(t: &NaiveTime, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&t.format("%H:%M").to_string())
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<NaiveTime, D::Error> {
        let s = String::deserialize(d)?;
        NaiveTime::parse_from_str(&s, "%H:%M").map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weekdays_bitmask_and_json() {
        let w = Weekdays::WEEKDAYS;
        assert!(w.contains(Weekday::Mon) && w.contains(Weekday::Fri));
        assert!(!w.contains(Weekday::Sat));
        assert_eq!(
            serde_json::to_string(&w).unwrap(),
            r#"["mon","tue","wed","thu","fri"]"#
        );
        let back: Weekdays = serde_json::from_str(r#"["Sat","sun"]"#).unwrap();
        assert_eq!(back, Weekdays::from_days([Weekday::Sat, Weekday::Sun]));
    }

    #[test]
    fn weekdays_reject_empty_and_unknown() {
        assert!(serde_json::from_str::<Weekdays>("[]").is_err());
        assert!(serde_json::from_str::<Weekdays>(r#"["funday"]"#).is_err());
        assert!(Weekdays::from_bits(0).is_none());
        assert!(Weekdays::from_bits(128).is_none());
    }
}

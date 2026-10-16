//! Row-level storage. Plain functions over a `&Connection` (a `Transaction`
//! derefs to one), so the operation layer can compose several writes and the
//! change-log entry into one transaction.
//!
//! Nothing here writes the change log; that is the operation layer's job.

pub mod blockouts;
pub mod changes;
pub mod chat;
pub mod items;
pub mod keybindings;
pub mod providers;
pub mod reminders;
pub mod settings;
pub mod subscriptions;
pub mod themes;

use chrono::{NaiveDate, NaiveTime};

use crate::error::Result;
use crate::time::{self, Instant};

pub(crate) fn opt_instant(column: &'static str, v: Option<String>) -> Result<Option<Instant>> {
    v.map(|s| time::parse_instant(column, &s)).transpose()
}

pub(crate) fn opt_date(column: &'static str, v: Option<String>) -> Result<Option<NaiveDate>> {
    v.map(|s| time::parse_date(column, &s)).transpose()
}

pub(crate) fn req_time(column: &'static str, v: String) -> Result<NaiveTime> {
    time::parse_time(column, &v)
}

pub(crate) fn fmt_opt_instant(v: Option<Instant>) -> Option<String> {
    v.map(time::fmt_instant)
}

pub(crate) fn fmt_opt_date(v: Option<NaiveDate>) -> Option<String> {
    v.map(time::fmt_date)
}

pub(crate) fn parse_enum<T: std::str::FromStr>(column: &'static str, v: String) -> Result<T> {
    v.parse()
        .map_err(|_| crate::error::CoreError::Corrupt { column, value: v })
}

pub(crate) fn json_value(
    column: &'static str,
    v: Option<String>,
) -> Result<Option<serde_json::Value>> {
    v.map(|s| {
        serde_json::from_str(&s).map_err(|_| crate::error::CoreError::Corrupt { column, value: s })
    })
    .transpose()
}

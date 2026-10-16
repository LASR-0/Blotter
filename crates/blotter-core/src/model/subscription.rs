use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::time::Instant;

/// A read-only calendar fetched from a URL (roadmap §5.8), such as a
/// published Outlook calendar. Its events are cached in `ics_events`; with
/// `as_blockout` on, its busy timed events act as block-outs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Subscription {
    pub id: String,
    pub name: String,
    /// As the user gave it; `webcal://` is fetched over https.
    pub url: String,
    /// Minutes between fetches (at least 5).
    pub refresh_min: u32,
    pub color: Option<String>,
    pub as_blockout: bool,
    pub enabled: bool,
    /// Last attempt, successful or not.
    pub last_fetch_at: Option<Instant>,
    pub last_ok_at: Option<Instant>,
    /// `ok` or `error` after the first fetch.
    pub last_status: Option<String>,
    /// Why the last fetch failed, or a note on what it skipped.
    pub last_error: Option<String>,
    pub created_at: Instant,
    pub updated_at: Instant,
}

impl Subscription {
    /// Whether it should be fetched at `now`: enabled, and never fetched or
    /// its refresh interval has passed since the last attempt.
    pub fn is_due(&self, now: Instant) -> bool {
        self.enabled
            && self
                .last_fetch_at
                .is_none_or(|at| now - at >= chrono::Duration::minutes(i64::from(self.refresh_min)))
    }
}

/// One cached occurrence of a subscribed event. Recurring events are
/// expanded when fetched, so each row is one concrete occurrence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubscriptionEvent {
    pub id: String,
    pub subscription_id: String,
    pub uid: String,
    pub title: String,
    pub location: String,
    pub all_day: bool,
    pub start_at: Option<Instant>,
    pub end_at: Option<Instant>,
    pub start_date: Option<NaiveDate>,
    /// Last day, inclusive (as on items); none for a one-day event.
    pub end_date: Option<NaiveDate>,
    /// False for events marked free (TRANSP:TRANSPARENT); those never block.
    pub busy: bool,
}

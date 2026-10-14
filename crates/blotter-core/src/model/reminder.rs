use serde::{Deserialize, Serialize};

use crate::time::Instant;

/// A notification attached to an item: either `offset_min` before the item's
/// start, or at an absolute instant. Exactly one of the two is set.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Reminder {
    pub id: String,
    pub item_id: String,
    pub offset_min: Option<u32>,
    pub at: Option<Instant>,
    pub fired_at: Option<Instant>,
    pub snoozed_until: Option<Instant>,
    pub created_at: Instant,
}

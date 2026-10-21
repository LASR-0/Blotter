use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::time::Instant;

/// A stretch of days away from the usual week (inclusive dates). Block-outs
/// are lifted on its days, work tasks aren't slotted into it, and tasks
/// tagged `holiday` get first pick of it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Holiday {
    pub id: String,
    pub label: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub color: Option<String>,
    pub created_at: Instant,
    pub updated_at: Instant,
    /// `public:<country or region>` for an imported public holiday; null
    /// for one the user or the AI made.
    #[serde(default)]
    pub source: Option<String>,
}

/// The `source` prefix of imported public holidays.
pub const PUBLIC_SOURCE: &str = "public:";

impl Holiday {
    pub fn is_public(&self) -> bool {
        self.source
            .as_deref()
            .is_some_and(|s| s.starts_with(PUBLIC_SOURCE))
    }
}

impl Holiday {
    pub fn covers(&self, date: NaiveDate) -> bool {
        self.start_date <= date && date <= self.end_date
    }

    /// Every date of the holiday, first to last.
    pub fn dates(&self) -> impl Iterator<Item = NaiveDate> + '_ {
        self.start_date
            .iter_days()
            .take_while(move |d| *d <= self.end_date)
    }
}

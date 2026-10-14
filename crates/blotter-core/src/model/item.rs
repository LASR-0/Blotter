use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use super::string_enum;
use crate::time::Instant;

string_enum!(
    /// What an item is. Tasks are things to do, events happen at a time,
    /// reminders exist only to notify.
    ItemKind { Task => "task", Event => "event", Reminder => "reminder" }
);

string_enum!(
    /// Fixed items stay exactly where they are put (and may sit inside
    /// block-outs). Flexible items may be moved by the slot engine and can
    /// never land in a block-out.
    Anchoring { Fixed => "fixed", Flexible => "flexible" }
);

string_enum!(ItemStatus { Open => "open", Done => "done", Cancelled => "cancelled" });

string_enum!(
    /// Who created the item.
    Source { User => "user", Ai => "ai", Import => "import", Mcp => "mcp" }
);

impl ItemKind {
    /// Events default to fixed; tasks and reminders to flexible.
    pub fn default_anchoring(self) -> Anchoring {
        match self {
            ItemKind::Event => Anchoring::Fixed,
            ItemKind::Task | ItemKind::Reminder => Anchoring::Flexible,
        }
    }
}

/// A task, event or reminder. See roadmap §4.4.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Item {
    pub id: String,
    pub kind: ItemKind,
    pub anchoring: Anchoring,
    pub status: ItemStatus,
    pub title: String,
    pub notes: String,
    pub all_day: bool,
    /// Timed items only.
    pub start_at: Option<Instant>,
    pub end_at: Option<Instant>,
    /// All-day items only; `end_date` is inclusive.
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    /// IANA zone for fixed items, e.g. `Australia/Melbourne`.
    pub tz: Option<String>,
    pub duration_min: Option<u32>,
    pub due_at: Option<Instant>,
    /// 0 (none) to 3 (highest).
    pub priority: u8,
    pub tags: Vec<String>,
    pub color: Option<String>,
    /// Manual position within a Diary day column.
    pub sort_order: f64,
    pub rrule: Option<String>,
    pub parent_id: Option<String>,
    pub recurrence_id: Option<String>,
    pub completed_at: Option<Instant>,
    pub source: Source,
    pub created_at: Instant,
    pub updated_at: Instant,
}

impl Item {
    /// A fresh open item with sensible defaults for its kind. Callers set
    /// scheduling fields afterwards.
    pub fn new(id: String, kind: ItemKind, title: impl Into<String>, now: Instant) -> Self {
        Self {
            id,
            kind,
            anchoring: kind.default_anchoring(),
            status: ItemStatus::Open,
            title: title.into(),
            notes: String::new(),
            all_day: false,
            start_at: None,
            end_at: None,
            start_date: None,
            end_date: None,
            tz: None,
            duration_min: None,
            due_at: None,
            priority: 0,
            tags: Vec::new(),
            color: None,
            sort_order: 0.0,
            rrule: None,
            parent_id: None,
            recurrence_id: None,
            completed_at: None,
            source: Source::User,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn is_scheduled(&self) -> bool {
        self.start_at.is_some() || self.start_date.is_some()
    }

    /// The Inbox is every flexible task with no time and no date.
    pub fn is_inbox(&self) -> bool {
        self.kind == ItemKind::Task && self.anchoring == Anchoring::Flexible && !self.is_scheduled()
    }

    pub fn is_done(&self) -> bool {
        self.status == ItemStatus::Done
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    fn now() -> Instant {
        Utc.with_ymd_and_hms(2026, 10, 14, 9, 0, 0).unwrap()
    }

    #[test]
    fn defaults_follow_kind() {
        assert_eq!(
            Item::new("a".into(), ItemKind::Event, "x", now()).anchoring,
            Anchoring::Fixed
        );
        assert_eq!(
            Item::new("b".into(), ItemKind::Task, "x", now()).anchoring,
            Anchoring::Flexible
        );
    }

    #[test]
    fn inbox_is_unscheduled_flexible_task() {
        let mut t = Item::new("a".into(), ItemKind::Task, "Call plumber", now());
        assert!(t.is_inbox());
        t.start_at = Some(now());
        assert!(!t.is_inbox());
        t.start_at = None;
        t.anchoring = Anchoring::Fixed;
        assert!(!t.is_inbox());
    }

    #[test]
    fn enums_serialize_lowercase() {
        assert_eq!(
            serde_json::to_string(&ItemKind::Event).unwrap(),
            "\"event\""
        );
        assert_eq!(
            "flexible".parse::<Anchoring>().unwrap(),
            Anchoring::Flexible
        );
        assert!("Flexible".parse::<Anchoring>().is_err());
    }
}

//! Blotter's domain core.
//!
//! Everything that owns or reasons about planner state lives here: models,
//! storage, the operation registry, the scheduler, recurrence and ICS. The
//! crate has no Tauri dependency so it can be unit-tested on its own and
//! shared by the desktop app and the MCP server.

pub mod alerts;
pub mod briefing;
pub mod context;
pub mod db;
pub mod error;
pub mod export;
pub mod ics;
pub mod id;
pub mod model;
pub mod ops;
mod planner;
pub mod proposals;
pub mod public_holidays;
pub mod recurrence;
pub mod settings;
pub mod slots;
pub mod smart_slot;
pub mod snapshots;
pub mod store;
pub mod subscriptions;
pub mod sync;
pub mod time;
pub mod windows;

pub use db::Database;
pub use error::{CoreError, Result};
pub use model::Actor;
pub use ops::{OpCall, OpInfo, OpOutcome};
pub use planner::Planner;

/// Crate version, surfaced in the About page and MCP server info.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_set() {
        assert!(!VERSION.is_empty());
    }
}

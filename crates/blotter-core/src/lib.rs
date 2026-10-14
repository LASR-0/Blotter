//! Blotter's domain core.
//!
//! Everything that owns or reasons about planner state lives here: models,
//! storage, the operation registry, the scheduler, recurrence and ICS. The
//! crate has no Tauri dependency so it can be unit-tested on its own and
//! shared by the desktop app and the MCP server.

pub mod db;
pub mod error;
pub mod id;
pub mod model;
pub mod store;
pub mod time;

pub use db::Database;
pub use error::{CoreError, Result};

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

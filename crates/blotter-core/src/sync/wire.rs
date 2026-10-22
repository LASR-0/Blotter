//! What replicas send each other: the sync protocol's types.
//!
//! Every change travels as a [`RowState`]: one row's existence stamp and the
//! values and stamps of its field groups. Pushes, pulls and bootstraps all
//! carry them, and the receiver merges each with [`super::merge`], so the
//! same function settles every replica. Blotter Sync's endpoints wrap these
//! in its own request and response types; anything that changes how a
//! replica reads them bumps [`SYNC_PROTOCOL`].

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::sync::Hlc;

/// The version of these types and the merge rules. Every sync request
/// carries it with [`super::registry::SYNC_SCHEMA`], and Blotter Sync refuses
/// a device whose version differs.
pub const SYNC_PROTOCOL: u32 = 1;

/// One row as a replica holds it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RowState {
    pub table: String,
    /// The primary key, as text.
    pub key: String,
    /// When the row was last created or deleted.
    pub row: Hlc,
    /// A tombstone: the row is gone, and `groups` is empty.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub deleted: bool,
    /// Field group name → its stamp and column values.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub groups: BTreeMap<String, GroupState>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroupState {
    pub hlc: Hlc,
    /// Column → value, for every column in the group.
    pub values: Map<String, Value>,
}

impl RowState {
    /// The newest stamp anywhere in the row.
    pub fn newest(&self) -> &Hlc {
        self.groups
            .values()
            .map(|g| &g.hlc)
            .fold(&self.row, |a, b| a.max(b))
    }
}

/// Why a merge left a row out.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rejection {
    pub table: String,
    pub key: String,
    pub reason: String,
}

/// What a merge did.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Merged {
    /// The state after the merge of every row it changed, including rows it
    /// deleted because a parent went or a unique key collided.
    pub changed: Vec<RowState>,
    pub rejected: Vec<Rejection>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn stamp(wall_ms: i64) -> Hlc {
        Hlc {
            wall_ms,
            counter: 0,
            device: "01JBDEVICE0000000000000000".into(),
        }
    }

    #[test]
    fn rows_round_trip_as_compact_json() {
        let live = RowState {
            table: "items".into(),
            key: "I1".into(),
            row: stamp(1),
            deleted: false,
            groups: BTreeMap::from([(
                "title".into(),
                GroupState {
                    hlc: stamp(2),
                    values: json!({ "title": "Buy milk" }).as_object().unwrap().clone(),
                },
            )]),
        };
        let text = serde_json::to_string(&live).unwrap();
        assert!(!text.contains("deleted"));
        assert_eq!(serde_json::from_str::<RowState>(&text).unwrap(), live);
        assert_eq!(live.newest(), &stamp(2));

        let gone = RowState {
            deleted: true,
            groups: BTreeMap::new(),
            ..live
        };
        let text = serde_json::to_string(&gone).unwrap();
        assert!(!text.contains("groups"));
        assert_eq!(serde_json::from_str::<RowState>(&text).unwrap(), gone);
    }
}

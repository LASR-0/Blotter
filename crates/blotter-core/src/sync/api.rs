//! Blotter Sync's HTTP API: the request and response bodies around the
//! [`RowState`]s, and its typed errors, shared by the app's client and the
//! server so the two can't drift (roadmap §14.5). The transport itself
//! (reqwest in the app, axum in the server) stays out of the core.
//!
//! Every request but `hello` carries [`HEADER_PROTOCOL`] and
//! [`HEADER_SCHEMA`], and an `Authorization: Bearer <token>` header once
//! paired. Changing anything here bumps [`SYNC_PROTOCOL`].

use serde::{Deserialize, Serialize};

use crate::sync::registry::SYNC_SCHEMA;
use crate::sync::{Rejection, RowState, SYNC_PROTOCOL};

pub const HEADER_PROTOCOL: &str = "x-blotter-protocol";
pub const HEADER_SCHEMA: &str = "x-blotter-schema";

/// The endpoints, relative to a server address.
pub mod path {
    pub const HELLO: &str = "/v1/hello";
    pub const PAIR: &str = "/v1/pair";
    pub const DEVICES_SELF: &str = "/v1/devices/self";
    pub const PUSH: &str = "/v1/push";
    /// With `?epoch=<epoch>&since=<seq>`.
    pub const PULL: &str = "/v1/pull";
    pub const BOOTSTRAP: &str = "/v1/bootstrap";
    /// Server-sent events, each `event: seq` with a [`super::Notice`].
    pub const EVENTS: &str = "/v1/events";
    pub const SNAPSHOTS: &str = "/v1/snapshots";
    pub const IMPORT: &str = "/v1/import";

    /// `POST` here restores snapshot `id`.
    pub fn restore(id: &str) -> String {
        format!("{SNAPSHOTS}/{id}/restore")
    }
}

/// The versions that must match for a device and a server to sync.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Versions {
    pub protocol: u32,
    pub schema: u32,
}

impl Versions {
    /// This build's.
    pub const CURRENT: Versions = Versions {
        protocol: SYNC_PROTOCOL,
        schema: SYNC_SCHEMA,
    };
}

/// `GET /v1/hello`, answered without authentication.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hello {
    pub name: String,
    /// Blotter Sync's version.
    pub version: String,
    /// The Blotter release whose core the server was built with.
    pub blotter_core: String,
    pub server_id: String,
    #[serde(flatten)]
    pub versions: Versions,
    pub epoch: String,
    /// Whether a pairing code is waiting to be used.
    pub pairing_open: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    /// The device's sync protocol differs from the server's.
    ProtocolMismatch,
    /// The device's sync schema differs from the server's.
    SchemaMismatch,
    /// No token, or one the server doesn't know.
    Unauthorized,
    /// This device's token was revoked.
    Revoked,
    /// No pairing code is open (`blotter-sync pair` makes one).
    PairingClosed,
    /// The pairing code is wrong.
    BadCode,
    /// The pairing code has expired or was used up.
    CodeExpired,
    /// Too many attempts from this address; try again shortly.
    RateLimited,
    /// The server's planner was replaced: bootstrap again.
    StaleEpoch,
    /// The request itself is malformed.
    Invalid,
    /// Something went wrong on the server.
    Internal,
    /// A code this build doesn't know (from a newer server).
    #[serde(other)]
    Unknown,
}

/// Every refusal's body. Version mismatches carry the server's versions, so
/// the app can say which side to update.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorBody {
    pub code: ErrorCode,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server: Option<Versions>,
}

/// `POST /v1/pair`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PairRequest {
    pub code: String,
    /// The device's id (a ULID), kept for good.
    pub device_id: String,
    /// What the user calls it ("Desktop", "Laptop").
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Paired {
    /// Shown once; the device keeps it in its OS keychain.
    pub token: String,
    pub device_id: String,
    pub server_id: String,
    pub epoch: String,
}

/// `POST /v1/push`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PushRequest {
    /// The epoch the device last bootstrapped from.
    pub epoch: String,
    /// The device's id for this push (a ULID): a resend with the same id
    /// changes nothing and gets the same answer.
    pub batch_id: String,
    pub rows: Vec<RowState>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PushResponse {
    /// The server's newest seq once the push is in.
    pub seq: i64,
    /// Rows the server couldn't take, and why.
    pub rejected: Vec<Rejection>,
}

/// `GET /v1/pull`: one page of changes after a seq.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PullResponse {
    pub epoch: String,
    pub rows: Vec<RowState>,
    /// The seq to ask from next time.
    pub seq: i64,
    /// More rows are waiting after this page.
    pub more: bool,
}

/// `GET /v1/bootstrap`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bootstrap {
    pub epoch: String,
    /// Pull from here afterwards.
    pub seq: i64,
    /// Every row the server holds, tombstones included.
    pub rows: Vec<RowState>,
}

/// What the change stream says: pull now (up to `seq`), or, when the epoch
/// moved, bootstrap again.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Notice {
    pub seq: i64,
    pub epoch: String,
}

/// A server snapshot as the API shows it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SnapshotInfo {
    pub id: String,
    pub reason: String,
    pub manual: bool,
    pub note: Option<String>,
    pub size_bytes: Option<i64>,
    pub created_at: String,
}

/// `POST /v1/snapshots`, optional.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotRequest {
    #[serde(default)]
    pub note: Option<String>,
}

/// What a server restore or import did.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Replaced {
    /// The new epoch every device bootstraps from.
    pub epoch: String,
    /// The snapshot taken just before, so the replace can be undone.
    pub before: SnapshotInfo,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn hello_reads_as_blotter_sync_sends_it() {
        let text = json!({
            "name": "Blotter Sync", "version": "1.0.0", "blotter_core": "1.2.1",
            "server_id": "01JBSERVER0000000000000000", "protocol": 1, "schema": 1,
            "epoch": "01JBEPOCH00000000000000000", "pairing_open": false
        });
        let hello: Hello = serde_json::from_value(text.clone()).unwrap();
        assert_eq!(hello.versions, Versions::CURRENT);
        assert_eq!(serde_json::to_value(&hello).unwrap(), text);
    }

    #[test]
    fn errors_carry_the_servers_versions_and_tolerate_new_codes() {
        let body: ErrorBody = serde_json::from_value(json!({
            "code": "schema_mismatch", "message": "update",
            "server": { "protocol": 1, "schema": 2 }
        }))
        .unwrap();
        assert_eq!(body.code, ErrorCode::SchemaMismatch);
        assert_eq!(body.server.unwrap().schema, 2);

        let newer: ErrorBody =
            serde_json::from_value(json!({ "code": "quota_exceeded", "message": "x" })).unwrap();
        assert_eq!(newer.code, ErrorCode::Unknown);
        assert!(!serde_json::to_string(&newer).unwrap().contains("server"));
    }

    #[test]
    fn paths_and_pages() {
        assert_eq!(path::restore("S1"), "/v1/snapshots/S1/restore");
        let page: PullResponse =
            serde_json::from_value(json!({ "epoch": "E", "rows": [], "seq": 7, "more": false }))
                .unwrap();
        assert_eq!((page.seq, page.more), (7, false));
    }
}

//! An in-memory Blotter Sync for the engine's and pairing's tests: a
//! server-role replica, a log of changed rows by seq, the open pairing
//! code and the devices' tokens, and switches for what goes wrong.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use blotter_core::sync::api::{
    ErrorBody, ErrorCode, Hello, PairRequest, Paired, PullResponse, PushRequest, PushResponse,
    Versions,
};
use blotter_core::sync::capture::{self, Role};
use blotter_core::sync::{RowState, link, merge};
use blotter_core::{Actor, Database, Planner};

use super::client::{SyncError, Transport};

pub const SERVER: &str = "01JBSERVER0000000000000000";
pub const EPOCH: &str = "01JBEPOCH00000000000000000";
pub const LAN: &str = "http://192.168.1.10:7458";
pub const TAILNET: &str = "https://blotter.tail1234.ts.net";
pub const CODE: &str = "ABCD-EFGH";

pub struct Fake {
    pub planner: Planner,
    pub log: Vec<(i64, RowState)>,
    pub versions: Versions,
    pub down: HashSet<String>,
    pub elsewhere: HashSet<String>,
    pub refuse: Option<ErrorCode>,
    /// The open pairing code, if any.
    pub code: Option<String>,
    /// Tokens by device id; a revoked device is gone from here.
    pub devices: HashMap<String, (String, String)>,
    pub revoked: HashSet<String>,
}

#[derive(Clone)]
pub struct Shared(pub Arc<Mutex<Fake>>);

impl Shared {
    pub fn new() -> Self {
        let mut planner = Planner::new(Database::open_in_memory().unwrap(), chrono_tz::UTC);
        capture::set_role(planner.db_mut().conn_mut(), Role::Server).unwrap();
        Self(Arc::new(Mutex::new(Fake {
            planner,
            log: Vec::new(),
            versions: Versions::CURRENT,
            down: HashSet::new(),
            elsewhere: HashSet::new(),
            refuse: None,
            code: None,
            devices: HashMap::new(),
            revoked: HashSet::new(),
        })))
    }

    pub fn with<R>(&self, f: impl FnOnce(&mut Fake) -> R) -> R {
        f(&mut self.0.lock().unwrap())
    }
}

pub fn refused(code: ErrorCode) -> SyncError {
    SyncError::Refused(ErrorBody {
        code,
        message: format!("{code:?}"),
        server: None,
    })
}

fn reachable(f: &Fake, base: &str) -> Result<(), SyncError> {
    if f.down.contains(base) {
        Err(SyncError::Unreachable("couldn't connect".into()))
    } else {
        Ok(())
    }
}

impl Transport for Shared {
    async fn hello(&self, base: &str) -> Result<Hello, SyncError> {
        self.with(|f| {
            reachable(f, base)?;
            let id = if f.elsewhere.contains(base) {
                "01JBSOMEONEELSE00000000000"
            } else {
                SERVER
            };
            Ok(Hello {
                name: "Blotter Sync".into(),
                version: "1.0.0".into(),
                blotter_core: "1.2.1".into(),
                server_id: id.into(),
                versions: f.versions,
                epoch: EPOCH.into(),
                pairing_open: f.code.is_some(),
            })
        })
    }

    async fn pair(&self, base: &str, req: &PairRequest) -> Result<Paired, SyncError> {
        self.with(|f| {
            reachable(f, base)?;
            if let Some(code) = f.refuse {
                return Err(refused(code));
            }
            let Some(open) = &f.code else {
                return Err(refused(ErrorCode::PairingClosed));
            };
            if req.code.replace('-', "").to_uppercase() != open.replace('-', "") {
                return Err(refused(ErrorCode::BadCode));
            }
            f.code = None;
            let token = format!("token-{}-{}", req.device_id, f.devices.len());
            f.devices
                .insert(req.device_id.clone(), (token.clone(), req.name.clone()));
            f.revoked.remove(&req.device_id);
            Ok(Paired {
                token,
                device_id: req.device_id.clone(),
                server_id: SERVER.into(),
                epoch: EPOCH.into(),
            })
        })
    }

    async fn disconnect(&self, base: &str, token: &str) -> Result<(), SyncError> {
        self.with(|f| {
            reachable(f, base)?;
            let Some(id) = f
                .devices
                .iter()
                .find(|(_, (t, _))| t == token)
                .map(|(id, _)| id.clone())
            else {
                return Err(refused(ErrorCode::Unauthorized));
            };
            f.devices.remove(&id);
            f.revoked.insert(id);
            Ok(())
        })
    }

    async fn push(
        &self,
        base: &str,
        _token: &str,
        req: &PushRequest,
    ) -> Result<PushResponse, SyncError> {
        self.with(|f| {
            reachable(f, base)?;
            if let Some(code) = f.refuse {
                return Err(refused(code));
            }
            let merged =
                merge(f.planner.db_mut().conn_mut(), &req.rows, chrono::Utc::now()).unwrap();
            for row in merged.changed {
                let seq = f.log.len() as i64 + 1;
                f.log.push((seq, row));
            }
            Ok(PushResponse {
                seq: f.log.len() as i64,
                rejected: merged.rejected,
            })
        })
    }

    async fn pull(
        &self,
        base: &str,
        _token: &str,
        _epoch: &str,
        since: i64,
    ) -> Result<PullResponse, SyncError> {
        self.with(|f| {
            reachable(f, base)?;
            if let Some(code) = f.refuse {
                return Err(refused(code));
            }
            Ok(PullResponse {
                epoch: EPOCH.into(),
                rows: f
                    .log
                    .iter()
                    .filter(|(s, _)| *s > since)
                    .map(|(_, r)| r.clone())
                    .collect(),
                seq: f.log.len() as i64,
                more: false,
            })
        })
    }
}

/// A device that hasn't paired.
pub fn alone() -> Mutex<Planner> {
    Mutex::new(Planner::new(
        Database::open_in_memory().unwrap(),
        chrono_tz::UTC,
    ))
}

/// A device paired with [`SERVER`] at `addresses`.
pub fn device(addresses: &[&str]) -> Mutex<Planner> {
    let db = alone();
    link::set(
        db.lock().unwrap().db().conn(),
        &link::Link {
            server_id: SERVER.into(),
            addresses: addresses.iter().map(|a| (*a).to_owned()).collect(),
            epoch: EPOCH.into(),
            seq: 0,
            paired_at: chrono::Utc::now(),
            last_sync_at: None,
        },
    )
    .unwrap();
    db
}

pub fn run(db: &Mutex<Planner>, op: &str, args: serde_json::Value) -> serde_json::Value {
    db.lock()
        .unwrap()
        .execute(op, args, &Actor::User)
        .unwrap()
        .result
}

pub fn title(db: &Mutex<Planner>, id: &str) -> Option<String> {
    blotter_core::store::items::get(db.lock().unwrap().db().conn(), id)
        .unwrap()
        .map(|i| i.title)
}

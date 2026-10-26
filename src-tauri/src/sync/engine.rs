//! One sync: pick the first address that answers as this device's server,
//! push the outbox, pull everything after the cursor and merge it (roadmap
//! §14.6). The loop around it, the change stream and the status the UI
//! shows are in `mod.rs`.

use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use blotter_core::Planner;
use blotter_core::snapshots::{self, Reason};
use blotter_core::sync::api::{ErrorCode, PushRequest, Versions};
use blotter_core::sync::{capture, join, link, merge, state};
use serde::Serialize;

use super::address;
use super::client::{SyncError, Transport};

/// While syncing through a later address, how often the earlier ones are
/// tried again, so a laptop back home moves back to the LAN.
pub const RECHECK: Duration = Duration::from_secs(5 * 60);

/// What a sync did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Synced {
    pub address: String,
    /// Rows sent.
    pub pushed: usize,
    /// Rows the merge changed here.
    pub pulled: usize,
    /// Rows the server wouldn't take.
    pub rejected: usize,
    /// Blotter Sync's version, from its last hello.
    pub server_version: Option<String>,
    /// The server's planner was replaced, so this device fetched it again.
    pub replaced: Option<Replaced>,
}

/// What fetching a replaced planner left behind (roadmap §14.8).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Replaced {
    /// The snapshot of this device's planner from just before.
    pub snapshot_id: String,
    /// Changes the server never got; they're in that snapshot.
    pub unsent: u64,
}

/// Something only the user (or an update) can fix; retrying won't help.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "reason", rename_all = "snake_case")]
pub enum Attention {
    /// The server is newer: update Blotter on this device.
    UpdateBlotter { server: Versions },
    /// The server is older: update Blotter Sync.
    UpdateServer { server: Versions },
    /// The server disconnected this device.
    Revoked,
    /// The server doesn't know this device's token (or it's missing here).
    Unauthorized,
    /// The server's planner was replaced: fetch it again.
    Rebootstrap,
    /// Every address that answered is a different Blotter Sync server.
    WrongServer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Problem {
    /// No server is paired.
    LocalOnly,
    /// No address answered. `tried` says why each failed.
    Offline {
        tried: Vec<(String, String)>,
        tailscale: bool,
    },
    /// The server answered but couldn't serve the request just now.
    Retry(String),
    Attention(Attention),
    /// This device's own database failed.
    Local(String),
}

impl From<blotter_core::CoreError> for Problem {
    fn from(err: blotter_core::CoreError) -> Self {
        Problem::Local(err.to_string())
    }
}

fn locked(db: &Mutex<Planner>) -> Result<std::sync::MutexGuard<'_, Planner>, Problem> {
    db.lock()
        .map_err(|_| Problem::Local("the planner is unavailable".into()))
}

/// A version mismatch, naming the side to update.
pub fn mismatch(server: Versions) -> Attention {
    let ours = Versions::CURRENT;
    if (server.protocol, server.schema) > (ours.protocol, ours.schema) {
        Attention::UpdateBlotter { server }
    } else {
        Attention::UpdateServer { server }
    }
}

pub struct Engine<T> {
    transport: T,
    /// The address syncing goes through, by position in the link's list.
    route: Option<usize>,
    /// When the earlier addresses were last tried.
    rechecked: Option<Instant>,
    /// Blotter Sync's version, from the last hello that answered.
    server_version: Option<String>,
    /// The database file, for the snapshot taken before fetching a
    /// replaced planner. Without it a replaced planner only needs attention.
    db_path: Option<PathBuf>,
}

impl<T: Transport> Engine<T> {
    pub fn new(transport: T) -> Self {
        Self {
            transport,
            route: None,
            rechecked: None,
            server_version: None,
            db_path: None,
        }
    }

    /// Lets the engine fetch a replaced planner, snapshotting this one into
    /// the backups folder beside `db_path` first.
    pub fn snapshots_beside(mut self, db_path: PathBuf) -> Self {
        self.db_path = Some(db_path);
        self
    }

    pub fn transport(&self) -> &T {
        &self.transport
    }

    /// Forgets the route, so the next sync tries every address in order.
    pub fn reset_route(&mut self) {
        self.route = None;
    }

    /// The first of `addresses[..end]` answering as `server_id` with this
    /// build's versions. Errors are collected in `tried`.
    async fn first_answering(
        &mut self,
        addresses: &[String],
        end: usize,
        server_id: &str,
        tried: &mut Vec<(String, String)>,
        wrong: &mut usize,
    ) -> Result<Option<usize>, Problem> {
        for (i, a) in addresses.iter().enumerate().take(end) {
            match self.transport.hello(a).await {
                Ok(h) if h.server_id != server_id => {
                    *wrong += 1;
                    tried.push((a.clone(), "a different Blotter Sync server".into()));
                }
                Ok(h) if h.versions != Versions::CURRENT => {
                    return Err(Problem::Attention(mismatch(h.versions)));
                }
                Ok(h) => {
                    self.server_version = Some(h.version);
                    return Ok(Some(i));
                }
                Err(e) => tried.push((a.clone(), e.to_string())),
            }
        }
        Ok(None)
    }

    /// The address to sync through: the current route, unless an earlier
    /// address is due a recheck and answers; otherwise the first that does.
    async fn pick(&mut self, l: &link::Link, now: Instant) -> Result<String, Problem> {
        let (mut tried, mut wrong) = (Vec::new(), 0);
        if let Some(i) = self.route.filter(|i| *i < l.addresses.len()) {
            let due = self
                .rechecked
                .is_none_or(|t| now.duration_since(t) >= RECHECK);
            if i > 0 && due {
                self.rechecked = Some(now);
                if let Some(better) = self
                    .first_answering(&l.addresses, i, &l.server_id, &mut tried, &mut wrong)
                    .await?
                {
                    self.route = Some(better);
                }
            }
            return Ok(l.addresses[self.route.unwrap_or(i)].clone());
        }
        self.rechecked = Some(now);
        let found = self
            .first_answering(
                &l.addresses,
                l.addresses.len(),
                &l.server_id,
                &mut tried,
                &mut wrong,
            )
            .await?;
        match found {
            Some(i) => {
                self.route = Some(i);
                Ok(l.addresses[i].clone())
            }
            None if wrong > 0 && wrong == l.addresses.len() => {
                Err(Problem::Attention(Attention::WrongServer))
            }
            None => Err(Problem::Offline {
                tried,
                tailscale: l.addresses.iter().any(|a| address::is_tailscale(a)),
            }),
        }
    }

    /// Turns a failed push or pull into a problem; a lost connection drops
    /// the route so the next sync looks again.
    fn failed(&mut self, address: &str, err: SyncError) -> Problem {
        match err {
            SyncError::Unreachable(m) | SyncError::Insecure(m) => {
                self.route = None;
                Problem::Offline {
                    tailscale: address::is_tailscale(address),
                    tried: vec![(address.to_owned(), m)],
                }
            }
            SyncError::Bad(m) => Problem::Retry(m),
            SyncError::Refused(body) => match body.code {
                ErrorCode::ProtocolMismatch | ErrorCode::SchemaMismatch => {
                    Problem::Attention(mismatch(body.server.unwrap_or(Versions::CURRENT)))
                }
                ErrorCode::Revoked => Problem::Attention(Attention::Revoked),
                ErrorCode::Unauthorized => Problem::Attention(Attention::Unauthorized),
                ErrorCode::StaleEpoch => Problem::Attention(Attention::Rebootstrap),
                _ => Problem::Retry(body.message),
            },
        }
    }

    /// Pushes the outbox and pulls to the server's newest seq. Merged rows
    /// reach the UI through the caller (it emits the changed event when
    /// `pulled > 0`).
    pub async fn sync_once(
        &mut self,
        db: &Mutex<Planner>,
        token: &str,
        now: Instant,
    ) -> Result<Synced, Problem> {
        let Some(l) = link::get(locked(db)?.db().conn())? else {
            self.route = None;
            return Err(Problem::LocalOnly);
        };
        if token.is_empty() {
            return Err(Problem::Attention(Attention::Unauthorized));
        }
        let address = self.pick(&l, now).await?;
        match self.exchange(db, token, &l, &address).await {
            Err(Problem::Attention(Attention::Rebootstrap)) if self.db_path.is_some() => {
                self.refetch(db, token, &address).await
            }
            other => other,
        }
    }

    /// The server's planner was replaced (restored or imported there):
    /// snapshot this one, take the server's in its place, and pull from its
    /// new epoch. Changes the server never got stay in the snapshot.
    async fn refetch(
        &mut self,
        db: &Mutex<Planner>,
        token: &str,
        address: &str,
    ) -> Result<Synced, Problem> {
        let boot = match self.transport.bootstrap(address, token).await {
            Ok(boot) => boot,
            Err(e) => return Err(self.failed(address, e)),
        };
        let Some(db_path) = &self.db_path else {
            return Err(Problem::Attention(Attention::Rebootstrap));
        };
        let now = chrono::Utc::now();
        let mut planner = locked(db)?;
        let conn = planner.db().conn();
        let unsent = capture::pending_count(conn)?;
        let dir = snapshots::dir(conn, db_path)?;
        let snap = snapshots::create(
            conn,
            &dir,
            Reason::BeforeSync,
            Some("Before the server’s replaced planner".into()),
            now,
        )?;
        let merged = join::replace_with(planner.db_mut().conn_mut(), &boot.rows, now)?;
        link::record_sync(planner.db().conn(), &boot.epoch, boot.seq, now)?;
        Ok(Synced {
            address: address.to_owned(),
            pushed: 0,
            pulled: merged.changed.len(),
            rejected: 0,
            server_version: self.server_version.clone(),
            replaced: Some(Replaced {
                snapshot_id: snap.id,
                unsent,
            }),
        })
    }

    /// Push the outbox, then pull every page after the cursor.
    async fn exchange(
        &mut self,
        db: &Mutex<Planner>,
        token: &str,
        l: &link::Link,
        address: &str,
    ) -> Result<Synced, Problem> {
        let address = address.to_owned();
        let (rows, pending) = state::outbox(locked(db)?.db().conn())?;
        let (pushed, mut rejected) = (rows.len(), 0);
        if !rows.is_empty() {
            let req = PushRequest {
                epoch: l.epoch.clone(),
                batch_id: blotter_core::id::new_id(),
                rows,
            };
            let out = match self.transport.push(&address, token, &req).await {
                Ok(out) => out,
                Err(e) => return Err(self.failed(&address, e)),
            };
            rejected = out.rejected.len();
            for r in &out.rejected {
                eprintln!(
                    "blotter: the server didn't take {} {}: {}",
                    r.table, r.key, r.reason
                );
            }
            // Rejected rows are acknowledged too: resending won't change
            // the server's mind, and they'd block the outbox for good.
            capture::acknowledge(locked(db)?.db().conn(), &pending)?;
        }

        let mut seq = l.seq;
        let mut pulled = 0;
        loop {
            let page = match self.transport.pull(&address, token, &l.epoch, seq).await {
                Ok(page) => page,
                Err(e) => return Err(self.failed(&address, e)),
            };
            let mut planner = locked(db)?;
            let merged = merge(planner.db_mut().conn_mut(), &page.rows, chrono::Utc::now())?;
            pulled += merged.changed.len();
            for r in &merged.rejected {
                eprintln!(
                    "blotter: couldn't apply {} {} from the server: {}",
                    r.table, r.key, r.reason
                );
            }
            seq = page.seq;
            link::record_sync(planner.db().conn(), &l.epoch, seq, chrono::Utc::now())?;
            if !page.more {
                break;
            }
        }
        Ok(Synced {
            address,
            pushed,
            pulled,
            rejected,
            server_version: self.server_version.clone(),
            replaced: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use blotter_core::Database;
    use serde_json::json;

    use super::super::fake::*;
    use super::*;

    #[tokio::test]
    async fn two_devices_sync_through_the_server() {
        let server = Shared::new();
        let (laptop, desktop) = (device(&[LAN]), device(&[LAN]));
        let (mut a, mut b) = (Engine::new(server.clone()), Engine::new(server.clone()));
        let now = Instant::now();

        let id = run(&laptop, "create_item", json!({ "title": "Dentist" }))["id"]
            .as_str()
            .unwrap()
            .to_owned();
        let sent = a.sync_once(&laptop, "token", now).await.unwrap();
        assert!(sent.pushed > 0);
        assert_eq!(sent.server_version.as_deref(), Some("1.0.0"));
        assert_eq!(
            capture::pending_count(laptop.lock().unwrap().db().conn()).unwrap(),
            0,
            "the outbox is empty once the server has it"
        );

        let got = b.sync_once(&desktop, "token", now).await.unwrap();
        assert!(got.pulled > 0);
        assert_eq!(title(&desktop, &id).as_deref(), Some("Dentist"));
        let cursor = link::get(desktop.lock().unwrap().db().conn())
            .unwrap()
            .unwrap();
        assert_eq!(cursor.seq, server.with(|f| f.log.len() as i64));
        assert!(cursor.last_sync_at.is_some());

        run(
            &desktop,
            "update_item",
            json!({ "id": id, "title": "Dentist at 3" }),
        );
        b.sync_once(&desktop, "token", now).await.unwrap();
        a.sync_once(&laptop, "token", now).await.unwrap();
        assert_eq!(title(&laptop, &id).as_deref(), Some("Dentist at 3"));

        // Nothing new: nothing pushed, nothing changed.
        let idle = a.sync_once(&laptop, "token", now).await.unwrap();
        assert_eq!((idle.pushed, idle.pulled), (0, 0));
    }

    #[tokio::test]
    async fn fails_over_and_comes_back_to_the_lan() {
        let server = Shared::new();
        server.with(|f| f.down.insert(LAN.into()));
        let laptop = device(&[LAN, TAILNET]);
        let mut e = Engine::new(server.clone());
        let t0 = Instant::now();

        assert_eq!(
            e.sync_once(&laptop, "token", t0).await.unwrap().address,
            TAILNET
        );
        server.with(|f| f.down.remove(LAN));
        let soon = t0 + Duration::from_secs(60);
        assert_eq!(
            e.sync_once(&laptop, "token", soon).await.unwrap().address,
            TAILNET,
            "the LAN isn't tried again on every sync"
        );
        let later = t0 + RECHECK;
        assert_eq!(
            e.sync_once(&laptop, "token", later).await.unwrap().address,
            LAN
        );
    }

    #[tokio::test]
    async fn offline_says_why_and_mentions_tailscale() {
        let server = Shared::new();
        server.with(|f| {
            f.down.insert(LAN.into());
            f.down.insert(TAILNET.into());
        });
        let laptop = device(&[LAN, TAILNET]);
        let err = Engine::new(server)
            .sync_once(&laptop, "token", Instant::now())
            .await
            .unwrap_err();
        let Problem::Offline { tried, tailscale } = err else {
            panic!("{err:?}");
        };
        assert!(tailscale);
        assert_eq!(tried.len(), 2);
        assert_eq!(tried[0].0, LAN);
    }

    #[tokio::test]
    async fn another_server_at_an_address_is_skipped() {
        let server = Shared::new();
        server.with(|f| f.elsewhere.insert(LAN.into()));
        let both = device(&[LAN, TAILNET]);
        let mut e = Engine::new(server.clone());
        assert_eq!(
            e.sync_once(&both, "token", Instant::now())
                .await
                .unwrap()
                .address,
            TAILNET
        );

        let only = device(&[LAN]);
        let err = Engine::new(server)
            .sync_once(&only, "token", Instant::now())
            .await
            .unwrap_err();
        assert_eq!(err, Problem::Attention(Attention::WrongServer));
    }

    #[tokio::test]
    async fn a_version_mismatch_names_the_side_to_update() {
        let server = Shared::new();
        let laptop = device(&[LAN]);
        let newer = Versions {
            schema: Versions::CURRENT.schema + 1,
            ..Versions::CURRENT
        };
        server.with(|f| f.versions = newer);
        let err = Engine::new(server.clone())
            .sync_once(&laptop, "token", Instant::now())
            .await
            .unwrap_err();
        assert_eq!(
            err,
            Problem::Attention(Attention::UpdateBlotter { server: newer })
        );

        let older = Versions {
            protocol: Versions::CURRENT.protocol - 1,
            ..Versions::CURRENT
        };
        server.with(|f| f.versions = older);
        let err = Engine::new(server)
            .sync_once(&laptop, "token", Instant::now())
            .await
            .unwrap_err();
        assert_eq!(
            err,
            Problem::Attention(Attention::UpdateServer { server: older })
        );
    }

    #[tokio::test]
    async fn refusals_need_attention_or_a_retry() {
        let laptop = device(&[LAN]);
        for (code, want) in [
            (ErrorCode::Revoked, Problem::Attention(Attention::Revoked)),
            (
                ErrorCode::Unauthorized,
                Problem::Attention(Attention::Unauthorized),
            ),
            (
                ErrorCode::StaleEpoch,
                Problem::Attention(Attention::Rebootstrap),
            ),
            (ErrorCode::RateLimited, Problem::Retry("RateLimited".into())),
        ] {
            let server = Shared::new();
            server.with(|f| f.refuse = Some(code));
            let err = Engine::new(server)
                .sync_once(&laptop, "token", Instant::now())
                .await
                .unwrap_err();
            assert_eq!(err, want, "{code:?}");
        }
    }

    fn count(db: &Mutex<Planner>, title: &str) -> i64 {
        db.lock()
            .unwrap()
            .db()
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM items WHERE title = ?1",
                [title],
                |r| r.get(0),
            )
            .unwrap()
    }

    #[tokio::test]
    async fn a_replaced_planner_is_fetched_again_after_a_snapshot() {
        let dir =
            std::env::temp_dir().join(format!("blotter-refetch-{}", blotter_core::id::new_id()));
        let server = Shared::new();
        let laptop = device(&[LAN]);
        let mut e = Engine::new(server.clone()).snapshots_beside(dir.join("blotter.db"));
        let now = Instant::now();
        run(&laptop, "create_item", json!({ "title": "Dentist" }));
        e.sync_once(&laptop, "token", now).await.unwrap();

        run(&laptop, "create_item", json!({ "title": "Unsent" }));
        server.replace("create_item", json!({ "title": "Restored" }));
        let done = e.sync_once(&laptop, "token", now).await.unwrap();
        let replaced = done.replaced.expect("fetched again");
        assert!(replaced.unsent > 0);
        assert_eq!(
            [
                count(&laptop, "Dentist"),
                count(&laptop, "Restored"),
                count(&laptop, "Unsent")
            ],
            [1, 1, 0]
        );
        {
            let p = laptop.lock().unwrap();
            let conn = p.db().conn();
            let snap = blotter_core::snapshots::get(conn, &replaced.snapshot_id).unwrap();
            assert_eq!(snap.reason, Reason::BeforeSync);
            assert!(std::path::Path::new(&snap.path).exists());
            assert_eq!(
                link::get(conn).unwrap().unwrap().epoch,
                server.with(|f| f.epoch.clone())
            );
            assert_eq!(capture::pending_count(conn).unwrap(), 0);
        }
        let after = e.sync_once(&laptop, "token", now).await.unwrap();
        assert_eq!((after.pushed, after.replaced), (0, None));

        // Without a folder for the snapshot it only asks for attention.
        server.replace("create_item", json!({ "title": "Again" }));
        server.with(|f| f.epoch = "01JBEPOCH00000000000000003".into());
        let err = Engine::new(server)
            .sync_once(&laptop, "token", now)
            .await
            .unwrap_err();
        assert_eq!(err, Problem::Attention(Attention::Rebootstrap));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[tokio::test]
    async fn local_only_and_a_missing_token_do_nothing() {
        let server = Shared::new();
        let alone = Mutex::new(Planner::new(
            Database::open_in_memory().unwrap(),
            chrono_tz::UTC,
        ));
        let mut e = Engine::new(server.clone());
        assert_eq!(
            e.sync_once(&alone, "token", Instant::now()).await,
            Err(Problem::LocalOnly)
        );
        let laptop = device(&[LAN]);
        assert_eq!(
            e.sync_once(&laptop, "", Instant::now()).await,
            Err(Problem::Attention(Attention::Unauthorized))
        );
        assert!(server.with(|f| f.log.is_empty()));
    }

    /// Against a real Blotter Sync: `BLOTTER_LIVE_SYNC=http://127.0.0.1:7458`
    /// and `BLOTTER_LIVE_SYNC_CODE` from `blotter-sync pair`.
    #[tokio::test]
    #[ignore = "needs a Blotter Sync server and a pairing code"]
    async fn live_sync_against_blotter_sync() {
        use blotter_core::sync::api::{Paired, path};
        use futures_util::StreamExt;

        let (Ok(base), Ok(code)) = (
            std::env::var("BLOTTER_LIVE_SYNC"),
            std::env::var("BLOTTER_LIVE_SYNC_CODE"),
        ) else {
            return;
        };
        let http = super::super::client::Http::new().unwrap();
        let hello = http.hello(&base).await.unwrap();
        let planner = Planner::new(Database::open_in_memory().unwrap(), chrono_tz::UTC);
        let device_id = blotter_core::sync::device::id(planner.db().conn()).unwrap();
        let paired: Paired = reqwest::Client::new()
            .post(format!("{base}{}", path::PAIR))
            .header("x-blotter-protocol", Versions::CURRENT.protocol)
            .header("x-blotter-schema", Versions::CURRENT.schema)
            .json(&json!({ "code": code, "device_id": device_id, "name": "Live test" }))
            .send()
            .await
            .unwrap()
            .error_for_status()
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(paired.server_id, hello.server_id);
        link::set(
            planner.db().conn(),
            &link::Link {
                server_id: paired.server_id,
                addresses: vec![base.clone()],
                epoch: paired.epoch,
                seq: 0,
                paired_at: chrono::Utc::now(),
                last_sync_at: None,
            },
        )
        .unwrap();
        let db = Mutex::new(planner);
        let mut events = http
            .events(&base, &paired.token)
            .await
            .unwrap()
            .bytes_stream();

        let mut e = Engine::new(http);
        run(&db, "create_item", json!({ "title": "Live sync" }));
        let done = e
            .sync_once(&db, &paired.token, Instant::now())
            .await
            .unwrap();
        assert!(done.pushed > 0 && done.rejected == 0, "{done:?}");
        let seq = link::get(db.lock().unwrap().db().conn())
            .unwrap()
            .unwrap()
            .seq;
        assert!(seq > 0);

        let mut seen = String::new();
        while !seen.contains(&format!("\"seq\":{seq}")) {
            let chunk = tokio::time::timeout(Duration::from_secs(5), events.next())
                .await
                .expect("a notice within 5 s")
                .unwrap()
                .unwrap();
            seen.push_str(&String::from_utf8_lossy(&chunk));
        }
        let again = e
            .sync_once(&db, &paired.token, Instant::now())
            .await
            .unwrap();
        assert_eq!((again.pushed, again.pulled), (0, 0));
        let wrong = e.sync_once(&db, "not-a-token", Instant::now()).await;
        assert_eq!(wrong, Err(Problem::Attention(Attention::Unauthorized)));
    }
}

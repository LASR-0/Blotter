//! What Settings → Sync asks of the server and of this device's link
//! (roadmap §14.6, §14.9): pairing, the address list, testing an address and
//! Disconnect. Errors are sentences for the page. The token reaches the OS
//! keychain through the caller and never the database.

use std::path::Path;
use std::sync::{Mutex, MutexGuard};

use blotter_core::Planner;
use blotter_core::snapshots::{self, Reason};
use blotter_core::sync::api::{Bootstrap, ErrorCode, PairRequest, Versions};
use blotter_core::sync::{capture, device, join, link, merge, state};
use serde::{Deserialize, Serialize};

use super::address;
use super::client::{SyncError, Transport};
use super::engine::{Attention, mismatch};

const CLOSED: &str = "No pairing code is open on the server. Run `blotter-sync pair` there, \
                      then enter the code it prints.";

fn locked(db: &Mutex<Planner>) -> Result<MutexGuard<'_, Planner>, String> {
    db.lock()
        .map_err(|_| "The planner is unavailable after an earlier crash.".to_owned())
}

/// A core error as a sentence ("the device name can't be empty" → "The
/// device name can't be empty.").
fn sentence(err: &blotter_core::CoreError) -> String {
    let text = match err {
        blotter_core::CoreError::Invalid(m) => m.clone(),
        other => other.to_string(),
    };
    let mut chars = text.chars();
    let first = chars.next().map(|c| c.to_uppercase().collect::<String>());
    let mut out = first.unwrap_or_default() + chars.as_str();
    if !out.ends_with('.') {
        out.push('.');
    }
    out
}

fn versions(attention: &Attention) -> String {
    match attention {
        Attention::UpdateBlotter { .. } => {
            "This server needs a newer Blotter. Update Blotter on this device first.".into()
        }
        _ => "This server's Blotter Sync is too old for this Blotter. Update Blotter Sync first."
            .into(),
    }
}

/// What went wrong asking `address`, as a sentence.
fn said(address: &str, err: SyncError) -> String {
    match err {
        SyncError::Unreachable(m) => {
            let hint = if address::is_tailscale(address) {
                " Is Tailscale connected on this device?"
            } else {
                ""
            };
            format!("Couldn't reach {address}: {m}.{hint}")
        }
        SyncError::Insecure(m) | SyncError::Bad(m) => m,
        SyncError::Refused(body) => match body.code {
            ErrorCode::PairingClosed => CLOSED.into(),
            ErrorCode::BadCode => "That pairing code is wrong. Check it and try again.".into(),
            ErrorCode::CodeExpired => "That pairing code has expired or been used up. \
                                       Run `blotter-sync pair` for a new one."
                .into(),
            ErrorCode::RateLimited => "Too many tries. Wait a minute, then try again.".into(),
            ErrorCode::ProtocolMismatch | ErrorCode::SchemaMismatch => {
                versions(&mismatch(body.server.unwrap_or(Versions::CURRENT)))
            }
            _ => body.message,
        },
    }
}

/// What pairing did with the two planners (roadmap §14.7).
#[derive(Debug)]
pub enum Joined {
    /// Paired again with the server already linked: nothing to join.
    Again,
    /// The server had no planner: this device's goes up whole.
    Seeded,
    /// This device had none: it now has the server's.
    Fetched,
    /// Both have one. Nothing is linked until the user picks
    /// ([`finish_join`]) or gives up ([`abandon`]).
    Choose(PendingJoin),
}

impl Joined {
    pub fn as_str(&self) -> &'static str {
        match self {
            Joined::Again => "again",
            Joined::Seeded => "seeded",
            Joined::Fetched => "fetched",
            Joined::Choose(_) => "choose",
        }
    }
}

/// A pairing waiting on the user's choice, with the server's planner as it
/// was fetched; pulling from its seq afterwards catches up on the rest.
#[derive(Debug)]
pub struct PendingJoin {
    pub server_id: String,
    pub address: String,
    boot: Bootstrap,
}

/// How a device with a planner joins a server that has one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Choice {
    /// Use the server's planner; this device's goes to a snapshot.
    Server,
    /// Merge the two.
    Merge,
}

fn new_link(server_id: &str, address: &str, boot: &Bootstrap) -> link::Link {
    link::Link {
        server_id: server_id.to_owned(),
        addresses: vec![address.to_owned()],
        epoch: boot.epoch.clone(),
        seq: boot.seq,
        paired_at: blotter_core::time::truncate(chrono::Utc::now()),
        last_sync_at: None,
    }
}

fn report(merged: &blotter_core::sync::Merged) {
    for r in &merged.rejected {
        eprintln!(
            "blotter: couldn't apply {} {} from the server: {}",
            r.table, r.key, r.reason
        );
    }
}

/// Pairs this device: checks the address answers as a Blotter Sync this
/// build can sync with and has a code open, trades the code for a token
/// (handed to `save_token` with the server's id), names the device and
/// fetches the server's planner. Then, by what each side has: an empty
/// server gets this device's whole planner; an empty device takes the
/// server's; when both have one the user chooses ([`Joined::Choose`]).
/// Pairing again with the server already linked (after a revoke, or a lost
/// token) keeps the link and adds the address; another server needs a
/// Disconnect first.
pub async fn pair<T: Transport>(
    t: &T,
    db: &Mutex<Planner>,
    address: &str,
    code: &str,
    name: &str,
    save_token: impl FnOnce(&str, &str) -> Result<(), String>,
) -> Result<Joined, String> {
    let address = address::normalize(address)?;
    let code = code.trim();
    if code.is_empty() {
        return Err("Enter the pairing code that `blotter-sync pair` printed.".into());
    }
    let (existing, me) = {
        let p = locked(db)?;
        let conn = p.db().conn();
        let me = device::set_name(conn, name).map_err(|e| sentence(&e))?;
        (link::get(conn).map_err(|e| sentence(&e))?, me)
    };

    let hello = t.hello(&address).await.map_err(|e| said(&address, e))?;
    if hello.versions != Versions::CURRENT {
        return Err(versions(&mismatch(hello.versions)));
    }
    if existing
        .as_ref()
        .is_some_and(|l| l.server_id != hello.server_id)
    {
        return Err(
            "This device already syncs with another server. Disconnect from it first.".into(),
        );
    }
    if !hello.pairing_open {
        return Err(CLOSED.into());
    }
    let req = PairRequest {
        code: code.to_owned(),
        device_id: me.id,
        name: me.name.unwrap_or_default(),
    };
    let paired = t
        .pair(&address, &req)
        .await
        .map_err(|e| said(&address, e))?;
    let now = blotter_core::time::truncate(chrono::Utc::now());

    if let Some(mut l) = existing {
        save_token(&paired.server_id, &paired.token)?;
        if !l.addresses.contains(&address) {
            l.addresses.push(address);
        }
        if l.epoch != paired.epoch {
            l.epoch = paired.epoch;
            l.seq = 0;
        }
        l.paired_at = now;
        link::set(locked(db)?.db().conn(), &l).map_err(|e| sentence(&e))?;
        return Ok(Joined::Again);
    }

    let boot = match t.bootstrap(&address, &paired.token).await {
        Ok(boot) => boot,
        Err(e) => {
            // Not joined, so the server needn't keep this device.
            let _ = t.disconnect(&address, &paired.token).await;
            return Err(said(&address, e));
        }
    };
    if let Err(e) = save_token(&paired.server_id, &paired.token) {
        // Without the token this device can't sync, so the server needn't
        // keep it either.
        let _ = t.disconnect(&address, &paired.token).await;
        return Err(e);
    }

    let mut p = locked(db)?;
    let conn = p.db().conn();
    let mine = state::all(conn).map_err(|e| sentence(&e))?;
    let l = new_link(&paired.server_id, &address, &boot);
    let joined = if !join::has_planner(&boot.rows) {
        capture::queue_all(conn).map_err(|e| sentence(&e))?;
        report(&merge(p.db_mut().conn_mut(), &boot.rows, now).map_err(|e| sentence(&e))?);
        Joined::Seeded
    } else if !join::has_planner(&mine) {
        report(
            &join::replace_with(p.db_mut().conn_mut(), &boot.rows, now)
                .map_err(|e| sentence(&e))?,
        );
        Joined::Fetched
    } else {
        return Ok(Joined::Choose(PendingJoin {
            server_id: paired.server_id,
            address,
            boot,
        }));
    };
    link::set(p.db().conn(), &l).map_err(|e| sentence(&e))?;
    Ok(joined)
}

/// Joins as the user chose, after a snapshot of this device's planner, and
/// links the device.
pub fn finish_join(
    db: &Mutex<Planner>,
    pending: PendingJoin,
    choice: Choice,
    db_path: &Path,
) -> Result<(), String> {
    let now = chrono::Utc::now();
    let mut p = locked(db)?;
    let conn = p.db().conn();
    let dir = snapshots::dir(conn, db_path).map_err(|e| sentence(&e))?;
    let note = match choice {
        Choice::Server => "Before using the home server’s planner",
        Choice::Merge => "Before merging with the home server’s planner",
    };
    snapshots::create(conn, &dir, Reason::BeforeSync, Some(note.into()), now)
        .map_err(|e| sentence(&e))?;
    let conn = p.db_mut().conn_mut();
    let merged = match choice {
        Choice::Server => join::replace_with(conn, &pending.boot.rows, now),
        Choice::Merge => join::merge_into(conn, &pending.boot.rows, now),
    }
    .map_err(|e| sentence(&e))?;
    report(&merged);
    let l = new_link(&pending.server_id, &pending.address, &pending.boot);
    link::set(p.db().conn(), &l).map_err(|e| sentence(&e))?;
    Ok(())
}

/// Gives up on a pairing before joining: the server forgets this device.
pub async fn abandon<T: Transport>(t: &T, pending: &PendingJoin, token: Option<&str>) {
    if let Some(token) = token {
        let _ = t.disconnect(&pending.address, token).await;
    }
}

/// Replaces the address list with `typed`, read as typed and without
/// repeats, in the order given.
pub fn set_addresses(db: &Mutex<Planner>, typed: &[String]) -> Result<Vec<String>, String> {
    let mut out: Vec<String> = Vec::new();
    for a in typed {
        let a = address::normalize(a)?;
        if !out.contains(&a) {
            out.push(a);
        }
    }
    if out.is_empty() {
        return Err("Keep at least one address, or disconnect instead.".into());
    }
    let p = locked(db)?;
    let conn = p.db().conn();
    if link::get(conn).map_err(|e| sentence(&e))?.is_none() {
        return Err("This device isn't paired with a server.".into());
    }
    link::set_addresses(conn, &out).map_err(|e| sentence(&e))?;
    Ok(out)
}

/// What testing one address found.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AddressCheck {
    pub address: String,
    pub ok: bool,
    pub message: String,
}

/// Asks `typed` for its hello: whether it answers, as `server_id` if this
/// device is paired, with versions this build can sync with.
pub async fn test_address<T: Transport>(
    t: &T,
    typed: &str,
    server_id: Option<&str>,
) -> Result<AddressCheck, String> {
    let address = address::normalize(typed)?;
    let (ok, message) = match t.hello(&address).await {
        Err(e) => (false, said(&address, e)),
        Ok(h) if server_id.is_some_and(|id| id != h.server_id) => (
            false,
            "That's a different Blotter Sync server from the one this device syncs with.".into(),
        ),
        Ok(h) if h.versions != Versions::CURRENT => (false, versions(&mismatch(h.versions))),
        Ok(h) => (true, format!("{} {} answered.", h.name, h.version)),
    };
    Ok(AddressCheck {
        address,
        ok,
        message,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Disconnected {
    /// The server revoked this device's token. When it couldn't be reached
    /// it still lists the device, until revoked there.
    pub revoked: bool,
    /// The server this device was linked to, whose token the caller removes.
    #[serde(skip)]
    pub server_id: Option<String>,
}

/// Back to Local only, keeping the planner: revokes this device's token at
/// the first address answering as its server, then forgets the link.
/// Unsent changes stay queued for whichever server is paired next.
pub async fn disconnect<T: Transport>(
    t: &T,
    db: &Mutex<Planner>,
    token: Option<&str>,
) -> Result<Disconnected, String> {
    let Some(l) = link::get(locked(db)?.db().conn()).map_err(|e| sentence(&e))? else {
        return Ok(Disconnected {
            revoked: true,
            server_id: None,
        });
    };
    let mut revoked = false;
    if let Some(token) = token.filter(|t| !t.is_empty()) {
        for a in &l.addresses {
            if !t.hello(a).await.is_ok_and(|h| h.server_id == l.server_id) {
                continue;
            }
            match t.disconnect(a, token).await {
                // Already gone is as good as revoked.
                Ok(()) => revoked = true,
                Err(SyncError::Refused(b))
                    if matches!(b.code, ErrorCode::Unauthorized | ErrorCode::Revoked) =>
                {
                    revoked = true;
                }
                Err(_) => continue,
            }
            break;
        }
    }
    link::clear(locked(db)?.db().conn()).map_err(|e| sentence(&e))?;
    Ok(Disconnected {
        revoked,
        server_id: Some(l.server_id),
    })
}

/// This device's name and the server it's linked to, for the page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SyncInfo {
    pub device_name: Option<String>,
    pub server: Option<ServerLink>,
    /// A pairing waiting on Use the server's planner or Merge: its address.
    pub joining: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ServerLink {
    pub server_id: String,
    pub addresses: Vec<String>,
    pub paired_at: String,
}

pub fn info(db: &Mutex<Planner>) -> Result<SyncInfo, String> {
    let p = locked(db)?;
    let conn = p.db().conn();
    let me = device::get(conn).map_err(|e| sentence(&e))?;
    let l = link::get(conn).map_err(|e| sentence(&e))?;
    Ok(SyncInfo {
        device_name: me.name,
        joining: None,
        server: l.map(|l| ServerLink {
            server_id: l.server_id,
            addresses: l.addresses,
            paired_at: blotter_core::time::fmt_instant(l.paired_at),
        }),
    })
}

/// The linked server's id, if any.
pub fn linked_server(db: &Mutex<Planner>) -> Result<Option<String>, String> {
    let l = link::get(locked(db)?.db().conn()).map_err(|e| sentence(&e))?;
    Ok(l.map(|l| l.server_id))
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex as StdMutex;

    use serde_json::json;

    use super::super::fake::*;
    use super::*;

    /// Pairs `db` at `address` with `code`, keeping the token it saves.
    async fn pair_with(
        server: &Shared,
        db: &Mutex<Planner>,
        address: &str,
        code: &str,
    ) -> (Result<Joined, String>, Option<String>) {
        let saved = StdMutex::new(None);
        let out = pair(server, db, address, code, " Laptop ", |id, token| {
            assert_eq!(id, SERVER);
            *saved.lock().unwrap() = Some(token.to_owned());
            Ok(())
        })
        .await;
        (out, saved.into_inner().unwrap())
    }

    fn linked(db: &Mutex<Planner>) -> Option<link::Link> {
        link::get(db.lock().unwrap().db().conn()).unwrap()
    }

    fn unsent(db: &Mutex<Planner>) -> u64 {
        capture::pending_count(db.lock().unwrap().db().conn()).unwrap()
    }

    #[tokio::test]
    async fn pairing_saves_the_token_names_the_device_and_queues_the_planner() {
        let server = Shared::new();
        server.with(|f| f.code = Some(CODE.into()));
        let db = alone();
        run(&db, "create_item", json!({ "title": "Dentist" }));
        {
            let p = db.lock().unwrap();
            let conn = p.db().conn();
            capture::acknowledge(conn, &capture::pending(conn).unwrap()).unwrap();
        }
        assert_eq!(unsent(&db), 0, "as if an earlier server had it all");

        let (out, token) = pair_with(&server, &db, "192.168.1.10", "abcd-efgh").await;
        assert!(matches!(out.unwrap(), Joined::Seeded));
        let l = linked(&db).unwrap();
        assert_eq!((l.server_id.as_str(), l.epoch.as_str()), (SERVER, EPOCH));
        assert_eq!(l.addresses, [LAN]);

        let me = device::get(db.lock().unwrap().db().conn()).unwrap();
        assert_eq!(me.name.as_deref(), Some("Laptop"));
        let (sent, name) = server.with(|f| f.devices[&me.id].clone());
        assert_eq!((token, name.as_str()), (Some(sent), "Laptop"));
        assert!(unsent(&db) > 0, "the whole planner goes to the new server");
        assert!(server.with(|f| f.code.is_none()), "the code is used up");
    }

    #[tokio::test]
    async fn pairing_refusals_say_what_to_do_and_change_nothing() {
        let server = Shared::new();
        let db = alone();
        type Setup = Box<dyn Fn(&mut Fake)>;
        let cases: [(&str, &str, &str, Setup); 6] = [
            (LAN, CODE, "blotter-sync pair", Box::new(|_| {})),
            (
                LAN,
                "WRONG-CODE",
                "pairing code is wrong",
                Box::new(|f| f.code = Some(CODE.into())),
            ),
            (
                "http://203.0.113.5",
                CODE,
                "Use https:// for 203.0.113.5",
                Box::new(|_| {}),
            ),
            (
                LAN,
                CODE,
                "Couldn't reach http://192.168.1.10:7458",
                Box::new(|f| {
                    f.down.insert(LAN.into());
                }),
            ),
            (
                LAN,
                CODE,
                "Update Blotter on this device",
                Box::new(|f| {
                    f.down.clear();
                    f.versions.schema += 1;
                }),
            ),
            (
                LAN,
                " ",
                "Enter the pairing code",
                Box::new(|f| f.versions = Versions::CURRENT),
            ),
        ];
        for (address, code, want, setup) in cases {
            server.with(|f| setup(f));
            let (out, token) = pair_with(&server, &db, address, code).await;
            let err = out.unwrap_err();
            assert!(err.contains(want), "{want}: {err}");
            assert_eq!((token, linked(&db)), (None, None), "{want}");
        }

        server.with(|f| f.code = Some(CODE.into()));
        let err = pair(&server, &db, LAN, CODE, "  ", |_, _| Ok(()))
            .await
            .unwrap_err();
        assert_eq!(err, "The device name can't be empty.");
    }

    #[tokio::test]
    async fn pairing_again_keeps_the_link_and_another_server_waits_for_disconnect() {
        let server = Shared::new();
        server.with(|f| f.code = Some(CODE.into()));
        let db = device(&[LAN]);
        {
            let p = db.lock().unwrap();
            link::record_sync(p.db().conn(), EPOCH, 5, chrono::Utc::now()).unwrap();
        }
        let (out, token) = pair_with(&server, &db, TAILNET, CODE).await;
        assert!(matches!(out.unwrap(), Joined::Again));
        let l = linked(&db).unwrap();
        assert!(token.is_some());
        assert_eq!(l.addresses, [LAN, TAILNET]);
        assert_eq!(l.seq, 5, "the cursor stays");

        server.with(|f| {
            f.code = Some(CODE.into());
            f.elsewhere.insert(LAN.into());
        });
        let (out, token) = pair_with(&server, &db, LAN, CODE).await;
        assert!(out.unwrap_err().contains("Disconnect from it first"));
        assert_eq!(token, None);
    }

    fn titles(db: &Mutex<Planner>) -> Vec<String> {
        let p = db.lock().unwrap();
        let mut t: Vec<String> = p
            .db()
            .conn()
            .prepare("SELECT title FROM items")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .map(Result::unwrap)
            .collect();
        t.sort();
        t
    }

    fn scratch() -> std::path::PathBuf {
        std::env::temp_dir()
            .join(format!("blotter-join-{}", blotter_core::id::new_id()))
            .join("blotter.db")
    }

    #[tokio::test]
    async fn an_empty_device_takes_the_servers_planner() {
        let server = Shared::new();
        server.run("create_item", json!({ "title": "From the server" }));
        server.with(|f| f.code = Some(CODE.into()));
        let db = alone();
        let (out, _) = pair_with(&server, &db, LAN, CODE).await;
        assert!(matches!(out.unwrap(), Joined::Fetched));
        assert_eq!(titles(&db), ["From the server"]);
        assert_eq!(unsent(&db), 0, "nothing of its own to send");
        let l = linked(&db).unwrap();
        assert_eq!(l.seq, server.with(|f| f.log.len() as i64));
    }

    #[tokio::test]
    async fn both_with_a_planner_wait_for_the_users_choice() {
        let server = Shared::new();
        server.run("create_item", json!({ "title": "From the server" }));
        let path = scratch();

        for (choice, want) in [
            (Choice::Server, vec!["From the server"]),
            (Choice::Merge, vec!["From the server", "Only here"]),
        ] {
            server.with(|f| f.code = Some(CODE.into()));
            let db = alone();
            run(&db, "create_item", json!({ "title": "Only here" }));
            let (out, token) = pair_with(&server, &db, LAN, CODE).await;
            let Ok(Joined::Choose(pending)) = out else {
                panic!("{out:?}");
            };
            assert!(token.is_some());
            assert_eq!(linked(&db), None, "nothing linked before the choice");

            finish_join(&db, pending, choice, &path).unwrap();
            assert_eq!(titles(&db), want, "{choice:?}");
            assert!(linked(&db).is_some());
            let p = db.lock().unwrap();
            let snaps = snapshots::list(p.db().conn()).unwrap();
            assert_eq!(snaps.len(), 1);
            assert_eq!(snaps[0].reason, Reason::BeforeSync);
            let queued = capture::pending_count(p.db().conn()).unwrap();
            assert_eq!(queued > 0, choice == Choice::Merge, "{choice:?}");
        }
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[tokio::test]
    async fn giving_up_before_joining_unpairs() {
        let server = Shared::new();
        server.run("create_item", json!({ "title": "From the server" }));
        server.with(|f| f.code = Some(CODE.into()));
        let db = alone();
        run(&db, "create_item", json!({ "title": "Only here" }));
        let (out, token) = pair_with(&server, &db, LAN, CODE).await;
        let Ok(Joined::Choose(pending)) = out else {
            panic!("{out:?}");
        };
        abandon(&server, &pending, token.as_deref()).await;
        assert_eq!(server.with(|f| f.revoked.len()), 1);
        assert_eq!(titles(&db), ["Only here"]);
        assert_eq!(linked(&db), None);
    }

    /// Pairing answers; fetching the planner doesn't.
    struct FailingBoot(Shared);

    impl Transport for FailingBoot {
        async fn hello(&self, b: &str) -> Result<blotter_core::sync::api::Hello, SyncError> {
            self.0.hello(b).await
        }
        async fn pair(
            &self,
            b: &str,
            r: &PairRequest,
        ) -> Result<blotter_core::sync::api::Paired, SyncError> {
            self.0.pair(b, r).await
        }
        async fn disconnect(&self, b: &str, t: &str) -> Result<(), SyncError> {
            self.0.disconnect(b, t).await
        }
        async fn bootstrap(&self, _: &str, _: &str) -> Result<Bootstrap, SyncError> {
            Err(SyncError::Unreachable("timed out".into()))
        }
        async fn push(
            &self,
            b: &str,
            t: &str,
            r: &blotter_core::sync::api::PushRequest,
        ) -> Result<blotter_core::sync::api::PushResponse, SyncError> {
            self.0.push(b, t, r).await
        }
        async fn pull(
            &self,
            b: &str,
            t: &str,
            e: &str,
            s: i64,
        ) -> Result<blotter_core::sync::api::PullResponse, SyncError> {
            self.0.pull(b, t, e, s).await
        }
    }

    #[tokio::test]
    async fn a_failed_fetch_unpairs_and_saves_no_token() {
        let server = Shared::new();
        server.with(|f| f.code = Some(CODE.into()));
        let db = alone();
        let saved = StdMutex::new(None);
        let err = pair(
            &FailingBoot(server.clone()),
            &db,
            LAN,
            CODE,
            "Laptop",
            |_, t| {
                *saved.lock().unwrap() = Some(t.to_owned());
                Ok(())
            },
        )
        .await
        .unwrap_err();
        assert!(err.contains("timed out"), "{err}");
        assert_eq!(saved.into_inner().unwrap(), None);
        assert_eq!(server.with(|f| f.revoked.len()), 1);
    }

    #[tokio::test]
    async fn a_token_the_keychain_wont_keep_unpairs() {
        let server = Shared::new();
        server.with(|f| f.code = Some(CODE.into()));
        let db = alone();
        let err = pair(&server, &db, LAN, CODE, "Laptop", |_, _| {
            Err("couldn't save the server's token: no keychain".into())
        })
        .await
        .unwrap_err();
        assert!(err.contains("no keychain"), "{err}");
        assert_eq!(server.with(|f| f.revoked.len()), 1);
        assert_eq!(linked(&db), None);
    }

    #[test]
    fn addresses_are_read_as_typed_without_repeats() {
        let db = device(&[LAN]);
        let typed = [
            "https://blotter.tail1234.ts.net/".to_owned(),
            "192.168.1.10".to_owned(),
            "http://192.168.1.10:7458".to_owned(),
        ];
        assert_eq!(set_addresses(&db, &typed).unwrap(), [TAILNET, LAN]);
        assert_eq!(linked(&db).unwrap().addresses, [TAILNET, LAN]);
        assert!(
            set_addresses(&db, &[])
                .unwrap_err()
                .contains("at least one")
        );
        assert!(
            set_addresses(&db, &["http://8.8.8.8".into()])
                .unwrap_err()
                .contains("https://")
        );
        assert!(
            set_addresses(&alone(), &typed)
                .unwrap_err()
                .contains("isn't paired")
        );
    }

    #[tokio::test]
    async fn testing_an_address_says_who_answered() {
        let server = Shared::new();
        let ok = test_address(&server, "192.168.1.10", Some(SERVER))
            .await
            .unwrap();
        assert_eq!(
            ok,
            AddressCheck {
                address: LAN.into(),
                ok: true,
                message: "Blotter Sync 1.0.0 answered.".into(),
            }
        );
        server.with(|f| {
            f.elsewhere.insert(LAN.into());
            f.down.insert(TAILNET.into());
        });
        let other = test_address(&server, LAN, Some(SERVER)).await.unwrap();
        assert!(!other.ok && other.message.contains("different"));
        assert!(
            test_address(&server, LAN, None).await.unwrap().ok,
            "before pairing any server will do"
        );
        let down = test_address(&server, TAILNET, Some(SERVER)).await.unwrap();
        assert!(!down.ok && down.message.contains("Is Tailscale connected"));
        assert!(test_address(&server, "", None).await.is_err());
    }

    #[tokio::test]
    async fn disconnect_revokes_the_token_and_keeps_the_planner() {
        let server = Shared::new();
        server.with(|f| {
            f.code = Some(CODE.into());
            f.elsewhere.insert(LAN.into());
        });
        let db = alone();
        let id = run(&db, "create_item", json!({ "title": "Dentist" }))["id"]
            .as_str()
            .unwrap()
            .to_owned();
        let (out, token) = pair_with(&server, &db, TAILNET, CODE).await;
        out.unwrap();
        set_addresses(&db, &[LAN.into(), TAILNET.into()]).unwrap();

        let done = disconnect(&server, &db, token.as_deref()).await.unwrap();
        assert_eq!(
            done,
            Disconnected {
                revoked: true,
                server_id: Some(SERVER.into())
            },
            "revoked at the Tailscale address, past the other server on the LAN"
        );
        assert_eq!(linked(&db), None);
        assert_eq!(title(&db, &id).as_deref(), Some("Dentist"));
        assert_eq!(server.with(|f| f.revoked.len()), 1);
    }

    #[tokio::test]
    async fn disconnect_works_offline_and_says_the_server_still_lists_the_device() {
        let server = Shared::new();
        server.with(|f| f.down.insert(LAN.into()));
        let db = device(&[LAN]);
        let done = disconnect(&server, &db, Some("token")).await.unwrap();
        assert!(!done.revoked);
        assert_eq!(linked(&db), None);

        let again = disconnect(&server, &db, None).await.unwrap();
        assert_eq!((again.revoked, again.server_id), (true, None));
    }

    /// Against a real Blotter Sync, as the engine's live test:
    /// `BLOTTER_LIVE_SYNC=http://127.0.0.1:7458` and `BLOTTER_LIVE_SYNC_CODE`.
    #[tokio::test]
    #[ignore = "needs a Blotter Sync server and a pairing code"]
    async fn live_pair_test_and_disconnect() {
        let (Ok(base), Ok(code)) = (
            std::env::var("BLOTTER_LIVE_SYNC"),
            std::env::var("BLOTTER_LIVE_SYNC_CODE"),
        ) else {
            return;
        };
        let http = super::super::client::Http::new().unwrap();
        let db = alone();
        run(&db, "create_item", json!({ "title": "Live pairing" }));
        let saved = StdMutex::new(None);
        let joined = pair(&http, &db, &base, &code, "Live pairing", |_, token| {
            *saved.lock().unwrap() = Some(token.to_owned());
            Ok(())
        })
        .await
        .unwrap();
        let dir = std::env::temp_dir().join(format!("blotter-live-{}", blotter_core::id::new_id()));
        if let Joined::Choose(pending) = joined {
            finish_join(&db, pending, Choice::Merge, &dir.join("blotter.db")).unwrap();
        }
        let l = linked(&db).unwrap();
        let token = saved.into_inner().unwrap().unwrap();
        let address = &l.addresses[0];
        assert!(unsent(&db) > 0);

        let check = test_address(&http, address, Some(&l.server_id))
            .await
            .unwrap();
        assert!(check.ok, "{check:?}");
        let used = pair(&http, &db, address, &code, "Live pairing", |_, _| Ok(()))
            .await
            .unwrap_err();
        assert!(used.contains("No pairing code is open"), "{used}");

        let done = disconnect(&http, &db, Some(&token)).await.unwrap();
        assert!(done.revoked);
        assert_eq!(linked(&db), None);
        let after = http.pull(address, &token, &l.epoch, 0).await.unwrap_err();
        assert!(
            matches!(&after, SyncError::Refused(b) if b.code == ErrorCode::Revoked),
            "{after:?}"
        );
    }

    /// Runs Blotter Sync's admin command on the live server's data dir.
    fn admin(args: &[&str]) -> String {
        let bin = std::env::var("BLOTTER_LIVE_SYNC_ADMIN").unwrap();
        let data = std::env::var("BLOTTER_LIVE_SYNC_DATA").unwrap();
        let out = std::process::Command::new(bin)
            .arg("--data-dir")
            .arg(data)
            .args(args)
            .output()
            .unwrap();
        assert!(out.status.success(), "{out:?}");
        String::from_utf8(out.stdout).unwrap()
    }

    /// A restore on a real Blotter Sync: the engine snapshots this device
    /// and fetches the restored planner. As the live test above, plus
    /// `BLOTTER_LIVE_SYNC_ADMIN` (the `blotter-sync` binary) and
    /// `BLOTTER_LIVE_SYNC_DATA` (the server's data dir).
    #[tokio::test]
    #[ignore = "needs a Blotter Sync server, its binary and a pairing code"]
    async fn live_refetch_after_a_restore() {
        use std::time::Instant;

        use super::super::engine::Engine;

        let (Ok(base), Ok(code)) = (
            std::env::var("BLOTTER_LIVE_SYNC"),
            std::env::var("BLOTTER_LIVE_SYNC_CODE"),
        ) else {
            return;
        };
        let http = super::super::client::Http::new().unwrap();
        let db = alone();
        let path = scratch();
        let saved = StdMutex::new(None);
        let joined = pair(&http, &db, &base, &code, "Live restore", |_, token| {
            *saved.lock().unwrap() = Some(token.to_owned());
            Ok(())
        })
        .await
        .unwrap();
        assert!(
            matches!(joined, Joined::Fetched | Joined::Seeded),
            "{joined:?}"
        );
        let token = saved.into_inner().unwrap().unwrap();
        let mut e = Engine::new(http.clone()).snapshots_beside(path.clone());
        e.sync_once(&db, &token, Instant::now()).await.unwrap();

        let taken = admin(&["snapshot"]);
        let id = taken.split_whitespace().nth(1).unwrap().to_owned();
        run(
            &db,
            "create_item",
            json!({ "title": "Pushed, then restored away" }),
        );
        e.sync_once(&db, &token, Instant::now()).await.unwrap();
        run(&db, "create_item", json!({ "title": "Never pushed" }));
        admin(&["restore", &id]);

        let done = e.sync_once(&db, &token, Instant::now()).await.unwrap();
        let replaced = done.replaced.expect("fetched again");
        assert!(replaced.unsent > 0);
        let t = titles(&db);
        assert!(!t.iter().any(|t| t == "Never pushed"), "{t:?}");
        assert!(
            !t.iter().any(|t| t == "Pushed, then restored away"),
            "{t:?}"
        );
        let again = e.sync_once(&db, &token, Instant::now()).await.unwrap();
        assert_eq!(again.replaced, None);

        disconnect(&http, &db, Some(&token)).await.unwrap();
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }
}

//! What Settings → Sync asks of the server and of this device's link
//! (roadmap §14.6, §14.9): pairing, the address list, testing an address and
//! Disconnect. Errors are sentences for the page. The token reaches the OS
//! keychain through the caller and never the database.

use std::sync::{Mutex, MutexGuard};

use blotter_core::Planner;
use blotter_core::sync::api::{ErrorCode, PairRequest, Versions};
use blotter_core::sync::{capture, device, link};
use serde::Serialize;

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

/// Pairs this device: checks the address answers as a Blotter Sync this
/// build can sync with and has a code open, trades the code for a token
/// (handed to `save_token` with the server's id), names the device and
/// links it. A first link queues the whole planner, so the next sync sends
/// all of it. Pairing again with the server already linked (after a revoke,
/// or a lost token) keeps the link and adds the address; another server
/// needs a Disconnect first.
pub async fn pair<T: Transport>(
    t: &T,
    db: &Mutex<Planner>,
    address: &str,
    code: &str,
    name: &str,
    save_token: impl FnOnce(&str, &str) -> Result<(), String>,
) -> Result<link::Link, String> {
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
    save_token(&paired.server_id, &paired.token)?;

    let p = locked(db)?;
    let conn = p.db().conn();
    let now = blotter_core::time::truncate(chrono::Utc::now());
    let l = match existing {
        Some(mut l) => {
            if !l.addresses.contains(&address) {
                l.addresses.push(address);
            }
            if l.epoch != paired.epoch {
                l.epoch = paired.epoch;
                l.seq = 0;
            }
            l.paired_at = now;
            l
        }
        None => {
            capture::queue_all(conn).map_err(|e| sentence(&e))?;
            link::Link {
                server_id: paired.server_id,
                addresses: vec![address],
                epoch: paired.epoch,
                seq: 0,
                paired_at: now,
                last_sync_at: None,
            }
        }
    };
    link::set(conn, &l).map_err(|e| sentence(&e))?;
    Ok(l)
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
    ) -> (Result<link::Link, String>, Option<String>) {
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
        let l = out.unwrap();
        assert_eq!((l.server_id.as_str(), l.epoch.as_str()), (SERVER, EPOCH));
        assert_eq!(l.addresses, [LAN]);
        assert_eq!(linked(&db), Some(l));
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
        let l = out.unwrap();
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
        let l = pair(&http, &db, &base, &code, "Live pairing", |_, token| {
            *saved.lock().unwrap() = Some(token.to_owned());
            Ok(())
        })
        .await
        .unwrap();
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
}

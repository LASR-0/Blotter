//! The device token from pairing, in the OS keychain beside the AI keys
//! (roadmap §14.6). It never touches the database, the logs or the webview.
//! Every call goes through [`keychain::call`]: the engine and the pairing
//! commands are async.

use keyring::{Entry, Error};

use crate::keychain;

const SERVICE: &str = "io.github.lasr0.Blotter";

fn entry(server_id: &str) -> Result<Entry, String> {
    Entry::new(SERVICE, &format!("sync-server:{server_id}"))
        .map_err(|e| format!("the system keychain isn't available: {e}"))
}

pub fn set(server_id: &str, token: &str) -> Result<(), String> {
    keychain::call(|| {
        entry(server_id)?
            .set_password(token)
            .map_err(|e| format!("couldn't save the server's token: {e}"))
    })
}

pub fn get(server_id: &str) -> Result<Option<String>, String> {
    keychain::call(|| match entry(server_id)?.get_password() {
        Ok(t) => Ok(Some(t)),
        Err(Error::NoEntry) => Ok(None),
        Err(e) => Err(format!("couldn't read the server's token: {e}")),
    })
}

pub fn delete(server_id: &str) -> Result<(), String> {
    keychain::call(|| match entry(server_id)?.delete_credential() {
        Ok(()) | Err(Error::NoEntry) => Ok(()),
        Err(e) => Err(format!("couldn't remove the server's token: {e}")),
    })
}

#[cfg(test)]
mod tests {
    /// Against this machine's keychain, from async code as the engine
    /// calls it: `cargo test -p blotter live_keychain -- --ignored`.
    #[tokio::test]
    #[ignore = "uses the OS keychain"]
    async fn live_keychain_from_async_code() {
        let id = "test-01M0000000000000000000000";
        super::set(id, "a-token").unwrap();
        assert_eq!(super::get(id).unwrap().as_deref(), Some("a-token"));
        super::delete(id).unwrap();
        assert_eq!(super::get(id).unwrap(), None);
    }
}

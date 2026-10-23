//! The device token from pairing, in the OS keychain beside the AI keys
//! (roadmap §14.6). It never touches the database, the logs or the webview.

use keyring::{Entry, Error};

const SERVICE: &str = "io.github.lasr0.Blotter";

fn entry(server_id: &str) -> Result<Entry, String> {
    Entry::new(SERVICE, &format!("sync-server:{server_id}"))
        .map_err(|e| format!("the system keychain isn't available: {e}"))
}

pub fn get(server_id: &str) -> Result<Option<String>, String> {
    match entry(server_id)?.get_password() {
        Ok(t) => Ok(Some(t)),
        Err(Error::NoEntry) => Ok(None),
        Err(e) => Err(format!("couldn't read the server's token: {e}")),
    }
}

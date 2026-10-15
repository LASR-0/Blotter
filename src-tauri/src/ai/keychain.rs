//! API keys in the OS keychain (Windows Credential Manager, the Secret
//! Service on Linux, Keychain on macOS), one entry per provider id. Keys
//! never touch the database, the logs or the webview.

use keyring::{Entry, Error};

use crate::commands::CommandError;

const SERVICE: &str = "io.github.lasr0.Blotter";

fn entry(provider_id: &str) -> Result<Entry, CommandError> {
    Entry::new(SERVICE, &format!("ai-provider:{provider_id}"))
        .map_err(|e| CommandError::internal(format!("the system keychain isn't available: {e}")))
}

pub fn set(provider_id: &str, key: &str) -> Result<(), CommandError> {
    entry(provider_id)?
        .set_password(key)
        .map_err(|e| CommandError::internal(format!("couldn't save the key: {e}")))
}

pub fn get(provider_id: &str) -> Result<Option<String>, CommandError> {
    match entry(provider_id)?.get_password() {
        Ok(k) => Ok(Some(k)),
        Err(Error::NoEntry) => Ok(None),
        Err(e) => Err(CommandError::internal(format!(
            "couldn't read the key: {e}"
        ))),
    }
}

pub fn delete(provider_id: &str) -> Result<(), CommandError> {
    match entry(provider_id)?.delete_credential() {
        Ok(()) | Err(Error::NoEntry) => Ok(()),
        Err(e) => Err(CommandError::internal(format!(
            "couldn't remove the key: {e}"
        ))),
    }
}

/// The last four characters, shown as "••••3f9a" so the user can tell
/// which key is saved.
pub fn hint(key: &str) -> String {
    let chars: Vec<char> = key.chars().collect();
    chars[chars.len().saturating_sub(4)..].iter().collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn hint_is_the_last_four() {
        assert_eq!(super::hint("sk-ant-abc123f9a"), "3f9a");
        assert_eq!(super::hint("ab"), "ab");
    }
}

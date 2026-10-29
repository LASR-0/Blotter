//! Reaching the OS keychain from any thread. On Linux `keyring` talks to
//! the Secret Service through zbus and blocks on a runtime of its own,
//! which panics on a thread already driving tokio ("Cannot start a runtime
//! from within a runtime"), as every async command and the sync engine are.
//! Each call gets a thread of its own, outside any runtime.

/// Runs `f` on a fresh thread and waits for it; a panic there carries on
/// here.
pub fn call<T: Send>(f: impl FnOnce() -> T + Send) -> T {
    std::thread::scope(|s| {
        s.spawn(f)
            .join()
            .unwrap_or_else(|p| std::panic::resume_unwind(p))
    })
}

#[cfg(test)]
mod tests {
    /// What the Secret Service store does inside `keyring` on Linux.
    fn block_on_a_runtime_of_its_own() -> u8 {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(async { 7 })
    }

    #[tokio::test]
    async fn a_call_from_async_code_doesnt_start_a_runtime_inside_this_one() {
        assert_eq!(super::call(block_on_a_runtime_of_its_own), 7);
    }

    #[test]
    fn a_call_from_plain_code_works_too() {
        assert_eq!(super::call(|| "ok"), "ok");
    }
}

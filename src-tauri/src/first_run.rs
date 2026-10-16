//! First-run setup (roadmap §5.10): whether to show it, and marking it done.

use blotter_core::Database;
use blotter_core::settings::KEY_SETUP_DONE;
use blotter_core::store;
use tauri::State;

use crate::AppState;
use crate::commands::CommandError;

fn lock_err() -> CommandError {
    CommandError::internal("planner state is unavailable after an earlier crash")
}

fn now() -> blotter_core::time::Instant {
    blotter_core::time::truncate(chrono::Utc::now())
}

/// Whether to show the setup: never finished here, and nothing planned yet.
/// A database that already has items or block-outs (from before the setup
/// existed, or restored) is marked done without asking.
fn pending(db: &Database) -> Result<bool, CommandError> {
    let conn = db.conn();
    if store::settings::get::<bool>(conn, KEY_SETUP_DONE)?.unwrap_or(false) {
        return Ok(false);
    }
    let used: i64 = conn
        .query_row(
            "SELECT (SELECT COUNT(*) FROM items) + (SELECT COUNT(*) FROM blockouts)",
            [],
            |r| r.get(0),
        )
        .map_err(|e| CommandError::internal(e.to_string()))?;
    if used > 0 {
        store::settings::set(conn, KEY_SETUP_DONE, &true, now())?;
        return Ok(false);
    }
    Ok(true)
}

#[tauri::command]
pub fn first_run_pending(state: State<'_, AppState>) -> Result<bool, CommandError> {
    let planner = state.planner.lock().map_err(|_| lock_err())?;
    pending(planner.db())
}

/// Marks the setup finished (or skipped), so it doesn't come back.
#[tauri::command]
pub fn finish_first_run(state: State<'_, AppState>) -> Result<(), CommandError> {
    let planner = state.planner.lock().map_err(|_| lock_err())?;
    store::settings::set(planner.db().conn(), KEY_SETUP_DONE, &true, now())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shown_once_and_only_on_a_fresh_database() {
        let fresh = Database::open_in_memory().unwrap();
        assert!(pending(&fresh).unwrap());
        store::settings::set(fresh.conn(), KEY_SETUP_DONE, &true, now()).unwrap();
        assert!(!pending(&fresh).unwrap());

        // A database already in use is marked done without asking.
        let used = Database::open_in_memory().unwrap();
        used.conn()
            .execute(
                "INSERT INTO blockouts (id, label, weekdays, start_time, end_time, tz, \
                 permits_fixed, sort_order, created_at, updated_at) \
                 VALUES ('b', 'Work', 31, '08:30', '17:00', 'UTC', 1, 0, '2026-10-16T00:00:00Z', \
                 '2026-10-16T00:00:00Z')",
                [],
            )
            .unwrap();
        assert!(!pending(&used).unwrap());
        assert_eq!(
            store::settings::get::<bool>(used.conn(), KEY_SETUP_DONE).unwrap(),
            Some(true)
        );
    }
}

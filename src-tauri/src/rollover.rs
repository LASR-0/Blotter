//! End-of-day rollover: checked at startup and then every minute, so it
//! runs on the first launch of a day, or at the rollover time while Blotter
//! sits in the tray. The core decides whether it's due (once a day, after
//! the rollover time) and does the moving.

use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use serde_json::{Value, json};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::AppState;
use crate::commands::CHANGED_EVENT;

/// Tells the UI a rollover moved something; it then takes the notice.
pub const ROLLOVER_EVENT: &str = "planner:rollover";

const CHECK_EVERY: Duration = Duration::from_secs(60);

/// The last automatic rollover that moved something, until the UI shows it.
/// Held here because the first one usually runs before the window loads.
#[derive(Default)]
pub struct RolloverNotice(Mutex<Option<Value>>);

pub fn start(app: AppHandle) {
    let spawned = thread::Builder::new()
        .name("rollover".into())
        .spawn(move || {
            loop {
                check(&app);
                thread::sleep(CHECK_EVERY);
            }
        });
    if let Err(err) = spawned {
        eprintln!("blotter: could not start the rollover check: {err}");
    }
}

fn check(app: &AppHandle) {
    let Some(state) = app.try_state::<AppState>() else {
        return;
    };
    let outcome = match state.planner.lock() {
        Ok(mut planner) => planner.rollover_if_due(),
        Err(_) => return,
    };
    match outcome {
        Ok(Some(outcome)) => {
            let Some(batch_id) = outcome.batch_id else {
                return; // Ran, but nothing was left behind.
            };
            if let Some(notice) = app.try_state::<RolloverNotice>() {
                if let Ok(mut slot) = notice.0.lock() {
                    *slot = Some(outcome.result);
                }
            }
            let _ = app.emit(
                CHANGED_EVENT,
                json!({ "batch_id": batch_id, "ops": ["rollover"] }),
            );
            let _ = app.emit(ROLLOVER_EVENT, ());
        }
        Ok(None) => {}
        Err(err) => eprintln!("blotter: rollover failed: {err}"),
    }
}

/// Returns (and clears) the pending rollover notice, if any.
#[tauri::command]
pub fn take_rollover_notice(notice: State<'_, RolloverNotice>) -> Option<Value> {
    notice.0.lock().ok().and_then(|mut slot| slot.take())
}

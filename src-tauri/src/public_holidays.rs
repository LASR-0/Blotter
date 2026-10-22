//! Settings → Holidays → Public holidays: fetched from Nager.Date for the
//! user's country and region (guessed from their time zone) and handed to
//! the core's import. Once a year's are in, the minute check fetches the
//! next year's in good time, if the user keeps that on.

use std::sync::Mutex;

use chrono::{Datelike, NaiveDate};
use serde_json::{Value, json};
use tauri::{AppHandle, Emitter, Manager, State};

use blotter_core::OpOutcome;
use blotter_core::model::Actor;
use blotter_core::public_holidays::{self, API, Country, Guess, PublicHoliday};

use crate::AppState;
use crate::commands::{CHANGED_EVENT, CommandError};
use crate::subscriptions::fetch_text;

/// The day the automatic fetch was last tried, so a failure (offline)
/// waits until tomorrow instead of retrying every minute.
#[derive(Default)]
pub struct Tried(Mutex<Option<NaiveDate>>);

fn lock_err() -> CommandError {
    CommandError::internal("planner state is unavailable after an earlier crash")
}

async fn get_url(url: &str) -> Result<String, CommandError> {
    fetch_text(url, "application/json")
        .await
        .map_err(|e| CommandError::internal(format!("couldn't reach date.nager.at: {e}")))
}

async fn get(path: &str) -> Result<String, CommandError> {
    get_url(&format!("{API}{path}")).await
}

/// A year's public holidays for `country` that apply in `region`.
async fn fetch_year(
    country: &str,
    region: Option<&str>,
    year: i32,
) -> Result<Vec<PublicHoliday>, CommandError> {
    let json = get_url(&public_holidays::year_url(country, year)).await?;
    Ok(public_holidays::select(&json, region)?)
}

/// Where the user probably is, from Blotter's time zone setting.
#[tauri::command]
pub fn public_holiday_guess(state: State<'_, AppState>) -> Result<Option<Guess>, CommandError> {
    let planner = state.planner.lock().map_err(|_| lock_err())?;
    Ok(public_holidays::guess(planner.settings()?.tz.name()))
}

/// Every country Nager.Date covers, by name.
#[tauri::command]
pub async fn public_holiday_countries() -> Result<Vec<Country>, CommandError> {
    Ok(public_holidays::countries(
        &get("/AvailableCountries").await?,
    )?)
}

/// The regions (states, provinces) `country`'s holidays name this year.
#[tauri::command]
pub async fn public_holiday_regions(country: String) -> Result<Vec<String>, CommandError> {
    let year = chrono::Local::now().year();
    Ok(public_holidays::regions(
        &get(&format!("/PublicHolidays/{year}/{country}")).await?,
    )?)
}

/// Imports this year's and next year's public holidays for `country` and
/// `region`, replacing any imported before, as one undoable batch.
#[tauri::command]
pub async fn import_public_holidays(
    app: AppHandle,
    country: String,
    region: Option<String>,
) -> Result<Value, CommandError> {
    let year = {
        let state = app.state::<AppState>();
        let planner = state.planner.lock().map_err(|_| lock_err())?;
        planner.now().with_timezone(&planner.settings()?.tz).year()
    };
    let mut holidays = fetch_year(&country, region.as_deref(), year).await?;
    holidays.extend(fetch_year(&country, region.as_deref(), year + 1).await?);
    let args = json!({
        "country": country,
        "region": region,
        "years": [year, year + 1],
        "holidays": holidays,
    });
    run(&app, args, &Actor::User).map_err(CommandError::from)
}

fn run(app: &AppHandle, args: Value, actor: &Actor) -> blotter_core::Result<Value> {
    with_planner(app, |planner| {
        planner.execute("import_public_holidays", args, actor)
    })
    .map(|outcome| outcome.result)
}

/// Runs an import against the planner and tells the windows.
fn with_planner(
    app: &AppHandle,
    import: impl FnOnce(&mut blotter_core::Planner) -> blotter_core::Result<OpOutcome>,
) -> blotter_core::Result<OpOutcome> {
    let state = app.state::<AppState>();
    let outcome = {
        let mut planner = state
            .planner
            .lock()
            .map_err(|_| blotter_core::CoreError::invalid("planner state is unavailable"))?;
        import(&mut planner)?
    };
    if let Some(batch_id) = &outcome.batch_id {
        let _ = app.emit(
            CHANGED_EVENT,
            json!({ "batch_id": batch_id, "ops": ["import_public_holidays"] }),
        );
    }
    Ok(outcome)
}

/// From the minute check: fetches next year's holidays once this year's
/// are the last ones in, at most one try a day. What is due, and applying
/// it, are the core's (Blotter Sync does the same upkeep for paired devices).
pub fn update_if_due(app: &AppHandle) {
    let Some(state) = app.try_state::<AppState>() else {
        return;
    };
    let (due, today) = {
        let Ok(planner) = state.planner.lock() else {
            return;
        };
        let Ok(Some(due)) = public_holidays::upkeep(&planner) else {
            return;
        };
        let Ok(settings) = planner.settings() else {
            return;
        };
        (due, planner.now().with_timezone(&settings.tz).date_naive())
    };
    let year = due.year;
    let tried = app.state::<Tried>();
    {
        let Ok(mut last) = tried.0.lock() else {
            return;
        };
        if *last == Some(today) {
            return;
        }
        *last = Some(today);
    }
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let fetched = get_url(&due.url()).await;
        let result = fetched.map_err(|e| e.message).and_then(|json| {
            with_planner(&app, |planner| due.apply(planner, &json)).map_err(|e| e.to_string())
        });
        if let Err(err) = result {
            eprintln!("blotter: fetching {year}'s public holidays failed: {err}");
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Against the real API: `cargo test -p blotter live_nager -- --ignored`.
    #[tokio::test]
    #[ignore = "needs the network"]
    async fn live_nager() {
        let vic = fetch_year("AU", Some("AU-VIC"), 2027).await.unwrap();
        assert!(vic.iter().any(|h| h.name == "Melbourne Cup"), "{vic:?}");
        assert!(vic.iter().any(|h| h.name == "Anzac Day"));
        let regions = public_holiday_regions("AU".into()).await.unwrap();
        assert!(regions.contains(&"AU-VIC".to_owned()), "{regions:?}");
        let countries = public_holiday_countries().await.unwrap();
        assert!(countries.len() > 50);
    }
}

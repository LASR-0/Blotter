//! Holidays: date ranges away from the usual week. Making one lifts the
//! block-outs that fall on its days, except ones that warn about fixed items
//! such as Sleep (exceptions carrying the holiday's id,
//! shown with the block-out as usual); changing or deleting it puts them
//! back. The slot engine keeps work tasks off its days and gives tasks
//! tagged `holiday` first pick. Once its last day has passed it is removed.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use chrono::Datelike;

use super::blockouts::EXCEPTION_ENTITY;
use super::input::{DateInput, NoArgs, patch};
use super::{Ctx, OpDef};
use crate::error::{CoreError, Result};
use crate::model::{BlockoutException, Holiday};
use crate::public_holidays::{Config, PublicHoliday};
use crate::settings::KEY_PUBLIC_HOLIDAYS;
use crate::store;

pub(crate) const ENTITY: &str = "holiday";

/// Longest holiday, in days.
const MAX_DAYS: i64 = 366;

pub(super) fn defs() -> Vec<OpDef> {
    vec![
        OpDef::new(
            "list_holidays",
            "List holidays (date ranges away from the usual week), soonest first.",
            false,
            list_holidays,
        ),
        OpDef::new(
            "create_holiday",
            "Add a holiday from `start_date` to `end_date` (inclusive). Block-outs on those \
             days are lifted (except ones with `permits_fixed` false, such as Sleep), work tasks aren't slotted into them, and tasks tagged `holiday` \
             get first pick. It is removed automatically after its last day.",
            true,
            create_holiday,
        ),
        OpDef::new(
            "update_holiday",
            "Change a holiday's name, dates or colour. Omitted fields are left as they are; \
             the block-outs it lifts follow its new dates.",
            true,
            update_holiday,
        ),
        OpDef::new(
            "delete_holiday",
            "Delete a holiday; the block-outs it lifted apply again.",
            true,
            delete_holiday,
        ),
        OpDef::new(
            "prune_holidays",
            "Remove holidays whose last day has passed.",
            true,
            prune_holidays,
        ),
        OpDef::new(
            "get_public_holidays",
            "Where the user's imported public holidays come from (country, region, whether \
             each next year's are fetched automatically, the last year imported), or null if \
             none were imported. Imported ones are holidays with `source` set.",
            false,
            get_public_holidays,
        ),
        OpDef::new(
            "import_public_holidays",
            "Replace the imported public holidays in `years` with `holidays` (from Settings → \
             Holidays, which fetches them for the user's country and region; don't make them \
             up). Each becomes a one-day holiday, from today on. Holidays the user made are \
             never touched.",
            true,
            import_public_holidays,
        ),
        OpDef::new(
            "set_public_holiday_updates",
            "Turn the automatic fetch of each next year's public holidays on or off.",
            true,
            set_public_holiday_updates,
        ),
        OpDef::new(
            "forget_public_holidays",
            "Remove every imported public holiday and stop fetching them.",
            true,
            forget_public_holidays,
        ),
    ]
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ImportPublicHolidaysArgs {
    /// ISO 3166-1 alpha-2, e.g. `AU`.
    #[schemars(regex(pattern = r"^[A-Z]{2}$"))]
    pub country: String,
    /// ISO 3166-2 within the country, e.g. `AU-VIC`; null for national ones only.
    pub region: Option<String>,
    /// The years being replaced; every holiday's date is in one of them.
    #[schemars(length(min = 1, max = 3))]
    pub years: Vec<i32>,
    #[schemars(length(max = 200))]
    pub holidays: Vec<PublicHoliday>,
    /// Defaults to what it was, or on.
    pub keep_updated: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PublicHolidayUpdatesArgs {
    pub keep_updated: bool,
}

#[derive(Debug, Serialize)]
pub struct PublicImport {
    pub added: usize,
    pub removed: usize,
    pub config: Config,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateHolidayArgs {
    #[schemars(length(min = 1, max = 100))]
    pub label: String,
    pub start_date: DateInput,
    pub end_date: DateInput,
    pub color: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct UpdateHolidayArgs {
    pub id: String,
    #[schemars(length(min = 1, max = 100))]
    pub label: Option<String>,
    pub start_date: Option<DateInput>,
    pub end_date: Option<DateInput>,
    /// `null` clears it.
    #[serde(default, deserialize_with = "patch")]
    pub color: Option<Option<String>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HolidayIdArgs {
    pub id: String,
}

#[derive(Debug, Serialize)]
pub struct HolidayDetail {
    #[serde(flatten)]
    pub holiday: Holiday,
    /// Block-out dates it lifted.
    pub lifted: Vec<BlockoutException>,
}

#[derive(Debug, Serialize)]
pub struct Pruned {
    pub removed: Vec<String>,
}

fn validate(h: &mut Holiday) -> Result<()> {
    h.label = h.label.trim().to_owned();
    if h.label.is_empty() {
        return Err(CoreError::invalid("label must not be empty"));
    }
    if h.end_date < h.start_date {
        return Err(CoreError::invalid("end_date is before start_date"));
    }
    if (h.end_date - h.start_date).num_days() >= MAX_DAYS {
        return Err(CoreError::invalid(format!(
            "a holiday is at most {MAX_DAYS} days"
        )));
    }
    if let Some(color) = &h.color {
        let hex = color.strip_prefix('#').unwrap_or("");
        if hex.len() != 6 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(CoreError::invalid(format!(
                "color '{color}' must look like #7d808c"
            )));
        }
    }
    Ok(())
}

/// Lifts the block-outs falling on the holiday's days, unless already lifted.
/// Ones that warn about fixed items (Sleep) stay: a day off still has a night.
fn lift(ctx: &mut Ctx<'_>, h: &Holiday) -> Result<Vec<BlockoutException>> {
    let mut blockouts = store::blockouts::list(ctx.conn)?;
    blockouts.retain(|b| b.permits_fixed);
    let taken =
        store::blockouts::list_exceptions(ctx.conn, None, Some((h.start_date, h.end_date)))?;
    let mut made = Vec::new();
    for date in h.dates() {
        for b in blockouts.iter().filter(|b| b.applies_on(date)) {
            if taken
                .iter()
                .any(|e| e.blockout_id == b.id && e.date == date)
            {
                continue;
            }
            let e = BlockoutException {
                id: ctx.new_id(),
                blockout_id: b.id.clone(),
                date,
                note: format!("Holiday: {}", h.label),
                created_at: ctx.now,
                holiday_id: Some(h.id.clone()),
            };
            store::blockouts::upsert_exception(ctx.conn, &e)?;
            ctx.record(EXCEPTION_ENTITY, &e.id, None, Some(&e))?;
            made.push(e);
        }
    }
    Ok(made)
}

/// Removes the exceptions a holiday made, so its block-outs apply again.
fn unlift(ctx: &mut Ctx<'_>, holiday_id: &str) -> Result<()> {
    for e in store::holidays::exceptions_of(ctx.conn, holiday_id)? {
        ctx.record(EXCEPTION_ENTITY, &e.id, Some(&e), None)?;
        store::blockouts::delete_exception(ctx.conn, &e.id)?;
    }
    Ok(())
}

fn list_holidays(ctx: &mut Ctx<'_>, _: NoArgs) -> Result<Vec<Holiday>> {
    store::holidays::list(ctx.conn)
}

fn create_holiday(ctx: &mut Ctx<'_>, a: CreateHolidayArgs) -> Result<HolidayDetail> {
    let mut h = Holiday {
        id: ctx.new_id(),
        label: a.label,
        start_date: a.start_date.0,
        end_date: a.end_date.0,
        color: a.color,
        created_at: ctx.now,
        updated_at: ctx.now,
        source: None,
    };
    validate(&mut h)?;
    store::holidays::upsert(ctx.conn, &h)?;
    ctx.record(ENTITY, &h.id, None, Some(&h))?;
    let lifted = lift(ctx, &h)?;
    Ok(HolidayDetail { holiday: h, lifted })
}

fn update_holiday(ctx: &mut Ctx<'_>, a: UpdateHolidayArgs) -> Result<HolidayDetail> {
    let before = store::holidays::require(ctx.conn, &a.id)?;
    let mut h = before.clone();
    if let Some(label) = a.label {
        h.label = label;
    }
    if let Some(d) = a.start_date {
        h.start_date = d.0;
    }
    if let Some(d) = a.end_date {
        h.end_date = d.0;
    }
    if let Some(color) = a.color {
        h.color = color;
    }
    validate(&mut h)?;
    h.updated_at = ctx.now;
    store::holidays::upsert(ctx.conn, &h)?;
    ctx.record(ENTITY, &h.id, Some(&before), Some(&h))?;
    // Re-lift for the new dates (and label, which the notes carry).
    unlift(ctx, &h.id)?;
    let lifted = lift(ctx, &h)?;
    Ok(HolidayDetail { holiday: h, lifted })
}

fn remove(ctx: &mut Ctx<'_>, h: &Holiday) -> Result<()> {
    unlift(ctx, &h.id)?;
    ctx.record(ENTITY, &h.id, Some(h), None)?;
    store::holidays::delete(ctx.conn, &h.id)?;
    Ok(())
}

fn delete_holiday(ctx: &mut Ctx<'_>, a: HolidayIdArgs) -> Result<super::blockouts::Deleted> {
    let h = store::holidays::require(ctx.conn, &a.id)?;
    remove(ctx, &h)?;
    Ok(super::blockouts::Deleted { deleted: h.id })
}

fn prune_holidays(ctx: &mut Ctx<'_>, _: NoArgs) -> Result<Pruned> {
    let today = ctx.now.with_timezone(&ctx.settings.tz).date_naive();
    let mut removed = Vec::new();
    for h in store::holidays::ended_before(ctx.conn, today)? {
        remove(ctx, &h)?;
        removed.push(h.label);
    }
    Ok(Pruned { removed })
}

fn config(ctx: &Ctx<'_>) -> Result<Option<Config>> {
    store::settings::get(ctx.conn, KEY_PUBLIC_HOLIDAYS)
}

fn get_public_holidays(ctx: &mut Ctx<'_>, _: NoArgs) -> Result<Option<Config>> {
    config(ctx)
}

fn import_public_holidays(ctx: &mut Ctx<'_>, a: ImportPublicHolidaysArgs) -> Result<PublicImport> {
    let country = a.country.trim().to_uppercase();
    if country.len() != 2 || !country.chars().all(|c| c.is_ascii_uppercase()) {
        return Err(CoreError::invalid(format!(
            "'{country}' isn't a two-letter country code"
        )));
    }
    let region = a
        .region
        .map(|r| r.trim().to_uppercase())
        .filter(|r| !r.is_empty());
    if let Some(r) = &region
        && !r.starts_with(&format!("{country}-"))
    {
        return Err(CoreError::invalid(format!(
            "region '{r}' isn't in {country}"
        )));
    }
    let Some(&last) = a.years.iter().max() else {
        return Err(CoreError::invalid("give at least one year"));
    };
    if let Some(h) = a
        .holidays
        .iter()
        .find(|h| !a.years.contains(&h.date.year()))
    {
        return Err(CoreError::invalid(format!(
            "{} ({}) isn't in the years given",
            h.name, h.date
        )));
    }

    let before = config(ctx)?;
    let config = Config {
        through_year: match &before {
            Some(b) if b.country == country && b.region == region => b.through_year.max(last),
            _ => last,
        },
        keep_updated: a
            .keep_updated
            .unwrap_or_else(|| before.as_ref().is_none_or(|b| b.keep_updated)),
        country,
        region,
    };

    // Out with what an earlier import brought in for those years (any
    // region, so changing region replaces them), then in with the new.
    let mut removed = 0;
    for h in store::holidays::list(ctx.conn)? {
        if h.is_public() && a.years.contains(&h.start_date.year()) {
            remove(ctx, &h)?;
            removed += 1;
        }
    }
    let today = ctx.now.with_timezone(&ctx.settings.tz).date_naive();
    let source = config.source();
    let mut added = 0;
    for p in a.holidays.into_iter().filter(|p| p.date >= today) {
        let mut h = Holiday {
            id: ctx.new_id(),
            label: p.name,
            start_date: p.date,
            end_date: p.date,
            color: None,
            created_at: ctx.now,
            updated_at: ctx.now,
            source: Some(source.clone()),
        };
        validate(&mut h)?;
        store::holidays::upsert(ctx.conn, &h)?;
        ctx.record(ENTITY, &h.id, None, Some(&h))?;
        lift(ctx, &h)?;
        added += 1;
    }
    super::settings::write(ctx, KEY_PUBLIC_HOLIDAYS, &config)?;
    Ok(PublicImport {
        added,
        removed,
        config,
    })
}

fn set_public_holiday_updates(ctx: &mut Ctx<'_>, a: PublicHolidayUpdatesArgs) -> Result<Config> {
    let mut config =
        config(ctx)?.ok_or_else(|| CoreError::invalid("no public holidays have been imported"))?;
    config.keep_updated = a.keep_updated;
    super::settings::write(ctx, KEY_PUBLIC_HOLIDAYS, &config)?;
    Ok(config)
}

fn forget_public_holidays(ctx: &mut Ctx<'_>, _: NoArgs) -> Result<Pruned> {
    let mut removed = Vec::new();
    for h in store::holidays::list(ctx.conn)? {
        if h.is_public() {
            remove(ctx, &h)?;
            removed.push(h.label);
        }
    }
    let before: Option<serde_json::Value> = store::settings::get(ctx.conn, KEY_PUBLIC_HOLIDAYS)?;
    if before.is_some() {
        store::settings::delete(ctx.conn, KEY_PUBLIC_HOLIDAYS)?;
        ctx.record(
            "setting",
            KEY_PUBLIC_HOLIDAYS,
            before.as_ref(),
            None::<&serde_json::Value>,
        )?;
    }
    Ok(Pruned { removed })
}

#[cfg(test)]
mod tests {
    use crate::model::Actor;
    use serde_json::{Value, json};

    fn run(p: &mut crate::Planner, op: &str, args: Value) -> Value {
        p.execute(op, args, &Actor::User).unwrap().result
    }

    /// The test clock is Wed 14 Oct 2026 09:00 Melbourne.
    fn with_work() -> crate::Planner {
        let mut p = crate::planner::tests::planner();
        run(
            &mut p,
            "create_blockout",
            json!({ "label": "Work", "weekdays": ["mon", "tue", "wed", "thu", "fri"],
                    "start_time": "09:00", "end_time": "17:00" }),
        );
        p
    }

    fn lifted_dates(p: &mut crate::Planner) -> Vec<String> {
        run(p, "list_blockouts", json!({}))[0]["exceptions"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e["date"].as_str().unwrap().to_owned())
            .collect()
    }

    #[test]
    fn a_holiday_lifts_the_block_outs_on_its_days_and_gives_them_back() {
        let mut p = with_work();
        // Fri 16 – Mon 19 Oct: Work runs Fri and Mon.
        let h = run(
            &mut p,
            "create_holiday",
            json!({ "label": "Coast", "start_date": "2026-10-16", "end_date": "2026-10-19" }),
        );
        assert_eq!(h["lifted"].as_array().unwrap().len(), 2);
        assert_eq!(lifted_dates(&mut p), ["2026-10-16", "2026-10-19"]);
        assert_eq!(
            run(&mut p, "list_blockouts", json!({}))[0]["exceptions"][0]["note"],
            "Holiday: Coast"
        );

        // Moving it moves what it lifts.
        run(
            &mut p,
            "update_holiday",
            json!({ "id": h["id"], "start_date": "2026-10-20", "end_date": "2026-10-20" }),
        );
        assert_eq!(lifted_dates(&mut p), ["2026-10-20"]);

        run(&mut p, "delete_holiday", json!({ "id": h["id"] }));
        assert!(lifted_dates(&mut p).is_empty());
        // One undo brings back the holiday and what it lifted.
        run(&mut p, "undo", json!({}));
        assert_eq!(lifted_dates(&mut p), ["2026-10-20"]);
        assert_eq!(
            run(&mut p, "list_holidays", json!({}))
                .as_array()
                .unwrap()
                .len(),
            1
        );
    }

    fn import(p: &mut crate::Planner, region: &str, holidays: Value) -> Value {
        run(
            p,
            "import_public_holidays",
            json!({ "country": "AU", "region": region, "years": [2026, 2027], "holidays": holidays }),
        )
    }

    #[test]
    fn public_holidays_import_from_today_and_replace_their_own() {
        let mut p = with_work();
        run(
            &mut p,
            "create_holiday",
            json!({ "label": "Coast", "start_date": "2026-12-28", "end_date": "2026-12-30" }),
        );
        let out = import(
            &mut p,
            "AU-VIC",
            json!([
                { "date": "2026-03-09", "name": "Labour Day" },
                { "date": "2026-11-03", "name": "Melbourne Cup" },
                { "date": "2027-01-01", "name": "New Year's Day" }
            ]),
        );
        assert_eq!(out["added"], 2, "the past one is skipped");
        assert_eq!(out["config"]["through_year"], 2027);
        assert_eq!(out["config"]["keep_updated"], true);
        let holidays = run(&mut p, "list_holidays", json!({}));
        let cup = &holidays.as_array().unwrap()[0];
        assert_eq!(cup["label"], "Melbourne Cup");
        assert_eq!(cup["source"], "public:AU-VIC");
        assert!(
            lifted_dates(&mut p).contains(&"2026-11-03".to_owned()),
            "Tuesday's Work"
        );

        // Moving to NSW swaps them; the user's own holiday stays.
        let out = import(
            &mut p,
            "AU-NSW",
            json!([{ "date": "2027-01-01", "name": "New Year's Day" }]),
        );
        assert_eq!(
            (out["removed"].clone(), out["added"].clone()),
            (json!(2), json!(1))
        );
        let labels: Vec<Value> = run(&mut p, "list_holidays", json!({}))
            .as_array()
            .unwrap()
            .iter()
            .map(|h| h["label"].clone())
            .collect();
        assert_eq!(labels, [json!("Coast"), json!("New Year's Day")]);
        assert_eq!(
            run(&mut p, "get_public_holidays", json!({}))["region"],
            "AU-NSW"
        );

        // Off, then forgotten: only the user's holiday is left.
        run(
            &mut p,
            "set_public_holiday_updates",
            json!({ "keep_updated": false }),
        );
        assert_eq!(
            run(&mut p, "get_public_holidays", json!({}))["keep_updated"],
            false
        );
        run(&mut p, "forget_public_holidays", json!({}));
        assert!(run(&mut p, "get_public_holidays", json!({})).is_null());
        assert_eq!(
            run(&mut p, "list_holidays", json!({}))
                .as_array()
                .unwrap()
                .len(),
            1
        );
        run(&mut p, "undo", json!({}));
        assert_eq!(
            run(&mut p, "get_public_holidays", json!({}))["region"],
            "AU-NSW"
        );
    }

    #[test]
    fn public_holiday_import_checks_its_input() {
        let mut p = with_work();
        for args in [
            json!({ "country": "Australia", "years": [2026], "holidays": [] }),
            json!({ "country": "AU", "region": "NZ-AUK", "years": [2026], "holidays": [] }),
            json!({ "country": "AU", "years": [2026],
                    "holidays": [{ "date": "2027-01-01", "name": "New Year's Day" }] }),
        ] {
            assert!(
                p.execute("import_public_holidays", args, &Actor::User)
                    .is_err()
            );
        }
    }

    #[test]
    fn a_holiday_leaves_sleep_in_place() {
        let mut p = with_work();
        run(
            &mut p,
            "create_blockout",
            json!({ "label": "Sleep", "weekdays": ["mon","tue","wed","thu","fri","sat","sun"],
                    "start_time": "23:00", "end_time": "06:30", "permits_fixed": false }),
        );
        let h = run(
            &mut p,
            "create_holiday",
            json!({ "label": "Coast", "start_date": "2026-10-16", "end_date": "2026-10-16" }),
        );
        let lifted = h["lifted"].as_array().unwrap();
        assert_eq!(lifted.len(), 1, "Work, not Sleep");
        let work = run(&mut p, "list_blockouts", json!({}))
            .as_array()
            .unwrap()
            .iter()
            .find(|b| b["label"] == "Work")
            .unwrap()["id"]
            .clone();
        assert_eq!(lifted[0]["blockout_id"], work);
    }

    #[test]
    fn a_lift_the_user_made_is_left_alone() {
        let mut p = with_work();
        let work = run(&mut p, "list_blockouts", json!({}))[0]["id"].clone();
        run(
            &mut p,
            "add_blockout_exception",
            json!({ "blockout_id": work, "date": "2026-10-16", "note": "Leave" }),
        );
        let h = run(
            &mut p,
            "create_holiday",
            json!({ "label": "Coast", "start_date": "2026-10-16", "end_date": "2026-10-16" }),
        );
        assert!(h["lifted"].as_array().unwrap().is_empty());
        run(&mut p, "delete_holiday", json!({ "id": h["id"] }));
        assert_eq!(
            lifted_dates(&mut p),
            ["2026-10-16"],
            "the user's lift stays"
        );
    }

    #[test]
    fn ended_holidays_are_pruned() {
        let mut p = with_work();
        for (label, from, to) in [
            ("Past", "2026-10-01", "2026-10-13"),
            ("Today", "2026-10-10", "2026-10-14"),
        ] {
            run(
                &mut p,
                "create_holiday",
                json!({ "label": label, "start_date": from, "end_date": to }),
            );
        }
        let out = p
            .execute("prune_holidays", json!({}), &Actor::System)
            .unwrap()
            .result;
        assert_eq!(out["removed"], json!(["Past"]));
        let left = run(&mut p, "list_holidays", json!({}));
        assert_eq!(left.as_array().unwrap().len(), 1);
        assert_eq!(left[0]["label"], "Today");
        assert!(
            p.execute(
                "create_holiday",
                json!({ "label": "Backwards", "start_date": "2026-10-20", "end_date": "2026-10-19" }),
                &Actor::User,
            )
            .is_err()
        );
    }

    #[test]
    fn work_stays_off_a_holiday_and_holiday_tasks_go_first() {
        let mut p = crate::planner::tests::planner();
        run(
            &mut p,
            "create_holiday",
            json!({ "label": "Coast", "start_date": "2026-10-14", "end_date": "2026-10-16" }),
        );
        let ids: Vec<Value> = [("Report", "work"), ("Pack", "holiday"), ("Read", "home")]
            .into_iter()
            .map(|(title, tag)| {
                run(
                    &mut p,
                    "create_item",
                    json!({ "title": title, "tags": [tag], "duration_min": 60 }),
                )["id"]
                    .clone()
            })
            .collect();
        let out = run(&mut p, "auto_slot", json!({ "until": "2026-10-16T23:59" }));
        let placed: Vec<&str> = out["placed"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t["item"]["title"].as_str().unwrap())
            .collect();
        assert_eq!(
            placed,
            ["Pack", "Read"],
            "holiday task first, work task left out"
        );
        assert_eq!(out["unplaced"][0]["id"], ids[0]);
    }
}

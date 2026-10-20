//! Holidays: date ranges away from the usual week. Making one lifts every
//! block-out that falls on its days (exceptions carrying the holiday's id,
//! shown with the block-out as usual); changing or deleting it puts them
//! back. The slot engine keeps work tasks off its days and gives tasks
//! tagged `holiday` first pick. Once its last day has passed it is removed.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::blockouts::EXCEPTION_ENTITY;
use super::input::{DateInput, NoArgs, patch};
use super::{Ctx, OpDef};
use crate::error::{CoreError, Result};
use crate::model::{BlockoutException, Holiday};
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
             days are lifted, work tasks aren't slotted into them, and tasks tagged `holiday` \
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
    ]
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

/// Lifts every block-out falling on the holiday's days, unless already lifted.
fn lift(ctx: &mut Ctx<'_>, h: &Holiday) -> Result<Vec<BlockoutException>> {
    let blockouts = store::blockouts::list(ctx.conn)?;
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

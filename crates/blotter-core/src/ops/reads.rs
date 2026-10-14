//! Read-only operations. These never write and always run immediately,
//! even in AI propose mode.

use chrono::{Duration, NaiveDate};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::input::{DateInput, NoArgs, start_of_day};
use super::occurrences::{Target, resolve};
use super::slots::{Conflict, conflicts};
use super::{Ctx, OpDef};
use crate::error::{CoreError, Result};
use crate::model::{Blockout, BlockoutException, Item, ItemStatus, Reminder};
use crate::recurrence;
use crate::store;
use crate::store::items::ItemRange;
use crate::time::Instant;
use crate::windows::{self, BlockoutWindow};

/// Longest range `list_agenda` serves in one call (a Year view is ~366 days).
const MAX_AGENDA_DAYS: i64 = 400;

pub(super) fn defs() -> Vec<OpDef> {
    vec![
        OpDef::new(
            "list_agenda",
            "List scheduled items between two dates (inclusive) in the user's timezone.",
            false,
            list_agenda,
        ),
        OpDef::new(
            "get_item",
            "Get one item with its reminders.",
            false,
            get_item,
        ),
        OpDef::new(
            "search_items",
            "Find items whose title, notes or tags contain the query (case-insensitive).",
            false,
            search_items,
        ),
        OpDef::new(
            "list_inbox",
            "List unscheduled flexible tasks waiting in the Inbox.",
            false,
            list_inbox,
        ),
        OpDef::new(
            "list_blockouts",
            "List block-out windows with their lifted dates.",
            false,
            list_blockouts,
        ),
        OpDef::new(
            "list_blockout_windows",
            "List the concrete block-out windows covering a date range (inclusive, user's \
             timezone): each occurrence with its start/end instants, after weekdays, active \
             ranges and lifted dates are applied. Flexible tasks can't go inside these; \
             fixed items can unless `permits_fixed` is false. A window past midnight belongs \
             to the date it starts on.",
            false,
            list_blockout_windows,
        ),
        OpDef::new(
            "list_overdue",
            "List open tasks left on earlier days or past their deadline, oldest first.",
            false,
            list_overdue,
        ),
        OpDef::new(
            "get_day_summary",
            "Summarize one day: items, planned minutes, done/open counts, overdue and Inbox \
             counts. Defaults to today.",
            false,
            get_day_summary,
        ),
    ]
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ListAgendaArgs {
    pub start: DateInput,
    /// Last day, inclusive. Defaults to `start`.
    pub end: Option<DateInput>,
    /// Include completed items. Defaults to true.
    pub include_done: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct Agenda {
    pub start: NaiveDate,
    pub end: NaiveDate,
    pub timezone: String,
    pub items: Vec<Item>,
    /// Items that overlap others, or sit fixed in a block-out that warns.
    pub conflicts: Vec<Conflict>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GetItemArgs {
    pub id: String,
}

#[derive(Debug, Serialize)]
pub struct ItemDetail {
    #[serde(flatten)]
    pub item: Item,
    pub reminders: Vec<Reminder>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SearchArgs {
    #[schemars(length(min = 1, max = 200))]
    pub query: String,
    /// Defaults to 20.
    #[schemars(range(min = 1, max = 200))]
    pub limit: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct BlockoutWithExceptions {
    #[serde(flatten)]
    pub blockout: Blockout,
    pub exceptions: Vec<BlockoutException>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DateRangeArgs {
    pub start: DateInput,
    /// Last day, inclusive. Defaults to `start`.
    pub end: Option<DateInput>,
}

#[derive(Debug, Serialize)]
pub struct BlockoutWindows {
    pub start: NaiveDate,
    pub end: NaiveDate,
    pub timezone: String,
    pub windows: Vec<BlockoutWindow>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DaySummaryArgs {
    /// Defaults to today.
    pub date: Option<DateInput>,
}

#[derive(Debug, Serialize)]
pub struct DaySummary {
    pub date: NaiveDate,
    pub timezone: String,
    pub items: Vec<Item>,
    /// Timed minutes scheduled on this day (clipped to the day).
    pub planned_min: i64,
    pub open_count: usize,
    pub done_count: usize,
    /// Open tasks left on earlier days or past their deadline (see list_overdue).
    pub overdue_count: usize,
    pub inbox_count: usize,
    pub conflicts: Vec<Conflict>,
}

/// `[start of first day, start of the day after last)` in the user's zone.
fn day_range(ctx: &Ctx<'_>, start: NaiveDate, end: NaiveDate) -> ItemRange {
    let tz = ctx.settings.tz;
    ItemRange {
        start: start_of_day(tz, start),
        end: start_of_day(tz, end + Duration::days(1)),
        start_date: start,
        end_date: end,
    }
}

pub(crate) fn today(ctx: &Ctx<'_>) -> NaiveDate {
    ctx.now.with_timezone(&ctx.settings.tz).date_naive()
}

/// Checks an inclusive `start..=end` day range (end defaults to start).
fn checked_days(start: DateInput, end: Option<DateInput>) -> Result<(NaiveDate, NaiveDate)> {
    let start = start.0;
    let end = end.map(|d| d.0).unwrap_or(start);
    if end < start {
        return Err(CoreError::invalid("`end` is before `start`"));
    }
    if (end - start).num_days() >= MAX_AGENDA_DAYS {
        return Err(CoreError::invalid(format!(
            "range is longer than {MAX_AGENDA_DAYS} days"
        )));
    }
    Ok((start, end))
}

fn list_agenda(ctx: &mut Ctx<'_>, a: ListAgendaArgs) -> Result<Agenda> {
    let (start, end) = checked_days(a.start, a.end)?;
    let mut items = recurrence::agenda(ctx.conn, day_range(ctx, start, end), ctx.settings.tz)?;
    if a.include_done == Some(false) {
        items.retain(|i| i.status != ItemStatus::Done);
    }
    Ok(Agenda {
        start,
        end,
        timezone: ctx.settings.tz.name().to_owned(),
        conflicts: conflicts(ctx, &items)?,
        items,
    })
}

fn get_item(ctx: &mut Ctx<'_>, a: GetItemArgs) -> Result<ItemDetail> {
    // An occurrence shows its override if it has one; reminders belong to the series.
    let (item, reminders_of) = match resolve(ctx, &a.id)? {
        Target::Item(item) => {
            let id = item.id.clone();
            (item, id)
        }
        Target::Occurrence {
            series,
            slot,
            existing,
        } => (
            existing.unwrap_or_else(|| recurrence::occurrence_item(&series, slot)),
            series.id,
        ),
    };
    Ok(ItemDetail {
        reminders: store::reminders::list_for_item(ctx.conn, &reminders_of)?,
        item,
    })
}

fn search_items(ctx: &mut Ctx<'_>, a: SearchArgs) -> Result<Vec<Item>> {
    store::items::search(ctx.conn, a.query.trim(), a.limit.unwrap_or(20))
}

fn list_inbox(ctx: &mut Ctx<'_>, _: NoArgs) -> Result<Vec<Item>> {
    store::items::list_inbox(ctx.conn)
}

fn list_blockouts(ctx: &mut Ctx<'_>, _: NoArgs) -> Result<Vec<BlockoutWithExceptions>> {
    store::blockouts::list(ctx.conn)?
        .into_iter()
        .map(|b| {
            let exceptions = store::blockouts::list_exceptions(ctx.conn, Some(&b.id), None)?;
            Ok(BlockoutWithExceptions {
                blockout: b,
                exceptions,
            })
        })
        .collect()
}

fn list_blockout_windows(ctx: &mut Ctx<'_>, a: DateRangeArgs) -> Result<BlockoutWindows> {
    let (start, end) = checked_days(a.start, a.end)?;
    let range = day_range(ctx, start, end);
    Ok(BlockoutWindows {
        start,
        end,
        timezone: ctx.settings.tz.name().to_owned(),
        windows: windows::load(ctx.conn, range.start, range.end)?,
    })
}

fn get_day_summary(ctx: &mut Ctx<'_>, a: DaySummaryArgs) -> Result<DaySummary> {
    let date = a.date.map(|d| d.0).unwrap_or_else(|| today(ctx));
    let range = day_range(ctx, date, date);
    let items = recurrence::agenda(ctx.conn, range, ctx.settings.tz)?;
    let planned_min = items
        .iter()
        .filter(|i| i.status != ItemStatus::Cancelled)
        .filter_map(|i| Some(clip_minutes(i.start_at?, i.end_at?, range.start, range.end)))
        .sum();
    let overdue_count = overdue(ctx)?.len();
    Ok(DaySummary {
        date,
        timezone: ctx.settings.tz.name().to_owned(),
        planned_min,
        open_count: items
            .iter()
            .filter(|i| i.status == ItemStatus::Open)
            .count(),
        done_count: items
            .iter()
            .filter(|i| i.status == ItemStatus::Done)
            .count(),
        overdue_count,
        inbox_count: store::items::list_inbox(ctx.conn)?
            .iter()
            .filter(|i| i.status == ItemStatus::Open)
            .count(),
        conflicts: conflicts(ctx, &items)?,
        items,
    })
}

fn overdue(ctx: &Ctx<'_>) -> Result<Vec<Item>> {
    let today = today(ctx);
    store::items::list_overdue(
        ctx.conn,
        ctx.now,
        start_of_day(ctx.settings.tz, today),
        today,
    )
}

fn list_overdue(ctx: &mut Ctx<'_>, _: NoArgs) -> Result<Vec<Item>> {
    overdue(ctx)
}

fn clip_minutes(start: Instant, end: Instant, lo: Instant, hi: Instant) -> i64 {
    (end.min(hi) - start.max(lo)).num_minutes().max(0)
}

#[cfg(test)]
mod tests {
    use crate::model::Actor;
    use serde_json::{Value, json};

    fn run(p: &mut crate::Planner, op: &str, args: Value) -> Value {
        p.execute(op, args, &Actor::User).unwrap().result
    }

    #[test]
    fn agenda_uses_local_day_boundaries() {
        let mut p = crate::planner::tests::planner();
        // 23:30 local on the 20th is 12:30Z the same day; 00:30 local on the
        // 21st is 13:30Z on the 20th. Only the first belongs to the 20th.
        run(
            &mut p,
            "create_item",
            json!({ "title": "Late", "start": "2026-10-20T23:30", "duration_min": 15 }),
        );
        run(
            &mut p,
            "create_item",
            json!({ "title": "Early", "start": "2026-10-21T00:30", "duration_min": 15 }),
        );
        let day = run(&mut p, "list_agenda", json!({ "start": "2026-10-20" }));
        let titles: Vec<_> = day["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|i| i["title"].clone())
            .collect();
        assert_eq!(titles, [json!("Late")]);
        assert_eq!(day["timezone"], "Australia/Melbourne");

        let both = run(
            &mut p,
            "list_agenda",
            json!({ "start": "2026-10-20", "end": "2026-10-21" }),
        );
        assert_eq!(both["items"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn agenda_filters_done_and_validates_range() {
        let mut p = crate::planner::tests::planner();
        let i = run(
            &mut p,
            "create_item",
            json!({ "title": "Run", "start": "2026-10-20T07:00" }),
        );
        run(&mut p, "complete_item", json!({ "id": i["id"] }));
        let open = run(
            &mut p,
            "list_agenda",
            json!({ "start": "2026-10-20", "include_done": false }),
        );
        assert!(open["items"].as_array().unwrap().is_empty());
        assert!(
            p.execute(
                "list_agenda",
                json!({ "start": "2026-10-20", "end": "2026-10-19" }),
                &Actor::User
            )
            .is_err()
        );
        assert!(
            p.execute(
                "list_agenda",
                json!({ "start": "2026-01-01", "end": "2027-12-31" }),
                &Actor::User
            )
            .is_err()
        );
    }

    #[test]
    fn reads_write_nothing() {
        let mut p = crate::planner::tests::planner();
        for (op, args) in [
            ("list_agenda", json!({ "start": "2026-10-20" })),
            ("list_inbox", json!({})),
            ("list_blockouts", json!({})),
            ("get_day_summary", json!({})),
            ("search_items", json!({ "query": "x" })),
        ] {
            let out = p.execute(op, args, &Actor::User).unwrap();
            assert!(out.batch_id.is_none() && out.changes.is_empty(), "{op}");
        }
    }

    #[test]
    fn get_item_includes_reminders_and_not_found_is_clear() {
        let mut p = crate::planner::tests::planner();
        let i = run(
            &mut p,
            "create_item",
            json!({ "title": "Dentist", "kind": "event", "start": "2026-10-20T15:00" }),
        );
        run(
            &mut p,
            "set_reminder",
            json!({ "item_id": i["id"], "offset_min": 10 }),
        );
        let d = run(&mut p, "get_item", json!({ "id": i["id"] }));
        assert_eq!(d["title"], "Dentist");
        assert_eq!(d["reminders"].as_array().unwrap().len(), 1);
        let err = p
            .execute("get_item", json!({ "id": "nope" }), &Actor::User)
            .unwrap_err();
        assert_eq!(err.to_string(), "item nope not found");
    }

    #[test]
    fn day_summary_counts() {
        let mut p = crate::planner::tests::planner();
        // NOW is 2026-10-14 09:00 local.
        run(
            &mut p,
            "create_item",
            json!({ "title": "Gym", "start": "2026-10-14T17:30", "duration_min": 45 }),
        );
        let done = run(
            &mut p,
            "create_item",
            json!({ "title": "Run", "start": "2026-10-14T07:00", "duration_min": 30 }),
        );
        run(&mut p, "complete_item", json!({ "id": done["id"] }));
        run(
            &mut p,
            "create_item",
            json!({ "title": "Late tax", "start": "2026-10-13T10:00" }),
        );
        run(&mut p, "create_item", json!({ "title": "Inbox thing" }));
        let s = run(&mut p, "get_day_summary", json!({}));
        assert_eq!(s["date"], "2026-10-14");
        assert_eq!(s["planned_min"], 75);
        assert_eq!(s["open_count"], 1);
        assert_eq!(s["done_count"], 1);
        assert_eq!(s["overdue_count"], 1);
        assert_eq!(s["inbox_count"], 1);
    }

    #[test]
    fn overdue_means_left_on_an_earlier_day_or_past_deadline() {
        let mut p = crate::planner::tests::planner();
        // NOW is 2026-10-14 09:00 local.
        let titles = |p: &mut crate::Planner| -> Vec<String> {
            run(p, "list_overdue", json!({}))
                .as_array()
                .unwrap()
                .iter()
                .map(|i| i["title"].as_str().unwrap().to_owned())
                .collect()
        };
        run(
            &mut p,
            "create_item",
            json!({ "title": "Renew rego", "start": "2026-10-12T19:30", "duration_min": 20 }),
        );
        run(
            &mut p,
            "create_item",
            json!({ "title": "Old all-day", "date": "2026-10-13" }),
        );
        run(
            &mut p,
            "create_item",
            json!({ "title": "Deadline passed", "due": "2026-10-14T08:00" }),
        );
        run(
            &mut p,
            "create_item",
            json!({ "title": "Earlier today", "start": "2026-10-14T07:00" }),
        );
        run(
            &mut p,
            "create_item",
            json!({ "title": "Past party", "kind": "event", "start": "2026-10-12T19:00" }),
        );
        run(&mut p, "create_item", json!({ "title": "Someday" }));
        let done = run(
            &mut p,
            "create_item",
            json!({ "title": "Done late", "start": "2026-10-11T10:00" }),
        );
        run(&mut p, "complete_item", json!({ "id": done["id"] }));

        assert_eq!(
            titles(&mut p),
            ["Renew rego", "Old all-day", "Deadline passed"]
        );
        let summary = run(&mut p, "get_day_summary", json!({}));
        assert_eq!(summary["overdue_count"], 3);
    }

    #[test]
    fn search_finds_by_tag() {
        let mut p = crate::planner::tests::planner();
        run(
            &mut p,
            "create_item",
            json!({ "title": "Presents", "tags": ["shopping"] }),
        );
        let found = run(&mut p, "search_items", json!({ "query": "shop" }));
        assert_eq!(found.as_array().unwrap().len(), 1);
    }

    #[test]
    fn blockout_windows_apply_lifted_dates() {
        let mut p = crate::planner::tests::planner();
        let work = run(
            &mut p,
            "create_blockout",
            json!({ "label": "Work", "weekdays": ["mon","tue","wed","thu","fri"],
                    "start_time": "08:30", "end_time": "17:00" }),
        );
        run(
            &mut p,
            "create_blockout",
            json!({ "label": "Sleep", "weekdays": ["mon","tue","wed","thu","fri","sat","sun"],
                    "start_time": "23:00", "end_time": "06:30", "permits_fixed": false }),
        );
        run(
            &mut p,
            "add_blockout_exception",
            json!({ "blockout_id": work["id"], "date": "2026-10-15", "note": "Leave" }),
        );

        let out = run(
            &mut p,
            "list_blockout_windows",
            json!({ "start": "2026-10-14", "end": "2026-10-15" }),
        );
        assert_eq!(out["timezone"], "Australia/Melbourne");
        let windows: Vec<(String, String, String)> = out["windows"]
            .as_array()
            .unwrap()
            .iter()
            .map(|w| {
                (
                    w["label"].as_str().unwrap().to_owned(),
                    w["date"].as_str().unwrap().to_owned(),
                    w["start"].as_str().unwrap().to_owned(),
                )
            })
            .collect();
        let expected = [
            ("Sleep", "2026-10-13", "2026-10-13T12:00:00Z"),
            ("Work", "2026-10-14", "2026-10-13T21:30:00Z"),
            ("Sleep", "2026-10-14", "2026-10-14T12:00:00Z"),
            // Thursday's Work is lifted.
            ("Sleep", "2026-10-15", "2026-10-15T12:00:00Z"),
        ];
        let expected: Vec<(String, String, String)> = expected
            .iter()
            .map(|(a, b, c)| ((*a).to_owned(), (*b).to_owned(), (*c).to_owned()))
            .collect();
        assert_eq!(windows, expected);
        assert_eq!(out["windows"][0]["permits_fixed"], false);

        let err = p
            .execute(
                "list_blockout_windows",
                json!({ "start": "2026-10-15", "end": "2026-10-14" }),
                &Actor::User,
            )
            .unwrap_err();
        assert!(err.to_string().contains("before"));
    }
}

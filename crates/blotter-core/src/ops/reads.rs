//! Read-only operations. These never write and always run immediately,
//! even in AI propose mode.

use chrono::{Duration, NaiveDate};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::input::{DateInput, NoArgs, start_of_day};
use super::{Ctx, OpDef};
use crate::error::{CoreError, Result};
use crate::model::{Blockout, BlockoutException, Item, ItemStatus, Reminder};
use crate::store;
use crate::store::items::ItemRange;
use crate::time::Instant;

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
    /// Open items, anywhere, whose end or due time has passed.
    pub overdue_count: usize,
    pub inbox_count: usize,
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

fn list_agenda(ctx: &mut Ctx<'_>, a: ListAgendaArgs) -> Result<Agenda> {
    let start = a.start.0;
    let end = a.end.map(|d| d.0).unwrap_or(start);
    if end < start {
        return Err(CoreError::invalid("`end` is before `start`"));
    }
    if (end - start).num_days() >= MAX_AGENDA_DAYS {
        return Err(CoreError::invalid(format!(
            "range is longer than {MAX_AGENDA_DAYS} days"
        )));
    }
    let mut items = store::items::list_in_range(ctx.conn, day_range(ctx, start, end))?;
    if a.include_done == Some(false) {
        items.retain(|i| i.status != ItemStatus::Done);
    }
    Ok(Agenda {
        start,
        end,
        timezone: ctx.settings.tz.name().to_owned(),
        items,
    })
}

fn get_item(ctx: &mut Ctx<'_>, a: GetItemArgs) -> Result<ItemDetail> {
    Ok(ItemDetail {
        item: store::items::require(ctx.conn, &a.id)?,
        reminders: store::reminders::list_for_item(ctx.conn, &a.id)?,
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

fn get_day_summary(ctx: &mut Ctx<'_>, a: DaySummaryArgs) -> Result<DaySummary> {
    let date = a.date.map(|d| d.0).unwrap_or_else(|| today(ctx));
    let range = day_range(ctx, date, date);
    let items = store::items::list_in_range(ctx.conn, range)?;
    let planned_min = items
        .iter()
        .filter(|i| i.status != ItemStatus::Cancelled)
        .filter_map(|i| Some(clip_minutes(i.start_at?, i.end_at?, range.start, range.end)))
        .sum();
    let overdue_count: i64 = ctx.conn.query_row(
        "SELECT COUNT(*) FROM items WHERE status = 'open' \
           AND ((end_at IS NOT NULL AND end_at < ?1) OR (due_at IS NOT NULL AND due_at < ?1))",
        [crate::time::fmt_instant(ctx.now)],
        |r| r.get(0),
    )?;
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
        overdue_count: usize::try_from(overdue_count).unwrap_or(0),
        inbox_count: store::items::list_inbox(ctx.conn)?
            .iter()
            .filter(|i| i.status == ItemStatus::Open)
            .count(),
        items,
    })
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
}

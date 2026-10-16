//! Completion history and streaks (roadmap §5.3 History, Phase 11).

use std::collections::BTreeSet;

use chrono::{Datelike, Duration, NaiveDate};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::input::{DateInput, start_of_day};
use super::reads::today;
use super::{Ctx, OpDef};
use crate::error::{CoreError, Result};
use crate::model::Item;
use crate::settings::{KEY_WEEK_START, WeekStart};
use crate::store;

/// Longest range one call lists.
const MAX_DAYS: i64 = 400;
/// Most completions one call returns.
const MAX_ITEMS: usize = 2000;
/// Days in the daily-count strip.
const RECENT_DAYS: i64 = 14;

pub(super) fn defs() -> Vec<OpDef> {
    vec![OpDef::new(
        "list_completions",
        "List completed items between two dates (inclusive, user's timezone; defaults to the \
         last 30 days), newest first, each with when and by whom it was completed (user, ai \
         or mcp). Optional `tag` and `query` (title or notes) filters. Also returns the \
         current and best streak (consecutive days with a completion), counts for today, \
         this week and this month, and the last 14 days' counts.",
        false,
        list_completions,
    )]
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ListCompletionsArgs {
    /// First day; defaults to 29 days before `end`.
    pub start: Option<DateInput>,
    /// Last day, inclusive; defaults to today.
    pub end: Option<DateInput>,
    /// Only items with this tag.
    #[schemars(length(max = 100))]
    pub tag: Option<String>,
    /// Only items whose title or notes contain this (case-insensitive).
    #[schemars(length(max = 200))]
    pub query: Option<String>,
}

/// Who completed an item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DoneBy {
    User,
    Ai,
    Mcp,
    System,
}

impl DoneBy {
    /// From a change-log actor (`user`, `ai:<provider>/<model>`, `mcp`, `system`);
    /// unknown (no change row left) counts as the user.
    fn from_actor(actor: Option<&str>) -> Self {
        match actor {
            Some(a) if a.starts_with("ai:") || a == "ai" => Self::Ai,
            Some("mcp") => Self::Mcp,
            Some("system") => Self::System,
            _ => Self::User,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Completion {
    #[serde(flatten)]
    pub item: Item,
    pub done_by: DoneBy,
    /// The local day it was completed on.
    pub done_on: NaiveDate,
}

#[derive(Debug, Serialize)]
pub struct DayCount {
    pub date: NaiveDate,
    pub count: usize,
}

#[derive(Debug, Default, PartialEq, Eq, Serialize)]
pub struct Streaks {
    /// Consecutive days with a completion up to today (or yesterday, while
    /// today has none yet).
    pub current: u32,
    pub best: u32,
}

#[derive(Debug, Serialize)]
pub struct History {
    pub start: NaiveDate,
    pub end: NaiveDate,
    pub timezone: String,
    pub completions: Vec<Completion>,
    /// True when the range held more than were returned.
    pub truncated: bool,
    pub streaks: Streaks,
    pub today: usize,
    pub this_week: usize,
    pub this_month: usize,
    /// The last 14 days up to today, oldest first.
    pub recent: Vec<DayCount>,
}

/// Streaks over the distinct `days` with a completion.
pub fn streaks(days: &BTreeSet<NaiveDate>, today: NaiveDate) -> Streaks {
    let mut best = 0;
    let mut run = 0;
    let mut prev: Option<NaiveDate> = None;
    for &d in days {
        run = match prev {
            Some(p) if p.succ_opt() == Some(d) => run + 1,
            _ => 1,
        };
        best = best.max(run);
        prev = Some(d);
    }
    // Today without a completion yet doesn't break the streak.
    let mut day = if days.contains(&today) {
        today
    } else {
        today.pred_opt().unwrap_or(today)
    };
    let mut current = 0;
    while days.contains(&day) {
        current += 1;
        match day.pred_opt() {
            Some(p) => day = p,
            None => break,
        }
    }
    Streaks { current, best }
}

fn week_start(ctx: &Ctx<'_>, today: NaiveDate) -> Result<NaiveDate> {
    let ws: WeekStart = store::settings::get(ctx.conn, KEY_WEEK_START)?.unwrap_or_default();
    let first = match ws {
        WeekStart::Mon => chrono::Weekday::Mon,
        WeekStart::Sun => chrono::Weekday::Sun,
        WeekStart::Sat => chrono::Weekday::Sat,
    };
    let back = (7 + today.weekday().num_days_from_monday() - first.num_days_from_monday()) % 7;
    Ok(today - Duration::days(i64::from(back)))
}

fn list_completions(ctx: &mut Ctx<'_>, a: ListCompletionsArgs) -> Result<History> {
    let tz = ctx.settings.tz;
    let today = today(ctx);
    let end = a.end.map(|d| d.0).unwrap_or(today);
    let start = a.start.map(|d| d.0).unwrap_or(end - Duration::days(29));
    if end < start {
        return Err(CoreError::invalid("`end` is before `start`"));
    }
    if (end - start).num_days() >= MAX_DAYS {
        return Err(CoreError::invalid(format!(
            "range is longer than {MAX_DAYS} days"
        )));
    }
    let local = |t: crate::time::Instant| t.with_timezone(&tz).date_naive();

    let tag = a
        .tag
        .map(|t| t.trim().to_lowercase())
        .filter(|t| !t.is_empty());
    let query = a
        .query
        .map(|q| q.trim().to_lowercase())
        .filter(|q| !q.is_empty());
    let from = start_of_day(tz, start);
    let to = start_of_day(tz, end + Duration::days(1));
    let mut completions: Vec<Completion> = store::items::list_completed(ctx.conn, from, to)?
        .into_iter()
        .filter(|(item, _)| {
            tag.as_ref()
                .is_none_or(|t| item.tags.iter().any(|x| x.to_lowercase() == *t))
                && query.as_ref().is_none_or(|q| {
                    item.title.to_lowercase().contains(q) || item.notes.to_lowercase().contains(q)
                })
        })
        .filter_map(|(item, actor)| {
            let done_on = local(item.completed_at?);
            Some(Completion {
                done_by: DoneBy::from_actor(actor.as_deref()),
                done_on,
                item,
            })
        })
        .collect();
    let truncated = completions.len() > MAX_ITEMS;
    completions.truncate(MAX_ITEMS);

    // Stats count everything, whatever the filters.
    let all_days: Vec<NaiveDate> = store::items::completion_times(ctx.conn)?
        .into_iter()
        .map(local)
        .collect();
    let distinct: BTreeSet<NaiveDate> = all_days.iter().copied().collect();
    let count_from = |first: NaiveDate| {
        all_days
            .iter()
            .filter(|d| **d >= first && **d <= today)
            .count()
    };
    let month_start = today.with_day(1).unwrap_or(today);
    let recent = (0..RECENT_DAYS)
        .rev()
        .map(|back| {
            let date = today - Duration::days(back);
            DayCount {
                date,
                count: all_days.iter().filter(|d| **d == date).count(),
            }
        })
        .collect();

    Ok(History {
        start,
        end,
        timezone: tz.name().to_owned(),
        completions,
        truncated,
        streaks: streaks(&distinct, today),
        today: count_from(today),
        this_week: count_from(week_start(ctx, today)?),
        this_month: count_from(month_start),
        recent,
    })
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;
    use crate::model::Actor;

    fn d(m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, m, day).unwrap()
    }

    #[test]
    fn streak_rules() {
        let days: BTreeSet<_> = [d(10, 1), d(10, 2), d(10, 3), d(10, 7), d(10, 8), d(10, 9)]
            .into_iter()
            .collect();
        // Today (the 10th) has nothing yet: the run up to yesterday still counts.
        assert_eq!(
            streaks(&days, d(10, 10)),
            Streaks {
                current: 3,
                best: 3
            }
        );
        assert_eq!(streaks(&days, d(10, 9)).current, 3);
        // A missed day ends it.
        assert_eq!(streaks(&days, d(10, 11)).current, 0);
        assert_eq!(streaks(&BTreeSet::new(), d(10, 11)), Streaks::default());
    }

    fn run(p: &mut crate::Planner, op: &str, args: Value, actor: &Actor) -> Value {
        p.execute(op, args, actor).unwrap().result
    }

    #[test]
    fn completions_from_every_path_with_local_days() {
        // The planner's clock: 09:00 Wed 14 Oct in Melbourne (22:00Z on the 13th).
        let mut p = crate::planner::tests::planner();
        let user = Actor::User;
        let ai: Actor = "ai:anthropic/claude".parse().unwrap();
        let done = |p: &mut crate::Planner, title: &str, tags: &[&str], by: &Actor| {
            let item = run(
                p,
                "create_item",
                json!({ "title": title, "tags": tags }),
                &user,
            );
            run(p, "complete_item", json!({ "id": item["id"] }), by);
        };
        done(&mut p, "Pay bill", &["home"], &user);
        done(&mut p, "Reply to strata", &["home"], &ai);
        done(&mut p, "Hardware store", &["shopping"], &Actor::Mcp);
        let undone = run(
            &mut p,
            "create_item",
            json!({ "title": "Not really" }),
            &user,
        );
        run(
            &mut p,
            "complete_item",
            json!({ "id": undone["id"] }),
            &user,
        );
        run(&mut p, "undo", json!({}), &user);

        let h = run(&mut p, "list_completions", json!({}), &user);
        let by: Vec<(String, String)> = h["completions"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| {
                (
                    c["title"].as_str().unwrap().to_owned(),
                    c["done_by"].as_str().unwrap().to_owned(),
                )
            })
            .collect();
        assert_eq!(by.len(), 3, "the undone one is gone: {by:?}");
        for (title, who) in [
            ("Pay bill", "user"),
            ("Reply to strata", "ai"),
            ("Hardware store", "mcp"),
        ] {
            assert!(
                by.contains(&(title.into(), who.into())),
                "{title} by {who}: {by:?}"
            );
        }
        // Completed at 22:00Z on the 13th, which is the 14th in Melbourne.
        assert_eq!(h["completions"][0]["done_on"], "2026-10-14");
        assert_eq!(
            (&h["today"], &h["this_week"], &h["this_month"]),
            (&json!(3), &json!(3), &json!(3))
        );
        assert_eq!(h["streaks"], json!({ "current": 1, "best": 1 }));
        assert_eq!(h["recent"].as_array().unwrap().len(), 14);
        assert_eq!(h["recent"][13], json!({ "date": "2026-10-14", "count": 3 }));

        let home = run(
            &mut p,
            "list_completions",
            json!({ "tag": "Home", "query": "strata" }),
            &user,
        );
        assert_eq!(home["completions"].as_array().unwrap().len(), 1);
        assert_eq!(home["this_month"], 3, "stats ignore the filters");

        let old = run(
            &mut p,
            "list_completions",
            json!({ "start": "2026-09-01", "end": "2026-09-30" }),
            &user,
        );
        assert!(old["completions"].as_array().unwrap().is_empty());
        assert!(
            p.execute(
                "list_completions",
                json!({ "start": "2026-10-14", "end": "2026-10-01" }),
                &user
            )
            .is_err()
        );
    }
}

//! Scheduling operations over the slot engine, and the rules every path that
//! places an item must obey: a flexible task never sits in a block-out, and
//! overlaps are reported as conflicts rather than prevented.

use std::cmp::Reverse;
use std::collections::HashSet;

use chrono::{Duration, NaiveDate};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::input::{DateTimeInput, start_of_day};
use super::items::{ENTITY as ITEM_ENTITY, finalize};
use super::{Ctx, OpDef};
use crate::error::{CoreError, Result};
use crate::model::{Anchoring, Item, ItemKind, ItemStatus};
use crate::slots::{Interval, Slot, SlotEngine, overlapping};
use crate::store;
use crate::store::items::ItemRange;
use crate::time::Instant;
use crate::windows::{self, BlockoutWindow};

/// Tasks with this tag aren't slotted on holidays.
const WORK_TAG: &str = "work";
/// Tasks with this tag are slotted first, so they get the holiday's days.
const HOLIDAY_TAG: &str = "holiday";

fn has_tag(item: &Item, tag: &str) -> bool {
    item.tags.iter().any(|t| t.eq_ignore_ascii_case(tag))
}

/// Whether `item` is a flexible task carrying a tag whose tasks share time
/// (`sharing`, from `store::tags::sharing`): it may sit over other tasks, and
/// they over it, but not over events or fixed items.
pub(crate) fn shares_time(item: &Item, sharing: &HashSet<String>) -> bool {
    item.kind == ItemKind::Task
        && item.anchoring == Anchoring::Flexible
        && item
            .tags
            .iter()
            .any(|t| sharing.contains(&t.to_lowercase()))
}

/// Whether an overlap between `a` and `b` is fine: one shares time and the
/// other is a flexible task.
fn may_overlap(a: &Item, b: &Item, sharing: &HashSet<String>) -> bool {
    let flexible_task = |i: &Item| i.kind == ItemKind::Task && i.anchoring == Anchoring::Flexible;
    (shares_time(a, sharing) && flexible_task(b)) || (shares_time(b, sharing) && flexible_task(a))
}

/// Each holiday overlapping `[from, until)` as one whole-days window.
fn holiday_windows(ctx: &Ctx<'_>, from: Instant, until: Instant) -> Result<Vec<BlockoutWindow>> {
    let tz = ctx.settings.tz;
    let first = from.with_timezone(&tz).date_naive();
    let last = until.with_timezone(&tz).date_naive();
    Ok(store::holidays::overlapping(ctx.conn, first, last)?
        .into_iter()
        .map(|h| BlockoutWindow {
            blockout_id: h.id,
            label: format!("Holiday: {}", h.label),
            date: h.start_date,
            start: start_of_day(tz, h.start_date),
            end: start_of_day(tz, h.end_date.succ_opt().unwrap_or(h.end_date)),
            permits_fixed: true,
            color: h.color,
            subscription_id: None,
            calendar: None,
        })
        .collect())
}

/// The hours outside `[ws, we)` on each local day of `[lo, hi)`, as windows,
/// so the engine only finds time inside a task's usual time of day.
fn outside_hours(
    tz: chrono_tz::Tz,
    lo: Instant,
    hi: Instant,
    ws: chrono::NaiveTime,
    we: chrono::NaiveTime,
) -> Vec<BlockoutWindow> {
    let mut out = Vec::new();
    let last = hi.with_timezone(&tz).date_naive();
    let mut day = lo.with_timezone(&tz).date_naive();
    while day <= last {
        let next = day.succ_opt().unwrap_or(day);
        for (start, end) in [
            (
                start_of_day(tz, day),
                super::input::resolve_local(tz, day.and_time(ws)),
            ),
            (
                super::input::resolve_local(tz, day.and_time(we)),
                start_of_day(tz, next),
            ),
        ] {
            if end > start {
                out.push(BlockoutWindow {
                    blockout_id: String::new(),
                    label: "Outside its usual time".into(),
                    date: day,
                    start,
                    end,
                    permits_fixed: true,
                    color: None,
                    subscription_id: None,
                    calendar: None,
                });
            }
        }
        if next == day {
            break;
        }
        day = next;
    }
    out
}

/// How far ahead slot searches look when no `until` is given.
const DEFAULT_SEARCH_DAYS: i64 = 14;
/// Longest span one search covers.
const MAX_SEARCH_DAYS: i64 = 62;

pub(super) fn defs() -> Vec<OpDef> {
    vec![
        OpDef::new(
            "find_free_slots",
            "Find free time for a flexible task of `duration_min`: within planning hours, \
             outside block-outs and clear of scheduled items (plus the buffer). Returns the \
             earliest fit in each free gap, soonest first, never in the past. Defaults to the \
             next 14 days; pass `until` (e.g. a due date) to search less. Tasks tagged with a \
             tag that shares time (list_tags) don't count as busy; with `shares_time` (or an \
             `exclude_item` carrying such a tag) no task does, only events and fixed items.",
            false,
            find_free_slots,
        ),
        OpDef::new(
            "auto_slot",
            "Place flexible tasks into free time: the given `ids`, or every open Inbox task. \
             Orders by due date, then priority, then longest first, and puts each in the \
             earliest free slot before its due date (a day-only task stays on its day). Returns \
             what was placed and, for anything that couldn't be, the reason and the longest \
             free gap. Nothing is dropped silently; unplaced tasks stay where they were.",
            true,
            auto_slot,
        ),
    ]
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FindFreeSlotsArgs {
    #[schemars(range(min = 5, max = 1440))]
    pub duration_min: u32,
    /// Earliest start; defaults to now and is never earlier than now.
    pub from: Option<DateTimeInput>,
    /// Latest end, e.g. a deadline. Defaults to 14 days after `from`.
    pub until: Option<DateTimeInput>,
    /// Defaults to 10.
    #[schemars(range(min = 1, max = 100))]
    pub limit: Option<u32>,
    /// Ignore this item's own time, when looking for somewhere to move it.
    pub exclude_item: Option<String>,
    /// Looking for time for a task that can share time with other tasks:
    /// only events and fixed items are busy. Defaults to whether
    /// `exclude_item` carries a tag that shares time.
    pub shares_time: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct FreeSlots {
    pub timezone: String,
    pub slots: Vec<Slot>,
}

/// `[from, until)` for a search: never before now, 14 days by default.
fn search_window(
    ctx: &Ctx<'_>,
    from: Option<DateTimeInput>,
    until: Option<DateTimeInput>,
) -> Result<(Instant, Instant)> {
    let tz = ctx.settings.tz;
    let from = from.map(|f| f.resolve(tz)).unwrap_or(ctx.now).max(ctx.now);
    let until = until
        .map(|u| u.resolve(tz))
        .unwrap_or(from + Duration::days(DEFAULT_SEARCH_DAYS));
    if until <= from {
        return Err(CoreError::invalid("`until` must be after `from` and now"));
    }
    if until - from > Duration::days(MAX_SEARCH_DAYS) {
        return Err(CoreError::invalid(format!(
            "search at most {MAX_SEARCH_DAYS} days at a time"
        )));
    }
    Ok((from, until))
}

fn buffer(ctx: &Ctx<'_>) -> Duration {
    Duration::minutes(i64::from(ctx.settings.slot_buffer_min))
}

fn find_free_slots(ctx: &mut Ctx<'_>, a: FindFreeSlotsArgs) -> Result<FreeSlots> {
    let (from, until) = search_window(ctx, a.from, a.until)?;
    let buffer = buffer(ctx);
    let blockouts = windows::load(ctx.conn, ctx.settings.tz, from, until)?;
    let sharing = store::tags::sharing(ctx.conn)?;
    let shares = match (a.shares_time, &a.exclude_item) {
        (Some(s), _) => s,
        (None, Some(id)) => {
            store::items::get(ctx.conn, id)?.is_some_and(|i| shares_time(&i, &sharing))
        }
        (None, None) => false,
    };
    let exclude: HashSet<String> = a.exclude_item.into_iter().collect();
    let busy = busy_time(ctx, from - buffer, until + buffer, &exclude, &sharing)?;
    let engine = SlotEngine {
        tz: ctx.settings.tz,
        planning: &ctx.settings.planning_hours,
        blockouts: &blockouts,
        busy: busy.for_task(shares),
        buffer,
    };
    let limit = a.limit.unwrap_or(10) as usize;
    Ok(FreeSlots {
        timezone: ctx.settings.tz.name().to_owned(),
        slots: engine.slots(
            Duration::minutes(i64::from(a.duration_min)),
            from,
            until,
            limit,
        ),
    })
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AutoSlotArgs {
    /// Tasks to place. Omit to place every open task in the Inbox.
    pub ids: Option<Vec<String>>,
    /// Earliest start; defaults to now and is never earlier than now.
    pub from: Option<DateTimeInput>,
    /// Latest end. Defaults to 14 days after `from`.
    pub until: Option<DateTimeInput>,
}

#[derive(Debug, Serialize)]
pub struct PlacedTask {
    pub item: Item,
    pub start: Instant,
    pub end: Instant,
    /// Where it was before: neither set means it came from the Inbox.
    pub previous_start: Option<Instant>,
    pub previous_date: Option<NaiveDate>,
}

#[derive(Debug, Serialize)]
pub struct Gap {
    pub start: Instant,
    pub end: Instant,
    pub minutes: i64,
}

#[derive(Debug, Serialize)]
pub struct UnplacedTask {
    pub id: String,
    pub title: String,
    pub duration_min: u32,
    pub due_at: Option<Instant>,
    /// A readable sentence, e.g. "No free 2h slot before its due date …".
    pub reason: String,
    /// The longest free gap in the window searched, to suggest shortening.
    pub longest_gap: Option<Gap>,
}

#[derive(Debug, Serialize)]
pub struct AutoSlotResult {
    pub timezone: String,
    pub placed: Vec<PlacedTask>,
    pub unplaced: Vec<UnplacedTask>,
}

/// Why `item` can't be auto-slotted at all, if it can't.
fn ineligible(item: &Item) -> Option<&'static str> {
    if item.kind != ItemKind::Task {
        Some("Only tasks are auto-slotted.")
    } else if item.anchoring == Anchoring::Fixed {
        Some("It's fixed, so it stays where you put it.")
    } else if item.status != ItemStatus::Open {
        Some("It's already done.")
    } else if item.rrule.is_some() {
        Some("Recurring tasks aren't auto-slotted yet.")
    } else if item.end_date.is_some() {
        Some("It spans several days.")
    } else {
        None
    }
}

/// "45m", "2h", "1h 15m".
fn minutes_text(m: i64) -> String {
    match (m / 60, m % 60) {
        (0, m) => format!("{m}m"),
        (h, 0) => format!("{h}h"),
        (h, m) => format!("{h}h {m}m"),
    }
}

fn auto_slot(ctx: &mut Ctx<'_>, a: AutoSlotArgs) -> Result<AutoSlotResult> {
    let tz = ctx.settings.tz;
    let (from, until) = search_window(ctx, a.from, a.until)?;
    let candidates: Vec<Item> = match a.ids {
        Some(ids) => ids
            .iter()
            .map(|id| store::items::require(ctx.conn, id))
            .collect::<Result<_>>()?,
        None => store::items::list_inbox(ctx.conn)?
            .into_iter()
            .filter(|i| i.status == ItemStatus::Open)
            .collect(),
    };
    let default_min = ctx.settings.default_task_duration_min;
    let unplaced_entry = |item: &Item, reason: String, gap: Option<Gap>| UnplacedTask {
        id: item.id.clone(),
        title: item.title.clone(),
        duration_min: item.duration_min.unwrap_or(default_min),
        due_at: item.due_at,
        reason,
        longest_gap: gap,
    };

    let mut unplaced = Vec::new();
    let mut tasks = Vec::new();
    for item in candidates {
        match ineligible(&item) {
            Some(reason) => unplaced.push(unplaced_entry(&item, reason.to_owned(), None)),
            None => tasks.push(item),
        }
    }
    let (placed, missed) = place(ctx, tasks, from, until, false)?;
    unplaced.extend(missed.into_iter().map(|(_, u)| u));
    Ok(AutoSlotResult {
        timezone: tz.name().to_owned(),
        placed,
        unplaced,
    })
}

/// What [`place`] returns: the placed tasks, and the rest (as stored) with why.
pub(crate) type Placement = (Vec<PlacedTask>, Vec<(Item, UnplacedTask)>);

/// Places `tasks` (as stored) with the slot engine, in auto-slot order. Each
/// placed task is written and change-logged, and blocks the next. Returns the
/// placed tasks and, for the rest, the stored item with the reason. With
/// `rollover`, a day-only task isn't held to its (past) day and unplaced
/// tasks are described as going back to the Inbox.
pub(crate) fn place(
    ctx: &mut Ctx<'_>,
    mut tasks: Vec<Item>,
    from: Instant,
    until: Instant,
    rollover: bool,
) -> Result<Placement> {
    let tz = ctx.settings.tz;
    let default_min = ctx.settings.default_task_duration_min;
    let unplaced_entry = |item: &Item, reason: String, gap: Option<Gap>| UnplacedTask {
        id: item.id.clone(),
        title: item.title.clone(),
        duration_min: item.duration_min.unwrap_or(default_min),
        due_at: item.due_at,
        reason,
        longest_gap: gap,
    };
    let mut placed = Vec::new();
    let mut unplaced = Vec::new();
    // Holiday tasks first (they get first pick of a holiday's days), then due
    // date (none last), then priority, then longest first.
    tasks.sort_by_key(|i| {
        (
            !has_tag(i, HOLIDAY_TAG),
            i.due_at.is_none(),
            i.due_at,
            Reverse(i.priority),
            Reverse(i.duration_min.unwrap_or(default_min)),
        )
    });

    let buffer = buffer(ctx);
    let blockouts = windows::load(ctx.conn, ctx.settings.tz, from, until)?;
    // Work tasks also stay off holidays: the holidays join their block-outs.
    let mut work_blockouts = blockouts.clone();
    work_blockouts.extend(holiday_windows(ctx, from, until)?);
    // The tasks being placed don't block each other's old spots.
    let moving: HashSet<String> = tasks.iter().map(|i| i.id.clone()).collect();
    let sharing = store::tags::sharing(ctx.conn)?;
    let mut busy = busy_time(ctx, from - buffer, until + buffer, &moving, &sharing)?;
    let fmt_day = |t: Instant| t.with_timezone(&tz).format("%a %-d %b").to_string();
    let fmt_time = |t: Instant| t.with_timezone(&tz).format("%H:%M").to_string();

    for mut item in tasks {
        let minutes = i64::from(item.duration_min.unwrap_or(default_min));
        let duration = Duration::minutes(minutes);
        let (mut lo, mut hi) = (from, until);
        let mut limit = format!("in the next {} days", (until - from).num_days().max(1));
        if let Some(day) = item.start_date.filter(|_| !rollover) {
            let day_start = start_of_day(tz, day);
            let day_end = day
                .succ_opt()
                .map(|d| start_of_day(tz, d))
                .unwrap_or(day_start);
            lo = lo.max(day_start);
            hi = hi.min(day_end);
            limit = format!("on {}", day.format("%a %-d %b"));
        }
        if let Some(due) = item.due_at
            && due < hi
        {
            hi = due;
            limit = format!("before its due date ({} {})", fmt_day(due), fmt_time(due));
        }
        // A do-window opens on its not-before date.
        let opens = item.not_before.map(|d| start_of_day(tz, d));
        if let Some(opens) = opens
            && opens > lo
        {
            lo = opens;
        }
        let stays = if rollover {
            "It goes back to the Inbox."
        } else if item.is_inbox() {
            "It stays in the Inbox."
        } else {
            "It stays where it was."
        };
        if hi <= lo {
            let reason = if item.due_at.is_some_and(|d| d <= from) {
                format!("Its due date has passed. {stays}")
            } else if item.start_date.is_some() {
                format!("Its day has passed. {stays}")
            } else if let Some(opens) = opens.filter(|o| *o >= until) {
                format!(
                    "Its window opens {}, later than this search. {stays}",
                    fmt_day(opens)
                )
            } else {
                format!("Nothing to search {limit}. {stays}")
            };
            unplaced.push((item.clone(), unplaced_entry(&item, reason, None)));
            continue;
        }

        let base = if has_tag(&item, WORK_TAG) {
            &work_blockouts
        } else {
            &blockouts
        };
        let shares = shares_time(&item, &sharing);
        let engine = SlotEngine {
            tz,
            planning: &ctx.settings.planning_hours,
            blockouts: base,
            busy: busy.for_task(shares),
            buffer,
        };
        // Its usual time of day first, then any free time.
        let usual = match (item.window_start, item.window_end) {
            (Some(ws), Some(we)) => {
                let mut outside = base.clone();
                outside.extend(outside_hours(tz, lo, hi, ws, we));
                SlotEngine {
                    tz,
                    planning: &ctx.settings.planning_hours,
                    blockouts: &outside,
                    busy: busy.for_task(shares),
                    buffer,
                }
                .slots(duration, lo, hi, 1)
                .into_iter()
                .next()
            }
            _ => None,
        };
        let Some(slot) = usual.or_else(|| engine.slots(duration, lo, hi, 1).into_iter().next())
        else {
            let gap = engine
                .free(lo, hi)
                .into_iter()
                .max_by_key(|(s, e)| *e - *s)
                .map(|(start, end)| Gap {
                    start,
                    end,
                    minutes: (end - start).num_minutes(),
                });
            let longest = match &gap {
                Some(g) if g.minutes > 0 => format!(
                    " The longest free gap is {}, {} {}–{}.",
                    minutes_text(g.minutes),
                    fmt_day(g.start),
                    fmt_time(g.start),
                    fmt_time(g.end),
                ),
                _ => " There's no free time there at all.".to_owned(),
            };
            let reason = format!(
                "No free {} slot {limit}.{longest} {stays}",
                minutes_text(minutes)
            );
            unplaced.push((item.clone(), unplaced_entry(&item, reason, gap)));
            continue;
        };

        let before = item.clone();
        item.all_day = false;
        item.start_date = None;
        item.end_date = None;
        item.start_at = Some(slot.start);
        item.end_at = Some(slot.end);
        item.duration_min = Some(u32::try_from(minutes).unwrap_or(default_min));
        item.updated_at = ctx.now;
        finalize(ctx, &mut item)?;
        store::items::update(ctx.conn, &item)?;
        ctx.record(ITEM_ENTITY, &item.id, Some(&before), Some(&item))?;
        if !shares {
            busy.all.push((slot.start, slot.end));
        }
        placed.push(PlacedTask {
            start: slot.start,
            end: slot.end,
            previous_start: before.start_at,
            previous_date: before.start_date,
            item,
        });
    }
    Ok((placed, unplaced))
}

/// Scheduled time, split by what it keeps a task out of.
pub(crate) struct Busy {
    /// Everything except tasks that share time: busy for an ordinary task.
    pub all: Vec<Interval>,
    /// Events, reminders and fixed items: busy for a task that shares time.
    pub fixed: Vec<Interval>,
}

impl Busy {
    pub fn for_task(&self, shares_time: bool) -> &[Interval] {
        if shares_time { &self.fixed } else { &self.all }
    }
}

/// Time taken by open, timed items overlapping `[from, to)`, except `exclude`.
/// Tasks that share time (`sharing` tags) take none.
pub(crate) fn busy_time(
    ctx: &Ctx<'_>,
    from: Instant,
    to: Instant,
    exclude: &HashSet<String>,
    sharing: &HashSet<String>,
) -> Result<Busy> {
    let tz = ctx.settings.tz;
    let range = ItemRange {
        start: from,
        end: to,
        start_date: from.with_timezone(&tz).date_naive(),
        end_date: to.with_timezone(&tz).date_naive(),
    };
    let mut busy = Busy {
        all: Vec::new(),
        fixed: Vec::new(),
    };
    for i in crate::recurrence::agenda(ctx.conn, range, tz)? {
        if i.status != ItemStatus::Open || exclude.contains(&i.id) || shares_time(&i, sharing) {
            continue;
        }
        let (Some(start), Some(end)) = (i.start_at, i.end_at) else {
            continue;
        };
        busy.all.push((start, end));
        if !(i.kind == ItemKind::Task && i.anchoring == Anchoring::Flexible) {
            busy.fixed.push((start, end));
        }
    }
    Ok(busy)
}

/// A scheduling problem with one item (roadmap §4.5 rule 3): it overlaps
/// other items, or it is fixed inside a block-out that warns about fixed
/// items (e.g. Sleep). Allowed, but shown in the UI and to the AI.
#[derive(Debug, Clone, Serialize)]
pub struct Conflict {
    pub item_id: String,
    /// Items it overlaps, in time order.
    pub overlaps: Vec<String>,
    /// Label of a block-out with `permits_fixed` off that it sits in.
    pub blockout: Option<String>,
}

/// Conflicts among the open, timed `items`.
pub(crate) fn conflicts(ctx: &Ctx<'_>, items: &[Item]) -> Result<Vec<Conflict>> {
    let timed: Vec<(String, Instant, Instant)> = items
        .iter()
        .filter(|i| i.status == ItemStatus::Open)
        .filter_map(|i| Some((i.id.clone(), i.start_at?, i.end_at?)))
        .filter(|(_, s, e)| e > s)
        .collect();
    let Some(lo) = timed.iter().map(|t| t.1).min() else {
        return Ok(Vec::new());
    };
    let hi = timed.iter().map(|t| t.2).max().unwrap_or(lo);
    let strict: Vec<_> = windows::load(ctx.conn, ctx.settings.tz, lo, hi)?
        .into_iter()
        .filter(|w| !w.permits_fixed)
        .collect();
    let mut overlaps = overlapping(&timed);
    let sharing = store::tags::sharing(ctx.conn)?;
    let by_id: std::collections::HashMap<&str, &Item> =
        items.iter().map(|i| (i.id.as_str(), i)).collect();
    for (id, with) in &mut overlaps {
        if let Some(a) = by_id.get(id.as_str()) {
            with.retain(|o| {
                by_id
                    .get(o.as_str())
                    .is_none_or(|b| !may_overlap(a, b, &sharing))
            });
        }
    }
    let mut out = Vec::new();
    for item in items {
        let blockout = match (item.anchoring, item.start_at, item.end_at) {
            (Anchoring::Fixed, Some(s), Some(e)) if item.status == ItemStatus::Open => strict
                .iter()
                .find(|w| w.overlaps(s, e))
                .map(|w| w.label.clone()),
            _ => None,
        };
        let with = overlaps.remove(&item.id).unwrap_or_default();
        if !with.is_empty() || blockout.is_some() {
            out.push(Conflict {
                item_id: item.id.clone(),
                overlaps: with,
                blockout,
            });
        }
    }
    Ok(out)
}

/// Refuses to put a flexible, open item inside a block-out (roadmap §4.5
/// rule 1). Runs on every create, update and schedule, but only when the
/// item's time or anchoring changes, so an item caught inside a block-out
/// created later can still be edited, completed or moved out.
pub(crate) fn check_placement(ctx: &Ctx<'_>, before: Option<&Item>, item: &Item) -> Result<()> {
    if item.anchoring != Anchoring::Flexible
        || item.status != ItemStatus::Open
        || item.rrule.is_some()
    {
        return Ok(());
    }
    let (Some(start), Some(end)) = (item.start_at, item.end_at) else {
        return Ok(());
    };
    let unchanged = before.is_some_and(|b| {
        b.start_at == item.start_at && b.end_at == item.end_at && b.anchoring == item.anchoring
    });
    if unchanged {
        return Ok(());
    }
    // A zero-length item still occupies its start minute.
    let end = end.max(start + Duration::minutes(1));
    let blocking = windows::load(ctx.conn, ctx.settings.tz, start, end)?;
    let Some(w) = blocking.iter().find(|w| w.overlaps(start, end)) else {
        return Ok(());
    };
    let tz = ctx.settings.tz;
    let (ws, we) = (w.start.with_timezone(&tz), w.end.with_timezone(&tz));
    if let Some(calendar) = &w.calendar {
        return Err(CoreError::invalid(format!(
            "'{}' would clash with '{}' ({} {}–{}) from {calendar}, which blocks time like a \
             block-out. Flexible tasks can't go there: pick another time, or make it fixed.",
            item.title,
            w.label,
            ws.format("%a %-d %b"),
            ws.format("%H:%M"),
            we.format("%H:%M"),
        )));
    }
    Err(CoreError::invalid(format!(
        "'{}' would be inside the {} block-out ({} {}–{}). Flexible tasks can't go in \
         block-outs: pick another time, or make it fixed.",
        item.title,
        w.label,
        ws.format("%a %-d %b"),
        ws.format("%H:%M"),
        we.format("%H:%M"),
    )))
}

#[cfg(test)]
mod tests {
    use crate::model::Actor;
    use serde_json::{Value, json};

    fn run(p: &mut crate::Planner, op: &str, args: Value) -> Value {
        p.execute(op, args, &Actor::User).unwrap().result
    }

    fn fails(p: &mut crate::Planner, op: &str, args: Value) -> String {
        p.execute(op, args, &Actor::User).unwrap_err().to_string()
    }

    /// The test clock is Wed 14 Oct 2026 09:00 Melbourne.
    fn with_work() -> crate::Planner {
        let mut p = crate::planner::tests::planner();
        run(
            &mut p,
            "create_blockout",
            json!({ "label": "Work", "weekdays": ["mon","tue","wed","thu","fri"],
                    "start_time": "08:30", "end_time": "17:00" }),
        );
        p
    }

    fn starts(out: &Value) -> Vec<String> {
        out["slots"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s["start"].as_str().unwrap().to_owned())
            .collect()
    }

    #[test]
    fn free_slots_skip_blockouts_and_busy_time() {
        let mut p = with_work();
        run(
            &mut p,
            "create_item",
            json!({ "title": "Gym", "start": "2026-10-14T17:30", "duration_min": 60 }),
        );
        let out = run(
            &mut p,
            "find_free_slots",
            json!({ "duration_min": 45, "until": "2026-10-15T09:00", "limit": 5 }),
        );
        // Now (Wed 09:00) is inside Work and 17:00–17:30 is too short before Gym,
        // so: 18:30 Wednesday, then 07:00 Thursday.
        assert_eq!(
            starts(&out),
            ["2026-10-14T07:30:00Z", "2026-10-14T20:00:00Z"],
            "18:30 Wed and 07:00 Thu, Melbourne"
        );
        assert_eq!(out["timezone"], "Australia/Melbourne");
    }

    #[test]
    fn free_slots_validate_their_range() {
        let mut p = with_work();
        assert!(
            fails(
                &mut p,
                "find_free_slots",
                json!({ "duration_min": 30, "until": "2026-10-01T00:00" })
            )
            .contains("after")
        );
        assert!(
            fails(
                &mut p,
                "find_free_slots",
                json!({ "duration_min": 30, "until": "2027-10-01T00:00" })
            )
            .contains("62 days")
        );
        assert!(
            fails(&mut p, "find_free_slots", json!({ "duration_min": 2 })).contains("duration_min")
        );
    }

    #[test]
    fn flexible_tasks_never_land_in_blockouts() {
        let mut p = with_work();
        let err = fails(
            &mut p,
            "create_item",
            json!({ "title": "Meal prep", "start": "2026-10-14T16:30" }),
        );
        assert!(
            err.contains("Work block-out (Wed 14 Oct 08:30–17:00)"),
            "{err}"
        );

        // Fixed items may go inside.
        run(
            &mut p,
            "create_item",
            json!({ "title": "Dentist", "kind": "event", "start": "2026-10-14T15:00" }),
        );
        run(
            &mut p,
            "create_item",
            json!({ "title": "Telehealth", "anchoring": "fixed", "start": "2026-10-14T10:00" }),
        );

        // Moving a flexible task in by update or schedule is refused too.
        let task = run(
            &mut p,
            "create_item",
            json!({ "title": "Run", "start": "2026-10-14T07:00" }),
        );
        assert!(
            fails(
                &mut p,
                "update_item",
                json!({ "id": task["id"], "start": "2026-10-14T08:15" })
            )
            .contains("Work")
        );
        assert!(
            fails(
                &mut p,
                "schedule_item",
                json!({ "id": task["id"], "start": "2026-10-15T12:00" })
            )
            .contains("Thu 15 Oct")
        );
        // Ending exactly when Work starts is fine.
        run(
            &mut p,
            "update_item",
            json!({ "id": task["id"], "start": "2026-10-14T08:00" }),
        );
        // Day-only (all-day) tasks have no time, so they're fine on a workday.
        run(
            &mut p,
            "schedule_item",
            json!({ "id": task["id"], "date": "2026-10-15" }),
        );
    }

    #[test]
    fn items_caught_by_a_new_blockout_can_still_be_edited() {
        let mut p = crate::planner::tests::planner();
        let task = run(
            &mut p,
            "create_item",
            json!({ "title": "Errand", "start": "2026-10-14T12:00" }),
        );
        run(
            &mut p,
            "create_blockout",
            json!({ "label": "Work", "weekdays": ["wed"], "start_time": "08:30", "end_time": "17:00" }),
        );
        run(
            &mut p,
            "update_item",
            json!({ "id": task["id"], "title": "Errand (renamed)" }),
        );
        run(&mut p, "complete_item", json!({ "id": task["id"] }));
        run(&mut p, "uncomplete_item", json!({ "id": task["id"] }));
        // But moving it within the block-out is refused.
        assert!(
            fails(
                &mut p,
                "update_item",
                json!({ "id": task["id"], "start": "2026-10-14T13:00" })
            )
            .contains("Work")
        );
        // Making it flexible again while inside is a change of anchoring, so it's checked.
        run(
            &mut p,
            "update_item",
            json!({ "id": task["id"], "anchoring": "fixed" }),
        );
        assert!(
            fails(
                &mut p,
                "update_item",
                json!({ "id": task["id"], "anchoring": "flexible" })
            )
            .contains("Work")
        );
    }

    #[test]
    fn lifted_dates_allow_flexible_tasks() {
        let mut p = with_work();
        let work = run(&mut p, "list_blockouts", json!({}))[0]["id"].clone();
        run(
            &mut p,
            "add_blockout_exception",
            json!({ "blockout_id": work, "date": "2026-10-15" }),
        );
        run(
            &mut p,
            "create_item",
            json!({ "title": "Day off errand", "start": "2026-10-15T11:00" }),
        );
    }

    fn titles_and_starts(out: &Value) -> Vec<(String, String)> {
        out["placed"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| {
                (
                    p["item"]["title"].as_str().unwrap().to_owned(),
                    p["start"].as_str().unwrap().to_owned(),
                )
            })
            .collect()
    }

    #[test]
    fn auto_slot_orders_by_due_then_priority_then_length() {
        let mut p = with_work();
        for args in [
            json!({ "title": "Shopping", "duration_min": 90 }),
            json!({ "title": "Email", "duration_min": 10, "priority": 2 }),
            json!({ "title": "Tax", "duration_min": 120, "due": "2026-10-16T17:00" }),
        ] {
            run(&mut p, "create_item", args);
        }
        let out = p.execute("auto_slot", json!({}), &Actor::User).unwrap();
        let batch = out.batch_id.clone().unwrap();
        assert_eq!(out.changes.len(), 3);
        assert!(
            out.changes.iter().all(|c| c.batch_id == batch),
            "one undoable batch"
        );
        // Now is Wed 09:00, inside Work, so everything goes in after 17:00.
        assert_eq!(
            titles_and_starts(&out.result),
            [
                ("Tax".to_owned(), "2026-10-14T06:00:00Z".to_owned()), // 17:00
                ("Email".to_owned(), "2026-10-14T08:00:00Z".to_owned()), // 19:00
                ("Shopping".to_owned(), "2026-10-14T08:10:00Z".to_owned()), // 19:10
            ]
        );
        assert!(
            out.result["placed"][0]["previous_start"].is_null(),
            "came from the Inbox"
        );
        assert_eq!(
            run(&mut p, "list_inbox", json!({}))
                .as_array()
                .unwrap()
                .len(),
            0
        );
    }

    #[test]
    fn auto_slot_explains_what_it_cannot_place() {
        let mut p = with_work();
        let big = run(
            &mut p,
            "create_item",
            json!({ "title": "Sort receipts", "duration_min": 180, "due": "2026-10-14T18:00" }),
        );
        let event = run(
            &mut p,
            "create_item",
            json!({ "title": "Dentist", "kind": "event", "start": "2026-10-15T15:00" }),
        );
        let out = run(
            &mut p,
            "auto_slot",
            json!({ "ids": [big["id"], event["id"]] }),
        );
        assert!(out["placed"].as_array().unwrap().is_empty());
        let reasons: Vec<&str> = out["unplaced"]
            .as_array()
            .unwrap()
            .iter()
            .map(|u| u["reason"].as_str().unwrap())
            .collect();
        assert_eq!(
            reasons,
            [
                "Only tasks are auto-slotted.",
                "No free 3h slot before its due date (Wed 14 Oct 18:00). The longest free gap is \
                 1h, Wed 14 Oct 17:00–18:00. It stays in the Inbox.",
            ]
        );
        assert_eq!(out["unplaced"][1]["longest_gap"]["minutes"], 60);
        // Unplaced means untouched.
        assert!(run(&mut p, "get_item", json!({ "id": big["id"] }))["start_at"].is_null());
    }

    #[test]
    fn auto_slot_keeps_a_day_task_on_its_day() {
        let mut p = with_work();
        let task = run(
            &mut p,
            "create_item",
            json!({ "title": "Call bank", "date": "2026-10-15", "duration_min": 20 }),
        );
        let out = run(&mut p, "auto_slot", json!({ "ids": [task["id"]] }));
        // Thursday's first free time is 07:00, before Work.
        assert_eq!(
            titles_and_starts(&out),
            [("Call bank".to_owned(), "2026-10-14T20:00:00Z".to_owned())]
        );
        assert_eq!(out["placed"][0]["previous_date"], "2026-10-15");
    }

    #[test]
    fn agenda_reports_conflicts() {
        let mut p = crate::planner::tests::planner();
        run(
            &mut p,
            "create_blockout",
            json!({ "label": "Sleep", "weekdays": ["mon","tue","wed","thu","fri","sat","sun"],
                    "start_time": "23:00", "end_time": "06:30", "permits_fixed": false }),
        );
        let dentist = run(
            &mut p,
            "create_item",
            json!({ "title": "Dentist", "kind": "event", "start": "2026-10-15T15:00", "duration_min": 45 }),
        );
        let pt = run(
            &mut p,
            "create_item",
            json!({ "title": "Interview", "kind": "event", "start": "2026-10-15T15:30", "duration_min": 20 }),
        );
        let flight = run(
            &mut p,
            "create_item",
            json!({ "title": "Flight", "kind": "event", "start": "2026-10-15T23:30" }),
        );
        let agenda = run(&mut p, "list_agenda", json!({ "start": "2026-10-15" }));
        let conflicts = agenda["conflicts"].as_array().unwrap();
        let find = |id: &Value| {
            conflicts
                .iter()
                .find(|c| c["item_id"] == *id)
                .unwrap()
                .clone()
        };
        assert_eq!(find(&dentist["id"])["overlaps"], json!([pt["id"]]));
        assert_eq!(find(&pt["id"])["overlaps"], json!([dentist["id"]]));
        assert_eq!(find(&flight["id"])["blockout"], "Sleep");
        assert_eq!(conflicts.len(), 3);

        let summary = run(&mut p, "get_day_summary", json!({ "date": "2026-10-15" }));
        assert_eq!(summary["conflicts"].as_array().unwrap().len(), 3);
    }

    /// Cleaning 17:00–19:00 and the dentist 19:00–20:00 on Wednesday, plus
    /// laundry (its tag shares time) and email waiting in the Inbox.
    fn with_laundry() -> (crate::Planner, Value, Value, Value) {
        let mut p = with_work();
        run(
            &mut p,
            "set_tag",
            json!({ "name": "laundry", "shares_time": true }),
        );
        let cleaning = run(
            &mut p,
            "create_item",
            json!({ "title": "Cleaning", "start": "2026-10-14T17:00", "duration_min": 120 }),
        );
        let dentist = run(
            &mut p,
            "create_item",
            json!({ "title": "Dentist", "kind": "event", "start": "2026-10-14T19:00", "duration_min": 60 }),
        );
        let laundry = run(
            &mut p,
            "create_item",
            json!({ "title": "Laundry", "duration_min": 30, "tags": ["Laundry"] }),
        );
        (p, cleaning, dentist, laundry)
    }

    #[test]
    fn tasks_that_share_time_go_over_other_tasks_but_not_events() {
        let (mut p, _, _, _) = with_laundry();
        run(
            &mut p,
            "create_item",
            json!({ "title": "Email", "duration_min": 30 }),
        );
        let out = run(&mut p, "auto_slot", json!({}));
        let placed = titles_and_starts(&out);
        // Laundry goes in during cleaning; email waits until after the dentist,
        // and laundry doesn't keep it out of anywhere.
        assert!(placed.contains(&("Laundry".to_owned(), "2026-10-14T06:00:00Z".to_owned())));
        assert!(placed.contains(&("Email".to_owned(), "2026-10-14T09:00:00Z".to_owned())));

        // Asked for directly, the dentist still blocks: 17:00 then 20:00.
        let free = run(
            &mut p,
            "find_free_slots",
            json!({ "duration_min": 90, "until": "2026-10-14T23:00", "shares_time": true }),
        );
        assert_eq!(starts(&free)[0], "2026-10-14T06:00:00Z");
        let free = run(
            &mut p,
            "find_free_slots",
            json!({ "duration_min": 150, "until": "2026-10-14T23:00", "shares_time": true }),
        );
        assert_ne!(
            starts(&free).first().map(String::as_str),
            Some("2026-10-14T06:00:00Z")
        );
    }

    #[test]
    fn tasks_that_share_time_are_not_conflicts_with_tasks() {
        let (mut p, cleaning, dentist, laundry) = with_laundry();
        run(
            &mut p,
            "schedule_item",
            json!({ "id": laundry["id"], "start": "2026-10-14T18:45" }),
        );
        let agenda = run(&mut p, "list_agenda", json!({ "start": "2026-10-14" }));
        let conflicts = agenda["conflicts"].as_array().unwrap();
        // Over cleaning is fine; over the dentist (19:00–19:15) isn't.
        let laundry_c = conflicts
            .iter()
            .find(|c| c["item_id"] == laundry["id"])
            .unwrap();
        assert_eq!(laundry_c["overlaps"], json!([dentist["id"]]));
        assert!(conflicts.iter().all(|c| c["item_id"] != cleaning["id"]));
    }
}

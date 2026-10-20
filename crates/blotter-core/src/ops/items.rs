//! Item mutations: create, update, delete, complete, schedule.

use chrono::Duration;
use chrono_tz::Tz;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::input::{DateInput, DateTimeInput, TimeInput, patch};
use super::occurrences::{self, Scope, Target};
use super::{Ctx, OpDef, require_item};
use crate::error::{CoreError, Result};
use crate::model::{Actor, Anchoring, Item, ItemKind, ItemStatus, Reminder, Source};
use crate::recurrence::{self, Slot};
use crate::store;
use crate::time::Instant;

pub(super) fn defs() -> Vec<OpDef> {
    vec![
        OpDef::new(
            "create_item",
            "Create a task, event or reminder. Give `start` for a timed item, `date` for an \
             all-day item, or neither to put a task in the Inbox. Events default to fixed \
             anchoring, tasks to flexible.",
            true,
            create_item,
        ),
        OpDef::new(
            "update_item",
            "Change fields of an item. Omitted fields are left as they are; `null` clears an \
             optional field. Giving `start` or `date` reschedules it.",
            true,
            update_item,
        ),
        OpDef::new(
            "delete_item",
            "Delete an item, its reminders and any recurrence overrides. For one occurrence of a \n             recurring item (id `<series>@<start>`), `scope` picks this occurrence (default), \n             this and following, or all.",
            true,
            delete_item,
        ),
        OpDef::new(
            "complete_item",
            "Mark an item done, recording the completion time.",
            true,
            complete_item,
        ),
        OpDef::new(
            "uncomplete_item",
            "Mark a completed item open again.",
            true,
            uncomplete_item,
        ),
        OpDef::new(
            "schedule_item",
            "Place an item at a time (`start`) or on a day as all-day (`date`). Duration comes \
             from `duration_min`, the item's own duration, or the default.",
            true,
            schedule_item,
        ),
        OpDef::new(
            "unschedule_item",
            "Remove a task's time and date, returning it to the Inbox as a flexible task.",
            true,
            unschedule_item,
        ),
    ]
}

/// Entity name used in the change log.
pub(crate) const ENTITY: &str = "item";

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateItemArgs {
    #[schemars(length(min = 1, max = 500))]
    pub title: String,
    /// Defaults to `task`.
    pub kind: Option<ItemKind>,
    /// Defaults to `fixed` for events and `flexible` otherwise.
    pub anchoring: Option<Anchoring>,
    pub notes: Option<String>,
    /// Start of a timed item.
    pub start: Option<DateTimeInput>,
    /// End of a timed item. If omitted, start + duration.
    pub end: Option<DateTimeInput>,
    #[schemars(range(min = 1, max = 10080))]
    pub duration_min: Option<u32>,
    /// Day of an all-day item.
    pub date: Option<DateInput>,
    /// Last day (inclusive) of a multi-day all-day item.
    pub end_date: Option<DateInput>,
    /// Optional deadline.
    pub due: Option<DateTimeInput>,
    /// Do-window start: the slot engine won't place it before this date (the window ends
    /// at `due`). Missed slots are re-planned inside the window.
    pub not_before: Option<DateInput>,
    /// The usual time of day to place it (`HH:MM`, with `window_end`); tried first.
    pub window_start: Option<TimeInput>,
    pub window_end: Option<TimeInput>,
    /// 0 (none) to 3 (highest).
    #[schemars(range(max = 3))]
    pub priority: Option<u8>,
    /// Lowercase tag names without `#`.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Hex colour like `#6c9cf2`.
    pub color: Option<String>,
    /// RFC 5545 recurrence rule, e.g. `FREQ=WEEKLY;BYDAY=MO`.
    pub rrule: Option<String>,
    /// IANA timezone for a fixed item; defaults to the user's.
    pub tz: Option<String>,
    /// Reminders, as minutes before the start. Omitted: events get the default
    /// reminder (10 minutes unless changed in settings); `[]` means none.
    pub reminders: Option<Vec<u32>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct UpdateItemArgs {
    pub id: String,
    #[schemars(length(min = 1, max = 500))]
    pub title: Option<String>,
    pub kind: Option<ItemKind>,
    pub anchoring: Option<Anchoring>,
    pub status: Option<ItemStatus>,
    pub notes: Option<String>,
    /// Reschedule to this start (timed).
    pub start: Option<DateTimeInput>,
    /// New end for a timed item.
    pub end: Option<DateTimeInput>,
    /// Reschedule as all-day on this date.
    pub date: Option<DateInput>,
    #[serde(default, deserialize_with = "patch")]
    pub end_date: Option<Option<DateInput>>,
    #[serde(default, deserialize_with = "patch")]
    #[schemars(range(min = 1, max = 10080))]
    pub duration_min: Option<Option<u32>>,
    #[serde(default, deserialize_with = "patch")]
    pub due: Option<Option<DateTimeInput>>,
    #[serde(default, deserialize_with = "patch")]
    pub not_before: Option<Option<DateInput>>,
    #[serde(default, deserialize_with = "patch")]
    pub window_start: Option<Option<TimeInput>>,
    #[serde(default, deserialize_with = "patch")]
    pub window_end: Option<Option<TimeInput>>,
    #[schemars(range(max = 3))]
    pub priority: Option<u8>,
    pub tags: Option<Vec<String>>,
    #[serde(default, deserialize_with = "patch")]
    pub color: Option<Option<String>>,
    #[serde(default, deserialize_with = "patch")]
    pub rrule: Option<Option<String>>,
    #[serde(default, deserialize_with = "patch")]
    pub tz: Option<Option<String>>,
    /// Position within a Diary day (lower sorts first); used when reordering.
    pub sort_order: Option<f64>,
    /// For a recurring item: change this occurrence (default), this and the
    /// following ones, or the whole series.
    pub scope: Option<Scope>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct IdArgs {
    pub id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DeleteItemArgs {
    pub id: String,
    /// For a recurring item: delete this occurrence (default), this and the
    /// following ones, or the whole series.
    pub scope: Option<Scope>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ScheduleItemArgs {
    pub id: String,
    /// Timed start. Give this or `date`.
    pub start: Option<DateTimeInput>,
    pub end: Option<DateTimeInput>,
    #[schemars(range(min = 1, max = 10080))]
    pub duration_min: Option<u32>,
    /// All-day date. Give this or `start`.
    pub date: Option<DateInput>,
}

#[derive(Debug, Serialize)]
pub struct Deleted {
    pub deleted: String,
}

impl Ctx<'_> {
    fn source(&self) -> Source {
        match self.actor {
            Actor::User | Actor::System => Source::User,
            Actor::Ai { .. } => Source::Ai,
            Actor::Mcp => Source::Mcp,
        }
    }
}

fn create_item(ctx: &mut Ctx<'_>, a: CreateItemArgs) -> Result<Item> {
    let kind = a.kind.unwrap_or(ItemKind::Task);
    let mut item = Item::new(ctx.new_id(), kind, a.title, ctx.now);
    item.anchoring = a.anchoring.unwrap_or(kind.default_anchoring());
    item.source = ctx.source();
    item.notes = a.notes.unwrap_or_default();
    item.duration_min = a.duration_min;
    item.due_at = a.due.map(|d| d.resolve(ctx.settings.tz));
    item.not_before = a.not_before.map(|d| d.0);
    item.window_start = a.window_start.map(|t| t.0);
    item.window_end = a.window_end.map(|t| t.0);
    item.priority = a.priority.unwrap_or(0);
    item.tags = a.tags;
    item.color = a.color;
    item.rrule = a.rrule;
    item.tz = a.tz;
    item.sort_order = ctx.now.timestamp() as f64;

    match (a.start, a.date) {
        (Some(_), Some(_)) => {
            return Err(CoreError::invalid(
                "give either `start` or `date`, not both",
            ));
        }
        (Some(start), None) => {
            let tz = ctx.settings.tz;
            set_timed(
                ctx,
                &mut item,
                start.resolve(tz),
                a.end.map(|e| e.resolve(tz)),
                a.duration_min,
            )?;
        }
        (None, Some(date)) => set_all_day(&mut item, date, a.end_date),
        (None, None) => {
            if a.end.is_some() || a.end_date.is_some() {
                return Err(CoreError::invalid("`end`/`end_date` need a `start`/`date`"));
            }
        }
    }

    finalize(ctx, &mut item)?;
    super::slots::check_placement(ctx, None, &item)?;
    store::items::insert(ctx.conn, &item)?;
    ctx.record(ENTITY, &item.id, None, Some(&item))?;
    let offsets =
        a.reminders.unwrap_or_else(
            || match (item.kind, ctx.settings.default_event_reminder_min) {
                (ItemKind::Event, Some(m)) if item.is_scheduled() => vec![m],
                _ => Vec::new(),
            },
        );
    if !offsets.is_empty() && !item.is_scheduled() {
        return Err(CoreError::invalid(
            "reminders before the start need the item to have a `start` or `date`",
        ));
    }
    for offset in offsets {
        let r = Reminder {
            id: ctx.new_id(),
            item_id: item.id.clone(),
            offset_min: Some(offset),
            at: None,
            fired_at: None,
            snoozed_until: None,
            created_at: ctx.now,
        };
        store::reminders::upsert(ctx.conn, &r)?;
        ctx.record(super::reminders::ENTITY, &r.id, None, Some(&r))?;
    }
    Ok(item)
}

fn update_item(ctx: &mut Ctx<'_>, mut a: UpdateItemArgs) -> Result<Item> {
    let scope = a.scope.take().unwrap_or_default();
    let Target::Occurrence {
        series,
        slot,
        existing,
    } = occurrences::resolve(ctx, &a.id)?
    else {
        return update_stored(ctx, a);
    };
    match scope {
        Scope::This => {
            if a.rrule.is_some() {
                return Err(CoreError::invalid(
                    "change the repeat rule for the whole series (scope `all`) or from this \n                     occurrence on (scope `following`)",
                ));
            }
            a.id = occurrences::override_for(ctx, &series, slot, existing)?.id;
        }
        Scope::All => {
            shift_to_series(ctx, &mut a, &series, slot);
            a.id = series.id.clone();
        }
        Scope::Following => a.id = occurrences::split(ctx, &series, slot)?.id,
    }
    update_stored(ctx, a)
}

/// Rewrites times given for one occurrence as the same change to the
/// series' first occurrence, so "move this to 10:00, all occurrences"
/// moves every occurrence by the same amount.
fn shift_to_series(ctx: &Ctx<'_>, a: &mut UpdateItemArgs, series: &Item, slot: Slot) {
    let tz = ctx.settings.tz;
    let (occ_start, occ_date) = match slot {
        Slot::Timed { start, .. } => (Some(start), start.with_timezone(&tz).date_naive()),
        Slot::AllDay { date, .. } => (None, date),
    };
    let first_date = series
        .start_date
        .or_else(|| series.start_at.map(|s| s.with_timezone(&tz).date_naive()));
    let shift_date = |d: DateInput| match first_date {
        Some(first) => DateInput(first + (d.0 - occ_date)),
        None => d,
    };
    let shift_time = |t: DateTimeInput| match (occ_start, series.start_at) {
        (Some(occ), Some(first)) => DateTimeInput::Absolute(first + (t.resolve(tz) - occ)),
        _ => {
            // All-day occurrence turned timed: keep the time, move the date.
            let local = t.resolve(tz).with_timezone(&tz);
            DateTimeInput::Local(
                shift_date(DateInput(local.date_naive()))
                    .0
                    .and_time(local.time()),
            )
        }
    };
    a.start = a.start.map(shift_time);
    a.end = a.end.map(shift_time);
    a.date = a.date.map(shift_date);
    a.end_date = a.end_date.map(|d| d.map(shift_date));
}

/// Applies `a` to one stored item.
fn update_stored(ctx: &mut Ctx<'_>, a: UpdateItemArgs) -> Result<Item> {
    let before = require_item(ctx, &a.id)?;
    let mut item = before.clone();
    let tz = ctx.settings.tz;

    if let Some(v) = a.title {
        item.title = v;
    }
    if let Some(v) = a.kind {
        item.kind = v;
    }
    if let Some(v) = a.anchoring {
        item.anchoring = v;
    }
    if let Some(v) = a.notes {
        item.notes = v;
    }
    if let Some(v) = a.priority {
        item.priority = v;
    }
    if let Some(v) = a.tags {
        item.tags = v;
    }
    if let Some(v) = a.color {
        item.color = v;
    }
    if let Some(v) = a.rrule {
        item.rrule = v;
    }
    if let Some(v) = a.sort_order {
        if !v.is_finite() {
            return Err(CoreError::invalid("sort_order must be a finite number"));
        }
        item.sort_order = v;
    }
    if let Some(v) = a.tz {
        item.tz = v;
    }
    if let Some(v) = a.due {
        item.due_at = v.map(|d| d.resolve(tz));
    }
    if let Some(v) = a.not_before {
        item.not_before = v.map(|d| d.0);
    }
    if let Some(v) = a.window_start {
        item.window_start = v.map(|t| t.0);
    }
    if let Some(v) = a.window_end {
        item.window_end = v.map(|t| t.0);
    }
    if let Some(status) = a.status {
        set_status(&mut item, status, ctx.now);
    }
    let duration_changed = a.duration_min.is_some();
    if let Some(v) = a.duration_min {
        item.duration_min = v;
    }

    match (a.start, a.date) {
        (Some(_), Some(_)) => {
            return Err(CoreError::invalid(
                "give either `start` or `date`, not both",
            ));
        }
        (Some(start), None) => {
            let duration = item.duration_min;
            set_timed(
                ctx,
                &mut item,
                start.resolve(tz),
                a.end.map(|e| e.resolve(tz)),
                duration,
            )?;
        }
        (None, Some(date)) => {
            let end_date = a.end_date.flatten();
            set_all_day(&mut item, date, end_date);
        }
        (None, None) => {
            if let Some(start) = item.start_at {
                if let Some(end) = a.end {
                    let end = end.resolve(tz);
                    item.end_at = Some(end);
                    item.duration_min = minutes_between(start, end);
                } else if duration_changed {
                    let duration = item.duration_min;
                    set_timed(ctx, &mut item, start, None, duration)?;
                }
            } else if a.end.is_some() {
                return Err(CoreError::invalid("`end` needs the item to have a `start`"));
            }
            if let Some(end_date) = a.end_date {
                if !item.all_day {
                    return Err(CoreError::invalid(
                        "`end_date` only applies to all-day items",
                    ));
                }
                item.end_date = end_date.map(|d| d.0);
            }
        }
    }

    item.updated_at = ctx.now;
    finalize(ctx, &mut item)?;
    super::slots::check_placement(ctx, Some(&before), &item)?;
    if item == before {
        return Ok(item);
    }
    store::items::update(ctx.conn, &item)?;
    ctx.record(ENTITY, &item.id, Some(&before), Some(&item))?;
    Ok(item)
}

fn delete_item(ctx: &mut Ctx<'_>, a: DeleteItemArgs) -> Result<Deleted> {
    let Target::Occurrence {
        series,
        slot,
        existing,
    } = occurrences::resolve(ctx, &a.id)?
    else {
        let item = require_item(ctx, &a.id)?;
        return delete_stored(ctx, item);
    };
    match a.scope.unwrap_or_default() {
        // A deleted occurrence is a cancelled override (an exception).
        Scope::This => {
            let before = occurrences::override_for(ctx, &series, slot, existing)?;
            let mut item = before.clone();
            set_status(&mut item, ItemStatus::Cancelled, ctx.now);
            item.updated_at = ctx.now;
            store::items::update(ctx.conn, &item)?;
            ctx.record(ENTITY, &item.id, Some(&before), Some(&item))?;
            Ok(Deleted { deleted: a.id })
        }
        Scope::Following => {
            if !occurrences::end_before(ctx, &series, slot)? {
                return delete_stored(ctx, *series);
            }
            for o in occurrences::overrides_from(ctx, &series, slot)? {
                delete_stored(ctx, o)?;
            }
            Ok(Deleted { deleted: a.id })
        }
        Scope::All => delete_stored(ctx, *series),
    }
}

fn delete_stored(ctx: &mut Ctx<'_>, item: Item) -> Result<Deleted> {
    // Record everything the cascade will remove, so undo can restore it.
    for child in store::items::list_children(ctx.conn, &item.id)? {
        record_reminder_deletions(ctx, &child.id)?;
        ctx.record(ENTITY, &child.id, Some(&child), None)?;
    }
    record_reminder_deletions(ctx, &item.id)?;
    ctx.record(ENTITY, &item.id, Some(&item), None)?;
    store::items::delete(ctx.conn, &item.id)?;
    Ok(Deleted { deleted: item.id })
}

fn record_reminder_deletions(ctx: &mut Ctx<'_>, item_id: &str) -> Result<()> {
    for r in store::reminders::list_for_item(ctx.conn, item_id)? {
        ctx.record(super::reminders::ENTITY, &r.id, Some(&r), None)?;
    }
    Ok(())
}

fn complete_item(ctx: &mut Ctx<'_>, a: IdArgs) -> Result<Item> {
    change_status(ctx, &a.id, ItemStatus::Done)
}

fn uncomplete_item(ctx: &mut Ctx<'_>, a: IdArgs) -> Result<Item> {
    change_status(ctx, &a.id, ItemStatus::Open)
}

fn change_status(ctx: &mut Ctx<'_>, id: &str, status: ItemStatus) -> Result<Item> {
    // Completing one occurrence stores it as an override; the series stays open.
    let before = match occurrences::resolve(ctx, id)? {
        Target::Item(item) if item.rrule.is_some() => {
            return Err(CoreError::invalid(format!(
                "'{}' repeats: complete one occurrence (its id is the series id, \n                 '@' and its start) instead of the whole series",
                item.title
            )));
        }
        Target::Item(item) => item,
        Target::Occurrence {
            series,
            slot,
            existing,
        } => occurrences::override_for(ctx, &series, slot, existing)?,
    };
    if before.status == status {
        return Ok(before);
    }
    let mut item = before.clone();
    set_status(&mut item, status, ctx.now);
    item.updated_at = ctx.now;
    store::items::update(ctx.conn, &item)?;
    ctx.record(ENTITY, &item.id, Some(&before), Some(&item))?;
    Ok(item)
}

fn set_status(item: &mut Item, status: ItemStatus, now: Instant) {
    if item.status == status {
        return;
    }
    item.status = status;
    item.completed_at = (status == ItemStatus::Done).then_some(now);
}

fn schedule_item(ctx: &mut Ctx<'_>, a: ScheduleItemArgs) -> Result<Item> {
    // Rescheduling one occurrence moves just that one (as an override).
    let before = match occurrences::resolve(ctx, &a.id)? {
        Target::Item(item) => item,
        Target::Occurrence {
            series,
            slot,
            existing,
        } => occurrences::override_for(ctx, &series, slot, existing)?,
    };
    let mut item = before.clone();
    let tz = ctx.settings.tz;
    match (a.start, a.date) {
        (Some(start), None) => {
            let duration = a.duration_min.or(item.duration_min);
            set_timed(
                ctx,
                &mut item,
                start.resolve(tz),
                a.end.map(|e| e.resolve(tz)),
                duration,
            )?;
        }
        (None, Some(date)) => set_all_day(&mut item, date, None),
        _ => return Err(CoreError::invalid("give exactly one of `start` or `date`")),
    }
    item.updated_at = ctx.now;
    finalize(ctx, &mut item)?;
    super::slots::check_placement(ctx, Some(&before), &item)?;
    store::items::update(ctx.conn, &item)?;
    ctx.record(ENTITY, &item.id, Some(&before), Some(&item))?;
    Ok(item)
}

fn unschedule_item(ctx: &mut Ctx<'_>, a: IdArgs) -> Result<Item> {
    let before = require_item(
        ctx,
        recurrence::split_occurrence_id(&a.id).map_or(a.id.as_str(), |(s, _)| s),
    )?;
    if before.rrule.is_some() || before.parent_id.is_some() {
        return Err(CoreError::invalid(
            "a recurring item can't go to the Inbox; delete the occurrence instead",
        ));
    }
    if before.kind != ItemKind::Task {
        return Err(CoreError::invalid(format!(
            "only tasks can go to the Inbox; this item is a {}",
            before.kind
        )));
    }
    if before.is_inbox() {
        return Ok(before);
    }
    let mut item = before.clone();
    clear_schedule(&mut item);
    item.anchoring = Anchoring::Flexible;
    item.updated_at = ctx.now;
    finalize(ctx, &mut item)?;
    store::items::update(ctx.conn, &item)?;
    ctx.record(ENTITY, &item.id, Some(&before), Some(&item))?;
    Ok(item)
}

/// Makes `item` a timed item from `start`. The end is `end` if given,
/// else start + duration (the item's, or the kind's default).
fn set_timed(
    ctx: &Ctx<'_>,
    item: &mut Item,
    start: Instant,
    end: Option<Instant>,
    duration_min: Option<u32>,
) -> Result<()> {
    let end = match end {
        Some(end) => {
            if end < start {
                return Err(CoreError::invalid("`end` is before `start`"));
            }
            item.duration_min = minutes_between(start, end).or(item.duration_min);
            end
        }
        None => {
            let minutes = duration_min.unwrap_or(match item.kind {
                ItemKind::Event => ctx.settings.default_event_duration_min,
                ItemKind::Task => ctx.settings.default_task_duration_min,
                ItemKind::Reminder => 0,
            });
            item.duration_min = duration_min.or(item.duration_min);
            start + Duration::minutes(i64::from(minutes))
        }
    };
    item.all_day = false;
    item.start_date = None;
    item.end_date = None;
    item.start_at = Some(start);
    item.end_at = Some(end);
    Ok(())
}

fn set_all_day(item: &mut Item, date: DateInput, end_date: Option<DateInput>) {
    item.all_day = true;
    item.start_at = None;
    item.end_at = None;
    item.start_date = Some(date.0);
    item.end_date = end_date.map(|d| d.0).filter(|d| *d != date.0);
}

fn clear_schedule(item: &mut Item) {
    item.all_day = false;
    item.start_at = None;
    item.end_at = None;
    item.start_date = None;
    item.end_date = None;
}

fn minutes_between(start: Instant, end: Instant) -> Option<u32> {
    u32::try_from((end - start).num_minutes())
        .ok()
        .filter(|m| *m > 0)
}

/// Normalizes and validates an item before it is written. Enforces the
/// invariants the schema's CHECK constraints guard, with readable errors.
pub(crate) fn finalize(ctx: &Ctx<'_>, item: &mut Item) -> Result<()> {
    item.title = item.title.trim().to_owned();
    if item.title.is_empty() {
        return Err(CoreError::invalid("title must not be empty"));
    }
    if item.title.chars().count() > 500 {
        return Err(CoreError::invalid("title is longer than 500 characters"));
    }

    let mut tags: Vec<String> = Vec::with_capacity(item.tags.len());
    for raw in &item.tags {
        let tag = raw.trim().trim_start_matches('#').to_lowercase();
        if tag.is_empty() || tag.len() > 32 || tag.chars().any(char::is_whitespace) {
            return Err(CoreError::invalid(format!(
                "tag '{raw}' must be 1–32 characters with no spaces"
            )));
        }
        if !tags.contains(&tag) {
            tags.push(tag);
        }
    }
    item.tags = tags;

    if let Some(color) = &item.color {
        let hex = color.strip_prefix('#').unwrap_or("");
        if hex.len() != 6 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(CoreError::invalid(format!(
                "color '{color}' must look like #6c9cf2"
            )));
        }
    }

    if item.priority > 3 {
        return Err(CoreError::invalid("priority must be 0–3"));
    }
    match (item.window_start, item.window_end) {
        (Some(s), Some(e)) if e <= s => {
            return Err(CoreError::invalid(
                "window_end must be after window_start (windows don't cross midnight)",
            ));
        }
        (Some(_), None) | (None, Some(_)) => {
            return Err(CoreError::invalid(
                "give both window_start and window_end, or neither",
            ));
        }
        _ => {}
    }
    if let (Some(from), Some(due)) = (item.not_before, item.due_at)
        && due.with_timezone(&ctx.settings.tz).date_naive() < from
    {
        return Err(CoreError::invalid("the due date is before not_before"));
    }
    if item.duration_min == Some(0) {
        item.duration_min = None;
    }
    if let Some(rrule) = &item.rrule {
        if item.parent_id.is_some() {
            return Err(CoreError::invalid(
                "an occurrence can't have its own repeat rule; change the series",
            ));
        }
        item.rrule = Some(recurrence::normalize_rule(rrule)?);
    }

    if matches!(item.kind, ItemKind::Event | ItemKind::Reminder) && !item.is_scheduled() {
        return Err(CoreError::invalid(format!(
            "a {} needs a `start` or `date`",
            item.kind
        )));
    }
    if item.rrule.is_some() && !item.is_scheduled() {
        return Err(CoreError::invalid(
            "a recurring item needs a `start` or `date`",
        ));
    }

    if item.anchoring == Anchoring::Fixed && item.is_scheduled() && item.tz.is_none() {
        item.tz = Some(ctx.settings.tz.name().to_owned());
    }
    if let Some(tz) = &item.tz {
        tz.parse::<Tz>()
            .map_err(|_| CoreError::invalid(format!("unknown timezone '{tz}'")))?;
    }

    item.completed_at = match item.status {
        ItemStatus::Done => item.completed_at.or(Some(ctx.now)),
        _ => None,
    };
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::Planner;
    use crate::model::{Actor, Anchoring, Item, ItemKind, ItemStatus, Source};
    use crate::store;
    use chrono::{TimeZone, Utc};
    use serde_json::{Value, json};

    fn planner() -> Planner {
        crate::planner::tests::planner()
    }

    fn run(p: &mut Planner, op: &str, args: Value) -> Value {
        p.execute(op, args, &Actor::User).unwrap().result
    }

    fn item(v: Value) -> Item {
        serde_json::from_value(v).unwrap()
    }

    #[test]
    fn create_inbox_task_with_defaults() {
        let mut p = planner();
        let out = p
            .execute(
                "create_item",
                json!({ "title": "  Call plumber " }),
                &Actor::User,
            )
            .unwrap();
        let i = item(out.result);
        assert_eq!(i.title, "Call plumber");
        assert!(i.is_inbox());
        assert_eq!(i.source, Source::User);
        assert_eq!(out.changes.len(), 1);
        assert_eq!(out.changes[0].op, "create_item");
        assert!(out.changes[0].before.is_none());
        assert_eq!(
            out.batch_id.as_deref(),
            Some(out.changes[0].batch_id.as_str())
        );
    }

    #[test]
    fn create_fixed_event_from_local_time_with_duration() {
        let mut p = planner();
        let i = item(run(
            &mut p,
            "create_item",
            json!({ "title": "Dentist", "kind": "event", "start": "2026-10-20T15:00",
                    "duration_min": 45, "tags": ["#Health"] }),
        ));
        assert_eq!(i.anchoring, Anchoring::Fixed);
        // Melbourne is UTC+11 in October.
        assert_eq!(
            i.start_at,
            Some(Utc.with_ymd_and_hms(2026, 10, 20, 4, 0, 0).unwrap())
        );
        assert_eq!(
            i.end_at,
            Some(Utc.with_ymd_and_hms(2026, 10, 20, 4, 45, 0).unwrap())
        );
        assert_eq!(i.tz.as_deref(), Some("Australia/Melbourne"));
        assert_eq!(i.tags, ["health"]);
    }

    #[test]
    fn event_without_duration_uses_event_default_and_task_uses_task_default() {
        let mut p = planner();
        let e = item(run(
            &mut p,
            "create_item",
            json!({ "title": "Party", "kind": "event", "start": "2026-10-20T19:00" }),
        ));
        assert_eq!((e.end_at.unwrap() - e.start_at.unwrap()).num_minutes(), 60);
        let t = item(run(
            &mut p,
            "create_item",
            json!({ "title": "Tidy", "start": "2026-10-20T19:00" }),
        ));
        assert_eq!((t.end_at.unwrap() - t.start_at.unwrap()).num_minutes(), 30);
    }

    #[test]
    fn create_all_day_multi_day() {
        let mut p = planner();
        let i = item(run(
            &mut p,
            "create_item",
            json!({ "title": "Leave", "kind": "event", "date": "2026-10-20", "end_date": "2026-10-23" }),
        ));
        assert!(i.all_day);
        assert_eq!(i.start_date.unwrap().to_string(), "2026-10-20");
        assert_eq!(i.end_date.unwrap().to_string(), "2026-10-23");
        assert!(i.start_at.is_none());
    }

    #[test]
    fn create_rejects_bad_input_with_readable_errors() {
        let mut p = planner();
        for (args, needle) in [
            (json!({ "title": "" }), "title"),
            (json!({ "title": "x", "priority": 7 }), "priority"),
            (json!({ "title": "x", "start": "tuesday" }), "start"),
            (json!({ "title": "x", "colour": "#fff" }), "colour"),
            (json!({ "title": "x", "kind": "event" }), "needs a `start`"),
            (
                json!({ "title": "x", "start": "2026-10-20T10:00", "date": "2026-10-20" }),
                "either",
            ),
            (
                json!({ "title": "x", "start": "2026-10-20T10:00", "end": "2026-10-20T09:00" }),
                "before",
            ),
            (json!({ "title": "x", "color": "blue" }), "color"),
            (json!({ "title": "x", "tags": ["two words"] }), "tag"),
            (
                json!({ "title": "x", "start": "2026-10-20T10:00", "tz": "Mars/Base" }),
                "timezone",
            ),
        ] {
            let err = p
                .execute("create_item", args.clone(), &Actor::User)
                .unwrap_err()
                .to_string();
            assert!(err.contains(needle), "{args} → {err}");
        }
        let n: i64 = p
            .db()
            .conn()
            .query_row("SELECT COUNT(*) FROM changes", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 0, "failed calls leave no change rows");
    }

    #[test]
    fn update_sets_sort_order_and_rejects_non_finite() {
        let mut p = planner();
        let i = item(run(
            &mut p,
            "create_item",
            json!({ "title": "Errand", "date": "2026-10-20" }),
        ));
        let u = item(run(
            &mut p,
            "update_item",
            json!({ "id": i.id, "sort_order": 2.5 }),
        ));
        assert_eq!(u.sort_order, 2.5);
        // JSON can't carry NaN, so only an absurd value can be tried; it must be finite.
        let err = p
            .execute(
                "update_item",
                json!({ "id": i.id, "sort_order": "high" }),
                &Actor::User,
            )
            .unwrap_err();
        assert!(err.to_string().contains("sort_order"));
    }

    #[test]
    fn update_patches_and_clears_fields() {
        let mut p = planner();
        let i = item(run(
            &mut p,
            "create_item",
            json!({ "title": "Insurance", "due": "2026-10-30T17:00", "color": "#6c9cf2", "priority": 1 }),
        ));
        let u = item(run(
            &mut p,
            "update_item",
            json!({ "id": i.id, "title": "Review insurance", "color": null, "priority": 2 }),
        ));
        assert_eq!(u.title, "Review insurance");
        assert_eq!(u.color, None);
        assert_eq!(u.priority, 2);
        assert_eq!(u.due_at, i.due_at, "untouched field kept");

        let log = store::changes::list_for_entity(p.db().conn(), "item", &i.id).unwrap();
        assert_eq!(log.len(), 2);
        assert_eq!(log[1].before.as_ref().unwrap()["title"], "Insurance");
        assert_eq!(log[1].after.as_ref().unwrap()["title"], "Review insurance");
    }

    #[test]
    fn update_with_no_effective_change_writes_nothing() {
        let mut p = planner();
        let i = item(run(&mut p, "create_item", json!({ "title": "Same" })));
        let out = p
            .execute(
                "update_item",
                json!({ "id": i.id, "title": "Same" }),
                &Actor::User,
            )
            .unwrap();
        assert!(out.batch_id.is_none());
        assert!(out.changes.is_empty());
    }

    #[test]
    fn update_duration_moves_end_and_end_updates_duration() {
        let mut p = planner();
        let i = item(run(
            &mut p,
            "create_item",
            json!({ "title": "Gym", "start": "2026-10-20T17:30", "duration_min": 45 }),
        ));
        let u = item(run(
            &mut p,
            "update_item",
            json!({ "id": i.id, "duration_min": 60 }),
        ));
        assert_eq!((u.end_at.unwrap() - u.start_at.unwrap()).num_minutes(), 60);
        let u = item(run(
            &mut p,
            "update_item",
            json!({ "id": i.id, "end": "2026-10-20T19:00" }),
        ));
        assert_eq!(u.duration_min, Some(90));
    }

    #[test]
    fn complete_and_uncomplete_set_timestamp_and_are_idempotent() {
        let mut p = planner();
        let i = item(run(&mut p, "create_item", json!({ "title": "Run" })));
        let done = item(run(&mut p, "complete_item", json!({ "id": i.id })));
        assert_eq!(done.status, ItemStatus::Done);
        assert_eq!(done.completed_at, Some(crate::planner::tests::now()));
        let again = p
            .execute("complete_item", json!({ "id": i.id }), &Actor::User)
            .unwrap();
        assert!(again.changes.is_empty());
        let open = item(run(&mut p, "uncomplete_item", json!({ "id": i.id })));
        assert_eq!(open.status, ItemStatus::Open);
        assert_eq!(open.completed_at, None);
    }

    #[test]
    fn schedule_and_unschedule_round_trip_through_inbox() {
        let mut p = planner();
        let i = item(run(
            &mut p,
            "create_item",
            json!({ "title": "Fix tap", "duration_min": 20 }),
        ));
        let s = item(run(
            &mut p,
            "schedule_item",
            json!({ "id": i.id, "start": "2026-10-21T18:00" }),
        ));
        assert!(!s.is_inbox());
        assert_eq!((s.end_at.unwrap() - s.start_at.unwrap()).num_minutes(), 20);
        run(
            &mut p,
            "update_item",
            json!({ "id": i.id, "anchoring": "fixed" }),
        );
        let back = item(run(&mut p, "unschedule_item", json!({ "id": i.id })));
        assert!(back.is_inbox(), "unscheduling makes it flexible again");
        assert_eq!(back.duration_min, Some(20), "duration survives");
    }

    #[test]
    fn unschedule_refuses_events() {
        let mut p = planner();
        let e = item(run(
            &mut p,
            "create_item",
            json!({ "title": "Party", "kind": "event", "start": "2026-10-20T19:00" }),
        ));
        let err = p
            .execute("unschedule_item", json!({ "id": e.id }), &Actor::User)
            .unwrap_err();
        assert!(err.to_string().contains("only tasks"));
    }

    #[test]
    fn delete_logs_item_and_its_reminders() {
        let mut p = planner();
        let i = item(run(
            &mut p,
            "create_item",
            json!({ "title": "Dentist", "kind": "event", "start": "2026-10-20T15:00", "reminders": [] }),
        ));
        run(
            &mut p,
            "set_reminder",
            json!({ "item_id": i.id, "offset_min": 10 }),
        );
        let out = p
            .execute("delete_item", json!({ "id": i.id }), &Actor::User)
            .unwrap();
        let entities: Vec<_> = out.changes.iter().map(|c| c.entity.as_str()).collect();
        assert_eq!(entities, ["reminder", "item"]);
        assert!(out.changes.iter().all(|c| c.after.is_none()));
        assert!(store::items::get(p.db().conn(), &i.id).unwrap().is_none());
    }

    #[test]
    fn ai_and_mcp_actors_set_source_and_actor() {
        let mut p = planner();
        let ai = Actor::Ai {
            provider: "anthropic".into(),
            model: "claude-sonnet-5-5".into(),
        };
        let out = p
            .execute("create_item", json!({ "title": "Call Mum" }), &ai)
            .unwrap();
        assert_eq!(item(out.result).source, Source::Ai);
        assert_eq!(out.changes[0].actor, ai);
        let out = p
            .execute(
                "create_item",
                json!({ "title": "From Claude Code" }),
                &Actor::Mcp,
            )
            .unwrap();
        assert_eq!(item(out.result).source, Source::Mcp);
    }

    #[test]
    fn kinds_parse_from_json() {
        assert_eq!(
            serde_json::from_value::<ItemKind>(json!("reminder")).unwrap(),
            ItemKind::Reminder
        );
    }
}

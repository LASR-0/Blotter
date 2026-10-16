//! `import_ics`: writes an iCalendar file's items as one batch, so a whole
//! import undoes in one step. The file is read by [`crate::ics::import`].

use std::collections::HashMap;

use schemars::JsonSchema;
use serde::Deserialize;

use super::items::{ENTITY, finalize};
use super::slots::check_placement;
use super::{Ctx, OpDef};
use crate::error::Result;
use crate::ics::import::{self, Alarm, ImportSummary};
use crate::model::{Anchoring, Item, ItemStatus, Reminder, Source};
use crate::recurrence;
use crate::store;

pub(super) fn defs() -> Vec<OpDef> {
    vec![OpDef::new(
        "import_ics",
        "Import the events and tasks in an iCalendar (.ics) file. Items already imported \
         (by UID) are left out. Returns what was added.",
        true,
        import_ics,
    )]
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ImportIcsArgs {
    /// The file's text.
    pub ics: String,
}

/// Validates `item` for writing. A flexible item that lands in a
/// block-out is made fixed: it's where the other calendar had it.
fn place(ctx: &Ctx<'_>, item: &mut Item) -> Result<()> {
    finalize(ctx, item)?;
    if check_placement(ctx, None, item).is_err() {
        item.anchoring = Anchoring::Fixed;
        finalize(ctx, item)?;
    }
    Ok(())
}

fn write(ctx: &mut Ctx<'_>, item: &Item) -> Result<()> {
    store::items::insert(ctx.conn, item)?;
    ctx.record(ENTITY, &item.id, None, Some(item))
}

/// Makes `item` new: its id, source and times of writing.
fn fresh(ctx: &mut Ctx<'_>, item: &mut Item, order: usize) {
    item.id = ctx.new_id();
    item.source = Source::Import;
    item.created_at = ctx.now;
    item.updated_at = ctx.now;
    item.sort_order = ctx.now.timestamp() as f64 + order as f64 / 1000.0;
}

fn import_ics(ctx: &mut Ctx<'_>, a: ImportIcsArgs) -> Result<ImportSummary> {
    let tz = ctx.settings.tz;
    let plan = import::plan(ctx.conn, tz, &a.ics, ctx.now)?;
    let mut summary = plan.summary;
    let mut failed: Vec<String> = Vec::new();
    // UID → the stored series, for its changed occurrences.
    let mut series: HashMap<String, Item> = HashMap::new();

    for (order, entry) in plan.entries.into_iter().enumerate() {
        let mut item = entry.item;
        fresh(ctx, &mut item, order);
        if let Err(e) = place(ctx, &mut item) {
            failed.push(format!("{} ({e})", item.title));
            if item.is_scheduled() {
                summary.scheduled -= 1;
            } else {
                summary.inbox -= 1;
            }
            continue;
        }
        write(ctx, &item)?;
        for alarm in entry.alarms {
            let (offset_min, at) = match alarm {
                Alarm::Before(m) => (Some(m), None),
                Alarm::At(t) => (None, Some(t)),
            };
            let r = Reminder {
                id: ctx.new_id(),
                item_id: item.id.clone(),
                offset_min,
                at,
                fired_at: None,
                snoozed_until: None,
                created_at: ctx.now,
            };
            store::reminders::upsert(ctx.conn, &r)?;
            ctx.record(super::reminders::ENTITY, &r.id, None, Some(&r))?;
        }
        for rid in entry.exdates {
            match recurrence::find_occurrence(&item, tz, &rid)? {
                Some(slot) => {
                    let mut gone = recurrence::occurrence_item(&item, slot);
                    fresh(ctx, &mut gone, order);
                    gone.rrule = None;
                    gone.ics_uid = None;
                    gone.status = ItemStatus::Cancelled;
                    write(ctx, &gone)?;
                }
                // Not one of its occurrences: nothing to delete.
                None => summary.occurrences -= 1,
            }
        }
        if let (Some(uid), Some(_)) = (item.ics_uid.clone(), &item.rrule) {
            series.insert(uid, item);
        }
    }

    for (order, change) in plan.changes.into_iter().enumerate() {
        summary.occurrences -= 1;
        let Some(parent) = series.get(&change.uid) else {
            continue;
        };
        let slot = recurrence::find_occurrence(parent, tz, &change.recurrence_id)?;
        let taken = store::items::list_children(ctx.conn, &parent.id)?
            .iter()
            .any(|c| c.recurrence_id.as_deref() == Some(change.recurrence_id.as_str()));
        let mut item = match (change.item, slot) {
            (_, Some(_)) if taken => continue,
            (Some(mut moved), Some(_)) => {
                moved.parent_id = Some(parent.id.clone());
                moved.recurrence_id = Some(change.recurrence_id);
                moved
            }
            (None, Some(slot)) => {
                let mut gone = recurrence::occurrence_item(parent, slot);
                gone.status = ItemStatus::Cancelled;
                gone.ics_uid = None;
                gone.rrule = None;
                gone
            }
            // Its series has no such occurrence: keep a changed one as an
            // item of its own.
            (Some(lone), None) => lone,
            (None, None) => continue,
        };
        fresh(ctx, &mut item, order);
        if item.status != ItemStatus::Cancelled
            && let Err(e) = place(ctx, &mut item)
        {
            failed.push(format!("{} ({e})", item.title));
            continue;
        }
        write(ctx, &item)?;
        summary.occurrences += 1;
    }

    if !failed.is_empty() {
        summary.skipped += failed.len();
        summary
            .warnings
            .push(format!("Left out: {}.", failed.join("; ")));
    }
    Ok(summary)
}

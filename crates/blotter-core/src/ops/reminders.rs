//! Reminder mutations.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::input::DateTimeInput;
use super::{Ctx, OpDef, require_item};
use crate::error::{CoreError, Result};
use crate::id::new_id;
use crate::model::Reminder;
use crate::store;

pub(crate) const ENTITY: &str = "reminder";

pub(super) fn defs() -> Vec<OpDef> {
    vec![
        OpDef::new(
            "set_reminder",
            "Add a reminder to an item, either `offset_min` minutes before its start or at an \
             absolute time `at`. Pass `reminder_id` to change an existing reminder instead.",
            true,
            set_reminder,
        ),
        OpDef::new(
            "clear_reminder",
            "Remove one reminder by `reminder_id`, or every reminder on `item_id`.",
            true,
            clear_reminder,
        ),
    ]
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SetReminderArgs {
    pub item_id: String,
    /// Change this reminder instead of adding one.
    pub reminder_id: Option<String>,
    /// Minutes before the item's start. Give this or `at`.
    #[schemars(range(max = 40320))]
    pub offset_min: Option<u32>,
    /// Absolute time. Give this or `offset_min`.
    pub at: Option<DateTimeInput>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ClearReminderArgs {
    pub reminder_id: Option<String>,
    pub item_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Cleared {
    pub cleared: Vec<String>,
}

/// The series id for an occurrence id, else `id` itself.
fn series_or_self(id: &str) -> &str {
    crate::recurrence::split_occurrence_id(id).map_or(id, |(series, _)| series)
}

fn set_reminder(ctx: &mut Ctx<'_>, a: SetReminderArgs) -> Result<Reminder> {
    // A reminder on one occurrence belongs to its series (it fires for each).
    let item = require_item(ctx, series_or_self(&a.item_id))?;
    let at = match (a.offset_min, a.at) {
        (Some(_), None) => {
            if !item.is_scheduled() {
                return Err(CoreError::invalid(
                    "an offset reminder needs the item to be scheduled; use `at` instead",
                ));
            }
            None
        }
        (None, Some(at)) => Some(at.resolve(ctx.settings.tz)),
        _ => {
            return Err(CoreError::invalid(
                "give exactly one of `offset_min` or `at`",
            ));
        }
    };

    let before = match &a.reminder_id {
        Some(id) => {
            let r = store::reminders::require(ctx.conn, id)?;
            if r.item_id != item.id {
                return Err(CoreError::invalid(format!(
                    "reminder {id} belongs to a different item"
                )));
            }
            Some(r)
        }
        None => None,
    };

    let reminder = Reminder {
        id: before.as_ref().map(|r| r.id.clone()).unwrap_or_else(new_id),
        item_id: item.id,
        offset_min: a.offset_min,
        at,
        // A changed reminder may fire again.
        fired_at: None,
        snoozed_until: None,
        created_at: before.as_ref().map(|r| r.created_at).unwrap_or(ctx.now),
    };
    store::reminders::upsert(ctx.conn, &reminder)?;
    ctx.record(ENTITY, &reminder.id, before.as_ref(), Some(&reminder))?;
    Ok(reminder)
}

fn clear_reminder(ctx: &mut Ctx<'_>, a: ClearReminderArgs) -> Result<Cleared> {
    let targets = match (a.reminder_id, a.item_id) {
        (Some(id), None) => vec![store::reminders::require(ctx.conn, &id)?],
        (None, Some(item_id)) => {
            let item = require_item(ctx, series_or_self(&item_id))?;
            store::reminders::list_for_item(ctx.conn, &item.id)?
        }
        _ => {
            return Err(CoreError::invalid(
                "give exactly one of `reminder_id` or `item_id`",
            ));
        }
    };
    let mut cleared = Vec::with_capacity(targets.len());
    for r in targets {
        ctx.record(ENTITY, &r.id, Some(&r), None)?;
        store::reminders::delete(ctx.conn, &r.id)?;
        cleared.push(r.id);
    }
    Ok(Cleared { cleared })
}

#[cfg(test)]
mod tests {
    use crate::model::{Actor, Item, Reminder};
    use serde_json::json;

    #[test]
    fn set_update_and_clear_reminders() {
        let mut p = crate::planner::tests::planner();
        let item: Item = serde_json::from_value(
            p.execute(
                "create_item",
                json!({ "title": "Dentist", "kind": "event", "start": "2026-10-20T15:00", "reminders": [] }),
                &Actor::User,
            )
            .unwrap()
            .result,
        )
        .unwrap();

        let r: Reminder = serde_json::from_value(
            p.execute(
                "set_reminder",
                json!({ "item_id": item.id, "offset_min": 10 }),
                &Actor::User,
            )
            .unwrap()
            .result,
        )
        .unwrap();
        assert_eq!(r.offset_min, Some(10));

        let out = p
            .execute(
                "set_reminder",
                json!({ "item_id": item.id, "reminder_id": r.id, "at": "2026-10-20T14:00" }),
                &Actor::User,
            )
            .unwrap();
        let changed: Reminder = serde_json::from_value(out.result).unwrap();
        assert_eq!(changed.id, r.id);
        assert_eq!(changed.offset_min, None);
        assert!(changed.at.is_some());
        assert!(
            out.changes[0].before.is_some(),
            "update logs the previous state"
        );

        p.execute(
            "set_reminder",
            json!({ "item_id": item.id, "offset_min": 60 }),
            &Actor::User,
        )
        .unwrap();
        let out = p
            .execute(
                "clear_reminder",
                json!({ "item_id": item.id }),
                &Actor::User,
            )
            .unwrap();
        assert_eq!(out.result["cleared"].as_array().unwrap().len(), 2);
        assert_eq!(out.changes.len(), 2);
    }

    #[test]
    fn offset_reminder_needs_scheduled_item() {
        let mut p = crate::planner::tests::planner();
        let item: Item = serde_json::from_value(
            p.execute("create_item", json!({ "title": "Someday" }), &Actor::User)
                .unwrap()
                .result,
        )
        .unwrap();
        let err = p
            .execute(
                "set_reminder",
                json!({ "item_id": item.id, "offset_min": 5 }),
                &Actor::User,
            )
            .unwrap_err();
        assert!(err.to_string().contains("use `at`"));
        assert!(
            p.execute(
                "set_reminder",
                json!({ "item_id": item.id, "offset_min": 5, "at": "2026-10-20T10:00" }),
                &Actor::User
            )
            .is_err()
        );
    }
}

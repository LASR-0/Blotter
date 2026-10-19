//! Undo and redo from the change log (roadmap §4.9).
//!
//! Undo reverses the newest batch of edits that hasn't been undone, by the
//! user or the AI (automatic system changes such as the rollover are
//! skipped), by writing each entity back to its `before` state; redo
//! re-applies the batch the latest undo reversed, as long as nothing has
//! been edited since. Both write their own change rows (op `undo` / `redo`,
//! `reverts` naming the batch), so the log stays append-only.
//!
//! Before writing, each entity must still be as the batch left it. If
//! something changed it since (say an older batch is undone by id after
//! newer edits), the undo is refused rather than overwriting that change.

use std::collections::BTreeSet;

use rusqlite::Connection;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{Ctx, OpDef};
use crate::error::{CoreError, Result};
use crate::model::{Blockout, BlockoutException, Change, Item, Reminder};
use crate::store;

pub(super) fn defs() -> Vec<OpDef> {
    vec![
        OpDef::new(
            "undo",
            "Undo the most recent change, or the batch `batch_id`. Refused if what it changed \
             has been edited since.",
            true,
            undo,
        ),
        OpDef::new(
            "redo",
            "Redo the change the last undo reversed, if nothing has been edited since.",
            true,
            redo,
        ),
        OpDef::new(
            "get_undo_state",
            "What undo and redo would do next, each with a short description.",
            false,
            get_undo_state,
        ),
    ]
}

#[derive(Debug, Default, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct UndoArgs {
    /// A specific batch to undo (e.g. an AI batch); default: the latest.
    pub batch_id: Option<String>,
}

#[derive(Debug, Default, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct NoArgs {}

/// A batch undo or redo would act on.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BatchInfo {
    pub batch_id: String,
    /// "Create item “Gym”", "Auto-slot 3 items".
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct UndoState {
    pub undo: Option<BatchInfo>,
    pub redo: Option<BatchInfo>,
}

/// Reads an entity's current state as the change log records it.
fn current(conn: &Connection, entity: &str, id: &str) -> Result<Option<Value>> {
    Ok(match entity {
        "item" => store::items::get(conn, id)?
            .map(serde_json::to_value)
            .transpose()?,
        "reminder" => store::reminders::get(conn, id)?
            .map(serde_json::to_value)
            .transpose()?,
        "blockout" => store::blockouts::get(conn, id)?
            .map(serde_json::to_value)
            .transpose()?,
        "blockout_exception" => store::blockouts::get_exception(conn, id)?
            .map(serde_json::to_value)
            .transpose()?,
        "setting" => store::settings::get::<Value>(conn, id)?,
        "tag" => store::tags::get(conn, id)?
            .map(serde_json::to_value)
            .transpose()?,
        other => {
            return Err(CoreError::invalid(format!(
                "changes to '{other}' can't be undone"
            )));
        }
    })
}

/// Fields the app updates on its own, which an undo neither checks nor
/// rewinds: when a reminder last fired, and its pending snooze.
const ENGINE_FIELDS: [(&str, &str); 2] = [("reminder", "fired_at"), ("reminder", "snoozed_until")];

fn comparable(entity: &str, state: Option<&Value>) -> Option<Value> {
    let mut v = state.cloned()?;
    if let Some(obj) = v.as_object_mut() {
        for (e, field) in ENGINE_FIELDS {
            if e == entity {
                obj.remove(field);
            }
        }
    }
    Some(v)
}

fn from_state<T: serde::de::DeserializeOwned>(v: &Value) -> Result<T> {
    Ok(serde_json::from_value(v.clone())?)
}

/// Writes `state` (or deletes, for `None`) without touching the change log.
fn write(ctx: &Ctx<'_>, entity: &str, id: &str, state: Option<&Value>) -> Result<()> {
    let conn = ctx.conn;
    let Some(state) = state else {
        match entity {
            "item" => store::items::delete(conn, id).map(|_| ())?,
            "reminder" => store::reminders::delete(conn, id).map(|_| ())?,
            "blockout" => store::blockouts::delete(conn, id).map(|_| ())?,
            "blockout_exception" => store::blockouts::delete_exception(conn, id).map(|_| ())?,
            "setting" => store::settings::delete(conn, id)?,
            "tag" => store::tags::delete(conn, id).map(|_| ())?,
            _ => unreachable!("checked by current()"),
        }
        return Ok(());
    };

    match entity {
        "item" => store::items::upsert(conn, &from_state::<Item>(state)?)?,
        "reminder" => {
            let mut r: Reminder = from_state(state)?;
            // Keep the engine's record of when it last fired, so a restored
            // reminder doesn't fire again for something already shown.
            if let Some(now) = store::reminders::get(conn, id)? {
                r.fired_at = now.fired_at;
            }
            store::reminders::upsert(conn, &r)?;
        }
        "blockout" => store::blockouts::upsert(conn, &from_state::<Blockout>(state)?)?,
        "blockout_exception" => {
            store::blockouts::upsert_exception(conn, &from_state::<BlockoutException>(state)?)?;
        }
        "setting" => store::settings::set(conn, id, state, ctx.now)?,
        "tag" => store::tags::upsert(conn, &from_state(state)?)?,
        _ => unreachable!("checked by current()"),
    }
    Ok(())
}

/// "create_item" → "Create item".
fn op_label(op: &str) -> String {
    let words = op
        .replace('_', " ")
        .replace("blockout", "block-out")
        .replace("auto slot", "auto-slot")
        .replace("import ics", "import ICS");
    let mut chars = words.chars();
    chars
        .next()
        .map(|c| c.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}

/// The entity's name, from whichever state has one.
fn name_of(c: &Change) -> Option<String> {
    [c.after.as_ref(), c.before.as_ref()]
        .into_iter()
        .flatten()
        .find_map(|v| {
            ["title", "label", "name"]
                .into_iter()
                .find_map(|k| v.get(k).and_then(Value::as_str).map(str::to_owned))
        })
        .or_else(|| (c.entity == "setting").then(|| c.entity_id.replace('_', " ")))
}

/// Which kind of entity names a batch: the lower, the more important.
fn rank(entity: &str) -> u8 {
    match entity {
        "item" => 0,
        "blockout" => 1,
        "reminder" => 2,
        "blockout_exception" => 3,
        _ => 4,
    }
}

/// An occurrence override: an item with a parent series.
fn is_override(c: &Change) -> bool {
    [c.after.as_ref(), c.before.as_ref()]
        .into_iter()
        .flatten()
        .any(|v| v.get("parent_id").is_some_and(|p| !p.is_null()))
}

/// "Create item “Gym”", "Auto slot (3 items)", "Delete block-out “Work”".
pub(crate) fn summarize(rows: &[Change]) -> String {
    // The batch is about its most important kind of entity (deleting a
    // block-out also logs its exceptions; quick add's create also logs its
    // reminder), leaving out a series' occurrence overrides.
    let Some(first) = rows.iter().min_by_key(|c| rank(&c.entity)) else {
        return String::new();
    };
    let label = op_label(&first.op);
    let principal: Vec<&Change> = rows
        .iter()
        .filter(|c| c.entity == first.entity && !is_override(c))
        .collect();
    let main: BTreeSet<&str> = principal.iter().map(|c| c.entity_id.as_str()).collect();
    if main.len() <= 1 {
        match principal
            .first()
            .copied()
            .and_then(name_of)
            .or_else(|| name_of(first))
        {
            Some(name) => format!("{label} “{name}”"),
            None => label,
        }
    } else {
        let what = match first.entity.as_str() {
            "item" => "items",
            "reminder" => "reminders",
            "setting" => "settings",
            "blockout" => "block-outs",
            _ => "changes",
        };
        format!("{label} ({} {what})", main.len())
    }
}

fn info(conn: &Connection, batch_id: Option<String>) -> Result<Option<BatchInfo>> {
    let Some(batch_id) = batch_id else {
        return Ok(None);
    };
    let rows = store::changes::list_batch(conn, &batch_id)?;
    Ok(Some(BatchInfo {
        summary: summarize(&rows),
        batch_id,
    }))
}

fn get_undo_state(ctx: &mut Ctx<'_>, _: NoArgs) -> Result<UndoState> {
    Ok(UndoState {
        undo: info(ctx.conn, store::changes::newest_undoable(ctx.conn)?)?,
        redo: info(ctx.conn, store::changes::newest_redoable(ctx.conn)?)?,
    })
}

/// Applies one direction of a batch: undo walks it backwards restoring
/// `before`, redo walks it forwards restoring `after`.
fn apply(ctx: &mut Ctx<'_>, batch_id: &str, undoing: bool) -> Result<BatchInfo> {
    let mut rows = store::changes::list_batch(ctx.conn, batch_id)?;
    if rows.is_empty() {
        return Err(CoreError::not_found("batch", batch_id));
    }
    if rows.iter().any(|c| c.op == "undo" || c.op == "redo") {
        return Err(CoreError::invalid(
            "that batch is itself an undo or redo; use redo or undo instead",
        ));
    }
    let summary = summarize(&rows);
    if undoing {
        rows.reverse();
    }
    // Restoring a deletion can put a reminder back before its item; foreign
    // keys are checked at commit, when everything is in place.
    ctx.conn.pragma_update(None, "defer_foreign_keys", "ON")?;
    ctx.reverts = Some(batch_id.to_owned());
    for c in &rows {
        let (expect, target) = if undoing {
            (c.after.as_ref(), c.before.as_ref())
        } else {
            (c.before.as_ref(), c.after.as_ref())
        };
        let now = current(ctx.conn, &c.entity, &c.entity_id)?;
        // Already gone: deleting an item takes its reminders and overrides
        // with it (ON DELETE CASCADE), before their own rows come up.
        if target.is_none() && now.is_none() {
            continue;
        }
        if comparable(&c.entity, now.as_ref()) != comparable(&c.entity, expect) {
            let name = name_of(c).map_or_else(|| c.entity.clone(), |n| format!("“{n}”"));
            return Err(CoreError::invalid(format!(
                "can't {} “{summary}”: {name} has changed since",
                if undoing { "undo" } else { "redo" }
            )));
        }
        write(ctx, &c.entity, &c.entity_id, target)?;
        let after = current(ctx.conn, &c.entity, &c.entity_id)?;
        ctx.record(&c.entity, &c.entity_id, now.as_ref(), after.as_ref())?;
    }
    ctx.reverts = None;
    store::changes::set_undone(ctx.conn, batch_id, undoing.then_some(ctx.now))?;
    Ok(BatchInfo {
        batch_id: batch_id.to_owned(),
        summary,
    })
}

fn undo(ctx: &mut Ctx<'_>, a: UndoArgs) -> Result<Option<BatchInfo>> {
    let target = match a.batch_id {
        Some(id) => {
            let rows = store::changes::list_batch(ctx.conn, &id)?;
            if rows.iter().any(|c| c.undone_at.is_some()) {
                return Err(CoreError::invalid("that change is already undone"));
            }
            Some(id)
        }
        None => store::changes::newest_undoable(ctx.conn)?,
    };
    target.map(|id| apply(ctx, &id, true)).transpose()
}

fn redo(ctx: &mut Ctx<'_>, _: NoArgs) -> Result<Option<BatchInfo>> {
    store::changes::newest_redoable(ctx.conn)?
        .map(|id| apply(ctx, &id, false))
        .transpose()
}

#[cfg(test)]
mod tests {
    use crate::model::Actor;
    use serde_json::{Value, json};

    fn run(p: &mut crate::Planner, op: &str, args: Value) -> Value {
        p.execute(op, args, &Actor::User).unwrap().result
    }

    fn try_run(p: &mut crate::Planner, op: &str, args: Value) -> crate::Result<Value> {
        p.execute(op, args, &Actor::User).map(|o| o.result)
    }

    fn item(p: &crate::Planner, id: &str) -> Option<crate::model::Item> {
        crate::store::items::get(p.db().conn(), id).unwrap()
    }

    #[test]
    fn op_labels_read_like_the_app() {
        assert_eq!(super::op_label("create_item"), "Create item");
        assert_eq!(super::op_label("auto_slot"), "Auto-slot");
        assert_eq!(super::op_label("delete_blockout"), "Delete block-out");
    }

    #[test]
    fn undo_and_redo_a_sequence_of_edits() {
        let mut p = crate::planner::tests::planner();
        let gym = run(&mut p, "create_item", json!({ "title": "Gym" }));
        let id = gym["id"].as_str().unwrap().to_owned();
        run(
            &mut p,
            "update_item",
            json!({ "id": id, "title": "Gym (legs)" }),
        );
        run(&mut p, "complete_item", json!({ "id": id }));

        let state = run(&mut p, "get_undo_state", json!({}));
        assert_eq!(state["undo"]["summary"], "Complete item “Gym (legs)”");
        assert!(state["redo"].is_null());

        // Undo all three, newest first.
        assert_eq!(
            run(&mut p, "undo", json!({}))["summary"],
            "Complete item “Gym (legs)”"
        );
        assert_eq!(
            item(&p, &id).unwrap().status,
            crate::model::ItemStatus::Open
        );
        assert_eq!(
            run(&mut p, "undo", json!({}))["summary"],
            "Update item “Gym (legs)”"
        );
        assert_eq!(item(&p, &id).unwrap().title, "Gym");
        assert_eq!(
            run(&mut p, "undo", json!({}))["summary"],
            "Create item “Gym”"
        );
        assert!(item(&p, &id).is_none());
        assert!(run(&mut p, "undo", json!({})).is_null(), "nothing left");

        // Redo them back, oldest first.
        assert_eq!(
            run(&mut p, "redo", json!({}))["summary"],
            "Create item “Gym”"
        );
        assert_eq!(
            run(&mut p, "redo", json!({}))["summary"],
            "Update item “Gym (legs)”"
        );
        assert_eq!(item(&p, &id).unwrap().title, "Gym (legs)");
        assert_eq!(
            run(&mut p, "redo", json!({}))["summary"],
            "Complete item “Gym (legs)”"
        );
        assert!(run(&mut p, "redo", json!({})).is_null());
        assert_eq!(
            item(&p, &id).unwrap().status,
            crate::model::ItemStatus::Done
        );

        // And undo works again after a redo.
        run(&mut p, "undo", json!({}));
        assert_eq!(
            item(&p, &id).unwrap().status,
            crate::model::ItemStatus::Open
        );
    }

    #[test]
    fn system_changes_are_skipped() {
        let mut p = crate::planner::tests::planner();
        run(&mut p, "create_item", json!({ "title": "Mine" }));
        // An automatic change after it, e.g. the rollover.
        p.execute(
            "create_item",
            json!({ "title": "Automatic" }),
            &Actor::System,
        )
        .unwrap();
        let state = run(&mut p, "get_undo_state", json!({}));
        assert_eq!(state["undo"]["summary"], "Create item “Mine”");
        run(&mut p, "undo", json!({}));
        // A system change after an undo doesn't clear redo either.
        p.execute("create_item", json!({ "title": "Later" }), &Actor::System)
            .unwrap();
        assert_eq!(
            run(&mut p, "get_undo_state", json!({}))["redo"]["summary"],
            "Create item “Mine”"
        );
    }

    #[test]
    fn a_new_edit_clears_redo() {
        let mut p = crate::planner::tests::planner();
        run(&mut p, "create_item", json!({ "title": "A" }));
        run(&mut p, "create_item", json!({ "title": "B" }));
        run(&mut p, "undo", json!({}));
        assert_eq!(
            run(&mut p, "get_undo_state", json!({}))["redo"]["summary"],
            "Create item “B”"
        );
        run(&mut p, "create_item", json!({ "title": "C" }));
        let state = run(&mut p, "get_undo_state", json!({}));
        assert!(state["redo"].is_null());
        assert_eq!(state["undo"]["summary"], "Create item “C”");
    }

    #[test]
    fn undoing_a_delete_restores_reminders_and_overrides() {
        let mut p = crate::planner::tests::planner();
        let s = run(
            &mut p,
            "create_item",
            json!({ "title": "Standup", "kind": "event", "start": "2026-10-12T09:00",
                    "rrule": "FREQ=DAILY", "reminders": [5] }),
        );
        let sid = s["id"].as_str().unwrap().to_owned();
        // Move Wednesday's occurrence: an override child.
        run(
            &mut p,
            "update_item",
            json!({ "id": format!("{sid}@2026-10-13T22:00:00Z"), "start": "2026-10-14T10:00" }),
        );
        let conn = |p: &crate::Planner| {
            (
                crate::store::reminders::list_for_item(p.db().conn(), &sid)
                    .unwrap()
                    .len(),
                crate::store::items::list_children(p.db().conn(), &sid)
                    .unwrap()
                    .len(),
            )
        };
        assert_eq!(conn(&p), (1, 1));
        run(&mut p, "delete_item", json!({ "id": sid }));
        assert!(item(&p, &sid).is_none());
        assert_eq!(conn(&p), (0, 0));

        let undone = run(&mut p, "undo", json!({}));
        assert_eq!(undone["summary"], "Delete item “Standup”");
        assert!(item(&p, &sid).is_some());
        assert_eq!(conn(&p), (1, 1), "reminder and override are back");

        run(&mut p, "redo", json!({}));
        assert!(item(&p, &sid).is_none());
    }

    #[test]
    fn refuses_to_clobber_later_edits() {
        let mut p = crate::planner::tests::planner();
        let gym = run(&mut p, "create_item", json!({ "title": "Gym" }));
        let id = gym["id"].as_str().unwrap().to_owned();
        let first = p
            .execute(
                "update_item",
                json!({ "id": id, "title": "Gym 2" }),
                &Actor::User,
            )
            .unwrap()
            .batch_id
            .unwrap();
        run(&mut p, "update_item", json!({ "id": id, "title": "Gym 3" }));
        let err = try_run(&mut p, "undo", json!({ "batch_id": first })).unwrap_err();
        assert!(err.to_string().contains("has changed since"), "{err}");
        assert_eq!(item(&p, &id).unwrap().title, "Gym 3", "nothing was written");
    }

    #[test]
    fn settings_and_blockouts_undo() {
        let mut p = crate::planner::tests::planner();
        run(
            &mut p,
            "update_settings",
            json!({ "snooze_options_min": [10] }),
        );
        let b = run(
            &mut p,
            "create_blockout",
            json!({ "label": "Work", "weekdays": ["mon", "tue", "wed", "thu", "fri"], "start_time": "09:00",
                    "end_time": "17:00" }),
        );
        run(
            &mut p,
            "add_blockout_exception",
            json!({ "blockout_id": b["id"], "date": "2026-10-16" }),
        );
        run(&mut p, "delete_blockout", json!({ "id": b["id"] }));
        assert_eq!(
            run(&mut p, "undo", json!({}))["summary"],
            "Delete block-out “Work”"
        );
        let back = crate::store::blockouts::list_exceptions(
            p.db().conn(),
            Some(b["id"].as_str().unwrap()),
            None,
        )
        .unwrap();
        assert_eq!(back.len(), 1, "its exception is restored too");

        run(&mut p, "undo", json!({})); // the exception
        run(&mut p, "undo", json!({})); // the block-out
        assert_eq!(
            run(&mut p, "undo", json!({}))["summary"],
            "Update settings “snooze options min”"
        );
        assert_eq!(
            run(&mut p, "get_settings", json!({}))["snooze_options_min"],
            json!([5, 15, 60]),
            "back to the default"
        );
    }

    #[test]
    fn undo_rows_are_logged_and_marked() {
        let mut p = crate::planner::tests::planner();
        let made = p
            .execute("create_item", json!({ "title": "Gym" }), &Actor::User)
            .unwrap();
        let batch = made.batch_id.unwrap();
        let out = p.execute("undo", json!({}), &Actor::User).unwrap();
        assert!(out.changes.iter().all(|c| c.op == "undo"));
        assert!(
            out.changes
                .iter()
                .all(|c| c.reverts.as_deref() == Some(batch.as_str()))
        );
        let rows = crate::store::changes::list_batch(p.db().conn(), &batch).unwrap();
        assert!(rows.iter().all(|c| c.undone_at.is_some()));
    }
}

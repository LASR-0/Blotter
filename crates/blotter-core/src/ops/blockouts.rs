//! Block-out mutations.

use chrono_tz::Tz;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::input::{DateInput, TimeInput, patch};
use super::{Ctx, OpDef};
use crate::error::{CoreError, Result};
use crate::id::new_id;
use crate::model::{Blockout, BlockoutException, Weekdays};
use crate::store;

pub(crate) const ENTITY: &str = "blockout";
pub(crate) const EXCEPTION_ENTITY: &str = "blockout_exception";

pub(super) fn defs() -> Vec<OpDef> {
    vec![
        OpDef::new(
            "create_blockout",
            "Create a recurring block-out window (e.g. Work, Mon–Fri 08:30–17:00) that flexible \
             tasks are never placed in. An `end_time` before `start_time` wraps past midnight.",
            true,
            create_blockout,
        ),
        OpDef::new(
            "update_blockout",
            "Change fields of a block-out. Omitted fields are left as they are.",
            true,
            update_blockout,
        ),
        OpDef::new(
            "delete_blockout",
            "Delete a block-out and its exceptions.",
            true,
            delete_blockout,
        ),
        OpDef::new(
            "add_blockout_exception",
            "Lift a block-out on one date, e.g. for leave or a public holiday.",
            true,
            add_blockout_exception,
        ),
        OpDef::new(
            "remove_blockout_exception",
            "Undo a lifted date so the block-out applies again.",
            true,
            remove_blockout_exception,
        ),
    ]
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateBlockoutArgs {
    #[schemars(length(min = 1, max = 100))]
    pub label: String,
    pub weekdays: Weekdays,
    pub start_time: TimeInput,
    pub end_time: TimeInput,
    /// IANA timezone; defaults to the user's.
    pub tz: Option<String>,
    pub active_from: Option<DateInput>,
    pub active_until: Option<DateInput>,
    /// Whether fixed items may sit inside without a warning. Defaults to true;
    /// set false for windows like Sleep.
    pub permits_fixed: Option<bool>,
    pub color: Option<String>,
    /// Phosphor icon name, e.g. `briefcase`.
    pub icon: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct UpdateBlockoutArgs {
    pub id: String,
    #[schemars(length(min = 1, max = 100))]
    pub label: Option<String>,
    pub weekdays: Option<Weekdays>,
    pub start_time: Option<TimeInput>,
    pub end_time: Option<TimeInput>,
    pub tz: Option<String>,
    #[serde(default, deserialize_with = "patch")]
    pub active_from: Option<Option<DateInput>>,
    #[serde(default, deserialize_with = "patch")]
    pub active_until: Option<Option<DateInput>>,
    pub permits_fixed: Option<bool>,
    #[serde(default, deserialize_with = "patch")]
    pub color: Option<Option<String>>,
    #[serde(default, deserialize_with = "patch")]
    pub icon: Option<Option<String>>,
    pub sort_order: Option<i64>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BlockoutIdArgs {
    pub id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AddExceptionArgs {
    pub blockout_id: String,
    pub date: DateInput,
    #[serde(default)]
    pub note: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RemoveExceptionArgs {
    pub exception_id: String,
}

#[derive(Debug, Serialize)]
pub struct Deleted {
    pub deleted: String,
}

fn validate(b: &mut Blockout) -> Result<()> {
    b.label = b.label.trim().to_owned();
    if b.label.is_empty() {
        return Err(CoreError::invalid("label must not be empty"));
    }
    if b.start_time == b.end_time {
        return Err(CoreError::invalid("start_time and end_time must differ"));
    }
    b.tz.parse::<Tz>()
        .map_err(|_| CoreError::invalid(format!("unknown timezone '{}'", b.tz)))?;
    if let (Some(from), Some(until)) = (b.active_from, b.active_until) {
        if until < from {
            return Err(CoreError::invalid("active_until is before active_from"));
        }
    }
    if let Some(color) = &b.color {
        let hex = color.strip_prefix('#').unwrap_or("");
        if hex.len() != 6 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(CoreError::invalid(format!(
                "color '{color}' must look like #7d808c"
            )));
        }
    }
    Ok(())
}

fn create_blockout(ctx: &mut Ctx<'_>, a: CreateBlockoutArgs) -> Result<Blockout> {
    let next_order: i64 = ctx.conn.query_row(
        "SELECT COALESCE(MAX(sort_order) + 1, 0) FROM blockouts",
        [],
        |r| r.get(0),
    )?;
    let mut b = Blockout {
        id: new_id(),
        label: a.label,
        weekdays: a.weekdays,
        start_time: a.start_time.0,
        end_time: a.end_time.0,
        tz: a.tz.unwrap_or_else(|| ctx.settings.tz.name().to_owned()),
        active_from: a.active_from.map(|d| d.0),
        active_until: a.active_until.map(|d| d.0),
        permits_fixed: a.permits_fixed.unwrap_or(true),
        color: a.color,
        icon: a.icon,
        sort_order: next_order,
        created_at: ctx.now,
        updated_at: ctx.now,
    };
    validate(&mut b)?;
    store::blockouts::upsert(ctx.conn, &b)?;
    ctx.record(ENTITY, &b.id, None, Some(&b))?;
    Ok(b)
}

fn update_blockout(ctx: &mut Ctx<'_>, a: UpdateBlockoutArgs) -> Result<Blockout> {
    let before = store::blockouts::require(ctx.conn, &a.id)?;
    let mut b = before.clone();
    if let Some(v) = a.label {
        b.label = v;
    }
    if let Some(v) = a.weekdays {
        b.weekdays = v;
    }
    if let Some(v) = a.start_time {
        b.start_time = v.0;
    }
    if let Some(v) = a.end_time {
        b.end_time = v.0;
    }
    if let Some(v) = a.tz {
        b.tz = v;
    }
    if let Some(v) = a.active_from {
        b.active_from = v.map(|d| d.0);
    }
    if let Some(v) = a.active_until {
        b.active_until = v.map(|d| d.0);
    }
    if let Some(v) = a.permits_fixed {
        b.permits_fixed = v;
    }
    if let Some(v) = a.color {
        b.color = v;
    }
    if let Some(v) = a.icon {
        b.icon = v;
    }
    if let Some(v) = a.sort_order {
        b.sort_order = v;
    }
    validate(&mut b)?;
    if b == before {
        return Ok(b);
    }
    b.updated_at = ctx.now;
    store::blockouts::upsert(ctx.conn, &b)?;
    ctx.record(ENTITY, &b.id, Some(&before), Some(&b))?;
    Ok(b)
}

fn delete_blockout(ctx: &mut Ctx<'_>, a: BlockoutIdArgs) -> Result<Deleted> {
    let b = store::blockouts::require(ctx.conn, &a.id)?;
    for e in store::blockouts::list_exceptions(ctx.conn, Some(&b.id), None)? {
        ctx.record(EXCEPTION_ENTITY, &e.id, Some(&e), None)?;
    }
    ctx.record(ENTITY, &b.id, Some(&b), None)?;
    store::blockouts::delete(ctx.conn, &b.id)?;
    Ok(Deleted { deleted: b.id })
}

fn add_blockout_exception(ctx: &mut Ctx<'_>, a: AddExceptionArgs) -> Result<BlockoutException> {
    store::blockouts::require(ctx.conn, &a.blockout_id)?;
    let existing = store::blockouts::list_exceptions(
        ctx.conn,
        Some(&a.blockout_id),
        Some((a.date.0, a.date.0)),
    )?;
    if let Some(e) = existing.into_iter().next() {
        return Ok(e);
    }
    let e = BlockoutException {
        id: new_id(),
        blockout_id: a.blockout_id,
        date: a.date.0,
        note: a.note.trim().to_owned(),
        created_at: ctx.now,
    };
    store::blockouts::upsert_exception(ctx.conn, &e)?;
    ctx.record(EXCEPTION_ENTITY, &e.id, None, Some(&e))?;
    Ok(e)
}

fn remove_blockout_exception(ctx: &mut Ctx<'_>, a: RemoveExceptionArgs) -> Result<Deleted> {
    let e = store::blockouts::get_exception(ctx.conn, &a.exception_id)?
        .ok_or_else(|| CoreError::not_found("blockout_exception", &a.exception_id))?;
    ctx.record(EXCEPTION_ENTITY, &e.id, Some(&e), None)?;
    store::blockouts::delete_exception(ctx.conn, &e.id)?;
    Ok(Deleted { deleted: e.id })
}

#[cfg(test)]
mod tests {
    use crate::model::{Actor, Blockout};
    use serde_json::json;

    #[test]
    fn blockout_lifecycle_with_exceptions() {
        let mut p = crate::planner::tests::planner();
        let b: Blockout = serde_json::from_value(
            p.execute(
                "create_blockout",
                json!({ "label": "Work", "weekdays": ["mon","tue","wed","thu","fri"],
                        "start_time": "08:30", "end_time": "17:00", "icon": "briefcase" }),
                &Actor::User,
            )
            .unwrap()
            .result,
        )
        .unwrap();
        assert_eq!(b.tz, "Australia/Melbourne");
        assert!(b.permits_fixed);

        let u: Blockout = serde_json::from_value(
            p.execute(
                "update_blockout",
                json!({ "id": b.id, "end_time": "17:30" }),
                &Actor::User,
            )
            .unwrap()
            .result,
        )
        .unwrap();
        assert_eq!(u.end_time.to_string(), "17:30:00");

        let e = p
            .execute(
                "add_blockout_exception",
                json!({ "blockout_id": b.id, "date": "2026-12-25", "note": "Christmas" }),
                &Actor::User,
            )
            .unwrap();
        let again = p
            .execute(
                "add_blockout_exception",
                json!({ "blockout_id": b.id, "date": "2026-12-25" }),
                &Actor::User,
            )
            .unwrap();
        assert!(
            again.changes.is_empty(),
            "adding the same date twice is a no-op"
        );
        assert_eq!(again.result["id"], e.result["id"]);

        let out = p
            .execute("delete_blockout", json!({ "id": b.id }), &Actor::User)
            .unwrap();
        let entities: Vec<_> = out.changes.iter().map(|c| c.entity.as_str()).collect();
        assert_eq!(entities, ["blockout_exception", "blockout"]);
    }

    #[test]
    fn blockout_validation() {
        let mut p = crate::planner::tests::planner();
        for (args, needle) in [
            (
                json!({ "label": "X", "weekdays": [], "start_time": "08:00", "end_time": "09:00" }),
                "weekdays",
            ),
            (
                json!({ "label": "X", "weekdays": ["mon"], "start_time": "08:00", "end_time": "08:00" }),
                "differ",
            ),
            (
                json!({ "label": "X", "weekdays": ["mon"], "start_time": "8am", "end_time": "09:00" }),
                "start_time",
            ),
            (
                json!({ "label": " ", "weekdays": ["mon"], "start_time": "08:00", "end_time": "09:00" }),
                "label",
            ),
        ] {
            let err = p
                .execute("create_blockout", args.clone(), &Actor::User)
                .unwrap_err()
                .to_string();
            assert!(err.contains(needle), "{args} → {err}");
        }
        p.execute(
            "create_blockout",
            json!({ "label": "Sleep", "weekdays": ["mon","tue","wed","thu","fri","sat","sun"],
                    "start_time": "22:30", "end_time": "06:30", "permits_fixed": false }),
            &Actor::User,
        )
        .unwrap();
    }
}

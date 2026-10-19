//! Tag mutations: colour, create, rename (merging) and delete. Renaming and
//! deleting rewrite every item carrying the tag, each logged, so one undo
//! puts them all back.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{Ctx, OpDef};
use crate::error::{CoreError, Result};
use crate::store;
use crate::store::tags::Tag;

pub(crate) const ENTITY: &str = "tag";

pub(super) fn defs() -> Vec<OpDef> {
    vec![
        OpDef::new(
            "set_tag",
            "Create a tag, or change its colour. `color` is `#rrggbb` or a theme tag colour \
             (`var(--tag-work)`, `-health`, `-home`, `-shopping`, `-personal`, `-ics`); null \
             goes back to the automatic colour.",
            true,
            set_tag,
        ),
        OpDef::new(
            "rename_tag",
            "Rename a tag on every item that carries it. Renaming onto an existing tag merges \
             the two.",
            true,
            rename_tag,
        ),
        OpDef::new(
            "delete_tag",
            "Remove a tag from every item that carries it, and forget its colour.",
            true,
            delete_tag,
        ),
    ]
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SetTagArgs {
    /// Tag name, with or without the leading `#`.
    pub name: String,
    pub color: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RenameTagArgs {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TagNameArgs {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct TagChange {
    pub name: String,
    /// Items rewritten.
    pub items: usize,
}

/// `#Shopping ` → `shopping`; the quick-add grammar's letters, digits, `-` and `_`.
fn tag_name(raw: &str) -> Result<String> {
    let name = raw.trim().trim_start_matches('#').to_lowercase();
    if name.is_empty() || name.chars().count() > 50 {
        return Err(CoreError::invalid("a tag name is 1–50 characters"));
    }
    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(CoreError::invalid(format!(
            "'{name}': tags use letters, digits, '-' and '_' only"
        )));
    }
    Ok(name)
}

fn check_color(color: &str) -> Result<()> {
    let hex = color.len() == 7
        && color.starts_with('#')
        && color[1..].chars().all(|c| c.is_ascii_hexdigit());
    let token = color
        .strip_prefix("var(--tag-")
        .and_then(|r| r.strip_suffix(')'))
        .is_some_and(|t| !t.is_empty() && t.chars().all(|c| c.is_ascii_lowercase()));
    if hex || token {
        Ok(())
    } else {
        Err(CoreError::invalid(format!(
            "'{color}' isn't a tag colour: use #rrggbb or var(--tag-<name>)"
        )))
    }
}

fn set_tag(ctx: &mut Ctx<'_>, a: SetTagArgs) -> Result<Tag> {
    let name = tag_name(&a.name)?;
    let color = a
        .color
        .map(|c| c.trim().to_owned())
        .filter(|c| !c.is_empty());
    if let Some(c) = &color {
        check_color(c)?;
    }
    let before = store::tags::get(ctx.conn, &name)?;
    let tag = Tag { name, color };
    if before.as_ref() == Some(&tag) {
        return Ok(tag);
    }
    store::tags::upsert(ctx.conn, &tag)?;
    ctx.record(ENTITY, &tag.name, before.as_ref(), Some(&tag))?;
    Ok(tag)
}

/// Rewrites `from` on every item carrying it: to `to`, or dropped for `None`.
fn retag(ctx: &mut Ctx<'_>, from: &str, to: Option<&str>) -> Result<usize> {
    let ids = store::tags::items_with(ctx.conn, from)?;
    for id in &ids {
        let before = store::items::require(ctx.conn, id)?;
        let mut item = before.clone();
        let mut tags: Vec<String> = Vec::with_capacity(item.tags.len());
        for t in &item.tags {
            let t = if t.to_lowercase() == from {
                match to {
                    Some(to) => to.to_owned(),
                    None => continue,
                }
            } else {
                t.clone()
            };
            if !tags.iter().any(|x| x.to_lowercase() == t.to_lowercase()) {
                tags.push(t);
            }
        }
        item.tags = tags;
        item.updated_at = ctx.now;
        store::items::update(ctx.conn, &item)?;
        ctx.record("item", id, Some(&before), Some(&item))?;
    }
    Ok(ids.len())
}

fn rename_tag(ctx: &mut Ctx<'_>, a: RenameTagArgs) -> Result<TagChange> {
    let from = tag_name(&a.from)?;
    let to = tag_name(&a.to)?;
    if from == to {
        return Ok(TagChange { name: to, items: 0 });
    }
    let record = store::tags::get(ctx.conn, &from)?;
    let items = retag(ctx, &from, Some(&to))?;
    if items == 0 && record.is_none() {
        return Err(CoreError::not_found("tag", from));
    }
    // The colour moves with the name, unless the tag merged into already has one.
    if let Some(old) = record {
        store::tags::delete(ctx.conn, &from)?;
        ctx.record(ENTITY, &from, Some(&old), None)?;
        let existing = store::tags::get(ctx.conn, &to)?;
        if existing.as_ref().is_none_or(|t| t.color.is_none()) && old.color.is_some() {
            let moved = Tag {
                name: to.clone(),
                color: old.color,
            };
            store::tags::upsert(ctx.conn, &moved)?;
            ctx.record(ENTITY, &to, existing.as_ref(), Some(&moved))?;
        }
    }
    Ok(TagChange { name: to, items })
}

fn delete_tag(ctx: &mut Ctx<'_>, a: TagNameArgs) -> Result<TagChange> {
    let name = tag_name(&a.name)?;
    let record = store::tags::get(ctx.conn, &name)?;
    let items = retag(ctx, &name, None)?;
    if let Some(old) = record {
        store::tags::delete(ctx.conn, &name)?;
        ctx.record(ENTITY, &name, Some(&old), None)?;
    } else if items == 0 {
        return Err(CoreError::not_found("tag", name));
    }
    Ok(TagChange { name, items })
}

#[cfg(test)]
mod tests {
    use crate::model::Actor;
    use serde_json::{Value, json};

    fn run(p: &mut crate::Planner, op: &str, args: Value) -> Value {
        p.execute(op, args, &Actor::User).unwrap().result
    }

    fn tags_of(p: &mut crate::Planner, id: &Value) -> Value {
        run(p, "get_item", json!({ "id": id }))["tags"].clone()
    }

    #[test]
    fn rename_merges_and_carries_the_colour() {
        let mut p = crate::planner::tests::planner();
        let a = run(
            &mut p,
            "create_item",
            json!({ "title": "A", "tags": ["Shops", "home"] }),
        )["id"]
            .clone();
        let b = run(
            &mut p,
            "create_item",
            json!({ "title": "B", "tags": ["shopping", "shops"] }),
        )["id"]
            .clone();
        run(
            &mut p,
            "set_tag",
            json!({ "name": "#shops", "color": "#123456" }),
        );

        let out = run(
            &mut p,
            "rename_tag",
            json!({ "from": "shops", "to": "shopping" }),
        );
        assert_eq!(out["items"], 2);
        assert_eq!(tags_of(&mut p, &a), json!(["shopping", "home"]));
        assert_eq!(tags_of(&mut p, &b), json!(["shopping"]));
        let list = run(&mut p, "list_tags", json!({}));
        assert_eq!(
            list,
            json!([
                { "name": "home", "items": 1, "color": null },
                { "name": "shopping", "items": 2, "color": "#123456" },
            ])
        );

        // One undo restores both items and the colour.
        run(&mut p, "undo", json!({}));
        assert_eq!(tags_of(&mut p, &a), json!(["shops", "home"]));
        assert_eq!(run(&mut p, "list_tags", json!({}))[2]["color"], "#123456");
    }

    #[test]
    fn delete_removes_it_everywhere_and_new_tags_list_unused() {
        let mut p = crate::planner::tests::planner();
        let a = run(
            &mut p,
            "create_item",
            json!({ "title": "A", "tags": ["errand", "home"] }),
        )["id"]
            .clone();
        run(
            &mut p,
            "set_tag",
            json!({ "name": "garden", "color": "var(--tag-health)" }),
        );
        assert_eq!(
            run(&mut p, "list_tags", json!({}))[1],
            json!({ "name": "garden", "items": 0, "color": "var(--tag-health)" })
        );

        assert_eq!(
            run(&mut p, "delete_tag", json!({ "name": "errand" }))["items"],
            1
        );
        assert_eq!(tags_of(&mut p, &a), json!(["home"]));
        assert!(
            p.execute("delete_tag", json!({ "name": "nope" }), &Actor::User)
                .is_err()
        );
        assert!(
            p.execute(
                "set_tag",
                json!({ "name": "x", "color": "red" }),
                &Actor::User
            )
            .is_err()
        );
        assert!(
            p.execute("set_tag", json!({ "name": "two words" }), &Actor::User)
                .is_err()
        );
    }
}

//! The operation layer: one registry of typed operations serving the UI
//! (Tauri commands), the AI tool loop and the MCP server.
//!
//! Each operation has a name, a description, a JSON Schema for its
//! arguments, and is either read-only or mutating. Arguments are validated
//! against the schema before the handler runs. Every mutation runs inside a
//! transaction that also writes its change-log rows under one batch id.

mod blockouts;
mod history;
mod holidays;
mod ics;
pub mod input;
mod items;
mod occurrences;
mod reads;
mod reminders;
mod rollover;
mod settings;
mod slots;
mod tags;
pub(crate) mod undo;

use std::collections::{BTreeMap, VecDeque};

use rusqlite::Connection;
use schemars::JsonSchema;
use schemars::generate::SchemaSettings;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::error::{CoreError, Result};
use crate::id::new_id;
use crate::model::{Actor, Change};
use crate::settings::PlannerSettings;
use crate::store;
use crate::time::Instant;

/// Everything a handler may touch while it runs.
pub struct Ctx<'a> {
    /// The open transaction.
    pub conn: &'a Connection,
    pub now: Instant,
    pub settings: &'a PlannerSettings,
    op: &'static str,
    actor: &'a Actor,
    batch_id: &'a str,
    /// Set while undoing or redoing: the batch being reversed or re-applied.
    reverts: Option<String>,
    changes: Vec<Change>,
    ids: IdSource,
}

/// The ids of the entities an operation creates. Fresh ones normally; when
/// a proposal is rehearsed again or accepted, the ids its first rehearsal
/// made are handed out again in the same order, so later proposals that
/// refer to them still find them.
#[derive(Debug, Default)]
struct IdSource {
    replay: VecDeque<String>,
    minted: Vec<String>,
}

impl Ctx<'_> {
    /// An id for a new entity (item, block-out, reminder, …).
    pub fn new_id(&mut self) -> String {
        let id = self.ids.replay.pop_front().unwrap_or_else(new_id);
        self.ids.minted.push(id.clone());
        id
    }

    /// Appends a change-log row for one entity. `before`/`after` are the
    /// full entity states (`None` for create/delete respectively).
    pub fn record<T: Serialize>(
        &mut self,
        entity: &str,
        entity_id: &str,
        before: Option<&T>,
        after: Option<&T>,
    ) -> Result<()> {
        let change = Change {
            id: new_id(),
            batch_id: self.batch_id.to_owned(),
            op: self.op.to_owned(),
            entity: entity.to_owned(),
            entity_id: entity_id.to_owned(),
            before: before.map(serde_json::to_value).transpose()?,
            after: after.map(serde_json::to_value).transpose()?,
            actor: self.actor.clone(),
            created_at: self.now,
            undone_at: None,
            reverts: self.reverts.clone(),
            origin: None,
        };
        store::changes::insert(self.conn, &change)?;
        self.changes.push(change);
        Ok(())
    }
}

type Handler = Box<dyn Fn(&mut Ctx<'_>, Value) -> Result<Value> + Send + Sync>;

/// One registered operation.
pub struct OpDef {
    pub name: &'static str,
    pub description: &'static str,
    pub mutates: bool,
    /// JSON Schema (draft 2020-12, subschemas inlined) for the arguments.
    pub schema: Value,
    validator: jsonschema::Validator,
    handler: Handler,
}

/// Serializable description of an operation, for tool lists and the UI.
#[derive(Debug, Clone, Serialize)]
pub struct OpInfo {
    pub name: &'static str,
    pub description: &'static str,
    pub mutates: bool,
    pub schema: Value,
}

impl OpDef {
    fn new<A, R>(
        name: &'static str,
        description: &'static str,
        mutates: bool,
        f: fn(&mut Ctx<'_>, A) -> Result<R>,
    ) -> Self
    where
        A: DeserializeOwned + JsonSchema + 'static,
        R: Serialize + 'static,
    {
        let schema = schema_for::<A>();
        // Schemas are generated from our own types, so failure is a bug.
        #[allow(clippy::expect_used)]
        let validator = jsonschema::draft202012::new(&schema)
            .expect("operation schema must be a valid JSON Schema");
        let handler: Handler = Box::new(move |ctx, args| {
            let typed: A = serde_json::from_value(args).map_err(|e| CoreError::InvalidArgs {
                op: name.to_owned(),
                errors: vec![e.to_string()],
            })?;
            Ok(serde_json::to_value(f(ctx, typed)?)?)
        });
        Self {
            name,
            description,
            mutates,
            schema,
            validator,
            handler,
        }
    }

    pub fn info(&self) -> OpInfo {
        OpInfo {
            name: self.name,
            description: self.description,
            mutates: self.mutates,
            schema: self.schema.clone(),
        }
    }

    /// Checks `args` against the schema, collecting every violation.
    pub fn validate(&self, args: &Value) -> Result<()> {
        let errors: Vec<String> = self
            .validator
            .iter_errors(args)
            .map(|e| {
                let path = e.instance_path().to_string();
                if path.is_empty() {
                    e.to_string()
                } else {
                    format!("{path}: {e}")
                }
            })
            .collect();
        if errors.is_empty() {
            Ok(())
        } else {
            Err(CoreError::InvalidArgs {
                op: self.name.to_owned(),
                errors,
            })
        }
    }
}

fn schema_for<A: JsonSchema>() -> Value {
    let generator = SchemaSettings::draft2020_12()
        .with(|s| {
            s.inline_subschemas = true;
            s.meta_schema = None;
        })
        .into_generator();
    let mut value: Value = generator.into_root_schema_for::<A>().into();
    if let Some(obj) = value.as_object_mut() {
        obj.remove("title");
    }
    value
}

/// Operations only the user (and the system) may run. They are left out of
/// the tools offered to the AI and MCP clients, and refused if an agent
/// calls them anyway: undo/redo work through the user's own history, the
/// rollover is the system's daily job, settings are the user's, snoozing
/// answers a notification on screen, and an import is a file the user
/// picked (with a snapshot first).
pub const USER_ONLY_OPS: &[&str] = &[
    "undo",
    "redo",
    "get_undo_state",
    "rollover",
    "update_settings",
    "snooze_reminder",
    "import_ics",
];

/// Whether agents (the AI and MCP clients) may run the operation `name`.
pub fn agent_may_run(name: &str) -> bool {
    !USER_ONLY_OPS.contains(&name)
}

/// All operations, keyed by name.
pub struct Registry {
    ops: BTreeMap<&'static str, OpDef>,
}

impl Registry {
    pub fn new() -> Self {
        let mut ops = BTreeMap::new();
        for def in reads::defs()
            .into_iter()
            .chain(items::defs())
            .chain(blockouts::defs())
            .chain(reminders::defs())
            .chain(slots::defs())
            .chain(rollover::defs())
            .chain(settings::defs())
            .chain(tags::defs())
            .chain(holidays::defs())
            .chain(undo::defs())
            .chain(ics::defs())
            .chain(history::defs())
        {
            let previous = ops.insert(def.name, def);
            debug_assert!(previous.is_none(), "duplicate operation name");
        }
        Self { ops }
    }

    pub fn get(&self, name: &str) -> Option<&OpDef> {
        self.ops.get(name)
    }

    pub fn iter(&self) -> impl Iterator<Item = &OpDef> {
        self.ops.values()
    }

    pub fn infos(&self) -> Vec<OpInfo> {
        self.iter().map(OpDef::info).collect()
    }

    /// The operations offered to agents as tools.
    pub fn agent_ops(&self) -> impl Iterator<Item = &OpDef> {
        self.iter().filter(|d| agent_may_run(d.name))
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

/// One call within a batch.
#[derive(Debug, Clone)]
pub struct OpCall {
    pub name: String,
    pub args: Value,
}

/// The result of executing one call or a batch.
#[derive(Debug, Clone, Serialize)]
pub struct OpOutcome {
    /// The handler's return value (for a batch: an array, one per call).
    pub result: Value,
    /// Set when at least one change was written.
    pub batch_id: Option<String>,
    pub changes: Vec<Change>,
}

/// Runs `calls` in one transaction under one batch id. Any failure rolls
/// back the whole batch.
pub(crate) fn execute(
    conn: &mut Connection,
    registry: &Registry,
    settings: &PlannerSettings,
    now: Instant,
    actor: &Actor,
    calls: &[OpCall],
) -> Result<(Vec<Value>, Vec<Change>, String)> {
    let defs = calls
        .iter()
        .map(|c| {
            let def = registry
                .get(&c.name)
                .ok_or_else(|| CoreError::UnknownOp(c.name.clone()))?;
            if matches!(actor, Actor::Ai { .. } | Actor::Mcp) && !agent_may_run(def.name) {
                return Err(CoreError::invalid(format!(
                    "'{}' is only available to the user",
                    def.name
                )));
            }
            def.validate(&c.args)?;
            Ok(def)
        })
        .collect::<Result<Vec<_>>>()?;

    let batch_id = new_id();
    let tx = conn.transaction()?;
    let mut results = Vec::with_capacity(calls.len());
    let mut changes = Vec::new();
    for (def, call) in defs.iter().zip(calls) {
        let mut ctx = Ctx {
            conn: &tx,
            now,
            settings,
            op: def.name,
            actor,
            batch_id: &batch_id,
            reverts: None,
            changes: Vec::new(),
            ids: IdSource::default(),
        };
        results.push((def.handler)(&mut ctx, call.args.clone())?);
        changes.append(&mut ctx.changes);
    }
    tx.commit()?;
    Ok((results, changes, batch_id))
}

/// One call for [`run_each`]: an operation, its arguments, and the entity
/// ids to hand out again (empty for fresh ones).
pub(crate) struct Planned<'a> {
    pub name: &'a str,
    pub args: &'a Value,
    pub ids: &'a [String],
}

/// What one [`Planned`] call did.
#[derive(Debug)]
pub(crate) struct Ran {
    pub result: Result<Value>,
    /// The entity ids it created, in order.
    pub ids: Vec<String>,
    pub changes: Vec<Change>,
}

/// Runs calls one after another under one batch id, each in its own
/// savepoint, so a call that fails leaves the others standing. With
/// `commit` false everything is rolled back at the end: a rehearsal that
/// shows what the calls would do without doing it.
pub(crate) fn run_each(
    conn: &mut Connection,
    registry: &Registry,
    settings: &PlannerSettings,
    now: Instant,
    actor: &Actor,
    calls: &[Planned<'_>],
    commit: bool,
) -> Result<(Vec<Ran>, String)> {
    let batch_id = new_id();
    let mut tx = conn.transaction()?;
    let mut out = Vec::with_capacity(calls.len());
    for call in calls {
        let def = match registry.get(call.name) {
            Some(def) => def,
            None => {
                out.push(Ran {
                    result: Err(CoreError::UnknownOp(call.name.to_owned())),
                    ids: Vec::new(),
                    changes: Vec::new(),
                });
                continue;
            }
        };
        let args = if call.args.is_null() {
            Value::Object(Default::default())
        } else {
            call.args.clone()
        };
        if let Err(err) = def.validate(&args) {
            out.push(Ran {
                result: Err(err),
                ids: Vec::new(),
                changes: Vec::new(),
            });
            continue;
        }
        let sp = tx.savepoint()?;
        let mut ctx = Ctx {
            conn: &sp,
            now,
            settings,
            op: def.name,
            actor,
            batch_id: &batch_id,
            reverts: None,
            changes: Vec::new(),
            ids: IdSource {
                replay: call.ids.iter().cloned().collect(),
                minted: Vec::new(),
            },
        };
        let result = (def.handler)(&mut ctx, args);
        let Ctx { changes, ids, .. } = ctx;
        match result {
            Ok(value) => {
                sp.commit()?;
                out.push(Ran {
                    result: Ok(value),
                    ids: ids.minted,
                    changes,
                });
            }
            // Dropping the savepoint rolls this call back.
            Err(err) => out.push(Ran {
                result: Err(err),
                ids: Vec::new(),
                changes: Vec::new(),
            }),
        }
    }
    if commit {
        tx.commit()?;
    } else {
        tx.rollback()?;
    }
    Ok((out, batch_id))
}

/// Shared helper for handlers: the item, or `NotFound`.
pub(crate) fn require_item(ctx: &Ctx<'_>, id: &str) -> Result<crate::model::Item> {
    store::items::require(ctx.conn, id)
}

//! AI proposals (roadmap §4.6, D8). In Propose mode (the default) a change
//! the AI or an MCP client asks for isn't made: it's rehearsed (run in a
//! transaction that is rolled back) and kept as a proposal operation, with
//! the before/after the rehearsal showed. The user accepts or rejects each
//! operation; accepting runs it for real.
//!
//! A rehearsal replays the proposal's earlier pending operations first,
//! so "create X, then schedule X" works, and the entity ids a rehearsal
//! made are recorded and handed out again on every later replay and on
//! accept. An id the model was told about therefore stays the id.
//!
//! One AI's operations gather in one open proposal (per chat thread, or
//! with no thread for an MCP client) until it is sealed, normally when the
//! chat turn ends; the next operation starts a new proposal.

use rusqlite::{Connection, OptionalExtension, Row, named_params};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::error::{CoreError, Result};
use crate::id::new_id;
use crate::model::{Actor, Change};
use crate::ops::{self, Planned};
use crate::planner::Planner;
use crate::settings::KEY_AI_MODE;
use crate::store;
use crate::time::{self, Instant};

/// Whether AI and MCP changes wait for the user.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    /// Changes become proposals the user accepts (default, D8).
    #[default]
    Propose,
    /// Changes apply at once, after a safety snapshot; each can be undone.
    Auto,
}

/// The mode the user left it in.
pub fn mode(conn: &Connection) -> Result<Mode> {
    Ok(store::settings::get(conn, KEY_AI_MODE)?.unwrap_or_default())
}

pub fn set_mode(conn: &Connection, mode: Mode, now: Instant) -> Result<()> {
    store::settings::set(conn, KEY_AI_MODE, &mode, now)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProposalStatus {
    Pending,
    Accepted,
    Rejected,
    /// Decided, some accepted and some not.
    Partial,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OpStatus {
    Pending,
    Accepted,
    Rejected,
    /// Accepted, but it couldn't be applied (see `error`).
    Failed,
}

fn parse<T: for<'de> Deserialize<'de>>(column: &'static str, s: String) -> Result<T> {
    serde_json::from_value(Value::String(s.clone()))
        .map_err(|_| CoreError::Corrupt { column, value: s })
}

impl<'de> Deserialize<'de> for ProposalStatus {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        match String::deserialize(d)?.as_str() {
            "pending" => Ok(Self::Pending),
            "accepted" => Ok(Self::Accepted),
            "rejected" => Ok(Self::Rejected),
            "partial" => Ok(Self::Partial),
            other => Err(serde::de::Error::custom(other)),
        }
    }
}

impl<'de> Deserialize<'de> for OpStatus {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        match String::deserialize(d)?.as_str() {
            "pending" => Ok(Self::Pending),
            "accepted" => Ok(Self::Accepted),
            "rejected" => Ok(Self::Rejected),
            "failed" => Ok(Self::Failed),
            other => Err(serde::de::Error::custom(other)),
        }
    }
}

fn as_str<T: Serialize>(v: T) -> String {
    serde_json::to_value(v)
        .ok()
        .and_then(|v| v.as_str().map(str::to_owned))
        .unwrap_or_default()
}

/// One proposed operation.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProposalOp {
    pub id: String,
    pub idx: i64,
    pub op: String,
    pub args: Value,
    /// What the rehearsal changed: `{"changes": [{entity, entity_id, before, after}]}`.
    pub preview: Value,
    pub status: OpStatus,
    pub error: Option<String>,
    #[serde(skip)]
    ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Proposal {
    pub id: String,
    pub actor: Actor,
    pub thread_id: Option<String>,
    pub status: ProposalStatus,
    pub sealed: bool,
    pub created_at: Instant,
    pub resolved_at: Option<Instant>,
    pub ops: Vec<ProposalOp>,
}

const PROPOSAL_COLUMNS: &str = "id, actor, thread_id, status, sealed, created_at, resolved_at";
const OP_COLUMNS: &str = "id, idx, op, args_json, preview_json, status, error, ids_json";

fn read_proposal(row: &Row<'_>) -> Result<Proposal> {
    Ok(Proposal {
        id: row.get("id")?,
        actor: row.get::<_, String>("actor")?.parse()?,
        thread_id: row.get("thread_id")?,
        status: parse("proposals.status", row.get("status")?)?,
        sealed: row.get("sealed")?,
        created_at: time::parse_instant("created_at", &row.get::<_, String>("created_at")?)?,
        resolved_at: store::opt_instant("resolved_at", row.get("resolved_at")?)?,
        ops: Vec::new(),
    })
}

fn read_op(row: &Row<'_>) -> Result<ProposalOp> {
    let json = |column: &'static str, v: Option<String>| -> Result<Value> {
        Ok(store::json_value(column, v)?.unwrap_or(Value::Null))
    };
    Ok(ProposalOp {
        id: row.get("id")?,
        idx: row.get("idx")?,
        op: row.get("op")?,
        args: json("proposal_ops.args_json", row.get("args_json")?)?,
        preview: json("proposal_ops.preview_json", row.get("preview_json")?)?,
        status: parse("proposal_ops.status", row.get("status")?)?,
        error: row.get("error")?,
        ids: serde_json::from_value(json("proposal_ops.ids_json", row.get("ids_json")?)?)
            .unwrap_or_default(),
    })
}

fn load_ops(conn: &Connection, proposal_id: &str) -> Result<Vec<ProposalOp>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {OP_COLUMNS} FROM proposal_ops WHERE proposal_id = ?1 ORDER BY idx"
    ))?;
    let mut rows = stmt.query([proposal_id])?;
    let mut out = Vec::new();
    while let Some(row) = rows.next()? {
        out.push(read_op(row)?);
    }
    Ok(out)
}

pub fn get(conn: &Connection, id: &str) -> Result<Proposal> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {PROPOSAL_COLUMNS} FROM proposals WHERE id = ?1"
    ))?;
    let mut p = stmt
        .query_row([id], |r| Ok(read_proposal(r)))
        .optional()?
        .transpose()?
        .ok_or_else(|| CoreError::not_found("proposal", id))?;
    p.ops = load_ops(conn, &p.id)?;
    Ok(p)
}

/// Which proposals to list.
pub enum Filter<'a> {
    /// Everything a chat thread proposed, oldest first.
    Thread(&'a str),
    /// Pending proposals with no thread (from MCP clients), oldest first.
    External,
}

pub fn list(conn: &Connection, filter: Filter<'_>) -> Result<Vec<Proposal>> {
    let (sql, param) = match filter {
        Filter::Thread(id) => ("thread_id = ?1", Some(id)),
        Filter::External => ("thread_id IS NULL AND status = 'pending'", None),
    };
    let mut stmt = conn.prepare(&format!(
        "SELECT {PROPOSAL_COLUMNS} FROM proposals WHERE {sql} ORDER BY created_at, rowid"
    ))?;
    let mut rows = match param {
        Some(p) => stmt.query([p])?,
        None => stmt.query([])?,
    };
    let mut out = Vec::new();
    while let Some(row) = rows.next()? {
        out.push(read_proposal(row)?);
    }
    for p in &mut out {
        p.ops = load_ops(conn, &p.id)?;
    }
    Ok(out)
}

/// Operations waiting for a decision, across all proposals.
pub fn pending_count(conn: &Connection) -> Result<i64> {
    Ok(conn.query_row(
        "SELECT COUNT(*) FROM proposal_ops o JOIN proposals p ON p.id = o.proposal_id \
         WHERE o.status = 'pending' AND p.status = 'pending'",
        [],
        |r| r.get(0),
    )?)
}

/// The proposal `actor` is adding to in this thread (or outside any).
fn open_proposal(
    conn: &Connection,
    actor: &Actor,
    thread_id: Option<&str>,
) -> Result<Option<String>> {
    Ok(conn
        .query_row(
            "SELECT id FROM proposals WHERE actor = ?1 AND thread_id IS ?2 \
             AND status = 'pending' AND sealed = 0 ORDER BY created_at DESC, rowid DESC LIMIT 1",
            rusqlite::params![actor.to_string(), thread_id],
            |r| r.get(0),
        )
        .optional()?)
}

/// Closes a chat turn's proposals: those `actor` made outside any thread
/// (through Blotter's MCP server) are given `thread_id`, and the thread's
/// open one is sealed, so the next turn starts a new card.
pub fn seal(conn: &Connection, actor: &Actor, thread_id: &str) -> Result<usize> {
    let claimed = conn.execute(
        "UPDATE proposals SET thread_id = ?2, sealed = 1 \
         WHERE actor = ?1 AND thread_id IS NULL AND sealed = 0",
        rusqlite::params![actor.to_string(), thread_id],
    )?;
    let sealed = conn.execute(
        "UPDATE proposals SET sealed = 1 WHERE thread_id = ?1 AND sealed = 0",
        [thread_id],
    )?;
    Ok(claimed + sealed)
}

/// What a proposed operation hands back to the model.
#[derive(Debug, Clone)]
pub struct Proposed {
    /// What the operation would return, from the rehearsal.
    pub result: Value,
    pub proposal_id: String,
}

/// The text a model gets back for a proposed change.
pub fn proposed_message(result: &Value) -> String {
    format!(
        "Proposed, not applied: the user will accept or reject this in Blotter. Ids below can \
         be used in further proposals. Rehearsal result: {result}"
    )
}

fn preview(changes: &[Change]) -> Value {
    json!({
        "changes": changes.iter().map(|c| json!({
            "entity": c.entity,
            "entity_id": c.entity_id,
            "before": c.before,
            "after": c.after,
        })).collect::<Vec<_>>()
    })
}

/// Records `name(args)` as a proposal from `actor` instead of running it.
/// Errors (bad arguments, a missing item) come back as errors and record
/// nothing, so the model can fix the call.
pub fn propose(
    planner: &mut Planner,
    actor: &Actor,
    thread_id: Option<&str>,
    name: &str,
    args: Value,
) -> Result<Proposed> {
    let def = planner
        .registry()
        .get(name)
        .ok_or_else(|| CoreError::UnknownOp(name.to_owned()))?;
    if !ops::agent_may_run(def.name) {
        return Err(CoreError::invalid(format!(
            "'{name}' is only available to the user"
        )));
    }
    if !def.mutates {
        return Err(CoreError::invalid(format!(
            "'{name}' changes nothing; just run it"
        )));
    }
    let open = open_proposal(planner.db().conn(), actor, thread_id)?;
    let prior = match &open {
        Some(id) => load_ops(planner.db().conn(), id)?
            .into_iter()
            .filter(|o| o.status == OpStatus::Pending)
            .collect(),
        None => Vec::new(),
    };
    let mut calls: Vec<Planned<'_>> = prior
        .iter()
        .map(|o| Planned {
            name: &o.op,
            args: &o.args,
            ids: &o.ids,
        })
        .collect();
    calls.push(Planned {
        name,
        args: &args,
        ids: &[],
    });
    let (mut ran, _) = planner.run_each(actor, &calls, false)?;
    let new = ran
        .pop()
        .ok_or_else(|| CoreError::invalid("nothing was rehearsed"))?;
    let result = new.result?;

    let now = planner.now();
    let conn = planner.db().conn();
    let proposal_id = match open {
        Some(id) => id,
        None => {
            let id = new_id();
            conn.execute(
                "INSERT INTO proposals (id, actor, thread_id, status, summary, created_at) \
                 VALUES (?1, ?2, ?3, 'pending', '', ?4)",
                rusqlite::params![id, actor.to_string(), thread_id, time::fmt_instant(now)],
            )?;
            id
        }
    };
    let idx: i64 = conn.query_row(
        "SELECT COALESCE(MAX(idx) + 1, 0) FROM proposal_ops WHERE proposal_id = ?1",
        [&proposal_id],
        |r| r.get(0),
    )?;
    conn.execute(
        &format!(
            "INSERT INTO proposal_ops (id, proposal_id, {}) VALUES \
             (:id, :proposal_id, :idx, :op, :args, :preview, 'pending', NULL, :ids)",
            OP_COLUMNS.trim_start_matches("id, ")
        ),
        named_params! {
            ":id": new_id(),
            ":proposal_id": proposal_id,
            ":idx": idx,
            ":op": name,
            ":args": args.to_string(),
            ":preview": preview(&new.changes).to_string(),
            ":ids": serde_json::to_string(&new.ids)?,
        },
    )?;
    Ok(Proposed {
        result,
        proposal_id,
    })
}

/// The result of accepting and rejecting.
#[derive(Debug, Clone, Serialize)]
pub struct Decision {
    pub proposal: Proposal,
    /// The change-log batch the accepted operations wrote (undo as one).
    pub batch_id: Option<String>,
}

/// Accepts and rejects operations of a proposal. Accepted ones run for
/// real, in order, as the AI that proposed them, under one batch; one
/// that can no longer apply (e.g. it needed a change that was rejected) is
/// marked failed without stopping the rest.
pub fn decide(
    planner: &mut Planner,
    proposal_id: &str,
    accept: &[String],
    reject: &[String],
) -> Result<Decision> {
    let proposal = get(planner.db().conn(), proposal_id)?;
    let to_accept: Vec<&ProposalOp> = proposal
        .ops
        .iter()
        .filter(|o| o.status == OpStatus::Pending && accept.contains(&o.id))
        .collect();
    let calls: Vec<Planned<'_>> = to_accept
        .iter()
        .map(|o| Planned {
            name: &o.op,
            args: &o.args,
            ids: &o.ids,
        })
        .collect();
    let (ran, batch_id) = if calls.is_empty() {
        (Vec::new(), String::new())
    } else {
        planner.run_each(&proposal.actor, &calls, true)?
    };

    let now = planner.now();
    let conn = planner.db().conn();
    let mut wrote = false;
    for (op, outcome) in to_accept.iter().zip(&ran) {
        let (status, error) = match &outcome.result {
            Ok(_) => {
                wrote |= !outcome.changes.is_empty();
                (OpStatus::Accepted, None)
            }
            Err(CoreError::NotFound { entity, .. }) => (
                OpStatus::Failed,
                Some(format!(
                    "Couldn't apply: the {entity} it works on doesn't exist (was a change it \
                     needed rejected?)"
                )),
            ),
            Err(err) => (OpStatus::Failed, Some(format!("Couldn't apply: {err}"))),
        };
        conn.execute(
            "UPDATE proposal_ops SET status = ?2, error = ?3 WHERE id = ?1",
            rusqlite::params![op.id, as_str(status), error],
        )?;
    }
    for op in &proposal.ops {
        if op.status == OpStatus::Pending && reject.contains(&op.id) && !accept.contains(&op.id) {
            conn.execute(
                "UPDATE proposal_ops SET status = 'rejected' WHERE id = ?1",
                [&op.id],
            )?;
        }
    }

    let ops = load_ops(conn, proposal_id)?;
    let any = |s: OpStatus| ops.iter().any(|o| o.status == s);
    let status = if any(OpStatus::Pending) {
        ProposalStatus::Pending
    } else if !any(OpStatus::Rejected) && !any(OpStatus::Failed) {
        ProposalStatus::Accepted
    } else if !any(OpStatus::Accepted) {
        ProposalStatus::Rejected
    } else {
        ProposalStatus::Partial
    };
    let resolved = (status != ProposalStatus::Pending).then(|| time::fmt_instant(now));
    conn.execute(
        "UPDATE proposals SET status = ?2, resolved_at = ?3 WHERE id = ?1",
        rusqlite::params![proposal_id, as_str(status), resolved],
    )?;
    Ok(Decision {
        proposal: get(conn, proposal_id)?,
        batch_id: wrote.then_some(batch_id),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planner::tests::planner;

    fn ai() -> Actor {
        Actor::Ai {
            provider: "claude-code".into(),
            model: "sonnet".into(),
        }
    }

    /// A real chat thread (proposals reference one).
    fn thread(p: &Planner) -> String {
        store::chat::create_thread(p.db().conn(), "t", None, crate::planner::tests::now())
            .unwrap()
            .id
    }

    fn count(p: &Planner, sql: &str) -> i64 {
        p.db().conn().query_row(sql, [], |r| r.get(0)).unwrap()
    }

    #[test]
    fn proposing_changes_nothing_and_chained_ids_survive_accept() {
        let mut p = planner();
        let t = thread(&p);
        assert_eq!(mode(p.db().conn()).unwrap(), Mode::Propose, "default");

        let made = propose(
            &mut p,
            &ai(),
            Some(t.as_str()),
            "create_item",
            json!({ "title": "Gym" }),
        )
        .unwrap();
        let id = made.result["id"].as_str().unwrap().to_owned();
        // The model can schedule the item it was told about.
        let moved = propose(
            &mut p,
            &ai(),
            Some(t.as_str()),
            "schedule_item",
            json!({ "id": id, "start": "2026-10-14T18:00" }),
        )
        .unwrap();
        assert_eq!(moved.proposal_id, made.proposal_id, "same open proposal");
        assert_eq!(
            count(&p, "SELECT COUNT(*) FROM items"),
            0,
            "nothing applied"
        );
        assert_eq!(
            count(&p, "SELECT COUNT(*) FROM changes"),
            0,
            "nothing logged"
        );
        assert_eq!(pending_count(p.db().conn()).unwrap(), 2);

        let prop = get(p.db().conn(), &made.proposal_id).unwrap();
        let after = &prop.ops[1].preview["changes"][0]["after"];
        assert!(after["start_at"].is_string(), "{after}");

        let all: Vec<String> = prop.ops.iter().map(|o| o.id.clone()).collect();
        let d = decide(&mut p, &made.proposal_id, &all, &[]).unwrap();
        assert_eq!(d.proposal.status, ProposalStatus::Accepted);
        let item = store::items::require(p.db().conn(), &id).unwrap();
        assert!(item.start_at.is_some(), "scheduled under the promised id");
        let batches = count(&p, "SELECT COUNT(DISTINCT batch_id) FROM changes");
        assert_eq!(batches, 1, "one undoable batch");
        let actor: String = p
            .db()
            .conn()
            .query_row("SELECT actor FROM changes LIMIT 1", [], |r| r.get(0))
            .unwrap();
        assert_eq!(actor, "ai:claude-code/sonnet");
        assert_eq!(pending_count(p.db().conn()).unwrap(), 0);
    }

    #[test]
    fn rejecting_a_prerequisite_fails_what_needed_it() {
        let mut p = planner();
        let made = propose(
            &mut p,
            &ai(),
            None,
            "create_item",
            json!({ "title": "Call Sam" }),
        )
        .unwrap();
        let id = made.result["id"].as_str().unwrap().to_owned();
        propose(&mut p, &ai(), None, "complete_item", json!({ "id": id })).unwrap();
        let ops = get(p.db().conn(), &made.proposal_id).unwrap().ops;

        let d = decide(
            &mut p,
            &made.proposal_id,
            &[ops[1].id.clone()],
            &[ops[0].id.clone()],
        )
        .unwrap();
        assert_eq!(d.proposal.ops[0].status, OpStatus::Rejected);
        assert_eq!(d.proposal.ops[1].status, OpStatus::Failed);
        assert!(
            d.proposal.ops[1]
                .error
                .as_deref()
                .unwrap()
                .contains("doesn't exist")
        );
        assert_eq!(d.proposal.status, ProposalStatus::Rejected);
        assert!(d.batch_id.is_none());
        assert_eq!(count(&p, "SELECT COUNT(*) FROM items"), 0);
    }

    #[test]
    fn partial_accept_and_bad_calls_record_nothing() {
        let mut p = planner();
        let a = propose(&mut p, &ai(), None, "create_item", json!({ "title": "A" })).unwrap();
        propose(&mut p, &ai(), None, "create_item", json!({ "title": "B" })).unwrap();
        assert!(propose(&mut p, &ai(), None, "create_item", json!({ "priority": 9 })).is_err());
        assert!(
            propose(&mut p, &ai(), None, "list_inbox", json!({})).is_err(),
            "reads just run"
        );
        assert!(
            propose(&mut p, &ai(), None, "undo", json!({})).is_err(),
            "user-only"
        );

        let ops = get(p.db().conn(), &a.proposal_id).unwrap().ops;
        assert_eq!(ops.len(), 2);
        let d = decide(&mut p, &a.proposal_id, &[ops[0].id.clone()], &[]).unwrap();
        assert_eq!(d.proposal.status, ProposalStatus::Pending, "B still waits");
        let d = decide(&mut p, &a.proposal_id, &[], &[ops[1].id.clone()]).unwrap();
        assert_eq!(d.proposal.status, ProposalStatus::Partial);
        assert_eq!(count(&p, "SELECT COUNT(*) FROM items"), 1);
    }

    #[test]
    fn sealing_starts_a_new_card_and_claims_mcp_proposals() {
        let mut p = planner();
        let t = thread(&p);
        let first = propose(&mut p, &ai(), None, "create_item", json!({ "title": "A" })).unwrap();
        assert_eq!(list(p.db().conn(), Filter::External).unwrap().len(), 1);

        seal(p.db().conn(), &ai(), &t).unwrap();
        assert!(list(p.db().conn(), Filter::External).unwrap().is_empty());
        let threaded = list(p.db().conn(), Filter::Thread(&t)).unwrap();
        assert_eq!(threaded[0].id, first.proposal_id);
        assert!(threaded[0].sealed);

        let second = propose(&mut p, &ai(), None, "create_item", json!({ "title": "B" })).unwrap();
        assert_ne!(
            second.proposal_id, first.proposal_id,
            "a new card after sealing"
        );
    }

    #[test]
    fn mode_is_remembered() {
        let p = planner();
        set_mode(p.db().conn(), Mode::Auto, crate::planner::tests::now()).unwrap();
        assert_eq!(mode(p.db().conn()).unwrap(), Mode::Auto);
    }
}

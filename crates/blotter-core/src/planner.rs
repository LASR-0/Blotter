//! The facade every consumer (UI, AI loop, MCP server) holds: a database,
//! the operation registry and a clock.

use chrono_tz::Tz;
use serde_json::Value;

use crate::db::Database;
use crate::error::Result;
use crate::model::Actor;
use crate::ops::{self, OpCall, OpOutcome, Registry};
use crate::settings::PlannerSettings;
use crate::time::{Clock, SystemClock};

pub struct Planner {
    db: Database,
    registry: Registry,
    clock: Box<dyn Clock>,
    default_tz: Tz,
}

impl Planner {
    /// `default_tz` is the OS timezone, used until the user picks one.
    pub fn new(db: Database, default_tz: Tz) -> Self {
        Self::with_clock(db, default_tz, Box::new(SystemClock))
    }

    pub fn with_clock(db: Database, default_tz: Tz, clock: Box<dyn Clock>) -> Self {
        Self {
            db,
            registry: Registry::new(),
            clock,
            default_tz,
        }
    }

    pub fn registry(&self) -> &Registry {
        &self.registry
    }

    pub fn db(&self) -> &Database {
        &self.db
    }

    pub fn settings(&self) -> Result<PlannerSettings> {
        PlannerSettings::load(self.db.conn(), self.default_tz)
    }

    /// Validates and runs one operation. Mutations commit atomically with
    /// their change-log rows.
    pub fn execute(&mut self, name: &str, args: Value, actor: &Actor) -> Result<OpOutcome> {
        let mut outcome = self.execute_batch(
            &[OpCall {
                name: name.to_owned(),
                args,
            }],
            actor,
        )?;
        outcome.result = match outcome.result {
            Value::Array(mut results) if results.len() == 1 => results.remove(0),
            other => other,
        };
        Ok(outcome)
    }

    /// Runs several operations as one atomic batch sharing one batch id, so
    /// they undo together. If any call fails, none take effect.
    pub fn execute_batch(&mut self, calls: &[OpCall], actor: &Actor) -> Result<OpOutcome> {
        let settings = self.settings()?;
        let now = self.clock.now();
        let calls: Vec<OpCall> = calls
            .iter()
            .map(|c| OpCall {
                name: c.name.clone(),
                // Treat a missing argument object as empty.
                args: if c.args.is_null() {
                    Value::Object(Default::default())
                } else {
                    c.args.clone()
                },
            })
            .collect();
        let (results, changes, batch_id) = ops::execute(
            self.db.conn_mut(),
            &self.registry,
            &settings,
            now,
            actor,
            &calls,
        )?;
        Ok(OpOutcome {
            result: Value::Array(results),
            batch_id: (!changes.is_empty()).then_some(batch_id),
            changes,
        })
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::error::CoreError;
    use crate::time::Instant;
    use chrono::{TimeZone, Utc};
    use serde_json::json;

    /// 2026-10-14 09:00 in Melbourne (UTC+11).
    pub fn now() -> Instant {
        Utc.with_ymd_and_hms(2026, 10, 13, 22, 0, 0).unwrap()
    }

    struct FixedClock;

    impl Clock for FixedClock {
        fn now(&self) -> Instant {
            now()
        }
    }

    pub fn planner() -> Planner {
        Planner::with_clock(
            Database::open_in_memory().unwrap(),
            chrono_tz::Australia::Melbourne,
            Box::new(FixedClock),
        )
    }

    #[test]
    fn registry_has_every_phase_one_operation() {
        let p = planner();
        let names: Vec<_> = p.registry().iter().map(|d| d.name).collect();
        for expected in [
            "list_agenda",
            "get_item",
            "search_items",
            "list_inbox",
            "list_blockouts",
            "get_day_summary",
            "create_item",
            "update_item",
            "delete_item",
            "complete_item",
            "uncomplete_item",
            "schedule_item",
            "unschedule_item",
            "create_blockout",
            "update_blockout",
            "delete_blockout",
            "add_blockout_exception",
            "set_reminder",
            "clear_reminder",
        ] {
            assert!(names.contains(&expected), "missing {expected}");
        }
    }

    #[test]
    fn every_schema_is_an_object_and_reads_are_flagged() {
        let p = planner();
        for def in p.registry().iter() {
            assert_eq!(def.schema["type"], "object", "{}", def.name);
            assert!(!def.description.is_empty());
            assert_eq!(
                def.mutates,
                !def.name.starts_with("list_")
                    && !def.name.starts_with("get_")
                    && !def.name.starts_with("search_"),
                "{}",
                def.name
            );
        }
        let create = p.registry().get("create_item").unwrap();
        assert_eq!(create.schema["required"], json!(["title"]));
        assert_eq!(create.schema["additionalProperties"], json!(false));
    }

    #[test]
    fn schema_validation_collects_all_errors() {
        let mut p = planner();
        let err = p
            .execute(
                "create_item",
                json!({ "priority": 9, "bogus": 1 }),
                &Actor::User,
            )
            .unwrap_err();
        match err {
            CoreError::InvalidArgs { op, errors } => {
                assert_eq!(op, "create_item");
                assert!(errors.len() >= 2, "{errors:?}");
                assert!(errors.iter().any(|e| e.contains("title")));
            }
            other => panic!("expected InvalidArgs, got {other:?}"),
        }
    }

    #[test]
    fn unknown_op_and_null_args() {
        let mut p = planner();
        assert!(matches!(
            p.execute("drop_tables", json!({}), &Actor::User),
            Err(CoreError::UnknownOp(_))
        ));
        assert!(p.execute("list_inbox", Value::Null, &Actor::User).is_ok());
    }

    #[test]
    fn batch_shares_one_id_and_rolls_back_on_failure() {
        let mut p = planner();
        let ok = p
            .execute_batch(
                &[
                    OpCall {
                        name: "create_item".into(),
                        args: json!({ "title": "A" }),
                    },
                    OpCall {
                        name: "create_item".into(),
                        args: json!({ "title": "B" }),
                    },
                ],
                &Actor::Mcp,
            )
            .unwrap();
        assert_eq!(ok.changes.len(), 2);
        assert!(
            ok.changes
                .iter()
                .all(|c| Some(&c.batch_id) == ok.batch_id.as_ref())
        );

        let failed = p.execute_batch(
            &[
                OpCall {
                    name: "create_item".into(),
                    args: json!({ "title": "C" }),
                },
                OpCall {
                    name: "delete_item".into(),
                    args: json!({ "id": "missing" }),
                },
            ],
            &Actor::Mcp,
        );
        assert!(failed.is_err());
        let titles: i64 = p
            .db()
            .conn()
            .query_row("SELECT COUNT(*) FROM items WHERE title = 'C'", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(titles, 0, "first call rolled back with the second");
    }

    #[test]
    fn every_mutation_writes_change_rows() {
        let mut p = planner();
        let item = p
            .execute("create_item", json!({ "title": "x" }), &Actor::User)
            .unwrap()
            .result;
        let b = p
            .execute("create_blockout",
                json!({ "label": "Work", "weekdays": ["mon"], "start_time": "09:00", "end_time": "17:00" }),
                &Actor::User)
            .unwrap()
            .result;
        let id = item["id"].clone();
        let calls = [
            ("update_item", json!({ "id": id, "title": "y" })),
            (
                "schedule_item",
                json!({ "id": id, "start": "2026-10-20T10:00" }),
            ),
            ("set_reminder", json!({ "item_id": id, "offset_min": 5 })),
            ("clear_reminder", json!({ "item_id": id })),
            ("complete_item", json!({ "id": id })),
            ("uncomplete_item", json!({ "id": id })),
            ("unschedule_item", json!({ "id": id })),
            (
                "update_blockout",
                json!({ "id": b["id"], "label": "Office" }),
            ),
            (
                "add_blockout_exception",
                json!({ "blockout_id": b["id"], "date": "2026-10-20" }),
            ),
            ("delete_blockout", json!({ "id": b["id"] })),
            ("delete_item", json!({ "id": id })),
        ];
        for (op, args) in calls {
            let out = p.execute(op, args, &Actor::User).unwrap();
            assert!(!out.changes.is_empty(), "{op} wrote no change row");
            assert!(out.changes.iter().all(|c| c.op == op));
        }
    }
}

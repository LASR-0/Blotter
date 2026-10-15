//! Blotter's MCP server (roadmap §4.8): the operation registry over stdio,
//! so external agents such as Claude Desktop or Claude Code can read and
//! change the planner through the same operations as the UI.
//!
//! It opens the app's own database (WAL lets both processes share it), and
//! every change is written as actor `mcp`, so it shows in the change log
//! and undoes like any other. The running app notices the writes and
//! refreshes. Before the first change of each session a snapshot is taken,
//! so a whole session can be rolled back from Settings → Backups.
//!
//! Operations reserved for the user (undo, settings, …) are not offered.

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use blotter_core::ops::OpDef;
use blotter_core::snapshots::{self, Reason};
use blotter_core::{Actor, CoreError, Planner};
use rmcp::model::{
    CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock, Implementation,
    ListToolsResult, PaginatedRequestParams, ServerCapabilities, ServerConfig, Tool,
    ToolAnnotations,
};
use rmcp::service::RequestContext;
use rmcp::{ErrorData, RoleServer, ServerHandler, ServiceExt};
use serde_json::Value;

/// What the server tells clients about using it.
const INSTRUCTIONS: &str = "Blotter is the user's daily planner. Times are local to the \
user's timezone (see get_settings) and written as YYYY-MM-DDTHH:MM. Unscheduled tasks live \
in the Inbox. Flexible tasks can never be placed inside block-out windows (e.g. work hours) \
or outside planning hours; fixed items (appointments) can. To place tasks, prefer \
find_free_slots or auto_slot over choosing times yourself. Look items up (list_agenda, \
search_items, list_inbox) before changing them, and use the ids they return.";

/// The MCP server over one open planner.
pub struct BlotterMcp {
    planner: Mutex<Planner>,
    db_path: PathBuf,
    /// Set once this session's safety snapshot exists.
    snapshotted: AtomicBool,
}

impl BlotterMcp {
    /// `db_path` locates the snapshots folder (beside the database unless
    /// the user moved it).
    pub fn new(planner: Planner, db_path: impl Into<PathBuf>) -> Self {
        Self {
            planner: Mutex::new(planner),
            db_path: db_path.into(),
            snapshotted: AtomicBool::new(false),
        }
    }

    /// Serves MCP on stdin/stdout until the client disconnects.
    pub async fn serve_stdio(self) -> Result<(), Box<dyn std::error::Error>> {
        let service = self.serve(rmcp::transport::stdio()).await?;
        service.waiting().await?;
        Ok(())
    }

    fn tools(&self) -> Result<Vec<Tool>, ErrorData> {
        let planner = self.lock()?;
        Ok(planner.registry().agent_ops().map(tool).collect())
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Planner>, ErrorData> {
        self.planner
            .lock()
            .map_err(|_| ErrorData::internal_error("the planner is unavailable", None))
    }

    /// Runs one tool call. Planner errors are tool results, not protocol
    /// errors, so the model reads them and can retry.
    fn call(&self, name: &str, args: Value, client: &str) -> Result<CallToolResult, ErrorData> {
        let mut planner = self.lock()?;
        let Some(def) = planner.registry().get(name) else {
            return Err(unknown_tool(name));
        };
        if !blotter_core::ops::agent_may_run(def.name) {
            return Err(unknown_tool(name));
        }
        if def.mutates
            && !self.snapshotted.load(Ordering::Acquire)
            && let Err(err) = self.snapshot(&planner, client)
        {
            return Ok(error_result(&format!(
                "Nothing was changed: Blotter couldn't take its safety snapshot first \
                     ({err})."
            )));
        }
        match planner.execute(name, args, &Actor::Mcp) {
            Ok(outcome) => Ok(CallToolResult::success(vec![ContentBlock::text(
                outcome.result.to_string(),
            )])),
            Err(err) => Ok(error_result(&describe(&err))),
        }
    }

    fn snapshot(&self, planner: &Planner, client: &str) -> blotter_core::Result<()> {
        let conn = planner.db().conn();
        let dir = snapshots::dir(conn, &self.db_path)?;
        let note = format!("Before changes from {client} (MCP)");
        snapshots::create(conn, &dir, Reason::BeforeAiBatch, Some(note), planner.now())?;
        self.snapshotted.store(true, Ordering::Release);
        Ok(())
    }
}

/// The MCP tool for one operation, with hints for clients that show them.
fn tool(def: &OpDef) -> Tool {
    let schema = match &def.schema {
        Value::Object(map) => map.clone(),
        _ => Default::default(),
    };
    let annotations = ToolAnnotations::new()
        .read_only(!def.mutates)
        .destructive(def.name.starts_with("delete_"))
        .open_world(false);
    Tool::new(def.name, def.description, schema).with_annotations(annotations)
}

fn unknown_tool(name: &str) -> ErrorData {
    ErrorData::invalid_params(format!("unknown tool '{name}'"), None)
}

fn error_result(text: &str) -> CallToolResult {
    CallToolResult::error(vec![ContentBlock::text(text.to_owned())])
}

/// An error as the model should read it, with each validation problem on
/// its own line.
fn describe(err: &CoreError) -> String {
    match err {
        CoreError::InvalidArgs { op, errors } => {
            format!("Invalid arguments for {op}:\n- {}", errors.join("\n- "))
        }
        other => other.to_string(),
    }
}

impl ServerHandler for BlotterMcp {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(
                Implementation::new("blotter", blotter_core::VERSION).with_title("Blotter"),
            )
            .with_instructions(INSTRUCTIONS)
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        Ok(ListToolsResult::with_all_items(self.tools()?))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, ErrorData> {
        let client = context
            .peer
            .peer_info()
            .map(|info| info.client_info.name.clone())
            .filter(|name| !name.trim().is_empty())
            .unwrap_or_else(|| "an MCP client".to_owned());
        let args = request.arguments.map(Value::Object).unwrap_or(Value::Null);
        self.call(&request.name, args, &client).map(Into::into)
    }
}

/// Opens the database at `db_path` (creating and migrating it if needed)
/// and serves MCP on stdio until the client disconnects.
pub fn run(db_path: &Path, default_tz: chrono_tz::Tz) -> Result<(), Box<dyn std::error::Error>> {
    let db = blotter_core::Database::open(db_path)?;
    let server = BlotterMcp::new(Planner::new(db, default_tz), db_path);
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(server.serve_stdio())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rmcp::service::RunningService;
    use rmcp::{RoleClient, ServiceError};
    use serde_json::json;

    struct Session {
        client: RunningService<RoleClient, ()>,
        dir: tempfile::TempDir,
    }

    impl Session {
        async fn start() -> Self {
            let dir = tempfile::tempdir().unwrap();
            let db_path = dir.path().join("blotter.db");
            let db = blotter_core::Database::open(&db_path).unwrap();
            let server = BlotterMcp::new(Planner::new(db, chrono_tz::UTC), &db_path);
            let (server_io, client_io) = tokio::io::duplex(64 * 1024);
            tokio::spawn(async move {
                let running = server.serve(server_io).await.unwrap();
                let _ = running.waiting().await;
            });
            let client = ().serve(client_io).await.unwrap();
            Self { client, dir }
        }

        async fn call(&self, name: &str, args: Value) -> Result<CallToolResult, ServiceError> {
            let mut params = CallToolRequestParams::new(name.to_owned());
            if let Value::Object(map) = args {
                params = params.with_arguments(map);
            }
            self.client.call_tool(params).await
        }

        /// A second connection to the same file, as the app would have.
        fn db(&self) -> blotter_core::Database {
            blotter_core::Database::open(&self.dir.path().join("blotter.db")).unwrap()
        }
    }

    fn text(result: &CallToolResult) -> String {
        result
            .content
            .iter()
            .filter_map(|c| c.as_text().map(|t| t.text.clone()))
            .collect()
    }

    fn snapshot_count(db: &blotter_core::Database) -> i64 {
        db.conn()
            .query_row("SELECT COUNT(*) FROM snapshots", [], |r| r.get(0))
            .unwrap()
    }

    #[tokio::test]
    async fn lists_agent_ops_as_tools() {
        let s = Session::start().await;
        let info = s.client.peer_info().unwrap();
        assert_eq!(info.server_info.as_ref().unwrap().name, "blotter");
        assert!(info.instructions.as_deref().unwrap().contains("block-out"));

        let tools = s.client.list_all_tools().await.unwrap();
        let names: Vec<&str> = tools.iter().map(|t| t.name.as_ref()).collect();
        for expected in ["create_item", "list_agenda", "auto_slot", "find_free_slots"] {
            assert!(names.contains(&expected), "missing {expected}");
        }
        for hidden in blotter_core::ops::USER_ONLY_OPS {
            assert!(!names.contains(hidden), "{hidden} offered");
        }
        let inbox = tools.iter().find(|t| t.name == "list_inbox").unwrap();
        assert_eq!(
            inbox.annotations.as_ref().unwrap().read_only_hint,
            Some(true)
        );
        let delete = tools.iter().find(|t| t.name == "delete_item").unwrap();
        assert_eq!(
            delete.annotations.as_ref().unwrap().destructive_hint,
            Some(true)
        );
        let create = tools.iter().find(|t| t.name == "create_item").unwrap();
        assert_eq!(create.input_schema["required"], json!(["title"]));
    }

    #[tokio::test]
    async fn changes_are_logged_as_mcp_after_one_snapshot() {
        let s = Session::start().await;
        s.call("list_inbox", json!({})).await.unwrap();
        assert_eq!(snapshot_count(&s.db()), 0, "reads take no snapshot");

        let made = s
            .call("create_item", json!({ "title": "Call Sam" }))
            .await
            .unwrap();
        assert_eq!(made.is_error, Some(false), "{}", text(&made));
        let item: Value = serde_json::from_str(&text(&made)).unwrap();
        assert_eq!(item["title"], "Call Sam");
        s.call("create_item", json!({ "title": "Buy milk" }))
            .await
            .unwrap();

        let db = s.db();
        assert_eq!(snapshot_count(&db), 1, "one snapshot per session");
        let note: String = db
            .conn()
            .query_row("SELECT note FROM snapshots", [], |r| r.get(0))
            .unwrap();
        assert!(note.contains("(MCP)"), "{note}");
        let actors: Vec<String> = db
            .conn()
            .prepare("SELECT DISTINCT actor FROM changes")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(actors, ["mcp"]);

        let inbox = s.call("list_inbox", json!({})).await.unwrap();
        assert!(text(&inbox).contains("Buy milk"));
    }

    #[tokio::test]
    async fn invalid_arguments_come_back_for_a_retry() {
        let s = Session::start().await;
        let bad = s
            .call("create_item", json!({ "priority": 9, "bogus": true }))
            .await
            .unwrap();
        assert_eq!(bad.is_error, Some(true));
        let msg = text(&bad);
        assert!(
            msg.starts_with("Invalid arguments for create_item"),
            "{msg}"
        );
        assert!(msg.lines().count() >= 3, "one line per problem: {msg}");

        let missing = s.call("get_item", json!({ "id": "nope" })).await.unwrap();
        assert_eq!(missing.is_error, Some(true), "{}", text(&missing));
    }

    #[tokio::test]
    async fn user_only_and_unknown_ops_are_refused() {
        let s = Session::start().await;
        for name in ["undo", "update_settings", "drop_tables"] {
            let err = s.call(name, json!({})).await;
            assert!(err.is_err(), "{name} should be refused");
        }
        assert_eq!(snapshot_count(&s.db()), 0);
    }
}

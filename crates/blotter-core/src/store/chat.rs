//! AI chat threads and their messages (roadmap §4.6: history persists per
//! thread). A message keeps its display text in `content` and, for the AI's
//! own turns, the full provider-neutral blocks (tool calls and results) as
//! JSON, so an API-provider thread can be replayed to the model.

use rusqlite::{Connection, OptionalExtension, Row, named_params};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{CoreError, Result};
use crate::id::new_id;
use crate::time::{self, Instant};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Thread {
    pub id: String,
    pub title: String,
    pub provider_id: Option<String>,
    /// The Claude Code session to resume, for threads run through it.
    pub external_session: Option<String>,
    pub created_at: Instant,
    pub updated_at: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChatRole {
    User,
    Assistant,
    /// Tool results sent back to the model.
    Tool,
}

impl ChatRole {
    fn as_str(self) -> &'static str {
        match self {
            ChatRole::User => "user",
            ChatRole::Assistant => "assistant",
            ChatRole::Tool => "tool",
        }
    }

    fn parse(s: &str) -> Result<Self> {
        match s {
            "user" => Ok(ChatRole::User),
            "assistant" => Ok(ChatRole::Assistant),
            "tool" => Ok(ChatRole::Tool),
            other => Err(CoreError::Corrupt {
                column: "chat_messages.role",
                value: other.to_owned(),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ChatMessage {
    pub id: String,
    pub thread_id: String,
    pub role: ChatRole,
    /// What the chat shows.
    pub content: String,
    /// The full blocks, when there is more than text (tool calls/results).
    pub blocks: Option<Value>,
    /// The change-log batch this turn wrote, for "Undo batch".
    pub batch_id: Option<String>,
    pub created_at: Instant,
}

const THREAD_COLUMNS: &str = "id, title, provider_id, external_session, created_at, updated_at";

fn read_thread(row: &Row<'_>) -> Result<Thread> {
    Ok(Thread {
        id: row.get("id")?,
        title: row.get("title")?,
        provider_id: row.get("provider_id")?,
        external_session: row.get("external_session")?,
        created_at: time::parse_instant("created_at", &row.get::<_, String>("created_at")?)?,
        updated_at: time::parse_instant("updated_at", &row.get::<_, String>("updated_at")?)?,
    })
}

/// Starts a thread titled from its first message.
pub fn create_thread(
    conn: &Connection,
    title: &str,
    provider_id: Option<&str>,
    now: Instant,
) -> Result<Thread> {
    let t = Thread {
        id: new_id(),
        title: title_from(title),
        provider_id: provider_id.map(str::to_owned),
        external_session: None,
        created_at: time::truncate(now),
        updated_at: time::truncate(now),
    };
    conn.execute(
        &format!(
            "INSERT INTO chat_threads ({THREAD_COLUMNS}) VALUES \
             (:id, :title, :provider_id, NULL, :created_at, :updated_at)"
        ),
        named_params! {
            ":id": t.id,
            ":title": t.title,
            ":provider_id": t.provider_id,
            ":created_at": time::fmt_instant(t.created_at),
            ":updated_at": time::fmt_instant(t.updated_at),
        },
    )?;
    Ok(t)
}

/// A short title: the first line, cut at a word near 60 characters.
fn title_from(text: &str) -> String {
    let line = text.lines().next().unwrap_or_default().trim();
    if line.chars().count() <= 60 {
        return line.to_owned();
    }
    let cut: String = line.chars().take(60).collect();
    let cut = cut.rsplit_once(' ').map_or(cut.as_str(), |(head, _)| head);
    format!("{}…", cut.trim_end())
}

pub fn get_thread(conn: &Connection, id: &str) -> Result<Option<Thread>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {THREAD_COLUMNS} FROM chat_threads WHERE id = ?1"
    ))?;
    stmt.query_row([id], |r| Ok(read_thread(r)))
        .optional()?
        .transpose()
}

pub fn require_thread(conn: &Connection, id: &str) -> Result<Thread> {
    get_thread(conn, id)?.ok_or_else(|| CoreError::NotFound {
        entity: "chat thread",
        id: id.to_owned(),
    })
}

/// Threads, most recently used first.
pub fn list_threads(conn: &Connection) -> Result<Vec<Thread>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {THREAD_COLUMNS} FROM chat_threads ORDER BY updated_at DESC, rowid DESC"
    ))?;
    let mut rows = stmt.query([])?;
    let mut out = Vec::new();
    while let Some(row) = rows.next()? {
        out.push(read_thread(row)?);
    }
    Ok(out)
}

/// Records on the thread's latest reply the change-log batches its turn
/// wrote (comma-separated in `batch_id`), for its "Undo batch".
pub fn set_turn_batches(conn: &Connection, thread_id: &str, batches: &[String]) -> Result<()> {
    if batches.is_empty() {
        return Ok(());
    }
    conn.execute(
        "UPDATE chat_messages SET batch_id = ?2 WHERE id = (SELECT id FROM chat_messages \n         WHERE thread_id = ?1 AND role = 'assistant' ORDER BY rowid DESC LIMIT 1)",
        rusqlite::params![thread_id, batches.join(",")],
    )?;
    Ok(())
}

pub fn set_external_session(conn: &Connection, id: &str, session: Option<&str>) -> Result<()> {
    conn.execute(
        "UPDATE chat_threads SET external_session = ?2 WHERE id = ?1",
        rusqlite::params![id, session],
    )?;
    Ok(())
}

pub fn delete_thread(conn: &Connection, id: &str) -> Result<bool> {
    Ok(conn.execute("DELETE FROM chat_threads WHERE id = ?1", [id])? > 0)
}

const MESSAGE_COLUMNS: &str = "id, thread_id, role, content, tool_calls_json, batch_id, created_at";

fn read_message(row: &Row<'_>) -> Result<ChatMessage> {
    Ok(ChatMessage {
        id: row.get("id")?,
        thread_id: row.get("thread_id")?,
        role: ChatRole::parse(&row.get::<_, String>("role")?)?,
        content: row.get("content")?,
        blocks: super::json_value("chat_messages.tool_calls_json", row.get("tool_calls_json")?)?,
        batch_id: row.get("batch_id")?,
        created_at: time::parse_instant("created_at", &row.get::<_, String>("created_at")?)?,
    })
}

/// Appends a message and marks the thread as just used.
pub fn append(
    conn: &Connection,
    thread_id: &str,
    role: ChatRole,
    content: &str,
    blocks: Option<&Value>,
    batch_id: Option<&str>,
    now: Instant,
) -> Result<ChatMessage> {
    let m = ChatMessage {
        id: new_id(),
        thread_id: thread_id.to_owned(),
        role,
        content: content.to_owned(),
        blocks: blocks.cloned(),
        batch_id: batch_id.map(str::to_owned),
        created_at: time::truncate(now),
    };
    conn.execute(
        &format!(
            "INSERT INTO chat_messages ({MESSAGE_COLUMNS}) VALUES \
             (:id, :thread_id, :role, :content, :blocks, :batch_id, :created_at)"
        ),
        named_params! {
            ":id": m.id,
            ":thread_id": m.thread_id,
            ":role": m.role.as_str(),
            ":content": m.content,
            ":blocks": m.blocks.as_ref().map(Value::to_string),
            ":batch_id": m.batch_id,
            ":created_at": time::fmt_instant(m.created_at),
        },
    )?;
    conn.execute(
        "UPDATE chat_threads SET updated_at = ?2 WHERE id = ?1",
        [thread_id, &time::fmt_instant(m.created_at)],
    )?;
    Ok(m)
}

/// A thread's messages in the order they were written.
pub fn messages(conn: &Connection, thread_id: &str) -> Result<Vec<ChatMessage>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {MESSAGE_COLUMNS} FROM chat_messages WHERE thread_id = ?1 ORDER BY rowid"
    ))?;
    let mut rows = stmt.query([thread_id])?;
    let mut out = Vec::new();
    while let Some(row) = rows.next()? {
        out.push(read_message(row)?);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::planner::tests::now;
    use chrono::Duration;
    use serde_json::json;

    #[test]
    fn threads_keep_messages_in_order_and_sort_by_use() {
        let db = Database::open_in_memory().unwrap();
        let c = db.conn();
        let a = create_thread(c, "Plan my week", None, now()).unwrap();
        let b = create_thread(c, "Second", None, now() + Duration::minutes(1)).unwrap();
        assert_eq!(list_threads(c).unwrap()[0].id, b.id);

        append(
            c,
            &a.id,
            ChatRole::User,
            "Plan my week",
            None,
            None,
            now() + Duration::minutes(2),
        )
        .unwrap();
        let blocks = json!([{ "type": "tool_use", "id": "t1", "name": "list_inbox", "input": {} }]);
        append(
            c,
            &a.id,
            ChatRole::Assistant,
            "",
            Some(&blocks),
            Some("B1"),
            now() + Duration::minutes(2),
        )
        .unwrap();
        append(
            c,
            &a.id,
            ChatRole::Tool,
            "[]",
            None,
            None,
            now() + Duration::minutes(2),
        )
        .unwrap();

        assert_eq!(list_threads(c).unwrap()[0].id, a.id, "used most recently");
        let msgs = messages(c, &a.id).unwrap();
        let roles: Vec<_> = msgs.iter().map(|m| m.role).collect();
        assert_eq!(roles, [ChatRole::User, ChatRole::Assistant, ChatRole::Tool]);
        assert_eq!(msgs[1].blocks.as_ref().unwrap()[0]["name"], "list_inbox");
        assert_eq!(msgs[1].batch_id.as_deref(), Some("B1"));

        set_external_session(c, &a.id, Some("sess-1")).unwrap();
        assert_eq!(
            require_thread(c, &a.id)
                .unwrap()
                .external_session
                .as_deref(),
            Some("sess-1")
        );
        assert!(delete_thread(c, &a.id).unwrap());
        assert!(
            messages(c, &a.id).unwrap().is_empty(),
            "messages go with it"
        );
    }

    #[test]
    fn a_turns_batches_go_on_its_latest_reply() {
        let db = Database::open_in_memory().unwrap();
        let c = db.conn();
        let t = create_thread(c, "x", None, now()).unwrap();
        append(c, &t.id, ChatRole::User, "hi", None, None, now()).unwrap();
        append(c, &t.id, ChatRole::Assistant, "one", None, None, now()).unwrap();
        append(c, &t.id, ChatRole::Assistant, "two", None, None, now()).unwrap();
        set_turn_batches(c, &t.id, &["B1".into(), "B2".into()]).unwrap();
        let msgs = messages(c, &t.id).unwrap();
        assert_eq!(msgs[1].batch_id, None);
        assert_eq!(msgs[2].batch_id.as_deref(), Some("B1,B2"));
    }

    #[test]
    fn titles_are_short_first_lines() {
        assert_eq!(
            title_from("  Book the dentist\nand more"),
            "Book the dentist"
        );
        let long = "Find time this week for the dentist, the plumber and a long walk with Sam";
        let t = title_from(long);
        assert!(t.ends_with('…') && t.chars().count() <= 61, "{t}");
        assert!(!t.contains("Sam"));
    }
}

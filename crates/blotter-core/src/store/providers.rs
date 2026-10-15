//! AI provider settings (roadmap §4.6). API keys are never stored here:
//! they live in the OS keychain, keyed by the provider's id. Only the last
//! few characters are kept, in `options`, so the settings page can show
//! which key is in use.

use rusqlite::{Connection, OptionalExtension, Row, named_params};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{CoreError, Result};
use crate::time::{self, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderKind {
    Anthropic,
    Openai,
    /// Any OpenAI-compatible endpoint: Ollama, LM Studio, OpenRouter, …
    OpenaiCompatible,
}

impl ProviderKind {
    fn as_str(self) -> &'static str {
        match self {
            ProviderKind::Anthropic => "anthropic",
            ProviderKind::Openai => "openai",
            ProviderKind::OpenaiCompatible => "openai_compatible",
        }
    }

    fn parse(s: &str) -> Result<Self> {
        match s {
            "anthropic" => Ok(ProviderKind::Anthropic),
            "openai" => Ok(ProviderKind::Openai),
            "openai_compatible" => Ok(ProviderKind::OpenaiCompatible),
            other => Err(CoreError::Corrupt {
                column: "providers.kind",
                value: other.to_owned(),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Provider {
    pub id: String,
    pub name: String,
    pub kind: ProviderKind,
    /// Required for OpenAI-compatible endpoints; optional overrides otherwise.
    pub base_url: Option<String>,
    pub model: String,
    /// Non-secret extras, e.g. `{"key_hint": "3f9a"}`.
    pub options: Value,
    pub is_default: bool,
    pub created_at: Instant,
    pub updated_at: Instant,
}

const COLUMNS: &str =
    "id, name, kind, base_url, model, options_json, is_default, created_at, updated_at";

fn read(row: &Row<'_>) -> Result<Provider> {
    let options: String = row.get("options_json")?;
    Ok(Provider {
        id: row.get("id")?,
        name: row.get("name")?,
        kind: ProviderKind::parse(&row.get::<_, String>("kind")?)?,
        base_url: row.get("base_url")?,
        model: row.get("model")?,
        options: serde_json::from_str(&options).map_err(|_| CoreError::Corrupt {
            column: "providers.options_json",
            value: options.clone(),
        })?,
        is_default: row.get("is_default")?,
        created_at: time::parse_instant("created_at", &row.get::<_, String>("created_at")?)?,
        updated_at: time::parse_instant("updated_at", &row.get::<_, String>("updated_at")?)?,
    })
}

pub fn list(conn: &Connection) -> Result<Vec<Provider>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {COLUMNS} FROM providers ORDER BY created_at, id"
    ))?;
    let mut rows = stmt.query([])?;
    let mut out = Vec::new();
    while let Some(row) = rows.next()? {
        out.push(read(row)?);
    }
    Ok(out)
}

pub fn get(conn: &Connection, id: &str) -> Result<Option<Provider>> {
    conn.query_row(
        &format!("SELECT {COLUMNS} FROM providers WHERE id = ?1"),
        [id],
        |r| Ok(read(r)),
    )
    .optional()?
    .transpose()
}

pub fn require(conn: &Connection, id: &str) -> Result<Provider> {
    get(conn, id)?.ok_or_else(|| CoreError::not_found("provider", id))
}

/// The provider the chat uses: the default one, else the first.
pub fn default_provider(conn: &Connection) -> Result<Option<Provider>> {
    let all = list(conn)?;
    Ok(all
        .iter()
        .find(|p| p.is_default)
        .or_else(|| all.first())
        .cloned())
}

pub fn upsert(conn: &Connection, p: &Provider) -> Result<()> {
    // The model may be left empty until one is picked from the provider's list.
    if p.name.trim().is_empty() {
        return Err(CoreError::invalid("a provider needs a name"));
    }
    if p.kind == ProviderKind::OpenaiCompatible
        && p.base_url.as_deref().is_none_or(|u| u.trim().is_empty())
    {
        return Err(CoreError::invalid(
            "an OpenAI-compatible provider needs a base URL",
        ));
    }
    conn.execute(
        &format!(
            "INSERT INTO providers ({COLUMNS}) VALUES (:id, :name, :kind, :base_url, :model, \
             :options, :is_default, :created_at, :updated_at) \
             ON CONFLICT (id) DO UPDATE SET name = excluded.name, kind = excluded.kind, \
             base_url = excluded.base_url, model = excluded.model, \
             options_json = excluded.options_json, is_default = excluded.is_default, \
             updated_at = excluded.updated_at"
        ),
        named_params! {
            ":id": p.id,
            ":name": p.name.trim(),
            ":kind": p.kind.as_str(),
            ":base_url": p.base_url.as_deref().map(str::trim).filter(|u| !u.is_empty()),
            ":model": p.model.trim(),
            ":options": serde_json::to_string(&p.options)?,
            ":is_default": p.is_default,
            ":created_at": time::fmt_instant(p.created_at),
            ":updated_at": time::fmt_instant(p.updated_at),
        },
    )?;
    if p.is_default {
        set_default(conn, &p.id)?;
    }
    Ok(())
}

/// Makes `id` the only default.
pub fn set_default(conn: &Connection, id: &str) -> Result<()> {
    require(conn, id)?;
    conn.execute("UPDATE providers SET is_default = (id = ?1)", [id])?;
    Ok(())
}

pub fn delete(conn: &Connection, id: &str) -> Result<bool> {
    Ok(conn.execute("DELETE FROM providers WHERE id = ?1", [id])? > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use chrono::{TimeZone, Utc};
    use serde_json::json;

    fn provider(id: &str, kind: ProviderKind) -> Provider {
        let now = Utc.with_ymd_and_hms(2026, 10, 15, 6, 0, 0).unwrap();
        Provider {
            id: id.into(),
            name: format!("P {id}"),
            kind,
            base_url: None,
            model: "m".into(),
            options: json!({ "key_hint": "3f9a" }),
            is_default: false,
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn crud_and_single_default() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.conn();
        upsert(conn, &provider("a", ProviderKind::Anthropic)).unwrap();
        upsert(
            conn,
            &Provider {
                is_default: true,
                ..provider("b", ProviderKind::Openai)
            },
        )
        .unwrap();
        assert_eq!(default_provider(conn).unwrap().unwrap().id, "b");
        set_default(conn, "a").unwrap();
        let all = list(conn).unwrap();
        assert_eq!(all.iter().filter(|p| p.is_default).count(), 1);
        assert_eq!(default_provider(conn).unwrap().unwrap().id, "a");
        assert_eq!(get(conn, "a").unwrap().unwrap().options["key_hint"], "3f9a");
        assert!(delete(conn, "a").unwrap());
        assert_eq!(
            default_provider(conn).unwrap().unwrap().id,
            "b",
            "falls back to the first"
        );
    }

    #[test]
    fn compatible_needs_a_base_url() {
        let db = Database::open_in_memory().unwrap();
        let bad = provider("c", ProviderKind::OpenaiCompatible);
        assert!(upsert(db.conn(), &bad).is_err());
        let good = Provider {
            base_url: Some("http://localhost:11434/v1".into()),
            ..bad
        };
        upsert(db.conn(), &good).unwrap();
    }
}

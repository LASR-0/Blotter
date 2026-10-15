//! Settings → AI providers commands. Keys go straight to the keychain;
//! the UI only ever sees whether one is saved and its last four characters.

use std::time::Instant;

use blotter_core::id::new_id;
use blotter_core::store::providers::{self, Provider, ProviderKind};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::State;

use super::{ANTHROPIC_DEFAULT_MODEL, ChatRequest, Message, ModelInfo, keychain};
use crate::AppState;
use crate::commands::CommandError;

fn lock_err() -> CommandError {
    CommandError::internal("planner state is unavailable after an earlier crash")
}

/// A provider as the settings page sees it.
#[derive(Serialize)]
pub struct ProviderView {
    #[serde(flatten)]
    pub provider: Provider,
    pub has_key: bool,
    pub key_hint: Option<String>,
    /// API providers haven't been checked against the real services (D29).
    pub experimental: bool,
}

fn view(p: Provider) -> Result<ProviderView, CommandError> {
    let has_key = keychain::get(&p.id)?.is_some();
    Ok(ProviderView {
        key_hint: p.options["key_hint"]
            .as_str()
            .map(str::to_owned)
            .filter(|_| has_key),
        provider: p,
        has_key,
        experimental: true,
    })
}

#[tauri::command]
pub fn list_providers(state: State<'_, AppState>) -> Result<Vec<ProviderView>, CommandError> {
    let all = {
        let planner = state.planner.lock().map_err(|_| lock_err())?;
        providers::list(planner.db().conn())?
    };
    all.into_iter().map(view).collect()
}

#[derive(Deserialize)]
pub struct ProviderInput {
    pub id: Option<String>,
    pub name: String,
    pub kind: ProviderKind,
    pub base_url: Option<String>,
    pub model: Option<String>,
    /// `Some("")` removes the saved key; `None` leaves it as it is.
    pub api_key: Option<String>,
    #[serde(default)]
    pub make_default: bool,
}

#[tauri::command]
pub fn save_provider(
    state: State<'_, AppState>,
    input: ProviderInput,
) -> Result<ProviderView, CommandError> {
    let now = Utc::now();
    let planner = state.planner.lock().map_err(|_| lock_err())?;
    let conn = planner.db().conn();
    let existing = match &input.id {
        Some(id) => Some(providers::require(conn, id)?),
        None => None,
    };
    let first = providers::list(conn)?.is_empty();
    let mut p = existing.unwrap_or_else(|| Provider {
        id: new_id(),
        name: String::new(),
        kind: input.kind,
        base_url: None,
        model: String::new(),
        options: json!({}),
        is_default: false,
        created_at: now,
        updated_at: now,
    });
    p.name = input.name;
    p.kind = input.kind;
    p.base_url = input.base_url;
    p.model = input
        .model
        .filter(|m| !m.trim().is_empty())
        .unwrap_or_else(|| match input.kind {
            ProviderKind::Anthropic => ANTHROPIC_DEFAULT_MODEL.to_owned(),
            _ => p.model.clone(),
        });
    p.is_default = p.is_default || input.make_default || first;
    p.updated_at = now;
    match input.api_key.as_deref().map(str::trim) {
        Some("") => {
            keychain::delete(&p.id)?;
            p.options["key_hint"] = serde_json::Value::Null;
        }
        Some(key) => {
            keychain::set(&p.id, key)?;
            p.options["key_hint"] = json!(keychain::hint(key));
        }
        None => {}
    }
    providers::upsert(conn, &p)?;
    view(providers::require(conn, &p.id)?)
}

#[tauri::command]
pub fn delete_provider(state: State<'_, AppState>, id: String) -> Result<(), CommandError> {
    let planner = state.planner.lock().map_err(|_| lock_err())?;
    providers::delete(planner.db().conn(), &id)?;
    keychain::delete(&id)
}

#[tauri::command]
pub fn set_default_provider(state: State<'_, AppState>, id: String) -> Result<(), CommandError> {
    let planner = state.planner.lock().map_err(|_| lock_err())?;
    Ok(providers::set_default(planner.db().conn(), &id)?)
}

fn load(state: &State<'_, AppState>, id: &str) -> Result<(Provider, Option<String>), CommandError> {
    let p = {
        let planner = state.planner.lock().map_err(|_| lock_err())?;
        providers::require(planner.db().conn(), id)?
    };
    let key = keychain::get(&p.id)?;
    Ok((p, key))
}

pub(crate) fn ai_err(e: super::AiError) -> CommandError {
    CommandError {
        code: "ai",
        message: e.to_string(),
        details: Vec::new(),
    }
}

#[tauri::command]
pub async fn fetch_models(
    state: State<'_, AppState>,
    id: String,
) -> Result<Vec<ModelInfo>, CommandError> {
    let (p, key) = load(&state, &id)?;
    super::list_models(&p, key.as_deref()).await.map_err(ai_err)
}

#[derive(Serialize)]
pub struct TestResult {
    pub ms: u128,
    pub reply: String,
}

/// Sends a tiny request and times it.
#[tauri::command]
pub async fn test_provider(
    state: State<'_, AppState>,
    id: String,
) -> Result<TestResult, CommandError> {
    let (p, key) = load(&state, &id)?;
    let req = ChatRequest {
        model: p.model.clone(),
        system: String::new(),
        messages: vec![Message::user_text("Reply with the single word OK.")],
        tools: Vec::new(),
        max_tokens: 1024,
    };
    let start = Instant::now();
    let res = super::chat(&p, key.as_deref(), &req, |_| {})
        .await
        .map_err(ai_err)?;
    let reply = res
        .content
        .iter()
        .filter_map(|b| match b {
            super::Block::Text { text } => Some(text.as_str()),
            _ => None,
        })
        .collect::<String>();
    Ok(TestResult {
        ms: start.elapsed().as_millis(),
        reply: reply.chars().take(80).collect(),
    })
}

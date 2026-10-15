//! AI providers (roadmap §4.6, D29). One provider-neutral conversation
//! shape, adapted to the Anthropic Messages API and to OpenAI-style chat
//! completions (OpenAI itself and compatible servers such as Ollama).
//! All HTTP happens here in Rust; keys come from the OS keychain and never
//! reach the webview, the database or the logs.
//!
//! The API-key providers are built and unit-tested against recorded
//! responses but are **experimental**: nobody has run them against the
//! real services yet (D29).

pub mod anthropic;
pub mod commands;
pub mod keychain;
pub mod openai;
pub mod sse;

use std::time::Duration;

use blotter_core::store::providers::{Provider, ProviderKind};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Generous cap: a turn can stream for a while before it finishes.
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(10 * 60);
pub const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
/// Default Anthropic model (D27).
pub const ANTHROPIC_DEFAULT_MODEL: &str = "claude-sonnet-5-5";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    User,
    Assistant,
}

/// One piece of a message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Block {
    Text {
        text: String,
    },
    ToolUse {
        id: String,
        name: String,
        input: Value,
        /// Set when the model's arguments weren't valid JSON; the tool loop
        /// sends this back as an error result instead of running the tool.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        invalid: Option<String>,
    },
    ToolResult {
        tool_use_id: String,
        content: String,
        #[serde(default)]
        is_error: bool,
    },
    /// A provider block kept verbatim to send back unchanged in the same
    /// conversation (Anthropic thinking blocks); other providers drop it.
    Raw {
        provider: ProviderKind,
        value: Value,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: Vec<Block>,
}

impl Message {
    pub fn user_text(text: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: vec![Block::Text { text: text.into() }],
        }
    }
}

/// A tool offered to the model: an operation from the registry.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ToolSpec {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

#[derive(Debug, Clone)]
pub struct ChatRequest {
    pub model: String,
    pub system: String,
    pub messages: Vec<Message>,
    pub tools: Vec<ToolSpec>,
    pub max_tokens: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Stop {
    EndTurn,
    ToolUse,
    /// Cut off: any tool call in this turn may be truncated; don't run it.
    MaxTokens,
    /// Declined by the provider's safety systems; don't run tools.
    Refusal,
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ChatResponse {
    pub content: Vec<Block>,
    pub stop: Stop,
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
}

/// What streams to the UI while a reply is generated.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Delta {
    Text { text: String },
    ToolStart { name: String },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ModelInfo {
    pub id: String,
    pub display_name: Option<String>,
}

/// Failures, classified so the UI and tool loop can react (retry or not).
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AiError {
    /// No key stored, or the provider rejected it (401/403).
    Auth { message: String },
    /// 429 / overloaded: worth retrying later.
    RateLimited { message: String },
    /// The request itself was rejected (400/404/422).
    Request { message: String },
    /// 5xx, network, timeouts.
    Unavailable { message: String },
    /// The response didn't parse as the provider's format.
    Protocol { message: String },
}

impl std::fmt::Display for AiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (what, m) = match self {
            AiError::Auth { message } => ("couldn't sign in", message),
            AiError::RateLimited { message } => ("busy or rate-limited", message),
            AiError::Request { message } => ("rejected the request", message),
            AiError::Unavailable { message } => ("couldn't be reached", message),
            AiError::Protocol { message } => ("sent something unexpected", message),
        };
        write!(f, "The AI provider {what}: {m}")
    }
}

impl AiError {
    /// Classifies an HTTP error status and its body's message.
    pub fn from_status(status: u16, message: String) -> Self {
        match status {
            401 | 403 => AiError::Auth { message },
            408 | 429 | 529 => AiError::RateLimited { message },
            400..=499 => AiError::Request { message },
            _ => AiError::Unavailable { message },
        }
    }

    pub fn from_reqwest(e: reqwest::Error) -> Self {
        // Never include the URL: it can carry query parameters, not keys,
        // but keep messages short and key-free regardless.
        let message = if e.is_timeout() {
            "the request timed out".to_owned()
        } else if e.is_connect() {
            "couldn't connect (check the address, your network or proxy)".to_owned()
        } else {
            e.without_url().to_string()
        };
        AiError::Unavailable { message }
    }
}

/// An HTTP client with sensible timeouts. TLS uses the OS certificate
/// store (rustls platform verifier), so corporate proxies' certificates
/// are trusted like in the browser.
pub fn http_client() -> Result<reqwest::Client, AiError> {
    reqwest::Client::builder()
        .connect_timeout(CONNECT_TIMEOUT)
        .timeout(REQUEST_TIMEOUT)
        .user_agent(concat!("Blotter/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| AiError::Unavailable {
            message: e.to_string(),
        })
}

/// Runs one streamed turn on `provider`, calling `on_delta` as text and
/// tool calls arrive, and returns the complete reply.
pub async fn chat(
    provider: &Provider,
    key: Option<&str>,
    req: &ChatRequest,
    on_delta: impl FnMut(Delta),
) -> Result<ChatResponse, AiError> {
    if req.model.trim().is_empty() {
        return Err(AiError::Request {
            message: "choose a model for this provider first".to_owned(),
        });
    }
    match provider.kind {
        ProviderKind::Anthropic => {
            let key = key.ok_or_else(missing_key)?;
            anthropic::chat(provider, key, req, on_delta).await
        }
        ProviderKind::Openai => {
            let key = key.ok_or_else(missing_key)?;
            openai::chat(provider, Some(key), req, on_delta).await
        }
        ProviderKind::OpenaiCompatible => openai::chat(provider, key, req, on_delta).await,
    }
}

/// Lists the models the provider offers to this key.
pub async fn list_models(
    provider: &Provider,
    key: Option<&str>,
) -> Result<Vec<ModelInfo>, AiError> {
    match provider.kind {
        ProviderKind::Anthropic => {
            anthropic::list_models(provider, key.ok_or_else(missing_key)?).await
        }
        ProviderKind::Openai => {
            openai::list_models(provider, Some(key.ok_or_else(missing_key)?)).await
        }
        ProviderKind::OpenaiCompatible => openai::list_models(provider, key).await,
    }
}

fn missing_key() -> AiError {
    AiError::Auth {
        message: "no API key is saved for this provider".to_owned(),
    }
}

/// Parses a tool call's accumulated arguments strictly: the model's JSON
/// can be cut off or malformed, and that must never run a tool.
pub fn parse_tool_input(raw: &str) -> (Value, Option<String>) {
    let raw = raw.trim();
    if raw.is_empty() {
        return (Value::Object(Default::default()), None);
    }
    match serde_json::from_str::<Value>(raw) {
        Ok(v @ Value::Object(_)) => (v, None),
        Ok(_) => (
            Value::Null,
            Some("arguments must be a JSON object".to_owned()),
        ),
        Err(e) => (
            Value::Null,
            Some(format!("arguments weren't valid JSON ({e})")),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_input_is_parsed_strictly() {
        assert_eq!(parse_tool_input("").0, serde_json::json!({}));
        assert_eq!(parse_tool_input(r#"{"title":"Gym"}"#).0["title"], "Gym");
        assert!(parse_tool_input(r#"{"title":"Gy"#).1.is_some(), "truncated");
        assert!(parse_tool_input("[1]").1.is_some(), "not an object");
    }

    #[test]
    fn errors_classify_by_status() {
        assert!(matches!(
            AiError::from_status(401, String::new()),
            AiError::Auth { .. }
        ));
        assert!(matches!(
            AiError::from_status(529, String::new()),
            AiError::RateLimited { .. }
        ));
        assert!(matches!(
            AiError::from_status(400, String::new()),
            AiError::Request { .. }
        ));
        assert!(matches!(
            AiError::from_status(502, String::new()),
            AiError::Unavailable { .. }
        ));
    }
}

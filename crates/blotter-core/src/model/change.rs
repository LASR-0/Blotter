use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::CoreError;
use crate::time::Instant;

/// Who made a change. Serialized as `user`, `mcp`, `system`, or
/// `ai:<provider>/<model>`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Actor {
    User,
    Ai { provider: String, model: String },
    Mcp,
    System,
}

impl fmt::Display for Actor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Actor::User => f.write_str("user"),
            Actor::Ai { provider, model } => write!(f, "ai:{provider}/{model}"),
            Actor::Mcp => f.write_str("mcp"),
            Actor::System => f.write_str("system"),
        }
    }
}

impl FromStr for Actor {
    type Err = CoreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "user" => Ok(Actor::User),
            "mcp" => Ok(Actor::Mcp),
            "system" => Ok(Actor::System),
            _ => {
                let rest = s
                    .strip_prefix("ai:")
                    .ok_or_else(|| CoreError::invalid(format!("unknown actor '{s}'")))?;
                let (provider, model) = rest
                    .split_once('/')
                    .ok_or_else(|| CoreError::invalid(format!("malformed AI actor '{s}'")))?;
                Ok(Actor::Ai {
                    provider: provider.to_owned(),
                    model: model.to_owned(),
                })
            }
        }
    }
}

impl Serialize for Actor {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Actor {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        String::deserialize(d)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}

/// One change-log row: the state of one entity before and after one
/// operation. Rows sharing a `batch_id` are undone and redone together.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Change {
    pub id: String,
    pub batch_id: String,
    pub op: String,
    pub entity: String,
    pub entity_id: String,
    pub before: Option<serde_json::Value>,
    pub after: Option<serde_json::Value>,
    pub actor: Actor,
    pub created_at: Instant,
    pub undone_at: Option<Instant>,
    /// For undo and redo rows: the batch they reverse or re-apply.
    #[serde(default)]
    pub reverts: Option<String>,
    /// The device a merged change came from; none for this device's own.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actor_round_trips() {
        for a in [
            Actor::User,
            Actor::Mcp,
            Actor::System,
            Actor::Ai {
                provider: "anthropic".into(),
                model: "claude-sonnet-5-5".into(),
            },
        ] {
            assert_eq!(a.to_string().parse::<Actor>().unwrap(), a);
        }
        assert_eq!(
            serde_json::to_string(&Actor::Ai {
                provider: "ollama".into(),
                model: "llama3.3:70b".into()
            })
            .unwrap(),
            r#""ai:ollama/llama3.3:70b""#
        );
        assert!("robot".parse::<Actor>().is_err());
        assert!("ai:nomodel".parse::<Actor>().is_err());
    }
}

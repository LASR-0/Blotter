use thiserror::Error;

/// Every fallible core function returns this. The app turns it into a toast;
/// the AI tool loop and MCP server turn it into a tool error the model can read.
#[derive(Debug, Error)]
pub enum CoreError {
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("migration failed: {0}")]
    Migration(#[from] rusqlite_migration::Error),

    #[error("{entity} {id} not found")]
    NotFound { entity: &'static str, id: String },

    #[error("invalid input: {0}")]
    Invalid(String),

    /// Arguments failed the operation's JSON Schema. Each entry reads like
    /// `/start: "tuesday" is not a valid date-time`, so a model can fix its call.
    #[error("invalid arguments for {op}: {}", errors.join("; "))]
    InvalidArgs { op: String, errors: Vec<String> },

    #[error("unknown operation '{0}'")]
    UnknownOp(String),

    #[error("corrupt stored value in {column}: {value}")]
    Corrupt { column: &'static str, value: String },

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

impl CoreError {
    pub fn not_found(entity: &'static str, id: impl Into<String>) -> Self {
        Self::NotFound {
            entity,
            id: id.into(),
        }
    }

    pub fn invalid(msg: impl Into<String>) -> Self {
        Self::Invalid(msg.into())
    }
}

pub type Result<T, E = CoreError> = std::result::Result<T, E>;

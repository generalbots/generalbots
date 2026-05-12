use thiserror::Error;

#[derive(Debug, Error)]
pub enum BotCoreBotError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("Session not found: {0}")]
    SessionNotFound(String),
    #[error("Bot not found: {0}")]
    BotNotFound(String),
    #[error("Access denied: {0}")]
    AccessDenied(String),
    #[error("LLM error: {0}")]
    Llm(String),
    #[error("Script error: {0}")]
    Script(String),
    #[error("Tool error: {0}")]
    Tool(String),
    #[error("IO error: {0}")]
    Io(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Cache error: {0}")]
    Cache(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<diesel::result::Error> for BotCoreBotError {
    fn from(e: diesel::result::Error) -> Self {
        BotCoreBotError::Database(e.to_string())
    }
}

impl From<std::io::Error> for BotCoreBotError {
    fn from(e: std::io::Error) -> Self {
        BotCoreBotError::Io(e.to_string())
    }
}

impl From<serde_json::Error> for BotCoreBotError {
    fn from(e: serde_json::Error) -> Self {
        BotCoreBotError::Serialization(e.to_string())
    }
}

impl From<r2d2::Error> for BotCoreBotError {
    fn from(e: r2d2::Error) -> Self {
        BotCoreBotError::Database(e.to_string())
    }
}

pub type BotResult<T> = Result<T, BotCoreBotError>;

use thiserror::Error;

pub type BotResult<T> = Result<T, BotError>;

#[derive(Error, Debug)]
pub enum BotError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("HTTP error: {status} - {message}")]
    Http { status: u16, message: String },

    #[error("Auth error: {0}")]
    Auth(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("{entity} not found")]
    NotFound { entity: String },

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Rate limited: retry after {retry_after_secs}s")]
    RateLimited { retry_after_secs: u64 },

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("Timeout after {duration_ms}ms")]
    Timeout { duration_ms: u64 },

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("{0}")]
    Other(String),
}

impl BotError {
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    pub fn database(msg: impl Into<String>) -> Self {
        Self::Database(msg.into())
    }

    pub fn http(status: u16, msg: impl Into<String>) -> Self {
        Self::Http {
            status,
            message: msg.into(),
        }
    }

    pub fn http_msg(msg: impl Into<String>) -> Self {
        Self::Http {
            status: 500,
            message: msg.into(),
        }
    }

    pub fn auth(msg: impl Into<String>) -> Self {
        Self::Auth(msg.into())
    }

    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }

    pub fn not_found(entity: impl Into<String>) -> Self {
        Self::NotFound {
            entity: entity.into(),
        }
    }

    pub fn conflict(msg: impl Into<String>) -> Self {
        Self::Conflict(msg.into())
    }

    #[must_use]
    pub const fn rate_limited(retry_after_secs: u64) -> Self {
        Self::RateLimited { retry_after_secs }
    }

    pub fn service_unavailable(msg: impl Into<String>) -> Self {
        Self::ServiceUnavailable(msg.into())
    }

    #[must_use]
    pub const fn timeout(duration_ms: u64) -> Self {
        Self::Timeout { duration_ms }
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }

    #[must_use]
    pub const fn status_code(&self) -> u16 {
        match self {
            Self::Http { status, .. } => *status,
            Self::Auth(_) => 401,
            Self::Validation(_) | Self::Json(_) => 400,
            Self::NotFound { .. } => 404,
            Self::Conflict(_) => 409,
            Self::RateLimited { .. } => 429,
            Self::ServiceUnavailable(_) => 503,
            Self::Timeout { .. } => 504,
            Self::Config(_)
            | Self::Database(_)
            | Self::Internal(_)
            | Self::Io(_)
            | Self::Other(_) => 500,
        }
    }

    #[must_use]
    pub const fn is_retryable(&self) -> bool {
        match self {
            Self::RateLimited { .. } | Self::ServiceUnavailable(_) | Self::Timeout { .. } => true,
            Self::Http { status, .. } => *status >= 500,
            _ => false,
        }
    }

    #[must_use]
    pub const fn is_client_error(&self) -> bool {
        let code = self.status_code();
        code >= 400 && code < 500
    }

    #[must_use]
    pub const fn is_server_error(&self) -> bool {
        self.status_code() >= 500
    }
}

impl From<anyhow::Error> for BotError {
    fn from(err: anyhow::Error) -> Self {
        Self::Other(err.to_string())
    }
}

impl From<String> for BotError {
    fn from(msg: String) -> Self {
        Self::Other(msg)
    }
}

impl From<&str> for BotError {
    fn from(msg: &str) -> Self {
        Self::Other(msg.to_string())
    }
}

#[cfg(feature = "http-client")]
impl From<reqwest::Error> for BotError {
    fn from(err: reqwest::Error) -> Self {
        let status = err.status().map_or(500, |s| s.as_u16());
        Self::Http {
            status,
            message: err.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = BotError::config("missing API key");
        assert_eq!(err.to_string(), "Configuration error: missing API key");
    }

    #[test]
    fn test_not_found_error() {
        let err = BotError::not_found("User");
        assert_eq!(err.to_string(), "User not found");
        assert_eq!(err.status_code(), 404);
    }

    #[test]
    fn test_http_error_with_status() {
        let err = BotError::http(503, "Service down");
        assert_eq!(err.status_code(), 503);
        assert!(err.is_server_error());
        assert!(!err.is_client_error());
    }

    #[test]
    fn test_validation_error() {
        let err = BotError::validation("Invalid email format");
        assert_eq!(err.status_code(), 400);
        assert!(err.is_client_error());
    }

    #[test]
    fn test_retryable_errors() {
        assert!(BotError::rate_limited(60).is_retryable());
        assert!(BotError::service_unavailable("down").is_retryable());
        assert!(BotError::timeout(5000).is_retryable());
        assert!(!BotError::validation("bad input").is_retryable());
        assert!(!BotError::not_found("User").is_retryable());
    }

    #[test]
    fn test_rate_limited_display() {
        let err = BotError::rate_limited(30);
        assert_eq!(err.to_string(), "Rate limited: retry after 30s");
        assert_eq!(err.status_code(), 429);
    }

    #[test]
    fn test_timeout_display() {
        let err = BotError::timeout(5000);
        assert_eq!(err.to_string(), "Timeout after 5000ms");
        assert_eq!(err.status_code(), 504);
    }
}

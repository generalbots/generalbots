use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

/// Errors surfaced by the integration connection control plane (#939).
///
/// Every variant maps to a safe, static client-facing message. Raw database,
/// Vault or provider error strings are never returned to HTTP clients; they
/// are only written to the server log.
#[derive(Debug, thiserror::Error)]
pub enum IntegrationError {
    #[error("unauthorized scope")]
    UnauthorizedScope,
    #[error("resource not found")]
    NotFound,
    #[error("credential store unavailable")]
    VaultUnavailable,
    #[error("{0}")]
    Validation(String),
    #[error("storage failure")]
    Storage(String),
    #[error("conflict")]
    Conflict,
}

impl IntegrationError {
    fn status_and_message(&self) -> (StatusCode, &'static str) {
        match self {
            Self::UnauthorizedScope => (
                StatusCode::FORBIDDEN,
                "Access to this bot is not allowed for the caller",
            ),
            Self::NotFound => (StatusCode::NOT_FOUND, "Resource not found"),
            Self::VaultUnavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "Credential store unavailable; request rejected",
            ),
            Self::Validation(_) => (StatusCode::BAD_REQUEST, "Validation failed"),
            Self::Storage(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal storage error"),
            Self::Conflict => (StatusCode::CONFLICT, "Resource conflict"),
        }
    }
}

impl IntoResponse for IntegrationError {
    fn into_response(self) -> Response {
        let (status, message) = self.status_and_message();
        match &self {
            // Expected authorization outcomes are logged at debug level.
            Self::UnauthorizedScope => {
                log::debug!("integration connection denied: unauthorized scope")
            }
            Self::NotFound => log::debug!("integration connection lookup miss"),
            Self::Validation(reason) => {
                log::warn!("integration connection validation failed: {reason}")
            }
            Self::Conflict => log::warn!("integration connection conflict"),
            // Infrastructure failures carry internal detail - keep it out of
            // the response body and log it server-side only.
            Self::VaultUnavailable => {
                log::error!("integration connection vault unavailable: {self}")
            }
            Self::Storage(detail) => {
                log::error!("integration connection storage failure: {detail}")
            }
        }
        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

impl From<diesel::result::Error> for IntegrationError {
    fn from(value: diesel::result::Error) -> Self {
        match value {
            diesel::result::Error::NotFound => Self::NotFound,
            other => Self::Storage(other.to_string()),
        }
    }
}

impl From<diesel::r2d2::PoolError> for IntegrationError {
    fn from(value: diesel::r2d2::PoolError) -> Self {
        Self::Storage(format!("pool error: {value}"))
    }
}

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug)]
pub enum AuthError {
    MissingToken,
    InvalidToken,
    ExpiredToken,
    InsufficientPermissions,
    InvalidApiKey,
    SessionExpired,
    UserNotFound,
    AccountDisabled,
    RateLimited,
    BotAccessDenied,
    BotNotFound,
    OrganizationAccessDenied,
    InternalError(String),
}

impl AuthError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::MissingToken => StatusCode::UNAUTHORIZED,
            Self::InvalidToken => StatusCode::UNAUTHORIZED,
            Self::ExpiredToken => StatusCode::UNAUTHORIZED,
            Self::InsufficientPermissions => StatusCode::FORBIDDEN,
            Self::InvalidApiKey => StatusCode::UNAUTHORIZED,
            Self::SessionExpired => StatusCode::UNAUTHORIZED,
            Self::UserNotFound => StatusCode::UNAUTHORIZED,
            Self::AccountDisabled => StatusCode::FORBIDDEN,
            Self::RateLimited => StatusCode::TOO_MANY_REQUESTS,
            Self::BotAccessDenied => StatusCode::FORBIDDEN,
            Self::BotNotFound => StatusCode::NOT_FOUND,
            Self::OrganizationAccessDenied => StatusCode::FORBIDDEN,
            Self::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn error_code(&self) -> &'static str {
        match self {
            Self::MissingToken => "missing_token",
            Self::InvalidToken => "invalid_token",
            Self::ExpiredToken => "expired_token",
            Self::InsufficientPermissions => "insufficient_permissions",
            Self::InvalidApiKey => "invalid_api_key",
            Self::SessionExpired => "session_expired",
            Self::UserNotFound => "user_not_found",
            Self::AccountDisabled => "account_disabled",
            Self::RateLimited => "rate_limited",
            Self::BotAccessDenied => "bot_access_denied",
            Self::BotNotFound => "bot_not_found",
            Self::OrganizationAccessDenied => "organization_access_denied",
            Self::InternalError(_) => "internal_error",
        }
    }

    pub fn message(&self) -> String {
        match self {
            Self::MissingToken => "Authentication token is required".to_string(),
            Self::InvalidToken => "Invalid authentication token".to_string(),
            Self::ExpiredToken => "Authentication token has expired".to_string(),
            Self::InsufficientPermissions => {
                "You don't have permission to access this resource".to_string()
            }
            Self::InvalidApiKey => "Invalid API key".to_string(),
            Self::SessionExpired => "Your session has expired".to_string(),
            Self::UserNotFound => "User not found".to_string(),
            Self::AccountDisabled => "Your account has been disabled".to_string(),
            Self::RateLimited => "Too many requests, please try again later".to_string(),
            Self::BotAccessDenied => "You don't have access to this bot".to_string(),
            Self::BotNotFound => "Bot not found".to_string(),
            Self::OrganizationAccessDenied => {
                "You don't have access to this organization".to_string()
            }
            Self::InternalError(_) => "An internal error occurred".to_string(),
        }
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let body = Json(json!({
            "error": self.error_code(),
            "message": self.message()
        }));
        (status, body).into_response()
    }
}

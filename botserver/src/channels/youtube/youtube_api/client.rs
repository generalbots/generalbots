//! HTTP Client Helper Functions
//!
//! Contains helper functions for making HTTP requests to the YouTube API
//! and parsing error responses.

use crate::channels::ChannelError;
use super::models::YouTubeErrorResponse;

/// Parse error response from YouTube API
pub async fn parse_error_response(response: reqwest::Response) -> ChannelError {
    let status = response.status();

    if status.as_u16() == 401 {
        return ChannelError::AuthenticationFailed("Invalid or expired token".to_string());
    }

    if status.as_u16() == 403 {
        return ChannelError::AuthenticationFailed("Insufficient permissions".to_string());
    }

    if status.as_u16() == 429 {
        let retry_after = response
            .headers()
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse().ok());
        return ChannelError::RateLimited { retry_after };
    }

    let error_text = response.text().await.unwrap_or_default();

    if let Ok(error_response) = serde_json::from_str::<YouTubeErrorResponse>(&error_text) {
        return ChannelError::ApiError {
            code: Some(error_response.error.code.to_string()),
            message: error_response.error.message,
        };
    }

    ChannelError::ApiError {
        code: Some(status.to_string()),
        message: error_text,
    }
}

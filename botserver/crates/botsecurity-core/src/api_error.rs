//! The single HTTP error envelope for the platform (#2414).
//!
//! Handlers that still return `(StatusCode, String)` are being migrated in
//! batches and are tracked by `scripts/check_raw_error_strings.py`. New code
//! should return [`ApiError`]: one JSON shape with a sanitised message, a
//! stable machine-readable code and a short request id for correlation.
//!
//! `ApiError` never puts the original error string on the wire. It calls the
//! same sanitizer the existing helpers use, so DB/IO details stay in logs.

use crate::error_sanitizer::{log_and_sanitize_str, SafeErrorResponse};
use crate::request_id::RequestId;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    body: SafeErrorResponse,
}

impl ApiError {
    /// Builds an envelope from an internal error, logging the unsanitized
    /// detail locally while returning only the sanitized text.
    #[must_use]
    pub fn internal(
        status: StatusCode,
        code: impl Into<String>,
        error: &(impl std::fmt::Display + ?Sized),
        context: &str,
    ) -> Self {
        let request_id = RequestId::new().short_id().to_string();
        let mut body = log_and_sanitize_str(&error.to_string(), context, Some(&request_id));
        body.code = Some(code.into());
        Self { status, body }
    }

    /// Builds an envelope for a caller-controlled failure (validation, 404).
    #[must_use]
    pub fn message(status: StatusCode, code: impl Into<String>, message: impl Into<String>) -> Self {
        let mut body = SafeErrorResponse {
            error: "request_failed".to_string(),
            message: message.into(),
            code: None,
            request_id: None,
            details: None,
        };
        body.code = Some(code.into());
        Self { status, body }
    }

    /// The status code of this failure.
    #[must_use]
    pub fn status(&self) -> StatusCode {
        self.status
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(self.body)).into_response()
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.status, self.body.error)
    }
}

impl std::error::Error for ApiError {}

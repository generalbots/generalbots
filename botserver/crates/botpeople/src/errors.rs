//! Localized API errors for the people routes (#2094).
//!
//! Every handler answers with a stable machine-readable `code` plus a
//! translated `message`; the legacy `error` string mirrors the message so
//! clients that still read a plain string keep working. The database or
//! driver cause is logged and never echoed to the caller.

use axum::http::{header::ACCEPT_LANGUAGE, HeaderMap, StatusCode};
use axum::Json;
use botlib::i18n::{self, Locale};
use serde_json::{json, Value};

/// Localized error response: `(status, { code, message, error })`.
pub type ApiError = (StatusCode, Json<Value>);

/// Request locale from `X-Locale` / `Accept-Language` (#2094).
#[must_use]
pub fn locale_from_headers(headers: &HeaderMap) -> Locale {
    i18n::locale_from_request(
        headers.get(i18n::LOCALE_HEADER).and_then(|v| v.to_str().ok()),
        headers.get(ACCEPT_LANGUAGE).and_then(|v| v.to_str().ok()),
    )
}

/// Localized error for a catalogue `code`.
#[must_use]
pub fn api_error(locale: &Locale, status: StatusCode, code: &str) -> ApiError {
    let message = i18n::get(locale, code);

    (
        status,
        Json(json!({ "code": code, "message": &message, "error": message })),
    )
}

/// Internal failure: the cause is logged, the client gets a generic localized
/// message.
pub fn api_error_db<E: std::fmt::Display>(
    locale: &Locale,
    status: StatusCode,
    error: E,
) -> ApiError {
    log::warn!(
        "botpeople api error ({}) at {}: {error}",
        status.as_u16(),
        module_path!()
    );

    api_error(locale, status, "error-db-unavailable")
}

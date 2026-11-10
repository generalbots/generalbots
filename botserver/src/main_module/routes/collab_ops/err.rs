use super::*;

/// Localized API error (#2094) — `code` stays machine-readable.
pub(super) fn err(
    locale: &botcore::i18n::Locale,
    status: StatusCode,
    key: &str,
) -> (StatusCode, Json<serde_json::Value>) {
    warn!("collab ops error ({}): {key}", status.as_u16());
    (status, Json(botcore::i18n::error_body(locale, key)))
}

/// Internal failure: the cause is logged, the client gets a generic localized
/// message — no internal error strings leak to the caller.
pub(super) fn err_db<E: std::fmt::Display>(
    locale: &botcore::i18n::Locale,
    status: StatusCode,
    error: E,
) -> (StatusCode, Json<serde_json::Value>) {
    warn!("collab ops internal error ({}): {error}", status.as_u16());
    (
        status,
        Json(botcore::i18n::error_body(locale, "error-db-unavailable")),
    )
}
pub(super) fn normalize_actor_type(actor_type: &str) -> &'static str {
    if actor_type == "llm" {
        "llm"
    } else {
        "human"
    }
}

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

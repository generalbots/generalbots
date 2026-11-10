use std::sync::Arc;

use axum::http::StatusCode;
use axum::response::Json;
use diesel::prelude::*;
use diesel::sql_types::Text;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::security::auth_api::types::AuthenticatedUser;
use botcore::shared::state::AppState;

use crate::core::ai_accounts;

pub(super) type ApiError = (StatusCode, Json<Value>);

/// Localized API error (#2094) — `code` stays machine-readable.
pub(super) fn err(
    locale: &botcore::i18n::Locale,
    status: StatusCode,
    key: &str,
) -> ApiError {
    (status, Json(botcore::i18n::error_body(locale, key)))
}

/// Upstream driver failure: the provider's own message is not ours to
/// translate, so it is passed through while the status code stays stable.
pub(super) fn err_upstream(status: StatusCode, message: &str) -> ApiError {
    log::warn!("ai accounts: upstream error ({}): {message}", status.as_u16());
    (status, Json(json!({ "error": message })))
}

/// Internal failure: the cause is logged, the client gets a generic localized
/// message — no internal error strings leak to the caller.
pub(super) fn err_db<E: std::fmt::Display>(
    locale: &botcore::i18n::Locale,
    status: StatusCode,
    error: E,
) -> ApiError {
    log::warn!("ai accounts: internal error ({}): {error}", status.as_u16());
    (
        status,
        Json(botcore::i18n::error_body(locale, "error-db-unavailable")),
    )
}

pub(super) fn provider_or_error(
    locale: &botcore::i18n::Locale,
    value: &str,
) -> Result<&'static str, ApiError> {
    ai_accounts::normalize_provider(value).ok_or_else(|| {
        err(
            locale,
            StatusCode::BAD_REQUEST,
            "error-ai-account-unknown-provider",
        )
    })
}

pub(super) fn agent_service(
    locale: &botcore::i18n::Locale,
) -> Result<&'static Arc<botagent::AgentService>, ApiError> {
    crate::core::bot::agent_vm_hook::service().ok_or_else(|| {
        err(
            locale,
            StatusCode::SERVICE_UNAVAILABLE,
            "error-ai-account-workspace-unavailable",
        )
    })
}

pub(super) fn ensure_user(
    locale: &botcore::i18n::Locale,
    state: &Arc<AppState>,
    user: &AuthenticatedUser,
) -> Result<(), ApiError> {
    ai_accounts::ensure_user(&state.conn, user.user_id, user.email.as_deref())
        .map_err(|error| err_db(locale, StatusCode::INTERNAL_SERVER_ERROR, error))
}

/// First bot is the desktop default; the VM is per user, so any stable bot id
/// works for provisioning.
pub(super) fn first_bot_id(
    locale: &botcore::i18n::Locale,
    state: &Arc<AppState>,
) -> Result<Uuid, ApiError> {
    #[derive(QueryableByName)]
    struct Row {
        #[diesel(sql_type = Text)]
        id: String,
    }
    let mut conn = state
        .conn
        .get()
        .map_err(|error| err_db(locale, StatusCode::INTERNAL_SERVER_ERROR, error))?;
    let row = diesel::sql_query("SELECT id::text AS id FROM bots ORDER BY created_at ASC LIMIT 1")
        .load::<Row>(&mut conn)
        .unwrap_or_default()
        .into_iter()
        .next();
    match row.and_then(|row| Uuid::parse_str(&row.id).ok()) {
        Some(id) => Ok(id),
        None => Ok(Uuid::nil()),
    }
}

pub(super) async fn ensure_vm(
    locale: &botcore::i18n::Locale,
    state: &Arc<AppState>,
    user_id: Uuid,
) -> Result<botagent::models::AgentSessionRow, ApiError> {
    let service = agent_service(locale)?;
    let bot_id = first_bot_id(locale, state)?;
    botagent::vm::ensure_vm(service, &user_id, &bot_id, None)
        .await
        .map_err(|(status, message)| err_upstream(status, &message))
}

pub(super) fn map_claude_state(state: &str) -> &'static str {
    match state {
        "linked" => "linked",
        "waiting" | "starting" | "verifying" => "connecting",
        "error" => "error",
        _ => "unlinked",
    }
}

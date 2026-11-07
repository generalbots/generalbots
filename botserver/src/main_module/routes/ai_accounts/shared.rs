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

pub(super) fn err(status: StatusCode, message: &str) -> ApiError {
    (status, Json(json!({ "error": message })))
}

pub(super) fn provider_or_error(value: &str) -> Result<&'static str, ApiError> {
    ai_accounts::normalize_provider(value)
        .ok_or_else(|| err(StatusCode::BAD_REQUEST, "Unknown AI account provider"))
}

pub(super) fn agent_service() -> Result<&'static Arc<botagent::AgentService>, ApiError> {
    crate::core::bot::agent_vm_hook::service().ok_or_else(|| {
        err(
            StatusCode::SERVICE_UNAVAILABLE,
            "The agent workspace is not available in this build",
        )
    })
}

pub(super) fn ensure_user(state: &Arc<AppState>, user: &AuthenticatedUser) -> Result<(), ApiError> {
    ai_accounts::ensure_user(&state.conn, user.user_id, user.email.as_deref())
        .map_err(|error| err(StatusCode::INTERNAL_SERVER_ERROR, &error))
}

/// First bot is the desktop default; the VM is per user, so any stable bot id
/// works for provisioning.
pub(super) fn first_bot_id(state: &Arc<AppState>) -> Result<Uuid, ApiError> {
    #[derive(QueryableByName)]
    struct Row {
        #[diesel(sql_type = Text)]
        id: String,
    }
    let mut conn = state.conn.get().map_err(|error| {
        log::warn!("ai accounts: database pool: {error}");
        err(StatusCode::INTERNAL_SERVER_ERROR, "Database unavailable")
    })?;
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
    state: &Arc<AppState>,
    user_id: Uuid,
) -> Result<botagent::models::AgentSessionRow, ApiError> {
    let service = agent_service()?;
    let bot_id = first_bot_id(state)?;
    botagent::vm::ensure_vm(service, &user_id, &bot_id, None)
        .await
        .map_err(|(status, message)| err(status, &message))
}

pub(super) fn map_claude_state(state: &str) -> &'static str {
    match state {
        "linked" => "linked",
        "waiting" | "starting" | "verifying" => "connecting",
        "error" => "error",
        _ => "unlinked",
    }
}

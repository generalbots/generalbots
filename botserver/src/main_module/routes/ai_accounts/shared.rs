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

/// A status read of a CLI account inside the workspace: the CLI's own
/// answer, or the workspace state when it is not running or not answering.
pub(super) enum VmProbe<T> {
    Ready(T),
    Workspace(botagent::vm::WorkspaceState),
}

/// Runs `read` only when the user's workspace is running, and gives the
/// whole check the probe budget (about 5 s): a stopped, starting or wedged
/// workspace answers with its state instead of holding the request. The
/// workspace is never started from here.
pub(super) async fn probe_vm<T, F>(
    service: &botagent::AgentService,
    row: &botagent::models::AgentSessionRow,
    read: F,
) -> Result<VmProbe<T>, (StatusCode, String)>
where
    F: std::future::Future<Output = Result<T, (StatusCode, String)>>,
{
    use botagent::vm::{WorkspaceState, INCUS_TIMEOUT_CODE};
    let timed_out = VmProbe::Workspace(WorkspaceState::Unavailable(INCUS_TIMEOUT_CODE));
    let checked = botagent::vm::within_budget(botagent::vm::PROBE_DEADLINE, async {
        match botagent::vm::workspace_state(service, &row.user_id, &row.vm_name, &row.status).await {
            WorkspaceState::Running => read.await.map(VmProbe::Ready),
            other => Ok(VmProbe::Workspace(other)),
        }
    })
    .await;
    match checked {
        None => Ok(timed_out),
        Some(Err(error)) if botagent::vm::is_timeout(&error) => Ok(timed_out),
        Some(result) => result,
    }
}

/// The stored account row, marked with the workspace state that kept it
/// from being checked live.
pub(super) fn stored_with_workspace(
    state: &Arc<AppState>,
    user_id: Uuid,
    provider: &str,
    workspace: botagent::vm::WorkspaceState,
) -> Value {
    let mut account = ai_accounts::list_accounts(&state.conn, user_id)
        .into_iter()
        .find(|a| a.get("provider").and_then(Value::as_str) == Some(provider))
        .unwrap_or_else(|| json!({ "provider": provider, "status": "unlinked" }));
    if let Some(fields) = account.as_object_mut() {
        fields.insert("workspace".into(), json!(workspace.as_str()));
        if let Some(code) = workspace.code() {
            fields.insert("workspace_code".into(), json!(code));
        }
    }
    account
}

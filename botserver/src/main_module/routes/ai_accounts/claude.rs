use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::Extension;
use axum::Json;
use botcore::shared::state::AppState;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use super::shared::{
    agent_service, ensure_user, ensure_vm, err_db, err_upstream, map_claude_state, ApiError,
};
use crate::core::ai_accounts;
use crate::security::auth_api::types::AuthenticatedUser;

pub(super) async fn connect_claude(
    locale: &botcore::i18n::Locale,
    state: &Arc<AppState>,
    user_id: Uuid,
) -> Result<Json<Value>, ApiError> {
    let service = agent_service(locale)?;
    let row = ensure_vm(locale, state, user_id).await?;
    let status = botagent::claude::start_login(service, &row.vm_name)
        .await
        .map_err(|(code, message)| err_upstream(code, &message))?;
    if status.state == "linked" {
        ai_accounts::mark_linked(
            &state.conn,
            user_id,
            ai_accounts::CLAUDE_CODE,
            None,
            None,
            Some(&row.vm_name),
            None,
        )
        .map_err(|error| err_db(locale, StatusCode::INTERNAL_SERVER_ERROR, error))?;
    } else {
        ai_accounts::update_status(
            &state.conn,
            user_id,
            ai_accounts::CLAUDE_CODE,
            map_claude_state(&status.state),
            None,
            None,
            Some(&row.vm_name),
        )
        .map_err(|error| err_db(locale, StatusCode::INTERNAL_SERVER_ERROR, error))?;
    }
    Ok(Json(json!({
        "provider": ai_accounts::CLAUDE_CODE,
        "state": status.state,
        "status": map_claude_state(&status.state),
        "url": status.url,
        "message": status.message,
        "vm_name": row.vm_name,
    })))
}

#[derive(Deserialize)]
pub struct ClaudeCodeBody {
    pub code: String,
}

/// POST /api/ai-accounts/claude-code/code — forwards the pasted OAuth code.
pub async fn claude_code(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    request_locale: botcore::i18n::RequestLocale,
    Json(body): Json<ClaudeCodeBody>,
) -> Result<Json<Value>, ApiError> {
    let locale = request_locale.locale();
    let service = agent_service(locale)?;
    ensure_user(locale, &state, &user)?;
    let row = ensure_vm(locale, &state, user.user_id).await?;
    let status = botagent::claude::submit_code(service, &row.vm_name, &body.code)
        .await
        .map_err(|(code, message)| err_upstream(code, &message))?;
    if status.state == "linked" {
        ai_accounts::mark_linked(
            &state.conn,
            user.user_id,
            ai_accounts::CLAUDE_CODE,
            None,
            None,
            Some(&row.vm_name),
            None,
        )
        .map_err(|error| err_db(locale, StatusCode::INTERNAL_SERVER_ERROR, error))?;
    } else {
        ai_accounts::update_status(
            &state.conn,
            user.user_id,
            ai_accounts::CLAUDE_CODE,
            map_claude_state(&status.state),
            None,
            None,
            Some(&row.vm_name),
        )
        .map_err(|error| err_db(locale, StatusCode::INTERNAL_SERVER_ERROR, error))?;
    }
    Ok(Json(json!({
        "provider": ai_accounts::CLAUDE_CODE,
        "state": status.state,
        "status": map_claude_state(&status.state),
        "message": status.message,
    })))
}

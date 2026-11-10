use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use botcore::shared::state::AppState;
use serde_json::{json, Value};
use uuid::Uuid;

use super::claude::connect_claude;
use super::shared::{
    agent_service, ensure_user, err, err_db, err_upstream, map_claude_state, provider_or_error,
    ApiError,
};
use crate::core::{ai_accounts, ai_accounts_chatgpt};
use crate::security::auth_api::types::AuthenticatedUser;

/// GET /api/ai-accounts
pub async fn list(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Json<Value> {
    let accounts = ai_accounts::list_accounts(&state.conn, user.user_id);
    Json(json!({
        "accounts": accounts,
        "providers": [
            { "id": "claude-code", "label": "Claude Code", "kind": "subscription",
              "description": "Run the unmodified Claude Code CLI in your agent VM; credentials stay there." },
            { "id": "chatgpt-plan", "label": "ChatGPT plan", "kind": "subscription",
              "description": "Sign in with ChatGPT and use your plan allowance (open-source client flow)." }
        ]
    }))
}

/// POST /api/ai-accounts/:provider/connect
pub async fn connect(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    request_locale: botcore::i18n::RequestLocale,
    Path(provider): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let locale = request_locale.locale();
    let provider = provider_or_error(locale, &provider)?;
    ensure_user(locale, &state, &user)?;
    match provider {
        ai_accounts::CLAUDE_CODE => connect_claude(locale, &state, user.user_id).await,
        ai_accounts::CHATGPT_PLAN => connect_chatgpt(&state, user.user_id).await,
        _ => Err(err(
            locale,
            StatusCode::BAD_REQUEST,
            "error-ai-account-unknown-provider",
        )),
    }
}

async fn connect_chatgpt(state: &Arc<AppState>, user_id: Uuid) -> Result<Json<Value>, ApiError> {
    ai_accounts_chatgpt::start_connect(&state.conn, user_id)
        .await
        .map_err(|(code, message)| err_upstream(code, &message))
}

/// POST /api/ai-accounts/:provider/reconnect
pub async fn reconnect(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    request_locale: botcore::i18n::RequestLocale,
    Path(provider): Path<String>,
) -> Result<Json<Value>, ApiError> {
    // Reconnect is the same flow; the provider decides whether an existing
    // session is still usable and returns it unchanged.
    connect(State(state), Extension(user), request_locale, Path(provider)).await
}

/// POST /api/ai-accounts/:provider/status
pub async fn status(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    request_locale: botcore::i18n::RequestLocale,
    Path(provider): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let locale = request_locale.locale();
    let provider = provider_or_error(locale, &provider)?;
    ensure_user(locale, &state, &user)?;
    let account = if provider == ai_accounts::CLAUDE_CODE {
        let vm = botagent::vm::get_user_vm(agent_service(locale)?, &user.user_id)
            .map_err(|(code, message)| err_upstream(code, &message))?;
        match vm {
            Some(row) => {
                let status = botagent::claude::status(agent_service(locale)?, &row.vm_name)
                    .await
                    .map_err(|(code, message)| err_upstream(code, &message))?;
                if status.state == "linked" {
                    let _ = ai_accounts::mark_linked(
                        &state.conn,
                        user.user_id,
                        ai_accounts::CLAUDE_CODE,
                        None,
                        None,
                        Some(&row.vm_name),
                        None,
                    );
                } else {
                    let _ = ai_accounts::update_status(
                        &state.conn,
                        user.user_id,
                        ai_accounts::CLAUDE_CODE,
                        map_claude_state(&status.state),
                        None,
                        None,
                        Some(&row.vm_name),
                    );
                }
                json!({
                    "provider": ai_accounts::CLAUDE_CODE,
                    "status": map_claude_state(&status.state),
                    "state": status.state,
                    "url": status.url,
                    "message": status.message,
                    "vm_name": row.vm_name,
                })
            }
            None => ai_accounts_chatgpt::status(&state.conn, user.user_id),
        }
    } else {
        let row = ai_accounts_chatgpt::status(&state.conn, user.user_id);
        if row.get("status").and_then(Value::as_str) == Some("unlinked")
            && ai_accounts_chatgpt::pending_for(user.user_id)
        {
            return Ok(Json(json!({
                "account": {
                    "provider": ai_accounts::CHATGPT_PLAN,
                    "status": "connecting",
                    "message": "Finish the ChatGPT sign-in in your browser."
                },
                "checked_at": chrono::Utc::now().to_rfc3339(),
            })));
        }
        if row.get("status").and_then(Value::as_str) == Some("linked") {
            // A stored refresh token can still be expired; surface the honest
            // state instead of reporting a dead account as linked.
            if ai_accounts_chatgpt::access_token(user.user_id)
                .await
                .is_err()
            {
                let _ = ai_accounts::update_status(
                    &state.conn,
                    user.user_id,
                    ai_accounts::CHATGPT_PLAN,
                    "expired",
                    None,
                    None,
                    None,
                );
                return Ok(Json(json!({
                    "account": { "provider": ai_accounts::CHATGPT_PLAN, "status": "expired" },
                    "checked_at": chrono::Utc::now().to_rfc3339(),
                })));
            }
        }
        row
    };
    Ok(Json(json!({
        "account": account,
        "checked_at": chrono::Utc::now().to_rfc3339(),
        "note": "Claude health is verified inside the agent VM; ChatGPT health is verified on the next request."
    })))
}

/// POST /api/ai-accounts/:provider/disconnect
pub async fn disconnect(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    request_locale: botcore::i18n::RequestLocale,
    Path(provider): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let locale = request_locale.locale();
    let provider = provider_or_error(locale, &provider)?;
    ensure_user(locale, &state, &user)?;
    match provider {
        ai_accounts::CLAUDE_CODE => {
            if let Some(service) = crate::core::bot::agent_vm_hook::service() {
                if let Ok(Some(row)) = botagent::vm::get_user_vm(service, &user.user_id) {
                    let _ = botagent::claude::logout(service, &row.vm_name).await;
                }
            }
            ai_accounts::update_status(
                &state.conn,
                user.user_id,
                ai_accounts::CLAUDE_CODE,
                "unlinked",
                None,
                None,
                None,
            )
            .map_err(|error| err_db(locale, StatusCode::INTERNAL_SERVER_ERROR, error))?;
        }
        ai_accounts::CHATGPT_PLAN => {
            ai_accounts_chatgpt::disconnect(&state.conn, user.user_id)
                .await
                .map_err(|error| err_db(locale, StatusCode::INTERNAL_SERVER_ERROR, error))?;
        }
        _ => {
            return Err(err(
                locale,
                StatusCode::BAD_REQUEST,
                "error-ai-account-unknown-provider",
            ))
        }
    }
    Ok(Json(json!({ "provider": provider, "status": "unlinked" })))
}

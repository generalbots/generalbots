use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use botcore::shared::state::AppState;
use serde_json::{json, Value};
use uuid::Uuid;

use super::antigravity::connect_antigravity;
use super::claude::connect_claude;
use super::codex::connect_codex;
use super::shared::{
    agent_service, ensure_user, err, err_db, err_upstream, map_claude_state, probe_vm,
    provider_or_error, stored_with_workspace, ApiError, VmProbe,
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
              "description": "Sign in with ChatGPT and use your plan allowance (open-source client flow)." },
            { "id": "codex-cli", "label": "Codex", "kind": "subscription",
              "description": "Connect your Codex/CLI subscription (flow in progress)." },
            { "id": "antigravity", "label": "Antigravity", "kind": "subscription",
              "description": "Connect your Antigravity account (flow in progress)." }
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
        ai_accounts::CODEX_CLI => connect_codex(locale, &state, user.user_id).await,
        ai_accounts::ANTIGRAVITY => connect_antigravity(locale, &state, user.user_id).await,
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
                let service = agent_service(locale)?;
                let status = match probe_vm(service, &row, botagent::claude::status(service, &row.vm_name))
                    .await
                    .map_err(|(code, message)| err_upstream(code, &message))?
                {
                    VmProbe::Ready(status) => status,
                    VmProbe::Workspace(workspace) => {
                        return Ok(Json(json!({
                            "account": stored_with_workspace(&state, user.user_id, provider, workspace),
                            "checked_at": chrono::Utc::now().to_rfc3339(),
                        })));
                    }
                };
                if status.state == "linked" {
                    if let Err(e) = ai_accounts::mark_linked(&state.conn, user.user_id, ai_accounts::CLAUDE_CODE, None, None, Some(&row.vm_name), None) {
                        log::warn!("Failed to record Claude Code link: {e}");
                    }
                } else {
                    if let Err(e) = ai_accounts::update_status(&state.conn, user.user_id, ai_accounts::CLAUDE_CODE, map_claude_state(&status.state), None, None, Some(&row.vm_name)) {
                        log::warn!("Failed to update Claude Code account status: {e}");
                    }
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
            None => ai_accounts::list_accounts(&state.conn, user.user_id)
                .into_iter()
                .find(|a| a.get("provider").and_then(Value::as_str) == Some(ai_accounts::CLAUDE_CODE))
                .unwrap_or_else(|| json!({"provider": ai_accounts::CLAUDE_CODE, "status": "unlinked"})),
        }
    } else if provider == ai_accounts::CHATGPT_PLAN {
        let row = ai_accounts_chatgpt::status(&state.conn, user.user_id);
        if row.get("status").and_then(Value::as_str) == Some("unlinked") && ai_accounts_chatgpt::pending_for(user.user_id) {
            return Ok(Json(json!({
                "account": { "provider": ai_accounts::CHATGPT_PLAN, "status": "connecting", "message": "Finish the ChatGPT sign-in in your browser." },
                "checked_at": chrono::Utc::now().to_rfc3339(),
            })));
        }
        if row.get("status").and_then(Value::as_str) == Some("linked") {
            if ai_accounts_chatgpt::access_token(user.user_id).await.is_err() {
                if let Err(e) = ai_accounts::update_status(&state.conn, user.user_id, ai_accounts::CHATGPT_PLAN, "expired", None, None, None) {
                    log::warn!("Failed to mark ChatGPT plan as expired: {e}");
                }
                return Ok(Json(json!({
                    "account": { "provider": ai_accounts::CHATGPT_PLAN, "status": "expired" },
                    "checked_at": chrono::Utc::now().to_rfc3339(),
                })));
            }
        }
        row
    } else if provider == ai_accounts::CODEX_CLI || provider == ai_accounts::ANTIGRAVITY {
        let vm = botagent::vm::get_user_vm(agent_service(locale)?, &user.user_id)
            .map_err(|(code, message)| err_upstream(code, &message))?;
        match vm {
            Some(row) => {
                let service = agent_service(locale)?;
                let probed = if provider == ai_accounts::CODEX_CLI {
                    probe_vm(service, &row, async {
                        botagent::codex::status(service, &row.vm_name)
                            .await
                            .map(|s| (s.state, s.url, json!({"device_code": s.device_code}), s.message))
                    })
                    .await
                } else {
                    probe_vm(service, &row, async {
                        botagent::antigravity::status(service, &row.vm_name)
                            .await
                            .map(|s| (s.state, s.url, json!({"auth_code": s.auth_code}), s.message))
                    })
                    .await
                };
                let (st, url, extra, msg) = match probed.map_err(|(c, m)| err_upstream(c, &m))? {
                    VmProbe::Ready(read) => read,
                    VmProbe::Workspace(workspace) => {
                        return Ok(Json(json!({
                            "account": stored_with_workspace(&state, user.user_id, provider, workspace),
                            "checked_at": chrono::Utc::now().to_rfc3339(),
                        })));
                    }
                };
                let mapped = map_claude_state(&st);
                if st == "linked" {
                    if let Err(e) = ai_accounts::mark_linked(&state.conn, user.user_id, provider, None, None, Some(&row.vm_name), None) {
                        log::warn!("Failed to record {provider} link: {e}");
                    }
                } else {
                    if let Err(e) = ai_accounts::update_status(&state.conn, user.user_id, provider, mapped, None, None, Some(&row.vm_name)) {
                        log::warn!("Failed to update {provider} account status: {e}");
                    }
                }
                let mut obj = json!({ "provider": provider, "status": mapped, "state": st, "url": url, "message": msg, "vm_name": row.vm_name });
                if let Some(extra_obj) = extra.as_object() {
                    for (k, v) in extra_obj {
                        obj[k] = v.clone();
                    }
                }
                obj
            }
            None => ai_accounts::list_accounts(&state.conn, user.user_id)
                .into_iter()
                .find(|a| a.get("provider").and_then(Value::as_str) == Some(provider))
                .unwrap_or_else(|| json!({"provider": provider, "status": "unlinked"})),
        }
    } else {
        ai_accounts::list_accounts(&state.conn, user.user_id)
            .into_iter()
            .find(|a| a.get("provider").and_then(Value::as_str) == Some(provider))
            .unwrap_or_else(|| json!({"provider": provider, "status": "unlinked"}))
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
                    if let Err(e) = botagent::claude::logout(service, &row.vm_name).await {
                        log::warn!("Claude agent VM logout failed (status {})", e.0);
                    }
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
        ai_accounts::CODEX_CLI => {
            if let Some(service) = crate::core::bot::agent_vm_hook::service() {
                if let Ok(Some(row)) = botagent::vm::get_user_vm(service, &user.user_id) {
                    if let Err(e) = botagent::codex::logout(service, &row.vm_name).await {
                        log::warn!("Codex agent VM logout failed (status {})", e.0);
                    }
                }
            }
            ai_accounts::update_status(&state.conn, user.user_id, provider, "unlinked", None, None, None)
                .map_err(|error| err_db(locale, StatusCode::INTERNAL_SERVER_ERROR, error))?;
        }
        ai_accounts::ANTIGRAVITY => {
            if let Some(service) = crate::core::bot::agent_vm_hook::service() {
                if let Ok(Some(row)) = botagent::vm::get_user_vm(service, &user.user_id) {
                    if let Err(e) = botagent::antigravity::logout(service, &row.vm_name).await {
                        log::warn!("Antigravity agent VM logout failed (status {})", e.0);
                    }
                }
            }
            ai_accounts::update_status(&state.conn, user.user_id, provider, "unlinked", None, None, None)
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

//! `/api/ai-accounts` — per-user AI account status (#1654/#1656).
//!
//! Credentials never pass through this API: Claude stays in the agent VM and
//! ChatGPT tokens live in Vault. Connect/reconnect return an actionable error
//! until the provider-specific flows (#1652/#1653) are available.

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Json;
use axum::routing::{get, post};
use axum::{Extension, Router};
use diesel::prelude::*;
use diesel::sql_types::Text;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use botcore::shared::state::AppState;
use crate::security::auth_api::types::AuthenticatedUser;

use crate::core::ai_accounts;
use crate::core::ai_accounts_chatgpt;

type ApiError = (StatusCode, Json<Value>);

fn err(status: StatusCode, message: &str) -> ApiError {
    (status, Json(json!({ "error": message })))
}

fn provider_or_error(value: &str) -> Result<&'static str, ApiError> {
    ai_accounts::normalize_provider(value)
        .ok_or_else(|| err(StatusCode::BAD_REQUEST, "Unknown AI account provider"))
}

fn agent_service() -> Result<&'static Arc<botagent::AgentService>, ApiError> {
    crate::core::bot::agent_vm_hook::service()
        .ok_or_else(|| err(StatusCode::SERVICE_UNAVAILABLE, "The agent workspace is not available in this build"))
}

fn ensure_user(state: &Arc<AppState>, user: &AuthenticatedUser) -> Result<(), ApiError> {
    ai_accounts::ensure_user(&state.conn, user.user_id, user.email.as_deref())
        .map_err(|error| err(StatusCode::INTERNAL_SERVER_ERROR, &error))
}

/// First bot is the desktop default; the VM is per user, so any stable bot id
/// works for provisioning.
fn first_bot_id(state: &Arc<AppState>) -> Result<Uuid, ApiError> {
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

async fn ensure_vm(
    state: &Arc<AppState>,
    user_id: Uuid,
) -> Result<botagent::models::AgentSessionRow, ApiError> {
    let service = agent_service()?;
    let bot_id = first_bot_id(state)?;
    botagent::vm::ensure_vm(service, &user_id, &bot_id, None)
        .await
        .map_err(|(status, message)| err(status, &message))
}

fn map_claude_state(state: &str) -> &'static str {
    match state {
        "linked" => "linked",
        "waiting" | "starting" | "verifying" => "connecting",
        "error" => "error",
        _ => "unlinked",
    }
}

async fn connect_claude(
    state: &Arc<AppState>,
    user_id: Uuid,
) -> Result<Json<Value>, ApiError> {
    let service = agent_service()?;
    let row = ensure_vm(state, user_id).await?;
    let status = botagent::claude::start_login(service, &row.vm_name)
        .await
        .map_err(|(code, message)| err(code, &message))?;
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
        .map_err(|error| err(StatusCode::INTERNAL_SERVER_ERROR, &error))?;
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
        .map_err(|error| err(StatusCode::INTERNAL_SERVER_ERROR, &error))?;
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

async fn connect_chatgpt(
    state: &Arc<AppState>,
    user_id: Uuid,
) -> Result<Json<Value>, ApiError> {
    ai_accounts_chatgpt::start_connect(&state.conn, user_id)
        .await
        .map_err(|(code, message)| err(code, &message))
}

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
    Path(provider): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let provider = provider_or_error(&provider)?;
    ensure_user(&state, &user)?;
    match provider {
        ai_accounts::CLAUDE_CODE => connect_claude(&state, user.user_id).await,
        ai_accounts::CHATGPT_PLAN => connect_chatgpt(&state, user.user_id).await,
        _ => Err(err(StatusCode::BAD_REQUEST, "Unknown AI account provider")),
    }
}

/// POST /api/ai-accounts/:provider/reconnect
pub async fn reconnect(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(provider): Path<String>,
) -> Result<Json<Value>, ApiError> {
    // Reconnect is the same flow; the provider decides whether an existing
    // session is still usable and returns it unchanged.
    connect(State(state), Extension(user), Path(provider)).await
}

/// POST /api/ai-accounts/:provider/status
pub async fn status(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(provider): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let provider = provider_or_error(&provider)?;
    ensure_user(&state, &user)?;
    let account = if provider == ai_accounts::CLAUDE_CODE {
        let vm = botagent::vm::get_user_vm(
            agent_service()?,
            &user.user_id,
        )
        .map_err(|(code, message)| err(code, &message))?;
        match vm {
            Some(row) => {
                let status = botagent::claude::status(agent_service()?, &row.vm_name)
                    .await
                    .map_err(|(code, message)| err(code, &message))?;
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
    Path(provider): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let provider = provider_or_error(&provider)?;
    ensure_user(&state, &user)?;
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
            .map_err(|error| {
                log::warn!("ai accounts: disconnect failed: {error}");
                err(StatusCode::INTERNAL_SERVER_ERROR, "Could not disconnect the account")
            })?;
        }
        ai_accounts::CHATGPT_PLAN => {
            ai_accounts_chatgpt::disconnect(&state.conn, user.user_id)
                .await
                .map_err(|error| {
                    log::warn!("ai accounts: chatgpt disconnect failed: {error}");
                    err(StatusCode::INTERNAL_SERVER_ERROR, "Could not disconnect the account")
                })?;
        }
        _ => return Err(err(StatusCode::BAD_REQUEST, "Unknown AI account provider")),
    }
    Ok(Json(json!({ "provider": provider, "status": "unlinked" })))
}

#[derive(Deserialize)]
pub struct ClaudeCodeBody {
    pub code: String,
}

/// POST /api/ai-accounts/claude-code/code — forwards the pasted OAuth code.
pub async fn claude_code(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(body): Json<ClaudeCodeBody>,
) -> Result<Json<Value>, ApiError> {
    let service = agent_service()?;
    ensure_user(&state, &user)?;
    let row = ensure_vm(&state, user.user_id).await?;
    let status = botagent::claude::submit_code(service, &row.vm_name, &body.code)
        .await
        .map_err(|(code, message)| err(code, &message))?;
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
        .map_err(|error| err(StatusCode::INTERNAL_SERVER_ERROR, &error))?;
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
        .map_err(|error| err(StatusCode::INTERNAL_SERVER_ERROR, &error))?;
    }
    Ok(Json(json!({
        "provider": ai_accounts::CLAUDE_CODE,
        "state": status.state,
        "status": map_claude_state(&status.state),
        "message": status.message,
    })))
}

#[derive(Deserialize)]
pub struct UseForVibeBody {
    pub bot_id: Uuid,
    #[serde(default = "default_agent")]
    pub agent: String,
}

fn default_agent() -> String {
    "agentic".to_string()
}

const VIBE_AGENTS: [&str; 3] = ["reasoning", "agentic", "fast"];

/// POST /api/ai-accounts/:provider/use-for-vibe
///
/// Assigns a linked subscription account to one Vibe agent slot. Only the
/// provider slug and model are written; the account token is resolved per run,
/// so no credential ever lands in the shared bot configuration.
pub async fn use_for_vibe(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(provider): Path<String>,
    Json(body): Json<UseForVibeBody>,
) -> Result<Json<Value>, ApiError> {
    let provider = provider_or_error(&provider)?;
    ensure_user(&state, &user)?;
    if !VIBE_AGENTS.contains(&body.agent.as_str()) {
        return Err(err(StatusCode::BAD_REQUEST, "agent must be reasoning, agentic or fast"));
    }
    let Some(linked) = ai_accounts::linked_account(&state.conn, user.user_id, provider) else {
        return Err(err(
            StatusCode::CONFLICT,
            "Connect the account before assigning it to Vibe.",
        ));
    };
    let model = ai_accounts::default_model(provider).to_string();
    let config = botcore::config::ConfigManager::new(state.conn.clone());
    let prefix = format!("vibe-llm-{}-", body.agent);
    for (key, value) in [
        (format!("{prefix}provider"), provider.to_string()),
        (format!("{prefix}model"), model.clone()),
    ] {
        config.set_config(&body.bot_id, &key, &value).map_err(|error| {
            log::warn!("ai accounts: cannot persist {key}: {error}");
            err(StatusCode::INTERNAL_SERVER_ERROR, "Could not save the Vibe assignment")
        })?;
    }
    // Remove any credential/url from a previous provider in this slot.
    for key in [format!("{prefix}url"), format!("{prefix}key")] {
        if let Err(error) = config.set_config(&body.bot_id, &key, "") {
            log::warn!("ai accounts: cannot clear {key}: {error}");
        }
    }
    Ok(Json(json!({
        "success": true,
        "provider": provider,
        "model": model,
        "agent": body.agent,
        "account_email": linked.account_email,
        "plan": linked.plan,
    })))
}

/// POST /api/ai-accounts/:provider/use-for-chat
///
/// Selects a linked subscription as the bot's chat provider. Only provider and
/// model are written; any previous HTTP endpoint/key is cleared so the request
/// cannot accidentally use a different account.
pub async fn use_for_chat(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(provider): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let provider = provider_or_error(&provider)?;
    ensure_user(&state, &user)?;
    let Some(_linked) = ai_accounts::linked_account(&state.conn, user.user_id, provider) else {
        return Err(err(
            StatusCode::CONFLICT,
            "Connect the account before selecting it for chat.",
        ));
    };
    let bot_id = first_bot_id(&state)?;
    let model = ai_accounts::default_model(provider).to_string();
    let config = botcore::config::ConfigManager::new(state.conn.clone());
    for (key, value) in [
        ("llm-provider".to_string(), provider.to_string()),
        ("llm-model".to_string(), model.clone()),
    ] {
        config.set_config(&bot_id, &key, &value).map_err(|error| {
            log::warn!("ai accounts: cannot persist {key}: {error}");
            err(StatusCode::INTERNAL_SERVER_ERROR, "Could not select the chat provider")
        })?;
    }
    for key in [
        "llm-url",
        "llm-key",
        "llm-model-fast",
        "llm-model-balanced",
        "llm-model-quality",
    ] {
        if let Err(error) = config.set_config(&bot_id, key, "") {
            log::warn!("ai accounts: cannot clear {key}: {error}");
        }
    }
    Ok(Json(json!({
        "success": true,
        "provider": provider,
        "model": model,
        "bot_id": bot_id,
    })))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/ai-accounts", get(list))
        .route("/api/ai-accounts/:provider/connect", post(connect))
        .route("/api/ai-accounts/:provider/reconnect", post(reconnect))
        .route("/api/ai-accounts/:provider/status", post(status))
        .route("/api/ai-accounts/:provider/disconnect", post(disconnect))
        .route("/api/ai-accounts/:provider/use-for-vibe", post(use_for_vibe))
        .route("/api/ai-accounts/:provider/use-for-chat", post(use_for_chat))
        .route("/api/ai-accounts/claude-code/code", post(claude_code))
}

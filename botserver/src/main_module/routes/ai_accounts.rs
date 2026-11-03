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
use serde_json::{json, Value};

use botcore::shared::state::AppState;
use crate::security::auth_api::types::AuthenticatedUser;

use crate::core::ai_accounts;

type ApiError = (StatusCode, Json<Value>);

fn err(status: StatusCode, message: &str) -> ApiError {
    (status, Json(json!({ "error": message })))
}

fn provider_or_error(value: &str) -> Result<&'static str, ApiError> {
    ai_accounts::normalize_provider(value)
        .ok_or_else(|| err(StatusCode::BAD_REQUEST, "Unknown AI account provider"))
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
    Path(provider): Path<String>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<Value>, ApiError> {
    let provider = provider_or_error(&provider)?;
    let _ = user;
    let message = match provider {
        ai_accounts::CLAUDE_CODE => {
            "Claude Code login requires the agent VM broker: open Settings → AI accounts on a machine with a running agent VM."
        }
        _ => "Sign in with ChatGPT requires the loopback OAuth flow; configure it on desktop with an OpenAI client.",
    };
    Err(err(StatusCode::CONFLICT, message))
}

/// POST /api/ai-accounts/:provider/reconnect
pub async fn reconnect(
    Path(provider): Path<String>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<Value>, ApiError> {
    let provider = provider_or_error(&provider)?;
    let _ = user;
    Err(err(
        StatusCode::CONFLICT,
        match provider {
            ai_accounts::CLAUDE_CODE => "Claude Code reconnect requires the agent VM login broker.",
            _ => "ChatGPT reconnect requires the loopback OAuth flow.",
        },
    ))
}

/// POST /api/ai-accounts/:provider/status
pub async fn status(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(provider): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let provider = provider_or_error(&provider)?;
    let accounts = ai_accounts::list_accounts(&state.conn, user.user_id);
    let account = accounts
        .into_iter()
        .find(|row| row.get("provider").and_then(Value::as_str) == Some(provider))
        .unwrap_or_else(|| json!({ "provider": provider, "status": "unlinked" }));
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
    ai_accounts::update_status(
        &state.conn,
        user.user_id,
        provider,
        "unlinked",
        None,
        None,
        None,
    )
    .map_err(|error| {
        log::warn!("ai accounts: disconnect failed: {error}");
        err(StatusCode::INTERNAL_SERVER_ERROR, "Could not disconnect the account")
    })?;
    Ok(Json(json!({ "provider": provider, "status": "unlinked" })))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/ai-accounts", get(list))
        .route("/api/ai-accounts/:provider/connect", post(connect))
        .route("/api/ai-accounts/:provider/reconnect", post(reconnect))
        .route("/api/ai-accounts/:provider/status", post(status))
        .route("/api/ai-accounts/:provider/disconnect", post(disconnect))
}

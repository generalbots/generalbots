//! Computer-use HTTP surface (GB7-19 / GB7-20).
//!
//! `POST /api/computer/actions` lets the assistant (or a tool) propose a
//! browser or terminal action for a conversation. The suite polls
//! `/api/computer/actions/pending`, renders the consent card, executes the
//! action and posts the outcome to `/api/computer/actions/:id/result`.
//!
//! `POST /api/terminal/exec` is the server-side executor for terminal actions:
//! it refuses to run without an explicit approval flag, refuses destructive
//! patterns, and always executes inside the user's own agent container — never
//! on the bot host.
//!
//! The HTTP handlers live in `actions` (browser consent surface), `results`
//! (execution outcome + audit) and `terminal` (server-side executor); `audit`,
//! `page` and `terminal_rules` hold the supporting pieces.

mod actions;
mod audit;
mod page;
mod results;
mod terminal;
mod terminal_rules;

pub use actions::{
    approve_action, create_action, execute_action, get_action, pending, ApproveActionRequest,
    CreateActionRequest, ExecuteActionRequest, PendingQuery,
};
pub use audit::browser_audit;
pub use audit::spawn_terminal_audit_purge;
pub use audit::terminal_audit;
pub use results::submit_result;
pub use terminal::{terminal_exec, TerminalExecRequest};

use std::sync::Arc;

use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use botcore::shared::state::AppState;
use serde_json::Value;
use uuid::Uuid;

type ApiError = (StatusCode, Json<Value>);

/// Localized API error (#2094) — `code` stays machine-readable.
fn api_error(
    locale: &botcore::i18n::Locale,
    status: StatusCode,
    key: &str,
) -> ApiError {
    (status, Json(botcore::i18n::error_body(locale, key)))
}

/// Internal failure: the cause is logged, the client gets a generic localized
/// message — no internal error strings leak to the caller.
fn api_error_db<E: std::fmt::Display>(
    locale: &botcore::i18n::Locale,
    status: StatusCode,
    error: E,
) -> ApiError {
    log::warn!("api error ({}) at {}: {error}", status.as_u16(), module_path!());
    (
        status,
        Json(botcore::i18n::error_body(locale, "error-db-unavailable")),
    )
}

fn resolve_user(headers: &HeaderMap) -> Result<Uuid, ApiError> {
    let locale = botcore::i18n::locale_from_headers(headers);
    let token = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or_else(|| {
            api_error(
                &locale,
                StatusCode::UNAUTHORIZED,
                "error-auth-token-missing",
            )
        })?;
    let (claims_user, _roles) =
        crate::main_module::routes::anonymous_auth::resolve_token_user(token).ok_or_else(|| {
            api_error(
                &locale,
                StatusCode::UNAUTHORIZED,
                "error-auth-token-invalid",
            )
        })?;
    Ok(crate::security::user_role::derive_stable_user_uuid(
        &claims_user,
    ))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/computer/actions", post(create_action))
        .route("/api/computer/actions/pending", get(pending))
        .route("/api/computer/actions/:id", get(get_action))
        .route("/api/computer/actions/:id/approve", post(approve_action))
        .route("/api/computer/actions/:id/execute", post(execute_action))
        .route("/api/computer/actions/:id/result", post(submit_result))
        .route("/api/terminal/exec", post(terminal_exec))
        .route("/api/terminal/audit", get(terminal_audit))
        .route("/api/browser/audit", get(browser_audit))
}

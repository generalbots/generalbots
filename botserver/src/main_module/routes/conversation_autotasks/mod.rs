//! Conversation-scoped autotasks (#1613).
//!
//! Long-running work started from a conversation — "remind me every Monday",
//! "keep this service running" — is persisted as a first-class row and mirrored
//! to a cron entry or a systemd unit inside the user's agent workspace. The row
//! is the source of truth the chat reads; the workspace entry is the live
//! execution. Reconciliation flips a row to `orphaned` when the workspace no
//! longer knows about the job.
//!
//! `handlers` is the HTTP surface, `store` reads rows, `types` the row shape and
//! summaries, `reconcile` the boot sweep and `workspace` the VM mirror.

mod handlers;
mod reconcile;
mod store;
mod types;
mod workspace;

pub use handlers::{cancel, create, delete, list, list_all};
pub use reconcile::reconcile_on_boot;
pub(super) use types::AutotaskRow;
pub use types::CreateAutotaskRequest;

use std::sync::Arc;

use axum::extract::Json;
use axum::http::StatusCode;
use axum::routing::{delete as delete_route, get, post};
use axum::Router;
use botcore::shared::state::AppState;
use serde_json::Value;
use uuid::Uuid;

pub(super) const STATUS_SCHEDULED: &str = "scheduled";
pub(super) const STATUS_RUNNING: &str = "running";
const STATUS_STOPPED: &str = "stopped";
pub(super) const STATUS_ORPHANED: &str = "orphaned";
pub(super) const STATUS_FAILED: &str = "failed";
pub(super) const STATUS_PENDING: &str = "pending";

pub(super) type ApiError = (StatusCode, Json<Value>);

/// Localized API error (#2094) — `code` stays machine-readable.
pub(super) fn api_error(
    locale: &botcore::i18n::Locale,
    status: StatusCode,
    key: &str,
) -> ApiError {
    (status, Json(botcore::i18n::error_body(locale, key)))
}

/// Internal failure: the cause is logged, the client gets a generic localized
/// message — no internal error strings leak to the caller.
pub(super) fn api_error_db<E: std::fmt::Display>(
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

fn resolve_user(headers: &axum::http::HeaderMap) -> Result<Uuid, ApiError> {
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

/// Routes are merged into the API router whose state is `Arc<AppState>`.
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/api/conversations/:conversation_id/autotasks",
            get(list).post(create),
        )
        .route("/api/conversation-autotasks", get(list_all))
        .route("/api/conversation-autotasks/:id/cancel", post(cancel))
        .route("/api/conversation-autotasks/:id", delete_route(delete))
}

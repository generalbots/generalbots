use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Extension, Json, Router};
use futures_util::StreamExt;
use serde::Deserialize;
use uuid::Uuid;

use crate::security::auth_api::types::AuthenticatedUser;

use super::{sanitize_container_cwd, TerminalManager};
use botcore::shared::utils::DbPool;

mod authz;
mod handlers;

#[derive(Deserialize)]
pub struct CreateTerminalRequest {
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub shell: Option<String>,
    /// Optional project VM container to exec into (e.g. `calculator-prod`).
    /// When set the shell runs inside that container via `incus exec`.
    #[serde(default)]
    pub container: Option<String>,
}

#[derive(Deserialize)]
pub struct KillTerminalRequest {
    pub id: String,
}

pub struct TerminalRouterState {
    pub manager: Arc<TerminalManager>,
    pub pool: DbPool,
}

pub fn configure_terminal_routes(pool: DbPool) -> Router {
    let manager = Arc::new(TerminalManager::new());
    Router::new()
        .route("/api/terminal/create", post(handlers::create_terminal))
        .route("/api/terminal/list", get(handlers::list_terminals))
        .route("/api/terminal/kill", post(handlers::kill_terminal))
        .route("/api/terminal/ws", get(handlers::terminal_ws))
        .with_state(Arc::new(TerminalRouterState { manager, pool }))
}

/// Routes for builds without the `terminal` feature.
///
/// The API fallback answered a generic `{"error":"not_found"}`, which the
/// terminal client rendered as "Terminal session failed: not_found" (#1841).
/// Advertising the missing capability with an actionable message is better
/// than pretending the route never existed.
pub fn configure_unavailable_terminal_routes() -> Router {
    Router::new()
        .route("/api/terminal/create", post(terminal_unavailable))
        .route("/api/terminal/list", get(terminal_unavailable))
        .route("/api/terminal/kill", post(terminal_unavailable))
        .route("/api/terminal/ws", get(terminal_unavailable))
}

async fn terminal_unavailable() -> impl IntoResponse {
    (
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({
            "error": "terminal_unavailable",
            "message": "Terminal is not available in this build. Install a build with the terminal feature (the desktop edition includes it) and try again."
        })),
    )
}

//! Tool-manifest discovery for the native surface tools (#1678 follow-up).
//!
//! `GET /api/runs/tools` answers with MCP-spec manifests (`name`, `title`,
//! `description`, `inputSchema`, `annotations`) generated from the single
//! declarative seed in `pipeline::surface_catalog`. Hosts, the toolkit and
//! the command catalogue consume the response instead of parsing prompts.

use std::sync::Arc;

use axum::{response::IntoResponse, routing::get, Json, Router};
use botcore::shared::state::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/api/runs/tools", get(tool_manifests))
}

async fn tool_manifests() -> impl IntoResponse {
    Json(super::super::pipeline::surface_catalog::manifests())
}

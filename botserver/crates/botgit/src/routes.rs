use axum::routing::{get, post};
use axum::Router;
use std::sync::Arc;

use botcore::shared::state::AppState;

use crate::handlers;

pub fn configure() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/git/status", get(handlers::git_status))
        .route("/api/git/diff/{file}", get(handlers::git_diff))
        .route("/api/git/commit", post(handlers::git_commit))
        .route("/api/git/push", post(handlers::git_push))
        .route("/api/git/branches", get(handlers::git_branches))
        .route("/api/git/branch/{name}", post(handlers::git_create_or_switch_branch))
        .route("/api/git/log", get(handlers::git_log))
}

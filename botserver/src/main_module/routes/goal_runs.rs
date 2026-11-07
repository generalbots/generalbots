//! Long-horizon goal runs (#1705).
//!
//! `POST /api/goals/runs` starts a bounded background runner for one key
//! result: every `interval_seconds` it advances the value by `step_value`
//! (clamped at the target), writes an `okr_checkins` row and updates the run.
//! It stops at the target, after `MAX_CHECKPOINTS`, or on `/stop`.
//!
//! Arithmetic lives in the tested `runs::goal_runner` module. This module keeps
//! the router, `handlers` (HTTP), `runner` (background loop), `advance` (one
//! checkpoint applied to the database) and the shared `types` apart.

mod advance;
mod handlers;
mod runner;
mod types;

use std::sync::Arc;

use axum::routing::post;
use axum::Router;
use botcore::shared::state::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/api/goals/runs",
            post(handlers::create).get(handlers::list),
        )
        .route("/api/goals/runs/:id/tick", post(handlers::tick_now))
        .route("/api/goals/runs/:id/stop", post(handlers::stop))
        .route("/api/goals/runs/:id/pause", post(handlers::pause))
        .route("/api/goals/runs/:id/resume", post(handlers::resume))
}

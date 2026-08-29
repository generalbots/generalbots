use axum::routing::{get, post};
use axum::Router;

use crate::handlers;

pub fn configure<S: Clone + Send + Sync + 'static>() -> Router<S> {
    Router::new()
        .route("/api/handoff/queue", get(handlers::list_queue))
        .route("/api/handoff/transfer/:id", post(handlers::transfer_item))
        .route("/api/handoff/analytics", get(handlers::get_analytics))
        .route("/api/handoff/channels", get(handlers::list_channels))
        .route("/api/handoff/csat", get(handlers::list_csat))
        .route("/api/handoff/agents", get(handlers::list_agents).post(handlers::create_agent))
        .route("/api/handoff/transcripts", get(handlers::list_transcripts))
        .route("/api/handoff/sla", get(handlers::get_sla))
        .route("/api/handoff/deflection", get(handlers::list_deflection))
}

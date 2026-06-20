use axum::routing::{get, post};
use axum::Router;
use crate::handlers;

pub fn configure<S: Clone + Send + Sync + 'static>() -> Router<S> {
    Router::new()
        .route("/api/vision/analyze", post(handlers::analyze_image))
        .route("/api/vision/history", get(handlers::list_history))
}

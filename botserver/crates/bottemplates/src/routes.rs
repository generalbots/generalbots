use axum::routing::{get, post};
use axum::Router;
use crate::handlers;

pub fn configure<S: Clone + Send + Sync + 'static>() -> Router<S> {
    Router::new()
        .route("/api/templates/list", get(handlers::list_templates))
        .route("/api/templates/preview/:id", get(handlers::preview_template))
        .route("/api/templates/deploy/:id", post(handlers::deploy_template))
}

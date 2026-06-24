use axum::routing::{get, post};
use axum::Router;
use crate::handlers;

pub fn configure<S: Clone + Send + Sync + 'static>() -> Router<S> {
    Router::new()
        .route("/api/integrations/connectors", get(handlers::list_connectors))
        .route("/api/integrations/connectors/{id}/connect", post(handlers::connect_connector))
        .route("/api/integrations/connectors/{id}/disconnect", post(handlers::disconnect_connector))
        .route("/api/integrations/etl", get(handlers::list_etl))
}

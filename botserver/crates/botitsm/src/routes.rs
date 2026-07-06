use axum::routing::{get, post, put};
use axum::Router;

use crate::handlers;

pub fn configure<S: Clone + Send + Sync + 'static>() -> Router<S> {
    Router::new()
        .route("/api/itsm/incidents", get(handlers::list_incidents))
        .route("/api/itsm/incidents", post(handlers::create_incident))
        .route("/api/itsm/incidents/{id}", put(handlers::update_incident))
        .route("/api/itsm/requests", get(handlers::list_requests))
        .route("/api/itsm/requests", post(handlers::create_request))
        .route("/api/itsm/cmdb", get(handlers::list_cmdb))
        .route("/api/itsm/kb", get(handlers::list_kb))
}

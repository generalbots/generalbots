use axum::routing::get;
use axum::Router;
use crate::handlers;

pub fn configure<S: Clone + Send + Sync + 'static>() -> Router<S> {
    Router::new()
        .route("/api/erp/financial", get(handlers::get_financial))
        .route("/api/erp/inventory", get(handlers::list_inventory))
        .route("/api/erp/procurement", get(handlers::list_procurement))
        .route("/api/erp/branches", get(handlers::list_branches))
}

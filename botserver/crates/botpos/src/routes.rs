use axum::routing::{get, post};
use axum::Router;

use crate::handlers;

pub fn configure<S: Clone + Send + Sync + 'static>() -> Router<S> {
    Router::new()
        .route("/api/pos/products", get(handlers::list_products))
        .route("/api/pos/products", post(handlers::create_product))
        .route("/api/pos/orders", get(handlers::list_orders))
        .route("/api/pos/orders", post(handlers::create_order))
        .route("/api/pos/orders/{id}", get(handlers::get_order))
}

use axum::{routing::get, Json, Router};
use std::sync::Arc;

pub fn configure_retail_routes() -> Router<Arc<RetailState>> {
    Router::new()
        .route("/api/retail/branches", get(list_branches))
        .route("/api/retail/stock", get(list_stock))
        .route("/api/retail/promotions", get(list_promotions))
        .route("/api/retail/suppliers", get(list_suppliers))
        .route("/api/retail/top-products", get(list_top_products))
}

pub struct RetailState;

async fn list_branches() -> Json<serde_json::Value> {
    Json(serde_json::json!([]))
}

async fn list_stock() -> Json<serde_json::Value> {
    Json(serde_json::json!([]))
}

async fn list_promotions() -> Json<serde_json::Value> {
    Json(serde_json::json!([]))
}

async fn list_suppliers() -> Json<serde_json::Value> {
    Json(serde_json::json!([]))
}

async fn list_top_products() -> Json<serde_json::Value> {
    Json(serde_json::json!([]))
}

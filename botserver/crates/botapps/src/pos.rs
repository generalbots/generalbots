use axum::{extract::{State, Json, Path}, routing::get, Router};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Product {
    pub id: Uuid,
    pub sku: String,
    pub name: String,
    pub description: String,
    pub price: f64,
    pub cost: f64,
    pub stock_quantity: i64,
    pub category: String,
    pub barcode: Option<String>,
    pub active: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrderItem {
    pub product_id: Uuid,
    pub product_name: String,
    pub quantity: i64,
    pub unit_price: f64,
    pub total: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PosOrder {
    pub id: Uuid,
    pub order_number: String,
    pub items: Vec<OrderItem>,
    pub subtotal: f64,
    pub tax: f64,
    pub total: f64,
    pub payment_method: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Default)]
pub struct PosState {
    pub products: HashMap<Uuid, Product>,
    pub orders: HashMap<Uuid, PosOrder>,
}

pub fn routes() -> axum::Router {
    let state = Arc::new(RwLock::new(PosState::default()));
    Router::new()
        .route("/api/pos/products", get(list_products).post(create_product))
        .route("/api/pos/products/{id}", get(get_product).put(update_product).delete(delete_product))
        .route("/api/pos/orders", get(list_orders).post(create_order))
        .route("/api/pos/orders/{id}", get(get_order).put(update_order))
        .with_state(state)
}

async fn list_products(State(state): State<Arc<RwLock<PosState>>>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    let items: Vec<&Product> = s.products.values().collect();
    Json(serde_json::json!({"products": items}))
}

async fn create_product(State(state): State<Arc<RwLock<PosState>>>, Json(mut product): Json<Product>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    let id = Uuid::new_v4();
    product.id = id;
    product.active = true;
    product.created_at = Utc::now().to_rfc3339();
    s.products.insert(id, product.clone());
    Json(serde_json::json!({"product": product}))
}

async fn get_product(State(state): State<Arc<RwLock<PosState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    match s.products.get(&id) {
        Some(p) => Json(serde_json::json!({"product": p})),
        None => Json(serde_json::json!({"error": "Product not found"})),
    }
}

async fn update_product(State(state): State<Arc<RwLock<PosState>>>, Path(id): Path<Uuid>, Json(product): Json<Product>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    if let Some(existing) = s.products.get_mut(&id) {
        *existing = product.clone();
        existing.id = id;
        Json(serde_json::json!({"product": existing.clone()}))
    } else {
        Json(serde_json::json!({"error": "Product not found"}))
    }
}

async fn delete_product(State(state): State<Arc<RwLock<PosState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    s.products.remove(&id);
    Json(serde_json::json!({"deleted": true}))
}

async fn list_orders(State(state): State<Arc<RwLock<PosState>>>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    let items: Vec<&PosOrder> = s.orders.values().collect();
    Json(serde_json::json!({"orders": items}))
}

async fn create_order(State(state): State<Arc<RwLock<PosState>>>, Json(mut order): Json<PosOrder>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    let id = Uuid::new_v4();
    order.id = id;
    order.order_number = format!("ORD-{}", &id.to_string()[..8]);
    order.status = "Pending".to_string();
    order.created_at = Utc::now().to_rfc3339();
    s.orders.insert(id, order.clone());
    Json(serde_json::json!({"order": order}))
}

async fn get_order(State(state): State<Arc<RwLock<PosState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    match s.orders.get(&id) {
        Some(o) => Json(serde_json::json!({"order": o})),
        None => Json(serde_json::json!({"error": "Order not found"})),
    }
}

async fn update_order(State(state): State<Arc<RwLock<PosState>>>, Path(id): Path<Uuid>, Json(order): Json<PosOrder>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    if let Some(existing) = s.orders.get_mut(&id) {
        *existing = order.clone();
        existing.id = id;
        Json(serde_json::json!({"order": existing.clone()}))
    } else {
        Json(serde_json::json!({"error": "Order not found"}))
    }
}

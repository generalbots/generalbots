use axum::extract::{Json, Path};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, OnceLock};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub sku: String,
    pub price: f64,
    pub stock: u64,
    pub category: String,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrderItem {
    pub product_id: String,
    pub quantity: u64,
    pub unit_price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Order {
    pub id: String,
    pub items: Vec<OrderItem>,
    pub total: f64,
    pub status: String,
    pub payment_method: String,
    pub created_at: String,
}

#[derive(Default)]
struct AppState {
    products: HashMap<String, Product>,
    orders: HashMap<String, Order>,
}

fn state() -> &'static Arc<RwLock<AppState>> {
    static S: OnceLock<Arc<RwLock<AppState>>> = OnceLock::new();
    S.get_or_init(|| Arc::new(RwLock::new(AppState::default())))
}

pub async fn list_products() -> Json<serde_json::Value> {
    let s = state().read().unwrap();
    let items: Vec<&Product> = s.products.values().collect();
    Json(serde_json::json!({"items": items}))
}

pub async fn create_product(Json(item): Json<Product>) -> Json<serde_json::Value> {
    let mut s = state().write().unwrap();
    let id = uuid::Uuid::new_v4().to_string();
    let mut new_item = item;
    new_item.id = id.clone();
    new_item.active = true;
    s.products.insert(id.clone(), new_item.clone());
    Json(serde_json::json!({"item": new_item}))
}

pub async fn list_orders() -> Json<serde_json::Value> {
    let s = state().read().unwrap();
    let items: Vec<&Order> = s.orders.values().collect();
    Json(serde_json::json!({"items": items}))
}

pub async fn create_order(Json(item): Json<Order>) -> Json<serde_json::Value> {
    let mut s = state().write().unwrap();
    let id = uuid::Uuid::new_v4().to_string();
    let mut new_item = item;
    new_item.id = id.clone();
    new_item.created_at = chrono::Utc::now().to_rfc3339();
    new_item.status = "created".to_string();
    s.orders.insert(id.clone(), new_item.clone());
    Json(serde_json::json!({"item": new_item}))
}

pub async fn get_order(Path(id): Path<String>) -> Json<serde_json::Value> {
    let s = state().read().unwrap();
    match s.orders.get(&id) {
        Some(order) => Json(serde_json::json!({"item": order})),
        None => Json(serde_json::json!({"error": "Order not found"})),
    }
}

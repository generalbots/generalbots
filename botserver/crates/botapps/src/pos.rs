use std::sync::{Arc, RwLock};

use axum::extract::{Path, State as AxumState};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Default)]
pub struct Product {
    pub id: Uuid,
    pub name: String,
    pub price: f64,
    pub stock: i32,
    pub category: String,
}

#[derive(Default)]
pub struct OrderItem {
    pub product_id: Uuid,
    pub quantity: i32,
    pub price: f64,
}

#[derive(Default)]
pub struct PoOrder {
    pub id: Uuid,
    pub items: Vec<OrderItem>,
    pub total: f64,
    pub payment_method: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Default)]
pub struct CreateProduct {
    pub name: String,
    pub price: f64,
    pub stock: i32,
    pub category: String,
}

#[derive(Default)]
pub struct CreateOrderRequest {
    pub items: Vec<OrderItemRequest>,
    pub payment_method: String,
}

#[derive(Default)]
pub struct OrderItemRequest {
    pub product_id: Uuid,
    pub quantity: i32,
}

#[derive(Default)]
pub struct PosState {
    pub products: Arc<RwLock<Vec<Product>>>,
    pub orders: Arc<RwLock<Vec<PoOrder>>>,
}

impl PosState {
    pub fn new() -> Self {
        Self {
            products: Arc::new(RwLock::new(Vec::new())),
            orders: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: T,
}

async fn list_products(AxumState(state): AxumState<PosState>) -> Json<ApiResponse<Vec<Product>>> {
    let products = state.products.read().unwrap().clone();
    Json(ApiResponse { success: true, data: products })
}

async fn create_product(
    AxumState(state): AxumState<PosState>,
    Json(payload): Json<CreateProduct>,
) -> Json<ApiResponse<Product>> {
    let product = Product {
        id: Uuid::new_v4(),
        name: payload.name,
        price: payload.price,
        stock: payload.stock,
        category: payload.category,
    };
    state.products.write().unwrap().push(product.clone());
    Json(ApiResponse { success: true, data: product })
}

async fn create_order(
    AxumState(state): AxumState<PosState>,
    Json(payload): Json<CreateOrderRequest>,
) -> Json<ApiResponse<PoOrder>> {
    let products = state.products.read().unwrap();
    let mut order_items: Vec<OrderItem> = Vec::new();
    let mut total = 0.0;
    for item_req in &payload.items {
        if let Some(product) = products.iter().find(|p| p.id == item_req.product_id) {
            let item_total = product.price * item_req.quantity as f64;
            total += item_total;
            order_items.push(OrderItem {
                product_id: item_req.product_id,
                quantity: item_req.quantity,
                price: product.price,
            });
        }
    }
    drop(products);
    let order = PoOrder {
        id: Uuid::new_v4(),
        items: order_items,
        total,
        payment_method: payload.payment_method,
        created_at: Utc::now(),
    };
    state.orders.write().unwrap().push(order.clone());
    Json(ApiResponse { success: true, data: order })
}

async fn get_order(
    AxumState(state): AxumState<PosState>,
    Path(id): Path<Uuid>,
) -> Json<ApiResponse<PoOrder>> {
    let orders = state.orders.read().unwrap();
    let order = orders.iter().find(|o| o.id == id).expect("Order not found").clone();
    Json(ApiResponse { success: true, data: order })
}

async fn list_orders(AxumState(state): AxumState<PosState>) -> Json<ApiResponse<Vec<PoOrder>>> {
    let orders = state.orders.read().unwrap().clone();
    Json(ApiResponse { success: true, data: orders })
}

pub fn routes() -> Router {
    let state = PosState::new();
    Router::new()
        .route("/api/pos/products", get(list_products).post(create_product))
        .route("/api/pos/orders", get(list_orders).post(create_order))
        .route("/api/pos/orders/{id}", get(get_order))
        .with_state(state)
}

use axum::{extract::State, routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use uuid::Uuid;

#[derive(Default)]
pub struct ErpFinancial {
    pub total_revenue: f64,
    pub total_expenses: f64,
    pub net_profit: f64,
    pub pending_invoices: usize,
    pub overdue_invoices: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryItem {
    pub id: Uuid,
    pub product_name: String,
    pub quantity: i32,
    pub reorder_point: i32,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseOrderItem {
    pub product_name: String,
    pub quantity: i32,
    pub unit_price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseOrder {
    pub id: Uuid,
    pub supplier: String,
    pub items: Vec<PurchaseOrderItem>,
    pub total: f64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    pub id: Uuid,
    pub name: String,
    pub address: String,
    pub manager: String,
    pub active: bool,
}

#[derive(Debug, Default)]
pub struct ErpState {
    pub financial: ErpFinancial,
    pub inventory: Vec<InventoryItem>,
    pub purchase_orders: Vec<PurchaseOrder>,
    pub branches: Vec<Branch>,
}



pub fn create_erp_state() -> SharedErpState {
    Arc::new(RwLock::new(ErpState {
        financial: ErpFinancial {
            total_revenue: 0.0,
            total_expenses: 0.0,
            net_profit: 0.0,
            pending_invoices: 0,
            overdue_invoices: 0,
        },
        inventory: Vec::new(),
        purchase_orders: Vec::new(),
        branches: Vec::new(),
    }))
}

async fn get_financial(
    State(state): State<SharedErpState>,
) -> Result<Json<ErpFinancial>, axum::http::StatusCode> {
    let data = state.read().map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(data.financial.clone()))
}

async fn get_inventory(
    State(state): State<SharedErpState>,
) -> Result<Json<Vec<InventoryItem>>, axum::http::StatusCode> {
    let data = state.read().map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(data.inventory.clone()))
}

async fn get_procurement(
    State(state): State<SharedErpState>,
) -> Result<Json<Vec<PurchaseOrder>>, axum::http::StatusCode> {
    let data = state.read().map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(data.purchase_orders.clone()))
}

async fn get_branches(
    State(state): State<SharedErpState>,
) -> Result<Json<Vec<Branch>>, axum::http::StatusCode> {
    let data = state.read().map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(data.branches.clone()))
}

pub fn routes() -> Router {
    let state = std::sync::Arc::new(std::sync::RwLock::new(Default::default()));
    Router::new()
        .route("/api/erp/financial", get(get_financial))
        .route("/api/erp/inventory", get(get_inventory))
        .route("/api/erp/procurement", get(get_procurement))
        .route("/api/erp/branches", get(get_branches))
        .with_state(state)
}

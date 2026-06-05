use axum::extract::Json;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, OnceLock};
use axum::http::StatusCode;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FinancialEntry {
    pub id: String,
    pub kind: String,
    pub description: String,
    pub amount: f64,
    pub category: String,
    pub date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InventoryItem {
    pub id: String,
    pub sku: String,
    pub name: String,
    pub quantity: u64,
    pub unit_cost: f64,
    pub location: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProcurementOrder {
    pub id: String,
    pub supplier: String,
    pub items: Vec<String>,
    pub total: f64,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Branch {
    pub id: String,
    pub name: String,
    pub address: String,
    pub manager: String,
    pub active: bool,
}

#[derive(Default)]
struct AppState {
    financial: HashMap<String, FinancialEntry>,
    inventory: HashMap<String, InventoryItem>,
    procurement: HashMap<String, ProcurementOrder>,
    branches: HashMap<String, Branch>,
}

fn state() -> &'static Arc<RwLock<AppState>> {
    static S: OnceLock<Arc<RwLock<AppState>>> = OnceLock::new();
    S.get_or_init(|| Arc::new(RwLock::new(AppState::default())))
}

pub async fn get_financial() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let s = state().read().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {}", e)))?;
    let items: Vec<&FinancialEntry> = s.financial.values().collect();
    Ok(Json(serde_json::json!({"items": items})))
}

pub async fn list_inventory() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let s = state().read().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {}", e)))?;
    let items: Vec<&InventoryItem> = s.inventory.values().collect();
    Ok(Json(serde_json::json!({"items": items})))
}

pub async fn list_procurement() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let s = state().read().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {}", e)))?;
    let items: Vec<&ProcurementOrder> = s.procurement.values().collect();
    Ok(Json(serde_json::json!({"items": items})))
}

pub async fn list_branches() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let s = state().read().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {}", e)))?;
    let items: Vec<&Branch> = s.branches.values().collect();
    Ok(Json(serde_json::json!({"items": items})))
}

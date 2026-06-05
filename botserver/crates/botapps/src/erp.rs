use axum::extract::Json;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, OnceLock};

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

pub async fn get_financial() -> Json<serde_json::Value> {
    let s = state().read().unwrap();
    let items: Vec<&FinancialEntry> = s.financial.values().collect();
    Json(serde_json::json!({"items": items}))
}

pub async fn list_inventory() -> Json<serde_json::Value> {
    let s = state().read().unwrap();
    let items: Vec<&InventoryItem> = s.inventory.values().collect();
    Json(serde_json::json!({"items": items}))
}

pub async fn list_procurement() -> Json<serde_json::Value> {
    let s = state().read().unwrap();
    let items: Vec<&ProcurementOrder> = s.procurement.values().collect();
    Json(serde_json::json!({"items": items}))
}

pub async fn list_branches() -> Json<serde_json::Value> {
    let s = state().read().unwrap();
    let items: Vec<&Branch> = s.branches.values().collect();
    Json(serde_json::json!({"items": items}))
}

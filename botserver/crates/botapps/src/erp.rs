use axum::{extract::{State, Json, Path}, routing::get, Router};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FinancialEntry {
    pub id: Uuid,
    pub entry_type: String,
    pub account: String,
    pub description: String,
    pub amount: f64,
    pub currency: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InventoryItem {
    pub id: Uuid,
    pub sku: String,
    pub name: String,
    pub quantity: i64,
    pub unit_price: f64,
    pub warehouse: String,
    pub reorder_level: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProcurementOrder {
    pub id: Uuid,
    pub order_number: String,
    pub supplier: String,
    pub items: String,
    pub total: f64,
    pub status: String,
    pub expected_delivery: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Branch {
    pub id: Uuid,
    pub name: String,
    pub code: String,
    pub address: String,
    pub city: String,
    pub state: String,
    pub manager: String,
    pub active: bool,
    pub created_at: String,
}

#[derive(Default)]
pub struct ErpState {
    pub financial_entries: HashMap<Uuid, FinancialEntry>,
    pub inventory_items: HashMap<Uuid, InventoryItem>,
    pub procurement_orders: HashMap<Uuid, ProcurementOrder>,
    pub branches: HashMap<Uuid, Branch>,
}

pub fn routes() -> axum::Router {
    let state = Arc::new(RwLock::new(ErpState::default()));
    Router::new()
        .route("/api/erp/financial", get(list_financial).post(create_financial))
        .route("/api/erp/financial/{id}", get(get_financial).put(update_financial).delete(delete_financial))
        .route("/api/erp/inventory", get(list_inventory).post(create_inventory))
        .route("/api/erp/inventory/{id}", get(get_inventory).put(update_inventory).delete(delete_inventory))
        .route("/api/erp/procurement", get(list_procurement).post(create_procurement))
        .route("/api/erp/procurement/{id}", get(get_procurement).put(update_procurement).delete(delete_procurement))
        .route("/api/erp/branches", get(list_branches).post(create_branch))
        .route("/api/erp/branches/{id}", get(get_branch).put(update_branch).delete(delete_branch))
        .with_state(state)
}

async fn list_financial(State(state): State<Arc<RwLock<ErpState>>>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    let items: Vec<&FinancialEntry> = s.financial_entries.values().collect();
    Json(serde_json::json!({"financial_entries": items}))
}

async fn create_financial(State(state): State<Arc<RwLock<ErpState>>>, Json(mut entry): Json<FinancialEntry>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    let id = Uuid::new_v4();
    entry.id = id;
    entry.status = "Pending".to_string();
    entry.created_at = Utc::now().to_rfc3339();
    s.financial_entries.insert(id, entry.clone());
    Json(serde_json::json!({"financial_entry": entry}))
}

async fn get_financial(State(state): State<Arc<RwLock<ErpState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    match s.financial_entries.get(&id) {
        Some(e) => Json(serde_json::json!({"financial_entry": e})),
        None => Json(serde_json::json!({"error": "Financial entry not found"})),
    }
}

async fn update_financial(State(state): State<Arc<RwLock<ErpState>>>, Path(id): Path<Uuid>, Json(entry): Json<FinancialEntry>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    if let Some(existing) = s.financial_entries.get_mut(&id) {
        *existing = entry.clone();
        existing.id = id;
        Json(serde_json::json!({"financial_entry": existing.clone()}))
    } else {
        Json(serde_json::json!({"error": "Financial entry not found"}))
    }
}

async fn delete_financial(State(state): State<Arc<RwLock<ErpState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    s.financial_entries.remove(&id);
    Json(serde_json::json!({"deleted": true}))
}

async fn list_inventory(State(state): State<Arc<RwLock<ErpState>>>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    let items: Vec<&InventoryItem> = s.inventory_items.values().collect();
    Json(serde_json::json!({"inventory_items": items}))
}

async fn create_inventory(State(state): State<Arc<RwLock<ErpState>>>, Json(mut item): Json<InventoryItem>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    let id = Uuid::new_v4();
    item.id = id;
    item.created_at = Utc::now().to_rfc3339();
    s.inventory_items.insert(id, item.clone());
    Json(serde_json::json!({"inventory_item": item}))
}

async fn get_inventory(State(state): State<Arc<RwLock<ErpState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    match s.inventory_items.get(&id) {
        Some(item) => Json(serde_json::json!({"inventory_item": item})),
        None => Json(serde_json::json!({"error": "Inventory item not found"})),
    }
}

async fn update_inventory(State(state): State<Arc<RwLock<ErpState>>>, Path(id): Path<Uuid>, Json(item): Json<InventoryItem>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    if let Some(existing) = s.inventory_items.get_mut(&id) {
        *existing = item.clone();
        existing.id = id;
        Json(serde_json::json!({"inventory_item": existing.clone()}))
    } else {
        Json(serde_json::json!({"error": "Inventory item not found"}))
    }
}

async fn delete_inventory(State(state): State<Arc<RwLock<ErpState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    s.inventory_items.remove(&id);
    Json(serde_json::json!({"deleted": true}))
}

async fn list_procurement(State(state): State<Arc<RwLock<ErpState>>>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    let items: Vec<&ProcurementOrder> = s.procurement_orders.values().collect();
    Json(serde_json::json!({"procurement_orders": items}))
}

async fn create_procurement(State(state): State<Arc<RwLock<ErpState>>>, Json(mut order): Json<ProcurementOrder>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    let id = Uuid::new_v4();
    order.id = id;
    order.status = "Draft".to_string();
    order.created_at = Utc::now().to_rfc3339();
    s.procurement_orders.insert(id, order.clone());
    Json(serde_json::json!({"procurement_order": order}))
}

async fn get_procurement(State(state): State<Arc<RwLock<ErpState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    match s.procurement_orders.get(&id) {
        Some(o) => Json(serde_json::json!({"procurement_order": o})),
        None => Json(serde_json::json!({"error": "Procurement order not found"})),
    }
}

async fn update_procurement(State(state): State<Arc<RwLock<ErpState>>>, Path(id): Path<Uuid>, Json(order): Json<ProcurementOrder>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    if let Some(existing) = s.procurement_orders.get_mut(&id) {
        *existing = order.clone();
        existing.id = id;
        Json(serde_json::json!({"procurement_order": existing.clone()}))
    } else {
        Json(serde_json::json!({"error": "Procurement order not found"}))
    }
}

async fn delete_procurement(State(state): State<Arc<RwLock<ErpState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    s.procurement_orders.remove(&id);
    Json(serde_json::json!({"deleted": true}))
}

async fn list_branches(State(state): State<Arc<RwLock<ErpState>>>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    let items: Vec<&Branch> = s.branches.values().collect();
    Json(serde_json::json!({"branches": items}))
}

async fn create_branch(State(state): State<Arc<RwLock<ErpState>>>, Json(mut branch): Json<Branch>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    let id = Uuid::new_v4();
    branch.id = id;
    branch.active = true;
    branch.created_at = Utc::now().to_rfc3339();
    s.branches.insert(id, branch.clone());
    Json(serde_json::json!({"branch": branch}))
}

async fn get_branch(State(state): State<Arc<RwLock<ErpState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    match s.branches.get(&id) {
        Some(b) => Json(serde_json::json!({"branch": b})),
        None => Json(serde_json::json!({"error": "Branch not found"})),
    }
}

async fn update_branch(State(state): State<Arc<RwLock<ErpState>>>, Path(id): Path<Uuid>, Json(branch): Json<Branch>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    if let Some(existing) = s.branches.get_mut(&id) {
        *existing = branch.clone();
        existing.id = id;
        Json(serde_json::json!({"branch": existing.clone()}))
    } else {
        Json(serde_json::json!({"error": "Branch not found"}))
    }
}

async fn delete_branch(State(state): State<Arc<RwLock<ErpState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    s.branches.remove(&id);
    Json(serde_json::json!({"deleted": true}))
}

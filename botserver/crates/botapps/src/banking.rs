use axum::{extract::{State, Json, Path}, routing::get, Router};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BankTransaction {
    pub id: Uuid,
    pub account_id: String,
    pub transaction_type: String,
    pub amount: f64,
    pub currency: String,
    pub description: String,
    pub counterparty: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BankingPlatform {
    pub id: Uuid,
    pub name: String,
    pub platform_type: String,
    pub api_endpoint: String,
    pub auth_type: String,
    pub status: String,
    pub last_sync_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReconciliationEntry {
    pub id: Uuid,
    pub bank_transaction_id: Uuid,
    pub internal_transaction_id: Uuid,
    pub match_status: String,
    pub matched_amount: f64,
    pub discrepancy: f64,
    pub notes: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BankReport {
    pub id: Uuid,
    pub report_type: String,
    pub period_start: String,
    pub period_end: String,
    pub total_inflow: f64,
    pub total_outflow: f64,
    pub balance: f64,
    pub status: String,
    pub created_at: String,
}

#[derive(Default)]
pub struct BankingState {
    pub transactions: HashMap<Uuid, BankTransaction>,
    pub platforms: HashMap<Uuid, BankingPlatform>,
    pub reconciliations: HashMap<Uuid, ReconciliationEntry>,
    pub reports: HashMap<Uuid, BankReport>,
}

pub fn routes() -> axum::Router {
    let state = Arc::new(RwLock::new(BankingState::default()));
    Router::new()
        .route("/api/banking/transactions", get(list_transactions).post(create_transaction))
        .route("/api/banking/transactions/{id}", get(get_transaction).put(update_transaction))
        .route("/api/banking/platforms", get(list_platforms).post(create_platform))
        .route("/api/banking/platforms/{id}", get(get_platform).put(update_platform).delete(delete_platform))
        .route("/api/banking/reconcile", get(list_reconciliations).post(create_reconciliation))
        .route("/api/banking/reconcile/{id}", get(get_reconciliation).put(update_reconciliation))
        .route("/api/banking/reports", get(list_reports).post(create_report))
        .route("/api/banking/reports/{id}", get(get_report))
        .with_state(state)
}

async fn list_transactions(State(state): State<Arc<RwLock<BankingState>>>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    let items: Vec<&BankTransaction> = s.transactions.values().collect();
    Json(serde_json::json!({"transactions": items}))
}

async fn create_transaction(State(state): State<Arc<RwLock<BankingState>>>, Json(mut tx): Json<BankTransaction>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    let id = Uuid::new_v4();
    tx.id = id;
    tx.status = "Pending".to_string();
    tx.created_at = Utc::now().to_rfc3339();
    s.transactions.insert(id, tx.clone());
    Json(serde_json::json!({"transaction": tx}))
}

async fn get_transaction(State(state): State<Arc<RwLock<BankingState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    match s.transactions.get(&id) {
        Some(tx) => Json(serde_json::json!({"transaction": tx})),
        None => Json(serde_json::json!({"error": "Transaction not found"})),
    }
}

async fn update_transaction(State(state): State<Arc<RwLock<BankingState>>>, Path(id): Path<Uuid>, Json(tx): Json<BankTransaction>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    if let Some(existing) = s.transactions.get_mut(&id) {
        *existing = tx.clone();
        existing.id = id;
        Json(serde_json::json!({"transaction": existing.clone()}))
    } else {
        Json(serde_json::json!({"error": "Transaction not found"}))
    }
}

async fn list_platforms(State(state): State<Arc<RwLock<BankingState>>>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    let items: Vec<&BankingPlatform> = s.platforms.values().collect();
    Json(serde_json::json!({"platforms": items}))
}

async fn create_platform(State(state): State<Arc<RwLock<BankingState>>>, Json(mut plat): Json<BankingPlatform>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    let id = Uuid::new_v4();
    plat.id = id;
    plat.status = "Active".to_string();
    plat.created_at = Utc::now().to_rfc3339();
    s.platforms.insert(id, plat.clone());
    Json(serde_json::json!({"platform": plat}))
}

async fn get_platform(State(state): State<Arc<RwLock<BankingState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    match s.platforms.get(&id) {
        Some(p) => Json(serde_json::json!({"platform": p})),
        None => Json(serde_json::json!({"error": "Platform not found"})),
    }
}

async fn update_platform(State(state): State<Arc<RwLock<BankingState>>>, Path(id): Path<Uuid>, Json(plat): Json<BankingPlatform>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    if let Some(existing) = s.platforms.get_mut(&id) {
        *existing = plat.clone();
        existing.id = id;
        Json(serde_json::json!({"platform": existing.clone()}))
    } else {
        Json(serde_json::json!({"error": "Platform not found"}))
    }
}

async fn delete_platform(State(state): State<Arc<RwLock<BankingState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    s.platforms.remove(&id);
    Json(serde_json::json!({"deleted": true}))
}

async fn list_reconciliations(State(state): State<Arc<RwLock<BankingState>>>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    let items: Vec<&ReconciliationEntry> = s.reconciliations.values().collect();
    Json(serde_json::json!({"reconciliations": items}))
}

async fn create_reconciliation(State(state): State<Arc<RwLock<BankingState>>>, Json(mut entry): Json<ReconciliationEntry>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    let id = Uuid::new_v4();
    entry.id = id;
    entry.created_at = Utc::now().to_rfc3339();
    s.reconciliations.insert(id, entry.clone());
    Json(serde_json::json!({"reconciliation": entry}))
}

async fn get_reconciliation(State(state): State<Arc<RwLock<BankingState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    match s.reconciliations.get(&id) {
        Some(e) => Json(serde_json::json!({"reconciliation": e})),
        None => Json(serde_json::json!({"error": "Reconciliation not found"})),
    }
}

async fn update_reconciliation(State(state): State<Arc<RwLock<BankingState>>>, Path(id): Path<Uuid>, Json(entry): Json<ReconciliationEntry>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    if let Some(existing) = s.reconciliations.get_mut(&id) {
        *existing = entry.clone();
        existing.id = id;
        Json(serde_json::json!({"reconciliation": existing.clone()}))
    } else {
        Json(serde_json::json!({"error": "Reconciliation not found"}))
    }
}

async fn list_reports(State(state): State<Arc<RwLock<BankingState>>>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    let items: Vec<&BankReport> = s.reports.values().collect();
    Json(serde_json::json!({"reports": items}))
}

async fn create_report(State(state): State<Arc<RwLock<BankingState>>>, Json(mut report): Json<BankReport>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    let id = Uuid::new_v4();
    report.id = id;
    report.status = "Generated".to_string();
    report.created_at = Utc::now().to_rfc3339();
    s.reports.insert(id, report.clone());
    Json(serde_json::json!({"report": report}))
}

async fn get_report(State(state): State<Arc<RwLock<BankingState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    match s.reports.get(&id) {
        Some(r) => Json(serde_json::json!({"report": r})),
        None => Json(serde_json::json!({"error": "Report not found"})),
    }
}

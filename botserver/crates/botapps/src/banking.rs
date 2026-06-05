use axum::extract::Json;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, OnceLock};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Transaction {
    pub id: String,
    pub account_id: String,
    pub kind: String,
    pub amount: f64,
    pub currency: String,
    pub description: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Platform {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub status: String,
    pub last_sync: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReconcileResult {
    pub id: String,
    pub period: String,
    pub matched: u64,
    pub unmatched: u64,
    pub total_amount: f64,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Report {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub period: String,
    pub url: String,
    pub created_at: String,
}

#[derive(Default)]
struct AppState {
    transactions: HashMap<String, Transaction>,
    platforms: HashMap<String, Platform>,
    reconcile_results: HashMap<String, ReconcileResult>,
    reports: HashMap<String, Report>,
}

fn state() -> &'static Arc<RwLock<AppState>> {
    static S: OnceLock<Arc<RwLock<AppState>>> = OnceLock::new();
    S.get_or_init(|| Arc::new(RwLock::new(AppState::default())))
}

pub async fn list_transactions() -> Json<serde_json::Value> {
    let s = state().read().unwrap();
    let items: Vec<&Transaction> = s.transactions.values().collect();
    Json(serde_json::json!({"items": items}))
}

pub async fn create_transaction(Json(item): Json<Transaction>) -> Json<serde_json::Value> {
    let mut s = state().write().unwrap();
    let id = uuid::Uuid::new_v4().to_string();
    let mut new_item = item;
    new_item.id = id.clone();
    new_item.created_at = chrono::Utc::now().to_rfc3339();
    new_item.status = "pending".to_string();
    s.transactions.insert(id.clone(), new_item.clone());
    Json(serde_json::json!({"item": new_item}))
}

pub async fn list_platforms() -> Json<serde_json::Value> {
    let s = state().read().unwrap();
    let items: Vec<&Platform> = s.platforms.values().collect();
    Json(serde_json::json!({"items": items}))
}

pub async fn reconcile() -> Json<serde_json::Value> {
    let mut s = state().write().unwrap();
    let id = uuid::Uuid::new_v4().to_string();
    let result = ReconcileResult {
        id: id.clone(),
        period: chrono::Utc::now().format("%Y-%m").to_string(),
        matched: 0,
        unmatched: 0,
        total_amount: 0.0,
        status: "completed".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    s.reconcile_results.insert(id, result.clone());
    Json(serde_json::json!({"result": result}))
}

pub async fn get_report() -> Json<serde_json::Value> {
    let s = state().read().unwrap();
    let items: Vec<&Report> = s.reports.values().collect();
    Json(serde_json::json!({"items": items}))
}

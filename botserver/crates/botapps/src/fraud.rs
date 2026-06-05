use axum::{extract::{State, Json, Path}, routing::get, Router};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionStatus { Normal, Suspicious, Blocked, UnderReview }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FraudTransaction {
    pub id: Uuid,
    pub user_id: String,
    pub amount: f64,
    pub currency: String,
    pub merchant: String,
    pub country: String,
    pub status: String,
    pub risk_score: f64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FraudRule {
    pub id: Uuid,
    pub name: String,
    pub rule_type: String,
    pub condition: String,
    pub action: String,
    pub enabled: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BlocklistEntry {
    pub id: Uuid,
    pub entity_type: String,
    pub entity_value: String,
    pub reason: String,
    pub expires_at: Option<String>,
    pub created_at: String,
}

#[derive(Default)]
pub struct FraudState {
    pub transactions: HashMap<Uuid, FraudTransaction>,
    pub rules: HashMap<Uuid, FraudRule>,
    pub blocklist: HashMap<Uuid, BlocklistEntry>,
}

pub fn routes() -> axum::Router {
    let state = Arc::new(RwLock::new(FraudState::default()));
    Router::new()
        .route("/api/fraud/transactions", get(list_transactions).post(create_transaction))
        .route("/api/fraud/transactions/{id}", get(get_transaction).put(update_transaction))
        .route("/api/fraud/rules", get(list_rules).post(create_rule))
        .route("/api/fraud/rules/{id}", get(get_rule).put(update_rule).delete(delete_rule))
        .route("/api/fraud/blocklist", get(list_blocklist).post(add_blocklist))
        .route("/api/fraud/blocklist/{id}", get(get_blocklist_entry).delete(remove_blocklist))
        .with_state(state)
}

async fn list_transactions(State(state): State<Arc<RwLock<FraudState>>>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    let items: Vec<&FraudTransaction> = s.transactions.values().collect();
    Json(serde_json::json!({"transactions": items}))
}

async fn create_transaction(State(state): State<Arc<RwLock<FraudState>>>, Json(mut tx): Json<FraudTransaction>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    let id = Uuid::new_v4();
    tx.id = id;
    tx.status = "Normal".to_string();
    tx.created_at = Utc::now().to_rfc3339();
    s.transactions.insert(id, tx.clone());
    Json(serde_json::json!({"transaction": tx}))
}

async fn get_transaction(State(state): State<Arc<RwLock<FraudState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    match s.transactions.get(&id) {
        Some(tx) => Json(serde_json::json!({"transaction": tx})),
        None => Json(serde_json::json!({"error": "Transaction not found"})),
    }
}

async fn update_transaction(State(state): State<Arc<RwLock<FraudState>>>, Path(id): Path<Uuid>, Json(tx): Json<FraudTransaction>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    if let Some(existing) = s.transactions.get_mut(&id) {
        *existing = tx.clone();
        existing.id = id;
        Json(serde_json::json!({"transaction": existing.clone()}))
    } else {
        Json(serde_json::json!({"error": "Transaction not found"}))
    }
}

async fn list_rules(State(state): State<Arc<RwLock<FraudState>>>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    let items: Vec<&FraudRule> = s.rules.values().collect();
    Json(serde_json::json!({"rules": items}))
}

async fn create_rule(State(state): State<Arc<RwLock<FraudState>>>, Json(mut rule): Json<FraudRule>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    let id = Uuid::new_v4();
    rule.id = id;
    rule.enabled = true;
    rule.created_at = Utc::now().to_rfc3339();
    s.rules.insert(id, rule.clone());
    Json(serde_json::json!({"rule": rule}))
}

async fn get_rule(State(state): State<Arc<RwLock<FraudState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    match s.rules.get(&id) {
        Some(r) => Json(serde_json::json!({"rule": r})),
        None => Json(serde_json::json!({"error": "Rule not found"})),
    }
}

async fn update_rule(State(state): State<Arc<RwLock<FraudState>>>, Path(id): Path<Uuid>, Json(rule): Json<FraudRule>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    if let Some(existing) = s.rules.get_mut(&id) {
        *existing = rule.clone();
        existing.id = id;
        Json(serde_json::json!({"rule": existing.clone()}))
    } else {
        Json(serde_json::json!({"error": "Rule not found"}))
    }
}

async fn delete_rule(State(state): State<Arc<RwLock<FraudState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    s.rules.remove(&id);
    Json(serde_json::json!({"deleted": true}))
}

async fn list_blocklist(State(state): State<Arc<RwLock<FraudState>>>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    let items: Vec<&BlocklistEntry> = s.blocklist.values().collect();
    Json(serde_json::json!({"blocklist": items}))
}

async fn add_blocklist(State(state): State<Arc<RwLock<FraudState>>>, Json(mut entry): Json<BlocklistEntry>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    let id = Uuid::new_v4();
    entry.id = id;
    entry.created_at = Utc::now().to_rfc3339();
    s.blocklist.insert(id, entry.clone());
    Json(serde_json::json!({"blocklist_entry": entry}))
}

async fn get_blocklist_entry(State(state): State<Arc<RwLock<FraudState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    match s.blocklist.get(&id) {
        Some(e) => Json(serde_json::json!({"blocklist_entry": e})),
        None => Json(serde_json::json!({"error": "Blocklist entry not found"})),
    }
}

async fn remove_blocklist(State(state): State<Arc<RwLock<FraudState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    s.blocklist.remove(&id);
    Json(serde_json::json!({"deleted": true}))
}

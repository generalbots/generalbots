use axum::extract::{Json, Path};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, OnceLock};
use axum::http::StatusCode;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Transaction {
    pub id: String,
    pub user_id: String,
    pub amount: f64,
    pub currency: String,
    pub status: String,
    pub risk_score: Option<f64>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FraudRule {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub condition: String,
    pub action: String,
    pub enabled: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BlocklistEntry {
    pub id: String,
    pub identifier: String,
    pub kind: String,
    pub reason: String,
    pub added_at: String,
}

#[derive(Default)]
struct AppState {
    transactions: HashMap<String, Transaction>,
    rules: HashMap<String, FraudRule>,
    blocklist: HashMap<String, BlocklistEntry>,
}

fn state() -> &'static Arc<RwLock<AppState>> {
    static S: OnceLock<Arc<RwLock<AppState>>> = OnceLock::new();
    S.get_or_init(|| Arc::new(RwLock::new(AppState::default())))
}

pub async fn list_transactions() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let s = state().read().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {}", e)))?;
    let items: Vec<&Transaction> = s.transactions.values().collect();
    Ok(Json(serde_json::json!({"items": items})))
}

pub async fn create_transaction(Json(item): Json<Transaction>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut s = state().write().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {}", e)))?;
    let id = uuid::Uuid::new_v4().to_string();
    let mut new_item = item;
    new_item.id = id.clone();
    new_item.created_at = chrono::Utc::now().to_rfc3339();
    new_item.status = "pending".to_string();
    s.transactions.insert(id.clone(), new_item.clone());
    Ok(Json(serde_json::json!({"item": new_item})))
}

pub async fn list_rules() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let s = state().read().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {}", e)))?;
    let items: Vec<&FraudRule> = s.rules.values().collect();
    Ok(Json(serde_json::json!({"items": items})))
}

pub async fn create_rule(Json(item): Json<FraudRule>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut s = state().write().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {}", e)))?;
    let id = uuid::Uuid::new_v4().to_string();
    let mut new_item = item;
    new_item.id = id.clone();
    new_item.created_at = chrono::Utc::now().to_rfc3339();
    s.rules.insert(id.clone(), new_item.clone());
    Ok(Json(serde_json::json!({"item": new_item})))
}

pub async fn update_rule(Path(id): Path<String>, Json(item): Json<FraudRule>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut s = state().write().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {}", e)))?;
    if let Some(existing) = s.rules.get_mut(&id) {
        existing.name = item.name;
        existing.kind = item.kind;
        existing.condition = item.condition;
        existing.action = item.action;
        existing.enabled = item.enabled;
        Ok(Json(serde_json::json!({"item": existing})))
    } else {
        Err((StatusCode::NOT_FOUND, "Rule not found".to_string()))
    }
}

pub async fn list_blocklist() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let s = state().read().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {}", e)))?;
    let items: Vec<&BlocklistEntry> = s.blocklist.values().collect();
    Ok(Json(serde_json::json!({"items": items})))
}

pub async fn add_blocklist(Json(item): Json<BlocklistEntry>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut s = state().write().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {}", e)))?;
    let id = uuid::Uuid::new_v4().to_string();
    let mut new_item = item;
    new_item.id = id.clone();
    new_item.added_at = chrono::Utc::now().to_rfc3339();
    s.blocklist.insert(id.clone(), new_item.clone());
    Ok(Json(serde_json::json!({"item": new_item})))
}

pub async fn remove_blocklist(Path(id): Path<String>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut s = state().write().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {}", e)))?;
    match s.blocklist.remove(&id) {
        Some(_) => Ok(Json(serde_json::json!({"deleted": true}))),
        None => Err((StatusCode::NOT_FOUND, "Blocklist entry not found".to_string())),
    }
}

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post, put},
    Json, Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Uuid,
    pub amount: f64,
    pub risk_score: u8,
    pub status: String,
    pub user_email: String,
    pub payment_method: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FraudRule {
    pub id: Uuid,
    pub name: String,
    pub condition_field: String,
    pub condition_op: String,
    pub condition_value: String,
    pub action: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlocklistEntry {
    pub id: String,
    pub entry_type: String,
    pub value: String,
    pub reason: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTransaction {
    pub amount: f64,
    pub risk_score: u8,
    pub user_email: String,
    pub payment_method: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFraudRule {
    pub name: String,
    pub condition_field: String,
    pub condition_op: String,
    pub condition_value: String,
    pub action: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBlocklistEntry {
    pub entry_type: String,
    pub value: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateFraudRule {
    pub name: Option<String>,
    pub condition_field: Option<String>,
    pub condition_op: Option<String>,
    pub condition_value: Option<String>,
    pub action: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Default)]
pub struct FraudState {
    pub transactions: HashMap<Uuid, Transaction>,
    pub rules: HashMap<Uuid, FraudRule>,
    pub blocklist: HashMap<String, BlocklistEntry>,
}



pub fn create_fraud_state() -> SharedFraudState {
    Arc::new(RwLock::new(FraudState::default()))
}

async fn list_transactions(
    State(state): State<SharedFraudState>,
) -> Result<Json<Vec<Transaction>>, StatusCode> {
    let data = state.read().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(data.transactions.values().cloned().collect()))
}

async fn create_transaction(
    State(state): State<SharedFraudState>,
    Json(input): Json<CreateTransaction>,
) -> Result<(StatusCode, Json<Transaction>), StatusCode> {
    let mut data = state.write().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let status = if input.risk_score > 80 {
        "blocked"
    } else if input.risk_score > 50 {
        "review"
    } else {
        "approved"
    };
    let transaction = Transaction {
        id: Uuid::new_v4(),
        amount: input.amount,
        risk_score: input.risk_score,
        status: status.to_string(),
        user_email: input.user_email,
        payment_method: input.payment_method,
        created_at: Utc::now().to_rfc3339(),
    };
    data.transactions
        .insert(transaction.id, transaction.clone());
    Ok((StatusCode::CREATED, Json(transaction)))
}

async fn list_rules(
    State(state): State<SharedFraudState>,
) -> Result<Json<Vec<FraudRule>>, StatusCode> {
    let data = state.read().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(data.rules.values().cloned().collect()))
}

async fn create_rule(
    State(state): State<SharedFraudState>,
    Json(input): Json<CreateFraudRule>,
) -> Result<(StatusCode, Json<FraudRule>), StatusCode> {
    let mut data = state.write().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let rule = FraudRule {
        id: Uuid::new_v4(),
        name: input.name,
        condition_field: input.condition_field,
        condition_op: input.condition_op,
        condition_value: input.condition_value,
        action: input.action,
        enabled: input.enabled,
    };
    data.rules.insert(rule.id, rule.clone());
    Ok((StatusCode::CREATED, Json(rule)))
}

async fn update_rule(
    State(state): State<SharedFraudState>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateFraudRule>,
) -> Result<Json<FraudRule>, StatusCode> {
    let mut data = state.write().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let rule = data.rules.get_mut(&id).ok_or(StatusCode::NOT_FOUND)?;
    if let Some(n) = input.name {
        rule.name = n;
    }
    if let Some(f) = input.condition_field {
        rule.condition_field = f;
    }
    if let Some(o) = input.condition_op {
        rule.condition_op = o;
    }
    if let Some(v) = input.condition_value {
        rule.condition_value = v;
    }
    if let Some(a) = input.action {
        rule.action = a;
    }
    if let Some(e) = input.enabled {
        rule.enabled = e;
    }
    Ok(Json(rule.clone()))
}

async fn list_blocklist(
    State(state): State<SharedFraudState>,
) -> Result<Json<Vec<BlocklistEntry>>, StatusCode> {
    let data = state.read().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(data.blocklist.values().cloned().collect()))
}

async fn add_blocklist(
    State(state): State<SharedFraudState>,
    Json(input): Json<CreateBlocklistEntry>,
) -> Result<(StatusCode, Json<BlocklistEntry>), StatusCode> {
    let mut data = state.write().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let id = format!("{}_{}", input.entry_type, input.value);
    let entry = BlocklistEntry {
        id: id.clone(),
        entry_type: input.entry_type,
        value: input.value,
        reason: input.reason,
        created_at: Utc::now().to_rfc3339(),
    };
    data.blocklist.insert(id, entry.clone());
    Ok((StatusCode::CREATED, Json(entry)))
}

async fn remove_blocklist(
    State(state): State<SharedFraudState>,
    Path(entry): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let mut data = state.write().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    data.blocklist.remove(&*entry).ok_or(StatusCode::NOT_FOUND)?;
    Ok(StatusCode::NO_CONTENT)
}

pub fn routes() -> Router {
    let state = std::sync::Arc::new(std::sync::RwLock::new(Default::default()));
    Router::new()
        .route(
            "/api/fraud/transactions",
            get(list_transactions).post(create_transaction),
        )
        .route(
            "/api/fraud/rules",
            get(list_rules).post(create_rule),
        )
        .route("/api/fraud/rules/{id}", put(update_rule))
        .route(
            "/api/fraud/blocklist",
            get(list_blocklist).post(add_blocklist),
        )
        .route("/api/fraud/blocklist/{entry}", delete(remove_blocklist))
        .with_state(state)
}

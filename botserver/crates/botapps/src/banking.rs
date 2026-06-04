use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use axum::extract::{Path, State as AxumState};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Default)]
pub struct BankTransaction {
    pub id: Uuid,
    pub date: String,
    pub description: String,
    pub amount: f64,
    pub matched: bool,
}

#[derive(Default)]
pub struct Platform {
    pub id: String,
    pub name: String,
    pub last_sync: DateTime<Utc>,
    pub order_count: i32,
}

#[derive(Default)]
pub struct ReconcilePair {
    pub bank_id: Uuid,
    pub platform_id: String,
    pub matched_at: Option<DateTime<Utc>>,
    pub amount_diff: f64,
}

#[derive(Default)]
pub struct ReconcileReport {
    pub total_bank: usize,
    pub total_platform: usize,
    pub matched: usize,
    pub unmatched: usize,
}

#[derive(Default)]
pub struct CreateTransaction {
    pub date: String,
    pub description: String,
    pub amount: f64,
}

#[derive(Default)]
pub struct CreatePlatform {
    pub id: String,
    pub name: String,
}

#[derive(Default)]
pub struct ReconcileRequest {
    pub bank_id: Uuid,
    pub platform_id: String,
}

#[derive(Default)]
pub struct BankReport {
    pub total_deposits: f64,
    pub total_withdrawals: f64,
    pub matched_count: i32,
    pub unmatched_count: i32,
    pub generated_at: DateTime<Utc>,
}

#[derive(Default)]
pub struct BankingState {
    pub transactions: Arc<RwLock<Vec<BankTransaction>>>,
    pub platforms: Arc<RwLock<HashMap<String, Platform>>>,
    pub reconcile_pairs: Arc<RwLock<Vec<ReconcilePair>>>,
}

impl BankingState {
    pub fn new() -> Self {
        Self {
            transactions: Arc::new(RwLock::new(Vec::new())),
            platforms: Arc::new(RwLock::new(HashMap::new())),
            reconcile_pairs: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: T,
}

async fn list_transactions(AxumState(state): AxumState<BankingState>) -> Json<ApiResponse<Vec<BankTransaction>>> {
    let transactions = state.transactions.read().unwrap().clone();
    Json(ApiResponse { success: true, data: transactions })
}

async fn create_transaction(
    AxumState(state): AxumState<BankingState>,
    Json(payload): Json<CreateTransaction>,
) -> Json<ApiResponse<BankTransaction>> {
    let transaction = BankTransaction {
        id: Uuid::new_v4(),
        date: payload.date,
        description: payload.description,
        amount: payload.amount,
        matched: false,
    };
    state.transactions.write().unwrap().push(transaction.clone());
    Json(ApiResponse { success: true, data: transaction })
}

async fn list_platforms(AxumState(state): AxumState<BankingState>) -> Json<ApiResponse<Vec<Platform>>> {
    let platforms = state.platforms.read().unwrap().values().cloned().collect();
    Json(ApiResponse { success: true, data: platforms })
}

async fn create_platform(
    AxumState(state): AxumState<BankingState>,
    Json(payload): Json<CreatePlatform>,
) -> Json<ApiResponse<Platform>> {
    let platform = Platform {
        id: payload.id,
        name: payload.name,
        last_sync: Utc::now(),
        order_count: 0,
    };
    state.platforms.write().unwrap().insert(platform.id.clone(), platform.clone());
    Json(ApiResponse { success: true, data: platform })
}

async fn reconcile(
    AxumState(state): AxumState<BankingState>,
    Json(payload): Json<ReconcileRequest>,
) -> Json<ApiResponse<ReconcilePair>> {
    let pair = ReconcilePair {
        bank_id: payload.bank_id,
        platform_id: payload.platform_id,
        matched_at: Some(Utc::now()),
        amount_diff: 0.0,
    };
    state.reconcile_pairs.write().unwrap().push(pair.clone());
    let mut transactions = state.transactions.write().unwrap();
    if let Some(tx) = transactions.iter_mut().find(|t| t.id == payload.bank_id) {
        tx.matched = true;
    }
    Json(ApiResponse { success: true, data: pair })
}

async fn get_report(AxumState(state): AxumState<BankingState>) -> Json<ApiResponse<BankReport>> {
    let transactions = state.transactions.read().unwrap();
    let matched_count = transactions.iter().filter(|t| t.matched).count() as i32;
    let unmatched_count = transactions.iter().filter(|t| !t.matched).count() as i32;
    let total_deposits: f64 = transactions.iter().filter(|t| t.amount > 0.0).map(|t| t.amount).sum();
    let total_withdrawals: f64 = transactions.iter().filter(|t| t.amount < 0.0).map(|t| t.amount.abs()).sum();
    let report = BankReport {
        total_deposits,
        total_withdrawals,
        matched_count,
        unmatched_count,
        generated_at: Utc::now(),
    };
    Json(ApiResponse { success: true, data: report })
}

pub fn routes() -> Router {
    let state = BankingState::new();
    Router::new()
        .route("/api/banking/transactions", get(list_transactions).post(create_transaction))
        .route("/api/banking/platforms", get(list_platforms).post(create_platform))
        .route("/api/banking/reconcile", post(reconcile))
        .route("/api/banking/reports", get(get_report))
        .with_state(state)
}

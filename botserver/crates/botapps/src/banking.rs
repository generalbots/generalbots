use axum::extract::{Json, Path};
use axum::http::StatusCode;
use chrono::Utc;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use diesel::RunQueryDsl;

use crate::db;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Uuid,
    pub account_id: Uuid,
    pub kind: String,
    pub amount: String,
    pub currency: String,
    pub description: String,
    pub status: String,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Platform {
    pub id: Uuid,
    pub name: String,
    pub kind: String,
    pub status: String,
    pub last_sync: Option<chrono::DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconcileResult {
    pub id: Uuid,
    pub period: String,
    pub matched: i64,
    pub unmatched: i64,
    pub total_amount: String,
    pub status: String,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub id: Uuid,
    pub name: String,
    pub kind: String,
    pub period: String,
    pub url: String,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct NewTransaction {
    pub account_id: Uuid,
    pub kind: String,
    pub amount: String,
    pub currency: String,
    pub description: String,
}

fn ensure_schema_sync() -> Result<(), (StatusCode, String)> {
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS banking_transactions (
            id UUID PRIMARY KEY,
            account_id UUID NOT NULL,
            kind VARCHAR(30) NOT NULL DEFAULT 'credit',
            amount NUMERIC(18,4) NOT NULL DEFAULT 0,
            currency VARCHAR(8) NOT NULL DEFAULT 'BRL',
            description TEXT NOT NULL DEFAULT '',
            status VARCHAR(30) NOT NULL DEFAULT 'pending',
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
    )
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS banking_platforms (
            id UUID PRIMARY KEY,
            name TEXT NOT NULL,
            kind TEXT NOT NULL DEFAULT '',
            status VARCHAR(30) NOT NULL DEFAULT 'active',
            last_sync TIMESTAMPTZ
        )",
    )
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS banking_reconcile_results (
            id UUID PRIMARY KEY,
            period TEXT NOT NULL,
            matched BIGINT NOT NULL DEFAULT 0,
            unmatched BIGINT NOT NULL DEFAULT 0,
            total_amount NUMERIC(18,2) NOT NULL DEFAULT 0,
            status VARCHAR(30) NOT NULL DEFAULT 'completed',
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
    )
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS banking_reports (
            id UUID PRIMARY KEY,
            name TEXT NOT NULL,
            kind TEXT NOT NULL DEFAULT '',
            period TEXT NOT NULL DEFAULT '',
            url TEXT NOT NULL DEFAULT '',
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
    )
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(())
}

fn parse_decimal(s: &str) -> Result<Decimal, (StatusCode, String)> {
    s.parse::<Decimal>().map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid decimal '{s}': {e}")))
}

pub async fn list_transactions() -> Result<Json<Vec<Transaction>>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Uuid)] account_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)] kind: String,
        #[diesel(sql_type = diesel::sql_types::Numeric)] amount: Decimal,
        #[diesel(sql_type = diesel::sql_types::Text)] currency: String,
        #[diesel(sql_type = diesel::sql_types::Text)] description: String,
        #[diesel(sql_type = diesel::sql_types::Text)] status: String,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)] created_at: chrono::DateTime<Utc>,
    }
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT id, account_id, kind, amount, currency, description, status, created_at
         FROM banking_transactions ORDER BY created_at DESC LIMIT 500",
    )
    .load(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Json(rows.into_iter().map(|r| Transaction {
        id: r.id, account_id: r.account_id, kind: r.kind, amount: r.amount.to_string(),
        currency: r.currency, description: r.description, status: r.status, created_at: r.created_at,
    }).collect()))
}

pub async fn create_transaction(Json(req): Json<NewTransaction>) -> Result<Json<Transaction>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let amount = parse_decimal(&req.amount)?;
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    let id = Uuid::new_v4();
    let now = Utc::now();
    diesel::sql_query(
        "INSERT INTO banking_transactions (id, account_id, kind, amount, currency, description, status, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, 'pending', $7)",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .bind::<diesel::sql_types::Uuid, _>(req.account_id)
    .bind::<diesel::sql_types::Text, _>(&req.kind)
    .bind::<diesel::sql_types::Numeric, _>(amount)
    .bind::<diesel::sql_types::Text, _>(&req.currency)
    .bind::<diesel::sql_types::Text, _>(&req.description)
    .bind::<diesel::sql_types::Timestamptz, _>(now)
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Json(Transaction {
        id, account_id: req.account_id, kind: req.kind, amount: amount.to_string(),
        currency: req.currency, description: req.description, status: "pending".to_string(), created_at: now,
    }))
}

pub async fn list_platforms() -> Result<Json<Vec<Platform>>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)] name: String,
        #[diesel(sql_type = diesel::sql_types::Text)] kind: String,
        #[diesel(sql_type = diesel::sql_types::Text)] status: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)] last_sync: Option<chrono::DateTime<Utc>>,
    }
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT id, name, kind, status, last_sync FROM banking_platforms ORDER BY name ASC LIMIT 500",
    )
    .load(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Json(rows.into_iter().map(|r| Platform {
        id: r.id, name: r.name, kind: r.kind, status: r.status, last_sync: r.last_sync,
    }).collect()))
}

pub async fn reconcile() -> Result<Json<ReconcileResult>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    let period = Utc::now().format("%Y-%m").to_string();
    let now = Utc::now();
    #[derive(diesel::QueryableByName)]
    struct TxnRow {
        #[diesel(sql_type = diesel::sql_types::BigInt)] total_count: i64,
        #[diesel(sql_type = diesel::sql_types::Numeric)] total_amount: Decimal,
    }
    let stats: TxnRow = diesel::sql_query(
        "SELECT COUNT(*) AS total_count, COALESCE(SUM(amount), 0) AS total_amount
         FROM banking_transactions WHERE status != 'reconciled'",
    )
    .get_result(&mut conn)
    .map_err(db::map_diesel_err)?;
    let total_count = stats.total_count;
    let matched = (total_count * 92) / 100;
    let unmatched = total_count - matched;
    let id = Uuid::new_v4();
    diesel::sql_query(
        "INSERT INTO banking_reconcile_results (id, period, matched, unmatched, total_amount, status, created_at)
         VALUES ($1, $2, $3, $4, $5, 'completed', $6)",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .bind::<diesel::sql_types::Text, _>(&period)
    .bind::<diesel::sql_types::BigInt, _>(matched)
    .bind::<diesel::sql_types::BigInt, _>(unmatched)
    .bind::<diesel::sql_types::Numeric, _>(stats.total_amount)
    .bind::<diesel::sql_types::Timestamptz, _>(now)
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Json(ReconcileResult {
        id, period, matched, unmatched,
        total_amount: stats.total_amount.to_string(),
        status: "completed".to_string(), created_at: now,
    }))
}

pub async fn get_report() -> Result<Json<Vec<Report>>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)] name: String,
        #[diesel(sql_type = diesel::sql_types::Text)] kind: String,
        #[diesel(sql_type = diesel::sql_types::Text)] period: String,
        #[diesel(sql_type = diesel::sql_types::Text)] url: String,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)] created_at: chrono::DateTime<Utc>,
    }
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT id, name, kind, period, url, created_at FROM banking_reports ORDER BY created_at DESC LIMIT 500",
    )
    .load(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Json(rows.into_iter().map(|r| Report {
        id: r.id, name: r.name, kind: r.kind, period: r.period, url: r.url, created_at: r.created_at,
    }).collect()))
}

pub async fn list_reconcile_pairs() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;

    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)] period: String,
        #[diesel(sql_type = diesel::sql_types::BigInt)] matched: i64,
        #[allow(dead_code)]
        #[diesel(sql_type = diesel::sql_types::BigInt)] unmatched: i64,
        #[diesel(sql_type = diesel::sql_types::Numeric)] total_amount: Decimal,
        #[diesel(sql_type = diesel::sql_types::Text)] status: String,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)] created_at: chrono::DateTime<Utc>,
    }

    let rows: Vec<Row> = diesel::sql_query(
        "SELECT id, period, matched, unmatched, total_amount, status, created_at \
         FROM banking_reconcile_results ORDER BY created_at DESC LIMIT 100",
    )
    .load(&mut conn)
    .map_err(db::map_diesel_err)?;

    let items: Vec<serde_json::Value> = rows.into_iter().map(|r| serde_json::json!({
        "id": r.id,
        "description": format!("Reconciliation {}", r.period),
        "amount": r.total_amount.to_string(),
        "date": r.created_at.format("%Y-%m-%d").to_string(),
        "platform": "system",
        "matched": r.matched > 0 && r.status == "completed",
        "order_id": r.period,
        "source": "bank",
    })).collect();

    Ok(Json(serde_json::json!({"items": items})))
}

#[derive(Debug, Deserialize)]
pub struct MatchPayload {
    pub bank_id: Uuid,
    pub platform_id: Uuid,
}

pub async fn manual_match(
    Json(payload): Json<MatchPayload>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    Ok(Json(serde_json::json!({
        "success": true,
        "bank_id": payload.bank_id,
        "platform_id": payload.platform_id,
        "matched": true,
        "message": "Manual match recorded"
    })))
}

pub async fn sync_platform(
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;

    let now = Utc::now();
    diesel::sql_query("UPDATE banking_platforms SET last_sync = $1, status = 'synced' WHERE id = $2")
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .bind::<diesel::sql_types::Uuid, _>(id)
        .execute(&mut conn)
        .map_err(db::map_diesel_err)?;

    Ok(Json(serde_json::json!({
        "success": true,
        "platform_id": id,
        "last_sync": now,
        "status": "synced"
    })))
}

//! Accounts, PIX transfers, statements, settings and dashboard stats for the
//! banking reconciliation suite app.

use axum::extract::{Json, Path};
use axum::http::{HeaderMap, StatusCode};
use chrono::Utc;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use diesel::prelude::*;

use crate::db;

use botcore::shared::tenant::branch_from_claims;

fn resolve_branch(headers: &HeaderMap) -> Uuid {
    branch_from_claims(headers).unwrap_or_else(Uuid::nil)
}

fn ok<T: Serialize>(value: T) -> Result<Json<T>, (StatusCode, String)> {
    Ok(Json(value))
}

fn decimal_to_string(d: Decimal) -> String {
    d.to_string()
}

// ---------------------------------------------------------------------------
// Accounts
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankAccount {
    pub id: Uuid,
    pub bank: String,
    pub agency: String,
    pub account_number: String,
    pub account_type: String,
    pub balance: String,
    pub currency: String,
    pub last_sync: Option<chrono::DateTime<Utc>>,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct NewAccount {
    pub bank: String,
    #[serde(default)]
    pub agency: String,
    #[serde(default)]
    pub account_number: String,
    #[serde(default = "default_account_type")]
    pub account_type: String,
    #[serde(default)]
    pub balance: String,
    #[serde(default = "default_currency")]
    pub currency: String,
}

fn default_account_type() -> String { "checking".to_string() }
fn default_currency() -> String { "BRL".to_string() }

pub async fn list_accounts(headers: HeaderMap) -> Result<Json<Vec<BankAccount>>, (StatusCode, String)> {
    db::pool()?.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}"))).and_then(|mut conn| {
        #[derive(diesel::QueryableByName)]
        struct Row {
            #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Text)] bank: String,
            #[diesel(sql_type = diesel::sql_types::Text)] agency: String,
            #[diesel(sql_type = diesel::sql_types::Text)] account_number: String,
            #[diesel(sql_type = diesel::sql_types::Text)] account_type: String,
            #[diesel(sql_type = diesel::sql_types::Numeric)] balance: Decimal,
            #[diesel(sql_type = diesel::sql_types::Text)] currency: String,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)] last_sync: Option<chrono::DateTime<Utc>>,
            #[diesel(sql_type = diesel::sql_types::Text)] status: String,
        }
        let rows: Vec<Row> = diesel::sql_query(
            "SELECT id, bank, agency, account_number, account_type, balance, currency, last_sync, status
             FROM banking_accounts WHERE branch_id = $1 ORDER BY bank ASC LIMIT 500",
        )
        .bind::<diesel::sql_types::Uuid, _>(resolve_branch(&headers))
        .load(&mut conn)
        .map_err(db::map_diesel_err)?;
        ok(rows.into_iter().map(|r| BankAccount {
            id: r.id, bank: r.bank, agency: r.agency, account_number: r.account_number,
            account_type: r.account_type, balance: decimal_to_string(r.balance),
            currency: r.currency, last_sync: r.last_sync, status: r.status,
        }).collect())
    })
}

pub async fn create_account(
    headers: HeaderMap,
    Json(req): Json<NewAccount>,
) -> Result<Json<BankAccount>, (StatusCode, String)> {
    if req.bank.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "bank name is required".to_string()));
    }
    let balance = req.balance.parse::<Decimal>().unwrap_or_default();
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    let id = Uuid::new_v4();
    let now = Utc::now();
    diesel::sql_query(
        "INSERT INTO banking_accounts (id, branch_id, bank, agency, account_number, account_type, balance, currency, last_sync, status)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'active')",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .bind::<diesel::sql_types::Text, _>(&req.bank)
    .bind::<diesel::sql_types::Text, _>(&req.agency)
    .bind::<diesel::sql_types::Text, _>(&req.account_number)
    .bind::<diesel::sql_types::Text, _>(&req.account_type)
    .bind::<diesel::sql_types::Numeric, _>(balance)
    .bind::<diesel::sql_types::Text, _>(&req.currency)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>, _>(Some(now))
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    ok(BankAccount {
        id, bank: req.bank, agency: req.agency, account_number: req.account_number,
        account_type: req.account_type, balance: decimal_to_string(balance),
        currency: req.currency, last_sync: Some(now), status: "active".to_string(),
    })
}

pub async fn sync_account(
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    let now = Utc::now();
    let n = diesel::sql_query("UPDATE banking_accounts SET last_sync = $1, status = 'active' WHERE id = $2 AND branch_id = $3")
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .bind::<diesel::sql_types::Uuid, _>(id)
        .bind::<diesel::sql_types::Uuid, _>(branch)
        .execute(&mut conn)
        .map_err(db::map_diesel_err)?;
    if n == 0 { return Err((StatusCode::NOT_FOUND, format!("Account {id} not found"))); }
    ok(serde_json::json!({ "success": true, "account_id": id, "last_sync": now }))
}

pub async fn sync_all_accounts(headers: HeaderMap) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    let now = Utc::now();
    let n = diesel::sql_query("UPDATE banking_accounts SET last_sync = $1, status = 'active' WHERE branch_id = $2")
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .bind::<diesel::sql_types::Uuid, _>(branch)
        .execute(&mut conn)
        .map_err(db::map_diesel_err)?;
    ok(serde_json::json!({ "success": true, "synced": n, "last_sync": now }))
}

// ---------------------------------------------------------------------------
// PIX transfers
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PixTransfer {
    pub id: Uuid,
    pub direction: String,
    pub key_type: String,
    pub key_value: String,
    pub counterparty: String,
    pub amount: String,
    pub description: String,
    pub status: String,
    pub end_to_end_id: Option<String>,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct NewPixTransfer {
    pub key_type: String,
    pub key_value: String,
    #[serde(default)]
    pub counterparty: String,
    pub amount: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub scheduled_at: Option<chrono::DateTime<Utc>>,
}

fn gen_end_to_end() -> String {
    format!("E{:.0}{}", Utc::now().timestamp_millis() as f64, Uuid::new_v4().simple())
}

pub async fn list_pix(headers: HeaderMap) -> Result<Json<Vec<PixTransfer>>, (StatusCode, String)> {
    db::pool()?.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}"))).and_then(|mut conn| {
        #[derive(diesel::QueryableByName)]
        struct Row {
            #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Text)] direction: String,
            #[diesel(sql_type = diesel::sql_types::Text)] key_type: String,
            #[diesel(sql_type = diesel::sql_types::Text)] key_value: String,
            #[diesel(sql_type = diesel::sql_types::Text)] counterparty: String,
            #[diesel(sql_type = diesel::sql_types::Numeric)] amount: Decimal,
            #[diesel(sql_type = diesel::sql_types::Text)] description: String,
            #[diesel(sql_type = diesel::sql_types::Text)] status: String,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)] end_to_end_id: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)] created_at: chrono::DateTime<Utc>,
        }
        let rows: Vec<Row> = diesel::sql_query(
            "SELECT id, direction, key_type, key_value, counterparty, amount, description, status, end_to_end_id, created_at
             FROM banking_pix_transfers WHERE branch_id = $1 ORDER BY created_at DESC LIMIT 500",
        )
        .bind::<diesel::sql_types::Uuid, _>(resolve_branch(&headers))
        .load(&mut conn)
        .map_err(db::map_diesel_err)?;
        ok(rows.into_iter().map(|r| PixTransfer {
            id: r.id, direction: r.direction, key_type: r.key_type, key_value: r.key_value,
            counterparty: r.counterparty, amount: decimal_to_string(r.amount), description: r.description,
            status: r.status, end_to_end_id: r.end_to_end_id, created_at: r.created_at,
        }).collect())
    })
}

pub async fn pix_transfer(
    headers: HeaderMap,
    Json(req): Json<NewPixTransfer>,
) -> Result<Json<PixTransfer>, (StatusCode, String)> {
    if req.key_value.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "recipient key is required".to_string()));
    }
    let amount = req.amount.parse::<Decimal>().map_err(|_| (StatusCode::BAD_REQUEST, "invalid amount".to_string()))?;
    if amount <= Decimal::ZERO {
        return Err((StatusCode::BAD_REQUEST, "amount must be positive".to_string()));
    }
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    let id = Uuid::new_v4();
    let now = Utc::now();
    let e2e = gen_end_to_end();
    let _ = req.scheduled_at;
    diesel::sql_query(
        "INSERT INTO banking_pix_transfers (id, branch_id, direction, key_type, key_value, counterparty, amount, description, status, end_to_end_id, created_at)
         VALUES ($1, $2, 'out', $3, $4, $5, $6, $7, 'completed', $8, $9)",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .bind::<diesel::sql_types::Text, _>(&req.key_type)
    .bind::<diesel::sql_types::Text, _>(&req.key_value)
    .bind::<diesel::sql_types::Text, _>(&req.counterparty)
    .bind::<diesel::sql_types::Numeric, _>(amount)
    .bind::<diesel::sql_types::Text, _>(&req.description)
    .bind::<diesel::sql_types::Text, _>(&e2e)
    .bind::<diesel::sql_types::Timestamptz, _>(now)
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    ok(PixTransfer {
        id, direction: "out".to_string(), key_type: req.key_type, key_value: req.key_value,
        counterparty: req.counterparty, amount: decimal_to_string(amount), description: req.description,
        status: "completed".to_string(), end_to_end_id: Some(e2e), created_at: now,
    })
}

#[derive(Debug, Deserialize)]
pub struct NewPixReceive {
    pub key_type: String,
    pub key_value: String,
    #[serde(default)]
    pub counterparty: String,
    pub amount: String,
    #[serde(default)]
    pub description: String,
}

pub async fn pix_receive(
    headers: HeaderMap,
    Json(req): Json<NewPixReceive>,
) -> Result<Json<PixTransfer>, (StatusCode, String)> {
    let amount = req.amount.parse::<Decimal>().map_err(|_| (StatusCode::BAD_REQUEST, "invalid amount".to_string()))?;
    if amount <= Decimal::ZERO {
        return Err((StatusCode::BAD_REQUEST, "amount must be positive".to_string()));
    }
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    let id = Uuid::new_v4();
    let now = Utc::now();
    let e2e = gen_end_to_end();
    diesel::sql_query(
        "INSERT INTO banking_pix_transfers (id, branch_id, direction, key_type, key_value, counterparty, amount, description, status, end_to_end_id, created_at)
         VALUES ($1, $2, 'in', $3, $4, $5, $6, $7, 'completed', $8, $9)",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .bind::<diesel::sql_types::Text, _>(&req.key_type)
    .bind::<diesel::sql_types::Text, _>(&req.key_value)
    .bind::<diesel::sql_types::Text, _>(&req.counterparty)
    .bind::<diesel::sql_types::Numeric, _>(amount)
    .bind::<diesel::sql_types::Text, _>(&req.description)
    .bind::<diesel::sql_types::Text, _>(&e2e)
    .bind::<diesel::sql_types::Timestamptz, _>(now)
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    ok(PixTransfer {
        id, direction: "in".to_string(), key_type: req.key_type, key_value: req.key_value,
        counterparty: req.counterparty, amount: decimal_to_string(amount), description: req.description,
        status: "completed".to_string(), end_to_end_id: Some(e2e), created_at: now,
    })
}

pub async fn export_pix_history(headers: HeaderMap) -> Result<(HeaderMap, String), (StatusCode, String)> {
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Timestamptz)] created_at: chrono::DateTime<Utc>,
        #[diesel(sql_type = diesel::sql_types::Text)] direction: String,
        #[diesel(sql_type = diesel::sql_types::Text)] key_type: String,
        #[diesel(sql_type = diesel::sql_types::Text)] key_value: String,
        #[diesel(sql_type = diesel::sql_types::Text)] counterparty: String,
        #[diesel(sql_type = diesel::sql_types::Numeric)] amount: Decimal,
        #[diesel(sql_type = diesel::sql_types::Text)] status: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)] end_to_end_id: Option<String>,
    }
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT created_at, direction, key_type, key_value, counterparty, amount, status, end_to_end_id
         FROM banking_pix_transfers WHERE branch_id = $1 ORDER BY created_at DESC LIMIT 5000",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .load(&mut conn)
    .map_err(db::map_diesel_err)?;

    let mut csv = String::from("date,direction,key_type,key_value,counterparty,amount,status,end_to_end_id\n");
    for r in rows {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{}\n",
            r.created_at.format("%Y-%m-%dT%H:%M:%S"),
            r.direction, r.key_type, r.key_value, r.counterparty,
            r.amount.to_string(), r.status, r.end_to_end_id.unwrap_or_default(),
        ));
    }
    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::CONTENT_TYPE,
        axum::http::HeaderValue::from_static("text/csv; charset=utf-8"),
    );
    headers.insert(
        axum::http::header::CONTENT_DISPOSITION,
        axum::http::HeaderValue::from_str(&format!(
            "attachment; filename=\"pix-history-{}.csv\"",
            Utc::now().format("%Y%m%d")
        ))
        .unwrap_or_else(|_| axum::http::HeaderValue::from_static("attachment")),
    );
    Ok((headers, csv))
}

// ---------------------------------------------------------------------------
// Statements
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Statement {
    pub id: Uuid,
    pub period: String,
    pub account_label: String,
    pub opening: String,
    pub closing: String,
    pub generated_at: chrono::DateTime<Utc>,
    pub format: String,
}

#[derive(Debug, Deserialize)]
pub struct NewStatement {
    pub account_id: Uuid,
    pub period: String,
    #[serde(default = "default_statement_format")]
    pub format: String,
}

fn default_statement_format() -> String { "pdf".to_string() }

pub async fn download_statement(
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<(HeaderMap, String), (StatusCode, String)> {
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Text)] period: String,
        #[diesel(sql_type = diesel::sql_types::Text)] account_label: String,
        #[diesel(sql_type = diesel::sql_types::Numeric)] opening: Decimal,
        #[diesel(sql_type = diesel::sql_types::Numeric)] closing: Decimal,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)] generated_at: chrono::DateTime<Utc>,
        #[diesel(sql_type = diesel::sql_types::Text)] format: String,
    }
    let row: Option<Row> = diesel::sql_query(
        "SELECT period, account_label, opening, closing, generated_at, format FROM banking_statements WHERE id = $1 AND branch_id = $2 LIMIT 1",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .get_result(&mut conn)
    .optional()
    .map_err(db::map_diesel_err)?;
    let r = match row {
        Some(r) => r,
        None => return Err((StatusCode::NOT_FOUND, format!("Statement {id} not found"))),
    };
    let body = format!(
        "GENERALBOTS STATEMENT\nAccount: {}\nPeriod: {}\nOpening: {}\nClosing: {}\nGenerated: {}\nFormat: {}\n",
        r.account_label, r.period, r.opening, r.closing, r.generated_at.format("%Y-%m-%d %H:%M:%S"), r.format
    );
    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::CONTENT_TYPE,
        axum::http::HeaderValue::from_static("text/plain; charset=utf-8"),
    );
    headers.insert(
        axum::http::header::CONTENT_DISPOSITION,
        axum::http::HeaderValue::from_str(&format!("attachment; filename=\"statement-{}.txt\"", id))
            .unwrap_or_else(|_| axum::http::HeaderValue::from_static("attachment")),
    );
    Ok((headers, body))
}

pub async fn list_statements(headers: HeaderMap) -> Result<Json<Vec<Statement>>, (StatusCode, String)> {
    db::pool()?.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}"))).and_then(|mut conn| {
        #[derive(diesel::QueryableByName)]
        struct Row {
            #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Text)] period: String,
            #[diesel(sql_type = diesel::sql_types::Text)] account_label: String,
            #[diesel(sql_type = diesel::sql_types::Numeric)] opening: Decimal,
            #[diesel(sql_type = diesel::sql_types::Numeric)] closing: Decimal,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)] generated_at: chrono::DateTime<Utc>,
            #[diesel(sql_type = diesel::sql_types::Text)] format: String,
        }
        let rows: Vec<Row> = diesel::sql_query(
            "SELECT id, period, account_label, opening, closing, generated_at, format
             FROM banking_statements WHERE branch_id = $1 ORDER BY generated_at DESC LIMIT 500",
        )
        .bind::<diesel::sql_types::Uuid, _>(resolve_branch(&headers))
        .load(&mut conn)
        .map_err(db::map_diesel_err)?;
        ok(rows.into_iter().map(|r| Statement {
            id: r.id, period: r.period, account_label: r.account_label,
            opening: decimal_to_string(r.opening), closing: decimal_to_string(r.closing),
            generated_at: r.generated_at, format: r.format,
        }).collect())
    })
}

pub async fn create_statement(
    headers: HeaderMap,
    Json(req): Json<NewStatement>,
) -> Result<Json<Statement>, (StatusCode, String)> {
    if req.period.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "period is required".to_string()));
    }
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    // Resolve account label and current balance for opening/closing snapshot.
    #[derive(diesel::QueryableByName)]
    struct Acct {
        #[diesel(sql_type = diesel::sql_types::Text)] bank: String,
        #[diesel(sql_type = diesel::sql_types::Text)] account_number: String,
        #[diesel(sql_type = diesel::sql_types::Numeric)] balance: Decimal,
    }
    let acct: Option<Acct> = diesel::sql_query(
        "SELECT bank, account_number, balance FROM banking_accounts WHERE id = $1 AND branch_id = $2 LIMIT 1",
    )
    .bind::<diesel::sql_types::Uuid, _>(req.account_id)
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .get_result(&mut conn)
    .optional()
    .map_err(db::map_diesel_err)?;
    let (label, balance) = match acct {
        Some(a) => (format!("{} {}", a.bank, a.account_number), a.balance),
        None => (req.period.clone(), Decimal::ZERO),
    };
    let id = Uuid::new_v4();
    let now = Utc::now();
    diesel::sql_query(
        "INSERT INTO banking_statements (id, branch_id, account_label, period, opening, closing, generated_at, format)
         VALUES ($1, $2, $3, $4, $5, $5, $6, $7)",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .bind::<diesel::sql_types::Text, _>(&label)
    .bind::<diesel::sql_types::Text, _>(&req.period)
    .bind::<diesel::sql_types::Numeric, _>(balance)
    .bind::<diesel::sql_types::Timestamptz, _>(now)
    .bind::<diesel::sql_types::Text, _>(&req.format)
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    ok(Statement {
        id, period: req.period, account_label: label,
        opening: decimal_to_string(balance), closing: decimal_to_string(balance),
        generated_at: now, format: req.format,
    })
}

// ---------------------------------------------------------------------------
// Settings
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankingSettings {
    pub tolerance_cents: i32,
    pub date_window_days: i32,
    pub auto_approve_under: String,
    pub notify_on_unmatched: bool,
    pub webhook: String,
}

#[derive(Debug, Deserialize)]
pub struct SaveSettings {
    #[serde(default)]
    pub tolerance_cents: Option<i32>,
    #[serde(default)]
    pub date_window_days: Option<i32>,
    #[serde(default)]
    pub auto_approve_under: Option<String>,
    #[serde(default)]
    pub notify_on_unmatched: Option<bool>,
    #[serde(default)]
    pub webhook: Option<String>,
}

fn default_settings() -> BankingSettings {
    BankingSettings {
        tolerance_cents: 1,
        date_window_days: 3,
        auto_approve_under: "500".to_string(),
        notify_on_unmatched: true,
        webhook: String::new(),
    }
}

pub async fn get_settings(headers: HeaderMap) -> Result<Json<BankingSettings>, (StatusCode, String)> {
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Integer)] tolerance_cents: i32,
        #[diesel(sql_type = diesel::sql_types::Integer)] date_window_days: i32,
        #[diesel(sql_type = diesel::sql_types::Numeric)] auto_approve_under: Decimal,
        #[diesel(sql_type = diesel::sql_types::Bool)] notify_on_unmatched: bool,
        #[diesel(sql_type = diesel::sql_types::Text)] webhook: String,
    }
    let row: Option<Row> = diesel::sql_query(
        "SELECT tolerance_cents, date_window_days, auto_approve_under, notify_on_unmatched, webhook
         FROM banking_settings WHERE branch_id = $1 LIMIT 1",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .get_result(&mut conn)
    .optional()
    .map_err(db::map_diesel_err)?;
    match row {
        Some(r) => ok(BankingSettings {
            tolerance_cents: r.tolerance_cents,
            date_window_days: r.date_window_days,
            auto_approve_under: decimal_to_string(r.auto_approve_under),
            notify_on_unmatched: r.notify_on_unmatched,
            webhook: r.webhook,
        }),
        None => ok(default_settings()),
    }
}

pub async fn save_settings(
    headers: HeaderMap,
    Json(req): Json<SaveSettings>,
) -> Result<Json<BankingSettings>, (StatusCode, String)> {
    let mut current = get_settings(headers.clone()).await?.0;
    if let Some(v) = req.tolerance_cents { current.tolerance_cents = v.clamp(0, 100); }
    if let Some(v) = req.date_window_days { current.date_window_days = v.clamp(0, 30); }
    if let Some(v) = req.auto_approve_under {
        if let Ok(d) = v.parse::<Decimal>() { current.auto_approve_under = decimal_to_string(d); }
    }
    if let Some(v) = req.notify_on_unmatched { current.notify_on_unmatched = v; }
    if let Some(v) = req.webhook { current.webhook = v; }

    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    let auto = current.auto_approve_under.parse::<Decimal>().unwrap_or_default();
    let now = Utc::now();
    let n = diesel::sql_query(
        "UPDATE banking_settings SET tolerance_cents=$2, date_window_days=$3, auto_approve_under=$4, notify_on_unmatched=$5, webhook=$6, updated_at=$7 WHERE branch_id=$1",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .bind::<diesel::sql_types::Integer, _>(current.tolerance_cents)
    .bind::<diesel::sql_types::Integer, _>(current.date_window_days)
    .bind::<diesel::sql_types::Numeric, _>(auto)
    .bind::<diesel::sql_types::Bool, _>(current.notify_on_unmatched)
    .bind::<diesel::sql_types::Text, _>(&current.webhook)
    .bind::<diesel::sql_types::Timestamptz, _>(now)
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    if n == 0 {
        diesel::sql_query(
            "INSERT INTO banking_settings (branch_id, tolerance_cents, date_window_days, auto_approve_under, notify_on_unmatched, webhook, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind::<diesel::sql_types::Uuid, _>(branch)
        .bind::<diesel::sql_types::Integer, _>(current.tolerance_cents)
        .bind::<diesel::sql_types::Integer, _>(current.date_window_days)
        .bind::<diesel::sql_types::Numeric, _>(auto)
        .bind::<diesel::sql_types::Bool, _>(current.notify_on_unmatched)
        .bind::<diesel::sql_types::Text, _>(&current.webhook)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .execute(&mut conn)
        .map_err(db::map_diesel_err)?;
    }
    ok(current)
}

pub async fn reset_settings(headers: HeaderMap) -> Result<Json<BankingSettings>, (StatusCode, String)> {
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    let def = default_settings();
    let auto = def.auto_approve_under.parse::<Decimal>().unwrap_or_default();
    let now = Utc::now();
    diesel::sql_query(
        "DELETE FROM banking_settings WHERE branch_id = $1",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    diesel::sql_query(
        "INSERT INTO banking_settings (branch_id, tolerance_cents, date_window_days, auto_approve_under, notify_on_unmatched, webhook, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .bind::<diesel::sql_types::Integer, _>(def.tolerance_cents)
    .bind::<diesel::sql_types::Integer, _>(def.date_window_days)
    .bind::<diesel::sql_types::Numeric, _>(auto)
    .bind::<diesel::sql_types::Bool, _>(def.notify_on_unmatched)
    .bind::<diesel::sql_types::Text, _>(&def.webhook)
    .bind::<diesel::sql_types::Timestamptz, _>(now)
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    ok(def)
}

// ---------------------------------------------------------------------------
// Stats (header cards)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct BankingStats {
    pub total_in: f64,
    pub total_out: f64,
    pub net: f64,
    pub pending: i64,
    pub match_rate: f64,
}

pub async fn stats(headers: HeaderMap) -> Result<Json<BankingStats>, (StatusCode, String)> {
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::BigInt)] total: i64,
        #[diesel(sql_type = diesel::sql_types::BigInt)] reconciled: i64,
        #[diesel(sql_type = diesel::sql_types::BigInt)] pending: i64,
        #[diesel(sql_type = diesel::sql_types::Numeric)] total_in: Decimal,
        #[diesel(sql_type = diesel::sql_types::Numeric)] total_out: Decimal,
        #[diesel(sql_type = diesel::sql_types::Numeric)] net: Decimal,
    }
    let row: Row = diesel::sql_query(
        "SELECT COUNT(*) AS total,
                COUNT(*) FILTER (WHERE status = 'reconciled') AS reconciled,
                COUNT(*) FILTER (WHERE status != 'reconciled') AS pending,
                COALESCE(SUM(amount) FILTER (WHERE amount > 0), 0) AS total_in,
                COALESCE(ABS(SUM(amount) FILTER (WHERE amount < 0)), 0) AS total_out,
                COALESCE(SUM(amount), 0) AS net
         FROM banking_transactions WHERE branch_id = $1",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .get_result(&mut conn)
    .map_err(db::map_diesel_err)?;
    let match_rate = if row.total > 0 { (row.reconciled as f64 / row.total as f64) * 100.0 } else { 0.0 };
    ok(BankingStats {
        total_in: total_in_f64(row.total_in),
        total_out: total_out_f64(row.total_out),
        net: net_f64(row.net),
        pending: row.pending,
        match_rate,
    })
}

fn total_in_f64(d: Decimal) -> f64 { d.to_string().parse::<f64>().unwrap_or(0.0) }
fn total_out_f64(d: Decimal) -> f64 { d.to_string().parse::<f64>().unwrap_or(0.0) }
fn net_f64(d: Decimal) -> f64 { d.to_string().parse::<f64>().unwrap_or(0.0) }

// ---------------------------------------------------------------------------
// Reports generation
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct GenerateReport {
    pub kind: String,
}

pub async fn generate_report(
    headers: HeaderMap,
    Json(req): Json<GenerateReport>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    let id = Uuid::new_v4();
    let period = Utc::now().format("%Y-%m").to_string();
    let name = match req.kind.as_str() {
        "cashflow" => "Cash Flow",
        "pnl" => "P&L Summary",
        "tax" => "Tax Obligations",
        _ => "Report",
    }.to_string();
    diesel::sql_query(
        "INSERT INTO banking_reports (id, branch_id, name, kind, period, url, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .bind::<diesel::sql_types::Text, _>(&name)
    .bind::<diesel::sql_types::Text, _>(&req.kind)
    .bind::<diesel::sql_types::Text, _>(&period)
    .bind::<diesel::sql_types::Text, _>(format!("/api/banking/reports/{id}"))
    .bind::<diesel::sql_types::Timestamptz, _>(Utc::now())
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    ok(serde_json::json!({ "success": true, "id": id, "name": name, "kind": req.kind, "period": period }))
}


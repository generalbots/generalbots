//! Leads, quotes and orders endpoints for the Sales CRM suite app.

use axum::extract::Json;
use axum::http::{HeaderMap, StatusCode};
use chrono::Utc;
use diesel::RunQueryDsl;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db;
use crate::storage::ensure_schema_sync;

fn resolve_branch(headers: &HeaderMap) -> Uuid {
    botcore::shared::tenant::branch_from_claims(headers).unwrap_or_else(Uuid::nil)
}

/// Missing scores are treated as "neutral" instead of being rejected.
fn lead_score(explicit: Option<i32>) -> i32 {
    explicit.unwrap_or(50)
}

/// Empty status values fall back to the lifecycle's entry state.
fn status_or_default<'a>(explicit: &'a str, default: &'a str) -> &'a str {
    if explicit.is_empty() {
        default
    } else {
        explicit
    }
}

/// Generated document numbers are stable, prefixed and always six digits;
/// an operator-supplied number is kept verbatim.
fn document_number(prefix: &str, explicit: &str, timestamp: i64) -> String {
    if explicit.trim().is_empty() {
        format!("{prefix}-{:06}", (timestamp % 1_000_000).abs())
    } else {
        explicit.to_string()
    }
}

/// Malformed money input becomes zero instead of aborting the request.
fn amount_or_zero(raw: &str) -> Decimal {
    raw.parse::<Decimal>().unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Leads
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lead {
    pub id: Uuid,
    pub name: String,
    pub company: String,
    pub source: String,
    pub score: i32,
    pub status: String,
    pub owner: String,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct NewLead {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub company: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub score: Option<i32>,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub owner: String,
}

pub async fn list_leads(headers: HeaderMap) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)] name: String,
        #[diesel(sql_type = diesel::sql_types::Text)] company: String,
        #[diesel(sql_type = diesel::sql_types::Text)] source: String,
        #[diesel(sql_type = diesel::sql_types::Integer)] score: i32,
        #[diesel(sql_type = diesel::sql_types::Text)] status: String,
        #[diesel(sql_type = diesel::sql_types::Text)] owner: String,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)] created_at: chrono::DateTime<Utc>,
    }
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT id, name, company, source, score, status, owner, created_at
         FROM sales_leads WHERE branch_id = $1 ORDER BY created_at DESC LIMIT 500",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .load(&mut conn)
    .map_err(db::map_diesel_err)?;
    let items: Vec<serde_json::Value> = rows.into_iter().map(|r| serde_json::json!({
        "id": r.id, "name": r.name, "company": r.company, "source": r.source,
        "score": r.score, "status": r.status, "owner": r.owner, "created_at": r.created_at,
    })).collect();
    Ok(Json(serde_json::json!({ "items": items })))
}

pub async fn create_lead(
    headers: HeaderMap,
    Json(req): Json<NewLead>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    ensure_schema_sync()?;
    if req.name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "lead name is required".to_string()));
    }
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    let id = Uuid::new_v4();
    let now = Utc::now();
    diesel::sql_query(
        "INSERT INTO sales_leads (id, name, company, source, score, status, owner, created_at, branch_id)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .bind::<diesel::sql_types::Text, _>(&req.name)
    .bind::<diesel::sql_types::Text, _>(&req.company)
    .bind::<diesel::sql_types::Text, _>(&req.source)
    .bind::<diesel::sql_types::Integer, _>(lead_score(req.score))
    .bind::<diesel::sql_types::Text, _>(status_or_default(&req.status, "new"))
    .bind::<diesel::sql_types::Text, _>(&req.owner)
    .bind::<diesel::sql_types::Timestamptz, _>(now)
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Json(serde_json::json!({ "item": { "id": id, "created_at": now } })))
}

// ---------------------------------------------------------------------------
// Quotes
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    pub id: Uuid,
    pub quote_number: String,
    pub title: String,
    pub customer: String,
    pub amount: String,
    pub valid_until: Option<chrono::NaiveDate>,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct NewQuote {
    #[serde(default)]
    pub quote_number: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub customer: String,
    #[serde(default)]
    pub amount: String,
    #[serde(default)]
    pub valid_until: Option<chrono::NaiveDate>,
    #[serde(default)]
    pub status: String,
}

pub async fn list_quotes(headers: HeaderMap) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)] quote_number: String,
        #[diesel(sql_type = diesel::sql_types::Text)] title: String,
        #[diesel(sql_type = diesel::sql_types::Text)] customer: String,
        #[diesel(sql_type = diesel::sql_types::Numeric)] amount: Decimal,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Date>)] valid_until: Option<chrono::NaiveDate>,
        #[diesel(sql_type = diesel::sql_types::Text)] status: String,
    }
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT id, quote_number, title, customer, amount, valid_until, status
         FROM sales_quotes WHERE branch_id = $1 ORDER BY created_at DESC LIMIT 500",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .load(&mut conn)
    .map_err(db::map_diesel_err)?;
    let items: Vec<serde_json::Value> = rows.into_iter().map(|r| serde_json::json!({
        "id": r.id, "quote_number": r.quote_number, "title": r.title, "customer": r.customer,
        "amount": r.amount.to_string(), "valid_until": r.valid_until, "status": r.status,
    })).collect();
    Ok(Json(serde_json::json!({ "items": items })))
}

pub async fn create_quote(
    headers: HeaderMap,
    Json(req): Json<NewQuote>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let amount = amount_or_zero(&req.amount);
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    let id = Uuid::new_v4();
    let qnum = document_number("Q", &req.quote_number, Utc::now().timestamp());
    diesel::sql_query(
        "INSERT INTO sales_quotes (id, quote_number, title, customer, amount, valid_until, status, created_at, branch_id)
         VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), $8)",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .bind::<diesel::sql_types::Text, _>(&qnum)
    .bind::<diesel::sql_types::Text, _>(&req.title)
    .bind::<diesel::sql_types::Text, _>(&req.customer)
    .bind::<diesel::sql_types::Numeric, _>(amount)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Date>, _>(req.valid_until)
    .bind::<diesel::sql_types::Text, _>(status_or_default(&req.status, "draft"))
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Json(serde_json::json!({ "item": { "id": id, "quote_number": qnum } })))
}

// ---------------------------------------------------------------------------
// Orders
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: Uuid,
    pub order_number: String,
    pub customer: String,
    pub items: i32,
    pub total: String,
    pub status: String,
    pub delivery: String,
}

#[derive(Debug, Deserialize)]
pub struct NewOrder {
    #[serde(default)]
    pub order_number: String,
    #[serde(default)]
    pub customer: String,
    #[serde(default)]
    pub items: Option<i32>,
    #[serde(default)]
    pub total: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub delivery: String,
}

pub async fn list_orders(headers: HeaderMap) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)] order_number: String,
        #[diesel(sql_type = diesel::sql_types::Text)] customer: String,
        #[diesel(sql_type = diesel::sql_types::Integer)] items: i32,
        #[diesel(sql_type = diesel::sql_types::Numeric)] total: Decimal,
        #[diesel(sql_type = diesel::sql_types::Text)] status: String,
        #[diesel(sql_type = diesel::sql_types::Text)] delivery: String,
    }
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT id, order_number, customer, items, total, status, delivery
         FROM sales_orders WHERE branch_id = $1 ORDER BY created_at DESC LIMIT 500",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .load(&mut conn)
    .map_err(db::map_diesel_err)?;
    let items: Vec<serde_json::Value> = rows.into_iter().map(|r| serde_json::json!({
        "id": r.id, "order_number": r.order_number, "customer": r.customer, "items": r.items,
        "total": r.total.to_string(), "status": r.status, "delivery": r.delivery,
    })).collect();
    Ok(Json(serde_json::json!({ "items": items })))
}

pub async fn create_order(
    headers: HeaderMap,
    Json(req): Json<NewOrder>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let total = amount_or_zero(&req.total);
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    let id = Uuid::new_v4();
    let onum = document_number("O", &req.order_number, Utc::now().timestamp());
    diesel::sql_query(
        "INSERT INTO sales_orders (id, order_number, customer, items, total, status, delivery, created_at, branch_id)
         VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), $8)",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .bind::<diesel::sql_types::Text, _>(&onum)
    .bind::<diesel::sql_types::Text, _>(&req.customer)
    .bind::<diesel::sql_types::Integer, _>(req.items.unwrap_or(0))
    .bind::<diesel::sql_types::Numeric, _>(total)
    .bind::<diesel::sql_types::Text, _>(status_or_default(&req.status, "pending"))
    .bind::<diesel::sql_types::Text, _>(&req.delivery)
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Json(serde_json::json!({ "item": { "id": id, "order_number": onum } })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lead_defaults_keep_the_documented_score_and_status() {
        assert_eq!(lead_score(None), 50);
        assert_eq!(lead_score(Some(0)), 0);
        assert_eq!(lead_score(Some(95)), 95);
        assert_eq!(status_or_default("", "new"), "new");
        assert_eq!(status_or_default("qualified", "new"), "qualified");
    }

    #[test]
    fn test_generated_document_numbers_are_prefixed_and_six_digits() {
        assert_eq!(document_number("Q", "", 1_234_567), "Q-234567");
        assert_eq!(document_number("O", "   ", -1), "O-000001");
        assert_eq!(document_number("Q", "Q-CUSTOM", 42), "Q-CUSTOM");
        assert_eq!(document_number("O", "", 0), "O-000000");
    }

    #[test]
    fn test_invalid_amounts_fall_back_to_zero() {
        assert_eq!(amount_or_zero("12.50"), Decimal::new(1250, 2));
        assert_eq!(amount_or_zero("not-a-number"), Decimal::ZERO);
        assert_eq!(amount_or_zero(""), Decimal::ZERO);
    }

    #[test]
    fn test_new_lead_payload_tolerates_missing_optional_fields() {
        let parsed: Result<NewLead, _> = serde_json::from_str(r#"{"name":"Ada"}"#);
        assert!(parsed.is_ok(), "a lead with only a name must deserialize");
        if let Ok(lead) = parsed {
            assert_eq!(lead.name, "Ada");
            assert_eq!(lead.score, None);
            assert!(lead.company.is_empty());
            assert!(lead.status.is_empty());
        }
    }

    #[test]
    fn test_new_quote_and_order_payloads_tolerate_missing_optional_fields() {
        let quote: Result<NewQuote, _> = serde_json::from_str(r#"{"title":"Proposal"}"#);
        assert!(quote.is_ok(), "a quote with only a title must deserialize");
        if let Ok(quote) = quote {
            assert_eq!(quote.title, "Proposal");
            assert!(quote.valid_until.is_none());
            assert!(quote.amount.is_empty());
        }
        let order: Result<NewOrder, _> = serde_json::from_str(r#"{"customer":"Ada"}"#);
        assert!(order.is_ok(), "an order with only a customer must deserialize");
        if let Ok(order) = order {
            assert_eq!(order.customer, "Ada");
            assert!(order.items.is_none());
            assert!(order.total.is_empty());
        }
    }
}

//! HTTP handlers for creating/updating retail branches, promotions and suppliers.

use axum::extract::{Json, Path};
use axum::http::{HeaderMap, StatusCode};
use diesel::prelude::*;
use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

use crate::db;

fn resolve_branch(headers: &HeaderMap) -> Uuid {
    botcore::shared::tenant::branch_from_claims(headers).unwrap_or_else(Uuid::nil)
}

// ---------------------------------------------------------------------------
// Branches
// ---------------------------------------------------------------------------

/// Request body for creating or updating a retail branch.
#[derive(Debug, Deserialize)]
pub struct BranchPayload {
    #[serde(default)]
    code: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    address: String,
    #[serde(default)]
    manager: String,
    #[serde(default)]
    status: String,
    #[serde(default)]
    stock_value: Option<String>,
}

/// Creates a new retail branch for the caller's branch/tenant.
pub async fn create_branch(
    headers: HeaderMap,
    Json(req): Json<BranchPayload>,
) -> Result<axum::Json<Value>, (StatusCode, String)> {
    db::ensure_schema_sync()?;
    if req.name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "branch name is required".to_string()));
    }
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    let id = Uuid::new_v4();
    let stock_value = req.stock_value.and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0);
    diesel::sql_query(
        "INSERT INTO retail_branches (id, branch_id, code, name, address, manager, stock_value, status, pricing_rules)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, '[]'::jsonb)",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .bind::<diesel::sql_types::Text, _>(&req.code)
    .bind::<diesel::sql_types::Text, _>(&req.name)
    .bind::<diesel::sql_types::Text, _>(&req.address)
    .bind::<diesel::sql_types::Text, _>(&req.manager)
    .bind::<diesel::sql_types::Double, _>(stock_value)
    .bind::<diesel::sql_types::Text, _>(if req.status.is_empty() { "active" } else { &req.status })
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(axum::Json(serde_json::json!({ "id": id, "name": req.name, "status": req.status })))
}

/// Updates an existing retail branch identified by its id.
pub async fn update_branch(
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(req): Json<BranchPayload>,
) -> Result<axum::Json<Value>, (StatusCode, String)> {
    db::ensure_schema_sync()?;
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    let n = diesel::sql_query(
        "UPDATE retail_branches SET code=$1, name=$2, address=$3, manager=$4, status=$5 WHERE id=$6 AND branch_id=$7",
    )
    .bind::<diesel::sql_types::Text, _>(&req.code)
    .bind::<diesel::sql_types::Text, _>(&req.name)
    .bind::<diesel::sql_types::Text, _>(&req.address)
    .bind::<diesel::sql_types::Text, _>(&req.manager)
    .bind::<diesel::sql_types::Text, _>(if req.status.is_empty() { "active" } else { &req.status })
    .bind::<diesel::sql_types::Uuid, _>(id)
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    if n == 0 { return Err((StatusCode::NOT_FOUND, format!("Branch {id} not found"))); }
    Ok(axum::Json(serde_json::json!({ "id": id, "updated": true })))
}

// ---------------------------------------------------------------------------
// Promotions
// ---------------------------------------------------------------------------

/// Request body for creating a retail promotion.
#[derive(Debug, Deserialize)]
pub struct PromotionPayload {
    #[serde(default)]
    name: String,
    #[serde(default)]
    r#type: String,
    #[serde(default)]
    discount: String,
    #[serde(default)]
    valid_from: Option<chrono::NaiveDate>,
    #[serde(default)]
    valid_to: Option<chrono::NaiveDate>,
    #[serde(default)]
    status: String,
}

/// Creates a new retail promotion for the caller's branch/tenant.
pub async fn create_promotion(
    headers: HeaderMap,
    Json(req): Json<PromotionPayload>,
) -> Result<axum::Json<Value>, (StatusCode, String)> {
    db::ensure_schema_sync()?;
    if req.name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "promotion name is required".to_string()));
    }
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    let id = Uuid::new_v4();
    diesel::sql_query(
        "INSERT INTO retail_promotions (id, branch_id, name, type, discount, valid_from, valid_to, status, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW())",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .bind::<diesel::sql_types::Text, _>(&req.name)
    .bind::<diesel::sql_types::Text, _>(&req.r#type)
    .bind::<diesel::sql_types::Text, _>(&req.discount)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Date>, _>(req.valid_from)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Date>, _>(req.valid_to)
    .bind::<diesel::sql_types::Text, _>(if req.status.is_empty() { "active" } else { &req.status })
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(axum::Json(serde_json::json!({ "id": id, "name": req.name })))
}

// ---------------------------------------------------------------------------
// Suppliers
// ---------------------------------------------------------------------------

/// Request body for creating a retail supplier.
#[derive(Debug, Deserialize)]
pub struct SupplierPayload {
    #[serde(default)]
    cnpj: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    contact: String,
    #[serde(default)]
    email: String,
    #[serde(default)]
    lead_time_days: Option<i32>,
    #[serde(default)]
    rating: Option<f64>,
}

/// Creates a new retail supplier for the caller's branch/tenant.
pub async fn create_supplier(
    headers: HeaderMap,
    Json(req): Json<SupplierPayload>,
) -> Result<axum::Json<Value>, (StatusCode, String)> {
    db::ensure_schema_sync()?;
    if req.name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "supplier name is required".to_string()));
    }
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    let id = Uuid::new_v4();
    diesel::sql_query(
        "INSERT INTO retail_suppliers (id, branch_id, cnpj, name, contact, email, lead_time_days, rating, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW())",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .bind::<diesel::sql_types::Text, _>(&req.cnpj)
    .bind::<diesel::sql_types::Text, _>(&req.name)
    .bind::<diesel::sql_types::Text, _>(&req.contact)
    .bind::<diesel::sql_types::Text, _>(&req.email)
    .bind::<diesel::sql_types::Integer, _>(req.lead_time_days.unwrap_or(0))
    .bind::<diesel::sql_types::Double, _>(req.rating.unwrap_or(0.0))
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(axum::Json(serde_json::json!({ "id": id, "name": req.name })))
}

// ---------------------------------------------------------------------------
// List handlers (replacement for the central stub GETs in src/retail/mod.rs)
// ---------------------------------------------------------------------------

/// Lists branches for the caller's branch/tenant.
pub async fn list_branches(
    headers: HeaderMap,
) -> Result<axum::Json<serde_json::Value>, (StatusCode, String)> {
    db::ensure_schema_sync()?;
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)] code: String,
        #[diesel(sql_type = diesel::sql_types::Text)] name: String,
        #[diesel(sql_type = diesel::sql_types::Text)] address: String,
        #[diesel(sql_type = diesel::sql_types::Text)] manager: String,
        #[diesel(sql_type = diesel::sql_types::Numeric)] stock_value: rust_decimal::Decimal,
        #[diesel(sql_type = diesel::sql_types::Text)] status: String,
        #[diesel(sql_type = diesel::sql_types::Json)] pricing_rules: serde_json::Value,
    }
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT id, code, name, address, manager, stock_value, status, pricing_rules
         FROM retail_branches WHERE branch_id = $1 ORDER BY name ASC LIMIT 500",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .load(&mut conn)
    .map_err(db::map_diesel_err)?;
    let items: Vec<serde_json::Value> = rows.into_iter().map(|r| serde_json::json!({
        "id": r.id, "code": r.code, "name": r.name, "address": r.address, "manager": r.manager,
        "stock_value": r.stock_value.to_string(), "status": r.status, "pricing_rules": r.pricing_rules,
    })).collect();
    Ok(axum::Json(serde_json::json!({ "items": items })))
}

/// Lists promotions for the caller's branch/tenant.
pub async fn list_promotions(
    headers: HeaderMap,
) -> Result<axum::Json<serde_json::Value>, (StatusCode, String)> {
    db::ensure_schema_sync()?;
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)] name: String,
        #[diesel(sql_type = diesel::sql_types::Text)] r#type: String,
        #[diesel(sql_type = diesel::sql_types::Text)] discount: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Date>)] valid_from: Option<chrono::NaiveDate>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Date>)] valid_to: Option<chrono::NaiveDate>,
        #[diesel(sql_type = diesel::sql_types::Text)] status: String,
    }
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT id, name, type, discount, valid_from, valid_to, status
         FROM retail_promotions WHERE branch_id = $1 ORDER BY created_at DESC LIMIT 500",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .load(&mut conn)
    .map_err(db::map_diesel_err)?;
    let items: Vec<serde_json::Value> = rows.into_iter().map(|r| serde_json::json!({
        "id": r.id, "name": r.name, "type": r.r#type, "discount": r.discount,
        "valid_from": r.valid_from, "valid_to": r.valid_to, "status": r.status,
    })).collect();
    Ok(axum::Json(serde_json::json!({ "items": items })))
}

/// Lists suppliers for the caller's branch/tenant.
pub async fn list_suppliers(
    headers: HeaderMap,
) -> Result<axum::Json<serde_json::Value>, (StatusCode, String)> {
    db::ensure_schema_sync()?;
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)] cnpj: String,
        #[diesel(sql_type = diesel::sql_types::Text)] name: String,
        #[diesel(sql_type = diesel::sql_types::Text)] contact: String,
        #[diesel(sql_type = diesel::sql_types::Text)] email: String,
        #[diesel(sql_type = diesel::sql_types::Integer)] lead_time_days: i32,
        #[diesel(sql_type = diesel::sql_types::Numeric)] rating: rust_decimal::Decimal,
    }
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT id, cnpj, name, contact, email, lead_time_days, rating
         FROM retail_suppliers WHERE branch_id = $1 ORDER BY name ASC LIMIT 500",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .load(&mut conn)
    .map_err(db::map_diesel_err)?;
    let items: Vec<serde_json::Value> = rows.into_iter().map(|r| serde_json::json!({
        "id": r.id, "cnpj": r.cnpj, "name": r.name, "contact": r.contact, "email": r.email,
        "lead_time_days": r.lead_time_days, "rating": r.rating.to_string(),
    })).collect();
    Ok(axum::Json(serde_json::json!({ "items": items })))
}

use axum::extract::{Json, Path};
use axum::http::{HeaderMap, StatusCode};
use chrono::Utc;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use diesel::RunQueryDsl;
use diesel::OptionalExtension;

use crate::db;
use crate::storage::ensure_schema_sync;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: Uuid,
    pub sku: String,
    pub name: String,
    pub price: String,
    pub stock: i64,
    pub category: String,
    pub active: bool,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderItem {
    pub product_id: Uuid,
    pub quantity: i64,
    pub unit_price: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: Uuid,
    pub items: Vec<OrderItem>,
    pub total: String,
    pub status: String,
    pub payment_method: String,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct NewProduct {
    pub sku: String,
    pub name: String,
    pub price: String,
    pub stock: i64,
    pub category: String,
}

#[derive(Debug, Deserialize)]
pub struct NewOrder {
    pub items: Vec<OrderItem>,
    pub payment_method: String,
}

/// Resolves the caller's tenant branch from the server-minted JWT claims
/// (issue #734). Falls back to the global nil branch so anonymous/system
/// callers keep working, but every query is still constrained by the resolved
/// branch — a tenant can never see another tenant's rows.
fn resolve_branch(headers: &HeaderMap) -> Uuid {
    botcore::shared::tenant::branch_from_claims(headers).unwrap_or_else(Uuid::nil)
}

fn parse_decimal(s: &str) -> Result<Decimal, (StatusCode, String)> {
    s.parse::<Decimal>().map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid decimal '{s}': {e}")))
}

pub async fn list_products(headers: HeaderMap) -> Result<Json<Vec<Product>>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)] sku: String,
        #[diesel(sql_type = diesel::sql_types::Text)] name: String,
        #[diesel(sql_type = diesel::sql_types::Numeric)] price: Decimal,
        #[diesel(sql_type = diesel::sql_types::BigInt)] stock: i64,
        #[diesel(sql_type = diesel::sql_types::Text)] category: String,
        #[diesel(sql_type = diesel::sql_types::Bool)] active: bool,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)] created_at: chrono::DateTime<Utc>,
    }
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT id, sku, name, price, stock, category, active, created_at
         FROM pos_products WHERE branch_id = $1 ORDER BY name ASC LIMIT 500",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .load(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Json(rows.into_iter().map(|r| Product {
        id: r.id, sku: r.sku, name: r.name, price: r.price.to_string(),
        stock: r.stock, category: r.category, active: r.active, created_at: r.created_at,
    }).collect()))
}

pub async fn create_product(headers: HeaderMap, Json(req): Json<NewProduct>) -> Result<Json<Product>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let branch = resolve_branch(&headers);
    let price = parse_decimal(&req.price)?;
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    let id = Uuid::new_v4();
    let now = Utc::now();
    diesel::sql_query(
        "INSERT INTO pos_products (id, sku, name, price, stock, category, active, created_at, branch_id)
         VALUES ($1, $2, $3, $4, $5, $6, true, $7, $8)",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .bind::<diesel::sql_types::Text, _>(&req.sku)
    .bind::<diesel::sql_types::Text, _>(&req.name)
    .bind::<diesel::sql_types::Numeric, _>(price)
    .bind::<diesel::sql_types::BigInt, _>(req.stock)
    .bind::<diesel::sql_types::Text, _>(&req.category)
    .bind::<diesel::sql_types::Timestamptz, _>(now)
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Json(Product {
        id, sku: req.sku, name: req.name, price: price.to_string(),
        stock: req.stock, category: req.category, active: true, created_at: now,
    }))
}

pub async fn list_orders(headers: HeaderMap) -> Result<Json<Vec<Order>>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Jsonb)] items: serde_json::Value,
        #[diesel(sql_type = diesel::sql_types::Numeric)] total: Decimal,
        #[diesel(sql_type = diesel::sql_types::Text)] status: String,
        #[diesel(sql_type = diesel::sql_types::Text)] payment_method: String,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)] created_at: chrono::DateTime<Utc>,
    }
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT id, items, total, status, payment_method, created_at
         FROM pos_orders WHERE branch_id = $1 ORDER BY created_at DESC LIMIT 500",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .load(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Json(rows.into_iter().map(|r| Order {
        id: r.id,
        items: r.items.as_array().map(|a| a.iter().filter_map(|v| {
            Some(OrderItem {
                product_id: v.get("product_id").and_then(|p| p.as_str()).and_then(|s| Uuid::parse_str(s).ok())?,
                quantity: v.get("quantity").and_then(|q| q.as_i64()).unwrap_or(0),
                unit_price: v.get("unit_price").and_then(|p| p.as_str()).unwrap_or("0").to_string(),
            })
        }).collect()).unwrap_or_default(),
        total: r.total.to_string(),
        status: r.status,
        payment_method: r.payment_method,
        created_at: r.created_at,
    }).collect()))
}

pub async fn create_order(headers: HeaderMap, Json(req): Json<NewOrder>) -> Result<Json<Order>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    let mut total = Decimal::ZERO;
    for item in &req.items {
        let unit = parse_decimal(&item.unit_price)?;
        total += unit * Decimal::from(item.quantity);
    }
    let id = Uuid::new_v4();
    let now = Utc::now();
    let items_json = serde_json::to_value(&req.items).map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Serialize: {e}"))
    })?;
    diesel::sql_query(
        "INSERT INTO pos_orders (id, items, total, status, payment_method, created_at, branch_id)
         VALUES ($1, $2, $3, 'created', $4, $5, $6)",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .bind::<diesel::sql_types::Jsonb, _>(&items_json)
    .bind::<diesel::sql_types::Numeric, _>(total)
    .bind::<diesel::sql_types::Text, _>(&req.payment_method)
    .bind::<diesel::sql_types::Timestamptz, _>(now)
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Json(Order {
        id, items: req.items, total: total.to_string(), status: "created".to_string(),
        payment_method: req.payment_method, created_at: now,
    }))
}

pub async fn get_order(headers: HeaderMap, Path(id): Path<String>) -> Result<Json<Order>, (StatusCode, String)> {
    let parsed = Uuid::parse_str(&id)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid id '{id}': {e}")))?;
    ensure_schema_sync()?;
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Jsonb)] items: serde_json::Value,
        #[diesel(sql_type = diesel::sql_types::Numeric)] total: Decimal,
        #[diesel(sql_type = diesel::sql_types::Text)] status: String,
        #[diesel(sql_type = diesel::sql_types::Text)] payment_method: String,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)] created_at: chrono::DateTime<Utc>,
    }
    let row: Option<Row> = diesel::sql_query(
        "SELECT id, items, total, status, payment_method, created_at FROM pos_orders WHERE id = $1 AND branch_id = $2",
    )
    .bind::<diesel::sql_types::Uuid, _>(parsed)
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .get_result(&mut conn)
    .optional()
    .map_err(db::map_diesel_err)?;
    let r = row.ok_or((StatusCode::NOT_FOUND, format!("Order {id} not found")))?;
    Ok(Json(Order {
        id: r.id,
        items: r.items.as_array().map(|a| a.iter().filter_map(|v| {
            Some(OrderItem {
                product_id: v.get("product_id").and_then(|p| p.as_str()).and_then(|s| Uuid::parse_str(s).ok())?,
                quantity: v.get("quantity").and_then(|q| q.as_i64()).unwrap_or(0),
                unit_price: v.get("unit_price").and_then(|p| p.as_str()).unwrap_or("0").to_string(),
            })
        }).collect()).unwrap_or_default(),
        total: r.total.to_string(),
        status: r.status,
        payment_method: r.payment_method,
        created_at: r.created_at,
    }))
}
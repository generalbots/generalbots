pub mod schema;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, put},
    Json, Router,
};

use bigdecimal::{BigDecimal, ToPrimitive};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

use crate::schema::{
    inventory_movements, price_list_items, price_lists, product_categories, product_variants,
    products, services,
};

pub type DbPool = r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>>;

pub type GetDefaultBotFn = fn(&mut diesel::PgConnection) -> (Uuid, String);

pub fn get_bot_context(pool: &DbPool, get_default_bot: &Option<GetDefaultBotFn>) -> (Uuid, Uuid) {
    let Ok(mut conn) = pool.get() else {
        return (Uuid::nil(), Uuid::nil());
    };
    match get_default_bot {
        Some(f) => {
            let (bot_id, _) = f(&mut conn);
            let org_id = Uuid::nil();
            (org_id, bot_id)
        }
        None => (Uuid::nil(), Uuid::nil()),
    }
}

pub struct ProductsState {
    pub pool: Arc<DbPool>,
    pub get_default_bot: Option<GetDefaultBotFn>,
}

fn bd(val: f64) -> BigDecimal {
    BigDecimal::from_str(&val.to_string()).unwrap_or_else(|_| BigDecimal::from(0))
}

fn bd_to_f64(val: &BigDecimal) -> f64 {
    val.to_f64().unwrap_or(0.0)
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn format_currency(amount: f64, currency: &str) -> String {
    match currency.to_uppercase().as_str() {
        "USD" => format!("${:.2}", amount),
        "EUR" => format!("\u{20ac}{:.2}", amount),
        "GBP" => format!("\u{00a3}{:.2}", amount),
        "BRL" => format!("R${:.2}", amount),
        _ => format!("{:.2} {}", amount, currency),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = products)]
pub struct Product {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub sku: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub product_type: String,
    pub price: BigDecimal,
    pub cost: Option<BigDecimal>,
    pub currency: String,
    pub tax_rate: BigDecimal,
    pub unit: String,
    pub stock_quantity: i32,
    pub low_stock_threshold: i32,
    pub is_active: bool,
    pub images: serde_json::Value,
    pub attributes: serde_json::Value,
    pub weight: Option<BigDecimal>,
    pub dimensions: Option<serde_json::Value>,
    pub barcode: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = services)]
pub struct Service {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub service_type: String,
    pub hourly_rate: Option<BigDecimal>,
    pub fixed_price: Option<BigDecimal>,
    pub currency: String,
    pub duration_minutes: Option<i32>,
    pub is_active: bool,
    pub attributes: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = product_categories)]
pub struct ProductCategory {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub parent_id: Option<Uuid>,
    pub slug: Option<String>,
    pub image_url: Option<String>,
    pub sort_order: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = price_lists)]
pub struct PriceList {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub currency: String,
    pub is_default: bool,
    pub valid_from: Option<chrono::NaiveDate>,
    pub valid_until: Option<chrono::NaiveDate>,
    pub customer_group: Option<String>,
    pub discount_percent: BigDecimal,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = price_list_items)]
pub struct PriceListItem {
    pub id: Uuid,
    pub price_list_id: Uuid,
    pub product_id: Option<Uuid>,
    pub service_id: Option<Uuid>,
    pub price: BigDecimal,
    pub min_quantity: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = inventory_movements)]
pub struct InventoryMovement {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub product_id: Uuid,
    pub movement_type: String,
    pub quantity: i32,
    pub reference_type: Option<String>,
    pub reference_id: Option<Uuid>,
    pub notes: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = product_variants)]
pub struct ProductVariant {
    pub id: Uuid,
    pub product_id: Uuid,
    pub sku: Option<String>,
    pub name: String,
    pub price_adjustment: BigDecimal,
    pub stock_quantity: i32,
    pub attributes: serde_json::Value,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub global_trade_number: Option<String>,
    pub net_weight: Option<BigDecimal>,
    pub gross_weight: Option<BigDecimal>,
    pub width: Option<BigDecimal>,
    pub height: Option<BigDecimal>,
    pub length: Option<BigDecimal>,
    pub color: Option<String>,
    pub size: Option<String>,
    pub images: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct CreateProductRequest {
    pub name: String,
    pub description: Option<String>,
    pub sku: Option<String>,
    pub category: Option<String>,
    pub product_type: Option<String>,
    pub price: f64,
    pub cost: Option<f64>,
    pub currency: Option<String>,
    pub tax_rate: Option<f64>,
    pub unit: Option<String>,
    pub stock_quantity: Option<i32>,
    pub low_stock_threshold: Option<i32>,
    pub images: Option<Vec<String>>,
    pub barcode: Option<String>,
    pub weight: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProductRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub sku: Option<String>,
    pub category: Option<String>,
    pub price: Option<f64>,
    pub cost: Option<f64>,
    pub tax_rate: Option<f64>,
    pub unit: Option<String>,
    pub stock_quantity: Option<i32>,
    pub low_stock_threshold: Option<i32>,
    pub is_active: Option<bool>,
    pub barcode: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateServiceRequest {
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub service_type: Option<String>,
    pub hourly_rate: Option<f64>,
    pub fixed_price: Option<f64>,
    pub currency: Option<String>,
    pub duration_minutes: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateServiceRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub hourly_rate: Option<f64>,
    pub fixed_price: Option<f64>,
    pub duration_minutes: Option<i32>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCategoryRequest {
    pub name: String,
    pub description: Option<String>,
    pub parent_id: Option<Uuid>,
    pub slug: Option<String>,
    pub image_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePriceListRequest {
    pub name: String,
    pub description: Option<String>,
    pub currency: Option<String>,
    pub discount_percent: Option<f64>,
    pub customer_group: Option<String>,
    pub valid_from: Option<String>,
    pub valid_until: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AdjustStockRequest {
    pub quantity: i32,
    pub movement_type: String,
    pub notes: Option<String>,
    pub reference_type: Option<String>,
    pub reference_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub search: Option<String>,
    pub category: Option<String>,
    pub is_active: Option<bool>,
    pub low_stock: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct ProductQuery {
    pub category: Option<String>,
    pub status: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ProductStats {
    pub total_products: i64,
    pub active_products: i64,
    pub total_services: i64,
    pub active_services: i64,
    pub low_stock_count: i64,
    pub total_stock_value: f64,
    pub categories_count: i64,
    pub price_lists_count: i64,
}

#[derive(Debug, Serialize)]
pub struct ProductWithVariants {
    pub product: Product,
    pub variants: Vec<ProductVariant>,
}

pub async fn create_product(
    State(state): State<Arc<ProductsState>>,
    Json(req): Json<CreateProductRequest>,
) -> Result<Json<Product>, (StatusCode, String)> {
    let mut conn = state.pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state.pool, &state.get_default_bot);
    let id = Uuid::new_v4();
    let now = Utc::now();

    let product = Product {
        id,
        org_id,
        bot_id,
        sku: req.sku,
        name: req.name,
        description: req.description,
        category: req.category,
        product_type: req.product_type.unwrap_or_else(|| "physical".to_string()),
        price: bd(req.price),
        cost: req.cost.map(bd),
        currency: req.currency.unwrap_or_else(|| "USD".to_string()),
        tax_rate: bd(req.tax_rate.unwrap_or(0.0)),
        unit: req.unit.unwrap_or_else(|| "unit".to_string()),
        stock_quantity: req.stock_quantity.unwrap_or(0),
        low_stock_threshold: req.low_stock_threshold.unwrap_or(10),
        is_active: true,
        images: serde_json::json!(req.images.unwrap_or_default()),
        attributes: serde_json::json!({}),
        weight: req.weight.map(bd),
        dimensions: None,
        barcode: req.barcode,
        created_at: now,
        updated_at: now,
    };

    diesel::insert_into(products::table)
        .values(&product)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert error: {e}")))?;

    Ok(Json(product))
}

pub async fn list_products(
    State(state): State<Arc<ProductsState>>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<Product>>, (StatusCode, String)> {
    let mut conn = state.pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state.pool, &state.get_default_bot);
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    let mut q = products::table
        .filter(products::org_id.eq(org_id))
        .filter(products::bot_id.eq(bot_id))
        .into_boxed();

    if let Some(is_active) = query.is_active {
        q = q.filter(products::is_active.eq(is_active));
    }

    if let Some(category) = query.category {
        q = q.filter(products::category.eq(category));
    }

    if let Some(true) = query.low_stock {
        q = q.filter(products::stock_quantity.le(products::low_stock_threshold));
    }

    if let Some(search) = query.search {
        let pattern = format!("%{search}%");
        q = q.filter(
            products::name
                .ilike(pattern.clone())
                .or(products::sku.ilike(pattern.clone()))
                .or(products::description.ilike(pattern)),
        );
    }

    let prods: Vec<Product> = q
        .order(products::name.asc())
        .limit(limit)
        .offset(offset)
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    Ok(Json(prods))
}

pub async fn get_product(
    State(state): State<Arc<ProductsState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<ProductWithVariants>, (StatusCode, String)> {
    let mut conn = state.pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let product: Product = products::table
        .filter(products::id.eq(id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Product not found".to_string()))?;

    let variants: Vec<ProductVariant> = product_variants::table
        .filter(product_variants::product_id.eq(id))
        .order(product_variants::name.asc())
        .load(&mut conn)
        .unwrap_or_default();

    Ok(Json(ProductWithVariants { product, variants }))
}

pub async fn update_product(
    State(state): State<Arc<ProductsState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateProductRequest>,
) -> Result<Json<Product>, (StatusCode, String)> {
    let mut conn = state.pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let now = Utc::now();

    diesel::update(products::table.filter(products::id.eq(id)))
        .set(products::updated_at.eq(now))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;

    if let Some(name) = req.name {
        diesel::update(products::table.filter(products::id.eq(id)))
            .set(products::name.eq(name))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    if let Some(description) = req.description {
        diesel::update(products::table.filter(products::id.eq(id)))
            .set(products::description.eq(description))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    if let Some(price) = req.price {
        diesel::update(products::table.filter(products::id.eq(id)))
            .set(products::price.eq(bd(price)))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    if let Some(stock_quantity) = req.stock_quantity {
        diesel::update(products::table.filter(products::id.eq(id)))
            .set(products::stock_quantity.eq(stock_quantity))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    if let Some(is_active) = req.is_active {
        diesel::update(products::table.filter(products::id.eq(id)))
            .set(products::is_active.eq(is_active))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    if let Some(category) = req.category {
        diesel::update(products::table.filter(products::id.eq(id)))
            .set(products::category.eq(category))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    let product: Product = products::table
        .filter(products::id.eq(id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Product not found".to_string()))?;

    Ok(Json(product))
}

pub async fn delete_product(
    State(state): State<Arc<ProductsState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut conn = state.pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    diesel::delete(products::table.filter(products::id.eq(id)))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Delete error: {e}")))?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn adjust_stock(
    State(state): State<Arc<ProductsState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<AdjustStockRequest>,
) -> Result<Json<Product>, (StatusCode, String)> {
    let mut conn = state.pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state.pool, &state.get_default_bot);
    let now = Utc::now();

    let product: Product = products::table
        .filter(products::id.eq(id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Product not found".to_string()))?;

    let new_quantity = match req.movement_type.as_str() {
        "in" | "purchase" | "return" | "adjustment_add" => product.stock_quantity + req.quantity,
        "out" | "sale" | "adjustment_remove" | "damage" => product.stock_quantity - req.quantity,
        "set" => req.quantity,
        _ => product.stock_quantity + req.quantity,
    };

    diesel::update(products::table.filter(products::id.eq(id)))
        .set((
            products::stock_quantity.eq(new_quantity),
            products::updated_at.eq(now),
        ))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;

    let movement = InventoryMovement {
        id: Uuid::new_v4(),
        org_id,
        bot_id,
        product_id: id,
        movement_type: req.movement_type,
        quantity: req.quantity,
        reference_type: req.reference_type,
        reference_id: req.reference_id,
        notes: req.notes,
        created_by: None,
        created_at: now,
    };

    diesel::insert_into(inventory_movements::table)
        .values(&movement)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert error: {e}")))?;

    let updated: Product = products::table
        .filter(products::id.eq(id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Product not found".to_string()))?;

    Ok(Json(updated))
}

pub async fn create_service(
    State(state): State<Arc<ProductsState>>,
    Json(req): Json<CreateServiceRequest>,
) -> Result<Json<Service>, (StatusCode, String)> {
    let mut conn = state.pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state.pool, &state.get_default_bot);
    let id = Uuid::new_v4();
    let now = Utc::now();

    let service = Service {
        id,
        org_id,
        bot_id,
        name: req.name,
        description: req.description,
        category: req.category,
        service_type: req.service_type.unwrap_or_else(|| "hourly".to_string()),
        hourly_rate: req.hourly_rate.map(bd),
        fixed_price: req.fixed_price.map(bd),
        currency: req.currency.unwrap_or_else(|| "USD".to_string()),
        duration_minutes: req.duration_minutes,
        is_active: true,
        attributes: serde_json::json!({}),
        created_at: now,
        updated_at: now,
    };

    diesel::insert_into(services::table)
        .values(&service)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert error: {e}")))?;

    Ok(Json(service))
}

pub async fn list_services(
    State(state): State<Arc<ProductsState>>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<Service>>, (StatusCode, String)> {
    let mut conn = state.pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state.pool, &state.get_default_bot);
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    let mut q = services::table
        .filter(services::org_id.eq(org_id))
        .filter(services::bot_id.eq(bot_id))
        .into_boxed();

    if let Some(is_active) = query.is_active {
        q = q.filter(services::is_active.eq(is_active));
    }

    if let Some(category) = query.category {
        q = q.filter(services::category.eq(category));
    }

    if let Some(search) = query.search {
        let pattern = format!("%{search}%");
        q = q.filter(
            services::name
                .ilike(pattern.clone())
                .or(services::description.ilike(pattern)),
        );
    }

    let svcs: Vec<Service> = q
        .order(services::name.asc())
        .limit(limit)
        .offset(offset)
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    Ok(Json(svcs))
}

pub async fn get_service(
    State(state): State<Arc<ProductsState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Service>, (StatusCode, String)> {
    let mut conn = state.pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let service: Service = services::table
        .filter(services::id.eq(id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Service not found".to_string()))?;

    Ok(Json(service))
}

pub async fn update_service(
    State(state): State<Arc<ProductsState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateServiceRequest>,
) -> Result<Json<Service>, (StatusCode, String)> {
    let mut conn = state.pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let now = Utc::now();

    diesel::update(services::table.filter(services::id.eq(id)))
        .set(services::updated_at.eq(now))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;

    if let Some(name) = req.name {
        diesel::update(services::table.filter(services::id.eq(id)))
            .set(services::name.eq(name))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    if let Some(description) = req.description {
        diesel::update(services::table.filter(services::id.eq(id)))
            .set(services::description.eq(description))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    if let Some(hourly_rate) = req.hourly_rate {
        diesel::update(services::table.filter(services::id.eq(id)))
            .set(services::hourly_rate.eq(Some(bd(hourly_rate))))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    if let Some(is_active) = req.is_active {
        diesel::update(services::table.filter(services::id.eq(id)))
            .set(services::is_active.eq(is_active))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    let service: Service = services::table
        .filter(services::id.eq(id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Service not found".to_string()))?;

    Ok(Json(service))
}

pub async fn delete_service(
    State(state): State<Arc<ProductsState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut conn = state.pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    diesel::delete(services::table.filter(services::id.eq(id)))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Delete error: {e}")))?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_categories(
    State(state): State<Arc<ProductsState>>,
) -> Result<Json<Vec<ProductCategory>>, (StatusCode, String)> {
    let mut conn = state.pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state.pool, &state.get_default_bot);

    let cats: Vec<ProductCategory> = product_categories::table
        .filter(product_categories::org_id.eq(org_id))
        .filter(product_categories::bot_id.eq(bot_id))
        .filter(product_categories::is_active.eq(true))
        .order(product_categories::sort_order.asc())
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    Ok(Json(cats))
}

pub async fn create_category(
    State(state): State<Arc<ProductsState>>,
    Json(req): Json<CreateCategoryRequest>,
) -> Result<Json<ProductCategory>, (StatusCode, String)> {
    let mut conn = state.pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state.pool, &state.get_default_bot);
    let id = Uuid::new_v4();
    let now = Utc::now();

    let max_order: Option<i32> = product_categories::table
        .filter(product_categories::org_id.eq(org_id))
        .filter(product_categories::bot_id.eq(bot_id))
        .select(diesel::dsl::max(product_categories::sort_order))
        .first(&mut conn)
        .unwrap_or(None);

    let category = ProductCategory {
        id,
        org_id,
        bot_id,
        name: req.name,
        description: req.description,
        parent_id: req.parent_id,
        slug: req.slug,
        image_url: req.image_url,
        sort_order: max_order.unwrap_or(0) + 1,
        is_active: true,
        created_at: now,
    };

    diesel::insert_into(product_categories::table)
        .values(&category)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert error: {e}")))?;

    Ok(Json(category))
}

pub async fn list_price_lists(
    State(state): State<Arc<ProductsState>>,
) -> Result<Json<Vec<PriceList>>, (StatusCode, String)> {
    let mut conn = state.pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state.pool, &state.get_default_bot);

    let lists: Vec<PriceList> = price_lists::table
        .filter(price_lists::org_id.eq(org_id))
        .filter(price_lists::bot_id.eq(bot_id))
        .filter(price_lists::is_active.eq(true))
        .order(price_lists::name.asc())
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    Ok(Json(lists))
}

pub async fn create_price_list(
    State(state): State<Arc<ProductsState>>,
    Json(req): Json<CreatePriceListRequest>,
) -> Result<Json<PriceList>, (StatusCode, String)> {
    let mut conn = state.pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state.pool, &state.get_default_bot);
    let id = Uuid::new_v4();
    let now = Utc::now();

    let valid_from = req
        .valid_from
        .and_then(|d| chrono::NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok());

    let valid_until = req
        .valid_until
        .and_then(|d| chrono::NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok());

    let price_list = PriceList {
        id,
        org_id,
        bot_id,
        name: req.name,
        description: req.description,
        currency: req.currency.unwrap_or_else(|| "USD".to_string()),
        is_default: false,
        valid_from,
        valid_until,
        customer_group: req.customer_group,
        discount_percent: bd(req.discount_percent.unwrap_or(0.0)),
        is_active: true,
        created_at: now,
        updated_at: now,
    };

    diesel::insert_into(price_lists::table)
        .values(&price_list)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert error: {e}")))?;

    Ok(Json(price_list))
}

pub async fn list_inventory_movements(
    State(state): State<Arc<ProductsState>>,
    Path(product_id): Path<Uuid>,
) -> Result<Json<Vec<InventoryMovement>>, (StatusCode, String)> {
    let mut conn = state.pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let movements: Vec<InventoryMovement> = inventory_movements::table
        .filter(inventory_movements::product_id.eq(product_id))
        .order(inventory_movements::created_at.desc())
        .limit(100)
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    Ok(Json(movements))
}

pub async fn get_product_stats(
    State(state): State<Arc<ProductsState>>,
) -> Result<Json<ProductStats>, (StatusCode, String)> {
    let mut conn = state.pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state.pool, &state.get_default_bot);

    let total_products: i64 = products::table
        .filter(products::org_id.eq(org_id))
        .filter(products::bot_id.eq(bot_id))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    let active_products: i64 = products::table
        .filter(products::org_id.eq(org_id))
        .filter(products::bot_id.eq(bot_id))
        .filter(products::is_active.eq(true))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    let total_services: i64 = services::table
        .filter(services::org_id.eq(org_id))
        .filter(services::bot_id.eq(bot_id))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    let active_services: i64 = services::table
        .filter(services::org_id.eq(org_id))
        .filter(services::bot_id.eq(bot_id))
        .filter(services::is_active.eq(true))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    let low_stock_count: i64 = products::table
        .filter(products::org_id.eq(org_id))
        .filter(products::bot_id.eq(bot_id))
        .filter(products::is_active.eq(true))
        .filter(products::stock_quantity.le(products::low_stock_threshold))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    let categories_count: i64 = product_categories::table
        .filter(product_categories::org_id.eq(org_id))
        .filter(product_categories::bot_id.eq(bot_id))
        .filter(product_categories::is_active.eq(true))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    let price_lists_count: i64 = price_lists::table
        .filter(price_lists::org_id.eq(org_id))
        .filter(price_lists::bot_id.eq(bot_id))
        .filter(price_lists::is_active.eq(true))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    let all_products: Vec<Product> = products::table
        .filter(products::org_id.eq(org_id))
        .filter(products::bot_id.eq(bot_id))
        .filter(products::is_active.eq(true))
        .load(&mut conn)
        .unwrap_or_default();

    let total_stock_value: f64 = all_products
        .iter()
        .map(|p| bd_to_f64(&p.price) * p.stock_quantity as f64)
        .sum();

    let stats = ProductStats {
        total_products,
        active_products,
        total_services,
        active_services,
        low_stock_count,
        total_stock_value,
        categories_count,
        price_lists_count,
    };

    Ok(Json(stats))
}

pub async fn list_low_stock(
    State(state): State<Arc<ProductsState>>,
) -> Result<Json<Vec<Product>>, (StatusCode, String)> {
    let mut conn = state.pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state.pool, &state.get_default_bot);

    let prods: Vec<Product> = products::table
        .filter(products::org_id.eq(org_id))
        .filter(products::bot_id.eq(bot_id))
        .filter(products::is_active.eq(true))
        .filter(products::stock_quantity.le(products::low_stock_threshold))
        .order(products::stock_quantity.asc())
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    Ok(Json(prods))
}

pub fn configure_products_api_routes() -> Router<Arc<ProductsState>> {
    Router::new()
        .route("/api/products/items", get(list_products).post(create_product))
        .route("/api/products/items/:id", get(get_product).put(update_product).delete(delete_product))
        .route("/api/products/items/:id/stock", put(adjust_stock))
        .route("/api/products/items/:id/movements", get(list_inventory_movements))
        .route("/api/products/services", get(list_services).post(create_service))
        .route("/api/products/services/:id", get(get_service).put(update_service).delete(delete_service))
        .route("/api/products/categories", get(list_categories).post(create_category))
        .route("/api/products/price-lists", get(list_price_lists).post(create_price_list))
        .route("/api/products/stats", get(get_product_stats))
        .route("/api/products/low-stock", get(list_low_stock))
}

async fn handle_products_items(
    State(state): State<Arc<ProductsState>>,
    Query(query): Query<ProductQuery>,
) -> impl IntoResponse {
    let pool = state.pool.clone();
    let get_default_bot = state.get_default_bot;

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let (bot_id, _) = match get_default_bot {
            Some(f) => f(&mut conn),
            None => return None,
        };

        let mut db_query = products::table
            .filter(products::bot_id.eq(bot_id))
            .into_boxed();

        if let Some(ref category) = query.category {
            db_query = db_query.filter(products::category.eq(category));
        }

        if let Some(ref status) = query.status {
            let is_active = status == "active";
            db_query = db_query.filter(products::is_active.eq(is_active));
        }

        db_query = db_query.order(products::created_at.desc());

        let limit = query.limit.unwrap_or(50);
        db_query = db_query.limit(limit);

        db_query
            .select((
                products::id,
                products::sku,
                products::name,
                products::description,
                products::category,
                products::price,
                products::currency,
                products::stock_quantity,
                products::is_active,
            ))
            .load::<(Uuid, Option<String>, String, Option<String>, Option<String>, BigDecimal, String, i32, bool)>(&mut conn)
            .ok()
    })
    .await
    .ok()
    .flatten();

    match result {
        Some(items) if !items.is_empty() => {
            let mut html = String::new();
            for (id, sku, name, desc, category, price, currency, stock, is_active) in items {
                let sku_str = sku.unwrap_or_else(|| "-".to_string());
                let desc_str = desc.unwrap_or_default();
                let cat_str = category.unwrap_or_else(|| "Uncategorized".to_string());
                let price_str = format_currency(bd_to_f64(&price), &currency);
                let stock_str = stock.to_string();
                let status_class = if is_active { "status-active" } else { "status-inactive" };
                let status_text = if is_active { "Active" } else { "Inactive" };

                html.push_str(&format!(
                    "<div class=\"product-card\" data-id=\"{id}\">\
                    <div class=\"product-header\">\
                    <span class=\"product-name\">{}</span>\
                    <span class=\"product-sku\">{}</span>\
                    </div>\
                    <div class=\"product-body\">\
                    <p class=\"product-desc\">{}</p>\
                    <div class=\"product-meta\">\
                    <span class=\"product-category\">{}</span>\
                    <span class=\"product-price\">{}</span>\
                    <span class=\"product-stock\">Stock: {}</span>\
                    <span class=\"{}\">{}</span>\
                    </div>\
                    </div>\
                    <div class=\"product-actions\">\
                    <button class=\"btn-sm\" hx-get=\"/api/products/{id}\" hx-target=\"#product-detail\">View</button>\
                    <button class=\"btn-sm btn-secondary\" hx-get=\"/api/products/{id}/edit\" hx-target=\"#modal-content\">Edit</button>\
                    </div>\
                    </div>",
                    html_escape(&name),
                    html_escape(&sku_str),
                    html_escape(&desc_str),
                    html_escape(&cat_str),
                    price_str,
                    stock_str,
                    status_class,
                    status_text
                ));
            }
            Html(format!("<div class=\"products-grid\">{html}</div>"))
        }
        _ => Html(
            "<div class=\"products-empty\">\
            <div class=\"empty-icon\">\u{1f4e6}</div>\
            <p>No products yet</p>\
            <p class=\"empty-hint\">Add your first product to get started</p>\
            </div>".to_string(),
        ),
    }
}

async fn handle_products_services(
    State(state): State<Arc<ProductsState>>,
    Query(query): Query<ProductQuery>,
) -> impl IntoResponse {
    let pool = state.pool.clone();
    let get_default_bot = state.get_default_bot;

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let (bot_id, _) = match get_default_bot {
            Some(f) => f(&mut conn),
            None => return None,
        };

        let mut db_query = services::table
            .filter(services::bot_id.eq(bot_id))
            .into_boxed();

        if let Some(ref category) = query.category {
            db_query = db_query.filter(services::category.eq(category));
        }

        if let Some(ref status) = query.status {
            let is_active = status == "active";
            db_query = db_query.filter(services::is_active.eq(is_active));
        }

        db_query = db_query.order(services::created_at.desc());

        let limit = query.limit.unwrap_or(50);
        db_query = db_query.limit(limit);

        db_query
            .select((
                services::id,
                services::name,
                services::description,
                services::category,
                services::service_type,
                services::hourly_rate,
                services::fixed_price,
                services::currency,
                services::duration_minutes,
                services::is_active,
            ))
            .load::<(Uuid, String, Option<String>, Option<String>, String, Option<BigDecimal>, Option<BigDecimal>, String, Option<i32>, bool)>(&mut conn)
            .ok()
    })
    .await
    .ok()
    .flatten();

    match result {
        Some(items) if !items.is_empty() => {
            let mut html = String::new();
            for (id, name, _desc, category, svc_type, hourly, fixed, currency, duration, is_active) in items {
                let cat_str = category.unwrap_or_else(|| "General".to_string());
                let type_str = svc_type;
                let price_str = if let Some(ref h) = hourly {
                    format!("{}/hr", format_currency(bd_to_f64(h), &currency))
                } else if let Some(ref f) = fixed {
                    format_currency(bd_to_f64(f), &currency)
                } else {
                    "-".to_string()
                };
                let duration_str = duration.map(|d| format!("{} min", d)).unwrap_or_else(|| "-".to_string());
                let status_class = if is_active { "status-active" } else { "status-inactive" };
                let status_text = if is_active { "Active" } else { "Inactive" };

                html.push_str(&format!(
                    "<tr class=\"service-row\" data-id=\"{id}\">\
                    <td class=\"service-name\">{}</td>\
                    <td class=\"service-category\">{}</td>\
                    <td class=\"service-type\">{}</td>\
                    <td class=\"service-price\">{}</td>\
                    <td class=\"service-duration\">{}</td>\
                    <td class=\"service-status\"><span class=\"{}\">{}</span></td>\
                    <td class=\"service-actions\">\
                    <button class=\"btn-sm\" hx-get=\"/api/products/services/{id}\" hx-target=\"#service-detail\">View</button>\
                    </td>\
                    </tr>",
                    html_escape(&name),
                    html_escape(&cat_str),
                    html_escape(&type_str),
                    price_str,
                    duration_str,
                    status_class,
                    status_text
                ));
            }
            Html(html)
        }
        _ => Html(
            "<tr class=\"empty-row\">\
            <td colspan=\"7\" class=\"empty-state\">\
            <div class=\"empty-icon\">\u{1f527}</div>\
            <p>No services yet</p>\
            <p class=\"empty-hint\">Add services to your catalog</p>\
            </td>\
            </tr>".to_string(),
        ),
    }
}

async fn handle_products_pricelists(
    State(state): State<Arc<ProductsState>>,
    Query(query): Query<ProductQuery>,
) -> impl IntoResponse {
    let pool = state.pool.clone();
    let get_default_bot = state.get_default_bot;

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let (bot_id, _) = match get_default_bot {
            Some(f) => f(&mut conn),
            None => return None,
        };

        let mut db_query = price_lists::table
            .filter(price_lists::bot_id.eq(bot_id))
            .into_boxed();

        if let Some(ref status) = query.status {
            let is_active = status == "active";
            db_query = db_query.filter(price_lists::is_active.eq(is_active));
        }

        db_query = db_query.order(price_lists::created_at.desc());

        let limit = query.limit.unwrap_or(50);
        db_query = db_query.limit(limit);

        db_query
            .select((
                price_lists::id,
                price_lists::name,
                price_lists::description,
                price_lists::currency,
                price_lists::is_default,
                price_lists::discount_percent,
                price_lists::customer_group,
                price_lists::is_active,
            ))
            .load::<(Uuid, String, Option<String>, String, bool, BigDecimal, Option<String>, bool)>(&mut conn)
            .ok()
    })
    .await
    .ok()
    .flatten();

    match result {
        Some(items) if !items.is_empty() => {
            let mut html = String::new();
            for (id, name, _desc, currency, is_default, discount, customer_group, is_active) in items {
                let discount_pct = bd_to_f64(&discount);
                let discount_str = if discount_pct > 0.0 { format!("{:.1}%", discount_pct) } else { "-".to_string() };
                let group_str = customer_group.unwrap_or_else(|| "All".to_string());
                let default_badge = if is_default { "<span class=\"badge badge-primary\">Default</span>" } else { "" };
                let status_class = if is_active { "status-active" } else { "status-inactive" };
                let status_text = if is_active { "Active" } else { "Inactive" };

                html.push_str(&format!(
                    "<tr class=\"pricelist-row\" data-id=\"{id}\">\
                    <td class=\"pricelist-name\">{} {}</td>\
                    <td class=\"pricelist-currency\">{}</td>\
                    <td class=\"pricelist-discount\">{}</td>\
                    <td class=\"pricelist-group\">{}</td>\
                    <td class=\"pricelist-status\"><span class=\"{}\">{}</span></td>\
                    <td class=\"pricelist-actions\">\
                    <button class=\"btn-sm\" hx-get=\"/api/products/pricelists/{id}\" hx-target=\"#pricelist-detail\">View</button>\
                    </td>\
                    </tr>",
                    html_escape(&name),
                    default_badge,
                    currency,
                    discount_str,
                    html_escape(&group_str),
                    status_class,
                    status_text
                ));
            }
            Html(html)
        }
        _ => Html(
            "<tr class=\"empty-row\">\
            <td colspan=\"6\" class=\"empty-state\">\
            <div class=\"empty-icon\">\u{1f4b0}</div>\
            <p>No price lists yet</p>\
            <p class=\"empty-hint\">Create price lists for different customer segments</p>\
            </td>\
            </tr>".to_string(),
        ),
    }
}

async fn handle_total_products(State(state): State<Arc<ProductsState>>) -> impl IntoResponse {
    let pool = state.pool.clone();
    let get_default_bot = state.get_default_bot;

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let (bot_id, _) = match get_default_bot {
            Some(f) => f(&mut conn),
            None => return None,
        };

        products::table
            .filter(products::bot_id.eq(bot_id))
            .count()
            .get_result::<i64>(&mut conn)
            .ok()
    })
    .await
    .ok()
    .flatten();

    Html(format!("{}", result.unwrap_or(0)))
}

async fn handle_total_services(State(state): State<Arc<ProductsState>>) -> impl IntoResponse {
    let pool = state.pool.clone();
    let get_default_bot = state.get_default_bot;

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let (bot_id, _) = match get_default_bot {
            Some(f) => f(&mut conn),
            None => return None,
        };

        services::table
            .filter(services::bot_id.eq(bot_id))
            .count()
            .get_result::<i64>(&mut conn)
            .ok()
    })
    .await
    .ok()
    .flatten();

    Html(format!("{}", result.unwrap_or(0)))
}

async fn handle_total_pricelists(State(state): State<Arc<ProductsState>>) -> impl IntoResponse {
    let pool = state.pool.clone();
    let get_default_bot = state.get_default_bot;

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let (bot_id, _) = match get_default_bot {
            Some(f) => f(&mut conn),
            None => return None,
        };

        price_lists::table
            .filter(price_lists::bot_id.eq(bot_id))
            .count()
            .get_result::<i64>(&mut conn)
            .ok()
    })
    .await
    .ok()
    .flatten();

    Html(format!("{}", result.unwrap_or(0)))
}

async fn handle_active_products(State(state): State<Arc<ProductsState>>) -> impl IntoResponse {
    let pool = state.pool.clone();
    let get_default_bot = state.get_default_bot;

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let (bot_id, _) = match get_default_bot {
            Some(f) => f(&mut conn),
            None => return None,
        };

        products::table
            .filter(products::bot_id.eq(bot_id))
            .filter(products::is_active.eq(true))
            .count()
            .get_result::<i64>(&mut conn)
            .ok()
    })
    .await
    .ok()
    .flatten();

    Html(format!("{}", result.unwrap_or(0)))
}

async fn handle_products_search(
    State(state): State<Arc<ProductsState>>,
    Query(query): Query<SearchQuery>,
) -> impl IntoResponse {
    let q = query.q.clone().unwrap_or_default();
    if q.is_empty() {
        return Html(String::new());
    }

    let pool = state.pool.clone();
    let get_default_bot = state.get_default_bot;
    let search_term = format!("%{}%", q);

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let (bot_id, _) = match get_default_bot {
            Some(f) => f(&mut conn),
            None => return None,
        };

        products::table
            .filter(products::bot_id.eq(bot_id))
            .filter(
                products::name.ilike(&search_term)
                    .or(products::sku.ilike(&search_term))
                    .or(products::description.ilike(&search_term))
            )
            .order(products::name.asc())
            .limit(20)
            .select((
                products::id,
                products::sku,
                products::name,
                products::category,
                products::price,
                products::currency,
            ))
            .load::<(Uuid, Option<String>, String, Option<String>, BigDecimal, String)>(&mut conn)
            .ok()
    })
    .await
    .ok()
    .flatten();

    match result {
        Some(items) if !items.is_empty() => {
            let mut html = String::new();
            for (id, sku, name, category, price, currency) in items {
                let sku_str = sku.unwrap_or_else(|| "-".to_string());
                let cat_str = category.unwrap_or_else(|| "Uncategorized".to_string());
                let price_str = format_currency(bd_to_f64(&price), &currency);

                html.push_str(&format!(
                    "<div class=\"search-result-item\" hx-get=\"/api/products/{id}\" hx-target=\"#product-detail\">\
                    <span class=\"result-name\">{}</span>\
                    <span class=\"result-sku\">{}</span>\
                    <span class=\"result-category\">{}</span>\
                    <span class=\"result-price\">{}</span>\
                    </div>",
                    html_escape(&name),
                    html_escape(&sku_str),
                    html_escape(&cat_str),
                    price_str
                ));
            }
            Html(format!("<div class=\"search-results\">{html}</div>"))
        }
        _ => Html(format!(
            "<div class=\"search-results-empty\">\
            <p>No results for \"{}\"</p>\
            </div>",
            html_escape(&q)
        )),
    }
}

pub fn configure_products_routes() -> Router<Arc<ProductsState>> {
    Router::new()
        .route("/api/ui/products/items", get(handle_products_items))
        .route("/api/ui/products/services", get(handle_products_services))
        .route("/api/ui/products/pricelists", get(handle_products_pricelists))
        .route("/api/ui/products/stats/total-products", get(handle_total_products))
        .route("/api/ui/products/stats/total-services", get(handle_total_services))
        .route("/api/ui/products/stats/pricelists", get(handle_total_pricelists))
        .route("/api/ui/products/stats/active", get(handle_active_products))
        .route("/api/ui/products/search", get(handle_products_search))
}

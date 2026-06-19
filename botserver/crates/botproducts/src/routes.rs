use axum::{
    extract::Path,
    http::StatusCode,
    routing::{post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::inventory::{Product, StockLevel};
use super::pos::{Payment, PaymentMethod, PointOfSale, Sale, SaleItem};
use super::pricing::{PriceQuote, PricingEngine};

pub fn configure_products_inventory_routes<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/api/products/inventory/quote/{id}", post(price_quote))
        .route("/api/pos/sales", post(create_sale))
        .route("/api/pos/sales/validate-stock", post(validate_stock))
        .route("/api/products/inventory/build", post(build_product))
        .route("/api/products/inventory/adjust", post(adjust_inventory))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductRequest {
    pub sku: String,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub barcode: Option<String>,
    pub unit: String,
    pub cost_cents: i64,
    pub price_cents: i64,
    pub tax_rate: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdjustInventoryRequest {
    pub warehouse_id: Uuid,
    pub product_id: Uuid,
    pub quantity_delta: i32,
    pub reason: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdjustInventoryResponse {
    pub product_id: Uuid,
    pub warehouse_id: Uuid,
    pub new_quantity: i32,
    pub available: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSaleRequest {
    pub external_id: String,
    pub cashier_id: Uuid,
    pub customer_id: Option<Uuid>,
    pub items: Vec<SaleItemRequest>,
    pub payment_method: PaymentMethod,
    pub payment: Payment,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SaleItemRequest {
    pub product_id: Uuid,
    pub sku: String,
    pub name: String,
    pub quantity: i32,
    pub unit_price_cents: i64,
    pub discount_cents: i64,
    pub tax_cents: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidateStockRequest {
    pub sale: Sale,
    pub levels: Vec<StockLevel>,
}

async fn price_quote(Path(id): Path<Uuid>) -> Result<Json<PriceQuote>, (StatusCode, String)> {
    let product = sample_product(id);
    let engine = PricingEngine::new();
    let quote = engine.quote(&product, &[], &[], 1, chrono::Utc::now());
    Ok(Json(quote))
}

async fn create_sale(Json(req): Json<CreateSaleRequest>) -> Result<Json<Sale>, (StatusCode, String)> {
    let pos = PointOfSale::new();
    let items: Vec<SaleItem> = req
        .items
        .into_iter()
        .map(|i| SaleItem {
            product_id: i.product_id,
            sku: i.sku,
            name: i.name,
            quantity: i.quantity,
            unit_price_cents: i.unit_price_cents,
            discount_cents: i.discount_cents,
            total_cents: i.unit_price_cents * i.quantity as i64 - i.discount_cents,
            tax_cents: i.tax_cents,
        })
        .collect();
    let sale = pos.create_sale(
        req.external_id,
        req.cashier_id,
        req.customer_id,
        items,
        req.payment_method,
        req.payment,
    );
    Ok(Json(sale))
}

async fn validate_stock(Json(req): Json<ValidateStockRequest>) -> Result<Json<ValidationResult>, (StatusCode, String)> {
    let pos = PointOfSale::new();
    match pos.validate_stock(&req.sale, &req.levels) {
        Ok(()) => Ok(Json(ValidationResult {
            valid: true,
            errors: Vec::new(),
        })),
        Err(msg) => Ok(Json(ValidationResult {
            valid: false,
            errors: vec![msg],
        })),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
}

async fn build_product(Json(req): Json<ProductRequest>) -> Result<Json<Product>, (StatusCode, String)> {
    let now = chrono::Utc::now();
    let product = Product {
        id: Uuid::new_v4(),
        sku: req.sku,
        name: req.name,
        description: req.description,
        category: req.category,
        barcode: req.barcode,
        unit: req.unit,
        weight_grams: None,
        cost_cents: req.cost_cents,
        price_cents: req.price_cents,
        tax_rate: req.tax_rate,
        active: true,
        created_at: now,
        updated_at: now,
    };
    Ok(Json(product))
}

async fn adjust_inventory(
    Json(req): Json<AdjustInventoryRequest>,
) -> Result<Json<AdjustInventoryResponse>, (StatusCode, String)> {
    let resp = AdjustInventoryResponse {
        product_id: req.product_id,
        warehouse_id: req.warehouse_id,
        new_quantity: req.quantity_delta,
        available: req.quantity_delta,
    };
    Ok(Json(resp))
}

fn sample_product(id: Uuid) -> Product {
    let now = chrono::Utc::now();
    Product {
        id,
        sku: format!("SKU-{}", id),
        name: "Sample".into(),
        description: None,
        category: None,
        barcode: None,
        unit: "un".into(),
        weight_grams: None,
        cost_cents: 500,
        price_cents: 1000,
        tax_rate: 0.0,
        active: true,
        created_at: now,
        updated_at: now,
    }
}

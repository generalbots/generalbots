use chrono::DateTime;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryItem {
    pub id: Uuid,
    pub bot_id: Uuid,
    pub product_id: Option<Uuid>,
    pub sku: String,
    pub name: String,
    pub description: Option<String>,
    pub quantity: rust_decimal::Decimal,
    pub unit: String,
    pub min_stock: Option<rust_decimal::Decimal>,
    pub max_stock: Option<rust_decimal::Decimal>,
    pub location: Option<String>,
    pub category: Option<String>,
    pub unit_cost: rust_decimal::Decimal,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryMovement {
    pub id: Uuid,
    pub bot_id: Uuid,
    pub item_id: Uuid,
    pub movement_type: String,
    pub quantity: rust_decimal::Decimal,
    pub reference_type: Option<String>,
    pub reference_id: Option<Uuid>,
    pub notes: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseOrder {
    pub id: Uuid,
    pub bot_id: Uuid,
    pub po_number: String,
    pub vendor_name: String,
    pub status: String,
    pub total_amount: rust_decimal::Decimal,
    pub currency: String,
    pub expected_date: Option<chrono::NaiveDate>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateItemRequest {
    pub sku: String,
    pub name: String,
    pub description: Option<String>,
    pub quantity: Option<rust_decimal::Decimal>,
    pub unit: Option<String>,
    pub min_stock: Option<rust_decimal::Decimal>,
    pub max_stock: Option<rust_decimal::Decimal>,
    pub location: Option<String>,
    pub category: Option<String>,
    pub unit_cost: Option<rust_decimal::Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMovementRequest {
    pub item_id: Uuid,
    pub movement_type: String,
    pub quantity: rust_decimal::Decimal,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePoRequest {
    pub vendor_name: String,
    pub expected_date: Option<chrono::NaiveDate>,
    pub notes: Option<String>,
    pub items: Vec<PoItemInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoItemInput {
    pub item_id: Option<Uuid>,
    pub description: String,
    pub quantity: rust_decimal::Decimal,
    pub unit_price: rust_decimal::Decimal,
}

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StockMovementType {
    Purchase,
    Sale,
    Return,
    Adjustment,
    TransferOut,
    TransferIn,
    Damaged,
    Expired,
    Initial,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: Uuid,
    pub sku: String,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub barcode: Option<String>,
    pub unit: String,
    pub weight_grams: Option<i32>,
    pub cost_cents: i64,
    pub price_cents: i64,
    pub tax_rate: f64,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockLevel {
    pub id: Uuid,
    pub product_id: Uuid,
    pub warehouse_id: Uuid,
    pub quantity: i32,
    pub reserved: i32,
    pub available: i32,
    pub min_stock: i32,
    pub max_stock: Option<i32>,
    pub reorder_point: i32,
    pub last_counted_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Warehouse {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub address: Option<String>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockMovement {
    pub id: Uuid,
    pub product_id: Uuid,
    pub warehouse_id: Uuid,
    pub movement_type: StockMovementType,
    pub quantity: i32,
    pub unit_cost_cents: Option<i64>,
    pub reference: Option<String>,
    pub notes: Option<String>,
    pub user_id: Option<Uuid>,
    pub occurred_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockAlert {
    pub product_id: Uuid,
    pub warehouse_id: Uuid,
    pub current_quantity: i32,
    pub reorder_point: i32,
    pub severity: AlertSeverity,
    pub suggested_reorder: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AlertSeverity {
    Critical,
    Low,
    Warning,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryCount {
    pub id: Uuid,
    pub warehouse_id: Uuid,
    pub scheduled_date: NaiveDate,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: CountStatus,
    pub items: Vec<InventoryCountItem>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CountStatus {
    Planned,
    InProgress,
    Completed,
    Cancelled,
    Reconciling,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryCountItem {
    pub product_id: Uuid,
    pub expected_quantity: i32,
    pub counted_quantity: i32,
    pub variance: i32,
    pub notes: Option<String>,
}

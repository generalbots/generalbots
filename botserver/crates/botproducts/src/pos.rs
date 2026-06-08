use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::inventory::{StockLevel, StockMovement, StockMovementType};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SaleStatus {
    Open,
    Completed,
    Cancelled,
    Refunded,
    PartiallyRefunded,
    OnHold,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PaymentMethod {
    Cash,
    CreditCard,
    DebitCard,
    Pix,
    BankTransfer,
    Voucher,
    StoreCredit,
    Mixed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sale {
    pub id: Uuid,
    pub external_id: String,
    pub status: SaleStatus,
    pub customer_id: Option<Uuid>,
    pub cashier_id: Uuid,
    pub terminal_id: Option<String>,
    pub items: Vec<SaleItem>,
    pub subtotal_cents: i64,
    pub discount_cents: i64,
    pub tax_cents: i64,
    pub total_cents: i64,
    pub payment_method: PaymentMethod,
    pub payments: Vec<Payment>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaleItem {
    pub product_id: Uuid,
    pub sku: String,
    pub name: String,
    pub quantity: i32,
    pub unit_price_cents: i64,
    pub discount_cents: i64,
    pub total_cents: i64,
    pub tax_cents: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    pub method: PaymentMethod,
    pub amount_cents: i64,
    pub installments: i32,
    pub authorization_code: Option<String>,
    pub card_brand: Option<String>,
    pub card_last_four: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashRegister {
    pub id: Uuid,
    pub terminal_id: String,
    pub operator_id: Uuid,
    pub opened_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub opening_amount_cents: i64,
    pub closing_amount_cents: Option<i64>,
    pub expected_amount_cents: Option<i64>,
    pub difference_cents: Option<i64>,
    pub sales_count: i32,
    pub status: CashRegisterStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CashRegisterStatus {
    Open,
    Closed,
    Reconciling,
}

pub struct PointOfSale;

impl PointOfSale {
    pub fn new() -> Self {
        Self
    }

    pub fn create_sale(
        &self,
        external_id: String,
        cashier_id: Uuid,
        customer_id: Option<Uuid>,
        items: Vec<SaleItem>,
        payment_method: PaymentMethod,
        payment: Payment,
    ) -> Sale {
        let subtotal: i64 = items.iter().map(|i| i.unit_price_cents * i.quantity as i64).sum();
        let discount: i64 = items.iter().map(|i| i.discount_cents).sum();
        let tax: i64 = items.iter().map(|i| i.tax_cents).sum();
        let total: i64 = subtotal - discount + tax;
        Sale {
            id: Uuid::new_v4(),
            external_id,
            status: if total > 0 { SaleStatus::Completed } else { SaleStatus::Open },
            customer_id,
            cashier_id,
            terminal_id: None,
            items,
            subtotal_cents: subtotal,
            discount_cents: discount,
            tax_cents: tax,
            total_cents: total,
            payment_method,
            payments: vec![payment],
            notes: None,
            created_at: Utc::now(),
            completed_at: Some(Utc::now()),
        }
    }

    pub fn validate_stock(
        &self,
        sale: &Sale,
        levels: &[StockLevel],
    ) -> Result<(), String> {
        for item in &sale.items {
            let available = levels
                .iter()
                .find(|l| l.product_id == item.product_id)
                .map(|l| l.available)
                .unwrap_or(0);
            if available < item.quantity {
                return Err(format!(
                    "Insufficient stock for product {}: need {}, have {}",
                    item.sku, item.quantity, available
                ));
            }
        }
        Ok(())
    }

    pub fn build_movements(&self, sale: &Sale) -> Vec<StockMovement> {
        sale.items
            .iter()
            .map(|item| StockMovement {
                id: Uuid::new_v4(),
                product_id: item.product_id,
                warehouse_id: Uuid::nil(),
                movement_type: StockMovementType::Sale,
                quantity: -item.quantity,
                unit_cost_cents: None,
                reference: Some(sale.external_id.clone()),
                notes: None,
                user_id: Some(sale.cashier_id),
                occurred_at: sale.completed_at.unwrap_or_else(Utc::now),
                created_at: Utc::now(),
            })
            .collect()
    }
}

impl Default for PointOfSale {
    fn default() -> Self {
        Self::new()
    }
}

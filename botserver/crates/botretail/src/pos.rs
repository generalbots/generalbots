//! Simplified POS (Point of Sale) types.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Payment method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaymentMethod {
    /// Cash.
    Cash,
    /// Credit card.
    Credit,
    /// Debit card.
    Debit,
    /// PIX.
    Pix,
    /// Bank transfer.
    BankTransfer,
    /// Voucher / store credit.
    Voucher,
}

/// A single line item in a POS sale.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PosLineItem {
    /// Line ID.
    pub id: Uuid,
    /// Variation ID.
    pub variation_id: Uuid,
    /// Quantity sold.
    pub quantity: i64,
    /// Unit price at the moment of sale (after promotions).
    pub unit_price: Decimal,
    /// Discount applied to this line.
    pub discount: Decimal,
    /// Total for this line (`quantity * unit_price - discount`).
    pub total: Decimal,
}

/// A single payment in a POS sale.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PosPayment {
    /// Payment ID.
    pub id: Uuid,
    /// Method.
    pub method: PaymentMethod,
    /// Amount.
    pub amount: Decimal,
    /// Authorization code (when applicable).
    pub auth_code: Option<String>,
}

/// A POS sale (single transaction).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PosSale {
    /// Server-assigned sale ID.
    pub id: Uuid,
    /// Tenant.
    pub tenant_id: String,
    /// Branch where the sale happened.
    pub branch_id: Uuid,
    /// Cashier user ID.
    pub cashier_id: String,
    /// Customer (optional).
    pub customer_id: Option<String>,
    /// All line items.
    pub lines: Vec<PosLineItem>,
    /// All payments.
    pub payments: Vec<PosPayment>,
    /// When the sale was finalized.
    pub finalized_at: DateTime<Utc>,
    /// Optional NFCe key when issued (Brazilian model).
    pub nfce_key: Option<String>,
}

impl PosSale {
    /// Gross total before discounts.
    pub fn gross(&self) -> Decimal {
        self.lines.iter().map(|l| l.total).sum()
    }

    /// Sum of all payments.
    pub fn paid(&self) -> Decimal {
        self.payments.iter().map(|p| p.amount).sum()
    }

    /// Outstanding balance (`gross - paid`).
    pub fn balance(&self) -> Decimal {
        self.gross() - self.paid()
    }
}

/// Active POS session (open shift).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PosSession {
    /// Server-assigned session ID.
    pub id: Uuid,
    /// Tenant.
    pub tenant_id: String,
    /// Branch.
    pub branch_id: Uuid,
    /// Cashier.
    pub cashier_id: String,
    /// When the session was opened.
    pub opened_at: DateTime<Utc>,
    /// When the session was closed (None = still open).
    pub closed_at: Option<DateTime<Utc>>,
    /// Opening float (cash in drawer at the start).
    pub opening_float: Decimal,
    /// Closing count (cash in drawer at the end).
    pub closing_count: Option<Decimal>,
}

/// Errors raised by the POS engine.
#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum PosError {
    /// Sale total is zero or negative.
    #[error("sale total must be > 0")]
    ZeroTotal,
    /// Payment total doesn't match sale total.
    #[error("paid {paid} != gross {gross}")]
    PaymentMismatch {
        /// Paid.
        paid: Decimal,
        /// Gross.
        gross: Decimal,
    },
    /// Variation out of stock.
    #[error("variation {0} out of stock")]
    OutOfStock(Uuid),
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn balance_is_gross_minus_paid() {
        let sale = PosSale {
            id: Uuid::new_v4(),
            tenant_id: "t1".to_string(),
            branch_id: Uuid::new_v4(),
            cashier_id: "u1".to_string(),
            customer_id: None,
            lines: vec![PosLineItem {
                id: Uuid::new_v4(),
                variation_id: Uuid::new_v4(),
                quantity: 2,
                unit_price: dec!(10.00),
                discount: Decimal::ZERO,
                total: dec!(20.00),
            }],
            payments: vec![PosPayment {
                id: Uuid::new_v4(),
                method: PaymentMethod::Pix,
                amount: dec!(20.00),
                auth_code: None,
            }],
            finalized_at: Utc::now(),
            nfce_key: None,
        };
        assert_eq!(sale.gross(), dec!(20.00));
        assert_eq!(sale.paid(), dec!(20.00));
        assert_eq!(sale.balance(), Decimal::ZERO);
    }
}

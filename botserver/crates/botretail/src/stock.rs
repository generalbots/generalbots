//! Stock level and stock-movement types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Physical branch (filial).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Branch {
    /// Server-assigned branch ID.
    pub id: Uuid,
    /// Tenant.
    pub tenant_id: String,
    /// Display name (e.g. "Filial Centro").
    pub name: String,
    /// Address.
    pub address: Option<String>,
    /// Whether this is the main branch.
    pub is_main: bool,
}

/// Stock level for a given variation at a given branch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StockLevel {
    /// Branch ID.
    pub branch_id: Uuid,
    /// Variation ID.
    pub variation_id: Uuid,
    /// Available quantity.
    pub quantity: i64,
    /// Reserved quantity (e.g. open POS sales).
    pub reserved: i64,
}

impl StockLevel {
    /// Available quantity minus reserved.
    pub fn available(&self) -> i64 {
        self.quantity - self.reserved
    }
}

/// Kind of stock movement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MovementKind {
    /// Inbound (purchase order received).
    Inbound,
    /// Outbound (sale).
    Outbound,
    /// Transfer between branches.
    Transfer,
    /// Manual adjustment (loss, breakage, count correction).
    Adjustment,
    /// Return from customer.
    Return,
}

/// A single stock-movement entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StockMovement {
    /// Server-assigned ID.
    pub id: Uuid,
    /// Variation affected.
    pub variation_id: Uuid,
    /// Branch.
    pub branch_id: Uuid,
    /// Kind.
    pub kind: MovementKind,
    /// Signed delta (positive = inbound, negative = outbound).
    pub delta: i64,
    /// Free-form reason.
    pub reason: Option<String>,
    /// Reference to source (PO number, POS sale ID, etc.).
    pub reference: Option<String>,
    /// When the movement was recorded.
    pub at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn available_subtracts_reserved() {
        let s = StockLevel {
            branch_id: Uuid::new_v4(),
            variation_id: Uuid::new_v4(),
            quantity: 100,
            reserved: 12,
        };
        assert_eq!(s.available(), 88);
    }
}

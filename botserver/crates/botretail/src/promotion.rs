//! Promotion / discount types.

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Kind of discount.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiscountKind {
    /// Percentage off (value in [0, 100]).
    Percentage,
    /// Fixed amount off.
    FixedAmount,
    /// "Pay for N, get M" (e.g. 3x2).
    PayForNM,
    /// Bundle (price for a set of SKUs).
    Bundle,
}

/// A promotion window.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromotionWindow {
    /// Start (inclusive).
    pub start: NaiveDate,
    /// End (inclusive).
    pub end: NaiveDate,
    /// Optional list of branches where the promotion applies.
    pub branch_ids: Vec<Uuid>,
}

/// A promotion rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Promotion {
    /// Server-assigned ID.
    pub id: Uuid,
    /// Tenant.
    pub tenant_id: String,
    /// Display name.
    pub name: String,
    /// Kind of discount.
    pub kind: DiscountKind,
    /// Numeric value:
    /// - For [`DiscountKind::Percentage`]: percent (0–100).
    /// - For [`DiscountKind::FixedAmount`]: amount in `currency`.
    /// - For [`DiscountKind::PayForNM`]: `min_qty / paid_qty` packed as "N/M".
    /// - For [`DiscountKind::Bundle`]: bundle price.
    pub value: Decimal,
    /// Free-form value for `PayForNM` (e.g. "3/2").
    pub bundle_spec: Option<String>,
    /// Variations the promotion applies to.
    pub applies_to: Vec<Uuid>,
    /// Window.
    pub window: PromotionWindow,
    /// Whether the promotion is currently active.
    pub active: bool,
    /// When the promotion was created.
    pub created_at: DateTime<Utc>,
}

impl Promotion {
    /// Returns true if `today` is inside the promotion window and `active` is true.
    pub fn is_applicable(&self, today: NaiveDate) -> bool {
        self.active && today >= self.window.start && today <= self.window.end
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn applicable_within_window() {
        let p = Promotion {
            id: Uuid::new_v4(),
            tenant_id: "t1".to_string(),
            name: "Spring".to_string(),
            kind: DiscountKind::Percentage,
            value: dec!(10),
            bundle_spec: None,
            applies_to: vec![],
            window: PromotionWindow {
                start: NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
                end: NaiveDate::from_ymd_opt(2026, 3, 31).unwrap(),
                branch_ids: vec![],
            },
            active: true,
            created_at: Utc::now(),
        };
        assert!(p.is_applicable(NaiveDate::from_ymd_opt(2026, 3, 15).unwrap()));
        assert!(!p.is_applicable(NaiveDate::from_ymd_opt(2026, 4, 1).unwrap()));
    }
}

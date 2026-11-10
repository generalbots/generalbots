//! Promotion / discount types.

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::{Decimal, RoundingStrategy};
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

    /// Returns true when the promotion applies at `branch_id` on `today`.
    ///
    /// An empty `branch_ids` list means "every branch"; otherwise the branch
    /// must be listed. `is_applicable` alone ignores the branch restriction,
    /// so callers that know the branch must use this method.
    pub fn applies_to_branch(&self, branch_id: Uuid, today: NaiveDate) -> bool {
        self.is_applicable(today)
            && (self.window.branch_ids.is_empty() || self.window.branch_ids.contains(&branch_id))
    }

    /// Discount for a line of `quantity` units priced at `unit_price`.
    ///
    /// - `Percentage`: `value` percent of the line subtotal, capped at 100% and
    ///   rounded to cents so a 7.5% discount cannot leak fractions of a cent;
    /// - `FixedAmount`: `value`, capped at the subtotal;
    /// - `PayForNM`: the free units of every complete `N/M` group
    ///   (`bundle_spec`, e.g. `"3/2"` is buy 3 pay 2), priced at `unit_price`;
    /// - `Bundle`: zero here — the bundle price belongs to the composed
    ///   basket, which the POS builder resolves before pricing the line.
    ///
    /// A negative or malformed value never produces a negative discount.
    pub fn discount_for(&self, quantity: i64, unit_price: Decimal) -> Decimal {
        let quantity = quantity.max(0);
        let subtotal = unit_price * Decimal::from(quantity);
        if subtotal <= Decimal::ZERO {
            return Decimal::ZERO;
        }
        match self.kind {
            DiscountKind::Percentage => {
                let pct = self.value.min(Decimal::ONE_HUNDRED).max(Decimal::ZERO);
                // Money rounds half away from zero: `round_dp` alone uses
                // banker's rounding, which turned 7.5% of R$999,00 into
                // R$74,92 instead of the expected R$74,93.
                (subtotal * pct / Decimal::ONE_HUNDRED)
                    .round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero)
                    .min(subtotal)
            }
            DiscountKind::FixedAmount => self.value.max(Decimal::ZERO).min(subtotal),
            DiscountKind::PayForNM => {
                let Some((buy, pay)) = self.bundle_spec.as_deref().and_then(parse_pay_for) else {
                    return Decimal::ZERO;
                };
                let groups = quantity / buy;
                let free_units = groups * (buy - pay);
                unit_price * Decimal::from(free_units.max(0))
            }
            DiscountKind::Bundle => Decimal::ZERO,
        }
    }
}

/// Parses a `PayForNM` spec (`"buy/pay"`): both sides positive, pay not
/// greater than buy.
fn parse_pay_for(spec: &str) -> Option<(i64, i64)> {
    let (buy, pay) = spec.trim().split_once('/')?;
    let buy: i64 = buy.trim().parse().ok()?;
    let pay: i64 = pay.trim().parse().ok()?;
    if buy <= 0 || pay <= 0 || pay > buy {
        return None;
    }
    Some((buy, pay))
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

#[cfg(test)]
#[path = "promotion_tests.rs"]
mod promotion_tests;

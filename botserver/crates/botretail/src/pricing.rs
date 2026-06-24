//! Price lists and price tiers.

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Tier in a price list (wholesale tiers, retail tiers, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PriceTier {
    /// Retail (B2C).
    Retail,
    /// Wholesale (B2B).
    Wholesale,
    /// Reseller.
    Reseller,
    /// VIP customer.
    Vip,
    /// Promotional.
    Promo,
}

/// A single price-list entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PriceListEntry {
    /// Variation ID this price applies to.
    pub variation_id: Uuid,
    /// Tier.
    pub tier: PriceTier,
    /// Unit price.
    pub price: Decimal,
    /// Minimum quantity for the tier price to apply.
    pub min_qty: u32,
}

/// A complete price list for a tenant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PriceList {
    /// Server-assigned price list ID.
    pub id: Uuid,
    /// Tenant.
    pub tenant_id: String,
    /// Display name.
    pub name: String,
    /// Currency.
    pub currency: String,
    /// Effective from.
    pub effective_from: NaiveDate,
    /// Effective until (None = open-ended).
    pub effective_until: Option<NaiveDate>,
    /// All entries.
    pub entries: Vec<PriceListEntry>,
    /// When the price list was created.
    pub created_at: DateTime<Utc>,
}

impl PriceList {
    /// Find the price for a given variation and tier.
    pub fn price_for(&self, variation_id: Uuid, tier: PriceTier) -> Option<Decimal> {
        self.entries
            .iter()
            .filter(|e| e.variation_id == variation_id && e.tier == tier)
            .min_by_key(|e| e.min_qty)
            .map(|e| e.price)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn price_for_returns_lowest_min_qty() {
        let v = Uuid::new_v4();
        let pl = PriceList {
            id: Uuid::new_v4(),
            tenant_id: "t1".to_string(),
            name: "Default".to_string(),
            currency: "BRL".to_string(),
            effective_from: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            effective_until: None,
            entries: vec![
                PriceListEntry {
                    variation_id: v,
                    tier: PriceTier::Wholesale,
                    price: dec!(10.00),
                    min_qty: 100,
                },
                PriceListEntry {
                    variation_id: v,
                    tier: PriceTier::Wholesale,
                    price: dec!(12.00),
                    min_qty: 1,
                },
            ],
            created_at: Utc::now(),
        };
        assert_eq!(pl.price_for(v, PriceTier::Wholesale), Some(dec!(12.00)));
    }
}

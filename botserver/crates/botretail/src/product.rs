//! Product catalog and variation types.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Kind of product variation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VariationKind {
    /// Different size (clothing).
    Size,
    /// Different color.
    Color,
    /// Different flavor (food).
    Flavor,
    /// Different material.
    Material,
    /// Generic SKU difference.
    Sku,
}

/// Product status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProductStatus {
    /// Active and purchasable.
    Active,
    /// Hidden from catalog but still queryable.
    Inactive,
    /// Discontinued.
    Discontinued,
}

/// A single product variation (SKU).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Variation {
    /// Server-assigned variation ID.
    pub id: Uuid,
    /// SKU code (unique per tenant).
    pub sku: String,
    /// Barcode (EAN-13) — optional.
    pub barcode: Option<String>,
    /// Variation kind.
    pub kind: VariationKind,
    /// Human value (e.g. "M", "Red", "1kg").
    pub value: String,
    /// Own cost.
    pub cost: Decimal,
    /// Default price.
    pub price: Decimal,
    /// Current stock (sum across all branches).
    pub stock: i64,
}

/// A product (catalog entry).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Product {
    /// Server-assigned product ID.
    pub id: Uuid,
    /// Tenant.
    pub tenant_id: String,
    /// Display name.
    pub name: String,
    /// Short description.
    pub description: Option<String>,
    /// Category (free-form).
    pub category: Option<String>,
    /// Brand.
    pub brand: Option<String>,
    /// NCM code (Brazilian tax classification).
    pub ncm: Option<String>,
    /// CFOP default.
    pub cfop: Option<String>,
    /// Unit of measure (UN, KG, L, etc.).
    pub unit: String,
    /// All variations (SKUs).
    pub variations: Vec<Variation>,
    /// Status.
    pub status: ProductStatus,
    /// When the product was created.
    pub created_at: DateTime<Utc>,
}

impl Product {
    /// Total stock across all variations.
    pub fn total_stock(&self) -> i64 {
        self.variations.iter().map(|v| v.stock).sum()
    }

    /// Find a variation by SKU.
    pub fn find_variation(&self, sku: &str) -> Option<&Variation> {
        self.variations.iter().find(|v| v.sku == sku)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn find_variation_by_sku() {
        let v = Variation {
            id: Uuid::new_v4(),
            sku: "SKU-1".to_string(),
            barcode: None,
            kind: VariationKind::Size,
            value: "M".to_string(),
            cost: dec!(10.00),
            price: dec!(29.90),
            stock: 5,
        };
        let p = Product {
            id: Uuid::new_v4(),
            tenant_id: "t1".to_string(),
            name: "T-Shirt".to_string(),
            description: None,
            category: Some("apparel".to_string()),
            brand: None,
            ncm: None,
            cfop: None,
            unit: "UN".to_string(),
            variations: vec![v.clone()],
            status: ProductStatus::Active,
            created_at: Utc::now(),
        };
        assert_eq!(p.find_variation("SKU-1").map(|v| v.value.clone()), Some("M".to_string()));
        assert_eq!(p.total_stock(), 5);
    }
}

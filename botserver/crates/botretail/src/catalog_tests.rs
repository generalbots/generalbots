//! Catalog, stock and POS wire shapes not covered by the arithmetic suites.

use chrono::{TimeZone, Utc};
use rust_decimal_macros::dec;
use uuid::Uuid;

use super::{MovementKind, StockLevel, StockMovement};
use crate::pos::{PosError, PosSession};
use crate::product::{Product, ProductStatus, Variation, VariationKind};

fn variation(sku: &str, stock: i64) -> Variation {
    Variation {
        id: Uuid::new_v4(),
        sku: sku.to_string(),
        barcode: None,
        kind: VariationKind::Size,
        value: "M".to_string(),
        cost: dec!(10.00),
        price: dec!(29.90),
        stock,
    }
}

fn product(variations: Vec<Variation>) -> Product {
    Product {
        id: Uuid::new_v4(),
        tenant_id: "t1".to_string(),
        name: "T-Shirt".to_string(),
        description: None,
        category: Some("apparel".to_string()),
        brand: None,
        ncm: None,
        cfop: None,
        unit: "UN".to_string(),
        variations,
        status: ProductStatus::Active,
        created_at: Utc.with_ymd_and_hms(2026, 3, 4, 5, 6, 7).single().expect("time"),
    }
}

#[test]
fn total_stock_sums_every_variation() {
    assert_eq!(product(vec![]).total_stock(), 0);
    let p = product(vec![variation("A", 5), variation("B", 7), variation("C", 0)]);
    assert_eq!(p.total_stock(), 12);
}

#[test]
fn find_variation_returns_none_for_an_unknown_sku() {
    let p = product(vec![variation("A", 5)]);
    assert!(p.find_variation("A").is_some());
    assert!(p.find_variation("a").is_none());
    assert!(p.find_variation("").is_none());
    assert!(product(vec![]).find_variation("A").is_none());
}

#[test]
fn product_statuses_round_trip() {
    for (status, wire) in [
        (ProductStatus::Active, "\"Active\""),
        (ProductStatus::Inactive, "\"Inactive\""),
        (ProductStatus::Discontinued, "\"Discontinued\""),
    ] {
        let text = serde_json::to_string(&status).expect("serializes");
        assert_eq!(text, wire);
        let back: ProductStatus = serde_json::from_str(&text).expect("deserializes");
        assert_eq!(back, status);
    }
}

#[test]
fn variation_kinds_round_trip() {
    for kind in [
        VariationKind::Size,
        VariationKind::Color,
        VariationKind::Flavor,
        VariationKind::Material,
        VariationKind::Sku,
    ] {
        let text = serde_json::to_string(&kind).expect("serializes");
        let back: VariationKind = serde_json::from_str(&text).expect("deserializes");
        assert_eq!(back, kind);
    }
}

#[test]
fn a_product_round_trips_with_its_variations() {
    let p = product(vec![variation("A", 5)]);
    let value = serde_json::to_value(&p).expect("serializes");
    assert_eq!(value["variations"][0]["sku"], "A");
    assert_eq!(value["status"], "Active");
    let back: Product = serde_json::from_value(value).expect("deserializes");
    assert_eq!(back, p);
}

#[test]
fn fulfilment_rejects_negative_and_oversized_requests() {
    let level = StockLevel {
        branch_id: Uuid::nil(),
        variation_id: Uuid::nil(),
        quantity: 10,
        reserved: 4,
    };
    assert_eq!(level.available(), 6);
    assert!(level.can_fulfill(0));
    assert!(level.can_fulfill(6));
    assert!(!level.can_fulfill(7));
    assert!(!level.can_fulfill(-1));
}

#[test]
fn stock_movements_keep_the_signed_delta() {
    let movement = StockMovement {
        id: Uuid::nil(),
        variation_id: Uuid::nil(),
        branch_id: Uuid::nil(),
        kind: MovementKind::Adjustment,
        delta: -3,
        reason: Some("breakage".to_string()),
        reference: None,
        at: Utc.with_ymd_and_hms(2026, 3, 4, 5, 6, 7).single().expect("time"),
    };
    let value = serde_json::to_value(&movement).expect("serializes");
    assert_eq!(value["delta"], -3);
    assert_eq!(value["kind"], "Adjustment");
    let back: StockMovement = serde_json::from_value(value).expect("deserializes");
    assert_eq!(back, movement);
}

#[test]
fn every_movement_kind_is_distinct() {
    let kinds = [
        MovementKind::Inbound,
        MovementKind::Outbound,
        MovementKind::Transfer,
        MovementKind::Adjustment,
        MovementKind::Return,
    ];
    let mut seen = Vec::new();
    for kind in kinds {
        let wire = serde_json::to_string(&kind).expect("serializes");
        assert!(!seen.contains(&wire), "duplicate wire value {wire}");
        seen.push(wire);
    }
}

#[test]
fn a_pos_session_reports_open_or_closed() {
    let open = PosSession {
        id: Uuid::nil(),
        tenant_id: "t1".to_string(),
        branch_id: Uuid::nil(),
        cashier_id: "u1".to_string(),
        opened_at: Utc.with_ymd_and_hms(2026, 3, 4, 8, 0, 0).single().expect("time"),
        closed_at: None,
        opening_float: dec!(100.00),
        closing_count: None,
    };
    let value = serde_json::to_value(&open).expect("serializes");
    assert_eq!(value["closed_at"], serde_json::Value::Null);
    let back: PosSession = serde_json::from_value(value).expect("deserializes");
    assert_eq!(back, open);
}

#[test]
fn pos_errors_render_their_wire_message() {
    assert_eq!(PosError::ZeroTotal.to_string(), "sale total must be > 0");
    assert_eq!(
        PosError::PaymentMismatch {
            paid: dec!(10),
            gross: dec!(12)
        }
        .to_string(),
        "paid 10 != gross 12"
    );
    let id = Uuid::nil();
    assert_eq!(
        PosError::OutOfStock(id).to_string(),
        format!("variation {id} out of stock")
    );
}

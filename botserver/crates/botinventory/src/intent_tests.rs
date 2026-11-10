//! Offline unit tests for the inventory/POS command vocabulary (#2075).
//!
//! BASIC passes every argument as text, so `parse_amount` is the only gate
//! between a script expression and a stored quantity. It must accept the
//! numeric forms a script can produce and fall back for everything else — in
//! particular `NaN`/`inf`, which parse as `f64` but are not amounts.
//! `InventoryRequestKind::as_str` is the wire contract the runtime routes on.

use super::{parse_amount, InventoryRequestKind};

const ALL_KINDS: [(InventoryRequestKind, &str); 9] = [
    (InventoryRequestKind::Product, "product"),
    (InventoryRequestKind::ProductVariation, "product_variation"),
    (InventoryRequestKind::ProductStock, "product_stock"),
    (InventoryRequestKind::InventoryMovement, "inventory_movement"),
    (InventoryRequestKind::StockQuery, "stock_query"),
    (InventoryRequestKind::ProductPrice, "product_price"),
    (InventoryRequestKind::PosSession, "pos_session"),
    (InventoryRequestKind::CartItem, "cart_item"),
    (InventoryRequestKind::PosSale, "pos_sale"),
];

#[test]
fn every_kind_has_its_snake_case_wire_name() {
    for (kind, expected) in ALL_KINDS {
        assert_eq!(kind.as_str(), expected, "{kind:?}");
        let json = serde_json::to_string(&kind).expect("kind is serializable");
        assert_eq!(json, format!("\"{expected}\""));
        let decoded: InventoryRequestKind =
            serde_json::from_str(&json).expect("kind round-trips");
        assert_eq!(decoded, kind);
    }
}

#[test]
fn integers_decimals_and_signs_are_accepted() {
    assert_eq!(parse_amount("10", 1.0), 10.0);
    assert_eq!(parse_amount("0", 1.0), 0.0);
    assert_eq!(parse_amount("-3", 1.0), -3.0);
    assert_eq!(parse_amount("2.5", 1.0), 2.5);
    assert_eq!(parse_amount("1e3", 1.0), 1000.0);
    assert_eq!(parse_amount("0.0001", 1.0), 0.0001);
}

#[test]
fn surrounding_whitespace_is_ignored() {
    assert_eq!(parse_amount("  42  ", 0.0), 42.0);
    assert_eq!(parse_amount("\t7\n", 0.0), 7.0);
}

#[test]
fn non_numeric_text_falls_back_to_the_default() {
    for raw in ["", "   ", "abc", "10 units", "R$ 10", "1,5", "--2", "0x10"] {
        assert_eq!(parse_amount(raw, 99.0), 99.0, "{raw:?}");
    }
}

#[test]
fn non_finite_values_fall_back_to_the_default() {
    // These parse as f64 but must never become a quantity or a price.
    for raw in ["NaN", "nan", "inf", "-inf", "Infinity"] {
        assert_eq!(parse_amount(raw, 1.0), 1.0, "{raw:?}");
    }
}

#[test]
fn the_default_is_returned_verbatim() {
    assert_eq!(parse_amount("nope", 0.0), 0.0);
    assert_eq!(parse_amount("nope", -1.5), -1.5);
}

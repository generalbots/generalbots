//! Offline unit tests for the POS order arithmetic (#2075).
//!
//! `create_order` writes a `NUMERIC(18,2)` total and the order readers decode
//! the `items` JSONB column. Both decisions are pure and live in `order_total`
//! and `items_from_json`; the SQL shell around them is integration-tested.

use super::{items_from_json, order_total, OrderItem};
use axum::http::StatusCode;
use rust_decimal::Decimal;
use std::str::FromStr;
use uuid::Uuid;

fn item(price: &str, quantity: i64) -> OrderItem {
    OrderItem {
        product_id: Uuid::nil(),
        quantity,
        unit_price: price.to_string(),
    }
}

#[test]
fn an_empty_order_totals_zero() {
    assert_eq!(order_total(&[]).expect("no lines is valid"), Decimal::ZERO);
}

#[test]
fn the_total_is_the_sum_of_quantity_times_price() {
    let items = [item("10.50", 2), item("0.99", 3), item("100", 1)];
    // 21.00 + 2.97 + 100.00
    assert_eq!(
        order_total(&items).expect("valid"),
        Decimal::from_str("123.97").expect("literal")
    );
}

#[test]
fn fractional_quantities_are_supported() {
    let total = order_total(&[item("12.00", 0), item("2.50", 4)]).expect("valid");
    assert_eq!(total, Decimal::from_str("10.00").expect("literal"));
}

#[test]
fn a_negative_quantity_credits_the_total() {
    let total = order_total(&[item("10.00", 5), item("-2.50", 2)]).expect("valid");
    assert_eq!(total, Decimal::from_str("45.00").expect("literal"));
}

#[test]
fn one_bad_price_rejects_the_whole_order() {
    let err = order_total(&[item("10.00", 1), item("ten", 1)]).expect_err("must reject");
    assert_eq!(err.0, StatusCode::BAD_REQUEST);
    assert!(err.1.contains("ten"), "message quotes the input: {}", err.1);
}

#[test]
fn items_json_is_decoded_and_malformed_lines_are_dropped() {
    let id = Uuid::from_u128(7);
    let value = serde_json::json!([
        {"product_id": id.to_string(), "quantity": 2, "unit_price": "3.50"},
        {"quantity": 4, "unit_price": "1.00"},
        {"product_id": "not-a-uuid", "quantity": 1},
        {"product_id": id.to_string()}
    ]);

    let items = items_from_json(&value);
    assert_eq!(items.len(), 2, "lines without a usable product_id are dropped");
    assert_eq!(items[0].product_id, id);
    assert_eq!(items[0].quantity, 2);
    assert_eq!(items[0].unit_price, "3.50");
    // Missing quantity/price fall back to the column defaults instead of failing.
    assert_eq!(items[1].quantity, 0);
    assert_eq!(items[1].unit_price, "0");
}

#[test]
fn a_non_array_items_column_decodes_to_no_lines() {
    assert!(items_from_json(&serde_json::Value::Null).is_empty());
    assert!(items_from_json(&serde_json::json!({"product_id": "x"})).is_empty());
    assert!(items_from_json(&serde_json::json!([])).is_empty());
}

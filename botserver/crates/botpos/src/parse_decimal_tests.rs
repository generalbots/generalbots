//! Offline unit tests for the POS money parser (#2068).
//!
//! Order totals arrive as form strings. `parse_decimal` is the single gate
//! between that text and `NUMERIC(18,2)`: it must accept plain and signed
//! decimals and reject anything else with a `400`, never silently coercing a
//! locale-formatted or empty value to zero.

use super::parse_decimal;
use axum::http::StatusCode;
use rust_decimal::Decimal;
use std::str::FromStr;

#[test]
fn plain_and_signed_decimals_are_accepted() {
    assert_eq!(parse_decimal("10.50").expect("valid"), Decimal::from_str("10.50").expect("lit"));
    assert_eq!(parse_decimal("0").expect("valid"), Decimal::from(0));
    assert_eq!(parse_decimal("1234").expect("valid"), Decimal::from(1234));
    assert_eq!(parse_decimal("-5.25").expect("valid"), Decimal::from_str("-5.25").expect("lit"));
    assert_eq!(parse_decimal("0.01").expect("valid"), Decimal::from_str("0.01").expect("lit"));
}

#[test]
fn leading_and_trailing_spaces_are_not_accepted() {
    // `Decimal::from_str` is strict; a padded field is a client bug that must
    // surface as a 400 rather than being trimmed into a valid price.
    assert!(parse_decimal(" 10").is_err());
    assert!(parse_decimal("10 ").is_err());
}

#[test]
fn malformed_amounts_return_a_bad_request() {
    for value in ["", "abc", "1,50", "10.5.5", "--1", "R$ 10", "NaN"] {
        let err = parse_decimal(value).expect_err("must be rejected");
        assert_eq!(err.0, StatusCode::BAD_REQUEST, "{value:?}");
        assert!(
            err.1.contains("Invalid decimal"),
            "message for {value:?} was {}",
            err.1
        );
        assert!(err.1.contains(value), "message must quote the input");
    }
}

#[test]
fn the_error_message_is_client_safe() {
    // No path, pool or SQL detail may leak through the 400 body.
    let err = parse_decimal("oops").expect_err("must be rejected");
    assert!(!err.1.contains('/'));
    assert!(!err.1.to_lowercase().contains("select"));
}

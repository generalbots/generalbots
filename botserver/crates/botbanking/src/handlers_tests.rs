//! Tests for the request-payload helpers shared by the banking handlers.

use super::*;
use rust_decimal_macros::dec;

#[test]
fn decimals_are_parsed_from_the_wire_format() {
    let cases = [
        ("10.50", dec!(10.50)),
        ("0", Decimal::ZERO),
        ("-5", dec!(-5)),
        ("1000000.0001", dec!(1000000.0001)),
    ];
    for (input, expected) in cases {
        assert_eq!(parse_decimal(input), Ok(expected), "input {input:?}");
    }
}

#[test]
fn an_unparseable_amount_is_a_bad_request_naming_the_input() {
    for input in ["", "abc", "1,50", "R$ 10"] {
        let (status, message) = parse_decimal(input).expect_err("must reject");
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(message.contains(input), "{message:?} must quote {input:?}");
    }
}

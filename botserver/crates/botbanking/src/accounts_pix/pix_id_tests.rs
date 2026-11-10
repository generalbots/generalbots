//! Tests for the PIX end-to-end id and the decimal wire formatting.

use super::gen_end_to_end::end_to_end_id;
use super::resolve_branch::decimal_to_string;
use rust_decimal_macros::dec;
use uuid::Uuid;

const FIXED_ID: u128 = 0x0011_2233_4455_6677_8899_aabb_ccdd_eeff;

#[test]
fn an_end_to_end_id_is_e_timestamp_and_32_hex_characters() {
    let id = end_to_end_id(1_723_000_000_000, Uuid::from_u128(FIXED_ID));
    assert_eq!(
        id,
        "E172300000000000112233445566778899aabbccddeeff"
    );
    assert!(id.starts_with('E'));
    assert_eq!(id.len(), 1 + 13 + 32);
    assert!(id[1..].chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn distinct_transfers_get_distinct_ids() {
    let first = end_to_end_id(1_000, Uuid::from_u128(FIXED_ID));
    let second = end_to_end_id(1_000, Uuid::from_u128(FIXED_ID + 1));
    let third = end_to_end_id(2_000, Uuid::from_u128(FIXED_ID));
    assert_ne!(first, second);
    assert_ne!(first, third);
}

#[test]
fn decimals_keep_their_scale_on_the_wire() {
    let cases = [
        (dec!(10.50), "10.50"),
        (dec!(10), "10"),
        (dec!(-1.5), "-1.5"),
        (dec!(0), "0"),
        (dec!(1234567.891), "1234567.891"),
    ];
    for (input, expected) in cases {
        assert_eq!(decimal_to_string(input), expected, "value {input}");
    }
}

#[test]
fn the_payload_defaults_are_published() {
    assert_eq!(super::default_account_type(), "checking");
    assert_eq!(super::default_currency(), "BRL");
    assert_eq!(super::default_statement_format(), "pdf");
}

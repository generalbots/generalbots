//! Contract tests for SYNCHRONIZE's type-coercion table (#2064).
//!
//! The coercion rules decide what reaches the database, so each target type is
//! pinned: a silent fallback (an unparsable integer becoming `0`, a boolean
//! string outside the accepted set becoming `true`) would corrupt a synced row.

use super::*;
use serde_json::json;

#[test]
fn strings_are_stringified_from_any_scalar() {
    assert_eq!(
        coerce_value(&json!("x"), &FieldType::String).expect("string"),
        json!("x")
    );
    assert_eq!(
        coerce_value(&json!(42), &FieldType::String).expect("string"),
        json!("42")
    );
    assert_eq!(
        coerce_value(&json!(true), &FieldType::String).expect("string"),
        json!("true")
    );
    // A missing value becomes the empty string, not the literal "null".
    assert_eq!(
        coerce_value(&Value::Null, &FieldType::String).expect("string"),
        json!("")
    );
}

#[test]
fn integers_parse_from_strings_and_booleans() {
    assert_eq!(
        coerce_value(&json!("42"), &FieldType::Integer).expect("integer"),
        json!(42)
    );
    assert_eq!(
        coerce_value(&json!(true), &FieldType::Integer).expect("integer"),
        json!(1)
    );
    assert_eq!(
        coerce_value(&json!(false), &FieldType::Integer).expect("integer"),
        json!(0)
    );
    // A fractional number is passed through untouched (no truncation).
    assert_eq!(
        coerce_value(&json!(1.5), &FieldType::Integer).expect("integer"),
        json!(1.5)
    );
}

#[test]
fn an_unparsable_integer_is_an_error_not_a_zero() {
    let error = coerce_value(&json!("abc"), &FieldType::Integer).expect_err("must fail");
    assert_eq!(error.error_type(), SyncErrorType::TransformError);
    assert!(!error.is_retryable());
    assert!(coerce_value(&Value::Null, &FieldType::Integer).is_err());
}

#[test]
fn floats_parse_from_strings_and_numbers() {
    assert_eq!(
        coerce_value(&json!("1.25"), &FieldType::Float).expect("float"),
        json!(1.25)
    );
    assert_eq!(
        coerce_value(&json!(3), &FieldType::Float).expect("float"),
        json!(3)
    );
    assert!(coerce_value(&json!("abc"), &FieldType::Float).is_err());
    assert!(coerce_value(&Value::Null, &FieldType::Float).is_err());
}

#[test]
fn boolean_strings_accept_only_the_documented_truthy_set() {
    for truthy in ["true", "TRUE", "1", "yes", "On"] {
        assert_eq!(
            coerce_value(&json!(truthy), &FieldType::Boolean).expect("boolean"),
            json!(true),
            "`{truthy}` must coerce to true"
        );
    }
    for falsy in ["false", "0", "no", "off", "", "anything"] {
        assert_eq!(
            coerce_value(&json!(falsy), &FieldType::Boolean).expect("boolean"),
            json!(false),
            "`{falsy}` must coerce to false"
        );
    }
}

#[test]
fn numbers_coerce_to_booleans_by_non_zero() {
    assert_eq!(
        coerce_value(&json!(7), &FieldType::Boolean).expect("boolean"),
        json!(true)
    );
    assert_eq!(
        coerce_value(&json!(0), &FieldType::Boolean).expect("boolean"),
        json!(false)
    );
    assert_eq!(
        coerce_value(&Value::Null, &FieldType::Boolean).expect("boolean"),
        json!(false)
    );
}

#[test]
fn a_unix_timestamp_becomes_an_rfc3339_string() {
    assert_eq!(
        coerce_value(&json!(0), &FieldType::DateTime).expect("datetime"),
        json!("1970-01-01T00:00:00+00:00")
    );
    // Strings are assumed to be already formatted and pass through.
    assert_eq!(
        coerce_value(&json!("2026-11-10T12:00:00Z"), &FieldType::DateTime).expect("datetime"),
        json!("2026-11-10T12:00:00Z")
    );
}

#[test]
fn json_values_are_never_rewritten() {
    let value = json!({"a": [1, 2], "b": null});
    assert_eq!(
        coerce_value(&value, &FieldType::Json).expect("json"),
        value
    );
}

#[test]
fn uuids_are_validated_and_kept_verbatim() {
    let uuid = "3f1d8b9e-8f2a-4c3b-9d4e-5a6b7c8d9e0f";
    assert_eq!(
        coerce_value(&json!(uuid), &FieldType::Uuid).expect("uuid"),
        json!(uuid)
    );
    assert!(coerce_value(&json!("not-a-uuid"), &FieldType::Uuid).is_err());
    assert!(coerce_value(&json!(1), &FieldType::Uuid).is_err());
}

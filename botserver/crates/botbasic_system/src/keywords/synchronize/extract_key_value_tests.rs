//! Contract tests for the SYNCHRONIZE key lookup and field-type names (#2064).
//!
//! `extract_key_value` decides whether a fetched record has an identity: only a
//! scalar counts, so an object or a list in the key column must not be
//! stringified into a bogus key. The `FieldType` wire names are the ones stored
//! in a sync configuration.

use super::*;
use serde_json::json;

#[test]
fn scalar_keys_are_read_as_text() {
    let record = json!({
        "id": "abc",
        "count": 3,
        "active": true,
    });

    assert_eq!(extract_key_value(&record, "id").as_deref(), Some("abc"));
    assert_eq!(extract_key_value(&record, "count").as_deref(), Some("3"));
    assert_eq!(extract_key_value(&record, "active").as_deref(), Some("true"));
}

#[test]
fn non_scalar_and_missing_keys_are_rejected() {
    let record = json!({
        "nested": {"x": 1},
        "list": [1, 2],
        "nothing": null,
    });

    for key in ["nested", "list", "nothing", "missing"] {
        assert_eq!(extract_key_value(&record, key), None, "key {key}");
    }
}

#[test]
fn field_type_wire_names_are_stable() {
    let cases = [
        (FieldType::String, "string"),
        (FieldType::Integer, "integer"),
        (FieldType::Float, "float"),
        (FieldType::Boolean, "boolean"),
        (FieldType::DateTime, "datetime"),
        (FieldType::Json, "json"),
        (FieldType::Uuid, "uuid"),
    ];
    for (field_type, expected) in cases {
        assert_eq!(field_type.to_string(), expected);
    }
}

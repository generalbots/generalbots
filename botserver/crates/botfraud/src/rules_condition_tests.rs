//! Malformed-operator behaviour: a condition that never expressed a value must
//! be false, never a wildcard.

use chrono::Utc;
use serde_json::json;
use uuid::Uuid;

use super::evaluate_rule;
use crate::types::{FraudAssessmentRequest, FraudRule};

fn rule(condition: serde_json::Value) -> FraudRule {
    FraudRule {
        id: Uuid::new_v4(),
        branch_id: Uuid::nil(),
        name: "test".to_string(),
        description: None,
        rule_type: "threshold".to_string(),
        condition_json: condition,
        action: "flag".to_string(),
        severity: "high".to_string(),
        is_active: true,
        created_at: Utc::now(),
    }
}

fn request(details: serde_json::Value) -> FraudAssessmentRequest {
    FraudAssessmentRequest {
        event_type: "payment".to_string(),
        entity_type: "order".to_string(),
        entity_id: Uuid::new_v4(),
        details,
        branch_id: None,
    }
}

#[test]
fn an_equality_without_a_value_never_matches() {
    let details = request(json!({ "country": "BR" }));
    for op in ["==", "!="] {
        let condition = json!({ "field": "country", "op": op });
        assert!(
            !evaluate_rule(&rule(condition.clone()), &details),
            "{condition} must not match"
        );
    }
}

#[test]
fn an_explicit_null_value_still_compares() {
    let nulls = request(json!({ "reason": null }));
    assert!(evaluate_rule(
        &rule(json!({ "field": "reason", "op": "==", "value": null })),
        &nulls
    ));
    let filled = request(json!({ "reason": "chargeback" }));
    assert!(!evaluate_rule(
        &rule(json!({ "field": "reason", "op": "==", "value": null })),
        &filled
    ));
    assert!(evaluate_rule(
        &rule(json!({ "field": "reason", "op": "!=", "value": null })),
        &filled
    ));
}

#[test]
fn contains_without_a_string_needle_never_matches() {
    let details = request(json!({ "email": "buyer@example.com" }));
    for condition in [
        json!({ "field": "email", "op": "contains" }),
        json!({ "field": "email", "op": "contains", "value": 123 }),
        json!({ "field": "email", "op": "contains", "value": ["buyer"] }),
        json!({ "field": "email", "op": "contains", "value": null }),
    ] {
        assert!(
            !evaluate_rule(&rule(condition.clone()), &details),
            "{condition} must not match"
        );
    }
}

#[test]
fn an_empty_string_needle_is_a_deliberate_match_all() {
    let details = request(json!({ "email": "buyer@example.com" }));
    assert!(evaluate_rule(
        &rule(json!({ "field": "email", "op": "contains", "value": "" })),
        &details
    ));
}

#[test]
fn in_requires_an_array() {
    let details = request(json!({ "country": "BR" }));
    for condition in [
        json!({ "field": "country", "op": "in", "value": "BR" }),
        json!({ "field": "country", "op": "in", "value": { "a": "BR" } }),
        json!({ "field": "country", "op": "in" }),
    ] {
        assert!(
            !evaluate_rule(&rule(condition.clone()), &details),
            "{condition} must not match"
        );
    }
    assert!(evaluate_rule(
        &rule(json!({ "field": "country", "op": "in", "value": ["BR"] })),
        &details
    ));
}

#[test]
fn numeric_operators_ignore_non_numeric_payloads() {
    let details = request(json!({ "amount": "500", "flag": true, "nothing": null }));
    for (field, op) in [
        ("amount", ">"),
        ("amount", "<"),
        ("flag", ">"),
        ("flag", "<"),
        ("nothing", ">"),
        ("nothing", "<"),
    ] {
        let condition = json!({ "field": field, "op": op, "value": 10 });
        assert!(
            !evaluate_rule(&rule(condition.clone()), &details),
            "{condition} must not match"
        );
    }
}

#[test]
fn equality_compares_json_values_structurally() {
    let details = request(json!({ "tags": ["a", "b"], "amount": 500 }));
    assert!(evaluate_rule(
        &rule(json!({ "field": "tags", "op": "==", "value": ["a", "b"] })),
        &details
    ));
    assert!(!evaluate_rule(
        &rule(json!({ "field": "tags", "op": "==", "value": ["b", "a"] })),
        &details
    ));
    // A number and its string spelling are different values.
    assert!(!evaluate_rule(
        &rule(json!({ "field": "amount", "op": "==", "value": "500" })),
        &details
    ));
}

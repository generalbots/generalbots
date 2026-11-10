//! Wire types of the fraud surface.

use chrono::{TimeZone, Utc};
use serde_json::json;
use uuid::Uuid;

use super::{
    FraudAssessmentRequest, FraudAssessmentResult, FraudRule, FraudStats, VelocityCheck,
    VelocityResult,
};

#[test]
fn an_assessment_request_defaults_the_branch() {
    let request: FraudAssessmentRequest = serde_json::from_value(json!({
        "event_type": "payment",
        "entity_type": "order",
        "entity_id": Uuid::nil(),
        "details": { "amount": 500 },
    }))
    .expect("parses");
    assert_eq!(request.branch_id, None);
    assert_eq!(request.details["amount"], 500);

    let explicit: FraudAssessmentRequest = serde_json::from_value(json!({
        "event_type": "payment",
        "entity_type": "order",
        "entity_id": Uuid::nil(),
        "details": {},
        "branch_id": "00000000-0000-0000-0000-000000000001",
    }))
    .expect("parses");
    assert_eq!(
        explicit.branch_id,
        Some(Uuid::from_u128(1))
    );
}

#[test]
fn an_assessment_result_round_trips() {
    let result = FraudAssessmentResult {
        risk_score: 85,
        risk_level: "critical".to_string(),
        action_taken: "block".to_string(),
        triggered_rules: vec!["blocklist_match".to_string()],
        ml_score: None,
    };
    let value = serde_json::to_value(&result).expect("serializes");
    assert_eq!(value["risk_score"], 85);
    assert_eq!(value["ml_score"], serde_json::Value::Null);
    let back: FraudAssessmentResult = serde_json::from_value(value).expect("deserializes");
    assert_eq!(back.triggered_rules, vec!["blocklist_match".to_string()]);
    assert_eq!(back.action_taken, "block");
}

#[test]
fn a_rule_round_trips_with_its_condition() {
    let rule = FraudRule {
        id: Uuid::from_u128(2),
        branch_id: Uuid::nil(),
        name: "large amount".to_string(),
        description: Some("amount above the threshold".to_string()),
        rule_type: "threshold".to_string(),
        condition_json: json!({ "field": "amount", "op": ">", "value": 1000 }),
        action: "review".to_string(),
        severity: "high".to_string(),
        is_active: true,
        created_at: Utc.with_ymd_and_hms(2026, 2, 3, 4, 5, 6).single().expect("time"),
    };
    let value = serde_json::to_value(&rule).expect("serializes");
    let back: FraudRule = serde_json::from_value(value).expect("deserializes");
    assert_eq!(back.condition_json["op"], ">");
    assert_eq!(back.severity, "high");
    assert!(back.is_active);
    assert_eq!(back.created_at, rule.created_at);
}

#[test]
fn stats_default_to_zero() {
    let stats = FraudStats::default();
    assert_eq!(stats.total_events, 0);
    assert_eq!(stats.blocked_count, 0);
    assert_eq!(stats.flagged_count, 0);
    assert_eq!(stats.reviewed_count, 0);
    assert_eq!(stats.high_risk_count, 0);
    assert_eq!(stats.rules_active, 0);
    let value = serde_json::to_value(&stats).expect("serializes");
    assert_eq!(value["total_events"], 0);
}

#[test]
fn a_velocity_result_reports_the_comparison() {
    let result = VelocityResult {
        current_count: 12,
        threshold: 10,
        exceeded: true,
    };
    let value = serde_json::to_value(&result).expect("serializes");
    assert_eq!(value["current_count"], 12);
    assert_eq!(value["exceeded"], true);
}

#[test]
fn a_velocity_check_carries_its_window() {
    let check = VelocityCheck {
        identifier: "device-abc123".to_string(),
        identifier_type: "ip".to_string(),
        event_type: "login".to_string(),
        threshold: 10,
        window_minutes: 60,
    };
    let back: VelocityCheck =
        serde_json::from_value(serde_json::to_value(&check).expect("serializes")).expect("parses");
    assert_eq!(back.window_minutes, 60);
    assert_eq!(back.identifier_type, "ip");
}

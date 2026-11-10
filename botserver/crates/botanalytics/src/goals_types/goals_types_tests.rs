//! Offline tests for the OKR enum labels and error mapping (issue #2075).
//! The label parsers are the wire format shared with the goals UI.

use super::*;

#[test]
fn objective_status_labels_round_trip() {
    let labels = [
        "draft",
        "active",
        "on_track",
        "at_risk",
        "behind",
        "completed",
        "cancelled",
    ];
    for label in labels {
        let status = ObjectiveStatus::from_label(label);
        assert_eq!(status.to_str(), label, "label {label}");
    }
}

#[test]
fn objective_status_falls_back_to_draft() {
    assert_eq!(ObjectiveStatus::from_label("").to_str(), "draft");
    assert_eq!(ObjectiveStatus::from_label("ACTIVE").to_str(), "draft");
    assert_eq!(ObjectiveStatus::from_label("archived").to_str(), "draft");
}

#[test]
fn visibility_labels_round_trip() {
    for label in ["private", "team", "organization"] {
        assert_eq!(Visibility::from_label(label).to_str(), label);
    }
    assert_eq!(Visibility::from_label("public").to_str(), "team");
}

#[test]
fn metric_type_labels_round_trip() {
    for label in ["percentage", "number", "currency", "boolean"] {
        assert_eq!(MetricType::from_label(label).to_str(), label);
    }
    assert_eq!(MetricType::from_label("usd").to_str(), "number");
}

#[test]
fn key_result_status_labels_round_trip() {
    for label in ["not_started", "in_progress", "at_risk", "completed"] {
        assert_eq!(KRStatus::from_label(label).to_str(), label);
    }
    assert_eq!(KRStatus::from_label("done").to_str(), "not_started");
}

#[test]
fn confidence_labels_round_trip_and_default_to_medium() {
    for label in ["low", "medium", "high"] {
        assert_eq!(Confidence::from_label(label).to_str(), label);
    }
    assert_eq!(Confidence::from_label("certain").to_str(), "medium");
}

#[test]
fn a_checkin_without_confidence_is_recorded_as_medium() {
    let record = CheckInRecord {
        id: Uuid::from_u128(1),
        org_id: Uuid::nil(),
        bot_id: Uuid::nil(),
        key_result_id: Uuid::from_u128(2),
        user_id: Uuid::from_u128(3),
        previous_value: Some(BigDecimal::from(10)),
        new_value: BigDecimal::from(20),
        note: None,
        confidence: None,
        blockers: None,
        created_at: Utc::now(),
        branch_id: Uuid::nil(),
    };

    let checkin = record_to_checkin(record);

    assert_eq!(checkin.confidence.to_str(), "medium");
    assert_eq!(checkin.previous_value, 10.0);
    assert_eq!(checkin.new_value, 20.0);
    assert_eq!(checkin.note, "");
}

fn status_of(error: GoalsError) -> u16 {
    use axum::response::IntoResponse;
    error.into_response().status().as_u16()
}

#[test]
fn goals_errors_map_to_the_published_http_status() {
    let cases = [
        (GoalsError::NotFound("o".to_string()), 404),
        (GoalsError::Unauthorized("u".to_string()), 401),
        (GoalsError::Validation("v".to_string()), 400),
        (GoalsError::Database("d".to_string()), 500),
    ];

    for (error, expected) in cases {
        assert_eq!(status_of(error), expected);
    }
}

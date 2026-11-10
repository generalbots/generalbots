//! Contract tests for the compliance domain enums.
//!
//! Every enum here is a wire format: the `Display`/`FromStr` labels and the
//! `snake_case` serde labels are read by API clients, SQL filters
//! (`compliance_checks.status = "pass"`, issue severity counters in
//! `handle_get_report`) and CSV exports. The tables below are the published
//! labels, not whatever the implementation happens to produce.

use super::*;
use std::fmt::Debug;
use std::str::FromStr;

fn assert_labels<T>(pairs: &[(T, &str)])
where
    T: std::fmt::Display + FromStr + PartialEq + Debug,
    T::Err: Debug,
{
    for (value, label) in pairs {
        assert_eq!(value.to_string(), *label, "Display label for {value:?}");
        let parsed = T::from_str(label).expect("every displayed label must parse back");
        assert_eq!(&parsed, value, "{label} must parse back to {value:?}");
    }
}

fn assert_serde_labels<T>(pairs: &[(T, &str)])
where
    T: Serialize + for<'de> Deserialize<'de> + PartialEq + Debug,
{
    for (value, label) in pairs {
        let encoded = serde_json::to_value(value).expect("enum must serialize");
        assert_eq!(
            encoded,
            serde_json::Value::String((*label).to_string()),
            "serde label for {value:?}"
        );
        let decoded: T = serde_json::from_value(encoded).expect("serialized enum must deserialize");
        assert_eq!(&decoded, value, "{label} must deserialize back to {value:?}");
    }
}

fn assert_rejects<T>(unknown: &[&str])
where
    T: FromStr + Debug,
    T::Err: std::fmt::Display,
{
    for input in unknown {
        let error = T::from_str(input).expect_err("unknown labels must be rejected");
        assert!(
            error.to_string().contains(input),
            "error for {input:?} should name the input, got: {error}"
        );
    }
}

#[test]
fn severity_labels_round_trip_and_serialize_as_snake_case() {
    let pairs = [
        (Severity::Low, "low"),
        (Severity::Medium, "medium"),
        (Severity::High, "high"),
        (Severity::Critical, "critical"),
    ];
    assert_labels(&pairs);
    assert_serde_labels(&pairs);
}

#[test]
fn severity_is_ordered_from_low_to_critical() {
    let ladder = [Severity::Low, Severity::Medium, Severity::High, Severity::Critical];
    for window in ladder.windows(2) {
        assert!(
            window[0] < window[1],
            "{:?} must sort before {:?}",
            window[0],
            window[1]
        );
    }
    assert_eq!(Severity::Critical, Severity::Critical.max(Severity::Low));
}

#[test]
fn risk_status_labels_round_trip() {
    let pairs = [
        (RiskStatus::Open, "open"),
        (RiskStatus::InProgress, "in_progress"),
        (RiskStatus::Mitigated, "mitigated"),
        (RiskStatus::Accepted, "accepted"),
        (RiskStatus::Closed, "closed"),
    ];
    assert_labels(&pairs);
    assert_serde_labels(&pairs);
}

#[test]
fn risk_category_labels_round_trip() {
    let pairs = [
        (RiskCategory::Technical, "technical"),
        (RiskCategory::Operational, "operational"),
        (RiskCategory::Financial, "financial"),
        (RiskCategory::Compliance, "compliance"),
        (RiskCategory::Reputational, "reputational"),
    ];
    assert_labels(&pairs);
    assert_serde_labels(&pairs);
}

#[test]
fn treatment_strategy_labels_round_trip() {
    let pairs = [
        (TreatmentStrategy::Mitigate, "mitigate"),
        (TreatmentStrategy::Accept, "accept"),
        (TreatmentStrategy::Transfer, "transfer"),
        (TreatmentStrategy::Avoid, "avoid"),
    ];
    assert_labels(&pairs);
    assert_serde_labels(&pairs);
}

#[test]
fn compliance_status_labels_round_trip() {
    let pairs = [
        (ComplianceStatus::Compliant, "compliant"),
        (ComplianceStatus::PartialCompliance, "partial_compliance"),
        (ComplianceStatus::NonCompliant, "non_compliant"),
        (ComplianceStatus::InProgress, "in_progress"),
        (ComplianceStatus::NotApplicable, "not_applicable"),
    ];
    assert_labels(&pairs);
    assert_serde_labels(&pairs);
}

#[test]
fn review_status_labels_round_trip() {
    let pairs = [
        (ReviewStatus::Pending, "pending"),
        (ReviewStatus::InProgress, "in_progress"),
        (ReviewStatus::Completed, "completed"),
        (ReviewStatus::Approved, "approved"),
    ];
    assert_labels(&pairs);
    assert_serde_labels(&pairs);
}

#[test]
fn audit_event_type_labels_round_trip() {
    let pairs = [
        (AuditEventType::Access, "access"),
        (AuditEventType::Modification, "modification"),
        (AuditEventType::Deletion, "deletion"),
        (AuditEventType::Security, "security"),
        (AuditEventType::Admin, "admin"),
        (AuditEventType::Authentication, "authentication"),
        (AuditEventType::Authorization, "authorization"),
    ];
    assert_labels(&pairs);
    assert_serde_labels(&pairs);
}

#[test]
fn action_result_labels_round_trip() {
    let pairs = [
        (ActionResult::Success, "success"),
        (ActionResult::Failure, "failure"),
        (ActionResult::Denied, "denied"),
        (ActionResult::Error, "error"),
    ];
    assert_labels(&pairs);
    assert_serde_labels(&pairs);
}

#[test]
fn training_type_labels_round_trip() {
    let pairs = [
        (TrainingType::SecurityAwareness, "security_awareness"),
        (TrainingType::DataProtection, "data_protection"),
        (TrainingType::IncidentResponse, "incident_response"),
        (TrainingType::ComplianceOverview, "compliance_overview"),
        (TrainingType::RoleSpecific, "role_specific"),
    ];
    assert_labels(&pairs);
    assert_serde_labels(&pairs);
}

#[test]
fn compliance_framework_labels_are_lowercase_and_accept_aliases() {
    let pairs = [
        (ComplianceFramework::Gdpr, "gdpr"),
        (ComplianceFramework::Soc2, "soc2"),
        (ComplianceFramework::Iso27001, "iso27001"),
        (ComplianceFramework::Hipaa, "hipaa"),
        (ComplianceFramework::PciDss, "pci_dss"),
    ];
    assert_labels(&pairs);
    assert_serde_labels(&pairs);

    assert_eq!(
        ComplianceFramework::from_str("GDPR").expect("framework parsing is case-insensitive"),
        ComplianceFramework::Gdpr
    );
    assert_eq!(
        ComplianceFramework::from_str("pcidss").expect("legacy alias must keep working"),
        ComplianceFramework::PciDss
    );
}

#[test]
fn unknown_labels_are_rejected_and_named_in_the_error() {
    assert_rejects::<Severity>(&["", "urgent", "LOW"]);
    assert_rejects::<RiskStatus>(&["", "done", "Open"]);
    assert_rejects::<RiskCategory>(&["", "legal"]);
    assert_rejects::<TreatmentStrategy>(&["", "ignore"]);
    assert_rejects::<ComplianceStatus>(&["", "partial"]);
    assert_rejects::<ReviewStatus>(&["", "rejected"]);
    assert_rejects::<AuditEventType>(&["", "login"]);
    assert_rejects::<ActionResult>(&["", "ok"]);
    assert_rejects::<TrainingType>(&["", "phishing"]);
    assert_rejects::<ComplianceFramework>(&["", "iso9001"]);
}

#[test]
fn every_display_label_is_unique_within_its_enum() {
    let labels: Vec<String> = [
        Severity::Low.to_string(),
        Severity::Medium.to_string(),
        Severity::High.to_string(),
        Severity::Critical.to_string(),
    ]
    .into_iter()
    .collect();
    let unique: std::collections::HashSet<_> = labels.iter().collect();
    assert_eq!(unique.len(), labels.len(), "severity labels must be unique");

    let statuses: Vec<String> = [
        RiskStatus::Open,
        RiskStatus::InProgress,
        RiskStatus::Mitigated,
        RiskStatus::Accepted,
        RiskStatus::Closed,
    ]
    .iter()
    .map(ToString::to_string)
    .collect();
    let unique: std::collections::HashSet<_> = statuses.iter().collect();
    assert_eq!(unique.len(), statuses.len(), "risk status labels must be unique");
}

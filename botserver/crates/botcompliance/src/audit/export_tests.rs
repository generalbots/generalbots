//! Exports, coverage and trail behaviour of the audit service.

use super::*;

fn event(
    event_type: AuditEventType,
    severity: AuditSeverity,
    user_id: Option<Uuid>,
    outcome: AuditOutcome,
    timestamp: DateTime<Utc>,
) -> AuditEvent {
    AuditEvent {
        id: Uuid::new_v4(),
        timestamp,
        event_type,
        severity,
        user_id,
        session_id: None,
        ip_address: None,
        resource_id: None,
        action: "test action".to_string(),
        outcome,
        details: HashMap::new(),
        metadata: serde_json::json!({}),
    }
}

#[tokio::test]
async fn csv_export_uses_the_published_header_and_quotes_fields() {
    let service = AuditService::new(10);
    service
        .log_event(
            AuditEventType::UserLogin,
            AuditSeverity::Warning,
            None,
            "login, failed".to_string(),
            AuditOutcome::Failure,
            HashMap::new(),
        )
        .await
        .expect("logging must succeed");

    let bytes = service
        .export_logs(ExportFormat::Csv, None)
        .await
        .expect("csv export must succeed");
    let csv = String::from_utf8(bytes).expect("csv is utf-8");
    let mut lines = csv.lines();
    assert_eq!(
        lines.next(),
        Some("ID,Timestamp,Type,Severity,User,Action,Outcome")
    );
    let row = lines.next().expect("one data row");
    assert!(row.contains("UserLogin"));
    assert!(row.contains("Warning"));
    assert!(row.contains("Failure"));
    assert!(row.contains("\"login, failed\""), "commas must be quoted: {row}");
    assert!(lines.next().is_none(), "no trailing rows");
}

#[tokio::test]
async fn json_export_round_trips_every_event() {
    let service = AuditService::new(10);
    service
        .log_event(
            AuditEventType::DataDeletion,
            AuditSeverity::Error,
            Some(Uuid::new_v4()),
            "delete row".to_string(),
            AuditOutcome::Partial,
            HashMap::new(),
        )
        .await
        .expect("logging must succeed");

    let bytes = service
        .export_logs(ExportFormat::Json, None)
        .await
        .expect("json export must succeed");
    let parsed: serde_json::Value = serde_json::from_slice(&bytes).expect("valid json");
    let array = parsed.as_array().expect("an array of events");
    assert_eq!(array.len(), 1);
    assert_eq!(array[0]["event_type"], serde_json::json!("DataDeletion"));
    assert_eq!(array[0]["outcome"], serde_json::json!("Partial"));
}

#[tokio::test]
async fn the_compliance_report_counts_findings_and_coverage() {
    let first = Uuid::new_v4();
    let second = Uuid::new_v4();
    let service = AuditService::new(100);
    for (kind, severity, user, outcome) in [
        (AuditEventType::SecurityAlert, AuditSeverity::Critical, first, AuditOutcome::Success),
        (AuditEventType::ComplianceViolation, AuditSeverity::Error, second, AuditOutcome::Failure),
        (AuditEventType::UserLogin, AuditSeverity::Info, first, AuditOutcome::Failure),
        (AuditEventType::UserLogin, AuditSeverity::Info, first, AuditOutcome::Success),
    ] {
        service
            .log_event(kind, severity, Some(user), "action".to_string(), outcome, HashMap::new())
            .await
            .expect("logging must succeed");
    }

    let report = service.get_compliance_report().await;
    assert_eq!(report.total_events, 4);
    assert_eq!(report.security_incidents, 1);
    assert_eq!(report.compliance_violations, 1);
    assert_eq!(report.failed_logins, 1);
    assert_eq!(report.unique_users, 2);
    assert_eq!(report.critical_events, 1);
    // Three of the twelve event types are represented.
    assert!((report.audit_coverage - 25.0).abs() < f64::EPSILON);
}

#[test]
fn audit_coverage_counts_distinct_event_types() {
    let now = Utc::now();
    let mut events = VecDeque::new();
    assert_eq!(AuditService::calculate_audit_coverage(&events), 0.0);

    events.push_back(event(
        AuditEventType::UserLogin,
        AuditSeverity::Info,
        None,
        AuditOutcome::Success,
        now,
    ));
    events.push_back(event(
        AuditEventType::UserLogin,
        AuditSeverity::Info,
        None,
        AuditOutcome::Success,
        now,
    ));
    let single = AuditService::calculate_audit_coverage(&events);
    assert!((single - 100.0 / 12.0).abs() < 1e-9, "duplicates count once");

    for kind in [
        AuditEventType::UserLogout,
        AuditEventType::PasswordChange,
        AuditEventType::PermissionGranted,
        AuditEventType::PermissionRevoked,
        AuditEventType::DataAccess,
        AuditEventType::DataModification,
        AuditEventType::DataDeletion,
        AuditEventType::ConfigurationChange,
        AuditEventType::SecurityAlert,
        AuditEventType::SystemError,
        AuditEventType::ComplianceViolation,
    ] {
        events.push_back(event(
            kind,
            AuditSeverity::Info,
            None,
            AuditOutcome::Success,
            now,
        ));
    }
    assert_eq!(AuditService::calculate_audit_coverage(&events), 100.0);
}

#[tokio::test]
async fn trails_can_be_created_extended_and_sealed() {
    let service = AuditService::new(10);
    let trail_id = service
        .create_trail("quarterly review".to_string(), vec!["sox".to_string()])
        .await
        .expect("trail creation must succeed");
    let event_id = service
        .log_event(
            AuditEventType::DataAccess,
            AuditSeverity::Info,
            None,
            "read".to_string(),
            AuditOutcome::Success,
            HashMap::new(),
        )
        .await
        .expect("logging must succeed");

    service
        .add_to_trail(trail_id, event_id)
        .await
        .expect("open trails accept events");
    service
        .end_trail(trail_id, "complete".to_string())
        .await
        .expect("ending must succeed");

    let error = service
        .add_to_trail(trail_id, event_id)
        .await
        .expect_err("sealed trails must reject new events");
    assert!(error.to_string().contains("Trail already ended"));

    let missing = service
        .end_trail(Uuid::new_v4(), "nope".to_string())
        .await
        .expect_err("unknown trails must be rejected");
    assert!(missing.to_string().contains("Trail not found"));
}

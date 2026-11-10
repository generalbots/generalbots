//! Behavioural tests for the incident response state machine and in-memory
//! service. Everything here is offline: the service keeps incidents, hooks,
//! runbooks and escalation policies in memory.

use super::new_incident::{NewAction, NewEvidence, NewIncident};
use super::*;

fn severity() -> IncidentSeverity {
    IncidentSeverity::Critical
}

fn new_incident(severity: IncidentSeverity, source: IncidentSource) -> NewIncident {
    NewIncident {
        title: "Database exposed".to_string(),
        description: "Public bucket with customer dumps".to_string(),
        severity,
        category: IncidentCategory::DataLeak,
        affected_systems: vec!["drive".to_string()],
        reported_by: "oncall@example.com".to_string(),
        source,
        tags: vec!["data".to_string()],
        metadata: HashMap::new(),
    }
}

fn manual(severity: IncidentSeverity) -> NewIncident {
    new_incident(severity, IncidentSource::Manual)
}

#[test]
fn the_status_ladder_is_linear_and_terminates_at_closed() {
    let chain = [
        IncidentStatus::New,
        IncidentStatus::Triaging,
        IncidentStatus::Investigating,
        IncidentStatus::Containing,
        IncidentStatus::Eradicating,
        IncidentStatus::Recovering,
        IncidentStatus::PostIncident,
        IncidentStatus::Resolved,
        IncidentStatus::Closed,
    ];

    let mut current = IncidentStatus::New;
    for expected_next in &chain[1..] {
        let next = current.next_status().expect("every status but Closed advances");
        assert_eq!(&next, expected_next, "{current:?} must advance to {expected_next:?}");
        current = next;
    }
    assert_eq!(current.next_status(), None, "Closed is terminal");
}

#[test]
fn only_resolved_and_closed_are_inactive() {
    for status in [
        IncidentStatus::New,
        IncidentStatus::Triaging,
        IncidentStatus::Investigating,
        IncidentStatus::Containing,
        IncidentStatus::Eradicating,
        IncidentStatus::Recovering,
        IncidentStatus::PostIncident,
    ] {
        assert!(status.is_active(), "{status:?} must count as active");
    }
    assert!(!IncidentStatus::Resolved.is_active());
    assert!(!IncidentStatus::Closed.is_active());
}

#[test]
fn status_labels_are_unique_snake_case_strings() {
    let pairs = [
        (IncidentStatus::New, "new"),
        (IncidentStatus::Triaging, "triaging"),
        (IncidentStatus::Investigating, "investigating"),
        (IncidentStatus::Containing, "containing"),
        (IncidentStatus::Eradicating, "eradicating"),
        (IncidentStatus::Recovering, "recovering"),
        (IncidentStatus::PostIncident, "post_incident"),
        (IncidentStatus::Resolved, "resolved"),
        (IncidentStatus::Closed, "closed"),
    ];
    let mut seen = std::collections::HashSet::new();
    for (status, label) in pairs {
        assert_eq!(status.as_str(), label);
        assert!(seen.insert(label), "{label} must be unique");
        assert!(
            label.chars().all(|c| c.is_ascii_lowercase() || c == '_'),
            "{label} must be a snake_case label"
        );
    }
}

#[test]
fn category_labels_are_snake_case_and_unique() {
    let pairs = [
        (IncidentCategory::SecurityBreach, "security_breach"),
        (IncidentCategory::DataLeak, "data_leak"),
        (IncidentCategory::ServiceOutage, "service_outage"),
        (IncidentCategory::PerformanceDegradation, "performance_degradation"),
        (IncidentCategory::UnauthorizedAccess, "unauthorized_access"),
        (IncidentCategory::Malware, "malware"),
        (IncidentCategory::PhishingAttack, "phishing_attack"),
        (IncidentCategory::DenialOfService, "denial_of_service"),
        (IncidentCategory::ConfigurationError, "configuration_error"),
        (IncidentCategory::ComplianceViolation, "compliance_violation"),
        (IncidentCategory::Other, "other"),
    ];
    let mut seen = std::collections::HashSet::new();
    for (category, label) in pairs {
        assert_eq!(category.as_str(), label);
        assert!(seen.insert(label), "{label} must be unique");
    }
}

#[tokio::test]
async fn a_new_incident_starts_in_new_with_one_creation_entry() {
    let service = IncidentResponseService::new();
    let incident = service.create_incident(manual(severity())).await;

    assert_eq!(incident.status, IncidentStatus::New);
    assert_eq!(incident.severity, IncidentSeverity::Critical);
    assert!(!incident.sla_breach);
    assert!(incident.resolved_at.is_none());
    assert!(incident.assigned_to.is_none());
    assert_eq!(incident.timeline.len(), 1);
    assert_eq!(incident.timeline[0].event_type, TimelineEventType::Created);
    assert_eq!(
        incident.timeline[0].user.as_deref(),
        Some("oncall@example.com")
    );
    assert_eq!(incident.created_at, incident.updated_at);
    assert_eq!(service.get_incident(incident.id).await.map(|i| i.id), Some(incident.id));
}

#[tokio::test]
async fn incident_numbers_are_unique_and_increase() {
    let service = IncidentResponseService::new();
    let first = service.create_incident(manual(severity())).await;
    let second = service.create_incident(manual(severity())).await;

    assert!(first.incident_number.starts_with("INC-"), "{}", first.incident_number);
    assert_ne!(first.incident_number, second.incident_number);

    let suffix = |number: &str| -> u64 {
        number
            .rsplit('-')
            .next()
            .and_then(|tail| tail.parse().ok())
            .expect("incident numbers end with a numeric sequence")
    };
    assert_eq!(suffix(&second.incident_number), suffix(&first.incident_number) + 1);
}

#[tokio::test]
async fn the_response_deadline_follows_the_severity_policy() {
    let service = IncidentResponseService::new();
    for severity in [
        IncidentSeverity::Critical,
        IncidentSeverity::High,
        IncidentSeverity::Medium,
        IncidentSeverity::Low,
        IncidentSeverity::Informational,
    ] {
        let expected = severity.response_time_minutes();
        let incident = service.create_incident(manual(severity.clone())).await;
        let allowed = (incident.response_due_at - incident.created_at).num_minutes();
        assert_eq!(allowed, expected, "{severity:?} must allow {expected} minutes");
    }
}

#[tokio::test]
async fn only_automated_sources_mark_the_creation_entry_as_automated() {
    let service = IncidentResponseService::new();
    for (source, automated) in [
        (IncidentSource::Manual, false),
        (IncidentSource::CustomerReport, false),
        (IncidentSource::AuditFinding, false),
        (IncidentSource::Automated, true),
        (IncidentSource::Alert, true),
    ] {
        let incident = service.create_incident(new_incident(severity(), source)).await;
        assert_eq!(
            incident.timeline[0].automated, automated,
            "creation entry for this source must be automated={automated}"
        );
    }
}

#[tokio::test]
async fn updating_the_status_appends_history_and_stamps_the_terminal_time() {
    let service = IncidentResponseService::new();
    let incident = service.create_incident(manual(severity())).await;

    let triaged = service
        .update_status(incident.id, IncidentStatus::Triaging, "analyst")
        .await
        .expect("known incident must update");
    assert_eq!(triaged.status, IncidentStatus::Triaging);
    assert_eq!(triaged.timeline.len(), 2);
    assert_eq!(
        triaged.timeline[1].event_type,
        TimelineEventType::StatusChanged
    );
    assert_eq!(
        triaged.timeline[1].description,
        "Status changed from new to triaging"
    );
    assert_eq!(triaged.timeline[1].user.as_deref(), Some("analyst"));
    assert!(triaged.resolved_at.is_none());

    let resolved = service
        .update_status(incident.id, IncidentStatus::Resolved, "analyst")
        .await
        .expect("known incident must update");
    assert!(resolved.resolved_at.is_some(), "resolving must stamp resolved_at");

    let closed = service
        .update_status(incident.id, IncidentStatus::Closed, "analyst")
        .await
        .expect("known incident must update");
    assert!(closed.closed_at.is_some(), "closing must stamp closed_at");
    assert_eq!(closed.timeline.len(), 4);
}

#[tokio::test]
async fn updating_an_unknown_incident_reports_not_found() {
    let service = IncidentResponseService::new();
    let error = service
        .update_status(Uuid::new_v4(), IncidentStatus::Triaging, "analyst")
        .await
        .expect_err("unknown incidents cannot be updated");
    assert_eq!(error, "Incident not found");

    let error = service
        .update_severity(Uuid::new_v4(), IncidentSeverity::Low, "analyst")
        .await
        .expect_err("unknown incidents cannot change severity");
    assert_eq!(error, "Incident not found");

    assert!(service.get_incident(Uuid::new_v4()).await.is_none());
}

#[tokio::test]
async fn raising_the_severity_tightens_the_deadline_from_the_creation_time() {
    let service = IncidentResponseService::new();
    let incident = service.create_incident(manual(IncidentSeverity::Low)).await;
    assert_eq!(
        (incident.response_due_at - incident.created_at).num_minutes(),
        1440
    );

    let updated = service
        .update_severity(incident.id, IncidentSeverity::Critical, "analyst")
        .await
        .expect("known incident must update");

    assert_eq!(updated.severity, IncidentSeverity::Critical);
    assert_eq!(
        (updated.response_due_at - updated.created_at).num_minutes(),
        15,
        "the deadline is measured from creation, not from the update"
    );
    assert_eq!(
        updated.timeline.last().map(|e| e.event_type.clone()),
        Some(TimelineEventType::SeverityChanged)
    );
    assert_eq!(
        updated.timeline.last().map(|e| e.description.clone()),
        Some("Severity changed from low to critical".to_string())
    );
}

#[tokio::test]
async fn assignment_actions_and_evidence_are_recorded_in_the_timeline() {
    let service = IncidentResponseService::new();
    let incident = service.create_incident(manual(severity())).await;

    let assigned = service
        .assign_incident(incident.id, "responder", "manager")
        .await
        .expect("known incident must accept an assignee");
    assert_eq!(assigned.assigned_to.as_deref(), Some("responder"));
    assert_eq!(
        assigned.timeline.last().map(|e| e.event_type.clone()),
        Some(TimelineEventType::Assigned)
    );

    let action = service
        .add_action(NewAction {
            incident_id: incident.id,
            action_type: ActionType::IsolateSystem,
            description: "Isolated the storage node".to_string(),
            performed_by: "responder".to_string(),
            automated: true,
            success: true,
            result: Some("node offline".to_string()),
        })
        .await
        .expect("known incident must accept an action");
    assert!(action.success);
    assert!(action.automated);

    let evidence = service
        .add_evidence(NewEvidence {
            incident_id: incident.id,
            evidence_type: EvidenceType::LogFile,
            title: "access.log".to_string(),
            description: None,
            file_path: Some("/tmp/access.log".to_string()),
            url: None,
            collected_by: "responder".to_string(),
        })
        .await
        .expect("known incident must accept evidence");
    assert_eq!(evidence.chain_of_custody.len(), 1);
    assert_eq!(evidence.chain_of_custody[0].action, "Collected");

    let stored = service
        .get_incident(incident.id)
        .await
        .expect("incident is stored");
    assert_eq!(stored.actions_taken.len(), 1);
    assert_eq!(stored.evidence.len(), 1);
    assert_eq!(stored.timeline.len(), 4, "create + assign + action + evidence");
}

#[tokio::test]
async fn actions_on_unknown_incidents_are_rejected() {
    let service = IncidentResponseService::new();
    let unknown = Uuid::new_v4();

    assert!(service
        .assign_incident(unknown, "responder", "manager")
        .await
        .is_err());
    assert!(service
        .add_action(NewAction {
            incident_id: unknown,
            action_type: ActionType::Custom,
            description: "noop".to_string(),
            performed_by: "responder".to_string(),
            automated: false,
            success: false,
            result: None,
        })
        .await
        .is_err());
    assert!(service
        .add_evidence(NewEvidence {
            incident_id: unknown,
            evidence_type: EvidenceType::Other,
            title: "none".to_string(),
            description: None,
            file_path: None,
            url: None,
            collected_by: "responder".to_string(),
        })
        .await
        .is_err());
}


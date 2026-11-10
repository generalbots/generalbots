//! SLA, listing and statistics behaviour of the incident response service.

use super::new_incident::NewIncident;
use super::*;

fn severity() -> IncidentSeverity {
    IncidentSeverity::Critical
}

fn manual(severity: IncidentSeverity) -> NewIncident {
    NewIncident {
        title: "Database exposed".to_string(),
        description: "Public bucket with customer dumps".to_string(),
        severity,
        category: IncidentCategory::DataLeak,
        affected_systems: vec!["drive".to_string()],
        reported_by: "oncall@example.com".to_string(),
        source: IncidentSource::Manual,
        tags: vec!["data".to_string()],
        metadata: HashMap::new(),
    }
}

#[tokio::test]
async fn sla_breaches_flag_only_overdue_active_incidents() {
    let service = IncidentResponseService::new();
    let overdue = service.create_incident(manual(severity())).await;
    let fresh = service.create_incident(manual(severity())).await;
    let resolved = service.create_incident(manual(severity())).await;
    let _ = service
        .update_status(resolved.id, IncidentStatus::Resolved, "analyst")
        .await
        .expect("known incident must resolve");

    {
        let mut incidents = service.incidents.write().await;
        let past = Utc::now() - Duration::minutes(5);
        if let Some(incident) = incidents.get_mut(&overdue.id) {
            incident.response_due_at = past;
        }
        if let Some(incident) = incidents.get_mut(&resolved.id) {
            incident.response_due_at = past;
        }
    }

    service.check_sla_breaches().await;

    let overdue = service.get_incident(overdue.id).await.expect("stored");
    assert!(overdue.sla_breach, "an overdue active incident must breach");
    assert!(overdue.sla_breach_at.is_some());
    assert_eq!(
        overdue.timeline.last().map(|e| e.event_type.clone()),
        Some(TimelineEventType::SlaBreached)
    );

    let fresh = service.get_incident(fresh.id).await.expect("stored");
    assert!(!fresh.sla_breach, "an incident inside its window must not breach");

    let resolved = service.get_incident(resolved.id).await.expect("stored");
    assert!(
        !resolved.sla_breach,
        "a resolved incident must not be marked breached"
    );

    // A second sweep must not duplicate the breach entry.
    service.check_sla_breaches().await;
    let overdue = service.get_incident(overdue.id).await.expect("stored");
    let breaches = overdue
        .timeline
        .iter()
        .filter(|e| e.event_type == TimelineEventType::SlaBreached)
        .count();
    assert_eq!(breaches, 1, "the breach is recorded exactly once");
}

#[tokio::test]
async fn listing_filters_by_status_and_severity() {
    let service = IncidentResponseService::new();
    let critical = service.create_incident(manual(IncidentSeverity::Critical)).await;
    let low = service.create_incident(manual(IncidentSeverity::Low)).await;
    let _ = service
        .update_status(low.id, IncidentStatus::Triaging, "analyst")
        .await
        .expect("known incident must update");

    let all = service.list_incidents(None, None, 1, 10).await;
    assert_eq!(all.total, 2);
    assert_eq!(all.total_pages, 1);

    let only_critical = service
        .list_incidents(None, Some(IncidentSeverity::Critical), 1, 10)
        .await;
    assert_eq!(only_critical.total, 1);
    assert_eq!(only_critical.incidents[0].id, critical.id);
    assert_eq!(only_critical.incidents[0].severity, "critical");

    let only_triaging = service
        .list_incidents(Some(IncidentStatus::Triaging), None, 1, 10)
        .await;
    assert_eq!(only_triaging.total, 1);
    assert_eq!(only_triaging.incidents[0].id, low.id);
    assert_eq!(only_triaging.incidents[0].status, "triaging");

    let combined = service
        .list_incidents(Some(IncidentStatus::Triaging), Some(IncidentSeverity::Critical), 1, 10)
        .await;
    assert_eq!(combined.total, 0, "filters must combine with AND");
    assert!(combined.incidents.is_empty());
}

#[tokio::test]
async fn listing_orders_newest_first() {
    let service = IncidentResponseService::new();
    let older = service.create_incident(manual(severity())).await;
    let newer = service.create_incident(manual(severity())).await;

    let page = service.list_incidents(None, None, 1, 10).await;
    assert_eq!(page.incidents.len(), 2);
    assert!(
        page.incidents[0].created_at >= page.incidents[1].created_at,
        "the newest incident must come first"
    );
    assert_eq!(page.incidents[0].id, newer.id);
    assert_eq!(page.incidents[1].id, older.id);
}

#[tokio::test]
async fn listing_clamps_out_of_range_pages_instead_of_panicking() {
    let service = IncidentResponseService::new();
    service.create_incident(manual(severity())).await;
    service.create_incident(manual(severity())).await;
    service.create_incident(manual(severity())).await;

    let first = service.list_incidents(None, None, 1, 2).await;
    assert_eq!(first.incidents.len(), 2);
    assert_eq!(first.total_pages, 2);

    let second = service.list_incidents(None, None, 2, 2).await;
    assert_eq!(second.incidents.len(), 1);

    let beyond = service.list_incidents(None, None, 9, 2).await;
    assert!(beyond.incidents.is_empty(), "a page past the end is empty");
    assert_eq!(beyond.total, 3);

    let zero_page = service.list_incidents(None, None, 0, 2).await;
    assert_eq!(zero_page.page, 1, "page 0 is treated as the first page");
    assert_eq!(zero_page.incidents.len(), 2);

    let zero_per_page = service.list_incidents(None, None, 1, 0).await;
    assert_eq!(zero_per_page.per_page, 1, "per_page 0 must not divide by zero");
    assert_eq!(zero_per_page.total_pages, 3);
}

#[tokio::test]
async fn statistics_cover_every_dimension() {
    let service = IncidentResponseService::new();
    let critical = service.create_incident(manual(IncidentSeverity::Critical)).await;
    let low = service.create_incident(manual(IncidentSeverity::Low)).await;
    let _ = service
        .update_status(low.id, IncidentStatus::Resolved, "analyst")
        .await
        .expect("known incident must resolve");

    let stats = service.get_stats().await;
    assert_eq!(stats.total_incidents, 2);
    assert_eq!(stats.open_incidents, 1, "only the critical one is still active");
    assert_eq!(stats.resolved_incidents, 1);
    assert_eq!(stats.sla_breaches, 0);
    assert_eq!(stats.by_severity.get("critical"), Some(&1));
    assert_eq!(stats.by_severity.get("low"), Some(&1));
    assert_eq!(stats.by_status.get("new"), Some(&1));
    assert_eq!(stats.by_status.get("resolved"), Some(&1));
    assert_eq!(stats.by_category.get("data_leak"), Some(&2));

    let stored = service
        .get_incident(critical.id)
        .await
        .expect("incident is stored");
    assert!(stored.status.is_active());
}

#[tokio::test]
async fn mean_time_to_resolve_is_reported_in_hours() {
    let service = IncidentResponseService::new();
    let incident = service.create_incident(manual(severity())).await;
    {
        let mut incidents = service.incidents.write().await;
        if let Some(stored) = incidents.get_mut(&incident.id) {
            stored.created_at = Utc::now() - Duration::hours(3);
            stored.resolved_at = Some(Utc::now());
        }
    }

    let stats = service.get_stats().await;
    assert_eq!(stats.resolved_incidents, 1);
    assert!(
        (stats.mean_time_to_resolve_hours - 3.0).abs() < 0.1,
        "expected ~3 hours, got {}",
        stats.mean_time_to_resolve_hours
    );
}

#[tokio::test]
async fn statistics_of_an_empty_service_are_zeroed() {
    let service = IncidentResponseService::new();
    let stats = service.get_stats().await;
    assert_eq!(stats.total_incidents, 0);
    assert_eq!(stats.open_incidents, 0);
    assert_eq!(stats.resolved_incidents, 0);
    assert_eq!(stats.mean_time_to_resolve_hours, 0.0);
    assert!(stats.by_severity.is_empty());
}

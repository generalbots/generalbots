//! Offline tests for the in-memory audit service: filtering, statistics,
//! retention, exports and the compliance summary.

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

async fn service_with(events: Vec<AuditEvent>) -> AuditService {
    let service = AuditService::new(1_000);
    {
        let mut stored = service.events.write().await;
        stored.extend(events);
    }
    service
}

#[tokio::test]
async fn an_empty_filter_matches_every_event() {
    let user = Uuid::new_v4();
    let service = service_with(vec![
        event(
            AuditEventType::UserLogin,
            AuditSeverity::Info,
            Some(user),
            AuditOutcome::Success,
            Utc::now(),
        ),
        event(
            AuditEventType::SecurityAlert,
            AuditSeverity::Critical,
            None,
            AuditOutcome::Failure,
            Utc::now(),
        ),
    ])
    .await;

    let found = service
        .query_events(AuditFilter::default())
        .await
        .expect("query must succeed");
    assert_eq!(found.len(), 2);
}

#[tokio::test]
async fn filters_combine_with_and_semantics() {
    let user = Uuid::new_v4();
    let other = Uuid::new_v4();
    let now = Utc::now();
    let service = service_with(vec![
        event(
            AuditEventType::UserLogin,
            AuditSeverity::Info,
            Some(user),
            AuditOutcome::Success,
            now - Duration::hours(2),
        ),
        event(
            AuditEventType::UserLogin,
            AuditSeverity::Warning,
            Some(user),
            AuditOutcome::Failure,
            now - Duration::hours(1),
        ),
        event(
            AuditEventType::DataAccess,
            AuditSeverity::Critical,
            Some(other),
            AuditOutcome::Failure,
            now,
        ),
    ])
    .await;

    let logins = service
        .query_events(AuditFilter {
            event_types: Some(vec![AuditEventType::UserLogin]),
            ..AuditFilter::default()
        })
        .await
        .expect("query must succeed");
    assert_eq!(logins.len(), 2);

    let failures = service
        .query_events(AuditFilter {
            outcome: Some(AuditOutcome::Failure),
            ..AuditFilter::default()
        })
        .await
        .expect("query must succeed");
    assert_eq!(failures.len(), 2);

    let warnings_and_above = service
        .query_events(AuditFilter {
            severity: Some(AuditSeverity::Warning),
            ..AuditFilter::default()
        })
        .await
        .expect("query must succeed");
    assert_eq!(
        warnings_and_above.len(),
        2,
        "the severity filter is a floor, not an equality"
    );

    let by_user = service
        .query_events(AuditFilter {
            user_id: Some(user),
            outcome: Some(AuditOutcome::Failure),
            ..AuditFilter::default()
        })
        .await
        .expect("query must succeed");
    assert_eq!(by_user.len(), 1);
    assert_eq!(by_user[0].severity, AuditSeverity::Warning);

    let anonymous_never_matches_a_user_filter = service
        .query_events(AuditFilter {
            user_id: Some(Uuid::new_v4()),
            ..AuditFilter::default()
        })
        .await
        .expect("query must succeed");
    assert!(anonymous_never_matches_a_user_filter.is_empty());
}

#[tokio::test]
async fn time_bounds_are_inclusive() {
    let start = Utc::now() - Duration::days(2);
    let end = start + Duration::days(1);
    let service = service_with(vec![
        event(
            AuditEventType::DataAccess,
            AuditSeverity::Info,
            None,
            AuditOutcome::Success,
            start - Duration::seconds(1),
        ),
        event(
            AuditEventType::DataAccess,
            AuditSeverity::Info,
            None,
            AuditOutcome::Success,
            start,
        ),
        event(
            AuditEventType::DataAccess,
            AuditSeverity::Info,
            None,
            AuditOutcome::Success,
            end,
        ),
        event(
            AuditEventType::DataAccess,
            AuditSeverity::Info,
            None,
            AuditOutcome::Success,
            end + Duration::seconds(1),
        ),
    ])
    .await;

    let found = service
        .query_events(AuditFilter {
            since: Some(start),
            until: Some(end),
            ..AuditFilter::default()
        })
        .await
        .expect("query must succeed");
    assert_eq!(found.len(), 2, "both boundary events are included");
}

#[tokio::test]
async fn statistics_group_by_type_severity_outcome_and_user() {
    let first = Uuid::new_v4();
    let second = Uuid::new_v4();
    let now = Utc::now();
    let service = service_with(vec![
        event(
            AuditEventType::UserLogin,
            AuditSeverity::Info,
            Some(first),
            AuditOutcome::Success,
            now,
        ),
        event(
            AuditEventType::UserLogin,
            AuditSeverity::Info,
            Some(first),
            AuditOutcome::Failure,
            now,
        ),
        event(
            AuditEventType::SecurityAlert,
            AuditSeverity::Critical,
            Some(second),
            AuditOutcome::Failure,
            now,
        ),
        event(
            AuditEventType::DataAccess,
            AuditSeverity::Warning,
            None,
            AuditOutcome::Success,
            now,
        ),
    ])
    .await;

    let stats = service.get_statistics(Some(now - Duration::hours(1)), Some(now)).await;
    assert_eq!(stats.total_events, 4);
    assert_eq!(stats.events_by_type.get(&AuditEventType::UserLogin), Some(&2));
    assert_eq!(stats.unique_users, 2, "anonymous events do not count as users");
    assert_eq!(stats.events_by_severity.get(&AuditSeverity::Info), Some(&2));
    assert_eq!(stats.events_by_outcome.get(&AuditOutcome::Failure), Some(&2));
    assert_eq!(stats.events_by_outcome.get(&AuditOutcome::Success), Some(&2));
    assert_eq!(stats.time_range.0, now - Duration::hours(1));
    assert_eq!(stats.time_range.1, now);
}

#[tokio::test]
async fn statistics_ignore_events_outside_the_window() {
    let now = Utc::now();
    let service = service_with(vec![
        event(
            AuditEventType::DataAccess,
            AuditSeverity::Info,
            None,
            AuditOutcome::Success,
            now - Duration::days(40),
        ),
        event(
            AuditEventType::DataAccess,
            AuditSeverity::Info,
            None,
            AuditOutcome::Success,
            now - Duration::days(1),
        ),
    ])
    .await;

    let recent = service.get_statistics(None, None).await;
    assert_eq!(recent.total_events, 1, "the default window is 30 days");

    let wide = service
        .get_statistics(Some(now - Duration::days(60)), Some(now))
        .await;
    assert_eq!(wide.total_events, 2);
}

#[tokio::test]
async fn logging_evicts_the_oldest_events_past_the_cap() {
    let service = AuditService::new(2);
    for index in 0..3 {
        service
            .log_event(
                AuditEventType::DataAccess,
                AuditSeverity::Info,
                None,
                format!("action {index}"),
                AuditOutcome::Success,
                HashMap::new(),
            )
            .await
            .expect("logging must succeed");
    }

    let found = service
        .query_events(AuditFilter::default())
        .await
        .expect("query must succeed");
    assert_eq!(found.len(), 2, "the cap is enforced");
    assert_eq!(found[0].action, "action 1");
    assert_eq!(found[1].action, "action 2");
}

#[tokio::test]
async fn retention_removes_only_expired_events_covered_by_a_policy() {
    let now = Utc::now();
    let service = service_with(vec![
        // Covered by "Security Events" (365 days) and above the warning floor.
        event(
            AuditEventType::SecurityAlert,
            AuditSeverity::Critical,
            None,
            AuditOutcome::Success,
            now - Duration::days(400),
        ),
        // Same policy, but below the severity floor, so it is kept.
        event(
            AuditEventType::SecurityAlert,
            AuditSeverity::Info,
            None,
            AuditOutcome::Success,
            now - Duration::days(400),
        ),
        // Not covered by any policy.
        event(
            AuditEventType::DataModification,
            AuditSeverity::Info,
            None,
            AuditOutcome::Success,
            now - Duration::days(400),
        ),
        // Covered by "Access Logs" (90 days).
        event(
            AuditEventType::UserLogin,
            AuditSeverity::Info,
            None,
            AuditOutcome::Success,
            now - Duration::days(100),
        ),
        // Covered by "Access Logs" but recent.
        event(
            AuditEventType::UserLogin,
            AuditSeverity::Info,
            None,
            AuditOutcome::Success,
            now - Duration::days(10),
        ),
    ])
    .await;

    let removed = service
        .apply_retention_policies()
        .await
        .expect("retention must succeed");
    assert_eq!(removed, 2);

    let remaining = service
        .query_events(AuditFilter::default())
        .await
        .expect("query must succeed");
    assert_eq!(remaining.len(), 3);
    assert!(remaining
        .iter()
        .any(|e| e.event_type == AuditEventType::SecurityAlert && e.severity == AuditSeverity::Info));
    assert!(remaining
        .iter()
        .any(|e| e.event_type == AuditEventType::DataModification));
}


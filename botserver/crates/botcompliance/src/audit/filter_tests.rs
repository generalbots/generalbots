//! Offline unit tests for the audit filter (#2075).
//!
//! `AuditFilter::matches` is the only predicate behind the audit search API.
//! A wrong answer either hides a security event from an investigator or leaks
//! another user's trail. Every clause is ANDed, and an absent clause matches.

use super::*;
use chrono::TimeZone;

fn event(timestamp: DateTime<Utc>, severity: AuditSeverity, user: Option<Uuid>) -> AuditEvent {
    AuditEvent {
        id: Uuid::from_u128(1),
        timestamp,
        event_type: AuditEventType::DataAccess,
        severity,
        user_id: user,
        session_id: None,
        ip_address: None,
        resource_id: None,
        action: "read".to_string(),
        outcome: AuditOutcome::Success,
        details: HashMap::new(),
        metadata: serde_json::json!({}),
    }
}

fn empty_filter() -> AuditFilter {
    AuditFilter {
        event_types: None,
        severity: None,
        user_id: None,
        since: None,
        until: None,
        outcome: None,
    }
}

fn at(day: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 1, day, 12, 0, 0)
        .single()
        .expect("valid timestamp")
}

#[test]
fn an_empty_filter_matches_everything() {
    let filter = empty_filter();
    assert!(filter.matches(&event(at(1), AuditSeverity::Info, None)));
    assert!(filter.matches(&event(at(9), AuditSeverity::Critical, Some(Uuid::nil()))));
}

#[test]
fn event_types_are_an_allow_list() {
    let mut filter = empty_filter();
    filter.event_types = Some(vec![AuditEventType::UserLogin]);
    assert!(!filter.matches(&event(at(1), AuditSeverity::Info, None)));

    filter.event_types = Some(vec![AuditEventType::DataAccess]);
    assert!(filter.matches(&event(at(1), AuditSeverity::Info, None)));
}

#[test]
fn severity_is_a_floor_not_an_exact_match() {
    let mut filter = empty_filter();
    filter.severity = Some(AuditSeverity::Error);

    assert!(!filter.matches(&event(at(1), AuditSeverity::Info, None)));
    assert!(!filter.matches(&event(at(1), AuditSeverity::Warning, None)));
    assert!(filter.matches(&event(at(1), AuditSeverity::Error, None)));
    assert!(filter.matches(&event(at(1), AuditSeverity::Critical, None)));
}

#[test]
fn a_user_filter_excludes_anonymous_and_other_users() {
    let mut filter = empty_filter();
    let user = Uuid::from_u128(42);
    filter.user_id = Some(user);

    assert!(filter.matches(&event(at(1), AuditSeverity::Info, Some(user))));
    assert!(!filter.matches(&event(
        at(1),
        AuditSeverity::Info,
        Some(Uuid::from_u128(43))
    )));
    assert!(!filter.matches(&event(at(1), AuditSeverity::Info, None)));
}

#[test]
fn the_time_window_is_inclusive_on_both_ends() {
    let mut filter = empty_filter();
    filter.since = Some(at(5));
    filter.until = Some(at(7));

    assert!(!filter.matches(&event(at(4), AuditSeverity::Info, None)));
    assert!(filter.matches(&event(at(5), AuditSeverity::Info, None)));
    assert!(filter.matches(&event(at(6), AuditSeverity::Info, None)));
    assert!(filter.matches(&event(at(7), AuditSeverity::Info, None)));
    assert!(!filter.matches(&event(at(8), AuditSeverity::Info, None)));
}

#[test]
fn the_outcome_must_match_exactly() {
    let mut filter = empty_filter();
    filter.outcome = Some(AuditOutcome::Failure);

    let mut failure = event(at(1), AuditSeverity::Info, None);
    failure.outcome = AuditOutcome::Failure;
    assert!(filter.matches(&failure));
    assert!(!filter.matches(&event(at(1), AuditSeverity::Info, None)));
}

#[test]
fn every_clause_must_hold_together() {
    let user = Uuid::from_u128(42);
    let mut filter = empty_filter();
    filter.event_types = Some(vec![AuditEventType::DataAccess]);
    filter.severity = Some(AuditSeverity::Warning);
    filter.user_id = Some(user);
    filter.since = Some(at(5));

    // Matches on every clause.
    assert!(filter.matches(&event(at(6), AuditSeverity::Error, Some(user))));
    // Fails exactly one clause at a time.
    assert!(!filter.matches(&event(at(4), AuditSeverity::Error, Some(user))));
    assert!(!filter.matches(&event(
        at(6),
        AuditSeverity::Error,
        Some(Uuid::from_u128(1))
    )));
    assert!(!filter.matches(&event(at(6), AuditSeverity::Info, Some(user))));
}

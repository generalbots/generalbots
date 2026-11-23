//! Tenant-isolation tests for the process-wide Office event runtime (#2825).
//!
//! The dead-letter list and replay surface is scoped by the caller's verified
//! org/branch. These tests exercise both the pure ownership predicate and the
//! two-tenant runtime behaviour without a server or a database.

use super::{scope_owns, EventTenantScope, OfficeEventsRuntime, NIL_SCOPE_ID};
use crate::office::event::DomainEvent;
use crate::office::taxonomy::OfficeEventType;
use crate::office::ReplayError;

fn tenant_scope(name: &str) -> EventTenantScope {
    EventTenantScope::new(
        format!("{name}-0000-0000-0000-000000000001"),
        format!("{name}-1111-1111-1111-111111111111"),
    )
    .expect("test scope has valid ids")
}

#[test]
fn scope_ownership_is_exact_and_fails_closed() {
    let owner = tenant_scope("tenant-a");
    let same = tenant_scope("tenant-a");
    let other_org = tenant_scope("tenant-b");
    let other_branch = EventTenantScope::new(
        "tenant-a-0000-0000-0000-000000000001",
        "2222-2222-2222-2222-222222222222",
    )
    .expect("test scope has valid ids");

    assert!(scope_owns(&owner, &same), "identical verified scopes match");
    assert!(!scope_owns(&owner, &other_org), "a foreign org is refused");
    assert!(
        !scope_owns(&owner, &other_branch),
        "a foreign branch is refused"
    );
    assert!(
        EventTenantScope::new(NIL_SCOPE_ID, "branch-1").is_none(),
        "a nil org is never a scope"
    );
    assert!(
        EventTenantScope::new("org-1", "").is_none(),
        "a missing branch is never a scope"
    );
}

#[test]
fn two_tenants_cannot_list_or_replay_each_others_dead_letters() {
    let runtime = OfficeEventsRuntime::new();
    let tenant_a = tenant_scope("tenant-a");
    let tenant_b = tenant_scope("tenant-b");

    // Use a type distinct from the seeded demo event (SheetsExported) so only
    // this tenant's own subscription fires — keeping the count assertion stable.
    let event_a = DomainEvent::new(
        "evt-tenant-a",
        OfficeEventType::DocCreated,
        "ws-a",
        "doc-a",
        "usr-a",
    );
    let key_a = event_a.idempotency_key(&format!("sub-{}", event_a.event_id));
    let report = runtime.publish_scoped(
        &tenant_a,
        &event_a,
        Some("https://events.example.test/hook"),
    );
    assert_eq!(report.dead_lettered, 1, "tenant A dead letters its event");

    // Tenant B has no entry for A's key: neither an unknown id nor A's id is
    // visible in its list or replayable through its scope.
    assert!(
        runtime
            .dead_letters_scoped(&tenant_b)
            .iter()
            .all(|letter| letter.idempotency_key != key_a),
        "tenant B must not list tenant A's dead letter"
    );
    assert!(matches!(
        runtime.replay_scoped(&tenant_b, &key_a),
        Err(ReplayError::NotFound)
    ));

    // Give tenant B its own dead letter; cross-tenant reads must still be
    // empty and a foreign replay key must never reach A's subscription URL.
    let event_b = DomainEvent::new(
        "evt-tenant-b",
        OfficeEventType::DocCreated,
        "ws-b",
        "doc-b",
        "usr-b",
    );
    let key_b = event_b.idempotency_key(&format!("sub-{}", event_b.event_id));
    assert_eq!(
        runtime
            .publish_scoped(
                &tenant_b,
                &event_b,
                Some("https://events.example.test/hook"),
            )
            .dead_lettered,
        1
    );
    assert!(
        runtime
            .dead_letters_scoped(&tenant_b)
            .iter()
            .all(|letter| letter.idempotency_key != key_a),
        "a populated foreign tenant still cannot list A's letter"
    );
    assert!(
        runtime
            .dead_letters_scoped(&tenant_a)
            .iter()
            .all(|letter| letter.idempotency_key != key_b),
        "tenant A cannot list tenant B's letter"
    );
    assert!(matches!(
        runtime.replay_scoped(&tenant_a, &key_b),
        Err(ReplayError::NotFound)
    ));
    assert!(matches!(
        runtime.replay_scoped(&tenant_a, "missing:key"),
        Err(ReplayError::NotFound)
    ));

    let replay = runtime.replay_scoped(&tenant_a, &key_a);
    assert!(replay.is_ok(), "the owner can replay its own letter");
    assert_eq!(
        replay.map_or(0, |result| result.receipt.status_code),
        200,
        "the owner's replay delivers to its own stored URL"
    );
    assert!(
        matches!(
            runtime.replay_scoped(&tenant_b, &key_a),
            Err(ReplayError::NotFound)
        ),
        "a foreign tenant still cannot replay after the owner drained its DLQ"
    );
}

#[test]
fn an_invalid_scope_has_no_pipeline_at_all() {
    let runtime = OfficeEventsRuntime::new();
    let invalid = EventTenantScope {
        org_id: NIL_SCOPE_ID.to_string(),
        branch_id: NIL_SCOPE_ID.to_string(),
    };
    assert!(!invalid.is_valid());
    let event = DomainEvent::new(
        "evt-invalid-scope",
        OfficeEventType::DocExported,
        "ws",
        "doc",
        "usr",
    );
    let report = runtime.publish_scoped(&invalid, &event, Some("https://events.example.test/hook"));
    assert_eq!(
        report.webhooks_attempted + report.native_enqueued + report.dead_lettered,
        0,
        "a nil scope performs no delivery"
    );
}

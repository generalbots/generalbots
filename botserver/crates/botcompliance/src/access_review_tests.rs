//! Offline tests for the access review ledger: permission checks, expiry,
//! review processing, violation filtering and the compliance score.

use super::*;

fn service() -> AccessReviewService {
    AccessReviewService::new()
}

fn grant(
    service: &mut AccessReviewService,
    user_id: Uuid,
    resource_id: &str,
    level: AccessLevel,
    expires_in: Option<Duration>,
) -> AccessPermission {
    service
        .grant_permission(PermissionGrant {
            user_id,
            resource_id: resource_id.to_string(),
            resource_type: ResourceType::File,
            access_level: level,
            granted_by: Uuid::new_v4(),
            justification: "job function".to_string(),
            expires_in,
        })
        .expect("granting a permission must succeed")
}

#[test]
fn access_levels_satisfy_themselves_and_everything_below() {
    let cases = [
        (AccessLevel::Read, AccessLevel::Read, true),
        (AccessLevel::Write, AccessLevel::Read, true),
        (AccessLevel::Admin, AccessLevel::Read, true),
        (AccessLevel::Owner, AccessLevel::Read, true),
        (AccessLevel::Read, AccessLevel::Write, false),
        (AccessLevel::Write, AccessLevel::Write, true),
        (AccessLevel::Admin, AccessLevel::Write, true),
        (AccessLevel::Owner, AccessLevel::Write, true),
        (AccessLevel::Read, AccessLevel::Admin, false),
        (AccessLevel::Write, AccessLevel::Admin, false),
        (AccessLevel::Admin, AccessLevel::Admin, true),
        (AccessLevel::Owner, AccessLevel::Admin, true),
        (AccessLevel::Read, AccessLevel::Owner, false),
        (AccessLevel::Admin, AccessLevel::Owner, false),
        (AccessLevel::Owner, AccessLevel::Owner, true),
    ];

    for (granted, required, expected) in cases {
        assert_eq!(
            AccessReviewService::has_sufficient_access(&granted, &required),
            expected,
            "{granted:?} vs required {required:?}"
        );
    }
}

#[test]
fn granted_access_is_accepted_and_denied_access_is_recorded() {
    let mut service = service();
    let user = Uuid::new_v4();
    grant(&mut service, user, "reports/q1.csv", AccessLevel::Write, None);

    assert!(service
        .check_access(user, "reports/q1.csv", AccessLevel::Read)
        .expect("check must succeed"));
    assert!(service
        .check_access(user, "reports/q1.csv", AccessLevel::Write)
        .expect("check must succeed"));
    assert!(service.violations.is_empty());

    assert!(!service
        .check_access(user, "reports/q1.csv", AccessLevel::Admin)
        .expect("check must succeed"));
    assert_eq!(service.violations.len(), 1);
    let violation = &service.violations[0];
    assert_eq!(violation.user_id, user);
    assert_eq!(violation.resource_id, "reports/q1.csv");
    assert_eq!(violation.severity, ViolationSeverity::Medium);
    assert_eq!(violation.attempted_action, "Admin access");

    assert!(!service
        .check_access(user, "other/file.csv", AccessLevel::Read)
        .expect("check must succeed"));
    assert_eq!(
        service.violations.len(),
        2,
        "unknown resources are denied and recorded"
    );
}

#[test]
fn expired_and_revoked_permissions_are_denied() {
    let mut service = service();
    let user = Uuid::new_v4();
    let expired = grant(
        &mut service,
        user,
        "legacy/report.csv",
        AccessLevel::Owner,
        Some(Duration::days(-1)),
    );
    let revoked = grant(&mut service, user, "hr/salaries.csv", AccessLevel::Read, None);

    service
        .revoke_permission(revoked.id, Uuid::new_v4())
        .expect("revoking a known permission must succeed");

    assert!(!service
        .check_access(user, "legacy/report.csv", AccessLevel::Read)
        .expect("check must succeed"));
    assert!(!service
        .check_access(user, "hr/salaries.csv", AccessLevel::Read)
        .expect("check must succeed"));

    assert_eq!(service.get_expired_permissions().len(), 1);
    assert_eq!(service.get_expired_permissions()[0].id, expired.id);
    assert!(
        !service
            .get_user_permissions(user)
            .iter()
            .any(|permission| permission.id == revoked.id),
        "revoked grants must not be listed as active permissions"
    );
}

#[test]
fn revoking_an_unknown_permission_fails() {
    let mut service = service();
    let error = service
        .revoke_permission(Uuid::new_v4(), Uuid::new_v4())
        .expect_err("unknown permissions cannot be revoked");
    assert!(error.to_string().contains("Permission not found"));
}

#[test]
fn review_requests_snapshot_the_user_permissions() {
    let mut service = service();
    let user = Uuid::new_v4();
    let reviewer = Uuid::new_v4();
    grant(&mut service, user, "finance/ledger.csv", AccessLevel::Read, None);

    let review = service
        .create_review_request(user, reviewer, 7)
        .expect("review creation must succeed");
    assert_eq!(review.status, ReviewStatus::Pending);
    assert_eq!(review.reviewer_id, reviewer);
    assert_eq!(review.permissions.len(), 1);
    assert_eq!((review.due_date - review.requested_at).num_days(), 7);
    assert!(review.comments.is_none());

    let pending = service.get_pending_reviews(Some(reviewer));
    assert_eq!(pending.len(), 1);
    assert!(service.get_pending_reviews(Some(Uuid::new_v4())).is_empty());
    assert_eq!(service.get_pending_reviews(None).len(), 1);
}

#[test]
fn processing_a_review_revokes_and_modifies_permissions() {
    let mut service = service();
    let user = Uuid::new_v4();
    let reviewer = Uuid::new_v4();
    let keep = grant(&mut service, user, "keep.csv", AccessLevel::Read, None);
    let drop = grant(&mut service, user, "drop.csv", AccessLevel::Read, None);

    let review = service
        .create_review_request(user, reviewer, 7)
        .expect("review creation must succeed");

    let result = service
        .process_review(
            review.id,
            vec![keep.id],
            vec![drop.id],
            vec![(keep.id, AccessLevel::Admin)],
            "quarterly review".to_string(),
        )
        .expect("pending reviews can be processed");

    assert_eq!(result.approved_permissions, vec![keep.id]);
    assert_eq!(result.revoked_permissions, vec![drop.id]);
    assert_eq!(result.comments, "quarterly review");

    let active = service.get_user_permissions(user);
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].id, keep.id);
    assert_eq!(active[0].access_level, AccessLevel::Admin);

    let stored = service.reviews.get(&review.id).expect("review is stored");
    assert_eq!(stored.status, ReviewStatus::Approved);
    assert_eq!(stored.comments.as_deref(), Some("quarterly review"));
    assert!(service.get_pending_reviews(None).is_empty());
}

#[test]
fn completed_or_unknown_reviews_cannot_be_processed() {
    let mut service = service();
    let user = Uuid::new_v4();
    let review = service
        .create_review_request(user, Uuid::new_v4(), 3)
        .expect("review creation must succeed");

    service
        .process_review(review.id, vec![], vec![], vec![], "done".to_string())
        .expect("first processing succeeds");

    let error = service
        .process_review(review.id, vec![], vec![], vec![], "again".to_string())
        .expect_err("completed reviews cannot be processed twice");
    assert!(error.to_string().contains("Review already completed"));

    let error = service
        .process_review(Uuid::new_v4(), vec![], vec![], vec![], "missing".to_string())
        .expect_err("unknown reviews cannot be processed");
    assert!(error.to_string().contains("Review not found"));
}

#[test]
fn violations_are_filtered_by_user_severity_and_time() {
    let mut service = service();
    let user = Uuid::new_v4();
    let other = Uuid::new_v4();
    service.violations.push(AccessViolation {
        id: Uuid::new_v4(),
        user_id: user,
        resource_id: "a".to_string(),
        attempted_action: "Write access".to_string(),
        denied_reason: "Insufficient permissions".to_string(),
        occurred_at: Utc::now(),
        severity: ViolationSeverity::Critical,
    });
    service.violations.push(AccessViolation {
        id: Uuid::new_v4(),
        user_id: other,
        resource_id: "b".to_string(),
        attempted_action: "Admin access".to_string(),
        denied_reason: "Insufficient permissions".to_string(),
        occurred_at: Utc::now() - Duration::days(40),
        severity: ViolationSeverity::Low,
    });

    assert_eq!(service.get_violations(None, None, None).len(), 2);
    assert_eq!(service.get_violations(Some(user), None, None).len(), 1);
    assert_eq!(
        service
            .get_violations(None, Some(ViolationSeverity::Low), None)
            .len(),
        1
    );
    assert_eq!(
        service
            .get_violations(None, None, Some(Utc::now() - Duration::days(30)))
            .len(),
        1,
        "the recent window drops the 40-day-old violation"
    );
}

#[test]
fn the_compliance_report_counts_ledger_state() {
    let mut service = service();
    let user = Uuid::new_v4();
    let reviewer = Uuid::new_v4();

    let _expired = grant(
        &mut service,
        user,
        "expired.csv",
        AccessLevel::Read,
        Some(Duration::days(-1)),
    );
    let active = grant(&mut service, user, "active.csv", AccessLevel::Read, None);
    let _ = service
        .check_access(user, "forbidden.csv", AccessLevel::Read)
        .expect("check must succeed");
    let _review = service
        .create_review_request(user, reviewer, -1)
        .expect("review creation must succeed");

    let report = service.generate_compliance_report();
    assert_eq!(report.total_permissions, 2);
    assert_eq!(report.active_permissions, 2);
    assert_eq!(report.expired_permissions, 1);
    assert_eq!(report.pending_reviews, 1);
    assert_eq!(report.violations_last_30_days, 1);
    assert_eq!(report.critical_violations, 0);
    // 100 - 2 (expired) - 5 (overdue review) - 3 (medium violation)
    assert!((report.compliance_score - 90.0).abs() < f64::EPSILON);

    service
        .revoke_permission(active.id, reviewer)
        .expect("revoking must succeed");
    let after = service.generate_compliance_report();
    assert_eq!(after.active_permissions, 1);
    assert_eq!(after.total_permissions, 2);
}

#[test]
fn the_compliance_score_is_clamped_to_zero() {
    let mut service = service();
    let user = Uuid::new_v4();
    for _ in 0..11 {
        service.violations.push(AccessViolation {
            id: Uuid::new_v4(),
            user_id: user,
            resource_id: "x".to_string(),
            attempted_action: "Admin access".to_string(),
            denied_reason: "Insufficient permissions".to_string(),
            occurred_at: Utc::now(),
            severity: ViolationSeverity::Critical,
        });
    }

    let report = service.generate_compliance_report();
    assert_eq!(report.critical_violations, 11);
    assert_eq!(report.compliance_score, 0.0, "the score never goes negative");
}

#[test]
fn a_clean_ledger_scores_one_hundred() {
    let mut service = service();
    let user = Uuid::new_v4();
    grant(&mut service, user, "active.csv", AccessLevel::Read, None);

    let report = service.generate_compliance_report();
    assert_eq!(report.compliance_score, 100.0);
    assert_eq!(report.violations_last_30_days, 0);
    assert_eq!(report.expired_permissions, 0);
}

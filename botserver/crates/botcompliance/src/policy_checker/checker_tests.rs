//! Adversarial tests for the in-memory compliance policy checker (#2066).
//!
//! The checker enforces the password-strength and session-timeout policies.
//! These tests cover the boundary values and the accounting used by the
//! compliance report, plus the rule-evaluation limitation of custom policies.

use super::*;
use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;

fn rule(name: &str, action: PolicyAction) -> PolicyRule {
    PolicyRule {
        id: Uuid::new_v4(),
        name: name.to_string(),
        condition: "always".to_string(),
        action,
        parameters: HashMap::new(),
    }
}

fn policy(
    name: &str,
    policy_type: PolicyType,
    status: PolicyStatus,
    severity: PolicySeverity,
    rules: Vec<PolicyRule>,
) -> SecurityPolicy {
    let now = Utc::now();
    SecurityPolicy {
        id: Uuid::new_v4(),
        name: name.to_string(),
        description: String::new(),
        policy_type,
        status,
        severity,
        rules,
        created_at: now,
        updated_at: now,
        effective_date: now,
        expiry_date: None,
        tags: Vec::new(),
    }
}

fn context() -> PolicyContext {
    PolicyContext {
        user_id: None,
        resource: "report".to_string(),
        action: "read".to_string(),
        parameters: HashMap::new(),
    }
}

#[test]
fn the_default_checker_starts_with_two_active_policies_and_a_clean_report() {
    let checker = PolicyChecker::new();
    let report = checker.get_compliance_report();
    assert_eq!(report.total_policies, 2);
    assert_eq!(report.active_policies, 2);
    assert_eq!(report.total_violations, 0);
    assert_eq!(report.unresolved_violations, 0);
    assert_eq!(report.compliance_rate, 100.0);
}

#[test]
fn a_strong_password_passes_the_default_policy() {
    let mut checker = PolicyChecker::new();
    let result = checker.check_password_policy("Abcdefgh123!");
    assert!(result.passed, "{:?}", result.violations);
    assert!(result.violations.is_empty());
    assert!(result.warnings.is_empty());
}

#[test]
fn a_short_password_reports_the_length_rule() {
    let mut checker = PolicyChecker::new();
    let result = checker.check_password_policy("Ab1!");
    assert!(!result.passed);
    assert!(result
        .violations
        .iter()
        .any(|v| v.violation_details.contains("less than required 12")));
}

#[test]
fn a_password_meeting_the_length_but_not_the_complexity_fails() {
    let mut checker = PolicyChecker::new();
    for password in ["abcdefghijkl", "ABCDEFGHIJKL", "Abcdefghijkl", "Abcdefgh1234"] {
        let result = checker.check_password_policy(password);
        assert!(!result.passed, "{password} must fail the complexity rule");
        assert!(result
            .violations
            .iter()
            .any(|v| v.violation_details.contains("complexity")));
    }
}

#[test]
fn a_multibyte_password_cannot_shortcut_the_length_rule() {
    // Regression: `len()` is bytes, so 9 characters with emoji measured 18 and
    // passed the 12-character minimum.
    let mut checker = PolicyChecker::new();
    let short_in_chars = "Aa1!\u{1F600}\u{1F600}\u{1F600}\u{1F600}\u{1F600}";
    assert_eq!(short_in_chars.chars().count(), 9);
    assert!(short_in_chars.len() >= 12);

    let result = checker.check_password_policy(short_in_chars);
    assert!(!result.passed);
    assert!(result
        .violations
        .iter()
        .any(|v| v.violation_details.contains("less than required 12")));
}

#[test]
fn a_password_containing_the_word_password_warns_but_passes() {
    let mut checker = PolicyChecker::new();
    let result = checker.check_password_policy("MyPassword123!");
    assert!(result.passed);
    assert!(result
        .warnings
        .iter()
        .any(|w| w.contains("contains the word 'password'")));
}

#[test]
fn the_session_timeout_boundary_is_inclusive() {
    let mut checker = PolicyChecker::new();
    assert!(checker.check_session_policy(0).passed);
    assert!(checker.check_session_policy(28_800).passed);

    let over = checker.check_session_policy(28_801);
    assert!(!over.passed);
    assert_eq!(over.violations.len(), 1);
    assert_eq!(over.violations[0].severity, PolicySeverity::Medium);
}

#[test]
fn an_inactive_policy_is_not_evaluated() {
    let mut checker = PolicyChecker::new();
    let inactive = policy(
        "Draft password policy",
        PolicyType::PasswordStrength,
        PolicyStatus::Draft,
        PolicySeverity::Low,
        vec![rule("deny all", PolicyAction::Deny)],
    );
    checker.add_policy(inactive).expect("add");

    let results = checker.check_all_policies(&context());
    assert_eq!(results.len(), 2, "only the two active defaults run");
}

#[test]
fn duplicate_and_unknown_policy_ids_are_rejected() {
    let mut checker = PolicyChecker::new();
    let custom = policy(
        "Retention",
        PolicyType::DataRetention,
        PolicyStatus::Active,
        PolicySeverity::High,
        vec![rule("keep", PolicyAction::Enforce)],
    );
    checker.add_policy(custom.clone()).expect("first add");
    assert!(checker.add_policy(custom).is_err());

    let unknown = policy(
        "Missing",
        PolicyType::BackupFrequency,
        PolicyStatus::Active,
        PolicySeverity::Low,
        Vec::new(),
    );
    assert!(checker.update_policy(Uuid::new_v4(), unknown).is_err());
    assert!(checker.check_policy(Uuid::new_v4(), &context()).is_err());
}

#[test]
fn updating_a_policy_keeps_the_registry_size() {
    let mut checker = PolicyChecker::new();
    let custom = policy(
        "Retention",
        PolicyType::DataRetention,
        PolicyStatus::Active,
        PolicySeverity::High,
        vec![rule("keep", PolicyAction::Enforce)],
    );
    let id = custom.id;
    checker.add_policy(custom).expect("add");

    let mut renamed = policy(
        "Retention v2",
        PolicyType::DataRetention,
        PolicyStatus::Active,
        PolicySeverity::Critical,
        vec![rule("keep", PolicyAction::Enforce)],
    );
    renamed.id = id;
    checker.update_policy(id, renamed).expect("update");

    assert_eq!(checker.get_compliance_report().total_policies, 3);
}

#[test]
fn violations_are_counted_and_can_be_resolved() {
    let mut checker = PolicyChecker::new();
    checker.check_password_policy("short");
    let report = checker.get_compliance_report();
    assert_eq!(report.total_violations, 2);
    assert_eq!(report.unresolved_violations, 2);

    let violation_id = checker.get_violations(true)[0].id;
    checker.resolve_violation(violation_id).expect("resolve");
    assert_eq!(checker.get_violations(true).len(), 1);
    assert_eq!(checker.get_compliance_report().unresolved_violations, 1);
    assert!(checker.resolve_violation(violation_id).is_ok());
    assert!(checker.resolve_violation(Uuid::new_v4()).is_err());
}

#[test]
fn the_compliance_rate_is_the_share_of_passed_checks() {
    let mut checker = PolicyChecker::new();
    checker.check_password_policy("Abcdefgh123!");
    checker.check_session_policy(28_801);
    let report = checker.get_compliance_report();
    assert_eq!(report.recent_checks, 2);
    assert!((report.compliance_rate - 50.0).abs() < f64::EPSILON);
}

#[test]
fn a_custom_deny_rule_produces_a_critical_violation() {
    let mut checker = PolicyChecker::new();
    let deny = policy(
        "No exports",
        PolicyType::DataRetention,
        PolicyStatus::Active,
        PolicySeverity::Critical,
        vec![rule("block export", PolicyAction::Deny)],
    );
    let id = deny.id;
    checker.add_policy(deny).expect("add");

    let result = checker.check_policy(id, &context()).expect("check");
    assert!(!result.passed);
    assert_eq!(result.violations.len(), 1);
    assert_eq!(checker.get_compliance_report().critical_violations, 1);
}

#[test]
fn a_non_deny_rule_is_not_evaluated_known_limitation() {
    // KNOWN LIMITATION (#2066): rule conditions are declarative strings and
    // `evaluate_rule` only treats `Deny` as a failure, so an `Enforce` rule
    // always passes. The password/session checks implement their rules
    // directly, which is why the defaults still work. Documented so a future
    // condition evaluator is a deliberate change.
    let mut checker = PolicyChecker::new();
    let enforce = policy(
        "Always enforce",
        PolicyType::AccessControl,
        PolicyStatus::Active,
        PolicySeverity::High,
        vec![rule("never satisfied", PolicyAction::Enforce)],
    );
    let id = enforce.id;
    checker.add_policy(enforce).expect("add");
    assert!(checker.check_policy(id, &context()).expect("check").passed);
}

//! Table tests for the compliance SOP middleware classifiers (#2066).
//!
//! `categorize_request`, `determine_severity` and `determine_outcome` feed the
//! audit trail (SOC 2 CC7). A silently inverted classification would hide
//! authentication and configuration changes from the evidence.

use super::categorize_request::{
    categorize_request as classify, determine_outcome, determine_severity, sanitize_path,
};
use super::*;
use axum::http::{Method, StatusCode};

fn method(value: &str) -> Method {
    Method::from_bytes(value.as_bytes()).expect("method")
}

fn category(value: &str, path: &str) -> SopCategory {
    classify(&method(value), path)
}

fn severity(value: &str, path: &str, status: u16) -> SopSeverity {
    determine_severity(
        &method(value),
        path,
        StatusCode::from_u16(status).expect("status"),
    )
}

#[test]
fn the_authentication_surface_is_recognised_in_every_form() {
    for path in ["/api/auth/login", "/api/login", "/api/logout", "/api/v1/auth/token"] {
        assert_eq!(category("POST", path), SopCategory::Authentication, "{path}");
    }
    assert_eq!(category("GET", "/API/AUTH/LOGIN"), SopCategory::Authentication);
}

#[test]
fn management_and_billing_paths_map_to_their_category() {
    let cases = [
        ("/api/users", SopCategory::UserManagement),
        ("/api/members", SopCategory::UserManagement),
        ("/api/roles", SopCategory::RoleManagement),
        ("/api/permissions", SopCategory::RoleManagement),
        ("/api/billing", SopCategory::BillingOperation),
        ("/api/subscription", SopCategory::BillingOperation),
        ("/api/invoice", SopCategory::BillingOperation),
        ("/api/compliance/report", SopCategory::ComplianceAction),
        ("/api/audit/logs", SopCategory::ComplianceAction),
        ("/api/settings", SopCategory::ConfigurationChange),
        ("/api/config", SopCategory::ConfigurationChange),
        ("/api/files", SopCategory::FileAccess),
        ("/api/documents", SopCategory::FileAccess),
        ("/api/drive/upload", SopCategory::FileAccess),
    ];
    for (path, expected) in cases {
        assert_eq!(category("GET", path), expected, "{path}");
    }
}

#[test]
fn the_path_rules_take_precedence_over_the_method() {
    assert_eq!(category("DELETE", "/api/auth/users"), SopCategory::Authentication);
    assert_eq!(category("POST", "/api/billing/files"), SopCategory::BillingOperation);
}

#[test]
fn the_method_is_the_fallback_for_unknown_paths() {
    let cases = [
        ("GET", SopCategory::DataAccess),
        ("HEAD", SopCategory::DataAccess),
        ("POST", SopCategory::DataModification),
        ("PUT", SopCategory::DataModification),
        ("PATCH", SopCategory::DataModification),
        ("DELETE", SopCategory::DataDeletion),
        ("OPTIONS", SopCategory::ApiAccess),
        ("TRACE", SopCategory::ApiAccess),
    ];
    for (verb, expected) in cases {
        assert_eq!(category(verb, "/api/things"), expected, "{verb}");
    }
}

#[test]
fn server_errors_are_always_high_severity() {
    assert_eq!(severity("GET", "/api/things", 500), SopSeverity::High);
    assert_eq!(severity("POST", "/api/auth/login", 503), SopSeverity::High);
}

#[test]
fn denied_requests_are_medium_severity() {
    assert_eq!(severity("GET", "/api/things", 401), SopSeverity::Medium);
    assert_eq!(severity("DELETE", "/api/things", 403), SopSeverity::Medium);
}

#[test]
fn security_and_admin_paths_are_medium_severity() {
    assert_eq!(severity("GET", "/api/admin/users", 200), SopSeverity::Medium);
    assert_eq!(severity("GET", "/api/security/scan", 200), SopSeverity::Medium);
}

#[test]
fn destructive_and_configuration_mutations_are_medium_severity() {
    assert_eq!(severity("DELETE", "/api/things/1", 200), SopSeverity::Medium);
    assert_eq!(severity("POST", "/api/config", 200), SopSeverity::Medium);
    assert_eq!(severity("PATCH", "/api/settings", 200), SopSeverity::Medium);
    assert_eq!(
        severity("GET", "/api/config", 200),
        SopSeverity::Info,
        "reading configuration is not a change"
    );
    assert_eq!(severity("GET", "/api/things", 200), SopSeverity::Info);
}

#[test]
fn the_outcome_maps_every_status_class() {
    let cases = [
        (200, SopOutcome::Success),
        (201, SopOutcome::Success),
        (204, SopOutcome::Success),
        (401, SopOutcome::Denied),
        (403, SopOutcome::Denied),
        (500, SopOutcome::Error),
        (502, SopOutcome::Error),
        (408, SopOutcome::Timeout),
        (504, SopOutcome::Timeout),
        (400, SopOutcome::Failure),
        (404, SopOutcome::Failure),
        (302, SopOutcome::Failure),
    ];
    for (status, expected) in cases {
        let code = StatusCode::from_u16(status).expect("status");
        assert_eq!(determine_outcome(code), expected, "{status}");
    }
}

#[test]
fn path_sanitisation_removes_identifiers_and_normalises_separators() {
    assert_eq!(sanitize_path("test/operation"), "test_operation");
    assert_eq!(sanitize_path("/api/users/123"), "api_users_{id}");
    assert_eq!(
        sanitize_path("/api/items/123e4567-e89b-12d3-a456-426614174000"),
        "api_items_:id"
    );
    assert_eq!(sanitize_path(""), "");
    assert_eq!(sanitize_path("/"), "");
    assert_eq!(sanitize_path("API/Users"), "api_users");
}

#[test]
fn path_sanitisation_is_case_sensitive_for_identifiers() {
    // The UUID rule matches lowercase hex only; an uppercase UUID is still
    // normalised by the numeric rule and the lowercase pass.
    let upper = "/api/items/123E4567-E89B-12D3-A456-426614174000";
    let sanitized = sanitize_path(upper);
    assert!(sanitized.starts_with("api_items_"));
    assert_eq!(sanitized, sanitized.to_lowercase());
    assert!(!sanitized.contains("123E4567-E89B"));
}

//! #2452 — deterministic request-DTO tests for the collab routes.
//!
//! `dto.rs` declares `mod dto_tests;`, so Rust resolves this module to
//! `collab_routes/dto/dto_tests.rs` (a child of the `dto` file module). The
//! original tests were placed as a sibling `collab_routes/dto_tests.rs`, which
//! the compiler never loaded (E0583 in CI). They are restored here and extended
//! with the pure validation helpers the handlers rely on. No database, network
//! or browser is involved.

use super::{LinkBody, sanitize_resource, valid_editable_role, valid_grantee_type};

fn parse_link_body(value: serde_json::Value) -> LinkBody {
    serde_json::from_value(value).unwrap_or_else(|_| LinkBody {
        resource_type: String::new(),
        resource_id: String::new(),
        role: String::new(),
        expires_in_hours: None,
        expires_at: None,
        allow_download: None,
    })
}

#[test]
fn link_body_without_a_download_policy_keeps_legacy_behaviour() {
    let body = parse_link_body(serde_json::json!({
        "resource_type": "docs",
        "resource_id": "doc-1",
        "role": "commenter"
    }));
    assert_eq!(body.allow_download, None);
    assert!(body.downloads_allowed());
}

#[test]
fn link_body_with_downloads_disabled_is_recorded_explicitly() {
    let disabled = parse_link_body(serde_json::json!({
        "resource_type": "docs",
        "resource_id": "doc-1",
        "role": "commenter",
        "allow_download": false
    }));
    assert_eq!(disabled.allow_download, Some(false));
    assert!(!disabled.downloads_allowed());

    let enabled = parse_link_body(serde_json::json!({
        "resource_type": "docs",
        "resource_id": "doc-1",
        "role": "commenter",
        "allow_download": true
    }));
    assert_eq!(enabled.allow_download, Some(true));
    assert!(enabled.downloads_allowed());
}

#[test]
fn link_expiry_fields_are_independently_optional() {
    let legacy = parse_link_body(serde_json::json!({
        "resource_type": "docs",
        "resource_id": "doc-1",
        "role": "viewer",
        "expires_in_hours": 1
    }));
    assert_eq!(legacy.expires_in_hours, Some(1));
    assert_eq!(legacy.expires_at, None);

    let rfc3339 = parse_link_body(serde_json::json!({
        "resource_type": "docs",
        "resource_id": "doc-1",
        "role": "viewer",
        "expires_at": "2026-12-01T00:00:00Z"
    }));
    assert_eq!(rfc3339.expires_in_hours, None);
    assert_eq!(rfc3339.expires_at.as_deref(), Some("2026-12-01T00:00:00Z"));
}

#[test]
fn only_editable_roles_pass_the_dto_gate() {
    for role in ["viewer", "commenter", "editor"] {
        assert!(valid_editable_role(role));
    }
    for role in ["", "owner", "admin", "VIEWER", "viewer; drop"] {
        assert!(!valid_editable_role(role));
    }
}

#[test]
fn grantee_types_are_limited_to_user_group_and_domain() {
    for value in ["user", "group", "domain"] {
        assert!(valid_grantee_type(value));
    }
    for value in ["", "users", "email", "user;DROP", "Owner"] {
        assert!(!valid_grantee_type(value));
    }
}

#[test]
fn resource_validation_accepts_known_shapes_and_refuses_malformed_types() {
    assert!(sanitize_resource("docs", "doc-1"));
    assert!(sanitize_resource("sheet:cell", "authored-by-user"));
    assert!(!sanitize_resource("", "doc-1"));
    assert!(!sanitize_resource("docs;drop", "doc-1"));
    assert!(!sanitize_resource("docs/Bad", "doc-1"));
    assert!(!sanitize_resource("docs", ""));
    assert!(!sanitize_resource("docs", &"r".repeat(256)));
}

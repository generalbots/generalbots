//! #2452 — request-side tests for the share-link download policy.

use super::LinkBody;

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

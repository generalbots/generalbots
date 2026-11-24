//! Pure unit tests for the Drive public-link helpers (#2797, #2799, #2843).
//! No DB, drive or server is involved — these pin string/URL logic offline.

use super::public_link_url::{mutation_invalidates_link, revoke_bucket_from_record};
use botsecurity_core::content_disposition::{attachment_disposition, inline_disposition};

#[test]
fn hostile_object_name_yields_one_sanitized_filename_parameter() {
    for name in [
        "quote\"injection.txt",
        "back\\slash.txt",
        "crlf\r\nX-Injected: 1.txt",
        "control\0\x01\x7f.txt",
    ] {
        let disposition = attachment_disposition(name);
        assert!(disposition.starts_with("attachment; filename=\""), "{name}");
        assert_eq!(disposition.matches('"').count(), 2, "{name}");
        assert!(
            disposition
                .chars()
                .all(|c| !matches!(c, '\r' | '\n' | '\0' | '\\')),
            "{name}"
        );
        assert!(
            axum::http::HeaderValue::from_str(&disposition).is_ok(),
            "{name}"
        );
    }
}

#[test]
fn inline_disposition_is_also_sanitized() {
    let disposition = inline_disposition("img\" onload=\"x\r\nX: y.png");
    assert_eq!(disposition, "inline; filename=\"img onload=xX: y.png\"");
    assert_eq!(disposition.matches('"').count(), 2);
    assert!(axum::http::HeaderValue::from_str(&disposition).is_ok());
}

#[test]
fn mutation_revokes_exact_key_and_subtree_only_in_same_bucket() {
    assert!(mutation_invalidates_link(
        "b",
        "docs/a.txt",
        "b",
        "docs/a.txt"
    ));
    assert!(mutation_invalidates_link("b", "docs/a.txt", "b", "docs/"));
    assert!(mutation_invalidates_link(
        "b",
        "docs/sub/a.txt",
        "b",
        "docs"
    ));
    assert!(!mutation_invalidates_link(
        "b",
        "docs/a.txt",
        "other",
        "docs/a.txt"
    ));
    assert!(!mutation_invalidates_link(
        "b",
        "docs/a.txt",
        "b",
        "docs/b.txt"
    ));
    assert!(!mutation_invalidates_link("b", "docs2/a.txt", "b", "docs"));
    assert!(!mutation_invalidates_link("b", "docs/a.txt", "b", "doc"));
}

#[test]
fn overwriting_a_key_revokes_its_link_but_not_a_sibling_or_other_bucket() {
    let key = "users/u1/report.pdf";
    let links = [
        ("bucket", key),
        ("bucket", "users/u1/other.pdf"),
        ("other-bucket", key),
    ];
    let revoked: Vec<&str> = links
        .iter()
        .filter(|(bucket, link_key)| mutation_invalidates_link(bucket, link_key, "bucket", key))
        .map(|(_, link_key)| *link_key)
        .collect();
    assert_eq!(revoked, vec![key]);
}

#[test]
fn revoke_bucket_comes_from_record_and_fails_closed_on_blank() {
    // Creation binds the record's bucket from the caller's scope; revoke
    // must use it, never the client param or the workspace default (#2797).
    let personal = "user-11111111-1111-1111-1111-111111111111.gbai";
    assert_eq!(revoke_bucket_from_record(personal), Some(personal));
    assert_eq!(
        revoke_bucket_from_record("  acme.gborg  "),
        Some("acme.gborg")
    );
    assert_eq!(revoke_bucket_from_record(""), None);
    assert_eq!(revoke_bucket_from_record("   "), None);
}

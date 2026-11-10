//! Offline tests for the post-login redirect validation (#2130).
//!
//! The matrix covers every bypass the issue names — `//host`, absolute URLs,
//! `javascript:`/`data:` — plus the shapes that must keep working.

use crate::redirect::{safe_redirect_target, RedirectPolicy, DEFAULT_REDIRECT};

#[test]
fn relative_paths_are_allowed() {
    for target in ["/", "/settings", "/suite/desktop", "/settings?section=accounts&linked=1"] {
        assert_eq!(safe_redirect_target(Some(target)), target);
    }
}

#[test]
fn a_missing_or_empty_redirect_lands_on_the_root() {
    assert_eq!(safe_redirect_target(None), DEFAULT_REDIRECT);
    assert_eq!(safe_redirect_target(Some("")), DEFAULT_REDIRECT);
}

#[test]
fn protocol_relative_urls_are_rejected() {
    for target in ["//evil.example", "///evil.example", "//evil.example/path"] {
        assert_eq!(
            safe_redirect_target(Some(target)),
            DEFAULT_REDIRECT,
            "{target} must not become an open redirect"
        );
    }
}

#[test]
fn backslash_variants_are_rejected() {
    // Browsers normalise `/\evil.example` to `//evil.example`.
    for target in ["/\\evil.example", "\\/evil.example", "/path\\to", "\\\\evil.example"] {
        assert_eq!(safe_redirect_target(Some(target)), DEFAULT_REDIRECT, "{target}");
    }
}

#[test]
fn absolute_urls_are_rejected_by_default() {
    for target in [
        "https://evil.example",
        "https://evil.example/path",
        "http://evil.example",
        "HTTPS://EVIL.EXAMPLE",
        "https://evil.example\\@allowed.example",
    ] {
        assert_eq!(safe_redirect_target(Some(target)), DEFAULT_REDIRECT, "{target}");
    }
}

#[test]
fn non_http_schemes_are_rejected() {
    for target in [
        "javascript:alert(1)",
        "JavaScript:alert(1)",
        "data:text/html;base64,PHNjcmlwdD4=",
        "vbscript:msgbox(1)",
        "file:///etc/passwd",
        "ftp://evil.example/x",
        "mailto:someone@evil.example",
    ] {
        assert_eq!(safe_redirect_target(Some(target)), DEFAULT_REDIRECT, "{target}");
    }
}

#[test]
fn control_characters_and_whitespace_are_rejected() {
    for target in [
        "/path\nLocation: https://evil.example",
        "/path\r\nSet-Cookie: x=1",
        "/path\t",
        "/path with spaces",
        " /settings",
        "/\u{0}",
    ] {
        assert_eq!(safe_redirect_target(Some(target)), DEFAULT_REDIRECT, "{target:?}");
    }
}

#[test]
fn overlong_targets_are_rejected() {
    let long = format!("/{}", "a".repeat(3000));
    assert_eq!(safe_redirect_target(Some(long.as_str())), DEFAULT_REDIRECT);
}

#[test]
fn allowlisted_hosts_are_accepted() {
    let policy = RedirectPolicy::new(vec![
        "app.example".to_string(),
        "Cloud.Example:8443".to_string(),
    ]);
    assert_eq!(
        policy.sanitize(Some("https://app.example/suite")),
        "https://app.example/suite"
    );
    assert_eq!(
        policy.sanitize(Some("https://cloud.example:8443/")),
        "https://cloud.example:8443/"
    );
    // Host is matched exactly: a lookalike subdomain or a different port is not
    // the same origin.
    for target in [
        "https://app.example.evil.com/",
        "https://evil.app.example/",
        "https://app.example:8443/",
        "https://app.example@evil.example/",
        "https://evil.example#https://app.example",
        "https://evil.example?next=https://app.example",
    ] {
        assert_eq!(policy.sanitize(Some(target)), DEFAULT_REDIRECT, "{target}");
    }
}

#[test]
fn the_policy_can_be_built_from_the_deployment_base_url() {
    let policy = RedirectPolicy::from_base_url("http://localhost:8080");
    assert_eq!(policy.allowed_hosts(), vec!["localhost:8080".to_string()]);
    assert_eq!(
        policy.sanitize(Some("http://localhost:8080/settings")),
        "http://localhost:8080/settings"
    );
    // A different port on the same host is a different origin.
    assert_eq!(
        policy.sanitize(Some("http://localhost:3000/settings")),
        DEFAULT_REDIRECT
    );
    // An unparseable base URL yields an empty allowlist, never a wildcard.
    assert!(RedirectPolicy::from_base_url("not a url").allowed_hosts().is_empty());
    assert!(RedirectPolicy::from_base_url("").allowed_hosts().is_empty());
}

#[test]
fn an_empty_policy_allows_relative_paths_only() {
    let policy = RedirectPolicy::default();
    assert_eq!(policy.sanitize(Some("/settings")), "/settings");
    assert_eq!(
        policy.sanitize(Some("https://app.example/settings")),
        DEFAULT_REDIRECT
    );
}

#[test]
fn sanitizing_twice_is_idempotent() {
    let policy = RedirectPolicy::new(vec!["app.example".to_string()]);
    for target in [
        "/settings",
        "//evil.example",
        "https://evil.example",
        "https://app.example/x",
    ] {
        let once = policy.sanitize(Some(target));
        let twice = policy.sanitize(Some(&once));
        assert_eq!(once, twice, "{target}");
    }
}

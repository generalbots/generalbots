//! Adversarial URL-policy tests for agentic browsing (#2066).
//!
//! `url_allowed` is the allowlist/denylist gate in front of every browser
//! step. These tests cover the parser ambiguities that differ from a real
//! browser (backslashes, userinfo, trailing dots) and the documented
//! substring heuristic of the credential guard.

use super::*;

fn cfg(allowed: &[&str], denied: &[&str]) -> PolicyConfig {
    PolicyConfig {
        allowed_domains: allowed.iter().map(|s| s.to_string()).collect(),
        denied_domains: denied.iter().map(|s| s.to_string()).collect(),
        ..PolicyConfig::default()
    }
}

#[test]
fn the_parser_extracts_scheme_and_host_from_every_documented_form() {
    let cases = [
        ("https://example.com", ("https", "example.com")),
        ("HTTP://EXAMPLE.COM/a", ("http", "example.com")),
        ("https://example.com:8443/a", ("https", "example.com")),
        ("https://user:pass@example.com/a", ("https", "example.com")),
        ("https://example.com/a?q=1#f", ("https", "example.com")),
        ("https://example.com.", ("https", "example.com.")),
    ];
    for (url, expected) in cases {
        assert_eq!(
            parse_url_parts(url),
            Some((expected.0.to_string(), expected.1.to_string())),
            "{url}"
        );
    }
}

#[test]
fn the_parser_refuses_ambiguous_authorities() {
    // A browser reads the backslash as a path separator (host evil.com); the
    // naive split would read example.com and bypass the allowlist.
    for url in [
        "https://evil.com\\@example.com/",
        "https://exa mple.com/",
        "https://example.com\t/",
        "https://example.com\u{0}/",
        "https://",
        "https:///path",
        "https://:443/",
        "https://@/",
    ] {
        assert_eq!(parse_url_parts(url), None, "{url:?} must be refused");
    }
}

#[test]
fn host_matching_is_exact_or_a_dot_suffix() {
    assert!(host_matches("example.com", "example.com"));
    assert!(host_matches("www.example.com", "example.com"));
    assert!(host_matches("a.b.example.com", "example.com"));
    assert!(host_matches("example.com.", "example.com"));
    assert!(!host_matches("notexample.com", "example.com"));
    assert!(!host_matches("example.com.evil.com", "example.com"));
    assert!(!host_matches("example.co", "example.com"));
    assert!(!host_matches("anything", ""));
}

#[test]
fn an_allowlist_rejects_lookalike_and_suffix_confusion_hosts() {
    let c = cfg(&["example.com"], &[]);
    for url in [
        "https://example.com.evil.com/",
        "https://notexample.com/",
        "https://example.com.br/",
        "https://evil.com/example.com",
        "https://evil.com/?next=example.com",
        "https://evil.com#example.com",
    ] {
        assert!(url_allowed(&c, url).is_err(), "{url} must be denied");
    }
    assert!(url_allowed(&c, "https://sub.example.com/x").is_ok());
    assert!(url_allowed(&c, "https://EXAMPLE.com/x").is_ok());
}

#[test]
fn a_unicode_homograph_is_not_the_ascii_domain() {
    // Cyrillic 'е' (U+0435) looks like 'e' but is a different host.
    let c = cfg(&["example.com"], &[]);
    assert!(url_allowed(&c, "https://\u{0435}xample.com/").is_err());
}

#[test]
fn a_backslash_authority_cannot_bypass_the_allowlist() {
    let c = cfg(&["example.com"], &[]);
    assert!(url_allowed(&c, "https://evil.com\\@example.com/").is_err());
    assert!(url_allowed(&c, "https://example.com\\@evil.com/").is_err());
}

#[test]
fn denial_always_wins_and_suffixes_are_covered() {
    let c = cfg(&[], &["evil.com"]);
    for url in [
        "https://evil.com/",
        "https://www.evil.com/",
        "https://a.b.evil.com/",
        "https://EVIL.COM/",
    ] {
        assert!(url_allowed(&c, url).is_err(), "{url} must be denied");
    }

    // A denial for a TLD blocks every host under it.
    let tld = cfg(&[], &["com"]);
    assert!(url_allowed(&tld, "https://anything.com/").is_err());
    assert!(url_allowed(&tld, "https://anything.org/").is_ok());
}

#[test]
fn an_empty_allowed_entry_never_matches() {
    let c = cfg(&[""], &[]);
    assert!(url_allowed(&c, "https://example.com/").is_err());
}

#[test]
fn only_http_and_https_are_ever_accepted() {
    let c = cfg(&[], &[]);
    for url in [
        "file:///etc/passwd",
        "ftp://example.com/",
        "data:text/html,<script>",
        "javascript:alert(1)",
        "chrome://settings",
        "https:/example.com",
        "//example.com",
    ] {
        assert!(url_allowed(&c, url).is_err(), "{url} must be refused");
    }
}

#[test]
fn the_credential_guard_blocks_login_surfaces_for_checkout() {
    let c = PolicyConfig::default();
    for url in [
        "https://shop.com/login",
        "https://shop.com/LOGIN?next=/pay",
        "https://shop.com/account/login/step2",
        "https://shop.com/checkout?return=/login",
    ] {
        assert!(
            credential_guard(&c, url, TASK_KIND_CHECKOUT).is_err(),
            "{url} must be blocked"
        );
    }
    assert!(credential_guard(&c, "https://shop.com/pay", TASK_KIND_CHECKOUT).is_ok());
    assert!(credential_guard(&c, "https://shop.com/login", TASK_KIND_DEFAULT).is_ok());
}

#[test]
fn the_credential_guard_is_a_documented_substring_heuristic() {
    // KNOWN LIMITATION (#2066): only the literal `login` is detected. Paths
    // that also present credentials (`/signin`, `/password`) pass. Documented
    // so widening the heuristic is a deliberate change.
    let c = PolicyConfig::default();
    assert!(credential_guard(&c, "https://shop.com/signin", TASK_KIND_CHECKOUT).is_ok());
    assert!(credential_guard(&c, "https://shop.com/password", TASK_KIND_CHECKOUT).is_ok());
}

#[test]
fn step_allowed_applies_both_gates_in_order() {
    let c = cfg(&["example.com"], &[]);
    assert!(step_allowed(&c, "https://example.com/pay", TASK_KIND_CHECKOUT).is_ok());
    assert!(step_allowed(&c, "https://evil.com/pay", TASK_KIND_CHECKOUT).is_err());
    assert!(step_allowed(&c, "https://example.com/login", TASK_KIND_CHECKOUT).is_err());

    let open = PolicyConfig {
        allow_credential_pages: true,
        ..c.clone()
    };
    assert!(step_allowed(&open, "https://example.com/login", TASK_KIND_CHECKOUT).is_ok());
}

#[test]
fn from_plan_reads_a_policy_and_falls_back_to_defaults() {
    let plan = serde_json::json!({
        "policy": { "allowed_domains": ["example.com"], "max_steps": 5, "max_cost_units": 7 }
    });
    let parsed = PolicyConfig::from_plan(Some(&plan));
    assert_eq!(parsed.allowed_domains, vec!["example.com".to_string()]);
    assert_eq!(parsed.max_steps, 5);
    assert_eq!(parsed.max_cost_units, 7);
    assert!(!parsed.allow_credential_pages);

    for plan in [
        None,
        Some(serde_json::json!({})),
        Some(serde_json::json!({ "policy": null })),
        Some(serde_json::json!({ "policy": "not-an-object" })),
        Some(serde_json::json!({ "policy": { "max_steps": "many" } })),
    ] {
        assert_eq!(PolicyConfig::from_plan(plan.as_ref()), PolicyConfig::default());
    }
}

#[test]
fn the_default_policy_is_fail_closed_for_credentials() {
    let c = PolicyConfig::default();
    assert!(!c.allow_credential_pages);
    assert_eq!(c.max_steps, 60);
    assert_eq!(c.max_cost_units, 10_000);
    assert!(c.allowed_domains.is_empty());
    assert!(c.denied_domains.is_empty());
}

//! #2683 — tests for the pure CSRF exemption decision predicate in
//! `csrf_middleware.rs`. Kept in a sibling file so the middleware stays below
//! the project's 450-line limit; the module is declared inline beside it.

use super::*;

fn config() -> CsrfConfig {
    CsrfConfig {
        exempt_paths: vec![
            "/webhook/whatsapp".into(),
            "/api/cloud/auth/login".into(),
        ],
        origin_check_paths: vec!["/api/cloud/auth/login".into()],
        ..CsrfConfig::default()
    }
}

#[test]
fn a_cookie_state_change_on_an_unguarded_path_requires_validation() {
    // The sheet/catalog paths are deliberately absent from `exempt_paths`.
    for path in ["/api/sheet/save", "/api/sheet/delete", "/api/catalog/products"] {
        let decision = csrf_decision(&config(), "POST", path, false, false, Some(true), false);
        assert_eq!(decision, CsrfDecision::Validate, "{path} must need a token");
    }
}

#[test]
fn a_bearer_state_change_is_stateless_and_skips_validation() {
    let decision = csrf_decision(
        &config(),
        "POST",
        "/api/sheet/save",
        true,
        false,
        Some(true),
        false,
    );
    assert_eq!(decision, CsrfDecision::Skip);
}

#[test]
fn a_read_only_method_skips_validation() {
    let decision = csrf_decision(
        &config(),
        "GET",
        "/api/sheet/list",
        false,
        false,
        Some(true),
        false,
    );
    assert_eq!(decision, CsrfDecision::Skip);
}

#[test]
fn a_documented_machine_path_skips_validation_without_a_cookie() {
    // Webhooks authenticate by provider signature, never a session cookie.
    let decision = csrf_decision(
        &config(),
        "POST",
        "/webhook/whatsapp",
        false,
        false,
        Some(true),
        false,
    );
    assert_eq!(decision, CsrfDecision::Skip);
}

#[test]
fn an_origin_checked_post_rejects_a_foreign_cookie_but_accepts_a_bearer() {
    let cfg = config();
    let foreign = csrf_decision(
        &cfg,
        "POST",
        "/api/cloud/auth/login",
        false,
        false,
        Some(true),
        false,
    );
    assert_eq!(foreign, CsrfDecision::RejectCrossOrigin);

    let same_origin = csrf_decision(
        &cfg,
        "POST",
        "/api/cloud/auth/login",
        false,
        true,
        Some(true),
        false,
    );
    assert_eq!(same_origin, CsrfDecision::Skip);

    let bearer = csrf_decision(
        &cfg,
        "POST",
        "/api/cloud/auth/login",
        true,
        false,
        Some(true),
        false,
    );
    assert_eq!(bearer, CsrfDecision::Skip);
}

#[test]
fn no_security_mode_keeps_the_loopback_origin_guard() {
    let mut cfg = config();
    cfg.no_security_mode = true;

    // A foreign Origin from a cross-site page is forbidden.
    let foreign = csrf_decision(&cfg, "POST", "/api/sheet/save", false, false, None, false);
    assert_eq!(foreign, CsrfDecision::RejectNonLoopback);

    // A loopback origin and a local non-browser client (no Origin) pass.
    let loopback = csrf_decision(
        &cfg,
        "POST",
        "/api/sheet/save",
        false,
        false,
        Some(true),
        false,
    );
    assert_eq!(loopback, CsrfDecision::Skip);
    let no_origin = csrf_decision(
        &cfg,
        "POST",
        "/api/sheet/save",
        false,
        false,
        Some(false),
        false,
    );
    assert_eq!(no_origin, CsrfDecision::Skip);
}

#[test]
fn a_verified_internal_service_skips_validation() {
    let decision = csrf_decision(
        &config(),
        "POST",
        "/api/sheet/save",
        false,
        false,
        Some(true),
        true,
    );
    assert_eq!(decision, CsrfDecision::Skip);
}

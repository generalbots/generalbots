//! Origin policy unit tests (#2129): resolution rules, allowlist normalisation
//! and the predicate that callers of `validate_origin` rely on.

use super::*;
use crate::cors::cors_test_support::*;
use axum::http::Request;

#[test]
fn dev_override_is_off_by_default() -> TestResult {
    let _state = CorsState::with_origins(&[]);
    let empty: Vec<String> = Vec::new();

    assert_eq!(resolve_origin_policy(&empty), OriginPolicy::Deny);
    assert!(!allow_any_origin_override());

    std::env::set_var(CORS_ALLOW_ANY_ORIGIN_ENV, "1");
    assert_eq!(
        resolve_origin_policy(&empty),
        OriginPolicy::AnyNoCredentials
    );

    std::env::remove_var(CORS_ALLOW_ANY_ORIGIN_ENV);
    assert_eq!(resolve_origin_policy(&empty), OriginPolicy::Deny);
    Ok(())
}

#[test]
fn production_ignores_the_any_origin_override() -> TestResult {
    let _state = CorsState::with_origins(&[]);
    let empty: Vec<String> = Vec::new();

    std::env::set_var(CORS_ALLOW_ANY_ORIGIN_ENV, "1");
    std::env::set_var(crate::jwt_secret::ENVIRONMENT_ENV, "production");

    assert!(any_origin_override_requested());
    assert!(
        !allow_any_origin_override(),
        "production must fail closed even with the dev switch set"
    );
    assert_eq!(resolve_origin_policy(&empty), OriginPolicy::Deny);
    Ok(())
}

#[test]
fn explicit_allowlist_wins_over_dev_override() -> TestResult {
    let state = CorsState::with_origins(&[]);
    state.enable_any_origin_override();

    assert_eq!(
        resolve_origin_policy(&[APP_ORIGIN.to_string()]),
        OriginPolicy::Allowlist(vec![APP_ORIGIN.to_string()]),
        "an operator-configured allowlist is never widened by the dev switch"
    );
    Ok(())
}

#[test]
fn x_user_id_is_not_a_default_allowed_header() {
    for config in [
        CorsConfig::default(),
        CorsConfig::production(),
        CorsConfig::development(),
        CorsConfig::api(),
    ] {
        assert!(
            config
                .allowed_headers
                .iter()
                .all(|name| !name.eq_ignore_ascii_case("x-user-id")),
            "x-user-id must stay out of the default CORS header list"
        );
    }
}

#[test]
fn validate_origin_fails_closed_without_configuration() -> TestResult {
    let _state = CorsState::with_origins(&[]);

    let parts = Request::builder().uri("/").body(())?.into_parts().0;
    let evil = HeaderValue::from_static(EVIL_ORIGIN);

    assert!(!validate_origin(&evil, &parts));
    Ok(())
}

#[test]
fn validate_origin_matches_only_configured_origins() -> TestResult {
    let _state = CorsState::with_origins(&[APP_ORIGIN]);

    let parts = Request::builder().uri("/").body(())?.into_parts().0;
    let allowed = HeaderValue::from_static(APP_ORIGIN);
    let evil = HeaderValue::from_static(EVIL_ORIGIN);

    assert!(validate_origin(&allowed, &parts));
    assert!(!validate_origin(&evil, &parts));
    Ok(())
}

#[test]
fn normalize_origins_keeps_only_exact_unique_entries() {
    let normalized = normalize_origins(&[
        " https://app.example ".to_string(),
        "https://app.example".to_string(),
        String::new(),
        "*".to_string(),
        "https://second.example:8443".to_string(),
    ]);

    assert_eq!(
        normalized,
        vec![
            APP_ORIGIN.to_string(),
            "https://second.example:8443".to_string()
        ]
    );
}

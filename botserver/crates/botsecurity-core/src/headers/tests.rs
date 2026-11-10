//! Tests for the shared IMP-09 header layer (`headers/mod.rs`).
//!
//! Split out of `mod.rs` to keep that file small. The point of these tests is
//! coverage of the *error* paths: a header layer that only decorates 200s is
//! useless, because a 401/403/404/429/500 is exactly the response an attacker
//! inspects for missing protections.

use super::*;
use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, Router},
};
use tower::ServiceExt;

/// The six headers IMP-09 requires on every response.
const REQUIRED: [&str; 6] = [
    "content-security-policy",
    "strict-transport-security",
    "x-frame-options",
    "x-content-type-options",
    "referrer-policy",
    "permissions-policy",
];

async fn ok_handler() -> impl IntoResponse {
    (StatusCode::OK, "ok")
}

async fn failing_handler() -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "boom")
}

async fn unauthorized_handler() -> impl IntoResponse {
    (StatusCode::UNAUTHORIZED, "nope")
}

/// A router shaped like a real server: a route, a nested sub-router with its
/// own fallback, an unmatched-path fallback and a 500 route.
fn test_router() -> Router {
    let nested = Router::new()
        .route("/ok", get(ok_handler))
        .fallback(|| async { StatusCode::NOT_FOUND });
    Router::new()
        .route("/ok", get(ok_handler))
        .route("/boom", get(failing_handler))
        .route("/private", get(unauthorized_handler))
        .nest("/nested", nested)
        .fallback(|| async { StatusCode::NOT_FOUND })
}

async fn call(path: &str) -> (StatusCode, axum::http::HeaderMap) {
    let app = with_security_headers(test_router(), SecurityHeadersConfig::from_env());
    let request = Request::builder()
        .uri(path)
        .body(Body::empty())
        .expect("test request is well formed");
    let response = app.oneshot(request).await.expect("router answers");
    (response.status(), response.headers().clone())
}

fn assert_all_six(headers: &axum::http::HeaderMap, context: &str) {
    for name in REQUIRED {
        assert!(
            headers.contains_key(name),
            "{name} missing from {context}: {headers:?}"
        );
        let value = headers
            .get(name)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default();
        assert!(!value.is_empty(), "{name} is empty on {context}");
    }
}

#[tokio::test]
async fn a_200_carries_every_imp09_header() {
    let (status, headers) = call("/ok").await;
    assert_eq!(status, StatusCode::OK);
    assert_all_six(&headers, "200");
}

#[tokio::test]
async fn a_404_carries_every_imp09_header() {
    let (status, headers) = call("/no-such-route").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_all_six(&headers, "404");
}

#[tokio::test]
async fn a_nested_404_carries_every_imp09_header() {
    let (status, headers) = call("/nested/no-such-route").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_all_six(&headers, "nested 404");
}

#[tokio::test]
async fn a_500_carries_every_imp09_header() {
    let (status, headers) = call("/boom").await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_all_six(&headers, "500");
}

#[tokio::test]
async fn a_401_carries_every_imp09_header() {
    let (status, headers) = call("/private").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_all_six(&headers, "401");
}

#[tokio::test]
async fn handler_scoped_csp_is_preserved() {
    async fn strict_csp_handler() -> Response {
        let mut response = Response::new(Body::from("preview"));
        response.headers_mut().insert(
            HeaderName::from_static("content-security-policy"),
            HeaderValue::from_static("default-src 'none'; style-src 'unsafe-inline'"),
        );
        response
    }

    let app = with_security_headers(
        Router::new().route("/strict", get(strict_csp_handler)),
        SecurityHeadersConfig::default(),
    );
    let request = Request::builder()
        .uri("/strict")
        .body(Body::empty())
        .expect("test request is well formed");
    let response = app.oneshot(request).await.expect("router answers");
    assert_eq!(
        response
            .headers()
            .get("content-security-policy")
            .and_then(|value| value.to_str().ok()),
        Some("default-src 'none'; style-src 'unsafe-inline'")
    );
}

#[tokio::test]
async fn the_extension_middleware_matches_the_router_layer() {
    // The Extension-based middleware stays for callers that already hold the
    // config in router state; both paths must emit the same six headers.
    let config = SecurityHeadersConfig::default();
    let app = Router::new()
        .route("/", get(ok_handler))
        .layer(axum::middleware::from_fn(security_headers_middleware))
        .layer(axum::Extension(config));
    let request = Request::builder()
        .uri("/")
        .body(Body::empty())
        .expect("test request is well formed");
    let response = app.oneshot(request).await.expect("router answers");
    assert_all_six(response.headers(), "extension middleware");
    assert_eq!(
        response
            .headers()
            .get("x-content-type-options")
            .and_then(|value| value.to_str().ok()),
        Some("nosniff")
    );
}

#[test]
fn hsts_is_on_unless_explicitly_disabled() {
    assert!(hsts_value_enabled(None));
    for value in ["1", "on", "true", "yes", "TRUE", ""] {
        assert!(hsts_value_enabled(Some(value)), "{value:?} keeps HSTS");
    }
    for value in ["0", "off", "false", "no", "OFF"] {
        assert!(!hsts_value_enabled(Some(value)), "{value:?} drops HSTS");
    }
}

#[test]
fn the_default_config_sets_every_required_header() {
    let config = SecurityHeadersConfig::default();
    assert!(config.content_security_policy.is_some());
    assert!(config.strict_transport_security.is_some());
    assert_eq!(config.x_frame_options.as_deref(), Some("DENY"));
    assert_eq!(config.x_content_type_options.as_deref(), Some("nosniff"));
    assert!(config.referrer_policy.is_some());
    assert!(config.permissions_policy.is_some());
}

#[test]
fn the_api_profile_keeps_all_six_headers() {
    // `api()` used to leave `permissions_policy` empty, so JSON callers got
    // five of the six. The six are not negotiable per profile.
    let config = SecurityHeadersConfig::api();
    assert!(config.content_security_policy.is_some());
    assert!(config.strict_transport_security.is_some());
    assert!(config.x_frame_options.is_some());
    assert!(config.x_content_type_options.is_some());
    assert!(config.referrer_policy.is_some());
    assert!(config.permissions_policy.is_some());
}

#[test]
fn the_relaxed_profile_keeps_all_six_headers() {
    let config = SecurityHeadersConfig::relaxed();
    assert!(config.content_security_policy.is_some());
    assert!(config.strict_transport_security.is_some());
    assert!(config.x_frame_options.is_some());
    assert!(config.x_content_type_options.is_some());
    assert!(config.referrer_policy.is_some());
    assert!(config.permissions_policy.is_some());
}

#[test]
fn the_strict_profile_keeps_all_six_headers() {
    let config = SecurityHeadersConfig::strict();
    assert!(config.content_security_policy.is_some());
    assert!(config.strict_transport_security.is_some());
    assert!(config.x_frame_options.is_some());
    assert!(config.x_content_type_options.is_some());
    assert!(config.referrer_policy.is_some());
    assert!(config.permissions_policy.is_some());
}

#[test]
fn default_permissions_policy_allows_self_media_capture() {
    // The chat composer records voice notes and camera shots through
    // getUserMedia from the same origin. `microphone=()` / `camera=()`
    // block the browser prompt entirely, so the buttons look dead.
    let config = SecurityHeadersConfig::default();
    let policy = config
        .permissions_policy
        .expect("default permissions policy");
    assert!(policy.contains("microphone=(self)"), "policy: {policy}");
    assert!(policy.contains("camera=(self)"), "policy: {policy}");
    assert!(policy.contains("geolocation=()"), "policy: {policy}");
}

#[test]
fn builder_methods_round_trip() {
    let config = SecurityHeadersConfig::default()
        .with_csp("default-src 'self'")
        .with_frame_options("SAMEORIGIN")
        .with_hsts(63072000, true, true)
        .with_referrer_policy("no-referrer")
        .with_custom_header("X-Custom", "value");

    assert_eq!(
        config.content_security_policy.as_deref(),
        Some("default-src 'self'")
    );
    assert_eq!(config.x_frame_options.as_deref(), Some("SAMEORIGIN"));
    assert!(config
        .strict_transport_security
        .as_deref()
        .is_some_and(|value| value.contains("63072000")));
    assert_eq!(config.referrer_policy.as_deref(), Some("no-referrer"));
    assert_eq!(
        config.custom_headers.get("X-Custom").map(String::as_str),
        Some("value")
    );
}

#[test]
fn disable_hsts_and_without_csp_only_touch_their_own_header() {
    let config = SecurityHeadersConfig::default()
        .disable_hsts()
        .without_csp();
    assert!(config.strict_transport_security.is_none());
    assert!(config.content_security_policy.is_none());
    assert_eq!(config.x_content_type_options.as_deref(), Some("nosniff"));
}

#[test]
fn csp_builder_composes_directives() {
    let csp = CspBuilder::new()
        .default_src(&["'self'"])
        .script_src(&["'self'", "'unsafe-inline'"])
        .style_src(&["'self'", "https://fonts.googleapis.com"])
        .img_src(&["'self'", "data:", "https:"])
        .upgrade_insecure_requests()
        .build();

    assert!(csp.contains("default-src 'self'"));
    assert!(csp.contains("script-src 'self' 'unsafe-inline'"));
    assert!(csp.contains("upgrade-insecure-requests"));

    let empty = CspBuilder::new()
        .default_src(&["'none'"])
        .block_all_mixed_content()
        .build();
    assert!(empty.contains("block-all-mixed-content"));
    assert!(empty.contains("default-src 'none'"));
}

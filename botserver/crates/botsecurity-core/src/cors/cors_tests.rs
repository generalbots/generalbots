//! CORS fail-closed regression tests (#2129).
//!
//! The contract under test: an empty origin configuration means "no
//! cross-origin access". A foreign `Origin` must never receive
//! `Access-Control-Allow-Origin`, credentials are only announced for an
//! explicitly allowlisted origin, a `*` entry never widens the allowlist, and
//! the `CORS_ALLOW_ANY_ORIGIN=1` development override stays off by default and
//! always refuses credentials.

use super::*;
use axum::{body::Body, http::Request, routing::get, Router};
use std::sync::{Mutex, MutexGuard};
use tower::ServiceExt;

type TestResult = Result<(), Box<dyn std::error::Error>>;

/// Serialises the tests that touch the process-wide origin list and the
/// `CORS_ALLOW_ANY_ORIGIN` switch (both are process globals).
static CORS_STATE_LOCK: Mutex<()> = Mutex::new(());

/// Owns the global CORS state for one test and restores the fail-closed default
/// on drop.
struct CorsState {
    _guard: MutexGuard<'static, ()>,
}

impl CorsState {
    fn with_origins(origins: &[&str]) -> Self {
        let guard = match CORS_STATE_LOCK.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        std::env::remove_var(CORS_ALLOW_ANY_ORIGIN_ENV);
        set_cors_allowed_origins(origins.iter().map(|origin| origin.to_string()).collect());

        Self { _guard: guard }
    }

    fn enable_any_origin_override(&self) {
        std::env::set_var(CORS_ALLOW_ANY_ORIGIN_ENV, "1");
    }
}

impl Drop for CorsState {
    fn drop(&mut self) {
        std::env::remove_var(CORS_ALLOW_ANY_ORIGIN_ENV);
        set_cors_allowed_origins(Vec::new());
    }
}

fn app_with(layer: CorsLayer) -> Router {
    Router::new()
        .route("/api/ping", get(|| async { "ok" }))
        .layer(layer)
}

/// Sends one request through a fresh router carrying `layer`. An `OPTIONS`
/// request is built as a real preflight asking for `POST` + `x-user-id`.
async fn send(
    layer: CorsLayer,
    origin: Option<&str>,
    method: Method,
) -> Result<axum::response::Response, Box<dyn std::error::Error>> {
    let is_preflight = method == Method::OPTIONS;
    let mut builder = Request::builder().method(method).uri("/api/ping");

    if let Some(origin) = origin {
        builder = builder.header(header::ORIGIN, origin);
    }

    if is_preflight {
        builder = builder
            .header(header::ACCESS_CONTROL_REQUEST_METHOD, "POST")
            .header(header::ACCESS_CONTROL_REQUEST_HEADERS, "x-user-id");
    }

    let request = builder.body(Body::empty())?;

    Ok(app_with(layer).oneshot(request).await?)
}

fn header_value<'a>(
    response: &'a axum::response::Response,
    name: header::HeaderName,
) -> Option<&'a str> {
    response
        .headers()
        .get(name)
        .and_then(|value| value.to_str().ok())
}

const EVIL_ORIGIN: &str = "https://evil.example";
const APP_ORIGIN: &str = "https://app.example";

#[tokio::test]
async fn empty_config_denies_arbitrary_origin() -> TestResult {
    let _state = CorsState::with_origins(&[]);

    let response = send(create_cors_layer(), Some(EVIL_ORIGIN), Method::GET).await?;

    assert_eq!(
        header_value(&response, header::ACCESS_CONTROL_ALLOW_ORIGIN),
        None,
        "an unconfigured deployment must not reflect the caller's origin"
    );
    assert_eq!(
        header_value(&response, header::ACCESS_CONTROL_ALLOW_CREDENTIALS),
        None,
        "no origin configured means no credentialed cross-origin access"
    );
    Ok(())
}

#[tokio::test]
async fn empty_config_denies_preflight() -> TestResult {
    let _state = CorsState::with_origins(&[]);

    let response = send(create_cors_layer(), Some(EVIL_ORIGIN), Method::OPTIONS).await?;

    assert_eq!(
        header_value(&response, header::ACCESS_CONTROL_ALLOW_ORIGIN),
        None
    );
    assert_eq!(
        header_value(&response, header::ACCESS_CONTROL_ALLOW_METHODS),
        None
    );
    assert_eq!(
        header_value(&response, header::ACCESS_CONTROL_ALLOW_HEADERS),
        None
    );
    assert_eq!(
        header_value(&response, header::ACCESS_CONTROL_ALLOW_CREDENTIALS),
        None
    );
    Ok(())
}

#[tokio::test]
async fn configured_origin_is_echoed_with_credentials() -> TestResult {
    let _state = CorsState::with_origins(&[APP_ORIGIN]);

    let response = send(create_cors_layer(), Some(APP_ORIGIN), Method::GET).await?;

    assert_eq!(
        header_value(&response, header::ACCESS_CONTROL_ALLOW_ORIGIN),
        Some(APP_ORIGIN)
    );
    assert_eq!(
        header_value(&response, header::ACCESS_CONTROL_ALLOW_CREDENTIALS),
        Some("true")
    );
    Ok(())
}

#[tokio::test]
async fn configured_list_rejects_unknown_and_lookalike_origin() -> TestResult {
    let _state = CorsState::with_origins(&[APP_ORIGIN]);

    for origin in [
        EVIL_ORIGIN,
        "https://app.example.evil.example",
        "http://app.example",
    ] {
        let response = send(create_cors_layer(), Some(origin), Method::GET).await?;

        assert_eq!(
            header_value(&response, header::ACCESS_CONTROL_ALLOW_ORIGIN),
            None,
            "{origin} must not be echoed"
        );
        assert_eq!(
            header_value(&response, header::ACCESS_CONTROL_ALLOW_CREDENTIALS),
            None,
            "{origin} must not be offered credentials"
        );
    }
    Ok(())
}

#[tokio::test]
async fn wildcard_entry_in_config_never_allows_any_origin() -> TestResult {
    let _state = CorsState::with_origins(&["*"]);

    for origin in [EVIL_ORIGIN, APP_ORIGIN] {
        let response = send(create_cors_layer(), Some(origin), Method::GET).await?;

        assert_eq!(
            header_value(&response, header::ACCESS_CONTROL_ALLOW_ORIGIN),
            None,
            "a `*` config entry must not become a reflective origin"
        );
        assert_eq!(
            header_value(&response, header::ACCESS_CONTROL_ALLOW_CREDENTIALS),
            None
        );
    }
    Ok(())
}

#[tokio::test]
async fn dev_override_allows_origin_without_credentials() -> TestResult {
    let state = CorsState::with_origins(&[]);
    state.enable_any_origin_override();

    let response = send(create_cors_layer(), Some(EVIL_ORIGIN), Method::GET).await?;

    assert_eq!(
        header_value(&response, header::ACCESS_CONTROL_ALLOW_ORIGIN),
        Some(EVIL_ORIGIN),
        "the dev override echoes a format-valid origin"
    );
    assert_eq!(
        header_value(&response, header::ACCESS_CONTROL_ALLOW_CREDENTIALS),
        None,
        "the dev override must never allow credentials"
    );
    assert!(
        !response
            .headers()
            .values()
            .any(|value| value.as_bytes() == b"*"),
        "no wildcard is ever emitted"
    );
    Ok(())
}

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

#[tokio::test]
async fn preflight_does_not_grant_x_user_id() -> TestResult {
    let _state = CorsState::with_origins(&[APP_ORIGIN]);

    let response = send(create_cors_layer(), Some(APP_ORIGIN), Method::OPTIONS).await?;

    assert_eq!(
        header_value(&response, header::ACCESS_CONTROL_ALLOW_ORIGIN),
        Some(APP_ORIGIN)
    );

    let allowed_headers = header_value(&response, header::ACCESS_CONTROL_ALLOW_HEADERS)
        .unwrap_or_default()
        .to_ascii_lowercase();

    assert!(
        !allowed_headers.contains("x-user-id"),
        "x-user-id must not be a CORS-allowed header, got: {allowed_headers}"
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

#[tokio::test]
async fn every_builder_fails_closed() -> TestResult {
    let _state = CorsState::with_origins(&[]);

    let layers = vec![
        create_cors_layer(),
        create_cors_layer_for_production(Vec::new()),
        create_cors_layer_with_origins(Vec::new()),
        CorsConfig::production().build(),
        CorsConfig::api().build(),
        CorsConfig::from_config_value(None).build(),
        CorsConfig::from_config_value(Some("  ,  ")).build(),
    ];

    for layer in layers {
        let response = send(layer, Some(EVIL_ORIGIN), Method::GET).await?;

        assert_eq!(
            header_value(&response, header::ACCESS_CONTROL_ALLOW_ORIGIN),
            None,
            "every CORS builder must share the fail-closed rule"
        );
    }
    Ok(())
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

    assert!(validate_origin(
        &HeaderValue::from_static(APP_ORIGIN),
        &parts
    ));
    assert!(!validate_origin(
        &HeaderValue::from_static(EVIL_ORIGIN),
        &parts
    ));
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

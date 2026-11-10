//! Shared harness for the CORS fail-closed tests (#2129).
//!
//! Every test that touches the process-wide origin list or the environment
//! switches takes [`CorsState`], which serialises the tests and restores the
//! fail-closed default on drop.

use super::*;
use axum::{body::Body, http::Request, routing::get, Router};
use std::sync::{Mutex, MutexGuard};
use tower::ServiceExt;

pub(super) type TestResult = Result<(), Box<dyn std::error::Error>>;

pub(super) const EVIL_ORIGIN: &str = "https://evil.example";
pub(super) const APP_ORIGIN: &str = "https://app.example";

/// Serialises the tests that touch process globals.
static CORS_STATE_LOCK: Mutex<()> = Mutex::new(());

/// Owns the global CORS state for one test and restores the fail-closed default
/// on drop.
pub(super) struct CorsState {
    _guard: MutexGuard<'static, ()>,
    prior_environment: Option<String>,
    prior_override: Option<String>,
}

impl CorsState {
    pub(super) fn with_origins(origins: &[&str]) -> Self {
        let guard = match CORS_STATE_LOCK.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        let prior_environment = std::env::var(crate::jwt_secret::ENVIRONMENT_ENV).ok();
        let prior_override = std::env::var(CORS_ALLOW_ANY_ORIGIN_ENV).ok();

        std::env::remove_var(CORS_ALLOW_ANY_ORIGIN_ENV);
        std::env::remove_var(crate::jwt_secret::ENVIRONMENT_ENV);
        set_cors_allowed_origins(origins.iter().map(|origin| origin.to_string()).collect());

        Self {
            _guard: guard,
            prior_environment,
            prior_override,
        }
    }

    pub(super) fn enable_any_origin_override(&self) {
        std::env::set_var(CORS_ALLOW_ANY_ORIGIN_ENV, "1");
    }
}

impl Drop for CorsState {
    fn drop(&mut self) {
        restore_env(
            crate::jwt_secret::ENVIRONMENT_ENV,
            self.prior_environment.take(),
        );
        restore_env(CORS_ALLOW_ANY_ORIGIN_ENV, self.prior_override.take());
        set_cors_allowed_origins(Vec::new());
    }
}

fn restore_env(name: &str, value: Option<String>) {
    match value {
        Some(value) => std::env::set_var(name, value),
        None => std::env::remove_var(name),
    }
}

pub(super) fn app_with(layer: CorsLayer) -> Router {
    Router::new()
        .route("/api/ping", get(|| async { "ok" }))
        .layer(layer)
}

/// Sends one request through a fresh router carrying `layer`. An `OPTIONS`
/// request is built as a real preflight asking for `POST` + `x-user-id`.
pub(super) async fn send(
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

pub(super) fn header_value<'a>(
    response: &'a axum::response::Response,
    name: header::HeaderName,
) -> Option<&'a str> {
    response
        .headers()
        .get(name)
        .and_then(|value| value.to_str().ok())
}

/// No CORS grant of any kind: the browser must reject the cross-origin read.
pub(super) fn assert_denied(response: &axum::response::Response, context: &str) {
    assert_eq!(
        header_value(response, header::ACCESS_CONTROL_ALLOW_ORIGIN),
        None,
        "{context}: the caller's origin must never be echoed"
    );
    assert_eq!(
        header_value(response, header::ACCESS_CONTROL_ALLOW_CREDENTIALS),
        None,
        "{context}: credentials must not be offered"
    );
}

/// Exact echo of a configured origin, with credentials (allowlist mode only).
pub(super) fn assert_allowed_with_credentials(response: &axum::response::Response, origin: &str) {
    assert_eq!(
        header_value(response, header::ACCESS_CONTROL_ALLOW_ORIGIN),
        Some(origin)
    );
    assert_eq!(
        header_value(response, header::ACCESS_CONTROL_ALLOW_CREDENTIALS),
        Some("true")
    );
}

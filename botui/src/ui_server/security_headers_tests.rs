//! IMP-09 header tests for the three BotUI servers (suite :3000, cloud :4000,
//! login :5000).
//!
//! Every router is built the way `main.rs` builds it and driven in-process, so
//! the assertions cover the wiring itself — not a hand-made router that happens
//! to look like it. `harden()` (the same call the routers make) is additionally
//! exercised against a 404, a 500 and a 401, because a header layer that only
//! decorates 200s is the classic regression: the error page is what an attacker
//! reads.

use super::*;
use axum::http::{Request, StatusCode};
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

async fn send(router: Router, path: &str) -> (StatusCode, axum::http::HeaderMap) {
    let request = Request::builder()
        .uri(path)
        .body(axum::body::Body::empty())
        .expect("test request is well formed");
    let response = router.oneshot(request).await.expect("router answers");
    (response.status(), response.headers().clone())
}

fn assert_all_six(headers: &axum::http::HeaderMap, context: &str) {
    for name in REQUIRED {
        let value = headers
            .get(name)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default();
        assert!(!value.is_empty(), "{name} missing from {context}");
    }
}

/// A router with the exact shapes the servers produce: a 200 route, a 404
/// fallback, a 500 route and a 401 route.
fn error_shapes() -> Router {
    async fn ok() -> StatusCode {
        StatusCode::OK
    }
    async fn boom() -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
    async fn private() -> StatusCode {
        StatusCode::UNAUTHORIZED
    }
    harden(
        Router::new()
            .route("/ok", get(ok))
            .route("/boom", get(boom))
            .route("/private", get(private))
            .fallback(|| async { StatusCode::NOT_FOUND }),
    )
}

#[tokio::test]
async fn the_suite_router_serves_health_with_every_header() {
    let (status, headers) = send(configure_router(), "/health").await;
    assert_eq!(status, StatusCode::OK);
    assert_all_six(&headers, "suite /health");
}

#[tokio::test]
async fn the_cloud_router_serves_health_with_every_header() {
    let (status, headers) = send(configure_cloud_router(), "/health").await;
    assert_eq!(status, StatusCode::OK);
    assert_all_six(&headers, "cloud /health");
}

#[tokio::test]
async fn the_login_router_serves_health_with_every_header() {
    let (status, headers) = send(configure_login_router(), "/health").await;
    assert_eq!(status, StatusCode::OK);
    assert_all_six(&headers, "login /health");
}

#[tokio::test]
async fn the_suite_router_answers_a_static_404_with_every_header() {
    let router = configure_router();
    let (status, headers) = send(router, "/suite/chat/__imp09_missing_probe__.js").await;
    // With `embed-ui` off this is a ServeDir 404; with it on the embedded
    // handler 404s the same way. Either way the layer must have run.
    assert_eq!(status, StatusCode::NOT_FOUND, "static miss must 404");
    assert_all_six(&headers, "suite static 404");
}

#[tokio::test]
async fn the_cloud_router_answers_a_backend_error_with_every_header() {
    // No botserver is running in a unit test, so the /api proxy fails. The
    // point is the failure response: it must still carry the six headers.
    let (status, headers) = send(configure_cloud_router(), "/api/__imp09_probe__").await;
    assert!(
        status.is_client_error() || status.is_server_error(),
        "proxy failure must be an error status, got {status}"
    );
    assert_all_six(&headers, "cloud proxy error");
}

#[tokio::test]
async fn the_login_router_answers_a_backend_error_with_every_header() {
    let (status, headers) = send(configure_login_router(), "/api/__imp09_probe__").await;
    assert!(
        status.is_client_error() || status.is_server_error(),
        "proxy failure must be an error status, got {status}"
    );
    assert_all_six(&headers, "login proxy error");
}

#[tokio::test]
async fn a_404_carries_every_header() {
    let (status, headers) = send(error_shapes(), "/nothing-here").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_all_six(&headers, "404");
}

#[tokio::test]
async fn a_500_carries_every_header() {
    let (status, headers) = send(error_shapes(), "/boom").await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_all_six(&headers, "500");
}

#[tokio::test]
async fn a_401_carries_every_header() {
    let (status, headers) = send(error_shapes(), "/private").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_all_six(&headers, "401");
}

#[tokio::test]
async fn a_200_carries_every_header() {
    let (status, headers) = send(error_shapes(), "/ok").await;
    assert_eq!(status, StatusCode::OK);
    assert_all_six(&headers, "200");
}

#[test]
fn hsts_can_be_turned_off_for_plain_http_local_dev() {
    // `harden()` reads `GB_SECURITY_HSTS`; the parser is shared with botserver
    // and tested there. Here we only pin that the layer honours a disabled
    // HSTS config, which is what the env flag produces.
    let config = SecurityHeadersConfig::from_env();
    if botsecurity_core::headers::hsts_enabled() {
        assert!(config.strict_transport_security.is_some());
    } else {
        assert!(config.strict_transport_security.is_none());
    }
}

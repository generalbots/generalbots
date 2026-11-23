//! #2760 — access-classification tests for the sandbox route table.
//!
//! The loopback listener serves signed resources only. Config, status and
//! revoke are management surfaces that must stay on the authenticated main
//! router; these tests keep the predicate and the listener's route table in
//! lockstep so a future route cannot silently become public.

use super::{
    classify_sandbox_route, SandboxRouteAccess, CONFIG_ROUTE, PUBLIC_ROUTES, RESOURCE_ROUTE,
    REVOKE_ROUTE, STATUS_ROUTE,
};

const HASH_PATH: &str =
    "/sandbox/ui/0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

/// Only the signed-resource path is public; every management path (and any
/// unknown path) must fail closed as management.
#[test]
fn only_signed_resource_routes_are_public() {
    assert_eq!(
        classify_sandbox_route(RESOURCE_ROUTE),
        SandboxRouteAccess::Public
    );
    assert_eq!(
        classify_sandbox_route(HASH_PATH),
        SandboxRouteAccess::Public
    );
    for path in [CONFIG_ROUTE, STATUS_ROUTE, REVOKE_ROUTE] {
        assert_eq!(
            classify_sandbox_route(path),
            SandboxRouteAccess::Management,
            "{path} must not be reachable without a suite session"
        );
    }
    assert_eq!(
        classify_sandbox_route("/api/sandbox/unknown"),
        SandboxRouteAccess::Management
    );
}

/// An anonymous config/status/revoke request is refused on the loopback
/// listener: those paths are absent from its route set entirely (404), while
/// the main router registers config/status/revoke as authenticated-only.
#[test]
fn management_routes_are_absent_from_the_public_listener() {
    assert_eq!(PUBLIC_ROUTES, &[RESOURCE_ROUTE]);
    for path in [CONFIG_ROUTE, STATUS_ROUTE, REVOKE_ROUTE] {
        assert!(
            !PUBLIC_ROUTES.contains(&path),
            "{path} leaked onto the unauthenticated listener"
        );
    }
}

/// #2760 — HTTP-level proof of the listener boundary. The router is built
/// from the very table `public_router()` iterates, so a management path that
/// ever leaks into that table fails this test with a 200 instead of a 404.
/// A stub handler keeps the test independent of `AppState`; the assertion is
/// about which paths are routable, not about resource contents.
#[tokio::test]
async fn public_listener_refuses_management_requests_but_routes_resources() {
    use axum::{body::Body, http::Request, http::StatusCode, routing::get, Router};
    use tower::ServiceExt;

    let mut app = Router::new();
    for path in PUBLIC_ROUTES {
        app = app.route(path, get(|| async { "ok" }));
    }

    let hash = &HASH_PATH[super::RESOURCE_ROUTE_PREFIX.len()..];
    for (method, path) in [
        ("GET", CONFIG_ROUTE.to_string()),
        ("GET", STATUS_ROUTE.replace(":hash", hash)),
        ("POST", REVOKE_ROUTE.replace(":hash", hash)),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(path.as_str())
                    .body(Body::empty())
                    .expect("test request"),
            )
            .await
            .expect("response");
        assert_eq!(
            response.status(),
            StatusCode::NOT_FOUND,
            "{method} {path} must be refused on the sandbox listener"
        );
    }

    let response = app
        .oneshot(
            Request::builder()
                .uri(HASH_PATH)
                .body(Body::empty())
                .expect("test request"),
        )
        .await
        .expect("response");
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "the signed-resource path must stay routable on the sandbox listener"
    );
}

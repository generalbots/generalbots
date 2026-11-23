//! Unit tests for `super` (run_axum_server.rs).
//!
//! Split out of `run_axum_server.rs` to keep the file inside the project size
//! limit; the production code stays beside it. The router itself needs
//! `AppState` and a database pool (out of scope), so the tests cover the
//! fallback handler — the only place where an unmatched request is classified.

use super::*;
use axum::body::to_bytes;
use axum::http::{header, StatusCode, Uri};
use serde_json::Value;

fn uri(path: &str) -> Uri {
    Uri::try_from(path).expect("test path is a valid URI")
}

fn headers() -> axum::http::HeaderMap {
    axum::http::HeaderMap::new()
}

async fn json_body(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body is readable");
    serde_json::from_slice(&bytes).expect("response body is JSON")
}

#[test]
fn the_api_dav_webhook_and_well_known_surfaces_are_machine_facing() {
    for path in [
        "/api/unknown",
        "/api/",
        "/caldav/x",
        "/caldav",
        "/webhook/x",
        "/webhook",
        "/.well-known/x",
    ] {
        assert!(is_machine_facing(path), "{path} must be machine facing");
    }
}

#[test]
fn application_routes_are_not_machine_facing() {
    for path in ["/", "/drive", "/suite/chat", "/cloud/store", "/index.html"] {
        assert!(!is_machine_facing(path), "{path} must serve the shell");
    }
}

#[test]
fn the_api_prefix_requires_the_trailing_slash() {
    // Pins today's behaviour: a bare `/api` is treated as an application route.
    // `/apix/y` must not match either — the check is a prefix, not a substring.
    assert!(!is_machine_facing("/api"));
    assert!(!is_machine_facing("/apix/y"));
}

#[tokio::test]
async fn a_machine_facing_path_reports_404_naming_the_path() {
    for path in ["/api/unknown", "/caldav/x", "/webhook/x", "/.well-known/x"] {
        let response = fallback_response(&uri(path), &headers(), "/nonexistent/desktop.html").await;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = json_body(response).await;
        assert_eq!(body["error"], "not_found");
        assert_eq!(body["path"], path);
    }
}

#[tokio::test]
async fn an_application_path_serves_the_desktop_shell() {
    let dir = tempfile::tempdir().expect("tempdir");
    let shell = dir.path().join("desktop.html");
    std::fs::write(&shell, b"<html>shell</html>").expect("shell is writable");
    let shell_path = shell.to_str().expect("tempdir path is UTF-8");

    let response = fallback_response(&uri("/drive"), &headers(), shell_path).await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers()[header::CONTENT_TYPE]
            .to_str()
            .expect("content type is ASCII"),
        "text/html; charset=utf-8"
    );
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body is readable");
    assert_eq!(&bytes[..], b"<html>shell</html>".as_slice());
}

#[tokio::test]
async fn a_missing_shell_reports_500_instead_of_panicking() {
    let response = fallback_response(&uri("/drive"), &headers(), "/nonexistent/desktop.html").await;

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

/// #2815 — the two anonymous auth POSTs are CSRF-exempt because the JS forms
/// post JSON (no double-submit token). The middleware must still reject a
/// cross-site POST by validating Origin/Referer before the exemption applies.
mod csrf_origin_tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use axum::routing::post;
    use tower::ServiceExt;

    fn manager() -> Arc<CsrfManager> {
        let mut config = CsrfConfig::default();
        config.exempt_paths = vec!["/api/cloud/auth*".to_string()];
        config.origin_check_paths = vec![
            "/api/cloud/auth/login".to_string(),
            "/api/cloud/auth/signup".to_string(),
        ];
        Arc::new(
            CsrfManager::new(config, b"this-is-a-very-long-secret-key-for-testing-csrf")
                .expect("test manager"),
        )
    }

    fn app() -> axum::Router {
        let manager = manager();
        axum::Router::new()
            .route("/api/cloud/auth/login", post(|| async { "ok" }))
            .route("/api/cloud/auth/signup", post(|| async { "ok" }))
            .layer(axum::middleware::from_fn(move |req, next| {
                let manager = manager.clone();
                async move { csrf_middleware(manager, req, next).await }
            }))
    }

    fn post_request(path: &str, origin: &str) -> Request<Body> {
        Request::builder()
            .method("POST")
            .uri(path)
            .header("host", "login.pragmatismo.com.br")
            .header("origin", origin)
            .header("content-type", "application/json")
            .body(Body::from("{}"))
            .expect("test request")
    }

    #[tokio::test]
    async fn a_foreign_origin_is_forbidden_on_login_and_signup() {
        for path in ["/api/cloud/auth/login", "/api/cloud/auth/signup"] {
            let response = app()
                .oneshot(post_request(path, "https://evil.example"))
                .await
                .expect("response");
            assert_eq!(
                response.status(),
                StatusCode::FORBIDDEN,
                "{path} must reject a foreign Origin"
            );
            let body = json_body(response).await;
            assert_eq!(body["error"], "csrf_validation_failed");
        }
    }

    #[tokio::test]
    async fn a_same_origin_post_passes_the_auth_exemption() {
        for path in ["/api/cloud/auth/login", "/api/cloud/auth/signup"] {
            let response = app()
                .oneshot(post_request(path, "https://login.pragmatismo.com.br"))
                .await
                .expect("response");
            assert_eq!(
                response.status(),
                StatusCode::OK,
                "{path} must accept its own origin"
            );
        }
    }
}

/// #2683 — the production exemption list must not cover cookie/session
/// state-changing endpoints. The sheet UI sends `Authorization: Bearer` (the
/// middleware exempts it), so a path exemption would only add a CSRF hole.
mod csrf_exemption_tests {
    use super::*;

    fn production_manager() -> CsrfManager {
        CsrfManager::new(
            build_csrf_config(false),
            b"this-is-a-very-long-secret-key-for-testing-csrf",
        )
        .expect("test manager")
    }

    #[test]
    fn sheet_and_catalog_paths_are_not_exempt() {
        let manager = production_manager();
        for path in [
            "/api/sheet/save",
            "/api/sheet/delete",
            "/api/sheet/cells",
            "/suite/sheet/fragments/charts",
            "/api/catalog",
            "/api/catalog/products",
        ] {
            assert!(
                !manager.is_exempt_path(path),
                "{path} must require the CSRF token for a cookie-auth write"
            );
        }
    }

    #[test]
    fn documented_machine_caller_exemptions_remain() {
        let manager = production_manager();
        for path in [
            "/api/health",
            "/webhook/whatsapp",
            "/webhook/telegram/acme",
            "/forms/contact",
            "/caldav/principal",
        ] {
            assert!(manager.is_exempt_path(path), "{path} must stay exempt");
        }
    }

    #[test]
    fn csrf_stays_enabled_in_no_security_desktop_mode() {
        let config = build_csrf_config(true);
        assert!(config.enabled, "CSRF is never switched off (#2683)");
        assert!(config.no_security_mode, "desktop mode must enable loopback checks");
    }
}

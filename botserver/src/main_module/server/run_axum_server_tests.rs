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

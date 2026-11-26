//! #2958 — route/caller parity guard for the Fraud suite app.
//!
//! `GET /api/fraud/reports` was absent from the fraud router while
//! `botui/ui/suite/fraud/fraud.js` polled it on every authenticated app open,
//! so the console showed a 404. These tests drive the production router
//! factory with a connection-less pool and assert that the path the frontend
//! actually calls is registered and answers 200. An unregistered path is the
//! non-vacuity control: routing still returns 404 where nothing is mounted.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use botfraud::handlers::FRAUD_REPORTS_PATH;
use std::path::PathBuf;
use std::sync::Arc;
use tower::ServiceExt;

/// The handlers answer before touching storage, so a pool that never connects
/// is enough to prove the route is mounted without a running database.
fn app() -> axum::Router {
    let manager =
        diesel::r2d2::ConnectionManager::<diesel::PgConnection>::new("postgres://localhost/unused");
    let pool = diesel::r2d2::Pool::builder()
        .max_size(1)
        .build_unchecked(manager);
    let state = Arc::new(botfraud::FraudState::new(pool));
    botfraud::configure_fraud_routes().with_state(state)
}

async fn request(method: &str, uri: &str) -> (StatusCode, String) {
    let response = app()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(uri)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("router response");
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), 64 * 1024)
        .await
        .expect("body");
    let body = String::from_utf8(bytes.to_vec()).expect("utf-8 body");
    (status, body)
}

#[tokio::test]
async fn reports_list_path_is_mounted_not_404() {
    let (status, body) = request("GET", FRAUD_REPORTS_PATH).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert!(body.contains("\"items\""), "body: {body}");
}

#[tokio::test]
async fn reports_generate_path_is_mounted_not_404() {
    let (status, body) = request("POST", FRAUD_REPORTS_PATH).await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
}

#[tokio::test]
async fn unregistered_fraud_path_still_404s() {
    let (status, _) = request("GET", "/api/fraud/definitely-not-a-route").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[test]
fn suite_fraud_caller_matches_registered_route() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let caller = root.join("../botui/ui/suite/fraud/fraud.js");
    let source = std::fs::read_to_string(&caller).expect("fraud app source");
    let base = source
        .lines()
        .find_map(|line| {
            let value = line.trim().strip_prefix("const API_BASE = ")?;
            value
                .trim()
                .strip_prefix('\'')
                .and_then(|rest| rest.strip_suffix("';"))
        })
        .expect("API_BASE in fraud.js");
    assert!(
        source.contains("fetchData('reports', 'reports')"),
        "reports poll missing"
    );
    assert_eq!(
        format!("{base}/reports"),
        FRAUD_REPORTS_PATH,
        "the Fraud app calls {base}/reports but the router registers {FRAUD_REPORTS_PATH}"
    );
}

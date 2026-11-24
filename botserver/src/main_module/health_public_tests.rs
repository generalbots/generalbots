//! Unit tests for the anonymous `/health` surface (#2756, #2757).

use axum::extract::Query;
use axum::http::StatusCode;
use axum::Json;

use super::{deep_probe_requested, health_check_simple, liveness_payload, HealthQuery};

#[test]
fn deep_probe_flag_is_detected_only_for_one() {
    assert!(deep_probe_requested(Some(1)));
    assert!(!deep_probe_requested(None));
    assert!(!deep_probe_requested(Some(0)));
    assert!(!deep_probe_requested(Some(2)));
    assert!(!deep_probe_requested(Some(255)));
}

#[test]
fn public_liveness_has_no_build_or_admission_telemetry() {
    let body = liveness_payload("ok", Some(true));
    for key in ["commit", "build_date", "admission", "failing", "dependencies"] {
        assert!(body.get(key).is_none(), "public health leaked '{key}'");
    }
    assert_eq!(body["status"], "ok");
    assert_eq!(body["database"], true);
}

#[tokio::test]
async fn anonymous_deep_request_returns_reduced_payload() {
    let (code, Json(body)) = health_check_simple(Query(HealthQuery { deep: Some(1) })).await;
    assert_eq!(code, StatusCode::OK);
    for key in ["commit", "build_date", "admission", "failing", "dependencies"] {
        assert!(
            body.get(key).is_none(),
            "anonymous deep /health leaked '{key}'"
        );
    }

    // The flag must not widen the payload: deep and plain answers are equal.
    let (_, Json(plain)) = health_check_simple(Query(HealthQuery { deep: None })).await;
    assert_eq!(body, plain, "deep flag must not widen the anonymous payload");
}

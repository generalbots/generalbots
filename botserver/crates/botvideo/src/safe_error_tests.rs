//! Offline unit tests for the sanitized error envelope (#2075).
//!
//! Every video route answers failures with `SafeErrorResponse`. The `error`
//! string doubles as the HTTP status discriminator, so an unknown code must
//! degrade to 500 (never to a success status) and the payload must not carry
//! empty optional fields.

use super::SafeErrorResponse;
use axum::http::StatusCode;
use axum::response::IntoResponse;

#[test]
fn the_error_code_selects_the_http_status() {
    let cases = [
        (SafeErrorResponse::bad_request("x"), StatusCode::BAD_REQUEST),
        (
            SafeErrorResponse::validation_error("field", "x"),
            StatusCode::BAD_REQUEST,
        ),
        (SafeErrorResponse::unauthorized(), StatusCode::UNAUTHORIZED),
        (SafeErrorResponse::forbidden(), StatusCode::FORBIDDEN),
        (SafeErrorResponse::not_found("Clip"), StatusCode::NOT_FOUND),
        (
            SafeErrorResponse::internal_error(),
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
        (
            SafeErrorResponse::new("unknown_code", "x"),
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
    ];

    for (body, expected) in cases {
        assert_eq!(body.into_response().status(), expected, "{expected:?}");
    }
}

#[test]
fn not_found_names_the_resource() {
    let body = SafeErrorResponse::not_found("Project");
    assert_eq!(body.error, "not_found");
    assert_eq!(body.message, "Project not found");
}

#[test]
fn validation_errors_carry_the_offending_field() {
    let body = SafeErrorResponse::validation_error("fps", "must be positive");
    assert_eq!(body.message, "must be positive");
    let details = body.details.expect("details map");
    assert_eq!(details.get("field").map(String::as_str), Some("fps"));
}

#[test]
fn the_builders_chain_without_losing_values() {
    let body = SafeErrorResponse::new("bad_request", "nope")
        .with_code("E42")
        .with_request_id("req-1")
        .with_detail("a", "1")
        .with_detail("b", "2");

    assert_eq!(body.code.as_deref(), Some("E42"));
    assert_eq!(body.request_id.as_deref(), Some("req-1"));
    let details = body.details.expect("details map");
    assert_eq!(details.len(), 2);
    assert_eq!(details.get("a").map(String::as_str), Some("1"));
    assert_eq!(details.get("b").map(String::as_str), Some("2"));
}

#[test]
fn optional_fields_are_omitted_from_the_json_body() {
    let body = SafeErrorResponse::internal_error();
    let json = serde_json::to_value(&body).expect("serializable");
    assert_eq!(json["error"], "internal_error");
    assert_eq!(json["message"], "An internal error occurred");
    assert!(json.get("code").is_none());
    assert!(json.get("request_id").is_none());
    assert!(json.get("details").is_none());
}

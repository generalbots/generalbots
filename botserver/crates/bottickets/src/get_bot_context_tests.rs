//! Ticket payload decoding: the create endpoint accepts both JSON (API
//! clients) and urlencoded bodies (HTML forms and HTMX's default encoding).
//! Requiring JSON alone made every HTMX submission fail with 415.
use super::*;

use axum::http::{header, HeaderMap};

fn headers(content_type: Option<&str>) -> HeaderMap {
    let mut map = HeaderMap::new();
    if let Some(value) = content_type {
        map.insert(
            header::CONTENT_TYPE,
            value.parse().expect("valid content type"),
        );
    }
    map
}

#[test]
fn a_json_body_is_decoded() {
    let body = br#"{"subject":"Printer on fire","priority":"urgent"}"#;

    let request = decode_create_ticket(&headers(Some("application/json")), body)
        .expect("json payload must decode");

    assert_eq!(request.subject, "Printer on fire");
    assert_eq!(request.priority.as_deref(), Some("urgent"));
}

#[test]
fn a_json_content_type_with_a_charset_is_still_json() {
    let body = br#"{"subject":"Slow VPN"}"#;

    let request = decode_create_ticket(
        &headers(Some("application/json; charset=utf-8")),
        body,
    )
    .expect("json payload must decode");

    assert_eq!(request.subject, "Slow VPN");
}

#[test]
fn malformed_json_is_a_bad_request() {
    let error = decode_create_ticket(&headers(Some("application/json")), b"{not json")
        .expect_err("malformed json must be rejected");

    assert_eq!(error.0, StatusCode::BAD_REQUEST);
    assert!(error.1.contains("Invalid JSON body"));
}

#[test]
fn a_urlencoded_body_is_decoded() {
    let body = b"subject=Reset+password&priority=high&requester_email=ana%40example.com";

    let request = decode_create_ticket(&headers(None), body).expect("form payload must decode");

    assert_eq!(request.subject, "Reset password");
    assert_eq!(request.priority.as_deref(), Some("high"));
    assert_eq!(request.requester_email.as_deref(), Some("ana@example.com"));
}

#[test]
fn a_missing_subject_in_a_form_body_is_rejected() {
    let error = decode_create_ticket(&headers(Some("application/x-www-form-urlencoded")), b"priority=low")
        .expect_err("a form without a subject must be rejected");

    assert_eq!(error.0, StatusCode::BAD_REQUEST);
    assert!(error.1.contains("Invalid form body"));
}

#[test]
fn a_json_body_without_a_subject_is_rejected() {
    let error = decode_create_ticket(&headers(Some("application/json")), br#"{"priority":"low"}"#)
        .expect_err("json without a subject must be rejected");

    assert_eq!(error.0, StatusCode::BAD_REQUEST);
}

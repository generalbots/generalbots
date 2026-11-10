//! Offline tests for the router's pure helpers (issue #2075, AI wave).
//!
//! `route_candidates` reads the process-wide profile table built from the
//! environment, so only the two decision helpers are exercised here: the
//! message shaping and the transport-error classification that decides how a
//! failed profile is logged.

use super::*;

#[test]
fn an_explicit_message_array_replaces_the_default_pair() {
    let explicit = serde_json::json!([
        { "role": "user", "content": "from the caller" }
    ]);

    assert_eq!(build_messages("system", "user", &explicit), explicit);
}

#[test]
fn an_empty_array_falls_back_to_the_system_user_pair() {
    let cases = [
        serde_json::json!([]),
        serde_json::json!({}),
        serde_json::Value::Null,
        serde_json::json!("not an array"),
    ];

    for params in cases {
        let messages = build_messages("be brief", "hello", &params);
        let array = messages.as_array().expect("array");
        assert_eq!(array.len(), 2, "params {params}");
        assert_eq!(array[0]["role"], "system");
        assert_eq!(array[0]["content"], "be brief");
        assert_eq!(array[1]["role"], "user");
        assert_eq!(array[1]["content"], "hello");
    }
}

#[test]
fn default_messages_keep_multibyte_content_intact() {
    // `sanitize_utf8` exists to drop lone surrogates; a Rust `&str` can never
    // contain one, so the contract is "content passes through unchanged".
    let messages = build_messages("sistema 🚀", "olá, mundo", &serde_json::json!(null));
    let array = messages.as_array().expect("array");
    assert_eq!(array[0]["content"], "sistema 🚀");
    assert_eq!(array[1]["content"], "olá, mundo");
}

#[test]
fn transport_errors_are_classified_for_the_operator_log() {
    let cases = [
        ("HTTP 429 Too Many Requests", "rate limited"),
        ("429", "rate limited"),
        ("HTTP 500 internal", "server error"),
        ("502 bad gateway", "server error"),
        ("503 unavailable", "server error"),
        ("504 timeout", "server error"),
        ("connection refused", "transport error"),
        ("error decoding response body", "transport error"),
        ("", "transport error"),
    ];

    for (detail, expected) in cases {
        assert_eq!(classify_error(detail), expected, "detail {detail:?}");
    }
}

#[test]
fn rate_limiting_wins_over_a_server_error_code_in_the_same_message() {
    // The first matching rule is the documented precedence.
    assert_eq!(classify_error("429 then 500"), "rate limited");
}

#[test]
fn a_route_meta_records_the_serving_profile() {
    let meta = RouteMeta {
        profile_id: "p1".to_string(),
        tokens_in: 12,
        tokens_out: 34,
        fallback_used: true,
    };

    assert_eq!(meta.profile_id, "p1");
    assert_eq!(meta.tokens_in, 12);
    assert_eq!(meta.tokens_out, 34);
    assert!(meta.fallback_used);
    assert_eq!(meta, meta.clone());
}

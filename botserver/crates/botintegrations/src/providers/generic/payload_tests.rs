//! Unit tests for the generic engine's request-body and outcome shaping.
//!
//! These functions decide what is sent to a provider and how much of the
//! response reaches the chat surface, so every branch is asserted here without
//! touching the network.

use super::*;
use serde_json::json;

fn read_action() -> ActionSpec {
    ActionSpec {
        key: "widgets.items.list",
        method: "GET",
        path: "/items/{item_id}",
        summary: "List items.",
        path_params: &["item_id"],
        query: &[("limit", "limit")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[
            ParamSpec {
                name: "item_id",
                kind: ParamKind::Str,
                required: true,
            },
            ParamSpec {
                name: "limit",
                kind: ParamKind::Str,
                required: false,
            },
        ],
    }
}

fn write_action() -> ActionSpec {
    ActionSpec {
        key: "widgets.items.create",
        method: "POST",
        path: "/items",
        summary: "Create item.",
        path_params: &[],
        query: &[],
        body_param: Some("payload"),
        body_wrapper: None,
        risk: Risk::Medium,
        params: &[ParamSpec {
            name: "payload",
            kind: ParamKind::Json,
            required: true,
        }],
    }
}

fn wrapped_write_action() -> ActionSpec {
    ActionSpec {
        body_wrapper: Some("ticket"),
        ..write_action()
    }
}

fn action_with_path(path: &'static str) -> ActionSpec {
    ActionSpec {
        path,
        path_params: &[],
        ..read_action()
    }
}

fn body_json(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).expect("body is JSON")
}

#[test]
fn an_action_without_a_body_param_sends_no_body() {
    assert_eq!(build_body(&read_action(), &json!({})), Ok(None));
    // Even a stray payload is ignored: the action declares no body field.
    assert_eq!(build_body(&read_action(), &json!({"payload": {"a": 1}})), Ok(None));
}

#[test]
fn a_body_param_must_be_a_json_object() {
    let error = build_body(&write_action(), &json!({"payload": "text"})).expect_err("rejected");
    assert!(error.contains("payload"), "error names the param: {error}");
    assert!(build_body(&write_action(), &json!({})).is_err());
    assert!(build_body(&write_action(), &json!({"payload": [1, 2]})).is_err());
    assert!(build_body(&write_action(), &json!({"payload": null})).is_err());
}

#[test]
fn a_body_object_is_forwarded_verbatim() {
    let params = json!({"payload": {"subject": "hi", "count": 2}});
    let body = build_body(&write_action(), &params).expect("built").expect("present");
    assert_eq!(body_json(&body), json!({"subject": "hi", "count": 2}));
}

#[test]
fn a_body_wrapper_nests_the_payload_under_one_key() {
    let params = json!({"payload": {"subject": "hi"}});
    let body = build_body(&wrapped_write_action(), &params)
        .expect("built")
        .expect("present");
    assert_eq!(body_json(&body), json!({"ticket": {"subject": "hi"}}));
}

#[test]
fn flat_body_puts_the_credential_first_and_copies_declared_params() {
    let credentials = json!({"api_key": "secret"});
    let params = json!({"item_id": "i-1", "limit": "5", "undeclared": "ignored"});
    let body = flat_body(&credentials, "api_key", &read_action(), &params).expect("built");
    assert_eq!(
        body_json(&body),
        json!({"api_key": "secret", "item_id": "i-1", "limit": "5"})
    );
}

#[test]
fn flat_body_skips_null_and_blank_params() {
    let credentials = json!({"api_key": "secret"});
    let params = json!({"item_id": null, "limit": "   "});
    let body = flat_body(&credentials, "api_key", &read_action(), &params).expect("built");
    assert_eq!(body_json(&body), json!({"api_key": "secret"}));
}

#[test]
fn flat_body_requires_the_credential_field() {
    let error = flat_body(&json!({}), "api_key", &read_action(), &json!({}))
        .expect_err("credential missing");
    assert!(error.contains("api_key"), "error names the field: {error}");
    assert!(flat_body(&json!({"api_key": "  "}), "api_key", &read_action(), &json!({})).is_err());
}

#[test]
fn a_short_list_is_preserved_and_counted_for_reads() {
    let items: Vec<Value> = (0..3).map(|i| json!({"id": i})).collect();
    let raw = serde_json::to_vec(&items).expect("serialised");
    let outcome = shape_outcome(&read_action(), 200, &raw);
    assert!(!outcome.truncated);
    assert_eq!(outcome.data.as_array().map(Vec::len), Some(3));
    assert_eq!(outcome.summary, "List items. 3 items (status 200)");
}

#[test]
fn a_long_list_is_capped_and_flagged() {
    let items: Vec<Value> = (0..MAX_LIST_ITEMS + 5).map(|i| json!({"id": i})).collect();
    let raw = serde_json::to_vec(&items).expect("serialised");
    let outcome = shape_outcome(&read_action(), 200, &raw);
    assert!(outcome.truncated);
    assert_eq!(outcome.data.as_array().map(Vec::len), Some(MAX_LIST_ITEMS));
    assert_eq!(
        outcome.summary,
        format!("List items. {} items (status 200)", MAX_LIST_ITEMS)
    );
}

#[test]
fn write_actions_never_gain_a_count_suffix() {
    let items: Vec<Value> = (0..3).map(|i| json!({"id": i})).collect();
    let raw = serde_json::to_vec(&items).expect("serialised");
    let outcome = shape_outcome(&write_action(), 201, &raw);
    assert_eq!(outcome.summary, "Create item. (status 201)");
    assert!(!outcome.summary.contains("items"));
}

#[test]
fn every_oversized_array_inside_an_object_is_capped() {
    let big: Vec<Value> = (0..MAX_LIST_ITEMS + 10).map(|i| json!({"id": i})).collect();
    let raw = serde_json::to_vec(&json!({"tickets": big, "users": big, "total": 1}))
        .expect("serialised");
    let outcome = shape_outcome(&read_action(), 200, &raw);

    assert!(outcome.truncated);
    let map = outcome.data.as_object().expect("object preserved");
    assert_eq!(map["tickets"].as_array().map(Vec::len), Some(MAX_LIST_ITEMS));
    assert_eq!(map["users"].as_array().map(Vec::len), Some(MAX_LIST_ITEMS));
    assert_eq!(map["total"], json!(1));
    // An object payload has no item count to report.
    assert_eq!(outcome.summary, "List items. (status 200)");
}

#[test]
fn an_object_with_small_arrays_is_not_flagged_as_truncated() {
    let raw = serde_json::to_vec(&json!({"items": [1, 2]})).expect("serialised");
    let outcome = shape_outcome(&read_action(), 200, &raw);
    assert!(!outcome.truncated);
    assert_eq!(outcome.data, json!({"items": [1, 2]}));
}

#[test]
fn a_scalar_payload_is_preserved() {
    let outcome = shape_outcome(&read_action(), 200, b"\"ok\"");
    assert_eq!(outcome.data, json!("ok"));
    assert!(!outcome.truncated);
}

#[test]
fn a_non_json_payload_degrades_to_null_without_failing_the_call() {
    let outcome = shape_outcome(&read_action(), 502, b"<html>bad gateway</html>");
    assert_eq!(outcome.data, Value::Null);
    assert!(!outcome.truncated);
    assert_eq!(outcome.summary, "List items. (status 502)");
}

#[test]
fn the_status_is_always_present_in_the_summary() {
    for status in [200u16, 201, 204, 400, 401, 429, 500] {
        let outcome = shape_outcome(&read_action(), status, b"[]");
        assert!(
            outcome.summary.contains(&format!("(status {status})")),
            "status {status} missing from {:?}",
            outcome.summary
        );
    }
}

#[test]
fn path_placeholders_are_encoded_exactly_once() {
    let url = build_url_from_parts(
        "https://api.widgets.test/v1",
        &read_action(),
        &json!({"item_id": "a b/c"}),
        vec![],
    )
    .expect("built");
    assert_eq!(url, "https://api.widgets.test/v1/items/a%20b%2Fc");
    assert!(!url.contains("%252F"), "double-encoded: {url}");
}

#[test]
fn a_missing_path_placeholder_is_rejected_before_any_request() {
    assert!(build_url_from_parts(
        "https://api.widgets.test/v1",
        &read_action(),
        &json!({}),
        vec![]
    )
    .is_err());
    assert!(build_url_from_parts(
        "https://api.widgets.test/v1",
        &read_action(),
        &json!({"item_id": "  "}),
        vec![]
    )
    .is_err());
}

#[test]
fn query_pairs_glue_with_a_question_mark_on_a_clean_path() {
    let url = build_url_from_parts(
        "https://api.widgets.test/v1",
        &read_action(),
        &json!({"item_id": "i-1"}),
        vec![("limit".to_string(), "5".to_string())],
    )
    .expect("built");
    assert_eq!(url, "https://api.widgets.test/v1/items/i-1?limit=5");
}

#[test]
fn query_pairs_glue_with_an_ampersand_when_the_path_already_has_a_query() {
    let url = build_url_from_parts(
        "https://api.widgets.test/v1",
        &action_with_path("/items?fixed=1"),
        &json!({}),
        vec![("limit".to_string(), "5".to_string())],
    )
    .expect("built");
    assert_eq!(url, "https://api.widgets.test/v1/items?fixed=1&limit=5");
}

#[test]
fn query_names_and_values_are_both_encoded() {
    let url = build_url_from_parts(
        "https://api.widgets.test/v1",
        &read_action(),
        &json!({"item_id": "i-1"}),
        vec![("q name".to_string(), "a&b".to_string())],
    )
    .expect("built");
    assert_eq!(url, "https://api.widgets.test/v1/items/i-1?q%20name=a%26b");
}

#[test]
fn no_query_pairs_means_no_trailing_separator() {
    let url = build_url_from_parts(
        "https://api.widgets.test/v1",
        &read_action(),
        &json!({"item_id": "i-1"}),
        vec![],
    )
    .expect("built");
    assert_eq!(url, "https://api.widgets.test/v1/items/i-1");
    assert!(!url.ends_with('?'));
    assert!(!url.ends_with('&'));
}

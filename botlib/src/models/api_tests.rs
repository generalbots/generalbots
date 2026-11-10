//! Contract tests for `ApiResponse` — the envelope every HTTP handler returns
//! (#2064). `map` must carry the envelope metadata across the type change, and
//! absent fields must disappear from the wire format, not serialize as `null`.

use super::ApiResponse;

#[test]
fn success_sets_data_and_clears_error() {
    let response = ApiResponse::success(1_i32);
    assert!(response.is_success());
    assert!(!response.is_error());
    assert_eq!(response.data, Some(1));
    assert!(response.error.is_none());
    assert!(response.message.is_none());
    assert!(response.code.is_none());
}

#[test]
fn error_sets_message_and_leaves_data_empty() {
    let response: ApiResponse<i32> = ApiResponse::error("boom");
    assert!(response.is_error());
    assert_eq!(response.error.as_deref(), Some("boom"));
    assert!(response.data.is_none());
    assert!(response.code.is_none());
}

#[test]
fn error_with_code_keeps_the_code() {
    let response: ApiResponse<i32> = ApiResponse::error_with_code("denied", "E_FORBIDDEN");
    assert_eq!(response.code.as_deref(), Some("E_FORBIDDEN"));
    assert_eq!(response.error.as_deref(), Some("denied"));
}

#[test]
fn map_transforms_the_data_and_preserves_the_envelope() {
    let response = ApiResponse::success_with_message(1_i32, "created")
        .map(|n| n + 1);
    assert!(response.is_success());
    assert_eq!(response.data, Some(2));
    assert_eq!(response.message.as_deref(), Some("created"));
}

#[test]
fn map_over_an_error_keeps_data_none_and_the_code() {
    let response: ApiResponse<i32> =
        ApiResponse::<i32>::error_with_code("bad", "E_BAD").map(|n| n + 1);
    assert!(!response.success);
    assert!(response.data.is_none());
    assert_eq!(response.code.as_deref(), Some("E_BAD"));
    assert_eq!(response.error.as_deref(), Some("bad"));
}

#[test]
fn default_is_a_success_with_the_default_value() {
    let response: ApiResponse<Vec<String>> = ApiResponse::default();
    assert!(response.is_success());
    assert_eq!(response.data, Some(Vec::new()));
}

#[test]
fn absent_fields_are_omitted_from_json() {
    let json = serde_json::to_value(ApiResponse::success(7_i32)).expect("serializable");
    assert_eq!(json, serde_json::json!({"success": true, "data": 7}));

    let error: ApiResponse<i32> = ApiResponse::error("x");
    let json = serde_json::to_value(error).expect("serializable");
    assert_eq!(json, serde_json::json!({"success": false, "error": "x"}));
}

#[test]
fn a_response_round_trips_through_json() {
    let original = ApiResponse::success_with_message(vec![1_i32, 2], "ok");
    let encoded = serde_json::to_string(&original).expect("serializable");
    let decoded: ApiResponse<Vec<i32>> = serde_json::from_str(&encoded).expect("deserializable");
    assert_eq!(decoded.success, original.success);
    assert_eq!(decoded.data, original.data);
    assert_eq!(decoded.message, original.message);
}

//! Offline tests for the OpenAI-compatible client's pure helpers
//! (issue #2075, AI wave): token estimation, history truncation and message
//! shaping. Nothing here opens a socket or reads the environment.

use super::*;

fn message(role: &str, content: &str) -> Value {
    serde_json::json!({ "role": role, "content": content })
}

fn roles(value: &Value) -> Vec<String> {
    value
        .as_array()
        .expect("array")
        .iter()
        .map(|m| m["role"].as_str().unwrap_or("").to_string())
        .collect()
}

#[test]
fn token_estimate_is_bytes_over_four_rounded_up() {
    let cases = [
        ("", 0),
        ("a", 1),
        ("abcd", 1),
        ("abcde", 2),
        ("12345678", 2),
        ("123456789", 3),
    ];

    for (text, expected) in cases {
        assert_eq!(OpenAIClient::estimate_tokens(text), expected, "text {text:?}");
    }
}

#[test]
fn message_token_estimate_sums_only_string_contents() {
    let messages = serde_json::json!([
        message("system", "abcd"),
        message("user", "12345678"),
        { "role": "assistant", "tool_calls": [] },
        { "content": 42 },
        "not an object",
    ]);

    assert_eq!(OpenAIClient::estimate_messages_tokens(&messages), 3);
}

#[test]
fn message_token_estimate_of_a_non_array_is_zero() {
    for value in [serde_json::json!({}), Value::Null, serde_json::json!("text")] {
        assert_eq!(OpenAIClient::estimate_messages_tokens(&value), 0);
    }
}

#[test]
fn truncation_keeps_every_system_message_that_fits() {
    let messages = serde_json::json!([
        message("system", "abcd"),
        message("system", "efgh"),
        message("user", "ijkl"),
    ]);

    let truncated = OpenAIClient::truncate_messages(&messages, 100);
    assert_eq!(roles(&truncated), vec!["system", "system", "user"]);
    assert_eq!(truncated.as_array().map(Vec::len), Some(3));
}

#[test]
fn truncation_keeps_the_newest_turns_in_chronological_order() {
    let messages = serde_json::json!([
        message("system", "abcd"),
        message("user", "older question"),
        message("assistant", "older answer"),
        message("user", "newer question"),
    ]);

    // Room for the system message plus the two newest turns only.
    let truncated = OpenAIClient::truncate_messages(&messages, 8);
    assert_eq!(roles(&truncated), vec!["system", "assistant", "user"]);
    assert_eq!(truncated[1]["content"], "older answer");
    assert_eq!(truncated[2]["content"], "newer question");
}

#[test]
fn truncation_drops_a_system_message_that_does_not_fit() {
    let messages = serde_json::json!([
        message("system", "0123456789abcdef"),
        message("user", "hi"),
    ]);

    let truncated = OpenAIClient::truncate_messages(&messages, 3);
    assert_eq!(roles(&truncated), vec!["user"]);
}

#[test]
fn truncation_of_a_non_array_yields_an_empty_array() {
    let truncated = OpenAIClient::truncate_messages(&serde_json::json!({}), 100);
    assert_eq!(truncated, serde_json::json!([]));
}

#[test]
fn messages_under_the_limit_are_returned_untouched() {
    let messages = serde_json::json!([message("user", "hi")]);
    assert_eq!(OpenAIClient::ensure_token_limit(&messages, 1_000), messages);
}

#[test]
fn messages_over_the_safe_limit_are_truncated() {
    // 10 % headroom: a 100-token window admits 90 estimated tokens.
    let long = "x".repeat(400); // 100 tokens
    let messages = serde_json::json!([message("user", &long), message("user", "ok")]);

    let limited = OpenAIClient::ensure_token_limit(&messages, 100);
    assert_eq!(roles(&limited), vec!["user"]);
    assert_eq!(limited[0]["content"], "ok");
}

#[test]
fn history_roles_are_mapped_to_the_wire_vocabulary() {
    let history = vec![
        ("user".to_string(), "u".to_string()),
        ("assistant".to_string(), "a".to_string()),
        ("system".to_string(), "s".to_string()),
        ("developer".to_string(), "d".to_string()),
        ("tool".to_string(), "t".to_string()),
        ("episodic".to_string(), "e".to_string()),
        ("compact".to_string(), "c".to_string()),
        ("unknown-role".to_string(), "x".to_string()),
    ];

    let built = OpenAIClient::build_messages("prompt", "context", &history);
    assert_eq!(
        roles(&built),
        vec![
            "system",
            "system",
            "user",
            "assistant",
            "system",
            "developer",
            "tool",
            "system",
            "system",
            "system",
        ]
    );
    assert_eq!(built[0]["content"], "prompt");
    assert_eq!(built[1]["content"], "context");
}

#[test]
fn empty_prompt_and_context_are_omitted_from_the_message_list() {
    let history = vec![("user".to_string(), "hello".to_string())];

    let built = OpenAIClient::build_messages("", "", &history);
    assert_eq!(roles(&built), vec!["user"]);
    assert_eq!(built[0]["content"], "hello");
}

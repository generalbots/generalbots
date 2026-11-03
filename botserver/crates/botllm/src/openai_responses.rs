//! ChatGPT plan Responses client (#1653).
//!
//! Subscription tokens use the ChatGPT backend (`chatgpt.tokens.use.direct`),
//! not `api.openai.com`. The OAuth exchange lives in the botserver account
//! module; this file owns the request shape and the output extraction so both
//! the chat pipeline and tests share one implementation.

use serde_json::Value;

/// Default backend used by the ChatGPT plan client flow.
pub const DEFAULT_BASE_URL: &str = "https://chatgpt.com/backend-api/codex";
/// Model used when the bot has no explicit subscription model.
pub const DEFAULT_MODEL: &str = "gpt-5-codex";

/// Extracts the assistant text from a Responses API payload. The endpoint
/// returns `output_text` on some versions and an `output` array on others.
pub fn extract_output_text(body: &Value) -> Option<String> {
    if let Some(text) = body.get("output_text").and_then(Value::as_str) {
        if !text.trim().is_empty() {
            return Some(text.to_string());
        }
    }
    let mut parts: Vec<String> = Vec::new();
    for item in body.get("output").and_then(Value::as_array).into_iter().flatten() {
        for content in item.get("content").and_then(Value::as_array).into_iter().flatten() {
            if let Some(text) = content.get("text").and_then(Value::as_str) {
                if !text.trim().is_empty() {
                    parts.push(text.to_string());
                }
            }
        }
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("\n"))
    }
}

/// Extracts streamed deltas from one SSE `data:` payload.
pub fn extract_stream_delta(body: &Value) -> Option<String> {
    if body.get("type").and_then(Value::as_str) == Some("response.output_text.delta") {
        return body.get("delta").and_then(Value::as_str).map(str::to_string);
    }
    body.get("delta").and_then(Value::as_str).map(str::to_string)
}

/// Builds the JSON request body. `messages` is the chat transcript; when it is
/// not an array the prompt is wrapped as one user turn.
pub fn request_body(model: &str, prompt: &str, messages: &Value) -> Value {
    let input = if messages.is_array() && !messages.as_array().map(|a| a.is_empty()).unwrap_or(true) {
        messages.clone()
    } else {
        serde_json::json!([{ "role": "user", "content": prompt }])
    };
    serde_json::json!({
        "model": model,
        "input": input,
        "stream": false,
        "store": false
    })
}

/// Sends one non-streaming turn to the ChatGPT backend.
pub async fn generate(
    client: &reqwest::Client,
    base_url: &str,
    token: &str,
    account_id: Option<&str>,
    model: &str,
    prompt: &str,
    messages: &Value,
) -> Result<String, String> {
    if token.trim().is_empty() {
        return Err("ChatGPT plan token is empty".to_string());
    }
    let url = format!("{}/responses", base_url.trim_end_matches('/'));
    let mut request = client
        .post(&url)
        .header("Authorization", format!("Bearer {token}"))
        .header("Content-Type", "application/json")
        .header("OpenAI-Beta", "responses=experimental")
        .header("originator", "general-bots");
    if let Some(account) = account_id.filter(|value| !value.trim().is_empty()) {
        request = request.header("chatgpt-account-id", account);
    }
    let response = request
        .json(&request_body(model, prompt, messages))
        .send()
        .await
        .map_err(|error| format!("ChatGPT plan request failed: {error}"))?;
    let status = response.status();
    let body: Value = response
        .json()
        .await
        .map_err(|error| format!("ChatGPT plan response was not JSON: {error}"))?;
    if !status.is_success() {
        let message = body
            .get("error")
            .and_then(|value| value.get("message"))
            .and_then(Value::as_str)
            .unwrap_or("ChatGPT plan rejected the request");
        return Err(format!("ChatGPT plan error ({status}): {message}"));
    }
    extract_output_text(&body)
        .ok_or_else(|| "ChatGPT plan returned no assistant text".to_string())
}

#[cfg(test)]
mod tests {
    use super::{extract_output_text, extract_stream_delta, request_body};

    #[test]
    fn output_text_field_wins() {
        let body = serde_json::json!({ "output_text": "hello" });
        assert_eq!(extract_output_text(&body).as_deref(), Some("hello"));
    }

    #[test]
    fn output_array_is_joined() {
        let body = serde_json::json!({
            "output": [
                { "content": [{ "type": "output_text", "text": "one" }] },
                { "content": [{ "type": "output_text", "text": "two" }] }
            ]
        });
        assert_eq!(extract_output_text(&body).as_deref(), Some("one\ntwo"));
    }

    #[test]
    fn empty_output_is_none() {
        assert!(extract_output_text(&serde_json::json!({ "output": [] })).is_none());
    }

    #[test]
    fn stream_delta_reads_the_delta_field() {
        let event = serde_json::json!({ "type": "response.output_text.delta", "delta": "hi" });
        assert_eq!(extract_stream_delta(&event).as_deref(), Some("hi"));
    }

    #[test]
    fn the_prompt_becomes_a_user_turn_without_messages() {
        let body = request_body("gpt-5-codex", "hello", &serde_json::Value::Null);
        assert_eq!(body["model"], "gpt-5-codex");
        assert_eq!(body["input"][0]["content"], "hello");
    }
}

//! `POST /api/llm/config/test` — one-shot connectivity check (#2561).

use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::Json,
};
use botcore::shared::state::AppState;
use std::sync::Arc;
use botsecurity::auth::AuthenticatedUser;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::Instant;

use super::{api_error, require_admin, ApiResult};

#[derive(Deserialize)]
pub(super) struct TestRequest {
    pub endpoint: String,
    pub model: String,
    pub api_key: Option<String>,
}

#[derive(Serialize)]
struct TestResult {
    reachable: bool,
    latency_ms: u64,
    model_responded: bool,
    error: Option<String>,
    /// Fixed code when the provider sent a message (the message itself is
    /// logged server-side, never returned to the client — see `safe_detail`).
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
}

/// Anthropic answers in its own format (`/v1/messages`, `x-api-key`), which is
/// also what the chat uses; testing its OpenAI-compatible door instead failed
/// with HTTP 400 for keys that chat fine.
fn anthropic_messages_url(endpoint: &str) -> Option<String> {
    let url = reqwest::Url::parse(endpoint).ok()?;
    let host = url.host_str()?.to_ascii_lowercase();
    if host == "anthropic.com" || host.ends_with(".anthropic.com") {
        Some(format!("{}://{}/v1/messages", url.scheme(), host))
    } else {
        None
    }
}

/// `error.message`, `error` or `message` from a provider's JSON error body.
/// The result is for SERVER-SIDE logging only: raw upstream text must never be
/// returned to a client (#security). Shortened to keep logs bounded.
pub(super) fn provider_message(body: &str) -> Option<String> {
    let v: Value = serde_json::from_str(body).ok()?;
    let msg = v
        .pointer("/error/message")
        .and_then(Value::as_str)
        .or_else(|| v.get("error").and_then(Value::as_str))
        .or_else(|| v.get("message").and_then(Value::as_str))
        .or_else(|| v.pointer("/0/error/message").and_then(Value::as_str))?;
    Some(msg.chars().take(300).collect())
}

/// Logs the provider's own message through the shared sanitizer and returns a
/// fixed, non-sensitive code for the HTTP response.
fn safe_detail(body: &str) -> Option<String> {
    let raw = provider_message(body)?;
    botsecurity::error_sanitizer::log_and_sanitize_str(&raw, "model_config::test_endpoint::provider_detail", None);
    Some("provider_error_logged".to_string())
}

pub(super) async fn handle_test(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<TestRequest>,
) -> ApiResult {
    require_admin(&user)?;

    let endpoint = req.endpoint.trim();
    if endpoint.is_empty() {
        return Err(api_error(StatusCode::BAD_REQUEST, "endpoint_required"));
    }
    if reqwest::Url::parse(endpoint).is_err() {
        return Err(api_error(StatusCode::BAD_REQUEST, "invalid_endpoint_url"));
    }

    let client = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(10))
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|_| api_error(StatusCode::INTERNAL_SERVER_ERROR, "http_client_failed"))?;

    // No key typed: test with the key already stored for this provider.
    let stored = if req.api_key.as_deref().map(str::trim).unwrap_or("").is_empty() {
        super::accounts::configured_key(&state, endpoint)
    } else {
        String::new()
    };
    let key = req.api_key.as_deref().map(str::trim).filter(|k| !k.is_empty()).unwrap_or(stored.as_str());
    let anthropic = anthropic_messages_url(endpoint);
    let request = if let Some(url) = &anthropic {
        let body = json!({
            "model": req.model.trim(),
            "max_tokens": 16,
            "messages": [{"role": "user", "content": "Say ok."}],
        });
        client
            .post(url)
            .header("x-api-key", key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
    } else {
        let body = json!({
            "model": req.model.trim(),
            "messages": [{"role": "user", "content": "Say ok."}],
            "max_tokens": 16,
        });
        let mut request = client.post(endpoint).json(&body);
        if !key.is_empty() {
            request = request.bearer_auth(key);
        }
        request
    };

    let started = Instant::now();
    let response = request.send().await;
    let latency_ms = started.elapsed().as_millis() as u64;

    match response {
        Ok(resp) => {
            let status = resp.status();
            if status.is_success() {
                let body_text = resp.text().await.unwrap_or_else(|e| {
                    log::warn!("[test_endpoint] cannot read response body: {e}");
                    String::new()
                });
                let parsed: Result<Value, _> = serde_json::from_str(&body_text);
                let answer_field = if anthropic.is_some() { "content" } else { "choices" };
                let model_responded = parsed
                    .as_ref()
                    .ok()
                    .and_then(|v| v.get(answer_field))
                    .and_then(Value::as_array)
                    .is_some_and(|a| !a.is_empty());
                Ok(Json(json!(TestResult {
                    reachable: true,
                    latency_ms,
                    model_responded,
                    error: if model_responded {
                        None
                    } else {
                        Some("endpoint_responded_but_no_choices".to_string())
                    },
                    detail: None,
                })))
            } else {
                let body_text = resp.text().await.unwrap_or_else(|e| {
                    log::warn!("[test_endpoint] cannot read response body: {e}");
                    String::new()
                });
                Ok(Json(json!(TestResult {
                    reachable: true,
                    latency_ms,
                    model_responded: false,
                    error: Some(format!("http_status_{}", status.as_u16())),
                    detail: safe_detail(&body_text),
                })))
            }
        }
        Err(e) => {
            let reason = if e.is_connect() {
                "connection_refused".to_string()
            } else if e.is_timeout() {
                "timeout".to_string()
            } else {
                "request_failed".to_string()
            };
            Ok(Json(json!(TestResult {
                reachable: false,
                latency_ms,
                model_responded: false,
                error: Some(reason),
                detail: None,
            })))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anthropic_is_tested_on_its_own_messages_door() {
        assert_eq!(
            anthropic_messages_url("https://api.anthropic.com").as_deref(),
            Some("https://api.anthropic.com/v1/messages")
        );
        assert_eq!(
            anthropic_messages_url("https://api.anthropic.com/v1/chat/completions").as_deref(),
            Some("https://api.anthropic.com/v1/messages")
        );
        assert!(anthropic_messages_url("https://api.openai.com/v1/chat/completions").is_none());
        assert!(anthropic_messages_url("https://anthropic.com.evil.example/v1").is_none());
    }

    #[test]
    fn provider_error_messages_are_read() {
        let anthropic = r#"{"type":"error","error":{"type":"invalid_request_error","message":"model: claude-x"}}"#;
        assert_eq!(provider_message(anthropic).as_deref(), Some("model: claude-x"));
        let google = r#"[{"error":{"code":400,"message":"API key not valid."}}]"#;
        assert_eq!(provider_message(google).as_deref(), Some("API key not valid."));
        assert!(provider_message("not json").is_none());
    }
}

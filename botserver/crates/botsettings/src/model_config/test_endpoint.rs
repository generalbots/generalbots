//! `POST /api/llm/config/test` — one-shot connectivity check (#2561).

use axum::{extract::Extension, http::StatusCode, response::Json};
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
}

pub(super) async fn handle_test(
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

    let body = json!({
        "model": req.model.trim(),
        "messages": [{"role": "user", "content": "Say ok."}],
        "max_tokens": 4,
        "temperature": 0,
    });

    let mut request = client.post(endpoint).json(&body);
    if let Some(key) = &req.api_key {
        let trimmed = key.trim();
        if !trimmed.is_empty() {
            request = request.bearer_auth(trimmed);
        }
    }

    let started = Instant::now();
    let response = request.send().await;
    let latency_ms = started.elapsed().as_millis() as u64;

    match response {
        Ok(resp) => {
            let status = resp.status();
            if status.is_success() {
                let body_text = resp.text().await.unwrap_or_default();
                let parsed: Result<Value, _> = serde_json::from_str(&body_text);
                let model_responded = parsed
                    .as_ref()
                    .ok()
                    .and_then(|v| v.get("choices"))
                    .and_then(|c| c.as_array())
                    .map(|a| !a.is_empty())
                    .unwrap_or(false);
                Ok(Json(json!(TestResult {
                    reachable: true,
                    latency_ms,
                    model_responded,
                    error: if model_responded {
                        None
                    } else {
                        Some("endpoint_responded_but_no_choices".to_string())
                    },
                })))
            } else {
                Ok(Json(json!(TestResult {
                    reachable: true,
                    latency_ms,
                    model_responded: false,
                    error: Some(format!("http_status_{}", status.as_u16())),
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
            })))
        }
    }
}

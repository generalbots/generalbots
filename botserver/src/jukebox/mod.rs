use axum::{
    body::Body,
    extract::{Path, Query},
    http::{header, Response, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Debug, Deserialize)]
struct AudioQuery {
    path: String,
}

fn valid_audio_path(path: &str) -> bool {
    if path.is_empty() || path.len() > 4096 || path.contains('\0') {
        return false;
    }
    if path
        .split(['/', '\\'])
        .any(|segment| segment == "..")
    {
        return false;
    }
    let bytes = path.as_bytes();
    let windows_absolute = bytes.len() > 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && (bytes[2] == b'/' || bytes[2] == b'\\');
    path.starts_with('/') || windows_absolute
}

fn botmodels_url() -> String {
    std::env::var("BOTMODELS_HOST")
        .or_else(|_| std::env::var("BOTMODELS_URL"))
        .unwrap_or_else(|_| "http://localhost:8085".to_string())
        .trim_end_matches('/')
        .to_string()
}

fn botmodels_key() -> String {
    std::env::var("BOTMODELS_API_KEY").unwrap_or_else(|_| "starter".to_string())
}

fn client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(35))
        .build()
        .map_err(|error| format!("Could not create BotModels client: {error}"))
}

// The health probe must not hold the app open for the full 35s generation
// timeout when the optional ACE-Step backend is not running (#2951).
fn health_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .map_err(|error| format!("Could not create Jukebox health client: {error}"))
}

fn status_from_upstream(status: reqwest::StatusCode) -> StatusCode {
    StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::BAD_GATEWAY)
}

fn proxy_error(context: &str, error: &dyn std::fmt::Display) -> axum::response::Response {
    tracing::error!(%error, "{context}");
    (
        StatusCode::BAD_GATEWAY,
        Json(json!({
            "error": "Music generation service is unavailable",
            "context": context,
        })),
    )
        .into_response()
}

async fn json_response(request: reqwest::RequestBuilder) -> axum::response::Response {
    let upstream = match request.send().await {
        Ok(response) => response,
        Err(error) => return proxy_error("BotModels request failed", &error),
    };
    let status = status_from_upstream(upstream.status());
    let content_type = upstream
        .headers()
        .get(header::CONTENT_TYPE)
        .cloned()
        .unwrap_or_else(|| header::HeaderValue::from_static("application/json"));
    let bytes = match upstream.bytes().await {
        Ok(bytes) => bytes,
        Err(error) => return proxy_error("BotModels response could not be read", &error),
    };

    let mut builder = Response::builder().status(status);
    builder = builder.header(header::CONTENT_TYPE, content_type);
    match builder.body(Body::from(bytes)) {
        Ok(response) => response,
        Err(error) => proxy_error("BotModels response could not be built", &error),
    }
}

async fn generate(Json(mut payload): Json<Value>) -> axum::response::Response {
    // JukeBox is an instrumental radio. Enforce this at the trusted API boundary so
    // a modified client cannot accidentally request vocals or submit lyrics.
    let object = match payload.as_object_mut() {
        Some(object) => object,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid music generation request" })),
            )
                .into_response();
        }
    };
    object.insert("instrumental".to_string(), Value::Bool(true));
    object.insert("lyrics".to_string(), Value::String(String::new()));

    let client = match client() {
        Ok(client) => client,
        Err(error) => return proxy_error("BotModels client configuration failed", &error),
    };
    json_response(
        client
            .post(format!("{}/api/music/generate", botmodels_url()))
            .header("X-API-Key", botmodels_key())
            .json(&payload),
    )
    .await
}

async fn format_input(Json(payload): Json<Value>) -> axum::response::Response {
    let client = match client() {
        Ok(client) => client,
        Err(error) => return proxy_error("BotModels client configuration failed", &error),
    };
    json_response(
        client
            .post(format!("{}/api/music/format", botmodels_url()))
            .header("X-API-Key", botmodels_key())
            .json(&payload),
    )
    .await
}

async fn job_status(Path(job_id): Path<String>) -> axum::response::Response {
    if job_id.is_empty() || job_id.len() > 160 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Invalid job identifier" })),
        )
            .into_response();
    }
    let client = match client() {
        Ok(client) => client,
        Err(error) => return proxy_error("BotModels client configuration failed", &error),
    };
    json_response(
        client
            .get(format!(
                "{}/api/music/jobs/{}",
                botmodels_url(),
                urlencoding::encode(&job_id)
            ))
            .header("X-API-Key", botmodels_key()),
    )
    .await
}

async fn models() -> axum::response::Response {
    proxy_get("/api/music/models").await
}

/// Pure health decision for the Jukebox probe (#2951).
///
/// This endpoint is a liveness probe used by the Suite app poller, not a
/// reverse proxy for the optional ACE-Step engine. An absent or unhealthy
/// backend is a *degraded* state reported with HTTP 200 and `healthy: false`,
/// never a 502 that surfaces as a console error when the app opens. The
/// `healthy` field is kept for the existing frontend (`checkEngine()`).
fn health_decision(upstream: Option<&Value>) -> (StatusCode, Value) {
    let healthy = upstream
        .and_then(|body| body.get("healthy"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    if healthy {
        (StatusCode::OK, json!({ "status": "ok", "healthy": true }))
    } else {
        (
            StatusCode::OK,
            json!({
                "status": "degraded",
                "healthy": false,
                "reason": "backend unavailable",
            }),
        )
    }
}

async fn health() -> axum::response::Response {
    let client = match health_client() {
        Ok(client) => client,
        Err(error) => {
            tracing::warn!(%error, "jukebox health probe client unavailable");
            let (status, payload) = health_decision(None);
            return (status, Json(payload)).into_response();
        }
    };
    let upstream = match client
        .get(format!("{}/api/music/health", botmodels_url()))
        .header("X-API-Key", botmodels_key())
        .send()
        .await
    {
        Ok(response) if response.status().is_success() => response.json::<Value>().await.ok(),
        Ok(response) => {
            tracing::warn!(status = %response.status(), "jukebox backend health check failed");
            None
        }
        Err(error) => {
            tracing::warn!(%error, "jukebox backend is unreachable");
            None
        }
    };
    let (status, payload) = health_decision(upstream.as_ref());
    (status, Json(payload)).into_response()
}

async fn proxy_get(path: &str) -> axum::response::Response {
    let client = match client() {
        Ok(client) => client,
        Err(error) => return proxy_error("BotModels client configuration failed", &error),
    };
    json_response(
        client
            .get(format!("{}{}", botmodels_url(), path))
            .header("X-API-Key", botmodels_key()),
    )
    .await
}

async fn audio(Query(query): Query<AudioQuery>) -> axum::response::Response {
    if !valid_audio_path(&query.path) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Invalid audio path" })),
        )
            .into_response();
    }
    let client = match client() {
        Ok(client) => client,
        Err(error) => return proxy_error("BotModels client configuration failed", &error),
    };
    let upstream = match client
        .get(format!("{}/api/music/audio", botmodels_url()))
        .header("X-API-Key", botmodels_key())
        .query(&[("path", &query.path)])
        .send()
        .await
    {
        Ok(response) => response,
        Err(error) => return proxy_error("BotModels audio request failed", &error),
    };
    let status = status_from_upstream(upstream.status());
    let content_type = upstream
        .headers()
        .get(header::CONTENT_TYPE)
        .cloned()
        .unwrap_or_else(|| header::HeaderValue::from_static("audio/mpeg"));
    // Buffer the locally generated track before handing it to BotUI. Passing the
    // reqwest stream through two reverse-proxy hops occasionally caused Chromium
    // to reject an otherwise complete WAV transfer with `Failed to fetch`.
    let bytes = match upstream.bytes().await {
        Ok(bytes) => bytes,
        Err(error) => return proxy_error("BotModels audio response could not be read", &error),
    };
    let builder = Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CACHE_CONTROL, "private, max-age=3600")
        .header(header::CONTENT_LENGTH, bytes.len().to_string());
    match builder.body(Body::from(bytes)) {
        Ok(response) => response,
        Err(error) => proxy_error("BotModels audio response could not be built", &error),
    }
}

pub fn configure_jukebox_routes() -> Router {
    Router::new()
        .route("/api/jukebox/generate", post(generate))
        .route("/api/jukebox/format", post(format_input))
        .route("/api/jukebox/jobs/:job_id", get(job_status))
        .route("/api/jukebox/models", get(models))
        .route("/api/jukebox/health", get(health))
        .route("/api/jukebox/audio", get(audio))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absent_backend_is_degraded_without_502() {
        let (status, payload) = health_decision(None);
        assert_eq!(status, StatusCode::OK);
        assert_eq!(payload["status"], "degraded");
        assert_eq!(payload["healthy"], false);
        assert_eq!(payload["reason"], "backend unavailable");
    }

    #[test]
    fn healthy_backend_reports_ok() {
        let upstream = json!({ "healthy": true, "engine": "ace-step" });
        let (status, payload) = health_decision(Some(&upstream));
        assert_eq!(status, StatusCode::OK);
        assert_eq!(payload["status"], "ok");
        assert_eq!(payload["healthy"], true);
    }

    #[test]
    fn unhealthy_backend_body_is_degraded() {
        let upstream = json!({ "healthy": false, "detail": "model not loaded" });
        let (status, payload) = health_decision(Some(&upstream));
        assert_eq!(status, StatusCode::OK);
        assert_eq!(payload["status"], "degraded");
        assert_eq!(payload["healthy"], false);
    }

    #[test]
    fn malformed_backend_body_is_degraded() {
        let missing_field = json!({ "engine": "ace-step" });
        let (status, payload) = health_decision(Some(&missing_field));
        assert_eq!(status, StatusCode::OK);
        assert_eq!(payload["status"], "degraded");

        let wrong_type = json!({ "healthy": "yes" });
        let (status, payload) = health_decision(Some(&wrong_type));
        assert_eq!(status, StatusCode::OK);
        assert_eq!(payload["status"], "degraded");
        assert_eq!(payload["healthy"], false);
    }
}

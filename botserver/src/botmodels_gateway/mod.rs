use axum::{
    body::Body,
    extract::Path,
    http::{header, Method, Request, StatusCode},
    response::IntoResponse,
    routing::any,
    Json, Router,
};
use serde_json::json;

fn botmodels_url() -> String {
    let url = std::env::var("BOTMODELS_HOST")
        .or_else(|_| std::env::var("BOTMODELS_URL"))
        .unwrap_or_else(|_| "http://localhost:8085".to_string())
        .trim_end_matches('/')
        .to_string();
    // #2590 — in containerized installs with mTLS provisioned, talk to the
    // internal BotModels service over HTTPS. This is a no-op for desktop-local
    // and single-host localhost installs (TLS integration is never initialized).
    #[cfg(feature = "security")]
    let url = botsecurity::integration::to_secure_url(&url);
    url
}

fn botmodels_key() -> String {
    std::env::var("BOTMODELS_API_KEY").unwrap_or_else(|_| "starter".to_string())
}

fn client() -> Result<reqwest::Client, String> {
    // #2590 — when internal mTLS is active, build the client with the
    // provisioned botserver client identity and internal CA trust. Otherwise
    // fall through to the unchanged plain client (localhost/desktop-local).
    #[cfg(feature = "security")]
    if botsecurity::integration::is_mtls_active() {
        return botsecurity::integration::create_https_client("api")
            .map_err(|error| format!("Could not create BotModels mTLS client: {error}"));
    }
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|error| format!("Could not create BotModels client: {error}"))
}

async fn proxy(Path(path): Path<String>, req: Request<Body>) -> impl IntoResponse {
    let method = req.method().clone();
    let query = req
        .uri()
        .query()
        .map(|q| format!("?{}", q))
        .unwrap_or(Default::default());
    let upstream_url = format!("{}/{}{}", botmodels_url(), path.trim_start_matches('/'), query);

    let client = match client() {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(json!({"error": e})),
            )
                .into_response();
        }
    };

    let headers = req.headers().clone();
    let body_bytes = match axum::body::to_bytes(req.into_body(), 1024 * 1024 * 50).await {
        Ok(b) => b,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": format!("body read failed: {e}")})),
            )
                .into_response();
        }
    };

    let mut builder = match method {
        Method::GET => client.get(&upstream_url),
        Method::POST => client.post(&upstream_url),
        Method::PUT => client.put(&upstream_url),
        Method::DELETE => client.delete(&upstream_url),
        Method::PATCH => client.patch(&upstream_url),
        _ => {
            return (
                StatusCode::METHOD_NOT_ALLOWED,
                Json(json!({"error": "method not allowed"})),
            )
                .into_response();
        }
    };
    builder = builder.header("X-API-Key", botmodels_key());
    if let Some(ct) = headers.get(header::CONTENT_TYPE) {
        builder = builder.header(header::CONTENT_TYPE, ct.clone());
    }
    if !body_bytes.is_empty() {
        builder = builder.body(body_bytes.to_vec());
    }

    let upstream = match builder.send().await {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(json!({"error": format!("BotModels request failed: {e}")})),
            )
                .into_response();
        }
    };
    let status = StatusCode::from_u16(upstream.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
    let content_type = upstream
        .headers()
        .get(header::CONTENT_TYPE)
        .cloned()
        .unwrap_or_else(|| header::HeaderValue::from_static("application/json"));
    let bytes = match upstream.bytes().await {
        Ok(b) => b,
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(json!({"error": format!("BotModels response read failed: {e}")})),
            )
                .into_response();
        }
    };
    let mut resp = axum::response::Response::builder().status(status);
    resp = resp.header(header::CONTENT_TYPE, content_type);
    match resp.body(Body::from(bytes)) {
        Ok(r) => r,
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "response build failed"})),
        )
            .into_response(),
    }
}

pub fn configure_botmodels_gateway_routes() -> Router {
    Router::new().route("/api/botmodels/*path", any(proxy))
}

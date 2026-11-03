//! HTTP surface for sandbox resources.
//!
//! `/sandbox/ui/{hash}` is mounted on a separate origin when configured (the
//! desktop build binds 127.0.0.1:8081). The main router also carries it so
//! tests and single-origin deployments can reach the same handler; isolation
//! still comes from the opaque iframe sandbox on the client.

use std::collections::HashMap;
use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use botcore::shared::state::AppState;

use super::csp::UiCsp;
use super::store::read_resource;
use super::sign::verify_resource;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/sandbox/ui/:hash", get(serve_resource))
        .route("/api/sandbox/config", get(config))
}

async fn serve_resource(
    State(state): State<Arc<AppState>>,
    Path(hash): Path<String>,
    Query(query): Query<HashMap<String, String>>,
) -> Response {
    if hash.len() != 64 || !hash.chars().all(|c| c.is_ascii_hexdigit()) {
        return (StatusCode::BAD_REQUEST, "invalid resource id").into_response();
    }
    let Some(token) = query.get("t") else {
        return (StatusCode::UNAUTHORIZED, "missing resource token").into_response();
    };
    if verify_resource(&hash, token).is_err() {
        return (StatusCode::UNAUTHORIZED, "invalid or expired resource token").into_response();
    }
    let bucket = query
        .get("b")
        .cloned()
        .unwrap_or_else(|| state.bucket_name.clone());
    let Some(resource) = read_resource(&state, &bucket, &hash).await else {
        return (StatusCode::NOT_FOUND, "resource not found").into_response();
    };
    resource_response(&resource.html, &resource.csp)
}

fn resource_response(html: &str, csp: &UiCsp) -> Response {
    let app_origin = app_origin();
    let mut response = Response::new(axum::body::Body::from(html.to_string()));
    let headers = response.headers_mut();
    if let Ok(value) = HeaderValue::from_str(&csp.resource_csp(&app_origin)) {
        headers.insert(
            axum::http::HeaderName::from_static("content-security-policy"),
            value,
        );
    }
    if let Ok(value) = HeaderValue::from_str(&csp.permissions_policy()) {
        headers.insert("permissions-policy", value);
    }
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/html; charset=utf-8"),
    );
    headers.insert("x-content-type-options", HeaderValue::from_static("nosniff"));
    headers.insert("referrer-policy", HeaderValue::from_static("no-referrer"));
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, max-age=30"),
    );
    response
}

async fn config(State(state): State<Arc<AppState>>) -> Response {
    let origin = sandbox_origin();
    let body = serde_json::json!({
        "enabled": !origin.is_empty(),
        "origin": origin,
        "app_origin": app_origin(),
        "mode": "opaque",
        "bucket": state.bucket_name,
    });
    axum::Json(body).into_response()
}

/// Configured origin of the sandbox listener. Empty means the deployment has
/// not split origins yet; the client must not render Tier 2 apps until it is
/// configured.
#[must_use]
pub fn sandbox_origin() -> String {
    if let Ok(value) = std::env::var("GB_SANDBOX_ORIGIN") {
        let value = value.trim().to_string();
        if !value.is_empty() {
            return value;
        }
    }
    if cfg!(feature = "desktop-local") {
        return "http://127.0.0.1:8081".to_string();
    }
    String::new()
}

#[must_use]
pub fn app_origin() -> String {
    std::env::var("GB_APP_ORIGIN")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "https: http://localhost:* http://127.0.0.1:*".to_string())
}

/// Starts the dedicated sandbox listener when the deployment configures one.
/// Desktop defaults to loopback port 8081; every other build stays quiet until
/// `GB_SANDBOX_PORT` is set.
pub fn spawn_listener(state: Arc<AppState>) {
    let port = match std::env::var("GB_SANDBOX_PORT") {
        Ok(value) => match value.trim().parse::<u16>() {
            Ok(port) if port > 0 => port,
            _ => return,
        },
        Err(_) => {
            if cfg!(feature = "desktop-local") {
                8081
            } else {
                return;
            }
        }
    };
    tokio::spawn(async move {
        let bind = format!("127.0.0.1:{port}");
        let listener = match tokio::net::TcpListener::bind(&bind).await {
            Ok(listener) => listener,
            Err(error) => {
                log::warn!("sandbox listener {bind} unavailable: {error}");
                return;
            }
        };
        log::info!("sandbox listener on {bind}");
        let app = router().with_state(state);
        if let Err(error) = axum::serve(listener, app).await {
            log::warn!("sandbox listener stopped: {error}");
        }
    });
}

#[cfg(test)]
mod tests {
    use super::{app_origin, sandbox_origin};

    #[test]
    fn desktop_defaults_to_loopback_sandbox_origin() {
        if cfg!(feature = "desktop-local") {
            assert!(sandbox_origin().contains("127.0.0.1:8081"));
        }
        assert!(!app_origin().is_empty());
    }
}

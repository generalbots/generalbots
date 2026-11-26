use axum::{
    body::{Body, Bytes, HttpBody},
    extract::{OriginalUri, State},
    http::{header, HeaderMap, Request, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use log::{debug, error, warn};
use serde::Deserialize;
use std::{future::poll_fn, pin::Pin, sync::OnceLock};

use crate::shared::AppState;

/// Default cap (64 MiB) for a request body the UI proxy buffers before
/// forwarding it to botserver. #2777: the previous `usize::MAX` let an
/// anonymous chunked POST (notably `/webhook/*`) allocate without bound.
pub const DEFAULT_PROXY_BODY_LIMIT_BYTES: usize = 64 * 1024 * 1024;

/// Backend `UploadClass::Drive` budget, used by `/api/files/write` and
/// `/api/files/upload-binary`. The proxy must not reject an upload the
/// backend itself accepts.
pub const DRIVE_UPLOAD_BODY_LIMIT_BYTES: usize = 100 * 1024 * 1024;

/// Backend video-media budget, used by `/api/video/projects/:id/upload`.
pub const VIDEO_UPLOAD_BODY_LIMIT_BYTES: usize = 512 * 1024 * 1024;

const PROXY_BODY_LIMIT_ENV: &str = "GB_PROXY_MAX_BODY_BYTES";

static PROXY_BODY_LIMIT: OnceLock<usize> = OnceLock::new();

/// Parses a deploy-time override. Empty, malformed and zero values are
/// ignored so a bad configuration can never disable the guard.
fn parse_body_limit(raw: Option<&str>) -> Option<usize> {
    raw.and_then(|value| value.trim().parse::<usize>().ok())
        .filter(|&bytes| bytes > 0)
}

/// Effective default cap, resolved once at first use.
pub fn proxy_body_limit_bytes() -> usize {
    *PROXY_BODY_LIMIT.get_or_init(|| {
        let override_value = std::env::var(PROXY_BODY_LIMIT_ENV).ok();
        parse_body_limit(override_value.as_deref()).unwrap_or(DEFAULT_PROXY_BODY_LIMIT_BYTES)
    })
}

/// Cap for `path`. The Drive write endpoints and the video upload keep their
/// larger backend budgets; every other path — including the anonymous
/// `/webhook/*` deliveries — uses the default.
pub fn body_limit_for_path(path: &str) -> usize {
    let default = proxy_body_limit_bytes();
    if path == "/api/files/write" || path == "/api/files/upload-binary" {
        return default.max(DRIVE_UPLOAD_BODY_LIMIT_BYTES);
    }
    if path.starts_with("/api/video/projects/") && path.ends_with("/upload") {
        return default.max(VIDEO_UPLOAD_BODY_LIMIT_BYTES);
    }
    default
}

/// Why reading a proxied request body failed.
#[derive(Debug)]
enum BodyReadError {
    /// Cap exceeded — the caller answers 413.
    TooLarge,
    /// Transport failure while reading — the caller answers 500.
    Source(String),
}

/// Reads at most `limit` bytes, stopping at the first chunk that would
/// exceed the cap so a chunked multi-GB POST never allocates past `limit`.
/// A true streaming forward to botserver would require enabling reqwest's
/// `stream` feature; the bounded buffer is the safe fix here.
async fn read_limited_body(mut body: Body, limit: usize) -> Result<Bytes, BodyReadError> {
    let mut bytes = Vec::new();
    loop {
        let frame = poll_fn(|cx| Pin::new(&mut body).poll_frame(cx)).await;
        match frame {
            Some(Ok(frame)) => {
                if let Ok(chunk) = frame.into_data() {
                    let extended = bytes
                        .len()
                        .checked_add(chunk.len())
                        .ok_or(BodyReadError::TooLarge)?;
                    if extended > limit {
                        return Err(BodyReadError::TooLarge);
                    }
                    bytes.extend_from_slice(&chunk);
                }
            }
            Some(Err(source)) => return Err(BodyReadError::Source(source.to_string())),
            None => break,
        }
    }
    Ok(Bytes::from(bytes))
}

fn extract_app_context(headers: &HeaderMap, path: &str) -> Option<String> {
    if let Some(referer) = headers.get("referer") {
        if let Ok(referer_str) = referer.to_str() {
            if let Some(start) = referer_str.find("/apps/") {
                let after_apps = &referer_str[start + 6..];
                if let Some(end) = after_apps.find('/') {
                    return Some(after_apps[..end].to_string());
                } else if !after_apps.is_empty() {
                    return Some(after_apps.to_string());
                }
            }
            if let Some(_start) = referer_str.find(".gb.solutions") {
                if let Some(host_start) = referer_str.find("://") {
                    let host_part = &referer_str[host_start + 3..];
                    if let Some(dot) = host_part.find('.') {
                        return Some(host_part[..dot].to_string());
                    }
                }
            }
        }
    }

    if let Some(after_apps) = path.strip_prefix("/apps/") {
        if let Some(end) = after_apps.find('/') {
            return Some(after_apps[..end].to_string());
        }
    }

    None
}

pub async fn proxy_api(
    State(state): State<AppState>,
    OriginalUri(uri): OriginalUri,
    req: Request<Body>,
) -> Response<Body> {
    let path = uri.path();
    let query = uri.query().map_or_else(String::new, |q| format!("?{q}"));
    let method = req.method().clone();
    let headers = req.headers().clone();

    // #2935 — the four `/suite/{dir}/{partials,fragments,modals,forms}/*`
    // suffixes are routed here so they beat the `ServeDir` nests. Answer the
    // shipped static assets from botui's own `ui/` tree first; the backend's
    // BOTUI_PATH must never be a second point of failure for the Settings
    // partials. Everything else still takes the proxy path below unchanged
    // (`/api/*`, `/ws`, `/cloud/partials/*`, writes, dynamic fragments).
    if let Some(local) =
        crate::ui_server::local_static::try_serve_local_suite_asset(&uri, &method).await
    {
        return local;
    }

    let app_context = extract_app_context(&headers, path);

    let target_url = format!("{}{path}{query}", state.client.base_url());
    debug!("Proxying {method} {path} to {target_url} (app: {app_context:?})");

    let client = crate::ui_server::tls_policy::backend_http_client(state.client.base_url());
    let mut proxy_req = client.request(method.clone(), &target_url);

    if let Some(host) = headers.get("host").and_then(|value| value.to_str().ok()) {
        proxy_req = proxy_req.header("x-forwarded-host", host);
    }
    if !headers.contains_key("x-forwarded-proto") {
        proxy_req = proxy_req.header("x-forwarded-proto", "http");
    }

    for (name, value) in &headers {
        if name != "host" {
            if let Ok(v) = value.to_str() {
                proxy_req = proxy_req.header(name.as_str(), v);
            }
        }
    }

    if let Some(app) = app_context {
        proxy_req = proxy_req.header("X-App-Context", app);
    }

    let limit = body_limit_for_path(path);

    // A declared oversized body is refused before a single byte is read.
    if let Some(declared) = headers
        .get(header::CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.trim().parse::<u64>().ok())
    {
        if declared > limit as u64 {
            warn!("Rejecting oversized request body: {method} {path} declares {declared} bytes (limit {limit})");
            return build_error_response(StatusCode::PAYLOAD_TOO_LARGE, "Request body too large");
        }
    }

    let body_bytes = match read_limited_body(req.into_body(), limit).await {
        Ok(bytes) => bytes,
        Err(BodyReadError::TooLarge) => {
            warn!("Rejecting oversized request body: {method} {path} exceeded {limit} bytes");
            return build_error_response(StatusCode::PAYLOAD_TOO_LARGE, "Request body too large");
        }
        Err(BodyReadError::Source(source)) => {
            error!("Failed to read request body: {source}");
            return build_error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to read request body",
            );
        }
    };

    if !body_bytes.is_empty() {
        proxy_req = proxy_req.body(body_bytes.to_vec());
    }

    match proxy_req.send().await {
        Ok(resp) => build_proxy_response(resp).await,
        Err(e) => {
            error!("Proxy request failed: {e}");
            build_error_response(StatusCode::BAD_GATEWAY, &format!("Proxy error: {e}"))
        }
    }
}

fn build_error_response(status: StatusCode, message: &str) -> Response<Body> {
    Response::builder()
        .status(status)
        .body(Body::from(message.to_string()))
        .unwrap_or_else(|_| {
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from("Failed to build error response"))
                .unwrap_or_default()
        })
}

async fn build_proxy_response(resp: reqwest::Response) -> Response<Body> {
    let status = resp.status();
    let headers = resp.headers().clone();

    match resp.bytes().await {
        Ok(body) => {
            let mut response = Response::builder().status(status);

            for (name, value) in &headers {
                response = response.header(name, value);
            }

            response.body(Body::from(body)).unwrap_or_else(|_| {
                build_error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to build response",
                )
            })
        }
        Err(e) => {
            error!("Failed to read response body: {e}");
            build_error_response(
                StatusCode::BAD_GATEWAY,
                &format!("Failed to read response: {e}"),
            )
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ClientError {
    message: String,
    stack: Option<String>,
    source: String,
    url: String,
    user_agent: String,
    timestamp: String,
}

pub async fn handle_client_error(Json(error): Json<ClientError>) -> impl IntoResponse {
    warn!(
        "CLIENT:{}: {} at {} ({}) - {}",
        error.source.to_uppercase(),
        error.message,
        error.url,
        error.timestamp,
        error.user_agent
    );

    if let Some(stack) = &error.stack {
        if !stack.is_empty() {
            warn!("CLIENT:STACK: {}", stack);
        }
    }

    StatusCode::OK
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_limit_is_bounded_and_sane() {
        assert!(DEFAULT_PROXY_BODY_LIMIT_BYTES >= 1024 * 1024);
        assert!(DEFAULT_PROXY_BODY_LIMIT_BYTES <= 128 * 1024 * 1024);
        assert_eq!(DEFAULT_PROXY_BODY_LIMIT_BYTES % (1024 * 1024), 0);
        assert!(DEFAULT_PROXY_BODY_LIMIT_BYTES < usize::MAX);
    }

    #[test]
    fn upload_limits_are_ordered_and_bounded() {
        assert!(DRIVE_UPLOAD_BODY_LIMIT_BYTES > DEFAULT_PROXY_BODY_LIMIT_BYTES);
        assert!(VIDEO_UPLOAD_BODY_LIMIT_BYTES > DRIVE_UPLOAD_BODY_LIMIT_BYTES);
        assert!(VIDEO_UPLOAD_BODY_LIMIT_BYTES < usize::MAX);
    }

    #[tokio::test]
    async fn read_limited_body_stops_at_the_cap() {
        let body = Body::from(vec![7u8; 16]);
        let bytes = read_limited_body(body, 16).await.expect("body fits the cap");
        assert_eq!(bytes.as_ref(), &[7u8; 16][..]);

        let body = Body::from(vec![7u8; 17]);
        let result = read_limited_body(body, 16).await;
        assert!(matches!(result, Err(BodyReadError::TooLarge)));
    }

    #[test]
    fn invalid_or_zero_overrides_are_ignored() {
        assert_eq!(parse_body_limit(Some("1048576")), Some(1024 * 1024));
        assert_eq!(parse_body_limit(Some(" 2048 ")), Some(2048));
        assert_eq!(parse_body_limit(Some("0")), None);
        assert_eq!(parse_body_limit(Some("not-a-number")), None);
        assert_eq!(parse_body_limit(None), None);
    }

    #[test]
    fn path_specific_limits_only_exempt_known_large_uploads() {
        let default = proxy_body_limit_bytes();
        assert_eq!(body_limit_for_path("/webhook/telegram"), default);
        assert_eq!(body_limit_for_path("/api/chat"), default);
        assert_eq!(
            body_limit_for_path("/api/files/write"),
            default.max(DRIVE_UPLOAD_BODY_LIMIT_BYTES)
        );
        assert_eq!(
            body_limit_for_path("/api/files/upload-binary"),
            default.max(DRIVE_UPLOAD_BODY_LIMIT_BYTES)
        );
        assert_eq!(
            body_limit_for_path("/api/video/projects/abc/upload"),
            default.max(VIDEO_UPLOAD_BODY_LIMIT_BYTES)
        );
        assert_eq!(body_limit_for_path("/api/video/projects/abc/preview"), default);
    }
}

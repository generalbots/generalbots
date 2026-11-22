//! HTTP surface for sandbox resources.
//!
//! `/sandbox/ui/{hash}` is mounted on a separate origin when configured (the
//! desktop build binds 127.0.0.1:8081). The main router also carries it so
//! tests and single-origin deployments can reach the same handler; isolation
//! still comes from the opaque iframe sandbox on the client.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
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
use super::preview_status;
use super::sign::verify_resource;
use super::store::read_resource;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/sandbox/ui/:hash", get(serve_resource))
        .route("/api/sandbox/config", get(config))
        .route("/api/sandbox/status/:hash", get(resource_status))
}

/// #2541 — a resource id is the sha256 of its bytes: always 64 hex digits.
fn valid_hash(hash: &str) -> bool {
    hash.len() == 64 && hash.chars().all(|c| c.is_ascii_hexdigit())
}

/// #2541 — the reason code travels in the response header *and* the JSON body
/// so the Vibe UI (which cannot read an iframe's headers) sees the same value.
fn preview_status_header(code: &str) -> HeaderValue {
    HeaderValue::from_str(code).unwrap_or_else(|_| HeaderValue::from_static("unknown"))
}

/// #2541 — machine-readable status body. A missing entry is reported as
/// `unknown`, never silently omitted; `live` is true only for a fresh preview.
fn status_body(reason: Option<&str>, hash: &str) -> serde_json::Value {
    let code = reason.unwrap_or("unknown");
    serde_json::json!({
        "hash": hash,
        "status": code,
        "reason": code,
        "live": reason == Some("fresh"),
    })
}

/// #2541 — JSON counterpart of `/sandbox/ui/{hash}`. The HTML route can only
/// carry the reason in a header; the UI reads this endpoint to render an
/// honest `expired` / `interrupted_by_restart` chip.
async fn resource_status(Path(hash): Path<String>) -> Response {
    if !valid_hash(&hash) {
        return (StatusCode::BAD_REQUEST, "invalid resource id").into_response();
    }
    let reason = preview_status(&hash);
    let body = status_body(reason.as_deref(), &hash);
    let code = reason.as_deref().unwrap_or("unknown");
    let mut response = axum::Json(body).into_response();
    response
        .headers_mut()
        .insert("x-gb-preview-status", preview_status_header(code));
    response
}

async fn serve_resource(
    State(state): State<Arc<AppState>>,
    Path(hash): Path<String>,
    Query(query): Query<HashMap<String, String>>,
) -> Response {
    if !valid_hash(&hash) {
        return (StatusCode::BAD_REQUEST, "invalid resource id").into_response();
    }
    let Some(token) = query.get("t") else {
        return (StatusCode::UNAUTHORIZED, "missing resource token").into_response();
    };
    if verify_resource(&hash, token).is_err() {
        return (
            StatusCode::UNAUTHORIZED,
            "invalid or expired resource token",
        )
            .into_response();
    }
    let bucket = query
        .get("b")
        .cloned()
        .unwrap_or_else(|| state.bucket_name.clone());
    let Some(resource) = read_resource(&state, &bucket, &hash).await else {
        return (StatusCode::NOT_FOUND, "resource not found").into_response();
    };
    let mut response = resource_response(&resource.html, &resource.csp);
    if let Some(status) = preview_status(&hash) {
        response
            .headers_mut()
            .insert("x-gb-preview-status", preview_status_header(&status));
    }
    response
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
    headers.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
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

/// Whether a sandbox listener is actually bound (#1896).
///
/// `sandbox_origin()` only says where the listener *would* live. Publishing a
/// resource URL before the bind succeeds hands the client a frame that can
/// never load: the app window opens blank with no error and nothing to retry.
static SANDBOX_LISTENER_READY: AtomicBool = AtomicBool::new(false);

#[must_use]
pub fn sandbox_listener_ready() -> bool {
    SANDBOX_LISTENER_READY.load(Ordering::Relaxed)
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
        SANDBOX_LISTENER_READY.store(true, Ordering::Relaxed);
        log::info!("sandbox listener on {bind}");
        // IMP-09: separate listener, so it carries the header layer itself.
        // The layer uses `or_insert` for CSP and frame options, so the
        // per-resource sandbox policy stays in charge of those two.
        let app = crate::security::headers::with_security_headers(
            router().with_state(state),
            crate::security::headers::SecurityHeadersConfig::from_env(),
        );
        if let Err(error) = axum::serve(listener, app).await {
            log::warn!("sandbox listener stopped: {error}");
        }
        SANDBOX_LISTENER_READY.store(false, Ordering::Relaxed);
    });
}

#[cfg(test)]
mod tests {
    use axum::http::HeaderValue;

    use super::{app_origin, preview_status_header, sandbox_origin, status_body, valid_hash};

    /// 64 hex digits — the shape every real resource id has.
    const HASH: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    #[test]
    fn desktop_defaults_to_loopback_sandbox_origin() {
        if cfg!(feature = "desktop-local") {
            assert!(sandbox_origin().contains("127.0.0.1:8081"));
        }
        assert!(!app_origin().is_empty());
    }

    #[test]
    fn resource_id_is_a_sha256_hex_digest() {
        assert!(valid_hash(&"a".repeat(64)));
        assert!(valid_hash(
            &"ABCDEF0123".repeat(7).chars().take(64).collect::<String>()
        ));
        assert!(!valid_hash("short"));
        assert!(!valid_hash(&format!("{}z", "a".repeat(63))));
        assert!(!valid_hash(&"a".repeat(65)));
    }

    /// #2541 — every canonical reason code is reported truthfully, including
    /// the `unknown` fallback for an entry no longer in the cache.
    #[test]
    fn status_body_reports_all_canonical_reason_codes() {
        for (reason, code, live) in [
            (Some("fresh"), "fresh", true),
            (Some("expired"), "expired", false),
            (
                Some("interrupted_by_restart"),
                "interrupted_by_restart",
                false,
            ),
            (None, "unknown", false),
        ] {
            let body = status_body(reason, HASH);
            assert_eq!(body["hash"].as_str(), Some(HASH), "hash for {code}");
            assert_eq!(body["status"].as_str(), Some(code), "status for {code}");
            assert_eq!(body["reason"].as_str(), Some(code), "reason for {code}");
            assert_eq!(body["live"].as_bool(), Some(live), "live for {code}");
        }
    }

    /// #2541 — the status endpoint is a status contract, not a document
    /// endpoint: exactly four keys, so no HTML/content field can leak.
    #[test]
    fn status_body_has_exactly_the_contract_keys() {
        let body = status_body(Some("fresh"), HASH);
        let keys = body.as_object().map(|object| {
            let mut keys: Vec<&str> = object.keys().map(String::as_str).collect();
            keys.sort_unstable();
            keys
        });
        assert_eq!(
            keys,
            Some(vec!["hash", "live", "reason", "status"]),
            "status body must not carry any field beyond the four contract keys"
        );
        assert_eq!(body.as_object().map(|object| object.len()), Some(4));
    }

    /// #2541 — no preview HTML/document content ever reaches the status body.
    #[test]
    fn status_body_never_carries_document_content() {
        let serialized = status_body(Some("fresh"), HASH).to_string();
        let lowered = serialized.to_ascii_lowercase();
        for marker in [
            "<html", "<script", "doctype", "content", "document", "secret",
        ] {
            assert!(
                !lowered.contains(marker),
                "status body leaked document marker {marker}: {serialized}"
            );
        }
    }

    /// #2541 — the header carries the same four codes as the JSON body.
    #[test]
    fn preview_status_header_emits_canonical_codes() {
        for code in ["fresh", "expired", "interrupted_by_restart", "unknown"] {
            assert_eq!(
                preview_status_header(code),
                HeaderValue::from_static(code),
                "header must carry the exact reason code {code}"
            );
        }
    }

    /// #2541 — a malformed or injected status cannot smuggle document content
    /// into the header; it degrades to `unknown` instead.
    #[test]
    fn preview_status_header_falls_back_for_injected_content() {
        let injected = "fresh\r\nX-Document: <html>secret</html>";
        assert_eq!(
            preview_status_header(injected),
            HeaderValue::from_static("unknown")
        );
    }

    #[test]
    fn a_missing_entry_is_reported_not_silently_dropped() {
        let body = status_body(None, "abc");
        assert_eq!(body["status"].as_str(), Some("unknown"));
        assert_eq!(body["reason"].as_str(), Some("unknown"));
        assert_eq!(body["live"].as_bool(), Some(false));
        // An explicit `unknown` reason is indistinguishable from a missing one.
        assert_eq!(status_body(Some("unknown"), "abc"), body);
    }
}

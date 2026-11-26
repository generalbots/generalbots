//! Local-first serving for the suite's static-suffix sub-paths (#2935).
//!
//! `configure_router` registers `/suite/{dir}/partials/*` (and the sibling
//! `fragments`, `modals`, `forms` suffixes) as `proxy_api` routes so they win
//! over the `/suite/{dir}` `ServeDir` nests. Serving those files only from
//! botserver made every Settings partial a 404 whenever that host's static
//! root pointed at a stale tree — while the same files shipped fine in this
//! crate's `ui/` directory.
//!
//! This module short-circuits **GET/HEAD requests for shipped static assets**
//! under those suffixes from botui's own `ui/` tree (the tree `ServeDir` uses
//! in dev). Everything else — `/api/*`, `/ws/*`, `/cloud/partials/*`, writes,
//! and server-generated fragments without a local file — keeps going through
//! the existing proxy unchanged.

use axum::{
    body::Body,
    http::{header, Method, Request, Uri},
    response::Response,
};
#[cfg(not(feature = "embed-ui"))]
use log::debug;
#[cfg(not(feature = "embed-ui"))]
use std::path::Path;
#[cfg(not(feature = "embed-ui"))]
use tower::Service;
#[cfg(not(feature = "embed-ui"))]
use tower_http::services::ServeDir;

#[cfg(feature = "embed-ui")]
use crate::ui_server::constants::Assets;
#[cfg(not(feature = "embed-ui"))]
use crate::ui_server::constants::get_ui_root;
use crate::ui_server::safe_path::is_safe_relative_path;

/// Suite sub-paths whose static files used to be forwarded to botserver.
/// The four suffixes share one route family in `configure_router`.
const LOCAL_SEGMENTS: &[&str] = &["partials", "fragments", "modals", "forms"];

/// Only shipped assets are eligible. A path without one of these extensions
/// is a server-generated endpoint and must keep going to botserver.
const STATIC_SUFFIXES: &[&str] = &[
    ".html", ".js", ".css", ".json", ".svg", ".png", ".jpg", ".jpeg", ".gif", ".webp", ".ico",
    ".woff", ".woff2", ".ttf", ".eot", ".mp4", ".webm", ".mp3", ".wav", ".map", ".txt",
];

/// True when `path` is a static asset of one of the proxied suite suffixes and
/// may therefore be answered from botui's local `ui/` tree.
pub fn is_local_suite_asset(method: &Method, path: &str) -> bool {
    if *method != Method::GET && *method != Method::HEAD {
        return false;
    }
    let Some(rest) = path.strip_prefix("/suite/") else {
        return false;
    };
    let mut segments = rest.split('/');
    let first = segments.next();
    let second = segments.next();
    // Two shapes are shipped: /suite/<dir>/<partials|fragments|modals|forms>/<file>
    // and the top-level /suite/<partials|fragments|modals|forms>/<file> nest
    // (e.g. /suite/partials/chat.html). In the latter the first segment is the
    // kind, not a suite directory. Accept both, never a bare directory path.
    let kind = match (first, second) {
        (_, Some(segment)) if LOCAL_SEGMENTS.contains(&segment) => segment,
        (Some(segment), _) if LOCAL_SEGMENTS.contains(&segment) => segment,
        _ => return false,
    };
    if !is_safe_relative_path(rest) {
        return false;
    }
    let lower = path.to_ascii_lowercase();
    STATIC_SUFFIXES.iter().any(|suffix| lower.ends_with(suffix))
}

/// Serve from the local `ui/` directory with `ServeDir`, so the exact same
/// file/mime/index rules as the dev-mode `/suite/{dir}` nests apply. A miss
/// (404 or any non-success) returns `None` and the caller proxies it, keeping
/// dynamic endpoints that happen to live under the same suffix working.
#[cfg(not(feature = "embed-ui"))]
async fn serve_from_disk(root: &Path, uri: &Uri, method: &Method) -> Option<Response<Body>> {
    let request = Request::builder()
        .uri(uri.clone())
        .method(method.clone())
        .body(Body::empty())
        .ok()?;
    let mut service = ServeDir::new(root);
    let response = service.call(request).await.ok()?;
    if !response.status().is_success() {
        return None;
    }
    let (mut parts, body) = response.into_parts();
    // Static routes registered in `add_static_routes` use `no-cache` so a
    // deploy is picked up immediately; local serving must behave the same.
    parts.headers.insert(
        header::CACHE_CONTROL,
        header::HeaderValue::from_static("no-cache"),
    );
    Some(Response::from_parts(parts, Body::new(body)))
}

/// Release builds embed the whole `ui/` tree, so serve from `Assets` instead
/// of the filesystem. Behaviour is otherwise identical to `serve_from_disk`.
#[cfg(feature = "embed-ui")]
fn serve_embedded(path: &str, method: &Method) -> Option<Response<Body>> {
    let asset_path = path.trim_start_matches('/');
    let content = Assets::get(asset_path)?;
    let mime = mime_guess::from_path(asset_path).first_or_text_plain();
    let body = if *method == Method::HEAD {
        Body::empty()
    } else {
        Body::from(content.data)
    };
    Response::builder()
        .status(axum::http::StatusCode::OK)
        .header(header::CONTENT_TYPE, mime.as_ref())
        .header(header::CACHE_CONTROL, "no-cache")
        .body(body)
        .ok()
}

/// Try to answer `uri` from the local UI tree. Returns `None` for every request
/// that must still be proxied to botserver.
pub async fn try_serve_local_suite_asset(
    uri: &Uri,
    method: &Method,
) -> Option<Response<Body>> {
    if !is_local_suite_asset(method, uri.path()) {
        return None;
    }

    #[cfg(feature = "embed-ui")]
    {
        return serve_embedded(uri.path(), method);
    }

    #[cfg(not(feature = "embed-ui"))]
    {
        let response = serve_from_disk(&get_ui_root(), uri, method).await?;
        debug!(
            "Served suite asset from the local UI tree (no backend static-root dependency): {}",
            uri.path()
        );
        Some(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_settings_partials_and_other_static_suite_assets() {
        let cases = [
            "/suite/settings/partials/01_profile_security.html",
            "/suite/partials/chat.html",
            "/suite/admin/modals/confirm.html",
            "/suite/docs/fragments/view.html",
            "/suite/partials/styles.css",
        ];
        for case in cases {
            assert!(
                is_local_suite_asset(&Method::GET, case),
                "must be served locally: {case}"
            );
            assert!(
                is_local_suite_asset(&Method::HEAD, case),
                "HEAD must be served locally: {case}"
            );
        }
    }

    #[test]
    fn api_ws_cloud_and_writes_are_never_locally_served() {
        let cases = [
            "/api/chat",
            "/api/files/write",
            "/ws/task-progress",
            "/ws/task-progress/42",
            "/cloud/partials/sidebar.html",
            "/webhook/telegram",
            "/suite/settings/index.html",
        ];
        for case in cases {
            assert!(
                !is_local_suite_asset(&Method::GET, case),
                "must stay proxied: {case}"
            );
        }
        let partial = "/suite/settings/partials/01_profile_security.html";
        assert!(!is_local_suite_asset(&Method::POST, partial));
        assert!(!is_local_suite_asset(&Method::DELETE, partial));
        assert!(!is_local_suite_asset(&Method::PUT, partial));
    }

    #[test]
    fn traversal_and_non_static_paths_stay_proxied() {
        let cases = [
            "/suite/settings/partials/../../secret.html",
            "/suite/settings/partials/%2e%2e/secret.html",
            "/suite/sheet/fragments/charts",
            "/suite/settings/partials/",
        ];
        for case in cases {
            assert!(
                !is_local_suite_asset(&Method::GET, case),
                "must not be served locally: {case}"
            );
        }
    }

    #[cfg(not(feature = "embed-ui"))]
    #[tokio::test]
    async fn disk_serving_reads_the_local_tree_and_misses_fall_through() {
        let base = std::env::temp_dir().join(format!(
            "gb-local-static-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let file = base.join("suite/settings/partials/probe.html");
        let Some(parent) = file.parent() else {
            return;
        };
        assert!(std::fs::create_dir_all(parent).is_ok());
        assert!(std::fs::write(&file, b"<p>local probe</p>").is_ok());

        let hit = Uri::from_static("/suite/settings/partials/probe.html");
        let served = serve_from_disk(&base, &hit, &Method::GET).await;
        assert!(served.is_some(), "existing local partial must be served");
        if let Some(response) = served {
            assert!(response.status().is_success());
            assert_eq!(
                response.headers().get(header::CACHE_CONTROL),
                Some(&header::HeaderValue::from_static("no-cache"))
            );
        }

        let miss = Uri::from_static("/suite/settings/partials/missing.html");
        assert!(
            serve_from_disk(&base, &miss, &Method::GET).await.is_none(),
            "a local miss must fall through to the backend proxy"
        );
        std::fs::remove_dir_all(&base).ok();
    }
}

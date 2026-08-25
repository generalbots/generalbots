use axum::{
    http::StatusCode,
    response::IntoResponse,
};

#[cfg(feature = "embed-ui")]
use crate::ui_server::constants::{Assets, ROOT_FILES, SITE_ROOT_FILES, SUITE_DIRS};
#[cfg(not(feature = "embed-ui"))]
use crate::ui_server::constants::get_ui_root;

pub async fn serve_favicon() -> impl IntoResponse {
    #[cfg(feature = "embed-ui")]
    {
        match Assets::get("suite/public/favicon.ico") {
            Some(content) => (
                StatusCode::OK,
                [("content-type", "image/x-icon")],
                content.data,
            )
                .into_response(),
            None => StatusCode::NOT_FOUND.into_response(),
        }
    }
    #[cfg(not(feature = "embed-ui"))]
    {
        let favicon_path = get_ui_root().join("suite/public/favicon.ico");
        match tokio::fs::read(&favicon_path).await {
            Ok(bytes) => {
                (StatusCode::OK, [("content-type", "image/x-icon")], bytes).into_response()
            }
            Err(_) => StatusCode::NOT_FOUND.into_response(),
        }
    }
}

#[cfg(feature = "embed-ui")]
pub async fn handle_embedded_asset(
    axum::extract::Path((dir, path)): axum::extract::Path<(String, String)>,
) -> impl IntoResponse {
    if !SUITE_DIRS.contains(&dir.as_str()) {
        return StatusCode::NOT_FOUND.into_response();
    }

    let asset_path = format!("suite/{}/{}", dir, path);
    match Assets::get(&asset_path) {
        Some(content) => {
            let mime = mime_guess::from_path(&asset_path).first_or_octet_stream();
            (
                [(axum::http::header::CONTENT_TYPE, mime.as_ref())],
                content.data,
            )
                .into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

#[cfg(feature = "embed-ui")]
pub async fn handle_embedded_root_asset(
    axum::extract::OriginalUri(uri): axum::extract::OriginalUri,
) -> impl IntoResponse {
    // The `/suite/{file}` routes are registered as literal paths (one per
    // ROOT_FILES entry), so the handler must not declare a `Path` extractor:
    // axum would reject the request with "Expected 1 but got 0". The
    // filename is derived from the request URI instead.
    let filename = uri.path().rsplit('/').next().unwrap_or("");
    if !ROOT_FILES.contains(&filename) {
        return StatusCode::NOT_FOUND.into_response();
    }

    let asset_path = format!("suite/{}", filename);
    match Assets::get(&asset_path) {
        Some(content) => {
            let mime = mime_guess::from_path(&asset_path).first_or_octet_stream();
            (
                [(axum::http::header::CONTENT_TYPE, mime.as_ref())],
                content.data,
            )
                .into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

/// Serves embedded site-root assets (`manifest.webmanifest`, `sw.js`)
/// at the `/` scope with a correct MIME type. Without this dedicated route,
/// those requests fall through to the desktop.html catch-all and the PWA
/// manifest is delivered as HTML, which Chrome rejects with a parse error.
#[cfg(feature = "embed-ui")]
pub async fn handle_site_root_asset(
    axum::extract::OriginalUri(uri): axum::extract::OriginalUri,
) -> impl IntoResponse {
    // The `/manifest.webmanifest` and `/sw.js` routes are registered as
    // literal paths, so the handler must not declare a `Path` extractor:
    // axum would reject the request with "Expected 1 but got 0". The
    // filename is derived from the request URI instead.
    let filename = uri.path().trim_start_matches('/');
    if !SITE_ROOT_FILES.contains(&filename) {
        return StatusCode::NOT_FOUND.into_response();
    }

    match Assets::get(&filename) {
        Some(content) => {
            let mime = mime_guess::from_path(&filename).first_or_octet_stream();
            (
                [(axum::http::header::CONTENT_TYPE, mime.as_ref())],
                content.data,
            )
                .into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}



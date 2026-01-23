use crate::core::shared::get_content_type;
use crate::shared::state::AppState;
use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use log::{error, info, trace, warn};
use std::sync::Arc;

/// Rewrite CDN URLs to local paths for HTMX and other vendor libraries
/// This ensures old apps with CDN references still work with local files
#[derive(Debug, serde::Deserialize)]
pub struct VendorFilePath {
    pub file_path: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct SuiteJsFilePath {
    pub file_path: String,
}

pub async fn serve_suite_js_file(
    State(state): State<Arc<AppState>>,
    Path(params): Path<SuiteJsFilePath>,
) -> Response {
    let file_path = sanitize_file_path(&params.file_path);

    if file_path.is_empty() {
        return (StatusCode::BAD_REQUEST, "Invalid path").into_response();
    }

    if file_path.starts_with("vendor/") || file_path.starts_with("vendor\\") {
        return serve_vendor_file(State(state), Path(VendorFilePath {
            file_path: file_path.strip_prefix("vendor/").unwrap_or(&file_path).to_string()
        })).await;
    }

    if !file_path.ends_with(".js") {
        return (StatusCode::BAD_REQUEST, "Only JS files allowed").into_response();
    }

    let content_type = get_content_type(&file_path);

    let ui_path = std::env::var("BOTUI_PATH").unwrap_or_else(|_| "./botui/ui/suite".to_string());
    let local_path = format!("{}/js/{}", ui_path, file_path);

    match tokio::fs::read(&local_path).await {
        Ok(content) => {
            trace!("Serving suite JS file from: {}", local_path);
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, content_type)
                .header(header::CACHE_CONTROL, "public, max-age=3600")
                .body(Body::from(content))
                .unwrap_or_else(|_| {
                    (StatusCode::INTERNAL_SERVER_ERROR, "Failed to build response")
                        .into_response()
                })
        }
        Err(e) => {
            warn!("Suite JS file not found: {} - {}", local_path, e);
            (StatusCode::NOT_FOUND, "JS file not found").into_response()
        }
    }
}

pub async fn serve_vendor_file(
    State(state): State<Arc<AppState>>,
    Path(params): Path<VendorFilePath>,
) -> Response {
    let file_path = sanitize_file_path(&params.file_path);

    if file_path.is_empty() {
        return (StatusCode::BAD_REQUEST, "Invalid path").into_response();
    }

    let content_type = get_content_type(&file_path);

    let local_paths = [
        format!("./botui/ui/suite/js/vendor/{}", file_path),
        format!("./botserver-stack/static/js/vendor/{}", file_path),
    ];

    for local_path in &local_paths {
        if let Ok(content) = tokio::fs::read(local_path).await {
            trace!("Serving vendor file from local path: {}", local_path);
            return Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, content_type)
                .header(header::CACHE_CONTROL, "public, max-age=86400")
                .body(Body::from(content))
                .unwrap_or_else(|_| {
                    (StatusCode::INTERNAL_SERVER_ERROR, "Failed to build response")
                        .into_response()
                });
        }
    }

    let bot_name = state.bucket_name
        .trim_end_matches(".gbai")
        .to_string();
    let sanitized_bot_name = bot_name.to_lowercase().replace(' ', "-").replace('_', "-");

    let bucket = format!("{}.gbai", sanitized_bot_name);
    let key = format!("{}.gblib/vendor/{}", sanitized_bot_name, file_path);

    trace!("Trying MinIO for vendor file: bucket={}, key={}", bucket, key);

    if let Some(ref drive) = state.drive {
        match drive
            .get_object()
            .bucket(&bucket)
            .key(&key)
            .send()
            .await
        {
            Ok(response) => {
                match response.body.collect().await {
                    Ok(body) => {
                        let content = body.into_bytes();

                        return Response::builder()
                            .status(StatusCode::OK)
                            .header(header::CONTENT_TYPE, content_type)
                            .header(header::CACHE_CONTROL, "public, max-age=86400")
                            .body(Body::from(content.to_vec()))
                            .unwrap_or_else(|_| {
                                (StatusCode::INTERNAL_SERVER_ERROR, "Failed to build response")
                                    .into_response()
                            });
                    }
                    Err(e) => {
                        error!("Failed to read MinIO response body: {}", e);
                    }
                }
            }
            Err(e) => {
                warn!("MinIO get_object failed for {}/{}: {}", bucket, key, e);
            }
        }
    }

    (StatusCode::NOT_FOUND, "Vendor file not found").into_response()
}

fn rewrite_cdn_urls(html: &str) -> String {
    html
        // HTMX from various CDNs
        .replace("https://unpkg.com/htmx.org@1.9.10", "/js/vendor/htmx.min.js")
        .replace("https://unpkg.com/htmx.org@1.9.10/dist/htmx.min.js", "/js/vendor/htmx.min.js")
        .replace("https://unpkg.com/htmx.org@1.9.11", "/js/vendor/htmx.min.js")
        .replace("https://unpkg.com/htmx.org@1.9.11/dist/htmx.min.js", "/js/vendor/htmx.min.js")
        .replace("https://unpkg.com/htmx.org@1.9.12", "/js/vendor/htmx.min.js")
        .replace("https://unpkg.com/htmx.org@1.9.12/dist/htmx.min.js", "/js/vendor/htmx.min.js")
        .replace("https://unpkg.com/htmx.org", "/js/vendor/htmx.min.js")
        .replace("https://cdn.jsdelivr.net/npm/htmx.org", "/js/vendor/htmx.min.js")
        .replace("https://cdnjs.cloudflare.com/ajax/libs/htmx/1.9.10/htmx.min.js", "/js/vendor/htmx.min.js")
        .replace("https://cdnjs.cloudflare.com/ajax/libs/htmx/1.9.11/htmx.min.js", "/js/vendor/htmx.min.js")
        .replace("https://cdnjs.cloudflare.com/ajax/libs/htmx/1.9.12/htmx.min.js", "/js/vendor/htmx.min.js")
}

pub fn configure_app_server_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Serve shared vendor files from MinIO: /js/vendor/*
        .route("/js/vendor/*file_path", get(serve_vendor_file))
        // Serve suite JS files (i18n.js, theme-manager.js, etc.)
        .route("/js/*file_path", get(serve_suite_js_file))
        // Serve app files: /apps/{app_name}/* (clean URLs)
        .route("/apps/:app_name", get(serve_app_index))
        .route("/apps/:app_name/", get(serve_app_index))
        .route("/apps/:app_name/*file_path", get(serve_app_file))
        // List all available apps
        .route("/apps", get(list_all_apps))
}

#[derive(Debug, serde::Deserialize)]
pub struct AppPath {
    pub app_name: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct AppFilePath {
    pub app_name: String,
    pub file_path: String,
}

pub async fn serve_app_index(
    State(state): State<Arc<AppState>>,
    Path(params): Path<AppPath>,
    original_uri: axum::extract::OriginalUri,
) -> impl IntoResponse {
    // Redirect to trailing slash so relative paths resolve correctly
    // /apps/calc-pro -> /apps/calc-pro/
    let path = original_uri.path();
    if !path.ends_with('/') {
        return axum::response::Redirect::permanent(&format!("{}/", path)).into_response();
    }
    serve_app_file_internal(&state, &params.app_name, "index.html").await
}

pub async fn serve_app_file(
    State(state): State<Arc<AppState>>,
    Path(params): Path<AppFilePath>,
) -> impl IntoResponse {
    serve_app_file_internal(&state, &params.app_name, &params.file_path).await
}

/// Sanitize app name - only alphanumeric, underscore, hyphen allowed
fn sanitize_app_name(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .collect::<String>()
}

/// Sanitize file path - preserve directory structure but remove dangerous characters
fn sanitize_file_path(path: &str) -> String {
    path.split('/')
        .filter(|segment| !segment.is_empty() && *segment != ".." && *segment != ".")
        .map(|segment| {
            segment
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-' || *c == '.')
                .collect::<String>()
        })
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("/")
}

async fn serve_app_file_internal(state: &AppState, app_name: &str, file_path: &str) -> Response {
    let sanitized_app_name = sanitize_app_name(app_name);
    let sanitized_file_path = sanitize_file_path(file_path);

    if sanitized_app_name.is_empty() || sanitized_file_path.is_empty() {
        return (StatusCode::BAD_REQUEST, "Invalid path").into_response();
    }

    // Get bot name from bucket_name config (default to "default")
    let bot_name = state.bucket_name
        .trim_end_matches(".gbai")
        .to_string();
    let sanitized_bot_name = bot_name.to_lowercase().replace(' ', "-").replace('_', "-");

    // MinIO bucket and path: botname.gbai / botname.gbapp/appname/file
    let bucket = format!("{}.gbai", sanitized_bot_name);
    let key = format!("{}.gbapp/{}/{}", sanitized_bot_name, sanitized_app_name, sanitized_file_path);

    info!("Serving app file from MinIO: bucket={}, key={}", bucket, key);

    // Try to serve from MinIO
    if let Some(ref drive) = state.drive {
        match drive
            .get_object()
            .bucket(&bucket)
            .key(&key)
            .send()
            .await
        {
            Ok(response) => {
                match response.body.collect().await {
                    Ok(body) => {
                        let content = body.into_bytes();
                        let content_type = get_content_type(&sanitized_file_path);

                        // For HTML files, rewrite CDN URLs to local paths
                        let final_content = if content_type.starts_with("text/html") {
                            let html = String::from_utf8_lossy(&content);
                            let rewritten = rewrite_cdn_urls(&html);
                            rewritten.into_bytes()
                        } else {
                            content.to_vec()
                        };

                        return Response::builder()
                            .status(StatusCode::OK)
                            .header(header::CONTENT_TYPE, content_type)
                            .header(header::CACHE_CONTROL, "public, max-age=3600")
                            .body(Body::from(final_content))
                            .unwrap_or_else(|_| {
                                (StatusCode::INTERNAL_SERVER_ERROR, "Failed to build response")
                                    .into_response()
                            });
                    }
                    Err(e) => {
                        error!("Failed to read MinIO response body: {}", e);
                    }
                }
            }
            Err(e) => {
                warn!("MinIO get_object failed for {}/{}: {}", bucket, key, e);
            }
        }
    }

    // Fallback to filesystem if MinIO fails
    let site_path = state
        .config
        .as_ref()
        .map(|c| c.site_path.clone())
        .unwrap_or_else(|| "./botserver-stack/sites".to_string());

    let full_path = format!(
        "{}/{}.gbai/{}.gbapp/{}/{}",
        site_path, sanitized_bot_name, sanitized_bot_name, sanitized_app_name, sanitized_file_path
    );

    trace!("Fallback: serving app file from filesystem: {full_path}");

    let path = std::path::Path::new(&full_path);
    if !path.exists() {
        warn!("App file not found: {full_path}");
        return (StatusCode::NOT_FOUND, "File not found").into_response();
    }

    let content_type = get_content_type(&sanitized_file_path);

    match std::fs::read(&full_path) {
        Ok(contents) => {
            // For HTML files, rewrite CDN URLs to local paths
            let final_content = if content_type.starts_with("text/html") {
                let html = String::from_utf8_lossy(&contents);
                rewrite_cdn_urls(&html).into_bytes()
            } else {
                contents
            };

            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, content_type)
                .header(header::CACHE_CONTROL, "public, max-age=3600")
                .body(Body::from(final_content))
                .unwrap_or_else(|_| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to build response",
                    )
                        .into_response()
                })
        }
        Err(e) => {
            error!("Failed to read file {full_path}: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read file").into_response()
        }
    }
}

pub async fn list_all_apps(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let site_path = state
        .config
        .as_ref()
        .map(|c| c.site_path.clone())
        .unwrap_or_else(|| "./botserver-stack/sites".to_string());

    let mut apps = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&site_path) {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                if let Some(name) = entry.file_name().to_str() {
                    if name.starts_with('.') || name.to_lowercase().ends_with(".gbai") {
                        continue;
                    }

                    let app_path = entry.path();
                    let has_index = app_path.join("index.html").exists();

                    if has_index {
                        apps.push(serde_json::json!({
                            "name": name,
                            "url": format!("/apps/{}", name),
                            "has_index": true
                        }));
                    }
                }
            }
        }
    }

    (
        StatusCode::OK,
        axum::Json(serde_json::json!({
            "apps": apps,
            "count": apps.len()
        })),
    )
        .into_response()
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::sanitize_path_component;

    #[test]
    fn test_sanitize_path_component() {
        assert_eq!(sanitize_path_component("clinic"), "clinic");
        assert_eq!(sanitize_path_component("../etc/passwd"), "etc/passwd");
        assert_eq!(sanitize_path_component("app/../secret"), "app/secret");
        assert_eq!(sanitize_path_component("/leading/slash"), "leading/slash");
        assert_eq!(sanitize_path_component("file.html"), "file.html");
        assert_eq!(sanitize_path_component("my-app_v2"), "my-app_v2");
    }

    #[test]
    fn test_get_content_type() {
        assert_eq!(get_content_type("index.html"), "text/html; charset=utf-8");
        assert_eq!(get_content_type("styles.css"), "text/css; charset=utf-8");
        assert_eq!(
            get_content_type("app.js"),
            "application/javascript; charset=utf-8"
        );
        assert_eq!(get_content_type("image.png"), "image/png");
        assert_eq!(get_content_type("unknown.xyz"), "application/octet-stream");
    }

    #[test]
    fn test_sanitize_app_name() {
        assert_eq!(sanitize_app_name("my-app"), "my-app");
        assert_eq!(sanitize_app_name("my_app_123"), "my_app_123");
        assert_eq!(sanitize_app_name("../hack"), "hack");
        assert_eq!(sanitize_app_name("app<script>"), "appscript");
    }

    #[test]
    fn test_sanitize_file_path() {
        assert_eq!(sanitize_file_path("styles.css"), "styles.css");
        assert_eq!(sanitize_file_path("css/styles.css"), "css/styles.css");
        assert_eq!(sanitize_file_path("assets/img/logo.png"), "assets/img/logo.png");
        assert_eq!(sanitize_file_path("../../../etc/passwd"), "etc/passwd");
        assert_eq!(sanitize_file_path("./styles.css"), "styles.css");
        assert_eq!(sanitize_file_path("path//double//slash.js"), "path/double/slash.js");
    }
}

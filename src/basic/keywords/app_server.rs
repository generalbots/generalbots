//! App Server Module
//!
//! Serves generated HTMX applications with clean URLs.
//! Apps are synced from drive to SITE_ROOT/{app_name}/ for serving.
//!
//! URL structure:
//! - `/apps/{app_name}/` -> {site_path}/{app_name}/index.html
//! - `/apps/{app_name}/patients.html` -> {site_path}/{app_name}/patients.html
//! - `/apps/{app_name}/styles.css` -> {site_path}/{app_name}/styles.css
//!
//! Flow:
//! 1. AppGenerator writes to S3 drive: {bucket}/.gbdrive/apps/{app_name}/
//! 2. sync_app_to_site_root() copies to: {site_path}/{app_name}/
//! 3. This module serves from: {site_path}/{app_name}/

use crate::shared::state::AppState;
use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use log::{error, trace, warn};
use std::sync::Arc;

/// Configure routes for serving generated apps
pub fn configure_app_server_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Serve app files: /apps/{app_name}/* (clean URLs)
        .route("/apps/:app_name", get(serve_app_index))
        .route("/apps/:app_name/", get(serve_app_index))
        .route("/apps/:app_name/*file_path", get(serve_app_file))
        // List all available apps
        .route("/apps", get(list_all_apps))
}

/// Path parameters for app serving
#[derive(Debug, serde::Deserialize)]
pub struct AppPath {
    pub app_name: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct AppFilePath {
    pub app_name: String,
    pub file_path: String,
}

/// Serve the index.html for an app
pub async fn serve_app_index(
    State(state): State<Arc<AppState>>,
    Path(params): Path<AppPath>,
) -> impl IntoResponse {
    serve_app_file_internal(&state, &params.app_name, "index.html").await
}

/// Serve any file from an app directory
pub async fn serve_app_file(
    State(state): State<Arc<AppState>>,
    Path(params): Path<AppFilePath>,
) -> impl IntoResponse {
    serve_app_file_internal(&state, &params.app_name, &params.file_path).await
}

/// Internal function to serve files from app directory
async fn serve_app_file_internal(state: &AppState, app_name: &str, file_path: &str) -> Response {
    // Sanitize paths to prevent directory traversal
    let sanitized_app_name = sanitize_path_component(app_name);
    let sanitized_file_path = sanitize_path_component(file_path);

    if sanitized_app_name.is_empty() || sanitized_file_path.is_empty() {
        return (StatusCode::BAD_REQUEST, "Invalid path").into_response();
    }

    // Construct full file path from SITE_ROOT
    // Apps are synced to: {site_path}/{app_name}/
    let site_path = state
        .config
        .as_ref()
        .map(|c| c.site_path.clone())
        .unwrap_or_else(|| "./botserver-stack/sites".to_string());

    let full_path = format!(
        "{}/{}/{}",
        site_path, sanitized_app_name, sanitized_file_path
    );

    trace!("Serving app file: {}", full_path);

    // Check if file exists
    let path = std::path::Path::new(&full_path);
    if !path.exists() {
        warn!("App file not found: {}", full_path);
        return (StatusCode::NOT_FOUND, "File not found").into_response();
    }

    // Determine content type
    let content_type = get_content_type(&sanitized_file_path);

    // Read and serve the file
    match std::fs::read(&full_path) {
        Ok(contents) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, content_type)
            .header(header::CACHE_CONTROL, "public, max-age=3600")
            .body(Body::from(contents))
            .unwrap_or_else(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to build response",
                )
                    .into_response()
            }),
        Err(e) => {
            error!("Failed to read file {}: {}", full_path, e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read file").into_response()
        }
    }
}

/// List all available apps from SITE_ROOT
pub async fn list_all_apps(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let site_path = state
        .config
        .as_ref()
        .map(|c| c.site_path.clone())
        .unwrap_or_else(|| "./botserver-stack/sites".to_string());

    let mut apps = Vec::new();

    // List all directories in site_path that have an index.html (are apps)
    if let Ok(entries) = std::fs::read_dir(&site_path) {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                if let Some(name) = entry.file_name().to_str() {
                    // Skip .gbai directories and other system folders
                    if name.starts_with('.') || name.ends_with(".gbai") {
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

/// Sanitize path component to prevent directory traversal
fn sanitize_path_component(component: &str) -> String {
    component
        .replace("..", "")
        .replace("//", "/")
        .trim_start_matches('/')
        .trim_end_matches('/')
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_' || *c == '.' || *c == '/')
        .collect()
}

/// Get content type based on file extension
fn get_content_type(file_path: &str) -> &'static str {
    let ext = file_path.rsplit('.').next().unwrap_or("").to_lowercase();

    match ext.as_str() {
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "application/javascript; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "eot" => "application/vnd.ms-fontobject",
        "txt" => "text/plain; charset=utf-8",
        "xml" => "application/xml; charset=utf-8",
        "pdf" => "application/pdf",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}

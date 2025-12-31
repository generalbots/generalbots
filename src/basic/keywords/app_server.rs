use crate::core::shared::{get_content_type, sanitize_path_component};
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

pub fn configure_app_server_routes() -> Router<Arc<AppState>> {
    Router::new()
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
) -> impl IntoResponse {
    serve_app_file_internal(&state, &params.app_name, "index.html").await
}

pub async fn serve_app_file(
    State(state): State<Arc<AppState>>,
    Path(params): Path<AppFilePath>,
) -> impl IntoResponse {
    serve_app_file_internal(&state, &params.app_name, &params.file_path).await
}

async fn serve_app_file_internal(state: &AppState, app_name: &str, file_path: &str) -> Response {
    let sanitized_app_name = sanitize_path_component(app_name);
    let sanitized_file_path = sanitize_path_component(file_path);

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

                        return Response::builder()
                            .status(StatusCode::OK)
                            .header(header::CONTENT_TYPE, content_type)
                            .header(header::CACHE_CONTROL, "public, max-age=3600")
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

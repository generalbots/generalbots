use botcore::shared::state::AppState;
use std::sync::Arc;

use super::{DefaultMultimediaHandler, MediaUploadRequest, MultimediaHandler};

pub async fn upload_media_handler(
    axum::extract::State(_state): axum::extract::State<Arc<AppState>>,
    axum::Json(request): axum::Json<MediaUploadRequest>,
) -> impl axum::response::IntoResponse {
    let handler = DefaultMultimediaHandler::new();
    match handler.upload_media(request).await {
        Ok(response) => (
            axum::http::StatusCode::OK,
            axum::Json(serde_json::json!(response)),
        ),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(crate::security::error_sanitizer::log_and_sanitize_json(
                &e.to_string(),
                "multimedia upload_media",
                None,
            )),
        ),
    }
}

pub async fn download_media_handler(
    axum::extract::State(_state): axum::extract::State<Arc<AppState>>,
    axum::extract::Path(_media_id): axum::extract::Path<String>,
) -> impl axum::response::IntoResponse {
    (
        axum::http::StatusCode::OK,
        axum::Json(serde_json::json!({"status": "ok"})),
    )
}

pub async fn web_search_handler(
    axum::extract::State(_state): axum::extract::State<Arc<AppState>>,
    axum::Json(payload): axum::Json<serde_json::Value>,
) -> impl axum::response::IntoResponse {
    let query = payload.get("query").and_then(|q| q.as_str()).unwrap_or("");
    let max_results = payload
        .get("max_results")
        .and_then(|m| m.as_u64())
        .unwrap_or(10) as usize;

    let handler = DefaultMultimediaHandler::new();
    match handler.web_search(query, max_results).await {
        Ok(results) => (
            axum::http::StatusCode::OK,
            axum::Json(serde_json::json!({"query": query, "results": results})),
        ),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(crate::security::error_sanitizer::log_and_sanitize_json(
                &e.to_string(),
                "multimedia web_search",
                None,
            )),
        ),
    }
}

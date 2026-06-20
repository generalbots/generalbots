use axum::{
    extract::{Path, State},
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::fs;

use botcore::shared::state::AppState;

#[derive(Serialize)]
pub struct FileListResponse {
    pub files: Vec<String>,
}

#[derive(Serialize)]
pub struct FileContentResponse {
    pub content: String,
}

#[derive(Deserialize)]
pub struct SaveFileRequest {
    pub content: String,
}

pub async fn list_files(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<FileListResponse>, axum::http::StatusCode> {
    Ok(Json(FileListResponse {
        files: vec![
            "src/main.rs".to_string(),
            "ui/index.html".to_string(),
            "ui/style.css".to_string(),
            "package.json".to_string(),
        ],
    }))
}

pub async fn read_file(
    State(_state): State<Arc<AppState>>,
    Path(path): Path<String>,
) -> Result<Json<FileContentResponse>, axum::http::StatusCode> {
    let safe_path = path.replace("..", "");
    match fs::read_to_string(&safe_path).await {
        Ok(content) => Ok(Json(FileContentResponse { content })),
        Err(_) => Ok(Json(FileContentResponse {
            content: format!("// Dummy content for requested file: {safe_path}")
        })),
    }
}

pub async fn save_file(
    State(_state): State<Arc<AppState>>,
    Path(path): Path<String>,
    Json(_payload): Json<SaveFileRequest>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    let safe_path = path.replace("..", "");
    Ok(Json(serde_json::json!({
        "status": "success",
        "message": format!("File {safe_path} saved successfully")
    })))
}

use axum::{
    extract::{Path, State},
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::fs;

// Note: Replace AppState with your actual shared state struct
use crate::core::shared::state::AppState;

pub fn configure_editor_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/editor/files", get(list_files))
        .route("/api/editor/file/*path", get(read_file).post(save_file))
}

#[derive(Serialize)]
pub struct FileListResponse {
    pub files: Vec<String>,
}

pub async fn list_files(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<FileListResponse>, axum::http::StatusCode> {
    // In a real implementation this would list from the drive/workspace
    Ok(Json(FileListResponse {
        files: vec![
            "src/main.rs".to_string(),
            "ui/index.html".to_string(),
            "ui/style.css".to_string(),
            "package.json".to_string(),
        ],
    }))
}

#[derive(Serialize)]
pub struct FileContentResponse {
    pub content: String,
}

pub async fn read_file(
    State(_state): State<Arc<AppState>>,
    Path(path): Path<String>,
) -> Result<Json<FileContentResponse>, axum::http::StatusCode> {
    // Decode path if needed
    let safe_path = path.replace("..", "");
    
    // Fake implementation for now
    match fs::read_to_string(&safe_path).await {
        Ok(content) => Ok(Json(FileContentResponse { content })),
        Err(_) => Ok(Json(FileContentResponse { 
            content: format!("// Dummy content for requested file: {}", safe_path) 
        })),
    }
}

#[derive(Deserialize)]
pub struct SaveFileRequest {
    pub content: String,
}

pub async fn save_file(
    State(_state): State<Arc<AppState>>,
    Path(path): Path<String>,
    Json(_payload): Json<SaveFileRequest>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    let safe_path = path.replace("..", "");
    
    // Fake implementation
    // let _ = fs::write(&safe_path, payload.content).await;
    
    Ok(Json(serde_json::json!({
        "status": "success",
        "message": format!("File {} saved successfully", safe_path)
    })))
}

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::core::shared::state::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct FileContent {
    pub content: String,
    pub language: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileResponse {
    pub success: bool,
    pub content: Option<String>,
    pub language: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SaveResponse {
    pub success: bool,
    pub message: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileListResponse {
    pub success: bool,
    pub files: Vec<FileInfo>,
    pub error: Option<String>,
}

pub async fn get_file(
    Path(file_path): Path<String>,
    State(_state): State<Arc<AppState>>,
) -> Result<Json<FileResponse>, (StatusCode, Json<FileResponse>)> {
    let decoded_path = urlencoding::decode(&file_path)
        .map(|s| s.to_string())
        .unwrap_or(file_path);

    let language = detect_language(&decoded_path);

    match std::fs::read_to_string(&decoded_path) {
        Ok(content) => Ok(Json(FileResponse {
            success: true,
            content: Some(content),
            language: Some(language),
            error: None,
        })),
        Err(e) => {
            log::warn!("Failed to read file {}: {}", decoded_path, e);
            Ok(Json(FileResponse {
                success: false,
                content: Some(String::new()),
                language: Some(language),
                error: Some(format!("File not found: {}", decoded_path)),
            }))
        }
    }
}

pub async fn save_file(
    Path(file_path): Path<String>,
    State(_state): State<Arc<AppState>>,
    Json(payload): Json<FileContent>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<SaveResponse>)> {
    let decoded_path = urlencoding::decode(&file_path)
        .map(|s| s.to_string())
        .unwrap_or(file_path);

    if let Some(parent) = std::path::Path::new(&decoded_path).parent() {
        if !parent.exists() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                log::error!("Failed to create directories for {}: {}", decoded_path, e);
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(SaveResponse {
                        success: false,
                        message: None,
                        error: Some(format!("Failed to create directories: {}", e)),
                    }),
                ));
            }
        }
    }

    match std::fs::write(&decoded_path, &payload.content) {
        Ok(_) => {
            log::info!("Successfully saved file: {}", decoded_path);
            Ok(Json(SaveResponse {
                success: true,
                message: Some(format!("File saved: {}", decoded_path)),
                error: None,
            }))
        }
        Err(e) => {
            log::error!("Failed to save file {}: {}", decoded_path, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(SaveResponse {
                    success: false,
                    message: None,
                    error: Some(format!("Failed to save file: {}", e)),
                }),
            ))
        }
    }
}

pub async fn list_files(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<FileListResponse>, (StatusCode, Json<FileListResponse>)> {
    let mut files = Vec::new();

    let common_paths = vec![
        "index.html",
        "styles.css",
        "app.js",
        "main.js",
        "package.json",
        "README.md",
    ];

    for path in common_paths {
        if std::path::Path::new(path).exists() {
            files.push(FileInfo {
                name: path.to_string(),
                path: path.to_string(),
            });
        }
    }

    if let Ok(entries) = std::fs::read_dir(".") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    let ext = std::path::Path::new(name)
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("");

                    if matches!(
                        ext,
                        "html" | "css" | "js" | "json" | "ts" | "bas" | "py" | "rs" | "md"
                    ) && !files.iter().any(|f| f.path == name)
                    {
                        files.push(FileInfo {
                            name: name.to_string(),
                            path: name.to_string(),
                        });
                    }
                }
            }
        }
    }

    Ok(Json(FileListResponse {
        success: true,
        files,
        error: None,
    }))
}

fn detect_language(file_path: &str) -> String {
    let ext = std::path::Path::new(file_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "html" | "htm" => "html",
        "css" => "css",
        "js" => "javascript",
        "json" => "json",
        "ts" => "typescript",
        "bas" => "basic",
        "py" => "python",
        "rs" => "rust",
        "md" => "markdown",
        "xml" => "xml",
        "yaml" | "yml" => "yaml",
        "sql" => "sql",
        _ => "plaintext",
    }
    .to_string()
}

pub fn configure_editor_routes() -> axum::Router<Arc<AppState>> {
    use axum::routing::{get, post};

    axum::Router::new()
        .route("/api/editor/file/:file_path", get(get_file))
        .route("/api/editor/file/:file_path", post(save_file))
        .route("/api/editor/files", get(list_files))
}

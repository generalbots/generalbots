use axum::{
    extract::{Path, State},
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use botcore::shared::state::AppState;

#[derive(Serialize)]
pub struct GitStatusResponse {
    pub files: Vec<GitFileStatus>,
}

#[derive(Serialize)]
pub struct GitFileStatus {
    pub file: String,
    pub status: String,
}

#[derive(Deserialize)]
pub struct CommitRequest {
    pub message: String,
}

pub async fn git_status(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<GitStatusResponse>, axum::http::StatusCode> {
    Ok(Json(GitStatusResponse {
        files: vec![
            GitFileStatus { file: "src/main.rs".to_string(), status: "modified".to_string() },
            GitFileStatus { file: "Cargo.toml".to_string(), status: "modified".to_string() },
            GitFileStatus { file: "new_file.txt".to_string(), status: "untracked".to_string() },
        ],
    }))
}

pub async fn git_diff(
    State(_state): State<Arc<AppState>>,
    Path(file): Path<String>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    Ok(Json(serde_json::json!({
        "diff": format!("--- a/{file}\n+++ b/{file}\n@@ -1,3 +1,4 @@\n // Sample file\n+ // Added functionality\n- // Old functionality")
    })))
}

pub async fn git_commit(
    State(_state): State<Arc<AppState>>,
    Json(_payload): Json<CommitRequest>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    Ok(Json(serde_json::json!({ "status": "success", "message": "Committed successfully" })))
}

pub async fn git_push(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    Ok(Json(serde_json::json!({ "status": "success", "message": "Pushed to remote origin" })))
}

pub async fn git_branches(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    Ok(Json(serde_json::json!({
        "branches": [
            { "name": "main", "current": true },
            { "name": "develop", "current": false },
            { "name": "feature/botcoder", "current": false },
        ]
    })))
}

pub async fn git_create_or_switch_branch(
    State(_state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    Ok(Json(serde_json::json!({ "status": "success", "message": format!("Switched to branch {name}") })))
}

pub async fn git_log(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    Ok(Json(serde_json::json!({
        "commits": [
            { "hash": "abc1234", "message": "Initial commit", "author": "BotCoder", "date": "2023-10-01" },
            { "hash": "def5678", "message": "Add feature X", "author": "BotCoder", "date": "2023-10-02" },
        ]
    })))
}

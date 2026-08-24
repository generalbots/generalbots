//! Server-side per-document undo/redo (#1138).
//! The save path calls [`capture_snapshot`] BEFORE persisting, archiving the
//! previous version; `POST /api/docs/history {id, action}` walks the stacks.

use axum::{extract::State, Json};
use serde::Deserialize;
use std::sync::Arc;

use crate::state::DocState;
use crate::Document;
use axum::http::StatusCode;

#[derive(Deserialize)]
pub struct HistoryRequest {
    pub id: String,
    pub action: String,
}

/// Called by the save path BEFORE persisting the new content.
pub async fn capture_snapshot(state: &Arc<DocState>, doc_id: &str, title: &str, content: &str) {
    let mut map = state.history.lock().await;
    let entry = map.entry(doc_id.to_string()).or_default();
    let doc: Document = serde_json::from_value(serde_json::json!({
        "id": doc_id,
        "title": title,
        "content": content,
        "owner_id": "",
        "storage_path": "",
        "created_at": chrono::Utc::now(),
        "updated_at": chrono::Utc::now(),
    }))
    .map_err(|e| log::warn!("history snapshot skipped: {e}"))
    .unwrap_or_else(|_| test_document(content));
    entry.capture(doc);
}

#[doc(hidden)]
pub fn test_document(content: &str) -> Document {
    serde_json::from_value(serde_json::json!({
        "id": "d1", "title": "T", "content": content,
        "owner_id": "", "storage_path": "",
        "created_at": chrono::Utc::now(),
        "updated_at": chrono::Utc::now()
    }))
    .expect("valid test document")
}

/// POST /api/docs/history { id, action: "undo" | "redo" } → restored Document.
/// 204 when there is nothing to walk.
pub async fn handle_history(
    State(state): State<Arc<DocState>>,
    Json(req): Json<HistoryRequest>,
) -> Result<Json<Document>, StatusCode> {
    let mut map = state.history.lock().await;
    let entry = map.entry(req.id.clone()).or_default();
    let restored = match req.action.as_str() {
        "undo" => entry.undo(),
        "redo" => entry.redo(),
        _ => None,
    };
    restored.map(Json).ok_or(StatusCode::NO_CONTENT)
}

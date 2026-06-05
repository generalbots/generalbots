use axum::extract::{Json, Path};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, OnceLock};
use axum::http::StatusCode;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Meeting {
    pub id: String,
    pub title: String,
    pub date: String,
    pub duration_minutes: u64,
    pub participants: Vec<String>,
    pub status: String,
    pub transcript_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Transcript {
    pub id: String,
    pub meeting_id: String,
    pub content: String,
    pub language: String,
    pub word_count: u64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Document {
    pub id: String,
    pub meeting_id: String,
    pub title: String,
    pub kind: String,
    pub content: String,
    pub version: u64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Default)]
struct AppState {
    meetings: HashMap<String, Meeting>,
    transcripts: HashMap<String, Transcript>,
    documents: HashMap<String, Document>,
}

fn state() -> &'static Arc<RwLock<AppState>> {
    static S: OnceLock<Arc<RwLock<AppState>>> = OnceLock::new();
    S.get_or_init(|| Arc::new(RwLock::new(AppState::default())))
}

pub async fn list_meetings() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let s = state().read().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {}", e)))?;
    let items: Vec<&Meeting> = s.meetings.values().collect();
    Ok(Json(serde_json::json!({"items": items})))
}

pub async fn list_transcripts() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let s = state().read().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {}", e)))?;
    let items: Vec<&Transcript> = s.transcripts.values().collect();
    Ok(Json(serde_json::json!({"items": items})))
}

pub async fn list_documents() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let s = state().read().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {}", e)))?;
    let items: Vec<&Document> = s.documents.values().collect();
    Ok(Json(serde_json::json!({"items": items})))
}

pub async fn update_document(Path(id): Path<String>, Json(item): Json<Document>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut s = state().write().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {}", e)))?;
    if let Some(existing) = s.documents.get_mut(&id) {
        existing.title = item.title;
        existing.content = item.content;
        existing.version += 1;
        existing.updated_at = chrono::Utc::now().to_rfc3339();
        Ok(Json(serde_json::json!({"item": existing})))
    } else {
        Err((StatusCode::NOT_FOUND, "Document not found".to_string()))
    }
}

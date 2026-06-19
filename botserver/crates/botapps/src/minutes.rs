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

#[derive(Debug, Deserialize)]
pub struct MeetingFormData {
    pub title: Option<String>,
    pub date: Option<String>,
    pub participants: Option<Vec<String>>,
    pub agenda: Option<String>,
    pub discussion: Option<String>,
    pub decisions: Option<String>,
    pub action_items: Option<Vec<String>>,
    pub status: Option<String>,
}

pub async fn start_meeting(Path(id): Path<String>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut s = state().write().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {e}")))?;
    let meeting = Meeting {
        id: id.clone(),
        title: format!("Meeting {id}"),
        date: chrono::Utc::now().to_rfc3339(),
        duration_minutes: 0,
        participants: vec![],
        status: "in_progress".to_string(),
        transcript_id: None,
    };
    s.meetings.insert(id.clone(), meeting);
    Ok(Json(serde_json::json!({"success": true, "id": id})))
}

pub async fn update_meeting(Path(id): Path<String>, Json(item): Json<MeetingFormData>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut s = state().write().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {e}")))?;
    if let Some(existing) = s.meetings.get_mut(&id) {
        if let Some(title) = item.title {
            existing.title = title;
        }
        if let Some(date) = item.date {
            existing.date = date;
        }
        if let Some(participants) = item.participants {
            existing.participants = participants;
        }
        if let Some(status) = item.status {
            existing.status = status;
        }
        Ok(Json(serde_json::json!({"success": true, "item": existing})))
    } else {
        Err((StatusCode::NOT_FOUND, "Meeting not found".to_string()))
    }
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

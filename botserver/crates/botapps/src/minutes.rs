use axum::{extract::{State, Json, Path}, routing::get, Router};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Meeting {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub organizer: String,
    pub attendees: String,
    pub scheduled_at: String,
    pub duration_minutes: u32,
    pub location: Option<String>,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Transcript {
    pub id: Uuid,
    pub meeting_id: Uuid,
    pub speaker: String,
    pub content: String,
    pub timestamp: String,
    pub confidence: f64,
    pub language: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MinutesDocument {
    pub id: Uuid,
    pub meeting_id: Uuid,
    pub title: String,
    pub content: String,
    pub action_items: String,
    pub summary: String,
    pub version: u32,
    pub status: String,
    pub created_at: String,
}

#[derive(Default)]
pub struct MinutesState {
    pub meetings: HashMap<Uuid, Meeting>,
    pub transcripts: HashMap<Uuid, Transcript>,
    pub documents: HashMap<Uuid, MinutesDocument>,
}

pub fn routes() -> axum::Router {
    let state = Arc::new(RwLock::new(MinutesState::default()));
    Router::new()
        .route("/api/minutes/meetings", get(list_meetings).post(create_meeting))
        .route("/api/minutes/meetings/{id}", get(get_meeting).put(update_meeting).delete(delete_meeting))
        .route("/api/minutes/transcripts", get(list_transcripts).post(create_transcript))
        .route("/api/minutes/transcripts/{id}", get(get_transcript).delete(delete_transcript))
        .route("/api/minutes/documents", get(list_documents).post(create_document))
        .route("/api/minutes/documents/{id}", get(get_document).put(update_document).delete(delete_document))
        .with_state(state)
}

async fn list_meetings(State(state): State<Arc<RwLock<MinutesState>>>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    let items: Vec<&Meeting> = s.meetings.values().collect();
    Json(serde_json::json!({"meetings": items}))
}

async fn create_meeting(State(state): State<Arc<RwLock<MinutesState>>>, Json(mut meeting): Json<Meeting>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    let id = Uuid::new_v4();
    meeting.id = id;
    meeting.status = "Scheduled".to_string();
    meeting.created_at = Utc::now().to_rfc3339();
    s.meetings.insert(id, meeting.clone());
    Json(serde_json::json!({"meeting": meeting}))
}

async fn get_meeting(State(state): State<Arc<RwLock<MinutesState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    match s.meetings.get(&id) {
        Some(m) => Json(serde_json::json!({"meeting": m})),
        None => Json(serde_json::json!({"error": "Meeting not found"})),
    }
}

async fn update_meeting(State(state): State<Arc<RwLock<MinutesState>>>, Path(id): Path<Uuid>, Json(meeting): Json<Meeting>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    if let Some(existing) = s.meetings.get_mut(&id) {
        *existing = meeting.clone();
        existing.id = id;
        Json(serde_json::json!({"meeting": existing.clone()}))
    } else {
        Json(serde_json::json!({"error": "Meeting not found"}))
    }
}

async fn delete_meeting(State(state): State<Arc<RwLock<MinutesState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    s.meetings.remove(&id);
    Json(serde_json::json!({"deleted": true}))
}

async fn list_transcripts(State(state): State<Arc<RwLock<MinutesState>>>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    let items: Vec<&Transcript> = s.transcripts.values().collect();
    Json(serde_json::json!({"transcripts": items}))
}

async fn create_transcript(State(state): State<Arc<RwLock<MinutesState>>>, Json(mut t): Json<Transcript>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    let id = Uuid::new_v4();
    t.id = id;
    t.created_at = Utc::now().to_rfc3339();
    s.transcripts.insert(id, t.clone());
    Json(serde_json::json!({"transcript": t}))
}

async fn get_transcript(State(state): State<Arc<RwLock<MinutesState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    match s.transcripts.get(&id) {
        Some(t) => Json(serde_json::json!({"transcript": t})),
        None => Json(serde_json::json!({"error": "Transcript not found"})),
    }
}

async fn delete_transcript(State(state): State<Arc<RwLock<MinutesState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    s.transcripts.remove(&id);
    Json(serde_json::json!({"deleted": true}))
}

async fn list_documents(State(state): State<Arc<RwLock<MinutesState>>>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    let items: Vec<&MinutesDocument> = s.documents.values().collect();
    Json(serde_json::json!({"documents": items}))
}

async fn create_document(State(state): State<Arc<RwLock<MinutesState>>>, Json(mut doc): Json<MinutesDocument>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    let id = Uuid::new_v4();
    doc.id = id;
    doc.version = 1;
    doc.status = "Draft".to_string();
    doc.created_at = Utc::now().to_rfc3339();
    s.documents.insert(id, doc.clone());
    Json(serde_json::json!({"document": doc}))
}

async fn get_document(State(state): State<Arc<RwLock<MinutesState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    match s.documents.get(&id) {
        Some(d) => Json(serde_json::json!({"document": d})),
        None => Json(serde_json::json!({"error": "Document not found"})),
    }
}

async fn update_document(State(state): State<Arc<RwLock<MinutesState>>>, Path(id): Path<Uuid>, Json(doc): Json<MinutesDocument>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    if let Some(existing) = s.documents.get_mut(&id) {
        *existing = doc.clone();
        existing.id = id;
        existing.version += 1;
        Json(serde_json::json!({"document": existing.clone()}))
    } else {
        Json(serde_json::json!({"error": "Document not found"}))
    }
}

async fn delete_document(State(state): State<Arc<RwLock<MinutesState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    s.documents.remove(&id);
    Json(serde_json::json!({"deleted": true}))
}

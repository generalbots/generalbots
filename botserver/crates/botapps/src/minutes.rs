use axum::{
    extract::{Path, State as AxumState},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, put},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MeetingStatus {
    Scheduled,
    InProgress,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DocumentStatus {
    Draft,
    Approved,
    Signed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meeting {
    pub id: Uuid,
    pub title: String,
    pub scheduled_at: DateTime<Utc>,
    pub participants: Vec<String>,
    pub status: MeetingStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transcript {
    pub id: Uuid,
    pub meeting_id: Uuid,
    pub content: String,
    pub language: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinuteDocument {
    pub id: Uuid,
    pub meeting_id: Uuid,
    pub content: String,
    pub status: DocumentStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateMeetingRequest {
    pub title: String,
    pub scheduled_at: DateTime<Utc>,
    pub participants: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTranscriptRequest {
    pub meeting_id: Uuid,
    pub content: String,
    pub language: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateMinuteDocumentRequest {
    pub meeting_id: Uuid,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDocumentRequest {
    pub content: Option<String>,
    pub status: Option<DocumentStatus>,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

#[derive(Clone)]
pub struct MinutesState {
    pub meetings: Arc<RwLock<Vec<Meeting>>>,
    pub transcripts: Arc<RwLock<Vec<Transcript>>>,
    pub documents: Arc<RwLock<Vec<MinuteDocument>>>,
}

impl MinutesState {
    pub fn new() -> Self {
        Self {
            meetings: Arc::new(RwLock::new(Vec::new())),
            transcripts: Arc::new(RwLock::new(Vec::new())),
            documents: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

pub fn routes() -> Router {
    let state = MinutesState::new();
    Router::new()
        .route("/api/minutes/meetings", get(list_meetings).post(create_meeting))
        .route("/api/minutes/transcripts", get(list_transcripts).post(create_transcript))
        .route("/api/minutes/documents", get(list_documents).post(create_document))
        .route("/api/minutes/documents/:id", put(update_document))
        .with_state(state)
}

async fn list_meetings(
    AxumState(state): AxumState<MinutesState>,
) -> Result<Json<ApiResponse<Vec<Meeting>>>, StatusCode> {
    let meetings = state
        .meetings
        .read()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(ApiResponse {
        success: true,
        data: Some(meetings.clone()),
        error: None,
    }))
}

async fn create_meeting(
    AxumState(state): AxumState<MinutesState>,
    Json(payload): Json<CreateMeetingRequest>,
) -> Result<(StatusCode, Json<ApiResponse<Meeting>>), StatusCode> {
    let meeting = Meeting {
        id: Uuid::new_v4(),
        title: payload.title,
        scheduled_at: payload.scheduled_at,
        participants: payload.participants,
        status: MeetingStatus::Scheduled,
    };
    let mut meetings = state
        .meetings
        .write()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    meetings.push(meeting.clone());
    Ok((StatusCode::CREATED, Json(ApiResponse {
        success: true,
        data: Some(meeting),
        error: None,
    })))
}

async fn list_transcripts(
    AxumState(state): AxumState<MinutesState>,
) -> Result<Json<ApiResponse<Vec<Transcript>>>, StatusCode> {
    let transcripts = state
        .transcripts
        .read()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(ApiResponse {
        success: true,
        data: Some(transcripts.clone()),
        error: None,
    }))
}

async fn create_transcript(
    AxumState(state): AxumState<MinutesState>,
    Json(payload): Json<CreateTranscriptRequest>,
) -> Result<(StatusCode, Json<ApiResponse<Transcript>>), StatusCode> {
    let transcript = Transcript {
        id: Uuid::new_v4(),
        meeting_id: payload.meeting_id,
        content: payload.content,
        language: payload.language,
        created_at: Utc::now(),
    };
    let mut transcripts = state
        .transcripts
        .write()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    transcripts.push(transcript.clone());
    Ok((StatusCode::CREATED, Json(ApiResponse {
        success: true,
        data: Some(transcript),
        error: None,
    })))
}

async fn list_documents(
    AxumState(state): AxumState<MinutesState>,
) -> Result<Json<ApiResponse<Vec<MinuteDocument>>>, StatusCode> {
    let documents = state
        .documents
        .read()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(ApiResponse {
        success: true,
        data: Some(documents.clone()),
        error: None,
    }))
}

async fn create_document(
    AxumState(state): AxumState<MinutesState>,
    Json(payload): Json<CreateMinuteDocumentRequest>,
) -> Result<(StatusCode, Json<ApiResponse<MinuteDocument>>), StatusCode> {
    let document = MinuteDocument {
        id: Uuid::new_v4(),
        meeting_id: payload.meeting_id,
        content: payload.content,
        status: DocumentStatus::Draft,
        created_at: Utc::now(),
    };
    let mut documents = state
        .documents
        .write()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    documents.push(document.clone());
    Ok((StatusCode::CREATED, Json(ApiResponse {
        success: true,
        data: Some(document),
        error: None,
    })))
}

async fn update_document(
    AxumState(state): AxumState<MinutesState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateDocumentRequest>,
) -> Result<Json<ApiResponse<MinuteDocument>>, StatusCode> {
    let mut documents = state
        .documents
        .write()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let document = documents
        .iter_mut()
        .find(|d| d.id == id)
        .ok_or(StatusCode::NOT_FOUND)?;
    if let Some(content) = payload.content {
        document.content = content;
    }
    if let Some(status) = payload.status {
        document.status = status;
    }
    Ok(Json(ApiResponse {
        success: true,
        data: Some(document.clone()),
        error: None,
    }))
}

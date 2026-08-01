use axum::{
    extract::{Path, Query, State},
    Json,
    http::StatusCode,
};
use serde::Deserialize;
use uuid::Uuid;
use std::sync::Arc;

use botcore::shared::state::AppState;

use crate::minutes::types::*;

#[derive(Deserialize)]
pub struct PaginationParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn handle_create_recording(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateRecordingRequest>,
) -> Result<Json<MeetRecording>, (StatusCode, String)> {
    let _ = state;
    let _ = req;
    Err((StatusCode::NOT_IMPLEMENTED, "Minutes handlers pending real DbPool implementation".into()))
}

pub async fn handle_get_recording(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<MeetRecording>, (StatusCode, String)> {
    let _ = state;
    let _ = id;
    Err((StatusCode::NOT_IMPLEMENTED, "Minutes handlers pending real DbPool implementation".into()))
}

pub async fn handle_list_recordings(
    State(state): State<Arc<AppState>>,
    Path(bot_id): Path<Uuid>,
    Query(p): Query<PaginationParams>,
) -> Result<Json<Vec<MeetRecording>>, (StatusCode, String)> {
    let _ = state;
    let _ = bot_id;
    let _ = p;
    Ok(Json(Vec::new()))
}

pub async fn handle_transcribe_recording(
    State(state): State<Arc<AppState>>,
    Path(recording_id): Path<Uuid>,
) -> Result<Json<Transcription>, (StatusCode, String)> {
    let _ = state;
    let _ = recording_id;
    Err((StatusCode::NOT_IMPLEMENTED, "Transcription pending implementation".into()))
}

pub async fn handle_generate_minutes(
    State(state): State<Arc<AppState>>,
    Path(recording_id): Path<Uuid>,
) -> Result<Json<MeetingMinute>, (StatusCode, String)> {
    let _ = state;
    let _ = recording_id;
    Err((StatusCode::NOT_IMPLEMENTED, "Minute generation pending implementation".into()))
}

pub async fn handle_get_minutes(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<MeetingMinute>, (StatusCode, String)> {
    let _ = state;
    let _ = id;
    Err((StatusCode::NOT_IMPLEMENTED, "Minute retrieval pending implementation".into()))
}

pub async fn handle_list_minutes(
    State(state): State<Arc<AppState>>,
    Path(bot_id): Path<Uuid>,
    Query(p): Query<PaginationParams>,
) -> Result<Json<Vec<MeetingMinute>>, (StatusCode, String)> {
    let _ = state;
    let _ = bot_id;
    let _ = p;
    Ok(Json(Vec::new()))
}

pub async fn handle_update_minutes(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateMinutesRequest>,
) -> Result<Json<MeetingMinute>, (StatusCode, String)> {
    let _ = state;
    let _ = id;
    let _ = req;
    Err((StatusCode::NOT_IMPLEMENTED, "Minute update pending implementation".into()))
}

pub async fn handle_finalize_minutes(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let _ = state;
    let _ = id;
    Ok(Json(serde_json::json!({"status": "finalized"})))
}

pub async fn handle_export_pdf(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Vec<u8>, (StatusCode, String)> {
    let _ = state;
    let _ = id;
    Ok(Vec::new())
}

pub async fn handle_get_signatures(
    State(state): State<Arc<AppState>>,
    Path(minute_id): Path<Uuid>,
) -> Result<Json<Vec<MinuteSignature>>, (StatusCode, String)> {
    let _ = state;
    let _ = minute_id;
    Ok(Json(Vec::new()))
}

pub async fn handle_sign_minutes(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<SignMinutesRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let _ = state;
    let _ = id;
    let _ = req;
    Err((StatusCode::NOT_IMPLEMENTED, "Digital signature pending implementation".into()))
}

pub fn minutes_routes() -> axum::Router<Arc<AppState>> {
    use axum::routing::{get, post, put};

    axum::Router::new()
        .route("/api/meet/recordings", post(handle_create_recording))
        .route("/api/meet/recordings/:id", get(handle_get_recording))
        .route("/api/meet/recordings/bot/:bot_id", get(handle_list_recordings))
        .route("/api/meet/recordings/:id/transcribe", post(handle_transcribe_recording))
        .route("/api/meet/minutes/from/:recording_id", post(handle_generate_minutes))
        .route("/api/meet/minutes/:id", get(handle_get_minutes))
        .route("/api/meet/minutes/bot/:bot_id", get(handle_list_minutes))
        .route("/api/meet/minutes/:id", put(handle_update_minutes))
        .route("/api/meet/minutes/:id/finalize", post(handle_finalize_minutes))
        .route("/api/meet/minutes/:id/export/pdf", get(handle_export_pdf))
        .route("/api/meet/minutes/:id/signatures", get(handle_get_signatures))
        .route("/api/meet/minutes/:id/sign", post(handle_sign_minutes))
}

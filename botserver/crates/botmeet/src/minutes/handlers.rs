use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use log::info;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use botcore::shared::state::AppState;
use botsecurity::auth::AuthenticatedUser;
use botsecurity_core::error_sanitizer::log_and_sanitize_tuple;

use crate::minutes::generator::MinutesGenerator;
use crate::minutes::signature::SignatureService;
use crate::minutes::storage::MinuteStorage;
use crate::minutes::transcriber::RealSttTranscriber;
use crate::minutes::types::*;

#[derive(Deserialize)]
pub struct PaginationParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// #2753 — log the cause server-side, hand the caller only the generic body.
fn internal_error(e: impl std::fmt::Display, context: &str) -> (StatusCode, String) {
    log_and_sanitize_tuple(&e, context, StatusCode::INTERNAL_SERVER_ERROR)
}

fn db_error(e: impl std::fmt::Display) -> (StatusCode, String) {
    internal_error(e, "botmeet::minutes::db")
}

pub async fn handle_create_recording(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateRecordingRequest>,
) -> Result<Json<MeetRecording>, (StatusCode, String)> {
    let recording = MeetRecording {
        id: Uuid::new_v4(),
        bot_id: Uuid::nil(),
        meeting_id: req.meeting_id,
        title: req.title.clone(),
        recording_path: format!(
            "meet/{}.webm",
            req.meeting_id
                .map(|m| m.to_string())
                .unwrap_or_else(|| Uuid::new_v4().to_string())
        ),
        duration_seconds: req.duration_seconds,
        file_size: None,
        language: req.language.unwrap_or_else(|| "auto".to_string()),
        status: RecordingStatus::Recorded,
        created_at: Utc::now(),
    };

    let pool = state.conn.clone();
    let rec = recording.clone();
    tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .map_err(|e| format!("Failed to get DB connection: {e}"))?;
        MinuteStorage::save_recording(&mut conn, &rec)
            .map_err(|e| format!("Failed to save recording: {e}"))
    })
    .await
    .map_err(|e| internal_error(e, "botmeet::minutes::spawn"))?
    .map_err(|e| internal_error(e, "botmeet::minutes::create"))?;

    info!("Created recording {}", recording.id);
    Ok(Json(recording))
}

pub async fn handle_get_recording(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<MeetRecording>, (StatusCode, String)> {
    let pool = state.conn.clone();
    let rec = tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .map_err(|e| format!("Failed to get DB connection: {e}"))?;
        MinuteStorage::get_recording(&mut conn, id)
            .map_err(|e| format!("Failed to load recording: {e}"))
    })
    .await
    .map_err(|e| internal_error(e, "botmeet::minutes::spawn"))?
    .map_err(|_| (StatusCode::NOT_FOUND, "Resource not found".to_string()))?;

    Ok(Json(rec))
}

pub async fn handle_list_recordings(
    State(state): State<Arc<AppState>>,
    Path(bot_id): Path<Uuid>,
    Query(p): Query<PaginationParams>,
) -> Result<Json<Vec<MeetRecording>>, (StatusCode, String)> {
    let limit = p.limit.unwrap_or(50);
    let offset = p.offset.unwrap_or(0);
    let pool = state.conn.clone();
    let recs = tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .map_err(|e| format!("Failed to get DB connection: {e}"))?;
        MinuteStorage::list_recordings(&mut conn, bot_id, limit, offset)
            .map_err(|e| format!("Failed to list recordings: {e}"))
    })
    .await
    .map_err(|e| internal_error(e, "botmeet::minutes::spawn"))?
    .map_err(db_error)?;

    Ok(Json(recs))
}

pub async fn handle_transcribe_recording(
    State(state): State<Arc<AppState>>,
    Path(recording_id): Path<Uuid>,
) -> Result<Json<Transcription>, (StatusCode, String)> {
    let pool = state.conn.clone();

    let recording = tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .map_err(|e| format!("Failed to get DB connection: {e}"))?;
        MinuteStorage::get_recording(&mut conn, recording_id)
            .map_err(|e| format!("Failed to load recording: {e}"))
    })
    .await
    .map_err(|e| internal_error(e, "botmeet::minutes::spawn"))?
    .map_err(|_| (StatusCode::NOT_FOUND, "Resource not found".to_string()))?;

    // Mark transcribing before the (possibly long) STT call
    let pool = state.conn.clone();
    let rec_id = recording.id;
    tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .map_err(|e| format!("Failed to get DB connection: {e}"))?;
        MinuteStorage::update_recording_status(&mut conn, rec_id, &RecordingStatus::Transcribing)
            .map_err(|e| format!("Failed to update recording status: {e}"))
    })
    .await
    .map_err(|e| internal_error(e, "botmeet::minutes::spawn"))?
    .map_err(db_error)?;

    let api_base = std::env::var("BOTMODELS_HOST").unwrap_or_else(|_| String::new());
    let api_key = std::env::var("BOTMODELS_API_KEY").unwrap_or_else(|_| String::new());

    let transcription = if api_base.is_empty() || api_key.is_empty() {
        RealSttTranscriber::transcribe_fallback(&recording.recording_path).await
    } else {
        RealSttTranscriber::transcribe(
            &recording.recording_path,
            &recording.language,
            &api_base,
            &api_key,
        )
        .await
    }
    .map_err(|e| internal_error(e, "botmeet::minutes::transcribe"))?;

    let mut transcription = transcription;
    transcription.recording_id = recording.id;

    let pool = state.conn.clone();
    let trans = transcription.clone();
    let final_status = if transcription.segments.is_empty() {
        RecordingStatus::Failed
    } else {
        RecordingStatus::Transcribed
    };
    tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .map_err(|e| format!("Failed to get DB connection: {e}"))?;
        MinuteStorage::save_transcription(&mut conn, &trans)
            .map_err(|e| format!("Failed to save transcription: {e}"))?;
        MinuteStorage::update_recording_status(&mut conn, trans.recording_id, &final_status)
            .map_err(|e| format!("Failed to update recording status: {e}"))
    })
    .await
    .map_err(|e| internal_error(e, "botmeet::minutes::spawn"))?
    .map_err(db_error)?;

    info!("Transcribed recording {}", recording.id);
    Ok(Json(transcription))
}

pub async fn handle_generate_minutes(
    State(state): State<Arc<AppState>>,
    Path(recording_id): Path<Uuid>,
) -> Result<Json<MeetingMinute>, (StatusCode, String)> {
    let pool = state.conn.clone();

    let (transcription, title) = tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .map_err(|e| format!("Failed to get DB connection: {e}"))?;
        let t = MinuteStorage::get_transcription(&mut conn, recording_id)
            .map_err(|e| format!("Failed to load transcription: {e}"))?;
        let title = MinuteStorage::get_recording(&mut conn, recording_id)
            .map(|r| r.title)
            .unwrap_or_else(|_| format!("Minutes for recording {recording_id}"));
        Ok::<_, String>((t, title))
    })
    .await
    .map_err(|e| internal_error(e, "botmeet::minutes::spawn"))?
    .map_err(|_| (StatusCode::NOT_FOUND, "Resource not found".to_string()))?;

    let llm_url = std::env::var("LLM_URL").unwrap_or_else(|_| String::new());
    let llm_key = std::env::var("LLM_KEY").unwrap_or_else(|_| String::new());
    let llm_model = std::env::var("LLM_MODEL").unwrap_or_else(|_| String::new());

    let mut minute = MinutesGenerator::from_transcription(
        &transcription,
        &title,
        None,
        if llm_url.is_empty() {
            None
        } else {
            Some(llm_url.as_str())
        },
        if llm_key.is_empty() {
            None
        } else {
            Some(llm_key.as_str())
        },
        if llm_model.is_empty() {
            None
        } else {
            Some(llm_model.as_str())
        },
    )
    .await
    .map_err(|e| internal_error(e, "botmeet::minutes::generate"))?;

    minute.bot_id = Uuid::nil();
    minute.recording_id = Some(recording_id);

    let pool = state.conn.clone();
    let m = minute.clone();
    let rec_id = recording_id;
    tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .map_err(|e| format!("Failed to get DB connection: {e}"))?;
        MinuteStorage::save_minute(&mut conn, &m)
            .map_err(|e| format!("Failed to save minute: {e}"))?;
        MinuteStorage::update_recording_status(&mut conn, rec_id, &RecordingStatus::Ready)
            .map_err(|e| format!("Failed to update recording status: {e}"))
    })
    .await
    .map_err(|e| internal_error(e, "botmeet::minutes::spawn"))?
    .map_err(db_error)?;

    info!(
        "Generated minutes {} from recording {}",
        minute.id, recording_id
    );
    Ok(Json(minute))
}

mod handlers_minutes;

pub use handlers_minutes::*;

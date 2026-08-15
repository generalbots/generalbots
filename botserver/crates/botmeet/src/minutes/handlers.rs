use axum::{
    extract::{Path, Query, State},
    Json,
    http::StatusCode,
};
use log::info;
use serde::Deserialize;
use uuid::Uuid;
use std::sync::Arc;
use chrono::Utc;

use botcore::shared::state::AppState;

use crate::minutes::types::*;
use crate::minutes::storage::MinuteStorage;
use crate::minutes::generator::MinutesGenerator;
use crate::minutes::transcriber::RealSttTranscriber;
use crate::minutes::signature::SignatureService;

#[derive(Deserialize)]
pub struct PaginationParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

fn db_error(e: impl std::fmt::Display) -> (StatusCode, String) {
    log::error!("Meeting minutes database error: {e}");
    (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {e}"))
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
        recording_path: format!("meet/{}.webm", req.meeting_id.map(|m| m.to_string()).unwrap_or_else(|| Uuid::new_v4().to_string())),
        duration_seconds: req.duration_seconds,
        file_size: None,
        language: req.language.unwrap_or_else(|| "auto".to_string()),
        status: RecordingStatus::Recorded,
        created_at: Utc::now(),
    };

    let pool = state.conn.clone();
    let rec = recording.clone();
    tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| format!("Failed to get DB connection: {e}"))?;
        MinuteStorage::save_recording(&mut conn, &rec)
            .map_err(|e| format!("Failed to save recording: {e}"))
    })
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Task failed: {e}")))?
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    info!("Created recording {}", recording.id);
    Ok(Json(recording))
}

pub async fn handle_get_recording(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<MeetRecording>, (StatusCode, String)> {
    let pool = state.conn.clone();
    let rec = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| format!("Failed to get DB connection: {e}"))?;
        MinuteStorage::get_recording(&mut conn, id)
            .map_err(|e| format!("Failed to load recording: {e}"))
    })
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Task failed: {e}")))?
    .map_err(|e| (StatusCode::NOT_FOUND, e))?;

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
        let mut conn = pool.get().map_err(|e| format!("Failed to get DB connection: {e}"))?;
        MinuteStorage::list_recordings(&mut conn, bot_id, limit, offset)
            .map_err(|e| format!("Failed to list recordings: {e}"))
    })
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Task failed: {e}")))?
    .map_err(db_error)?;

    Ok(Json(recs))
}

pub async fn handle_transcribe_recording(
    State(state): State<Arc<AppState>>,
    Path(recording_id): Path<Uuid>,
) -> Result<Json<Transcription>, (StatusCode, String)> {
    let pool = state.conn.clone();

    let recording = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| format!("Failed to get DB connection: {e}"))?;
        MinuteStorage::get_recording(&mut conn, recording_id)
            .map_err(|e| format!("Failed to load recording: {e}"))
    })
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Task failed: {e}")))?
    .map_err(|e| (StatusCode::NOT_FOUND, e))?;

    // Mark transcribing before the (possibly long) STT call
    let pool = state.conn.clone();
    let rec_id = recording.id;
    tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| format!("Failed to get DB connection: {e}"))?;
        MinuteStorage::update_recording_status(&mut conn, rec_id, &RecordingStatus::Transcribing)
            .map_err(|e| format!("Failed to update recording status: {e}"))
    })
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Task failed: {e}")))?
    .map_err(db_error)?;

    let api_base = std::env::var("BOTMODELS_HOST").unwrap_or_default();
    let api_key = std::env::var("BOTMODELS_API_KEY").unwrap_or_default();

    let transcription = if api_base.is_empty() || api_key.is_empty() {
        RealSttTranscriber::transcribe_fallback(&recording.recording_path).await
    } else {
        RealSttTranscriber::transcribe(&recording.recording_path, &recording.language, &api_base, &api_key).await
    }
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Transcription failed: {e}")))?;

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
        let mut conn = pool.get().map_err(|e| format!("Failed to get DB connection: {e}"))?;
        MinuteStorage::save_transcription(&mut conn, &trans)
            .map_err(|e| format!("Failed to save transcription: {e}"))?;
        MinuteStorage::update_recording_status(&mut conn, trans.recording_id, &final_status)
            .map_err(|e| format!("Failed to update recording status: {e}"))
    })
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Task failed: {e}")))?
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
        let mut conn = pool.get().map_err(|e| format!("Failed to get DB connection: {e}"))?;
        let t = MinuteStorage::get_transcription(&mut conn, recording_id)
            .map_err(|e| format!("Failed to load transcription: {e}"))?;
        let title = MinuteStorage::get_recording(&mut conn, recording_id)
            .map(|r| r.title)
            .unwrap_or_else(|_| format!("Minutes for recording {recording_id}"));
        Ok::<_, String>((t, title))
    })
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Task failed: {e}")))?
    .map_err(|e| (StatusCode::NOT_FOUND, e))?;

    let llm_url = std::env::var("LLM_URL").unwrap_or_default();
    let llm_key = std::env::var("LLM_KEY").unwrap_or_default();
    let llm_model = std::env::var("LLM_MODEL").unwrap_or_default();

    let mut minute = MinutesGenerator::from_transcription(
        &transcription,
        &title,
        None,
        if llm_url.is_empty() { None } else { Some(llm_url.as_str()) },
        if llm_key.is_empty() { None } else { Some(llm_key.as_str()) },
        if llm_model.is_empty() { None } else { Some(llm_model.as_str()) },
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Minute generation failed: {e}")))?;

    minute.bot_id = Uuid::nil();
    minute.recording_id = Some(recording_id);

    let pool = state.conn.clone();
    let m = minute.clone();
    let rec_id = recording_id;
    tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| format!("Failed to get DB connection: {e}"))?;
        MinuteStorage::save_minute(&mut conn, &m)
            .map_err(|e| format!("Failed to save minute: {e}"))?;
        MinuteStorage::update_recording_status(&mut conn, rec_id, &RecordingStatus::Ready)
            .map_err(|e| format!("Failed to update recording status: {e}"))
    })
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Task failed: {e}")))?
    .map_err(db_error)?;

    info!("Generated minutes {} from recording {}", minute.id, recording_id);
    Ok(Json(minute))
}

pub async fn handle_get_minutes(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<MeetingMinute>, (StatusCode, String)> {
    let pool = state.conn.clone();
    let minute = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| format!("Failed to get DB connection: {e}"))?;
        MinuteStorage::get_minute(&mut conn, id)
            .map_err(|e| format!("Failed to load minutes: {e}"))
    })
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Task failed: {e}")))?
    .map_err(|e| (StatusCode::NOT_FOUND, e))?;

    Ok(Json(minute))
}

pub async fn handle_list_minutes(
    State(state): State<Arc<AppState>>,
    Path(bot_id): Path<Uuid>,
    Query(p): Query<PaginationParams>,
) -> Result<Json<Vec<MeetingMinute>>, (StatusCode, String)> {
    let limit = p.limit.unwrap_or(50);
    let offset = p.offset.unwrap_or(0);
    let pool = state.conn.clone();
    let minutes = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| format!("Failed to get DB connection: {e}"))?;
        MinuteStorage::list_minutes(&mut conn, bot_id, limit, offset)
            .map_err(|e| format!("Failed to list minutes: {e}"))
    })
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Task failed: {e}")))?
    .map_err(db_error)?;

    Ok(Json(minutes))
}

pub async fn handle_update_minutes(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateMinutesRequest>,
) -> Result<Json<MeetingMinute>, (StatusCode, String)> {
    let pool = state.conn.clone();
    let updated = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| format!("Failed to get DB connection: {e}"))?;
        MinuteStorage::update_minute(&mut conn, id, &req)
            .map_err(|e| format!("Failed to update minutes: {e}"))?;
        MinuteStorage::get_minute(&mut conn, id)
            .map_err(|e| format!("Failed to reload minutes: {e}"))
    })
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Task failed: {e}")))?
    .map_err(db_error)?;

    Ok(Json(updated))
}

pub async fn handle_finalize_minutes(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let pool = state.conn.clone();
    let minute = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| format!("Failed to get DB connection: {e}"))?;
        MinuteStorage::finalize_minute(&mut conn, id)
            .map_err(|e| format!("Failed to finalize minutes: {e}"))?;
        MinuteStorage::get_minute(&mut conn, id)
            .map_err(|e| format!("Failed to reload minutes: {e}"))
    })
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Task failed: {e}")))?
    .map_err(db_error)?;

    info!("Finalized minutes {}", minute.id);
    Ok(Json(serde_json::json!({
        "id": minute.id,
        "status": minute.status.to_string(),
        "finalized_at": minute.updated_at,
    })))
}

pub async fn handle_export_pdf(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Vec<u8>, (StatusCode, String)> {
    // Load the minute so the export endpoint at least validates existence
    let pool = state.conn.clone();
    let _minute = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| format!("Failed to get DB connection: {e}"))?;
        MinuteStorage::get_minute(&mut conn, id)
            .map_err(|e| format!("Failed to load minutes: {e}"))
    })
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Task failed: {e}")))?
    .map_err(|e| (StatusCode::NOT_FOUND, e))?;

    Err((StatusCode::NOT_IMPLEMENTED, "PDF export not yet implemented".to_string()))
}

pub async fn handle_get_signatures(
    State(state): State<Arc<AppState>>,
    Path(minute_id): Path<Uuid>,
) -> Result<Json<Vec<MinuteSignature>>, (StatusCode, String)> {
    let pool = state.conn.clone();
    let sigs = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| format!("Failed to get DB connection: {e}"))?;
        MinuteStorage::get_signatures(&mut conn, minute_id)
            .map_err(|e| format!("Failed to load signatures: {e}"))
    })
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Task failed: {e}")))?
    .map_err(db_error)?;

    Ok(Json(sigs))
}

pub async fn handle_sign_minutes(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<SignMinutesRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let pool = state.conn.clone();
    let minute = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| format!("Failed to get DB connection: {e}"))?;
        MinuteStorage::get_minute(&mut conn, id)
            .map_err(|e| format!("Failed to load minutes: {e}"))
    })
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Task failed: {e}")))?
    .map_err(|e| (StatusCode::NOT_FOUND, e))?;

    let user_id = Uuid::nil();
    let signature = SignatureService::sign_minute(&minute, user_id, req.signature_id, req.ip_address);

    let pool = state.conn.clone();
    let sig = signature.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| format!("Failed to get DB connection: {e}"))?;
        MinuteStorage::save_signature(&mut conn, &sig)
            .map_err(|e| format!("Failed to save signature: {e}"))?;
        // Recompute status: signed when enough signatures exist
        let mut sigs = MinuteStorage::get_signatures(&mut conn, id)
            .map_err(|e| format!("Failed to load signatures: {e}"))?;
        sigs.push(sig.clone());
        let mut m = MinuteStorage::get_minute(&mut conn, id)
            .map_err(|e| format!("Failed to load minutes: {e}"))?;
        SignatureService::update_minute_status(&mut m, &sigs);
        let status = m.status.to_string();
        MinuteStorage::set_minute_status(&mut conn, id, &m.status)
            .map_err(|e| format!("Failed to update minute status: {e}"))?;
        Ok::<_, String>((sig, status))
    })
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Task failed: {e}")))?
    .map_err(db_error)?;

    let (signature, status) = result;
    info!("Minute {} signed (status: {status})", id);
    Ok(Json(serde_json::json!({
        "success": true,
        "signature": signature,
        "status": status,
    })))
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

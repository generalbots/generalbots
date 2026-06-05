use axum::{
    extract::{Path, Query, State},
    Json,
    http::StatusCode,
};
use serde::Deserialize;
use uuid::Uuid;
use std::sync::Arc;
use diesel::PgConnection;
use log::error;

use crate::minutes::types::*;
use crate::minutes::storage::MinuteStorage;
use crate::minutes::signature::MinuteSignature;

pub type DbPool = deadpool_diesel::postgres::Pool;

#[derive(Deserialize)]
pub struct PaginationParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

async fn get_conn(pool: &DbPool) -> Result<PgConnection, (StatusCode, String)> {
    pool.get().await.map_err(|e| {
        error!("Failed to get DB connection: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Database connection failed".into())
    })
}

pub async fn handle_create_recording(
    State(pool): State<Arc<DbPool>>,
    Json(req): Json<CreateRecordingRequest>,
) -> Result<Json<MeetRecording>, (StatusCode, String)> {
    let mut conn = get_conn(&pool).await?;
    let recording = MeetRecording::new(req);
    MinuteStorage::save_recording(&mut conn, &recording).await.map_err(|e| {
        error!("Failed to save recording: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create recording".into())
    })?;
    Ok(Json(recording))
}

pub async fn handle_get_recording(
    State(pool): State<Arc<DbPool>>,
    Path(id): Path<Uuid>,
) -> Result<Json<MeetRecording>, (StatusCode, String)> {
    let mut conn = get_conn(&pool).await?;
    let recording = MinuteStorage::get_recording(&mut conn, id).await.map_err(|e| {
        error!("Recording not found: {e}");
        (StatusCode::NOT_FOUND, "Recording not found".into())
    })?;
    Ok(Json(recording))
}

pub async fn handle_list_recordings(
    State(pool): State<Arc<DbPool>>,
    Path(bot_id): Path<Uuid>,
    Query(p): Query<PaginationParams>,
) -> Result<Json<Vec<MeetRecording>>, (StatusCode, String)> {
    let mut conn = get_conn(&pool).await?;
    let limit = p.limit.unwrap_or(20).min(100);
    let offset = p.offset.unwrap_or(0);
    let recordings = MinuteStorage::list_recordings(&mut conn, bot_id, limit, offset).await.map_err(|e| {
        error!("Failed to list recordings: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to list recordings".into())
    })?;
    Ok(Json(recordings))
}

pub async fn handle_transcribe_recording(
    State(pool): State<Arc<DbPool>>,
    Path(recording_id): Path<Uuid>,
) -> Result<Json<Transcription>, (StatusCode, String)> {
    let mut conn = get_conn(&pool).await?;
    let recording = MinuteStorage::get_recording(&mut conn, recording_id).await.map_err(|e| {
        error!("Recording not found: {e}");
        (StatusCode::NOT_FOUND, "Recording not found".into())
    })?;

    let transcriber = crate::minutes::transcriber::RealSttTranscriber::new(
        std::env::var("STT_PROVIDER").unwrap_or_else(|_| "whisper".into()),
        std::env::var("STT_API_KEY").unwrap_or_default(),
    );

    let transcription = tokio::task::spawn_blocking(move || {
        transcriber.transcribe(&recording)
    }).await.map_err(|e| {
        error!("Transcription task panicked: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Transcription failed".into())
    })?.map_err(|e| {
        error!("Transcription failed: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Transcription failed".into())
    })?;

    MinuteStorage::save_transcription(&mut conn, &transcription).await.map_err(|e| {
        error!("Failed to save transcription: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save transcription".into())
    })?;

    MinuteStorage::update_recording_status(&mut conn, recording_id, &RecordingStatus::Transcribed).await.map_err(|e| {
        error!("Failed to update recording status: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update status".into())
    })?;

    Ok(Json(transcription))
}

pub async fn handle_generate_minutes(
    State(pool): State<Arc<DbPool>>,
    Path(recording_id): Path<Uuid>,
) -> Result<Json<MeetingMinute>, (StatusCode, String)> {
    let mut conn = get_conn(&pool).await?;
    let transcription = MinuteStorage::get_transcription(&mut conn, recording_id).await.map_err(|e| {
        error!("Transcription not found: {e}");
        (StatusCode::NOT_FOUND, "Transcription not found".into())
    })?;

    let generator = crate::minutes::generator::MinutesGenerator::new(
        std::env::var("LLM_PROVIDER").unwrap_or_else(|_| "openai".into()),
        std::env::var("LLM_API_KEY").unwrap_or_default(),
        std::env::var("LLM_MODEL").unwrap_or_else(|_| "gpt-4".into()),
    );

    let minute = tokio::task::spawn_blocking(move || {
        generator.generate(&transcription)
    }).await.map_err(|e| {
        error!("Generation task panicked: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Minute generation failed".into())
    })?.map_err(|e| {
        error!("Generation failed: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Minute generation failed".into())
    })?;

    MinuteStorage::save_minute(&mut conn, &minute).await.map_err(|e| {
        error!("Failed to save minute: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save minute".into())
    })?;

    Ok(Json(minute))
}

pub async fn handle_get_minutes(
    State(pool): State<Arc<DbPool>>,
    Path(id): Path<Uuid>,
) -> Result<Json<MeetingMinute>, (StatusCode, String)> {
    let mut conn = get_conn(&pool).await?;
    let minute = MinuteStorage::get_minute(&mut conn, id).await.map_err(|e| {
        error!("Minutes not found: {e}");
        (StatusCode::NOT_FOUND, "Minutes not found".into())
    })?;
    Ok(Json(minute))
}

pub async fn handle_list_minutes(
    State(pool): State<Arc<DbPool>>,
    Path(bot_id): Path<Uuid>,
    Query(p): Query<PaginationParams>,
) -> Result<Json<Vec<MeetingMinute>>, (StatusCode, String)> {
    let mut conn = get_conn(&pool).await?;
    let limit = p.limit.unwrap_or(20).min(100);
    let offset = p.offset.unwrap_or(0);
    let minutes = MinuteStorage::list_minutes(&mut conn, bot_id, limit, offset).await.map_err(|e| {
        error!("Failed to list minutes: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to list minutes".into())
    })?;
    Ok(Json(minutes))
}

pub async fn handle_update_minutes(
    State(pool): State<Arc<DbPool>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateMinutesRequest>,
) -> Result<Json<MeetingMinute>, (StatusCode, String)> {
    let mut conn = get_conn(&pool).await?;
    MinuteStorage::update_minute(&mut conn, id, &req).await.map_err(|e| {
        error!("Failed to update minutes: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update minutes".into())
    })?;
    let minute = MinuteStorage::get_minute(&mut conn, id).await.map_err(|e| {
        error!("Updated minutes not found: {e}");
        (StatusCode::NOT_FOUND, "Updated minutes not found".into())
    })?;
    Ok(Json(minute))
}

pub async fn handle_finalize_minutes(
    State(pool): State<Arc<DbPool>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = get_conn(&pool).await?;
    MinuteStorage::finalize_minute(&mut conn, id).await.map_err(|e| {
        error!("Failed to finalize minutes: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to finalize minutes".into())
    })?;
    Ok(Json(serde_json::json!({"status": "finalized"})))
}

pub async fn handle_export_pdf(
    State(pool): State<Arc<DbPool>>,
    Path(id): Path<Uuid>,
) -> Result<Vec<u8>, (StatusCode, String)> {
    let mut conn = get_conn(&pool).await?;
    let minute = MinuteStorage::get_minute(&mut conn, id).await.map_err(|e| {
        error!("Minutes not found: {e}");
        (StatusCode::NOT_FOUND, "Minutes not found".into())
    })?;

    let pdf_bytes = crate::minutes::generator::MinutesGenerator::export_pdf(&minute).map_err(|e| {
        error!("PDF export failed: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "PDF export failed".into())
    })?;

    Ok(pdf_bytes)
}

pub async fn handle_get_signatures(
    State(pool): State<Arc<DbPool>>,
    Path(minute_id): Path<Uuid>,
) -> Result<Json<Vec<MinuteSignature>>, (StatusCode, String)> {
    let mut conn = get_conn(&pool).await?;
    let sigs = MinuteStorage::get_signatures(&mut conn, minute_id).await.map_err(|e| {
        error!("Failed to get signatures: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to get signatures".into())
    })?;
    Ok(Json(sigs))
}

pub async fn handle_sign_minutes(
    State(pool): State<Arc<DbPool>>,
    Path(id): Path<Uuid>,
    Json(req): Json<SignMinutesRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = get_conn(&pool).await?;
    let minute = MinuteStorage::get_minute(&mut conn, id).await.map_err(|e| {
        error!("Minutes not found: {e}");
        (StatusCode::NOT_FOUND, "Minutes not found".into())
    })?;

    let signer = crate::minutes::signature::DigitalSigner::new(
        std::env::var("SIGNATURE_PRIVATE_KEY").unwrap_or_default(),
        std::env::var("SIGNATURE_CERTIFICATE").unwrap_or_default(),
    );

    let sig = tokio::task::spawn_blocking(move || {
        signer.sign(&minute)
    }).await.map_err(|e| {
        error!("Signature task panicked: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Digital signature failed".into())
    })?.map_err(|e| {
        error!("Digital signature failed: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Digital signature failed".into())
    })?;

    MinuteStorage::save_signature(&mut conn, &sig).await.map_err(|e| {
        error!("Failed to save signature: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save signature".into())
    })?;

    Ok(Json(serde_json::json!({"signature_id": sig.id, "signed_hash": sig.signed_hash})))
}

pub fn minutes_routes() -> axum::Router<Arc<DbPool>> {
    use axum::routing::{get, post, put};

    axum::Router::new()
        .route("/api/meet/recordings", post(handle_create_recording))
        .route("/api/meet/recordings/{id}", get(handle_get_recording))
        .route("/api/meet/recordings/bot/{bot_id}", get(handle_list_recordings))
        .route("/api/meet/recordings/{id}/transcribe", post(handle_transcribe_recording))
        .route("/api/meet/minutes/from/{recording_id}", post(handle_generate_minutes))
        .route("/api/meet/minutes/{id}", get(handle_get_minutes))
        .route("/api/meet/minutes/bot/{bot_id}", get(handle_list_minutes))
        .route("/api/meet/minutes/{id}", put(handle_update_minutes))
        .route("/api/meet/minutes/{id}/finalize", post(handle_finalize_minutes))
        .route("/api/meet/minutes/{id}/export/pdf", get(handle_export_pdf))
        .route("/api/meet/minutes/{id}/signatures", get(handle_get_signatures))
        .route("/api/meet/minutes/{id}/sign", post(handle_sign_minutes))
}

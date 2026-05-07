use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use std::sync::Arc;
use tracing::error;
use uuid::Uuid;

use crate::engine::VideoEngine;
use crate::requests::*;
use crate::routes::AppState;
use crate::safe_error::SafeErrorResponse;

pub async fn get_audio_tracks(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<Uuid>,
) -> impl IntoResponse {
    let engine = VideoEngine::new(state.conn.clone());
    match engine.get_audio_tracks(project_id).await {
        Ok(tracks) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({ "audio_tracks": tracks })),
        ),
        Err(e) => {
            error!("Failed to get audio tracks: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!(SafeErrorResponse::internal_error())),
            )
        }
    }
}

pub async fn add_audio_track(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<Uuid>,
    axum::Json(req): axum::Json<AddAudioRequest>,
) -> impl IntoResponse {
    let engine = VideoEngine::new(state.conn.clone());
    match engine.add_audio_track(project_id, req).await {
        Ok(track) => (
            StatusCode::CREATED,
            axum::Json(serde_json::json!({ "audio_track": track })),
        ),
        Err(e) => {
            error!("Failed to add audio track: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!(SafeErrorResponse::internal_error())),
            )
        }
    }
}

pub async fn delete_audio_track(
    State(state): State<Arc<AppState>>,
    Path(track_id): Path<Uuid>,
) -> impl IntoResponse {
    let engine = VideoEngine::new(state.conn.clone());
    match engine.delete_audio_track(track_id).await {
        Ok(()) => (
            StatusCode::NO_CONTENT,
            axum::Json(serde_json::json!({})),
        ),
        Err(e) => {
            error!("Failed to delete audio track: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!(SafeErrorResponse::internal_error())),
            )
        }
    }
}

pub async fn tts_handler(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<Uuid>,
    axum::Json(req): axum::Json<TTSRequest>,
) -> impl IntoResponse {
    let engine = VideoEngine::new(state.conn.clone());
    let output_dir =
        std::env::var("VIDEO_AUDIO_DIR").unwrap_or_else(|_| "./audio/video".to_string());

    if let Err(e) = std::fs::create_dir_all(&output_dir) {
        error!("Failed to create audio directory: {e}");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": SafeErrorResponse::internal_error() })),
        );
    }

    let voice = req.voice.as_deref().unwrap_or("alloy");
    let speed = req.speed.unwrap_or(1.0);
    let language = req.language.as_deref().unwrap_or("en");

    match engine
        .text_to_speech(&req.text, voice, speed, language, &output_dir)
        .await
    {
        Ok(tts_response) => match engine
            .add_audio_track(
                project_id,
                AddAudioRequest {
                    name: Some("Narration".to_string()),
                    source_url: tts_response.audio_url.clone(),
                    track_type: Some("narration".to_string()),
                    start_ms: None,
                    duration_ms: Some(tts_response.duration_ms),
                    volume: Some(1.0),
                },
            )
            .await
        {
            Ok(track) => (
                StatusCode::OK,
                axum::Json(serde_json::json!({
                    "audio_url": tts_response.audio_url,
                    "duration_ms": tts_response.duration_ms,
                    "audio_track": track,
                })),
            ),
            Err(e) => {
                error!("Failed to add audio track: {e}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    axum::Json(serde_json::json!({ "error": SafeErrorResponse::internal_error() })),
                )
            }
        },
        Err(e) => {
            error!("TTS failed: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!(SafeErrorResponse::internal_error())),
            )
        }
    }
}

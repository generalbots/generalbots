use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json},
};
use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;

use crate::security::error_sanitizer::sanitize_error;
use crate::shared::state::AppState;

use super::engine::VideoEngine;
use super::models::*;

pub async fn list_projects(
    State(state): State<Arc<AppState>>,
    Query(filters): Query<ProjectFilters>,
) -> impl IntoResponse {
    let engine = VideoEngine::new(state.db.clone());
    match engine.list_projects(None, filters).await {
        Ok(projects) => (
            StatusCode::OK,
            Json(serde_json::json!({ "projects": projects })),
        ),
        Err(e) => {
            error!("Failed to list video projects: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": sanitize_error("list_projects") })),
            )
        }
    }
}

pub async fn create_project(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateProjectRequest>,
) -> impl IntoResponse {
    let engine = VideoEngine::new(state.db.clone());
    match engine.create_project(None, None, req).await {
        Ok(project) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "project": project })),
        ),
        Err(e) => {
            error!("Failed to create video project: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": sanitize_error("create_project") })),
            )
        }
    }
}

pub async fn get_project(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let engine = VideoEngine::new(state.db.clone());
    match engine.get_project_detail(id).await {
        Ok(detail) => (StatusCode::OK, Json(serde_json::json!(detail))),
        Err(diesel::result::Error::NotFound) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Project not found" })),
        ),
        Err(e) => {
            error!("Failed to get video project: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": sanitize_error("get_project") })),
            )
        }
    }
}

pub async fn update_project(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateProjectRequest>,
) -> impl IntoResponse {
    let engine = VideoEngine::new(state.db.clone());
    match engine.update_project(id, req).await {
        Ok(project) => (
            StatusCode::OK,
            Json(serde_json::json!({ "project": project })),
        ),
        Err(diesel::result::Error::NotFound) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Project not found" })),
        ),
        Err(e) => {
            error!("Failed to update video project: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": sanitize_error("update_project") })),
            )
        }
    }
}

pub async fn delete_project(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let engine = VideoEngine::new(state.db.clone());
    match engine.delete_project(id).await {
        Ok(()) => (StatusCode::NO_CONTENT, Json(serde_json::json!({}))),
        Err(e) => {
            error!("Failed to delete video project: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": sanitize_error("delete_project") })),
            )
        }
    }
}

pub async fn get_clips(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<Uuid>,
) -> impl IntoResponse {
    let engine = VideoEngine::new(state.db.clone());
    match engine.get_clips(project_id).await {
        Ok(clips) => (StatusCode::OK, Json(serde_json::json!({ "clips": clips }))),
        Err(e) => {
            error!("Failed to get clips: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": sanitize_error("get_clips") })),
            )
        }
    }
}

pub async fn add_clip(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<Uuid>,
    Json(req): Json<AddClipRequest>,
) -> impl IntoResponse {
    let engine = VideoEngine::new(state.db.clone());
    match engine.add_clip(project_id, req).await {
        Ok(clip) => (StatusCode::CREATED, Json(serde_json::json!({ "clip": clip }))),
        Err(e) => {
            error!("Failed to add clip: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": sanitize_error("add_clip") })),
            )
        }
    }
}

pub async fn update_clip(
    State(state): State<Arc<AppState>>,
    Path(clip_id): Path<Uuid>,
    Json(req): Json<UpdateClipRequest>,
) -> impl IntoResponse {
    let engine = VideoEngine::new(state.db.clone());
    match engine.update_clip(clip_id, req).await {
        Ok(clip) => (StatusCode::OK, Json(serde_json::json!({ "clip": clip }))),
        Err(diesel::result::Error::NotFound) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Clip not found" })),
        ),
        Err(e) => {
            error!("Failed to update clip: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": sanitize_error("update_clip") })),
            )
        }
    }
}

pub async fn delete_clip(
    State(state): State<Arc<AppState>>,
    Path(clip_id): Path<Uuid>,
) -> impl IntoResponse {
    let engine = VideoEngine::new(state.db.clone());
    match engine.delete_clip(clip_id).await {
        Ok(()) => (StatusCode::NO_CONTENT, Json(serde_json::json!({}))),
        Err(e) => {
            error!("Failed to delete clip: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": sanitize_error("delete_clip") })),
            )
        }
    }
}

pub async fn split_clip_handler(
    State(state): State<Arc<AppState>>,
    Path(clip_id): Path<Uuid>,
    Json(req): Json<SplitClipRequest>,
) -> impl IntoResponse {
    let engine = VideoEngine::new(state.db.clone());
    match engine.split_clip(clip_id, req.at_ms).await {
        Ok((first, second)) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "first_clip": first,
                "second_clip": second,
            })),
        ),
        Err(diesel::result::Error::NotFound) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Invalid split position or clip not found" })),
        ),
        Err(e) => {
            error!("Failed to split clip: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": sanitize_error("split_clip") })),
            )
        }
    }
}

pub async fn get_layers(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<Uuid>,
) -> impl IntoResponse {
    let engine = VideoEngine::new(state.db.clone());
    match engine.get_layers(project_id).await {
        Ok(layers) => (StatusCode::OK, Json(serde_json::json!({ "layers": layers }))),
        Err(e) => {
            error!("Failed to get layers: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": sanitize_error("get_layers") })),
            )
        }
    }
}

pub async fn add_layer(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<Uuid>,
    Json(req): Json<AddLayerRequest>,
) -> impl IntoResponse {
    let engine = VideoEngine::new(state.db.clone());
    match engine.add_layer(project_id, req).await {
        Ok(layer) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "layer": layer })),
        ),
        Err(e) => {
            error!("Failed to add layer: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": sanitize_error("add_layer") })),
            )
        }
    }
}

pub async fn update_layer(
    State(state): State<Arc<AppState>>,
    Path(layer_id): Path<Uuid>,
    Json(req): Json<UpdateLayerRequest>,
) -> impl IntoResponse {
    let engine = VideoEngine::new(state.db.clone());
    match engine.update_layer(layer_id, req).await {
        Ok(layer) => (StatusCode::OK, Json(serde_json::json!({ "layer": layer }))),
        Err(diesel::result::Error::NotFound) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Layer not found" })),
        ),
        Err(e) => {
            error!("Failed to update layer: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": sanitize_error("update_layer") })),
            )
        }
    }
}

pub async fn delete_layer(
    State(state): State<Arc<AppState>>,
    Path(layer_id): Path<Uuid>,
) -> impl IntoResponse {
    let engine = VideoEngine::new(state.db.clone());
    match engine.delete_layer(layer_id).await {
        Ok(()) => (StatusCode::NO_CONTENT, Json(serde_json::json!({}))),
        Err(e) => {
            error!("Failed to delete layer: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": sanitize_error("delete_layer") })),
            )
        }
    }
}

pub async fn get_audio_tracks(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<Uuid>,
) -> impl IntoResponse {
    let engine = VideoEngine::new(state.db.clone());
    match engine.get_audio_tracks(project_id).await {
        Ok(tracks) => (
            StatusCode::OK,
            Json(serde_json::json!({ "audio_tracks": tracks })),
        ),
        Err(e) => {
            error!("Failed to get audio tracks: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": sanitize_error("get_audio_tracks") })),
            )
        }
    }
}

pub async fn add_audio_track(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<Uuid>,
    Json(req): Json<AddAudioRequest>,
) -> impl IntoResponse {
    let engine = VideoEngine::new(state.db.clone());
    match engine.add_audio_track(project_id, req).await {
        Ok(track) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "audio_track": track })),
        ),
        Err(e) => {
            error!("Failed to add audio track: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": sanitize_error("add_audio_track") })),
            )
        }
    }
}

pub async fn delete_audio_track(
    State(state): State<Arc<AppState>>,
    Path(track_id): Path<Uuid>,
) -> impl IntoResponse {
    let engine = VideoEngine::new(state.db.clone());
    match engine.delete_audio_track(track_id).await {
        Ok(()) => (StatusCode::NO_CONTENT, Json(serde_json::json!({}))),
        Err(e) => {
            error!("Failed to delete audio track: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": sanitize_error("delete_audio_track") })),
            )
        }
    }
}

pub async fn get_keyframes(
    State(state): State<Arc<AppState>>,
    Path(layer_id): Path<Uuid>,
) -> impl IntoResponse {
    let engine = VideoEngine::new(state.db.clone());
    match engine.get_keyframes(layer_id).await {
        Ok(keyframes) => (
            StatusCode::OK,
            Json(serde_json::json!({ "keyframes": keyframes })),
        ),
        Err(e) => {
            error!("Failed to get keyframes: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": sanitize_error("get_keyframes") })),
            )
        }
    }
}

pub async fn add_keyframe(
    State(state): State<Arc<AppState>>,
    Path(layer_id): Path<Uuid>,
    Json(req): Json<AddKeyframeRequest>,
) -> impl IntoResponse {
    let engine = VideoEngine::new(state.db.clone());
    match engine.add_keyframe(layer_id, req).await {
        Ok(keyframe) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "keyframe": keyframe })),
        ),
        Err(e) => {
            error!("Failed to add keyframe: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": sanitize_error("add_keyframe") })),
            )
        }
    }
}

pub async fn delete_keyframe(
    State(state): State<Arc<AppState>>,
    Path(keyframe_id): Path<Uuid>,
) -> impl IntoResponse {
    let engine = VideoEngine::new(state.db.clone());
    match engine.delete_keyframe(keyframe_id).await {
        Ok(()) => (StatusCode::NO_CONTENT, Json(serde_json::json!({}))),
        Err(e) => {
            error!("Failed to delete keyframe: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": sanitize_error("delete_keyframe") })),
            )
        }
    }
}

pub async fn upload_media(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<Uuid>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let upload_dir =
        std::env::var("VIDEO_UPLOAD_DIR").unwrap_or_else(|_| "./uploads/video".to_string());

    if let Err(e) = std::fs::create_dir_all(&upload_dir) {
        error!("Failed to create upload directory: {e}");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": sanitize_error("upload_media") })),
        );
    }

    while let Ok(Some(field)) = multipart.next_field().await {
        let file_name = field
            .file_name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("{}.mp4", Uuid::new_v4()));

        let content_type = field
            .content_type()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "video/mp4".to_string());

        let data = match field.bytes().await {
            Ok(d) => d,
            Err(e) => {
                error!("Failed to read upload data: {e}");
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": "Failed to read upload" })),
                );
            }
        };

        let file_size = data.len() as u64;
        let safe_name = format!("{}_{}", project_id, sanitize_filename(&file_name));
        let file_path = format!("{}/{}", upload_dir, safe_name);

        if let Err(e) = std::fs::write(&file_path, &data) {
            error!("Failed to write uploaded file: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": sanitize_error("upload_media") })),
            );
        }

        let file_url = format!("/video/uploads/{}", safe_name);
        info!(
            "Uploaded file {} ({} bytes) for project {}",
            safe_name, file_size, project_id
        );

        return (
            StatusCode::OK,
            Json(serde_json::json!(UploadResponse {
                file_url,
                file_name: safe_name,
                file_size,
                mime_type: content_type,
            })),
        );
    }

    (
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({ "error": "No file provided" })),
    )
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

pub async fn get_preview_frame(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<Uuid>,
    Query(params): Query<PreviewFrameRequest>,
) -> impl IntoResponse {
    let engine = VideoEngine::new(state.db.clone());
    let at_ms = params.at_ms.unwrap_or(0);
    let width = params.width.unwrap_or(640);
    let height = params.height.unwrap_or(360);

    let output_dir =
        std::env::var("VIDEO_PREVIEW_DIR").unwrap_or_else(|_| "./previews/video".to_string());

    if let Err(e) = std::fs::create_dir_all(&output_dir) {
        error!("Failed to create preview directory: {e}");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": sanitize_error("get_preview_frame") })),
        );
    }

    match engine
        .generate_preview_frame(project_id, at_ms, width, height, &output_dir)
        .await
    {
        Ok(url) => (
            StatusCode::OK,
            Json(serde_json::json!({ "preview_url": url, "at_ms": at_ms })),
        ),
        Err(e) => {
            error!("Failed to generate preview: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": sanitize_error("get_preview_frame") })),
            )
        }
    }
}

pub async fn transcribe_handler(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<Uuid>,
    Json(req): Json<TranscribeRequest>,
) -> impl IntoResponse {
    let engine = VideoEngine::new(state.db.clone());
    match engine
        .transcribe_audio(project_id, req.clip_id, req.language)
        .await
    {
        Ok(transcription) => (StatusCode::OK, Json(serde_json::json!(transcription))),
        Err(e) => {
            error!("Failed to transcribe: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": sanitize_error("transcribe") })),
            )
        }
    }
}

pub async fn generate_captions_handler(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<Uuid>,
    Json(req): Json<GenerateCaptionsRequest>,
) -> impl IntoResponse {
    let engine = VideoEngine::new(state.db.clone());

    let transcription = match engine.transcribe_audio(project_id, None, None).await {
        Ok(t) => t,
        Err(e) => {
            error!("Transcription failed: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": sanitize_error("generate_captions") })),
            );
        }
    };

    let style = req.style.as_deref().unwrap_or("default");
    let max_chars = req.max_chars_per_line.unwrap_or(40);
    let font_size = req.font_size.unwrap_or(32);
    let color = req.color.as_deref().unwrap_or("#FFFFFF");
    let with_bg = req.background.is_some();

    match engine
        .generate_captions_from_transcription(
            project_id,
            &transcription,
            style,
            max_chars,
            font_size,
            color,
            with_bg,
        )
        .await
    {
        Ok(layers) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "captions_count": layers.len(),
                "layers": layers,
            })),
        ),
        Err(e) => {
            error!("Failed to generate captions: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": sanitize_error("generate_captions") })),
            )
        }
    }
}

pub async fn tts_handler(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<Uuid>,
    Json(req): Json<TTSRequest>,
) -> impl IntoResponse {
    let engine = VideoEngine::new(state.db.clone());
    let output_dir =
        std::env::var("VIDEO_AUDIO_DIR").unwrap_or_else(|_| "./audio/video".to_string());

    if let Err(e) = std::fs::create_dir_all(&output_dir) {
        error!("Failed to create audio directory: {e}");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": sanitize_error("tts") })),
        );
    }

    let voice = req.voice.as_deref().unwrap_or("alloy");
    let speed = req.speed.unwrap_or(1.0);
    let language = req.language.as_deref().unwrap_or("en");

    match engine
        .text_to_speech(&req.text, voice, speed, language, &output_dir)
        .await
    {
        Ok(tts_response) => {
            match engine
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
                    Json(serde_json::json!({
                        "audio_url": tts_response.audio_url,
                        "duration_ms": tts_response.duration_ms,
                        "audio_track": track,
                    })),
                ),
                Err(e) => {
                    error!("Failed to add audio track: {e}");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({ "error": sanitize_error("tts") })),
                    )
                }
            }
        }
        Err(e) => {
            error!("TTS failed: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": sanitize_error("tts") })),
            )
        }
    }
}

pub async fn detect_scenes_handler(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<Uuid>,
) -> impl IntoResponse {
    let engine = VideoEngine::new(state.db.clone());
    let output_dir =
        std::env::var("VIDEO_THUMBNAILS_DIR").unwrap_or_else(|_| "./thumbnails/video".to_string());

    if let Err(e) = std::fs::create_dir_all(&output_dir) {
        error!("Failed to create thumbnails directory: {e}");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": sanitize_error("detect_scenes") })),
        );
    }

    match engine.detect_scenes(project_id, 0.3, &output_dir).await {
        Ok(response) => (StatusCode::OK, Json(serde_json::json!(response))),
        Err(e) => {
            error!("Scene detection failed: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": sanitize_error("detect_scenes") })),
            )
        }
    }
}

pub async fn auto_reframe_handler(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<Uuid>,
    Json(req): Json<AutoReframeRequest>,
) -> impl IntoResponse {
    let engine = VideoEngine::new(state.db.clone());
    let clips = match engine.get_clips(project_id).await {
        Ok(c) => c,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Project not found" })),
            )
        }
    };

    let clip = match clips.first() {
        Some(c) => c,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "No clips in project" })),
            )
        }
    };

    let output_dir =
        std::env::var("VIDEO_REFRAME_DIR").unwrap_or_else(|_| "./reframed/video".to_string());
    if let Err(e) = std::fs::create_dir_all(&output_dir) {
        error!("Failed to create reframe directory: {e}");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": sanitize_error("auto_reframe") })),
        );
    }

    match engine
        .auto_reframe(
            project_id,
            clip.id,
            req.target_width,
            req.target_height,
            &output_dir,
        )
        .await
    {
        Ok(url) => (
            StatusCode::OK,
            Json(serde_json::json!({ "reframed_url": url })),
        ),
        Err(e) => {
            error!("Auto-reframe failed: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": sanitize_error("auto_reframe") })),
            )
        }
    }
}

pub async fn remove_background_handler(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<Uuid>,
    Json(req): Json<BackgroundRemovalRequest>,
) -> impl IntoResponse {
    let engine = VideoEngine::new(state.db.clone());

    match engine
        .remove_background(project_id, req.clip_id, req.replacement)
        .await
    {
        Ok(response) => (StatusCode::OK, Json(serde_json::json!(response))),
        Err(e) => {
            error!("Background removal failed: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": sanitize_error("remove_background") })),
            )
        }
    }
}

pub async fn enhance_video_handler(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<Uuid>,
    Json(req): Json<VideoEnhanceRequest>,
) -> impl IntoResponse {
    let engine = VideoEngine::new(state.db.clone());

    match engine.enhance_video(project_id, req).await {
        Ok(response) => (StatusCode::OK, Json(serde_json::json!(response))),
        Err(e) => {
            error!("Video enhancement failed: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": sanitize_error("enhance_video") })),
            )
        }
    }
}

pub async fn beat_sync_handler(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<Uuid>,
    Json(req): Json<BeatSyncRequest>,
) -> impl IntoResponse {
    let engine = VideoEngine::new(state.db.clone());

    match engine
        .detect_beats(project_id, req.audio_track_id, req.sensitivity)
        .await
    {
        Ok(response) => (StatusCode::OK, Json(serde_json::json!(response))),
        Err(e) => {
            error!("Beat sync failed: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": sanitize_error("beat_sync

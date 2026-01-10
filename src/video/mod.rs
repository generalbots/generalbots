mod analytics;
mod engine;
mod handlers;
mod models;
mod render;
mod schema;
mod websocket;

pub mod mcp_tools;

pub use analytics::{get_analytics_handler, record_view_handler, AnalyticsEngine};
pub use engine::VideoEngine;
pub use handlers::*;
pub use models::*;
pub use render::{start_render_worker, VideoRenderWorker};
pub use schema::*;
pub use websocket::{broadcast_export_progress, export_progress_websocket, ExportProgressBroadcaster};

use axum::{
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;

use crate::shared::state::AppState;

pub fn configure_video_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/video/projects", get(list_projects).post(create_project))
        .route(
            "/api/video/projects/:id",
            get(get_project).put(update_project).delete(delete_project),
        )
        .route(
            "/api/video/projects/:id/clips",
            get(get_clips).post(add_clip),
        )
        .route("/api/video/clips/:id", put(update_clip).delete(delete_clip))
        .route("/api/video/clips/:id/split", post(split_clip_handler))
        .route(
            "/api/video/projects/:id/layers",
            get(get_layers).post(add_layer),
        )
        .route(
            "/api/video/layers/:id",
            put(update_layer).delete(delete_layer),
        )
        .route(
            "/api/video/projects/:id/audio",
            get(get_audio_tracks).post(add_audio_track),
        )
        .route("/api/video/audio/:id", delete(delete_audio_track))
        .route("/api/video/projects/:id/upload", post(upload_media))
        .route("/api/video/projects/:id/preview", get(get_preview_frame))
        .route(
            "/api/video/projects/:id/transcribe",
            post(transcribe_handler),
        )
        .route(
            "/api/video/projects/:id/captions",
            post(generate_captions_handler),
        )
        .route("/api/video/projects/:id/tts", post(tts_handler))
        .route("/api/video/projects/:id/scenes", post(detect_scenes_handler))
        .route("/api/video/projects/:id/reframe", post(auto_reframe_handler))
        .route(
            "/api/video/projects/:id/remove-background",
            post(remove_background_handler),
        )
        .route("/api/video/projects/:id/enhance", post(enhance_video_handler))
        .route(
            "/api/video/projects/:id/beat-sync",
            post(beat_sync_handler),
        )
        .route(
            "/api/video/projects/:id/waveform",
            post(generate_waveform_handler),
        )
        .route(
            "/api/video/layers/:id/keyframes",
            get(get_keyframes).post(add_keyframe),
        )
        .route("/api/video/keyframes/:id", delete(delete_keyframe))
        .route("/api/video/templates", get(list_templates))
        .route(
            "/api/video/projects/:id/template",
            post(apply_template_handler),
        )
        .route(
            "/api/video/clips/:from_id/transition/:to_id",
            post(add_transition_handler),
        )
        .route("/api/video/projects/:id/chat", post(chat_edit))
        .route("/api/video/projects/:id/export", post(start_export))
        .route("/api/video/exports/:id/status", get(get_export_status))
        .route(
            "/api/video/projects/:id/analytics",
            get(get_analytics_handler),
        )
        .route("/api/video/analytics/view", post(record_view_handler))
        .route("/api/video/ws/export/:id", get(export_progress_websocket))
}

pub fn configure(router: Router<Arc<AppState>>) -> Router<Arc<AppState>> {
    router.merge(configure_video_routes())
}

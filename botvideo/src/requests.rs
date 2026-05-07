use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
    pub resolution_width: Option<i32>,
    pub resolution_height: Option<i32>,
    pub fps: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub playhead_ms: Option<i64>,
    pub selection_json: Option<serde_json::Value>,
    pub zoom_level: Option<f32>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddClipRequest {
    pub name: Option<String>,
    pub source_url: String,
    pub at_ms: Option<i64>,
    pub duration_ms: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateClipRequest {
    pub name: Option<String>,
    pub start_ms: Option<i64>,
    pub duration_ms: Option<i64>,
    pub trim_in_ms: Option<i64>,
    pub trim_out_ms: Option<i64>,
    pub volume: Option<f32>,
    pub transition_in: Option<String>,
    pub transition_out: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddLayerRequest {
    pub name: Option<String>,
    pub layer_type: String,
    pub start_ms: Option<i64>,
    pub end_ms: Option<i64>,
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub properties: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateLayerRequest {
    pub name: Option<String>,
    pub start_ms: Option<i64>,
    pub end_ms: Option<i64>,
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub rotation: Option<f32>,
    pub opacity: Option<f32>,
    pub properties: Option<serde_json::Value>,
    pub animation_in: Option<String>,
    pub animation_out: Option<String>,
    pub locked: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddAudioRequest {
    pub name: Option<String>,
    pub source_url: String,
    pub track_type: Option<String>,
    pub start_ms: Option<i64>,
    pub duration_ms: Option<i64>,
    pub volume: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateAudioRequest {
    pub name: Option<String>,
    pub start_ms: Option<i64>,
    pub duration_ms: Option<i64>,
    pub volume: Option<f32>,
    pub fade_in_ms: Option<i64>,
    pub fade_out_ms: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitClipRequest {
    pub at_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatEditRequest {
    pub message: String,
    pub playhead_ms: Option<i64>,
    pub selection: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewFrameRequest {
    pub at_ms: Option<i64>,
    pub width: Option<i32>,
    pub height: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscribeRequest {
    pub clip_id: Option<Uuid>,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateCaptionsRequest {
    pub style: Option<String>,
    pub max_chars_per_line: Option<i32>,
    pub font_size: Option<i32>,
    pub color: Option<String>,
    pub background: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TTSRequest {
    pub text: String,
    pub voice: Option<String>,
    pub speed: Option<f32>,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoReframeRequest {
    pub target_width: i32,
    pub target_height: i32,
    pub focus_mode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyTemplateRequest {
    pub template_id: String,
    pub customizations: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionRequest {
    pub transition_type: String,
    pub duration_ms: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportRequest {
    pub format: Option<String>,
    pub quality: Option<String>,
    pub save_to_library: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundRemovalRequest {
    pub clip_id: Uuid,
    pub replacement: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoEnhanceRequest {
    pub clip_id: Uuid,
    pub upscale_factor: Option<i32>,
    pub denoise: Option<bool>,
    pub stabilize: Option<bool>,
    pub color_correct: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeatSyncRequest {
    pub audio_track_id: Uuid,
    pub sensitivity: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddKeyframeRequest {
    pub property_name: String,
    pub time_ms: i64,
    pub value: serde_json::Value,
    pub easing: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaveformRequest {
    pub audio_track_id: Uuid,
    pub samples_per_second: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordViewRequest {
    pub export_id: Uuid,
    pub watch_time_ms: i64,
    pub completed: bool,
    pub country: Option<String>,
    pub device: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectFilters {
    pub status: Option<String>,
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

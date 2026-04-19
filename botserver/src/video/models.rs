use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::schema::*;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable, Insertable)]
#[diesel(table_name = video_projects)]
pub struct VideoProject {
    pub id: Uuid,
    pub organization_id: Option<Uuid>,
    pub created_by: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub resolution_width: i32,
    pub resolution_height: i32,
    pub fps: i32,
    pub total_duration_ms: i64,
    pub timeline_json: serde_json::Value,
    pub layers_json: serde_json::Value,
    pub audio_tracks_json: serde_json::Value,
    pub playhead_ms: i64,
    pub selection_json: serde_json::Value,
    pub zoom_level: f32,
    pub thumbnail_url: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable, Insertable)]
#[diesel(table_name = video_clips)]
pub struct VideoClip {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub source_url: String,
    pub start_ms: i64,
    pub duration_ms: i64,
    pub trim_in_ms: i64,
    pub trim_out_ms: i64,
    pub volume: f32,
    pub clip_order: i32,
    pub transition_in: Option<String>,
    pub transition_out: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable, Insertable)]
#[diesel(table_name = video_layers)]
pub struct VideoLayer {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub layer_type: String,
    pub track_index: i32,
    pub start_ms: i64,
    pub end_ms: i64,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub rotation: f32,
    pub opacity: f32,
    pub properties_json: serde_json::Value,
    pub animation_in: Option<String>,
    pub animation_out: Option<String>,
    pub locked: bool,
    pub keyframes_json: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable, Insertable)]
#[diesel(table_name = video_audio_tracks)]
pub struct VideoAudioTrack {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub source_url: String,
    pub track_type: String,
    pub start_ms: i64,
    pub duration_ms: i64,
    pub volume: f32,
    pub fade_in_ms: i64,
    pub fade_out_ms: i64,
    pub waveform_json: Option<serde_json::Value>,
    pub beat_markers_json: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable, Insertable)]
#[diesel(table_name = video_exports)]
pub struct VideoExport {
    pub id: Uuid,
    pub project_id: Uuid,
    pub format: String,
    pub quality: String,
    pub status: String,
    pub progress: i32,
    pub output_url: Option<String>,
    pub gbdrive_path: Option<String>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable, Insertable)]
#[diesel(table_name = video_command_history)]
pub struct VideoCommandHistory {
    pub id: Uuid,
    pub project_id: Uuid,
    pub user_id: Option<Uuid>,
    pub command_type: String,
    pub command_json: serde_json::Value,
    pub executed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable, Insertable)]
#[diesel(table_name = video_analytics)]
pub struct VideoAnalytics {
    pub id: Uuid,
    pub project_id: Uuid,
    pub export_id: Option<Uuid>,
    pub views: i64,
    pub unique_viewers: i64,
    pub total_watch_time_ms: i64,
    pub avg_watch_percent: f32,
    pub completions: i64,
    pub shares: i64,
    pub likes: i64,
    pub engagement_score: f32,
    pub viewer_retention_json: Option<serde_json::Value>,
    pub geography_json: Option<serde_json::Value>,
    pub device_json: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable, Insertable)]
#[diesel(table_name = video_keyframes)]
pub struct VideoKeyframe {
    pub id: Uuid,
    pub layer_id: Uuid,
    pub property_name: String,
    pub time_ms: i64,
    pub value_json: serde_json::Value,
    pub easing: String,
    pub created_at: DateTime<Utc>,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub resolution_width: i32,
    pub resolution_height: i32,
    pub fps: i32,
    pub total_duration_ms: i64,
    pub playhead_ms: i64,
    pub zoom_level: f32,
    pub thumbnail_url: Option<String>,
    pub status: String,
    pub clips_count: usize,
    pub layers_count: usize,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectDetailResponse {
    pub project: ProjectResponse,
    pub clips: Vec<VideoClip>,
    pub layers: Vec<VideoLayer>,
    pub audio_tracks: Vec<VideoAudioTrack>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatEditResponse {
    pub success: bool,
    pub message: String,
    pub commands_executed: Vec<String>,
    pub project: Option<ProjectDetailResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadResponse {
    pub file_url: String,
    pub file_name: String,
    pub file_size: u64,
    pub mime_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionSegment {
    pub start_ms: i64,
    pub end_ms: i64,
    pub text: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionResponse {
    pub segments: Vec<TranscriptionSegment>,
    pub full_text: String,
    pub language: String,
    pub duration_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TTSResponse {
    pub audio_url: String,
    pub duration_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneInfo {
    pub start_ms: i64,
    pub end_ms: i64,
    pub thumbnail_url: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneDetectionResponse {
    pub scenes: Vec<SceneInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub thumbnail_url: String,
    pub duration_ms: i64,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportStatusResponse {
    pub id: Uuid,
    pub status: String,
    pub progress: i32,
    pub output_url: Option<String>,
    pub gbdrive_path: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundRemovalResponse {
    pub processed_url: String,
    pub duration_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoEnhanceResponse {
    pub enhanced_url: String,
    pub enhancements_applied: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeatMarker {
    pub time_ms: i64,
    pub strength: f32,
    pub beat_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeatSyncResponse {
    pub beats: Vec<BeatMarker>,
    pub tempo_bpm: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaveformResponse {
    pub samples: Vec<f32>,
    pub duration_ms: i64,
    pub sample_rate: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPoint {
    pub percent: f32,
    pub viewers: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoData {
    pub country: String,
    pub views: i64,
    pub percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceBreakdown {
    pub desktop: f32,
    pub mobile: f32,
    pub tablet: f32,
    pub tv: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsResponse {
    pub views: i64,
    pub unique_viewers: i64,
    pub total_watch_time_ms: i64,
    pub avg_watch_percent: f32,
    pub completions: i64,
    pub shares: i64,
    pub likes: i64,
    pub engagement_score: f32,
    pub viewer_retention: Vec<RetentionPoint>,
    pub top_countries: Vec<GeoData>,
    pub devices: DeviceBreakdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportProgressEvent {
    pub export_id: Uuid,
    pub project_id: Uuid,
    pub status: String,
    pub progress: i32,
    pub message: Option<String>,
    pub output_url: Option<String>,
    pub gbdrive_path: Option<String>,
}

pub enum ProjectStatus {
    Draft,
    Editing,
    Exporting,
    Published,
    Archived,
}

impl std::fmt::Display for ProjectStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Draft => write!(f, "draft"),
            Self::Editing => write!(f, "editing"),
            Self::Exporting => write!(f, "exporting"),
            Self::Published => write!(f, "published"),
            Self::Archived => write!(f, "archived"),
        }
    }
}

impl From<&str> for ProjectStatus {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "editing" => Self::Editing,
            "exporting" => Self::Exporting,
            "published" => Self::Published,
            "archived" => Self::Archived,
            _ => Self::Draft,
        }
    }
}

pub enum ExportStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

impl std::fmt::Display for ExportStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Processing => write!(f, "processing"),
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
        }
    }
}

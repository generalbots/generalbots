use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::*;

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

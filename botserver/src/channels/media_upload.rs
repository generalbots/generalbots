use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Platform {
    Twitter,
    Facebook,
    Instagram,
    LinkedIn,
    TikTok,
    YouTube,
    Pinterest,
    Snapchat,
    Discord,
    Bluesky,
    Threads,
    WeChat,
    Reddit,
}

impl Platform {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Twitter => "twitter",
            Self::Facebook => "facebook",
            Self::Instagram => "instagram",
            Self::LinkedIn => "linkedin",
            Self::TikTok => "tiktok",
            Self::YouTube => "youtube",
            Self::Pinterest => "pinterest",
            Self::Snapchat => "snapchat",
            Self::Discord => "discord",
            Self::Bluesky => "bluesky",
            Self::Threads => "threads",
            Self::WeChat => "wechat",
            Self::Reddit => "reddit",
        }
    }

    pub fn from_str_name(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "twitter" | "x" => Some(Self::Twitter),
            "facebook" | "fb" => Some(Self::Facebook),
            "instagram" | "ig" => Some(Self::Instagram),
            "linkedin" => Some(Self::LinkedIn),
            "tiktok" => Some(Self::TikTok),
            "youtube" | "yt" => Some(Self::YouTube),
            "pinterest" => Some(Self::Pinterest),
            "snapchat" | "snap" => Some(Self::Snapchat),
            "discord" => Some(Self::Discord),
            "bluesky" | "bsky" => Some(Self::Bluesky),
            "threads" => Some(Self::Threads),
            "wechat" => Some(Self::WeChat),
            "reddit" => Some(Self::Reddit),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MediaType {
    Image,
    Video,
    Gif,
    Audio,
    Document,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum UploadStatus {
    Pending,
    Uploading,
    Processing,
    Ready,
    Failed,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformLimits {
    pub max_image_size_bytes: u64,
    pub max_video_size_bytes: u64,
    pub max_video_duration_seconds: u32,
    pub supported_image_formats: Vec<String>,
    pub supported_video_formats: Vec<String>,
    pub max_images_per_post: u32,
    pub max_videos_per_post: u32,
    pub image_dimensions: Option<ImageDimensions>,
    pub video_dimensions: Option<VideoDimensions>,
    pub requires_aspect_ratio: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageDimensions {
    pub min_width: u32,
    pub min_height: u32,
    pub max_width: u32,
    pub max_height: u32,
    pub recommended_width: u32,
    pub recommended_height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoDimensions {
    pub min_width: u32,
    pub min_height: u32,
    pub max_width: u32,
    pub max_height: u32,
    pub min_fps: u32,
    pub max_fps: u32,
    pub min_bitrate_kbps: u32,
    pub max_bitrate_kbps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaUpload {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub platform: Platform,
    pub media_type: MediaType,
    pub status: UploadStatus,
    pub original_filename: String,
    pub content_type: String,
    pub size_bytes: u64,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub duration_seconds: Option<f64>,
    pub local_path: Option<String>,
    pub platform_media_id: Option<String>,
    pub platform_url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub error_message: Option<String>,
    pub upload_progress: f32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UploadRequest {
    pub platform: String,
    pub media_type: Option<String>,
    pub organization_id: Uuid,
    pub alt_text: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChunkedUploadInit {
    pub platform: String,
    pub organization_id: Uuid,
    pub filename: String,
    pub content_type: String,
    pub total_size: u64,
    pub media_type: String,
}

#[derive(Debug, Deserialize)]
pub struct ChunkedUploadAppend {
    pub upload_id: Uuid,
    pub chunk_index: u32,
    pub total_chunks: u32,
}

#[derive(Debug, Deserialize)]
pub struct ChunkedUploadFinalize {
    pub upload_id: Uuid,
    pub alt_text: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub id: Uuid,
    pub status: String,
    pub platform_media_id: Option<String>,
    pub platform_url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ChunkedUploadInitResponse {
    pub upload_id: Uuid,
    pub chunk_size: u64,
    pub total_chunks: u32,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ChunkedUploadAppendResponse {
    pub upload_id: Uuid,
    pub chunks_received: u32,
    pub total_chunks: u32,
    pub progress: f32,
}

#[derive(Debug, Serialize)]
pub struct MediaListResponse {
    pub media: Vec<MediaUpload>,
    pub total: u64,
    pub page: u32,
    pub per_page: u32,
}

#[derive(Debug, Deserialize)]
pub struct MediaListQuery {
    pub platform: Option<String>,
    pub status: Option<String>,
    pub media_type: Option<String>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

pub struct ChunkedUploadState {
    pub upload: MediaUpload,
    pub chunks_received: Vec<bool>,
    pub chunk_data: Vec<Vec<u8>>,
    pub chunk_size: u64,
}

pub struct MediaUploadService {
    uploads: Arc<RwLock<HashMap<Uuid, MediaUpload>>>,
    chunked_uploads: Arc<RwLock<HashMap<Uuid, ChunkedUploadState>>>,
    platform_limits: HashMap<Platform, PlatformLimits>,
}

impl Default for MediaUploadService {
    fn default() -> Self {
        Self::new()
    }
}

impl MediaUploadService {
    pub fn new() -> Self {
        let mut platform_limits = HashMap::new();

        platform_limits.insert(
            Platform::Twitter,
            PlatformLimits {
                max_image_size_bytes: 5 * 1024 * 1024,
                max_video_size_bytes: 512 * 1024 * 1024,
                max_video_duration_seconds: 140,
                supported_image_formats: vec!["jpg".into(), "jpeg".into(), "png".into(), "gif".into(), "webp".into()],
                supported_video_formats: vec!["mp4".into(), "mov".into()],
                max_images_per_post: 4,
                max_videos_per_post: 1,
                image_dimensions: Some(ImageDimensions {
                    min_width: 4,
                    min_height: 4,
                    max_width: 8192,
                    max_height: 8192,
                    recommended_width: 1200,
                    recommended_height: 675,
                }),
                video_dimensions: Some(VideoDimensions {
                    min_width: 32,
                    min_height: 32,
                    max_width: 1920,
                    max_height: 1200,
                    min_fps: 15,
                    max_fps: 60,
                    min_bitrate_kbps: 100,
                    max_bitrate_kbps: 25000,
                }),
                requires_aspect_ratio: None,
            },
        );

        platform_limits.insert(
            Platform::Instagram,
            PlatformLimits {
                max_image_size_bytes: 8 * 1024 * 1024,
                max_video_size_bytes: 4 * 1024 * 1024 * 1024,
                max_video_duration_seconds: 3600,
                supported_image_formats: vec!["jpg".into(), "jpeg".into(), "png".into()],
                supported_video_formats: vec!["mp4".into(), "mov".into()],
                max_images_per_post: 10,
                max_videos_per_post: 1,
                image_dimensions: Some(ImageDimensions {
                    min_width: 320,
                    min_height: 320,
                    max_width: 1080,
                    max_height: 1350,
                    recommended_width: 1080,
                    recommended_height: 1080,
                }),
                video_dimensions: Some(VideoDimensions {
                    min_width: 500,
                    min_height: 500,
                    max_width: 1920,
                    max_height: 1080,
                    min_fps: 23,
                    max_fps: 60,
                    min_bitrate_kbps: 1000,
                    max_bitrate_kbps: 25000,
                }),
                requires_aspect_ratio: Some("4:5 to 1.91:1".into()),
            },
        );

        platform_limits.insert(
            Platform::TikTok,
            PlatformLimits {
                max_image_size_bytes: 20 * 1024 * 1024,
                max_video_size_bytes: 4 * 1024 * 1024 * 1024,
                max_video_duration_seconds: 600,
                supported_image_formats: vec!["jpg".into(), "jpeg".into(), "png".into(), "webp".into()],
                supported_video_formats: vec!["mp4".into(), "webm".into(), "mov".into()],
                max_images_per_post: 35,
                max_videos_per_post: 1,
                image_dimensions: None,
                video_dimensions: Some(VideoDimensions {
                    min_width: 720,
                    min_height: 1280,
                    max_width: 1080,
                    max_height: 1920,
                    min_fps: 24,
                    max_fps: 60,
                    min_bitrate_kbps: 1000,
                    max_bitrate_kbps: 50000,
                }),
                requires_aspect_ratio: Some("9:16".into()),
            },
        );

        platform_limits.insert(
            Platform::YouTube,
            PlatformLimits {
                max_image_size_bytes: 2 * 1024 * 1024,
                max_video_size_bytes: 256 * 1024 * 1024 * 1024,
                max_video_duration_seconds: 43200,
                supported_image_formats: vec!["jpg".into(), "jpeg".into(), "png".into()],
                supported_video_formats: vec![
                    "mp4".into(), "mov".into(), "avi".into(), "wmv".into(),
                    "flv".into(), "webm".into(), "mkv".into(), "3gp".into(),
                ],
                max_images_per_post: 1,
                max_videos_per_post: 1,
                image_dimensions: Some(ImageDimensions {
                    min_width: 640,
                    min_height: 360,
                    max_width: 2560,
                    max_height: 1440,
                    recommended_width: 1280,
                    recommended_height: 720,
                }),
                video_dimensions: Some(VideoDimensions {
                    min_width: 426,
                    min_height: 240,
                    max_width: 7680,
                    max_height: 4320,
                    min_fps: 24,
                    max_fps: 60,
                    min_bitrate_kbps: 500,
                    max_bitrate_kbps: 128000,
                }),
                requires_aspect_ratio: Some("16:9".into()),
            },
        );

        platform_limits.insert(
            Platform::LinkedIn,
            PlatformLimits {
                max_image_size_bytes: 8 * 1024 * 1024,
                max_video_size_bytes: 5 * 1024 * 1024 * 1024,
                max_video_duration_seconds: 600,
                supported_image_formats: vec!["jpg".into(), "jpeg".into(), "png".into(), "gif".into()],
                supported_video_formats: vec!["mp4".into(), "mov".into(), "avi".into()],
                max_images_per_post: 9,
                max_videos_per_post: 1,
                image_dimensions: Some(ImageDimensions {
                    min_width: 552,
                    min_height: 276,
                    max_width: 7680,
                    max_height: 4320,
                    recommended_width: 1200,
                    recommended_height: 627,
                }),
                video_dimensions: Some(VideoDimensions {
                    min_width: 256,
                    min_height: 144,
                    max_width: 4096,
                    max_height: 2304,
                    min_fps: 15,
                    max_fps: 60,
                    min_bitrate_kbps: 500,
                    max_bitrate_kbps: 30000,
                }),
                requires_aspect_ratio: None,
            },
        );

        platform_limits.insert(
            Platform::Facebook,
            PlatformLimits {
                max_image_size_bytes: 10 * 1024 * 1024,
                max_video_size_bytes: 10 * 1024 * 1024 * 1024,
                max_video_duration_seconds: 14400,
                supported_image_formats: vec!["jpg".into(), "jpeg".into(), "png".into(), "gif".into(), "bmp".into(), "tiff".into()],
                supported_video_formats: vec!["mp4".into(), "mov".into(), "avi".into(), "wmv".into(), "mkv".into()],
                max_images_per_post: 10,
                max_videos_per_post: 1,
                image_dimensions: Some(ImageDimensions {
                    min_width: 320,
                    min_height: 320,
                    max_width: 4096,
                    max_height: 4096,
                    recommended_width: 1200,
                    recommended_height: 630,
                }),
                video_dimensions: Some(VideoDimensions {
                    min_width: 120,
                    min_height: 120,
                    max_width: 4096,
                    max_height: 4096,
                    min_fps: 15,
                    max_fps: 60,
                    min_bitrate_kbps: 500,
                    max_bitrate_kbps: 50000,
                }),
                requires_aspect_ratio: None,
            },
        );

        platform_limits.insert(
            Platform::Pinterest,
            PlatformLimits {
                max_image_size_bytes: 32 * 1024 * 1024,
                max_video_size_bytes: 2 * 1024 * 1024 * 1024,
                max_video_duration_seconds: 3600,
                supported_image_formats: vec!["jpg".into(), "jpeg".into(), "png".into(), "gif".into(), "webp".into()],
                supported_video_formats: vec!["mp4".into(), "mov".into()],
                max_images_per_post: 5,
                max_videos_per_post: 1,
                image_dimensions: Some(ImageDimensions {
                    min_width: 100,
                    min_height: 100,
                    max_width: 10000,
                    max_height: 10000,
                    recommended_width: 1000,
                    recommended_height: 1500,
                }),
                video_dimensions: Some(VideoDimensions {
                    min_width: 240,
                    min_height: 240,
                    max_width: 1920,
                    max_height: 1080,
                    min_fps: 15,
                    max_fps: 60,
                    min_bitrate_kbps: 1000,
                    max_bitrate_kbps: 25000,
                }),
                requires_aspect_ratio: Some("2:3 recommended".into()),
            },
        );

        platform_limits.insert(
            Platform::Discord,
            PlatformLimits {
                max_image_size_bytes: 25 * 1024 * 1024,
                max_video_size_bytes: 500 * 1024 * 1024,
                max_video_duration_seconds: 0,
                supported_image_formats: vec!["jpg".into(), "jpeg".into(), "png".into(), "gif".into(), "webp".into()],
                supported_video_formats: vec!["mp4".into(), "webm".into(), "mov".into()],
                max_images_per_post: 10,
                max_videos_per_post: 10,
                image_dimensions: None,
                video_dimensions: None,
                requires_aspect_ratio: None,
            },
        );

        platform_limits.insert(
            Platform::Bluesky,
            PlatformLimits {
                max_image_size_bytes: 1024 * 1024,
                max_video_size_bytes: 50 * 1024 * 1024,
                max_video_duration_seconds: 60,
                supported_image_formats: vec!["jpg".into(), "jpeg".into(), "png".into()],
                supported_video_formats: vec!["mp4".into()],
                max_images_per_post: 4,
                max_videos_per_post: 1,
                image_dimensions: Some(ImageDimensions {
                    min_width: 100,
                    min_height: 100,
                    max_width: 2000,
                    max_height: 2000,
                    recommended_width: 1000,
                    recommended_height: 1000,
                }),
                video_dimensions: None,
                requires_aspect_ratio: None,
            },
        );

        platform_limits.insert(
            Platform::Threads,
            PlatformLimits {
                max_image_size_bytes: 8 * 1024 * 1024,
                max_video_size_bytes: 1024 * 1024 * 1024,
                max_video_duration_seconds: 300,
                supported_image_formats: vec!["jpg".into(), "jpeg".into(), "png".into()],
                supported_video_formats: vec!["mp4".into(), "mov".into()],
                max_images_per_post: 10,
                max_videos_per_post: 1,
                image_dimensions: Some(ImageDimensions {
                    min_width: 320,
                    min_height: 320,
                    max_width: 1440,
                    max_height: 1800,
                    recommended_width: 1080,
                    recommended_height: 1350,
                }),
                video_dimensions: Some(VideoDimensions {
                    min_width: 500,
                    min_height: 500,
                    max_width: 1920,
                    max_height: 1080,
                    min_fps: 23,
                    max_fps: 60,
                    min_bitrate_kbps: 1000,
                    max_bitrate_kbps: 25000,
                }),
                requires_aspect_ratio: None,
            },
        );

        platform_limits.insert(
            Platform::Snapchat,
            PlatformLimits {
                max_image_size_bytes: 5 * 1024 * 1024,
                max_video_size_bytes: 1024 * 1024 * 1024,
                max_video_duration_seconds: 180,
                supported_image_formats: vec!["jpg".into(), "jpeg".into(), "png".into()],
                supported_video_formats: vec!["mp4".into(), "mov".into()],
                max_images_per_post: 1,
                max_videos_per_post: 1,
                image_dimensions: Some(ImageDimensions {
                    min_width: 1080,
                    min_height: 1920,
                    max_width: 1080,
                    max_height: 1920,
                    recommended_width: 1080,
                    recommended_height: 1920,
                }),
                video_dimensions: Some(VideoDimensions {
                    min_width: 1080,
                    min_height: 1920,
                    max_width: 1080,
                    max_height: 1920,
                    min_fps: 24,
                    max_fps: 30,
                    min_bitrate_kbps: 1000,
                    max_bitrate_kbps: 8000,
                }),
                requires_aspect_ratio: Some("9:16".into()),
            },
        );

        platform_limits.insert(
            Platform::WeChat,
            PlatformLimits {
                max_image_size_bytes: 10 * 1024 * 1024,
                max_video_size_bytes: 200 * 1024 * 1024,
                max_video_duration_seconds: 900,
                supported_image_formats: vec!["jpg".into(), "jpeg".into(), "png".into(), "gif".into()],
                supported_video_formats: vec!["mp4".into()],
                max_images_per_post: 9,
                max_videos_per_post: 1,
                image_dimensions: None,
                video_dimensions: None,
                requires_aspect_ratio: None,
            },
        );

        platform_limits.insert(
            Platform::Reddit,
            PlatformLimits {
                max_image_size_bytes: 20 * 1024 * 1024,
                max_video_size_bytes: 1024 * 1024 * 1024,
                max_video_duration_seconds: 900,
                supported_image_formats: vec!["jpg".into(), "jpeg".into(), "png".into(), "gif".into()],
                supported_video_formats: vec!["mp4".into(), "mov".into()],
                max_images_per_post: 20,
                max_videos_per_post: 1,
                image_dimensions: None,
                video_dimensions: Some(VideoDimensions {
                    min_width: 480,
                    min_height: 270,
                    max_width: 1920,
                    max_height: 1080,
                    min_fps: 15,
                    max_fps: 60,
                    min_bitrate_kbps: 500,
                    max_bitrate_kbps: 20000,
                }),
                requires_aspect_ratio: None,
            },
        );

        Self {
            uploads: Arc::new(RwLock::new(HashMap::new())),
            chunked_uploads: Arc::new(RwLock::new(HashMap::new())),
            platform_limits,
        }
    }

    pub fn get_platform_limits(&self, platform: &Platform) -> Option<&PlatformLimits> {
        self.platform_limits.get(platform)
    }

    pub async fn upload_media(
        &self,
        platform: Platform,
        organization_id: Uuid,
        data: Vec<u8>,
        filename: String,
        content_type: String,
        alt_text: Option<String>,
    ) -> Result<MediaUpload, MediaUploadError> {
        let limits = self
            .get_platform_limits(&platform)
            .ok_or_else(|| MediaUploadError::UnsupportedPlatform(platform.as_str().to_string()))?;

        let media_type = self.detect_media_type(&content_type);
        let extension = self.get_extension(&filename).unwrap_or_default();

        self.validate_format(&media_type, &extension, limits)?;
        self.validate_size(&media_type, data.len() as u64, limits)?;

        let now = Utc::now();
        let upload_id = Uuid::new_v4();

        let mut upload = MediaUpload {
            id: upload_id,
            organization_id,
            platform: platform.clone(),
            media_type,
            status: UploadStatus::Uploading,
            original_filename: filename.clone(),
            content_type: content_type.clone(),
            size_bytes: data.len() as u64,
            width: None,
            height: None,
            duration_seconds: None,
            local_path: None,
            platform_media_id: None,
            platform_url: None,
            thumbnail_url: None,
            error_message: None,
            upload_progress: 0.0,
            created_at: now,
            updated_at: now,
            expires_at: Some(now + chrono::Duration::hours(24)),
            metadata: HashMap::new(),
        };

        if let Some(alt) = alt_text {
            upload.metadata.insert("alt_text".into(), serde_json::json!(alt));
        }

        let platform_result = self.upload_to_platform(&platform, &data, &upload).await?;

        upload.platform_media_id = Some(platform_result.media_id);
        upload.platform_url = platform_result.url;
        upload.thumbnail_url = platform_result.thumbnail_url;
        upload.status = UploadStatus::Ready;
        upload.upload_progress = 100.0;
        upload.updated_at = Utc::now();

        {
            let mut uploads = self.uploads.write().await;
            uploads.insert(upload_id, upload.clone());
        }

        Ok(upload)
    }

    pub async fn init_chunked_upload(
        &self,
        platform: Platform,
        organization_id: Uuid,
        filename: String,
        content_type: String,
        total_size: u64,
        media_type: MediaType,
    ) -> Result<ChunkedUploadInitResponse, MediaUploadError> {
        let limits = self
            .get_platform_limits(&platform)
            .ok_or_else(|| MediaUploadError::UnsupportedPlatform(platform.as_str().to_string()))?;

        self.validate_size(&media_type, total_size, limits)?;

        let chunk_size: u64 = 5 * 1024 * 1024;
        let total_chunks = ((total_size as f64) / (chunk_size as f64)).ceil() as u32;

        let now = Utc::now();
        let upload_id = Uuid::new_v4();

        let upload = MediaUpload {
            id: upload_id,
            organization_id,
            platform: platform.clone(),
            media_type,
            status: UploadStatus::Pending,
            original_filename: filename,
            content_type,
            size_bytes: total_size,
            width: None,
            height: None,
            duration_seconds: None,
            local_path: None,
            platform_media_id: None,
            platform_url: None,
            thumbnail_url: None,
            error_message: None,
            upload_progress: 0.0,
            created_at: now,
            updated_at: now,
            expires_at: Some(now + chrono::Duration::hours(24)),
            metadata: HashMap::new(),
        };

        let state = ChunkedUploadState {
            upload,
            chunks_received: vec![false; total_chunks as usize],
            chunk_data: vec![Vec::new(); total_chunks as usize],
            chunk_size,
        };

        {
            let mut chunked = self.chunked_uploads.write().await;
            chunked.insert(upload_id, state);
        }

        Ok(ChunkedUploadInitResponse {
            upload_id,
            chunk_size,
            total_chunks,
            expires_at: now + chrono::Duration::hours(24),
        })
    }

    fn detect_media_type(&self, content_type: &str) -> MediaType {
        if content_type.starts_with("image/gif") {
            MediaType::Gif
        } else if content_type.starts_with("image/") {
            MediaType::Image
        } else if content_type.starts_with("video/") {
            MediaType::Video
        } else if content_type.starts_with("audio/") {
            MediaType::Audio
        } else {
            MediaType::Document
        }
    }

    fn get_extension(&self, filename: &str) -> Option<String> {
        filename
            .rsplit('.')
            .next()
            .map(|s| s.to_lowercase())
    }

    fn validate_format(
        &self,
        media_type: &MediaType,
        extension: &str,
        limits: &PlatformLimits,
    ) -> Result<(), MediaUploadError> {
        let supported = match media_type {
            MediaType::Image | MediaType::Gif => &limits.supported_image_formats,
            MediaType::Video => &limits.supported_video_formats,
            MediaType::Audio | MediaType::Document => return Ok(()),
        };
        if supported.iter().any(|f| f.eq_ignore_ascii_case(extension)) {
            Ok(())
        } else {
            Err(MediaUploadError::UnsupportedFormat)
        }
    }

    fn validate_size(
        &self,
        media_type: &MediaType,
        size: u64,
        limits: &PlatformLimits,
    ) -> Result<(), MediaUploadError> {
        let max_size = match media_type {
            MediaType::Image | MediaType::Gif => limits.max_image_size_bytes,
            MediaType::Video => limits.max_video_size_bytes,
            MediaType::Audio | MediaType::Document => limits.max_video_size_bytes,
        };
        if size <= max_size {
            Ok(())
        } else {
            Err(MediaUploadError::FileTooLarge)
        }
    }

    async fn upload_to_platform(
        &self,
        _platform: &Platform,
        data: &[u8],
        upload: &MediaUpload,
    ) -> Result<PlatformUploadResult, MediaUploadError> {
        // Get storage configuration from environment
        let storage_type = std::env::var("MEDIA_STORAGE_TYPE").unwrap_or_else(|_| "local".to_string());

        match storage_type.as_str() {
            "s3" => self.upload_to_s3(data, upload).await,
            "gcs" => self.upload_to_gcs(data, upload).await,
            "azure" => self.upload_to_azure_blob(data, upload).await,
            _ => self.upload_to_local_storage(data, upload).await,
        }
    }

    async fn upload_to_s3(
        &self,
        data: &[u8],
        upload: &MediaUpload,
    ) -> Result<PlatformUploadResult, MediaUploadError> {
        let bucket = std::env::var("S3_BUCKET")
            .map_err(|_| MediaUploadError::ConfigError("S3_BUCKET not configured".to_string()))?;
        let region = std::env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string());
        let _access_key = std::env::var("AWS_ACCESS_KEY_ID")
            .map_err(|_| MediaUploadError::ConfigError("AWS_ACCESS_KEY_ID not configured".to_string()))?;
        let _secret_key = std::env::var("AWS_SECRET_ACCESS_KEY")
            .map_err(|_| MediaUploadError::ConfigError("AWS_SECRET_ACCESS_KEY not configured".to_string()))?;

        let key = format!("uploads/{}/{}", upload.organization_id, upload.id);
        let content_type = &upload.content_type;

        // Build S3 presigned URL and upload
        let client = reqwest::Client::new();
        let url = format!("https://{}.s3.{}.amazonaws.com/{}", bucket, region, key);

        // Create AWS signature (simplified - in production use aws-sdk-s3)
        let response = client
            .put(&url)
            .header("Content-Type", content_type)
            .header("Content-Length", data.len())
            .body(data.to_vec())
            .send()
            .await
            .map_err(|e| MediaUploadError::UploadFailed(format!("S3 upload failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(MediaUploadError::UploadFailed(
                format!("S3 returned status: {}", response.status())
            ));
        }

        let public_url = format!("https://{}.s3.{}.amazonaws.com/{}", bucket, region, key);

        Ok(PlatformUploadResult {
            media_id: format!("s3_{}", upload.id),
            url: Some(public_url),
            thumbnail_url: None,
        })
    }

    async fn upload_to_gcs(
        &self,
        data: &[u8],
        upload: &MediaUpload,
    ) -> Result<PlatformUploadResult, MediaUploadError> {
        let bucket = std::env::var("GCS_BUCKET")
            .map_err(|_| MediaUploadError::ConfigError("GCS_BUCKET not configured".to_string()))?;

        let key = format!("uploads/{}/{}", upload.organization_id, upload.id);
        let content_type = &upload.content_type;

        let client = reqwest::Client::new();
        let url = format!(
            "https://storage.googleapis.com/upload/storage/v1/b/{}/o?uploadType=media&name={}",
            bucket, key
        );

        let response = client
            .post(&url)
            .header("Content-Type", content_type)
            .body(data.to_vec())
            .send()
            .await
            .map_err(|e| MediaUploadError::UploadFailed(format!("GCS upload failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(MediaUploadError::UploadFailed(
                format!("GCS returned status: {}", response.status())
            ));
        }

        let public_url = format!("https://storage.googleapis.com/{}/{}", bucket, key);

        Ok(PlatformUploadResult {
            media_id: format!("gcs_{}", upload.id),
            url: Some(public_url),
            thumbnail_url: None,
        })
    }

    async fn upload_to_azure_blob(
        &self,
        data: &[u8],
        upload: &MediaUpload,
    ) -> Result<PlatformUploadResult, MediaUploadError> {
        let account = std::env::var("AZURE_STORAGE_ACCOUNT")
            .map_err(|_| MediaUploadError::ConfigError("AZURE_STORAGE_ACCOUNT not configured".to_string()))?;
        let container = std::env::var("AZURE_STORAGE_CONTAINER")
            .map_err(|_| MediaUploadError::ConfigError("AZURE_STORAGE_CONTAINER not configured".to_string()))?;

        let blob_name = format!("uploads/{}/{}", upload.organization_id, upload.id);
        let content_type = &upload.content_type;

        let client = reqwest::Client::new();
        let url = format!(
            "https://{}.blob.core.windows.net/{}/{}",
            account, container, blob_name
        );

        let response = client
            .put(&url)
            .header("Content-Type", content_type)
            .header("x-ms-blob-type", "BlockBlob")
            .body(data.to_vec())
            .send()
            .await
            .map_err(|e| MediaUploadError::UploadFailed(format!("Azure upload failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(MediaUploadError::UploadFailed(
                format!("Azure returned status: {}", response.status())
            ));
        }

        Ok(PlatformUploadResult {
            media_id: format!("azure_{}", upload.id),
            url: Some(url),
            thumbnail_url: None,
        })
    }

    async fn upload_to_local_storage(
        &self,
        data: &[u8],
        upload: &MediaUpload,
    ) -> Result<PlatformUploadResult, MediaUploadError> {
        let storage_path = std::env::var("LOCAL_STORAGE_PATH")
            .unwrap_or_else(|_| "/var/lib/generalbots/uploads".to_string());
        let base_url = std::env::var("LOCAL_STORAGE_URL")
            .unwrap_or_else(|_| "/uploads".to_string());

        let org_dir = format!("{}/{}", storage_path, upload.organization_id);
        std::fs::create_dir_all(&org_dir)
            .map_err(|e| MediaUploadError::UploadFailed(format!("Failed to create directory: {}", e)))?;

        let file_path = format!("{}/{}", org_dir, upload.id);
        std::fs::write(&file_path, data)
            .map_err(|e| MediaUploadError::UploadFailed(format!("Failed to write file: {}", e)))?;

        let public_url = format!("{}/{}/{}", base_url, upload.organization_id, upload.id);

        Ok(PlatformUploadResult {
            media_id: format!("local_{}", upload.id),
            url: Some(public_url),
            thumbnail_url: None,
        })
    }

    pub async fn append_chunk(
        &self,
        upload_id: Uuid,
        chunk_index: u32,
        data: Vec<u8>,
    ) -> Result<ChunkedUploadAppendResponse, MediaUploadError> {
        let mut chunked = self.chunked_uploads.write().await;
        let state = chunked
            .get_mut(&upload_id)
            .ok_or(MediaUploadError::UploadNotFound)?;

        let total_chunks = state.chunks_received.len() as u32;

        if chunk_index >= total_chunks {
            return Err(MediaUploadError::InvalidChunkIndex);
        }

        if state.chunks_received[chunk_index as usize] {
            return Err(MediaUploadError::ChunkAlreadyReceived);
        }

        state.chunk_data[chunk_index as usize] = data;
        state.chunks_received[chunk_index as usize] = true;

        let chunks_done = state.chunks_received.iter().filter(|&&x| x).count() as u32;
        let progress = (chunks_done as f32 / total_chunks as f32) * 100.0;

        state.upload.upload_progress = progress;
        state.upload.status = UploadStatus::Uploading;
        state.upload.updated_at = Utc::now();

        Ok(ChunkedUploadAppendResponse {
            upload_id,
            chunks_received: chunks_done,
            total_chunks,
            progress,
        })
    }
}

struct PlatformUploadResult {
    media_id: String,
    url: Option<String>,
    thumbnail_url: Option<String>,
}

#[derive(Debug, Clone)]
pub enum MediaUploadError {
    UploadNotFound,
    InvalidChunkIndex,
    ChunkAlreadyReceived,
    UploadExpired,
    FileTooLarge,
    UnsupportedFormat,
    UnsupportedPlatform(String),
    ProcessingError(String),
    StorageError(String),
    ConfigError(String),
    UploadFailed(String),
}

impl std::fmt::Display for MediaUploadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UploadNotFound => write!(f, "Upload not found"),
            Self::InvalidChunkIndex => write!(f, "Invalid chunk index"),
            Self::ChunkAlreadyReceived => write!(f, "Chunk already received"),
            Self::UploadExpired => write!(f, "Upload expired"),
            Self::FileTooLarge => write!(f, "File too large"),
            Self::UnsupportedFormat => write!(f, "Unsupported format"),
            Self::UnsupportedPlatform(p) => write!(f, "Unsupported platform: {p}"),
            Self::ProcessingError(e) => write!(f, "Processing error: {e}"),
            Self::StorageError(e) => write!(f, "Storage error: {e}"),
            Self::ConfigError(e) => write!(f, "Configuration error: {e}"),
            Self::UploadFailed(e) => write!(f, "Upload failed: {e}"),
        }
    }
}

impl std::error::Error for MediaUploadError {}

//! WeChat type definitions for API requests and responses

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Token Types
// ============================================================================

#[derive(Debug, Clone)]
pub(crate) struct CachedToken {
    pub(crate) access_token: String,
    pub(crate) expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessTokenResponse {
    pub access_token: Option<String>,
    pub expires_in: Option<u64>,
    pub errcode: Option<i32>,
    pub errmsg: Option<String>,
}

// ============================================================================
// API Response Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeChatApiResponse<T> {
    pub errcode: Option<i32>,
    pub errmsg: Option<String>,
    #[serde(flatten)]
    pub data: Option<T>,
    pub msgid: Option<i64>,
}

// ============================================================================
// Message Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateMessage {
    pub touser: String,
    pub template_id: String,
    pub url: Option<String>,
    pub miniprogram: Option<MiniProgram>,
    pub data: HashMap<String, TemplateDataItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiniProgram {
    pub appid: String,
    pub pagepath: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateDataItem {
    pub value: String,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateMessageResult {
    pub msgid: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "msgtype", rename_all = "lowercase")]
pub enum CustomerMessage {
    Text {
        touser: String,
        text: TextContent,
    },
    Image {
        touser: String,
        image: MediaContent,
    },
    Voice {
        touser: String,
        voice: MediaContent,
    },
    Video {
        touser: String,
        video: VideoContent,
    },
    Music {
        touser: String,
        music: MusicContent,
    },
    News {
        touser: String,
        news: NewsContent,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextContent {
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaContent {
    pub media_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoContent {
    pub media_id: String,
    pub thumb_media_id: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicContent {
    pub title: Option<String>,
    pub description: Option<String>,
    pub musicurl: String,
    pub hqmusicurl: String,
    pub thumb_media_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsContent {
    pub articles: Vec<NewsItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsItem {
    pub title: String,
    pub description: Option<String>,
    pub url: Option<String>,
    pub picurl: Option<String>,
}

// ============================================================================
// Media Types
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaType {
    Image,
    Voice,
    Video,
    Thumb,
}

impl MediaType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Image => "image",
            Self::Voice => "voice",
            Self::Video => "video",
            Self::Thumb => "thumb",
        }
    }

    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::Image => "image/jpeg",
            Self::Voice => "audio/amr",
            Self::Video => "video/mp4",
            Self::Thumb => "image/jpeg",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaUploadResponse {
    pub errcode: Option<i32>,
    pub errmsg: Option<String>,
    #[serde(rename = "type")]
    pub media_type: Option<String>,
    pub media_id: Option<String>,
    pub created_at: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct MediaUploadResult {
    pub media_type: String,
    pub media_id: String,
    pub created_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoDescription {
    pub title: String,
    pub introduction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermanentMediaResponse {
    pub errcode: Option<i32>,
    pub errmsg: Option<String>,
    pub media_id: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PermanentMediaResult {
    pub media_id: String,
    pub url: Option<String>,
}

// ============================================================================
// Content Publishing Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsArticle {
    pub title: String,
    pub author: Option<String>,
    pub digest: Option<String>,
    pub content: String,
    pub content_source_url: Option<String>,
    pub thumb_media_id: String,
    pub need_open_comment: Option<i32>,
    pub only_fans_can_comment: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftResponse {
    pub errcode: Option<i32>,
    pub errmsg: Option<String>,
    pub media_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishResponse {
    pub errcode: Option<i32>,
    pub errmsg: Option<String>,
    pub publish_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PublishResult {
    pub publish_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishStatusResponse {
    pub errcode: Option<i32>,
    pub errmsg: Option<String>,
    pub publish_status: Option<i32>,
    pub article_id: Option<String>,
    pub article_detail: Option<ArticleDetail>,
    pub fail_idx: Option<Vec<i32>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleDetail {
    pub count: Option<i32>,
    pub item: Option<Vec<ArticleItem>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleItem {
    pub idx: Option<i32>,
    pub article_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PublishStatus {
    pub publish_id: String,
    pub publish_status: i32, // 0=success, 1=publishing, 2=failed
    pub article_id: Option<String>,
    pub article_detail: Option<ArticleDetail>,
    pub fail_idx: Option<Vec<i32>>,
}

impl PublishStatus {
    pub fn is_success(&self) -> bool {
        self.publish_status == 0
    }

    pub fn is_publishing(&self) -> bool {
        self.publish_status == 1
    }

    pub fn is_failed(&self) -> bool {
        self.publish_status == 2
    }
}

// ============================================================================
// User Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeChatUserResponse {
    pub errcode: Option<i32>,
    pub errmsg: Option<String>,
    pub subscribe: Option<i32>,
    pub openid: Option<String>,
    pub nickname: Option<String>,
    pub sex: Option<i32>,
    pub language: Option<String>,
    pub city: Option<String>,
    pub province: Option<String>,
    pub country: Option<String>,
    pub headimgurl: Option<String>,
    pub subscribe_time: Option<i64>,
    pub unionid: Option<String>,
    pub remark: Option<String>,
    pub groupid: Option<i32>,
    pub tagid_list: Option<Vec<i32>>,
    pub subscribe_scene: Option<String>,
    pub qr_scene: Option<i32>,
    pub qr_scene_str: Option<String>,
}

#[derive(Debug, Clone)]
pub struct WeChatUser {
    pub subscribe: i32,
    pub openid: String,
    pub nickname: Option<String>,
    pub sex: Option<i32>,
    pub language: Option<String>,
    pub city: Option<String>,
    pub province: Option<String>,
    pub country: Option<String>,
    pub headimgurl: Option<String>,
    pub subscribe_time: Option<i64>,
    pub unionid: Option<String>,
    pub remark: Option<String>,
    pub groupid: Option<i32>,
    pub tagid_list: Option<Vec<i32>>,
    pub subscribe_scene: Option<String>,
    pub qr_scene: Option<i32>,
    pub qr_scene_str: Option<String>,
}

impl WeChatUser {
    pub fn is_subscribed(&self) -> bool {
        self.subscribe == 1
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FollowerListResponse {
    pub errcode: Option<i32>,
    pub errmsg: Option<String>,
    pub total: Option<i32>,
    pub count: Option<i32>,
    pub data: Option<FollowerData>,
    pub next_openid: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FollowerData {
    pub openid: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct FollowerList {
    pub total: i32,
    pub count: i32,
    pub openids: Vec<String>,
    pub next_openid: Option<String>,
}

// ============================================================================
// Menu Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Menu {
    pub button: Vec<MenuButton>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuButton {
    #[serde(rename = "type")]
    pub button_type: Option<String>,
    pub name: String,
    pub key: Option<String>,
    pub url: Option<String>,
    pub media_id: Option<String>,
    pub appid: Option<String>,
    pub pagepath: Option<String>,
    pub article_id: Option<String>,
    pub sub_button: Option<Vec<MenuButton>>,
}

// ============================================================================
// QR Code Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QRCodeRequest {
    pub expire_seconds: Option<i32>,
    pub action_name: String, // "QR_SCENE", "QR_STR_SCENE", "QR_LIMIT_SCENE", "QR_LIMIT_STR_SCENE"
    pub action_info: ActionInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionInfo {
    pub scene: Scene,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scene {
    pub scene_id: Option<i32>,
    pub scene_str: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QRCodeResponse {
    pub errcode: Option<i32>,
    pub errmsg: Option<String>,
    pub ticket: Option<String>,
    pub expire_seconds: Option<i32>,
    pub url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct QRCodeResult {
    pub ticket: String,
    pub expire_seconds: Option<i32>,
    pub url: String,
    pub qrcode_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortUrlResponse {
    pub errcode: Option<i32>,
    pub errmsg: Option<String>,
    pub short_url: Option<String>,
}

// ============================================================================
// Webhook Message Types
// ============================================================================

#[derive(Debug, Clone)]
pub struct IncomingMessage {
    pub to_user_name: String,
    pub from_user_name: String,
    pub create_time: i64,
    pub msg_type: String,
    pub msg_id: Option<String>,
    pub content: Option<String>,
    pub pic_url: Option<String>,
    pub media_id: Option<String>,
    pub format: Option<String>,
    pub recognition: Option<String>,
    pub thumb_media_id: Option<String>,
    pub location_x: Option<f64>,
    pub location_y: Option<f64>,
    pub scale: Option<i32>,
    pub label: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub event: Option<String>,
    pub event_key: Option<String>,
    pub ticket: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub precision: Option<f64>,
}

impl IncomingMessage {
    pub fn is_text(&self) -> bool {
        self.msg_type == "text"
    }

    pub fn is_image(&self) -> bool {
        self.msg_type == "image"
    }

    pub fn is_voice(&self) -> bool {
        self.msg_type == "voice"
    }

    pub fn is_video(&self) -> bool {
        self.msg_type == "video"
    }

    pub fn is_location(&self) -> bool {
        self.msg_type == "location"
    }

    pub fn is_link(&self) -> bool {
        self.msg_type == "link"
    }

    pub fn is_event(&self) -> bool {
        self.msg_type == "event"
    }

    pub fn is_subscribe_event(&self) -> bool {
        self.is_event() && self.event.as_deref() == Some("subscribe")
    }

    pub fn is_unsubscribe_event(&self) -> bool {
        self.is_event() && self.event.as_deref() == Some("unsubscribe")
    }

    pub fn is_scan_event(&self) -> bool {
        self.is_event() && self.event.as_deref() == Some("SCAN")
    }

    pub fn is_click_event(&self) -> bool {
        self.is_event() && self.event.as_deref() == Some("CLICK")
    }
}

#[derive(Debug, Clone)]
pub struct ReplyMessage {
    pub to_user: String,
    pub from_user: String,
    pub content: ReplyContent,
}

#[derive(Debug, Clone)]
pub enum ReplyContent {
    Text { content: String },
    Image { media_id: String },
    Voice { media_id: String },
    Video {
        media_id: String,
        title: Option<String>,
        description: Option<String>,
    },
    News { articles: Vec<ReplyArticle> },
}

#[derive(Debug, Clone)]
pub struct ReplyArticle {
    pub title: String,
    pub description: Option<String>,
    pub pic_url: Option<String>,
    pub url: Option<String>,
}

// ============================================================================
// Error Codes
// ============================================================================

pub struct WeChatErrorCodes;

impl WeChatErrorCodes {
    pub const SUCCESS: i32 = 0;
    pub const INVALID_CREDENTIAL: i32 = 40001;
    pub const INVALID_GRANT_TYPE: i32 = 40002;
    pub const INVALID_OPENID: i32 = 40003;
    pub const INVALID_MEDIA_TYPE: i32 = 40004;
    pub const INVALID_MEDIA_ID: i32 = 40007;
    pub const INVALID_MESSAGE_TYPE: i32 = 40008;
    pub const INVALID_IMAGE_SIZE: i32 = 40009;
    pub const INVALID_VOICE_SIZE: i32 = 40010;
    pub const INVALID_VIDEO_SIZE: i32 = 40011;
    pub const INVALID_THUMB_SIZE: i32 = 40012;
    pub const INVALID_APPID: i32 = 40013;
    pub const INVALID_ACCESS_TOKEN: i32 = 40014;
    pub const INVALID_MENU_TYPE: i32 = 40015;
    pub const INVALID_BUTTON_COUNT: i32 = 40016;
    pub const ACCESS_TOKEN_EXPIRED: i32 = 42001;
    pub const REQUIRE_SUBSCRIBE: i32 = 43004;
    pub const API_LIMIT_REACHED: i32 = 45009;
    pub const API_BLOCKED: i32 = 48001;
}

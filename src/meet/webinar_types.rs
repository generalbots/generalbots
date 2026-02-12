// Webinar types extracted from webinar.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const MAX_WEBINAR_PARTICIPANTS: usize = 10000;
const MAX_RAISED_HANDS_VISIBLE: usize = 50;
const QA_QUESTION_MAX_LENGTH: usize = 1000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Webinar {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub meeting_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub scheduled_start: DateTime<Utc>,
    pub scheduled_end: Option<DateTime<Utc>>,
    pub actual_start: Option<DateTime<Utc>>,
    pub actual_end: Option<DateTime<Utc>>,
    pub status: WebinarStatus,
    pub settings: WebinarSettings,
    pub registration_required: bool,
    pub registration_url: Option<String>,
    pub host_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WebinarStatus {
    Draft,
    Scheduled,
    Live,
    Paused,
    Ended,
    Cancelled,
}

impl std::fmt::Display for WebinarStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Draft => write!(f, "draft"),
            Self::Scheduled => write!(f, "scheduled"),
            Self::Live => write!(f, "live"),
            Self::Paused => write!(f, "paused"),
            Self::Ended => write!(f, "ended"),
            Self::Cancelled => write!(f, "cancelled"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebinarSettings {
    pub allow_attendee_video: bool,
    pub allow_attendee_audio: bool,
    pub allow_chat: bool,
    pub allow_qa: bool,
    pub allow_hand_raise: bool,
    pub allow_reactions: bool,
    pub moderated_qa: bool,
    pub anonymous_qa: bool,
    pub auto_record: bool,
    pub waiting_room_enabled: bool,
    pub max_attendees: u32,
    pub mute_on_entry: bool,
    pub allow_screen_share: bool,
    pub enable_waiting_room: bool,
    pub breakout_rooms_enabled: bool,
}

impl Default for WebinarSettings {
    fn default() -> Self {
        Self {
            allow_attendee_video: true,
            allow_attendee_audio: true,
            allow_chat: true,
            allow_qa: true,
            allow_hand_raise: true,
            allow_reactions: true,
            moderated_qa: false,
            anonymous_qa: false,
            auto_record: false,
            waiting_room_enabled: false,
            max_attendees: 100,
            mute_on_entry: false,
            allow_screen_share: true,
            enable_waiting_room: false,
            breakout_rooms_enabled: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationField {
    pub id: Uuid,
    pub webinar_id: Uuid,
    pub field_label: String,
    pub field_type: FieldType,
    pub required: bool,
    pub options: Option<Vec<String>>,
    pub placeholder: Option<String>,
    pub display_order: i32,
}

impl RegistrationField {
    pub fn new(field_label: &str, field_type: FieldType, required: bool) -> Self {
        Self {
            id: Uuid::new_v4(),
            webinar_id: Uuid::new_v4(),
            field_label: field_label.to_string(),
            field_type,
            required,
            options: None,
            placeholder: None,
            display_order: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldType {
    Text,
    TextArea,
    Email,
    Phone,
    Number,
    Dropdown,
    Checkbox,
    Radio,
    Date,
    Url,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParticipantRole {
    Host,
    CoHost,
    Presenter,
    Moderator,
    Attendee,
}

impl std::fmt::Display for ParticipantRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Host => write!(f, "host"),
            Self::CoHost => write!(f, "co_host"),
            Self::Presenter => write!(f, "presenter"),
            Self::Moderator => write!(f, "moderator"),
            Self::Attendee => write!(f, "attendee"),
        }
    }
}

impl ParticipantRole {
    pub fn can_mute(&self) -> bool {
        matches!(self, Self::Host | Self::CoHost | Self::Moderator)
    }

    pub fn can_manage_polls(&self) -> bool {
        matches!(self, Self::Host | Self::CoHost | Self::Moderator)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebinarParticipant {
    pub id: Uuid,
    pub webinar_id: Uuid,
    pub user_id: Option<Uuid>,
    pub display_name: String,
    pub role: ParticipantRole,
    pub joined_at: DateTime<Utc>,
    pub left_at: Option<DateTime<Utc>>,
    pub hand_raised: bool,
    pub hand_raised_at: Option<DateTime<Utc>>,
    pub muted: bool,
    pub video_enabled: bool,
    pub screen_sharing: bool,
    pub connection_quality: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParticipantStatus {
    Waiting,
    InWaitingRoom,
    Active,
    Disconnected,
    Kicked,
}

impl std::fmt::Display for ParticipantStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Waiting => write!(f, "waiting"),
            Self::InWaitingRoom => write!(f, "in_waiting_room"),
            Self::Active => write!(f, "active"),
            Self::Disconnected => write!(f, "disconnected"),
            Self::Kicked => write!(f, "kicked"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QAQuestion {
    pub id: Uuid,
    pub webinar_id: Uuid,
    pub participant_id: Option<Uuid>,
    pub display_name: Option<String>,
    pub question: String,
    pub upvotes: i32,
    pub answered: bool,
    pub answered_at: Option<DateTime<Utc>>,
    pub answered_by: Option<Uuid>,
    pub status: QuestionStatus,
    pub asked_at: DateTime<Utc>,
    pub moderated: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuestionStatus {
    Pending,
    Approved,
    Answered,
    Rejected,
    Hidden,
}

impl std::fmt::Display for QuestionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Approved => write!(f, "approved"),
            Self::Answered => write!(f, "answered"),
            Self::Rejected => write!(f, "rejected"),
            Self::Hidden => write!(f, "hidden"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebinarPoll {
    pub id: Uuid,
    pub webinar_id: Uuid,
    pub question: String,
    pub poll_type: PollType,
    pub options: Vec<PollOption>,
    pub allow_multiple: bool,
    pub anonymous: bool,
    pub status: PollStatus,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub closes_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PollType {
    SingleChoice,
    MultipleChoice,
    Rating,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PollStatus {
    Open,
    Closed,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollOption {
    pub id: Uuid,
    pub poll_id: Uuid,
    pub option_text: String,
    pub display_order: i32,
    pub votes_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollVote {
    pub id: Uuid,
    pub poll_id: Uuid,
    pub option_id: Uuid,
    pub participant_id: Option<Uuid>,
    pub voted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebinarRegistration {
    pub id: Uuid,
    pub webinar_id: Uuid,
    pub user_id: Option<Uuid>,
    pub email: String,
    pub name: String,
    pub approved: bool,
    pub approved_at: Option<DateTime<Utc>>,
    pub approved_by: Option<Uuid>,
    pub cancel_token: Option<String>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub custom_fields: Option<serde_json::Value>,
    pub registered_at: DateTime<Utc>,
    pub status: RegistrationStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RegistrationStatus {
    Pending,
    Approved,
    Rejected,
    Cancelled,
    CheckedIn,
    NoShow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebinarAnalytics {
    pub webinar_id: Uuid,
    pub total_registrations: i32,
    pub total_attendees: i32,
    pub peak_concurrent: i32,
    pub avg_duration_minutes: f64,
    pub total_questions: i32,
    pub total_polls: i32,
    pub engagement_score: f64,
    pub chat_messages_count: i32,
    pub hand_raises_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebinarRecording {
    pub id: Uuid,
    pub webinar_id: Uuid,
    pub storage_path: String,
    pub duration_seconds: i32,
    pub size_bytes: i64,
    pub quality: RecordingQuality,
    pub status: RecordingStatus,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordingQuality {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordingStatus {
    Started,
    Processing,
    Completed,
    Failed,
    Deleted,
}

impl std::fmt::Display for RecordingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Started => write!(f, "started"),
            Self::Processing => write!(f, "processing"),
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
            Self::Deleted => write!(f, "deleted"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebinarTranscription {
    pub id: Uuid,
    pub webinar_id: Uuid,
    pub recording_id: Option<Uuid>,
    pub language: String,
    pub format: TranscriptionFormat,
    pub segments: Vec<TranscriptionSegment>,
    pub full_text: String,
    pub status: TranscriptionStatus,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TranscriptionStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

impl std::fmt::Display for TranscriptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::InProgress => write!(f, "in_progress"),
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionSegment {
    pub id: Uuid,
    pub transcription_id: Uuid,
    pub speaker_id: Option<Uuid>,
    pub speaker_name: Option<String>,
    pub start_time: f64,
    pub end_time: f64,
    pub text: String,
    pub confidence: f64,
    pub words: Vec<TranscriptionWord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionWord {
    pub id: Uuid,
    pub segment_id: Uuid,
    pub word: String,
    pub start_time: f64,
    pub end_time: f64,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartRecordingRequest {
    pub webinar_id: Uuid,
    pub quality: RecordingQuality,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTranscriptionRequest {
    pub webinar_id: Uuid,
    pub language: Option<String>,
    pub format: TranscriptionFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TranscriptionFormat {
    Text,
    Srt,
    Vtt,
    Json,
}

impl std::fmt::Display for TranscriptionFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text => write!(f, "text"),
            Self::Srt => write!(f, "srt"),
            Self::Vtt => write!(f, "vtt"),
            Self::Json => write!(f, "json"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPoint {
    pub id: Uuid,
    pub webinar_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub duration_seconds: i32,
    pub participant_count: i32,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWebinarRequest {
    pub title: String,
    pub description: Option<String>,
    pub scheduled_start: DateTime<Utc>,
    pub scheduled_end: Option<DateTime<Utc>>,
    pub settings: Option<WebinarSettings>,
    pub registration_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateWebinarRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub scheduled_start: Option<DateTime<Utc>>,
    pub scheduled_end: Option<DateTime<Utc>>,
    pub settings: Option<WebinarSettings>,
    pub status: Option<WebinarStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebinarStatsResponse {
    pub active_webinars: i32,
    pub total_participants: i32,
    pub total_minutes: i64,
    pub storage_used_bytes: i64,
}

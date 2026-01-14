use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Bool, Integer, Nullable, Text, Timestamptz, Uuid as DieselUuid};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::shared::state::AppState;

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
    pub practice_session_enabled: bool,
    pub attendee_registration_fields: Vec<RegistrationField>,
    /// Enable automatic transcription during recording
    pub auto_transcribe: bool,
    /// Language for transcription (e.g., "en-US", "es-ES")
    pub transcription_language: Option<String>,
    /// Enable speaker identification in transcription
    pub transcription_speaker_identification: bool,
    /// Store recording in cloud storage
    pub cloud_recording: bool,
    /// Recording quality setting
    pub recording_quality: RecordingQuality,
}

/// Recording quality settings
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum RecordingQuality {
    #[default]
    Standard,  // 720p
    High,      // 1080p
    Ultra,     // 4K
    AudioOnly, // Audio only recording
}

impl std::fmt::Display for RecordingQuality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecordingQuality::Standard => write!(f, "standard"),
            RecordingQuality::High => write!(f, "high"),
            RecordingQuality::Ultra => write!(f, "ultra"),
            RecordingQuality::AudioOnly => write!(f, "audio_only"),
        }
    }
}

impl Default for WebinarSettings {
    fn default() -> Self {
        Self {
            allow_attendee_video: false,
            allow_attendee_audio: false,
            allow_chat: true,
            allow_qa: true,
            allow_hand_raise: true,
            allow_reactions: true,
            moderated_qa: true,
            anonymous_qa: false,
            auto_record: false,
            waiting_room_enabled: true,
            max_attendees: MAX_WEBINAR_PARTICIPANTS as u32,
            practice_session_enabled: false,
            attendee_registration_fields: vec![
                RegistrationField::required("name"),
                RegistrationField::required("email"),
            ],
            auto_transcribe: true,
            transcription_language: Some("en-US".to_string()),
            transcription_speaker_identification: true,
            cloud_recording: true,
            recording_quality: RecordingQuality::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationField {
    pub name: String,
    pub field_type: FieldType,
    pub required: bool,
    pub options: Option<Vec<String>>,
}

impl RegistrationField {
    pub fn required(name: &str) -> Self {
        Self {
            name: name.to_string(),
            field_type: FieldType::Text,
            required: true,
            options: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldType {
    Text,
    Email,
    Phone,
    Select,
    Checkbox,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParticipantRole {
    Host,
    CoHost,
    Presenter,
    Panelist,
    Attendee,
}

impl std::fmt::Display for ParticipantRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Host => write!(f, "host"),
            Self::CoHost => write!(f, "co_host"),
            Self::Presenter => write!(f, "presenter"),
            Self::Panelist => write!(f, "panelist"),
            Self::Attendee => write!(f, "attendee"),
        }
    }
}

impl ParticipantRole {
    pub fn can_present(&self) -> bool {
        matches!(self, Self::Host | Self::CoHost | Self::Presenter | Self::Panelist)
    }

    pub fn can_manage(&self) -> bool {
        matches!(self, Self::Host | Self::CoHost)
    }

    pub fn can_speak(&self) -> bool {
        matches!(self, Self::Host | Self::CoHost | Self::Presenter | Self::Panelist)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebinarParticipant {
    pub id: Uuid,
    pub webinar_id: Uuid,
    pub user_id: Option<Uuid>,
    pub name: String,
    pub email: Option<String>,
    pub role: ParticipantRole,
    pub status: ParticipantStatus,
    pub hand_raised: bool,
    pub hand_raised_at: Option<DateTime<Utc>>,
    pub is_speaking: bool,
    pub video_enabled: bool,
    pub audio_enabled: bool,
    pub screen_sharing: bool,
    pub joined_at: Option<DateTime<Utc>>,
    pub left_at: Option<DateTime<Utc>>,
    pub registration_data: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParticipantStatus {
    Registered,
    InWaitingRoom,
    Joined,
    Left,
    Removed,
}

impl std::fmt::Display for ParticipantStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Registered => write!(f, "registered"),
            Self::InWaitingRoom => write!(f, "in_waiting_room"),
            Self::Joined => write!(f, "joined"),
            Self::Left => write!(f, "left"),
            Self::Removed => write!(f, "removed"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QAQuestion {
    pub id: Uuid,
    pub webinar_id: Uuid,
    pub asker_id: Option<Uuid>,
    pub asker_name: String,
    pub is_anonymous: bool,
    pub question: String,
    pub status: QuestionStatus,
    pub upvotes: i32,
    pub upvoted_by: Vec<Uuid>,
    pub answer: Option<String>,
    pub answered_by: Option<Uuid>,
    pub answered_at: Option<DateTime<Utc>>,
    pub is_pinned: bool,
    pub is_highlighted: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuestionStatus {
    Pending,
    Approved,
    Answered,
    Dismissed,
    AnsweredLive,
}

impl std::fmt::Display for QuestionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Approved => write!(f, "approved"),
            Self::Answered => write!(f, "answered"),
            Self::Dismissed => write!(f, "dismissed"),
            Self::AnsweredLive => write!(f, "answered_live"),
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
    pub status: PollStatus,
    pub show_results_to_attendees: bool,
    pub allow_multiple_answers: bool,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub launched_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PollType {
    SingleChoice,
    MultipleChoice,
    Rating,
    OpenEnded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PollStatus {
    Draft,
    Launched,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollOption {
    pub id: Uuid,
    pub text: String,
    pub vote_count: i32,
    pub percentage: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollVote {
    pub poll_id: Uuid,
    pub participant_id: Uuid,
    pub option_ids: Vec<Uuid>,
    pub open_response: Option<String>,
    pub voted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebinarRegistration {
    pub id: Uuid,
    pub webinar_id: Uuid,
    pub email: String,
    pub name: String,
    pub custom_fields: HashMap<String, String>,
    pub status: RegistrationStatus,
    pub join_link: String,
    pub registered_at: DateTime<Utc>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub cancelled_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RegistrationStatus {
    Pending,
    Confirmed,
    Cancelled,
    Attended,
    NoShow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebinarAnalytics {
    pub webinar_id: Uuid,
    pub total_registrations: u32,
    pub total_attendees: u32,
    pub peak_attendees: u32,
    pub average_watch_time_seconds: u64,
    pub total_questions: u32,
    pub answered_questions: u32,
    pub total_reactions: u32,
    pub poll_participation_rate: f32,
    pub engagement_score: f32,
    pub attendee_retention: Vec<RetentionPoint>,
    /// Recording information if available
    pub recording: Option<WebinarRecording>,
    /// Transcription information if available
    pub transcription: Option<WebinarTranscription>,
}

/// Webinar recording information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebinarRecording {
    pub id: Uuid,
    pub webinar_id: Uuid,
    pub status: RecordingStatus,
    pub duration_seconds: u64,
    pub file_size_bytes: u64,
    pub file_url: Option<String>,
    pub download_url: Option<String>,
    pub quality: RecordingQuality,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub processed_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub view_count: u32,
    pub download_count: u32,
}

/// Recording status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RecordingStatus {
    Recording,
    Processing,
    Ready,
    Failed,
    Deleted,
    Expired,
}

impl std::fmt::Display for RecordingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecordingStatus::Recording => write!(f, "recording"),
            RecordingStatus::Processing => write!(f, "processing"),
            RecordingStatus::Ready => write!(f, "ready"),
            RecordingStatus::Failed => write!(f, "failed"),
            RecordingStatus::Deleted => write!(f, "deleted"),
            RecordingStatus::Expired => write!(f, "expired"),
        }
    }
}

/// Webinar transcription information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebinarTranscription {
    pub id: Uuid,
    pub webinar_id: Uuid,
    pub recording_id: Uuid,
    pub status: TranscriptionStatus,
    pub language: String,
    pub duration_seconds: u64,
    pub word_count: u32,
    pub speaker_count: u32,
    pub segments: Vec<TranscriptionSegment>,
    pub full_text: Option<String>,
    pub vtt_url: Option<String>,
    pub srt_url: Option<String>,
    pub json_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub confidence_score: f32,
}

/// Transcription status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TranscriptionStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    PartiallyCompleted,
}

impl std::fmt::Display for TranscriptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TranscriptionStatus::Pending => write!(f, "pending"),
            TranscriptionStatus::InProgress => write!(f, "in_progress"),
            TranscriptionStatus::Completed => write!(f, "completed"),
            TranscriptionStatus::Failed => write!(f, "failed"),
            TranscriptionStatus::PartiallyCompleted => write!(f, "partially_completed"),
        }
    }
}

/// A segment of transcription with timing and speaker info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionSegment {
    pub id: Uuid,
    pub start_time_ms: u64,
    pub end_time_ms: u64,
    pub text: String,
    pub speaker_id: Option<String>,
    pub speaker_name: Option<String>,
    pub confidence: f32,
    pub words: Vec<TranscriptionWord>,
}

/// Individual word in transcription with timing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionWord {
    pub word: String,
    pub start_time_ms: u64,
    pub end_time_ms: u64,
    pub confidence: f32,
}

/// Request to start recording
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartRecordingRequest {
    pub quality: Option<RecordingQuality>,
    pub enable_transcription: Option<bool>,
    pub transcription_language: Option<String>,
}

/// Request to get transcription
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTranscriptionRequest {
    pub format: TranscriptionFormat,
    pub include_timestamps: bool,
    pub include_speaker_names: bool,
}

/// Transcription output format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TranscriptionFormat {
    PlainText,
    Vtt,
    Srt,
    Json,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPoint {
    pub minutes_from_start: i32,
    pub attendee_count: i32,
    pub percentage: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWebinarRequest {
    pub title: String,
    pub description: Option<String>,
    pub scheduled_start: DateTime<Utc>,
    pub scheduled_end: Option<DateTime<Utc>>,
    pub settings: Option<WebinarSettings>,
    pub registration_required: bool,
    pub panelists: Option<Vec<PanelistInvite>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelistInvite {
    pub email: String,
    pub name: String,
    pub role: ParticipantRole,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateWebinarRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub scheduled_start: Option<DateTime<Utc>>,
    pub scheduled_end: Option<DateTime<Utc>>,
    pub settings: Option<WebinarSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub name: String,
    pub email: String,
    pub custom_fields: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitQuestionRequest {
    pub question: String,
    pub is_anonymous: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerQuestionRequest {
    pub answer: String,
    pub mark_as_live: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePollRequest {
    pub question: String,
    pub poll_type: PollType,
    pub options: Vec<String>,
    pub allow_multiple_answers: Option<bool>,
    pub show_results_to_attendees: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VotePollRequest {
    pub option_ids: Vec<Uuid>,
    pub open_response: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleChangeRequest {
    pub participant_id: Uuid,
    pub new_role: ParticipantRole,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebinarEvent {
    pub event_type: WebinarEventType,
    pub webinar_id: Uuid,
    pub data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WebinarEventType {
    WebinarStarted,
    WebinarEnded,
    WebinarPaused,
    WebinarResumed,
    ParticipantJoined,
    ParticipantLeft,
    HandRaised,
    HandLowered,
    RoleChanged,
    QuestionSubmitted,
    QuestionAnswered,
    PollLaunched,
    PollClosed,
    ReactionSent,
    PresenterChanged,
    ScreenShareStarted,
    ScreenShareEnded,
    // Recording events
    RecordingStarted,
    RecordingStopped,
    RecordingPaused,
    RecordingResumed,
    RecordingProcessed,
    RecordingFailed,
    // Transcription events
    TranscriptionStarted,
    TranscriptionCompleted,
    TranscriptionFailed,
    TranscriptionSegmentReady,
}

#[derive(QueryableByName)]
struct WebinarRow {
    #[diesel(sql_type = DieselUuid)]
    id: Uuid,
    #[diesel(sql_type = DieselUuid)]
    organization_id: Uuid,
    #[diesel(sql_type = DieselUuid)]
    meeting_id: Uuid,
    #[diesel(sql_type = Text)]
    title: String,
    #[diesel(sql_type = Nullable<Text>)]
    description: Option<String>,
    #[diesel(sql_type = Timestamptz)]
    scheduled_start: DateTime<Utc>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    scheduled_end: Option<DateTime<Utc>>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    actual_start: Option<DateTime<Utc>>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    actual_end: Option<DateTime<Utc>>,
    #[diesel(sql_type = Text)]
    status: String,
    #[diesel(sql_type = Text)]
    settings_json: String,
    #[diesel(sql_type = Bool)]
    registration_required: bool,
    #[diesel(sql_type = Nullable<Text>)]
    registration_url: Option<String>,
    #[diesel(sql_type = DieselUuid)]
    host_id: Uuid,
    #[diesel(sql_type = Timestamptz)]
    created_at: DateTime<Utc>,
    #[diesel(sql_type = Timestamptz)]
    updated_at: DateTime<Utc>,
}

#[derive(QueryableByName)]
struct ParticipantRow {
    #[diesel(sql_type = DieselUuid)]
    id: Uuid,
    #[diesel(sql_type = DieselUuid)]
    webinar_id: Uuid,
    #[diesel(sql_type = Nullable<DieselUuid>)]
    user_id: Option<Uuid>,
    #[diesel(sql_type = Text)]
    name: String,
    #[diesel(sql_type = Nullable<Text>)]
    email: Option<String>,
    #[diesel(sql_type = Text)]
    role: String,
    #[diesel(sql_type = Text)]
    status: String,
    #[diesel(sql_type = Bool)]
    hand_raised: bool,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    hand_raised_at: Option<DateTime<Utc>>,
    #[diesel(sql_type = Bool)]
    is_speaking: bool,
    #[diesel(sql_type = Bool)]
    video_enabled: bool,
    #[diesel(sql_type = Bool)]
    audio_enabled: bool,
    #[diesel(sql_type = Bool)]
    screen_sharing: bool,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    joined_at: Option<DateTime<Utc>>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    left_at: Option<DateTime<Utc>>,
    #[diesel(sql_type = Nullable<Text>)]
    registration_data: Option<String>,
}

#[derive(QueryableByName)]
struct QuestionRow {
    #[diesel(sql_type = DieselUuid)]
    id: Uuid,
    #[diesel(sql_type = DieselUuid)]
    webinar_id: Uuid,
    #[diesel(sql_type = Nullable<DieselUuid>)]
    asker_id: Option<Uuid>,
    #[diesel(sql_type = Text)]
    asker_name: String,
    #[diesel(sql_type = Bool)]
    is_anonymous: bool,
    #[diesel(sql_type = Text)]
    question: String,
    #[diesel(sql_type = Text)]
    status: String,
    #[diesel(sql_type = Integer)]
    upvotes: i32,
    #[diesel(sql_type = Nullable<Text>)]
    upvoted_by: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    answer: Option<String>,
    #[diesel(sql_type = Nullable<DieselUuid>)]
    answered_by: Option<Uuid>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    answered_at: Option<DateTime<Utc>>,
    #[diesel(sql_type = Bool)]
    is_pinned: bool,
    #[diesel(sql_type = Bool)]
    is_highlighted: bool,
    #[diesel(sql_type = Timestamptz)]
    created_at: DateTime<Utc>,
}

#[derive(QueryableByName)]
struct CountRow {
    #[diesel(sql_type = BigInt)]
    count: i64,
}

pub struct WebinarService {
    pool: Arc<diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>>>,
    event_sender: broadcast::Sender<WebinarEvent>,
}

impl WebinarService {
    pub fn new(
        pool: Arc<diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>>>,
    ) -> Self {
        let (event_sender, _) = broadcast::channel(1000);
        Self { pool, event_sender }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<WebinarEvent> {
        self.event_sender.subscribe()
    }

    pub async fn create_webinar(
        &self,
        organization_id: Uuid,
        host_id: Uuid,
        request: CreateWebinarRequest,
    ) -> Result<Webinar, WebinarError> {
        let mut conn = self.pool.get().map_err(|e| {
            error!("Failed to get database connection: {e}");
            WebinarError::DatabaseConnection
        })?;

        let id = Uuid::new_v4();
        let meeting_id = Uuid::new_v4();
        let settings = request.settings.unwrap_or_default();
        let settings_json = serde_json::to_string(&settings).unwrap_or_else(|_| "{}".to_string());

        let registration_url = if request.registration_required {
            Some(format!("/webinar/{}/register", id))
        } else {
            None
        };

        let sql = r#"
            INSERT INTO webinars (
                id, organization_id, meeting_id, title, description,
                scheduled_start, scheduled_end, status, settings_json,
                registration_required, registration_url, host_id,
                created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, 'scheduled', $8, $9, $10, $11, NOW(), NOW()
            )
        "#;

        diesel::sql_query(sql)
            .bind::<DieselUuid, _>(id)
            .bind::<DieselUuid, _>(organization_id)
            .bind::<DieselUuid, _>(meeting_id)
            .bind::<Text, _>(&request.title)
            .bind::<Nullable<Text>, _>(request.description.as_deref())
            .bind::<Timestamptz, _>(request.scheduled_start)
            .bind::<Nullable<Timestamptz>, _>(request.scheduled_end)
            .bind::<Text, _>(&settings_json)
            .bind::<Bool, _>(request.registration_required)
            .bind::<Nullable<Text>, _>(registration_url.as_deref())
            .bind::<DieselUuid, _>(host_id)
            .execute(&mut conn)
            .map_err(|e| {
                error!("Failed to create webinar: {e}");
                WebinarError::CreateFailed
            })?;

        self.add_participant_internal(
            &mut conn,
            id,
            Some(host_id),
            "Host".to_string(),
            None,
            ParticipantRole::Host,
        )?;

        if let Some(panelists) = request.panelists {
            for panelist in panelists {
                self.add_participant_internal(
                    &mut conn,
                    id,
                    None,
                    panelist.name,
                    Some(panelist.email),
                    panelist.role,
                )?;
            }
        }

        info!("Created webinar {} for org {}", id, organization_id);

        self.get_webinar(id).await
    }

    pub async fn get_webinar(&self, webinar_id: Uuid) -> Result<Webinar, WebinarError> {
        let mut conn = self.pool.get().map_err(|_| WebinarError::DatabaseConnection)?;

        let sql = r#"
            SELECT id, organization_id, meeting_id, title, description,
                   scheduled_start, scheduled_end, actual_start, actual_end,
                   status, settings_json, registration_required, registration_url,
                   host_id, created_at, updated_at
            FROM webinars WHERE id = $1
        "#;

        let rows: Vec<WebinarRow> = diesel::sql_query(sql)
            .bind::<DieselUuid, _>(webinar_id)
            .load(&mut conn)
            .map_err(|e| {
                error!("Failed to get webinar: {e}");
                WebinarError::DatabaseConnection
            })?;

        let row = rows.into_iter().next().ok_or(WebinarError::NotFound)?;
        Ok(self.row_to_webinar(row))
    }

    pub async fn start_webinar(&self, webinar_id: Uuid, host_id: Uuid) -> Result<Webinar, WebinarError> {
        let webinar = self.get_webinar(webinar_id).await?;

        if webinar.host_id != host_id {
            return Err(WebinarError::NotAuthorized);
        }

        if webinar.status != WebinarStatus::Scheduled && webinar.status != WebinarStatus::Paused {
            return Err(WebinarError::InvalidState("Webinar cannot be started".to_string()));
        }

        let mut conn = self.pool.get().map_err(|_| WebinarError::DatabaseConnection)?;

        diesel::sql_query(
            "UPDATE webinars SET status = 'live', actual_start = COALESCE(actual_start, NOW()), updated_at = NOW() WHERE id = $1"
        )
        .bind::<DieselUuid, _>(webinar_id)
        .execute(&mut conn)
        .map_err(|e| {
            error!("Failed to start webinar: {e}");
            WebinarError::UpdateFailed
        })?;

        self.broadcast_event(WebinarEventType::WebinarStarted, webinar_id, serde_json::json!({}));

        info!("Started webinar {}", webinar_id);
        self.get_webinar(webinar_id).await
    }

    pub async fn end_webinar(&self, webinar_id: Uuid, host_id: Uuid) -> Result<Webinar, WebinarError> {
        let webinar = self.get_webinar(webinar_id).await?;

        if webinar.host_id != host_id {
            return Err(WebinarError::NotAuthorized);
        }

        let mut conn = self.pool.get().map_err(|_| WebinarError::DatabaseConnection)?;

        diesel::sql_query(
            "UPDATE webinars SET status = 'ended', actual_end = NOW(), updated_at = NOW() WHERE id = $1"
        )
        .bind::<DieselUuid, _>(webinar_id)
        .execute(&mut conn)
        .map_err(|e| {
            error!("Failed to end webinar: {e}");
            WebinarError::UpdateFailed
        })?;

        self.broadcast_event(WebinarEventType::WebinarEnded, webinar_id, serde_json::json!({}));

        info!("Ended webinar {}", webinar_id);
        self.get_webinar(webinar_id).await
    }

    pub async fn register_attendee(
        &self,
        webinar_id: Uuid,
        request: RegisterRequest,
    ) -> Result<WebinarRegistration, WebinarError> {
        let webinar = self.get_webinar(webinar_id).await?;

        if !webinar.registration_required {
            return Err(WebinarError::RegistrationNotRequired);
        }

        let mut conn = self.pool.get().map_err(|_| WebinarError::DatabaseConnection)?;

        let existing: Vec<CountRow> = diesel::sql_query(
            "SELECT COUNT(*) as count FROM webinar_registrations WHERE webinar_id = $1 AND email = $2"
        )
        .bind::<DieselUuid, _>(webinar_id)
        .bind::<Text, _>(&request.email)
        .load(&mut conn)
        .unwrap_or_default();

        if existing.first().map(|r| r.count > 0).unwrap_or(false) {
            return Err(WebinarError::AlreadyRegistered);
        }

        let id = Uuid::new_v4();
        let join_link = format!("/webinar/{}/join?token={}", webinar_id, Uuid::new_v4());
        let custom_fields = request.custom_fields.clone().unwrap_or_default();
        let custom_fields_json = serde_json::to_string(&custom_fields)
            .unwrap_or_else(|_| "{}".to_string());

        let sql = r#"
            INSERT INTO webinar_registrations (
                id, webinar_id, email, name, custom_fields, status, join_link,
                registered_at, confirmed_at
            ) VALUES ($1, $2, $3, $4, $5, 'confirmed', $6, NOW(), NOW())
        "#;

        diesel::sql_query(sql)
            .bind::<DieselUuid, _>(id)
            .bind::<DieselUuid, _>(webinar_id)
            .bind::<Text, _>(&request.email)
            .bind::<Text, _>(&request.name)
            .bind::<Text, _>(&custom_fields_json)
            .bind::<Text, _>(&join_link)
            .execute(&mut conn)
            .map_err(|e| {
                error!("Failed to register: {e}");
                WebinarError::RegistrationFailed
            })?;

        self.add_participant_internal(
            &mut conn,
            webinar_id,
            None,
            request.name.clone(),
            Some(request.email.clone()),
            ParticipantRole::Attendee,
        )?;

        Ok(WebinarRegistration {
            id,
            webinar_id,
            email: request.email,
            name: request.name,
            custom_fields,
            status: RegistrationStatus::Confirmed,
            join_link,
            registered_at: Utc::now(),
            confirmed_at: Some(Utc::now()),
            cancelled_at: None,
        })
    }

    pub async fn join_webinar(
        &self,
        webinar_id: Uuid,
        participant_id: Uuid,
    ) -> Result<WebinarParticipant, WebinarError> {
        let webinar = self.get_webinar(webinar_id).await?;

        if webinar.status != WebinarStatus::Live && webinar.status != WebinarStatus::Scheduled {
            return Err(WebinarError::InvalidState("Webinar is not active".to_string()));
        }

        let mut conn = self.pool.get().map_err(|_| WebinarError::DatabaseConnection)?;

        let status = if webinar.settings.waiting_room_enabled {
            "in_waiting_room"
        } else {
            "joined"
        };

        diesel::sql_query(
            "UPDATE webinar_participants SET status = $1, joined_at = NOW() WHERE id = $2"
        )
        .bind::<Text, _>(status)
        .bind::<DieselUuid, _>(participant_id)
        .execute(&mut conn)
        .map_err(|e| {
            error!("Failed to join webinar: {e}");
            WebinarError::JoinFailed
        })?;

        self.broadcast_event(
            WebinarEventType::ParticipantJoined,
            webinar_id,
            serde_json::json!({"participant_id": participant_id}),
        );

        self.get_participant(participant_id).await
    }

    pub async fn raise_hand(&self, webinar_id: Uuid, participant_id: Uuid) -> Result<(), WebinarError> {
        let webinar = self.get_webinar(webinar_id).await?;

        if !webinar.settings.allow_hand_raise {
            return Err(WebinarError::FeatureDisabled("Hand raising is disabled".to_string()));
        }

        let mut conn = self.pool.get().map_err(|_| WebinarError::DatabaseConnection)?;

        diesel::sql_query(
            "UPDATE webinar_participants SET hand_raised = TRUE, hand_raised_at = NOW() WHERE id = $1 AND webinar_id = $2"
        )
        .bind::<DieselUuid, _>(participant_id)
        .bind::<DieselUuid, _>(webinar_id)
        .execute(&mut conn)
        .map_err(|e| {
            error!("Failed to raise hand: {e}");
            WebinarError::UpdateFailed
        })?;

        self.broadcast_event(
            WebinarEventType::HandRaised,
            webinar_id,
            serde_json::json!({"participant_id": participant_id}),
        );

        Ok(())
    }

    pub async fn lower_hand(&self, webinar_id: Uuid, participant_id: Uuid) -> Result<(), WebinarError> {
        let mut conn = self.pool.get().map_err(|_| WebinarError::DatabaseConnection)?;

        diesel::sql_query(
            "UPDATE webinar_participants SET hand_raised = FALSE, hand_raised_at = NULL WHERE id = $1 AND webinar_id = $2"
        )
        .bind::<DieselUuid, _>(participant_id)
        .bind::<DieselUuid, _>(webinar_id)
        .execute(&mut conn)
        .map_err(|e| {
            error!("Failed to lower hand: {e}");
            WebinarError::UpdateFailed
        })?;

        self.broadcast_event(
            WebinarEventType::HandLowered,
            webinar_id,
            serde_json::json!({"participant_id": participant_id}),
        );

        Ok(())
    }

    pub async fn get_raised_hands(&self, webinar_id: Uuid) -> Result<Vec<WebinarParticipant>, WebinarError> {
        let mut conn = self.pool.get().map_err(|_| WebinarError::DatabaseConnection)?;

        let sql = r#"
            SELECT id, webinar_id, user_id, name, email, role, status,
                   hand_raised, hand_raised_at, is_speaking, video_enabled,
                   audio_enabled, screen_sharing, joined_at, left_at, registration_data
            FROM webinar_participants
            WHERE webinar_id = $1 AND hand_raised = TRUE
            ORDER BY hand_raised_at ASC
            LIMIT $2
        "#;

        let rows: Vec<ParticipantRow> = diesel::sql_query(sql)
            .bind::<DieselUuid, _>(webinar_id)
            .bind::<Integer, _>(MAX_RAISED_HANDS_VISIBLE as i32)
            .load(&mut conn)
            .unwrap_or_default();

        Ok(rows.into_iter().map(|r| self.row_to_participant(r)).collect())
    }

    pub async fn submit_question(
        &self,
        webinar_id: Uuid,
        asker_id: Option<Uuid>,
        asker_name: String,
        request: SubmitQuestionRequest,
    ) -> Result<QAQuestion, WebinarError> {
        let webinar = self.get_webinar(webinar_id).await?;

        if !webinar.settings.allow_qa {
            return Err(WebinarError::FeatureDisabled("Q&A is disabled".to_string()));
        }

        if request.question.len() > QA_QUESTION_MAX_LENGTH {
            return Err(WebinarError::InvalidInput("Question too long".to_string()));
        }

        let mut conn = self.pool.get().map_err(|_| WebinarError::DatabaseConnection)?;

        let id = Uuid::new_v4();
        let is_anonymous = request.is_anonymous.unwrap_or(false) && webinar.settings.anonymous_qa;
        let status = if webinar.settings.moderated_qa { "pending" } else { "approved" };
        let display_name = if is_anonymous { "Anonymous".to_string() } else { asker_name };

        let sql = r#"
            INSERT INTO webinar_questions (
                id, webinar_id, asker_id, asker_name, is_anonymous, question,
                status, upvotes, is_pinned, is_highlighted, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, 0, FALSE, FALSE, NOW())
        "#;

        diesel::sql_query(sql)
            .bind::<DieselUuid, _>(id)
            .bind::<DieselUuid, _>(webinar_id)
            .bind::<Nullable<DieselUuid>, _>(asker_id)
            .bind::<Text, _>(&display_name)
            .bind::<Bool, _>(is_anonymous)
            .bind::<Text, _>(&request.question)
            .bind::<Text, _>(status)
            .execute(&mut conn)
            .map_err(|e| {
                error!("Failed to submit question: {e}");
                WebinarError::CreateFailed
            })?;

        self.broadcast_event(
            WebinarEventType::QuestionSubmitted,
            webinar_id,
            serde_json::json!({"question_id": id}),
        );

        Ok(QAQuestion {
            id,
            webinar_id,
            asker_id,
            asker_name: display_name,
            is_anonymous,
            question: request.question,
            status: if webinar.settings.moderated_qa { QuestionStatus::Pending } else { QuestionStatus::Approved },
            upvotes: 0,
            upvoted_by: vec![],
            answer: None,
            answered_by: None,
            answered_at: None,
            is_pinned: false,
            is_highlighted: false,
            created_at: Utc::now(),
        })
    }

    pub async fn answer_question(
        &self,
        question_id: Uuid,
        answerer_id: Uuid,
        request: AnswerQuestionRequest,
    ) -> Result<QAQuestion, WebinarError> {
        let mut conn = self.pool.get().map_err(|_| WebinarError::DatabaseConnection)?;

        let status = if request.mark_as_live.unwrap_or(false) { "answered_live" } else { "answered" };

        diesel::sql_query(
            "UPDATE webinar_questions SET answer = $1, answered_by = $2, answered_at = NOW(), status = $3 WHERE id = $4"
        )
        .bind::<Text, _>(&request.answer)
        .bind::<DieselUuid, _>(answerer_id)
        .bind::<Text, _>(status)
        .bind::<DieselUuid, _>(question_id)
        .execute(&mut conn)
        .map_err(|e| {
            error!("Failed to answer question: {e}");
            WebinarError::UpdateFailed
        })?;

        self.get_question(question_id).await
    }

    pub async fn upvote_question(&self, question_id: Uuid, voter_id: Uuid) -> Result<QAQuestion, WebinarError> {
        let mut conn = self.pool.get().map_err(|_| WebinarError::DatabaseConnection)?;

        diesel::sql_query(
            "UPDATE webinar_questions SET upvotes = upvotes + 1, upvoted_by = COALESCE(upvoted_by, '[]')::jsonb || $1::jsonb WHERE id = $2"
        )
        .bind::<Text, _>(serde_json::json!([voter_id]).to_string())
        .bind::<DieselUuid, _>(question_id)
        .execute(&mut conn)
        .map_err(|e| {
            error!("Failed to upvote question: {e}");
            WebinarError::UpdateFailed
        })?;

        self.get_question(question_id).await
    }

    pub async fn get_questions(&self, webinar_id: Uuid, include_pending: bool) -> Result<Vec<QAQuestion>, WebinarError> {
        let mut conn = self.pool.get().map_err(|_| WebinarError::DatabaseConnection)?;

        let status_filter = if include_pending { "" } else { "AND status != 'pending'" };

        let sql = format!(r#"
            SELECT id, webinar_id, asker_id, asker_name, is_anonymous, question,
                   status, upvotes, upvoted_by, answer, answered_by, answered_at,
                   is_pinned, is_highlighted, created_at
            FROM webinar_questions
            WHERE webinar_id = $1 {status_filter}
            ORDER BY is_pinned DESC, upvotes DESC, created_at ASC
        "#);

        let rows: Vec<QuestionRow> = diesel::sql_query(&sql)
            .bind::<DieselUuid, _>(webinar_id)
            .load(&mut conn)
            .unwrap_or_default();

        Ok(rows.into_iter().map(|r| self.row_to_question(r)).collect())
    }

    async fn get_question(&self, question_id: Uuid) -> Result<QAQuestion, WebinarError> {
        let mut conn = self.pool.get().map_err(|_| WebinarError::DatabaseConnection)?;

        let sql = r#"
            SELECT id, webinar_id, asker_id, asker_name, is_anonymous, question,
                   status, upvotes, upvoted_by, answer, answered_by, answered_at,
                   is_pinned, is_highlighted, created_at
            FROM webinar_questions WHERE id = $1
        "#;

        let rows: Vec<QuestionRow> = diesel::sql_query(sql)
            .bind::<DieselUuid, _>(question_id)
            .load(&mut conn)
            .map_err(|_| WebinarError::DatabaseConnection)?;

        let row = rows.into_iter().next().ok_or(WebinarError::NotFound)?;
        Ok(self.row_to_question(row))
    }

    async fn get_participant(&self, participant_id: Uuid) -> Result<WebinarParticipant, WebinarError> {
        let mut conn = self.pool.get().map_err(|_| WebinarError::DatabaseConnection)?;

        let sql = r#"
            SELECT id, webinar_id, user_id, name, email, role, status,
                   hand_raised, hand_raised_at, is_speaking, video_enabled,
                   audio_enabled, screen_sharing, joined_at, left_at, registration_data
            FROM webinar_participants WHERE id = $1
        "#;

        let rows: Vec<ParticipantRow> = diesel::sql_query(sql)
            .bind::<DieselUuid, _>(participant_id)
            .load(&mut conn)
            .map_err(|_| WebinarError::DatabaseConnection)?;

        let row = rows.into_iter().next().ok_or(WebinarError::NotFound)?;
        Ok(self.row_to_participant(row))
    }

    fn add_participant_internal(
        &self,
        conn: &mut diesel::PgConnection,
        webinar_id: Uuid,
        user_id: Option<Uuid>,
        name: String,
        email: Option<String>,
        role: ParticipantRole,
    ) -> Result<Uuid, WebinarError> {
        let id = Uuid::new_v4();

        diesel::sql_query(r#"
            INSERT INTO webinar_participants (
                id, webinar_id, user_id, name, email, role, status,
                hand_raised, is_speaking, video_enabled, audio_enabled, screen_sharing
            ) VALUES ($1, $2, $3, $4, $5, $6, 'registered', FALSE, FALSE, FALSE, FALSE, FALSE)
        "#)
        .bind::<DieselUuid, _>(id)
        .bind::<DieselUuid, _>(webinar_id)
        .bind::<Nullable<DieselUuid>, _>(user_id)
        .bind::<Text, _>(&name)
        .bind::<Nullable<Text>, _>(email.as_deref())
        .bind::<Text, _>(role.to_string())
        .execute(conn)
        .map_err(|e| {
            error!("Failed to add participant: {e}");
            WebinarError::CreateFailed
        })?;

        Ok(id)
    }

    fn broadcast_event(&self, event_type: WebinarEventType, webinar_id: Uuid, data: serde_json::Value) {
        let event = WebinarEvent {
            event_type,
            webinar_id,
            data,
            timestamp: Utc::now(),
        };
        let _ = self.event_sender.send(event);
    }

    fn row_to_webinar(&self, row: WebinarRow) -> Webinar {
        let settings: WebinarSettings = serde_json::from_str(&row.settings_json).unwrap_or_default();
        let status = match row.status.as_str() {
            "draft" => WebinarStatus::Draft,
            "scheduled" => WebinarStatus::Scheduled,
            "live" => WebinarStatus::Live,
            "paused" => WebinarStatus::Paused,
            "ended" => WebinarStatus::Ended,
            "cancelled" => WebinarStatus::Cancelled,
            _ => WebinarStatus::Draft,
        };

        Webinar {
            id: row.id,
            organization_id: row.organization_id,
            meeting_id: row.meeting_id,
            title: row.title,
            description: row.description,
            scheduled_start: row.scheduled_start,
            scheduled_end: row.scheduled_end,
            actual_start: row.actual_start,
            actual_end: row.actual_end,
            status,
            settings,
            registration_required: row.registration_required,
            registration_url: row.registration_url,
            host_id: row.host_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }

    fn row_to_participant(&self, row: ParticipantRow) -> WebinarParticipant {
        let role = match row.role.as_str() {
            "host" => ParticipantRole::Host,
            "co_host" => ParticipantRole::CoHost,
            "presenter" => ParticipantRole::Presenter,
            "panelist" => ParticipantRole::Panelist,
            _ => ParticipantRole::Attendee,
        };
        let status = match row.status.as_str() {
            "registered" => ParticipantStatus::Registered,
            "in_waiting_room" => ParticipantStatus::InWaitingRoom,
            "joined" => ParticipantStatus::Joined,
            "left" => ParticipantStatus::Left,
            "removed" => ParticipantStatus::Removed,
            _ => ParticipantStatus::Registered,
        };
        let registration_data: Option<HashMap<String, String>> = row
            .registration_data
            .and_then(|d| serde_json::from_str(&d).ok());

        WebinarParticipant {
            id: row.id,
            webinar_id: row.webinar_id,
            user_id: row.user_id,
            name: row.name,
            email: row.email,
            role,
            status,
            hand_raised: row.hand_raised,
            hand_raised_at: row.hand_raised_at,
            is_speaking: row.is_speaking,
            video_enabled: row.video_enabled,
            audio_enabled: row.audio_enabled,
            screen_sharing: row.screen_sharing,
            joined_at: row.joined_at,
            left_at: row.left_at,
            registration_data,
        }
    }

    fn row_to_question(&self, row: QuestionRow) -> QAQuestion {
        let status = match row.status.as_str() {
            "pending" => QuestionStatus::Pending,
            "approved" => QuestionStatus::Approved,
            "answered" => QuestionStatus::Answered,
            "dismissed" => QuestionStatus::Dismissed,
            "answered_live" => QuestionStatus::AnsweredLive,
            _ => QuestionStatus::Pending,
        };
        let upvoted_by: Vec<Uuid> = row
            .upvoted_by
            .and_then(|u| serde_json::from_str(&u).ok())
            .unwrap_or_default();

        QAQuestion {
            id: row.id,
            webinar_id: row.webinar_id,
            asker_id: row.asker_id,
            asker_name: row.asker_name,
            is_anonymous: row.is_anonymous,
            question: row.question,
            status,
            upvotes: row.upvotes,
            upvoted_by,
            answer: row.answer,
            answered_by: row.answered_by,
            answered_at: row.answered_at,
            is_pinned: row.is_pinned,
            is_highlighted: row.is_highlighted,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, Clone)]
pub enum WebinarError {
    DatabaseConnection,
    NotFound,
    NotAuthorized,
    CreateFailed,
    UpdateFailed,
    JoinFailed,
    InvalidState(String),
    InvalidInput(String),
    FeatureDisabled(String),
    RegistrationNotRequired,
    RegistrationFailed,
    AlreadyRegistered,
    MaxParticipantsReached,
}

impl std::fmt::Display for WebinarError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DatabaseConnection => write!(f, "Database connection failed"),
            Self::NotFound => write!(f, "Webinar not found"),
            Self::NotAuthorized => write!(f, "Not authorized"),
            Self::CreateFailed => write!(f, "Failed to create"),
            Self::UpdateFailed => write!(f, "Failed to update"),
            Self::JoinFailed => write!(f, "Failed to join"),
            Self::InvalidState(msg) => write!(f, "Invalid state: {msg}"),
            Self::InvalidInput(msg) => write!(f, "Invalid input: {msg}"),
            Self::FeatureDisabled(msg) => write!(f, "Feature disabled: {msg}"),
            Self::RegistrationNotRequired => write!(f, "Registration not required"),
            Self::RegistrationFailed => write!(f, "Registration failed"),
            Self::AlreadyRegistered => write!(f, "Already registered"),
            Self::MaxParticipantsReached => write!(f, "Maximum participants reached"),
        }
    }
}

impl std::error::Error for WebinarError {}

impl IntoResponse for WebinarError {
    fn into_response(self) -> axum::response::Response {
        let status = match self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::NotAuthorized => StatusCode::FORBIDDEN,
            Self::AlreadyRegistered => StatusCode::CONFLICT,
            Self::InvalidInput(_) | Self::InvalidState(_) => StatusCode::BAD_REQUEST,
            Self::MaxParticipantsReached => StatusCode::SERVICE_UNAVAILABLE,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, self.to_string()).into_response()
    }
}

pub fn create_webinar_tables_migration() -> &'static str {
    r#"
    CREATE TABLE IF NOT EXISTS webinars (
        id UUID PRIMARY KEY,
        organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
        meeting_id UUID NOT NULL,
        title TEXT NOT NULL,
        description TEXT,
        scheduled_start TIMESTAMPTZ NOT NULL,
        scheduled_end TIMESTAMPTZ,
        actual_start TIMESTAMPTZ,
        actual_end TIMESTAMPTZ,
        status TEXT NOT NULL DEFAULT 'scheduled',
        settings_json TEXT NOT NULL DEFAULT '{}',
        registration_required BOOLEAN NOT NULL DEFAULT FALSE,
        registration_url TEXT,
        host_id UUID NOT NULL REFERENCES users(id),
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );

    CREATE TABLE IF NOT EXISTS webinar_participants (
        id UUID PRIMARY KEY,
        webinar_id UUID NOT NULL REFERENCES webinars(id) ON DELETE CASCADE,
        user_id UUID REFERENCES users(id),
        name TEXT NOT NULL,
        email TEXT,
        role TEXT NOT NULL DEFAULT 'attendee',
        status TEXT NOT NULL DEFAULT 'registered',
        hand_raised BOOLEAN NOT NULL DEFAULT FALSE,
        hand_raised_at TIMESTAMPTZ,
        is_speaking BOOLEAN NOT NULL DEFAULT FALSE,
        video_enabled BOOLEAN NOT NULL DEFAULT FALSE,
        audio_enabled BOOLEAN NOT NULL DEFAULT FALSE,
        screen_sharing BOOLEAN NOT NULL DEFAULT FALSE,
        joined_at TIMESTAMPTZ,
        left_at TIMESTAMPTZ,
        registration_data TEXT
    );

    CREATE TABLE IF NOT EXISTS webinar_registrations (
        id UUID PRIMARY KEY,
        webinar_id UUID NOT NULL REFERENCES webinars(id) ON DELETE CASCADE,
        email TEXT NOT NULL,
        name TEXT NOT NULL,
        custom_fields TEXT DEFAULT '{}',
        status TEXT NOT NULL DEFAULT 'pending',
        join_link TEXT NOT NULL,
        registered_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        confirmed_at TIMESTAMPTZ,
        cancelled_at TIMESTAMPTZ,
        UNIQUE(webinar_id, email)
    );

    CREATE TABLE IF NOT EXISTS webinar_questions (
        id UUID PRIMARY KEY,
        webinar_id UUID NOT NULL REFERENCES webinars(id) ON DELETE CASCADE,
        asker_id UUID REFERENCES users(id),
        asker_name TEXT NOT NULL,
        is_anonymous BOOLEAN NOT NULL DEFAULT FALSE,
        question TEXT NOT NULL,
        status TEXT NOT NULL DEFAULT 'pending',
        upvotes INTEGER NOT NULL DEFAULT 0,
        upvoted_by TEXT,
        answer TEXT,
        answered_by UUID REFERENCES users(id),
        answered_at TIMESTAMPTZ,
        is_pinned BOOLEAN NOT NULL DEFAULT FALSE,
        is_highlighted BOOLEAN NOT NULL DEFAULT FALSE,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );

    CREATE INDEX IF NOT EXISTS idx_webinars_org ON webinars(organization_id);
    CREATE INDEX IF NOT EXISTS idx_webinar_participants_webinar ON webinar_participants(webinar_id);
    CREATE INDEX IF NOT EXISTS idx_webinar_questions_webinar ON webinar_questions(webinar_id);
    "#
}

pub fn webinar_routes(_state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/", post(create_webinar_handler))
        .route("/:id", get(get_webinar_handler))
        .route("/:id/start", post(start_webinar_handler))
        .route("/:id/end", post(end_webinar_handler))
        .route("/:id/register", post(register_handler))
        .route("/:id/join", post(join_handler))
        .route("/:id/hand/raise", post(raise_hand_handler))
        .route("/:id/hand/lower", post(lower_hand_handler))
        .route("/:id/hands", get(get_raised_hands_handler))
        .route("/:id/questions", get(get_questions_handler))
        .route("/:id/questions", post(submit_question_handler))
        .route("/:id/questions/:question_id/answer", post(answer_question_handler))
        .route("/:id/questions/:question_id/upvote", post(upvote_question_handler))
        // Recording and transcription routes
        .route("/:id/recording/start", post(start_recording_handler))
        .route("/:id/recording/stop", post(stop_recording_handler))
}

async fn start_recording_handler(
    State(state): State<Arc<AppState>>,
    Path(webinar_id): Path<Uuid>,
) -> impl IntoResponse {
    let pool = state.conn.clone();
    let recording_id = Uuid::new_v4();
    let started_at = chrono::Utc::now();

    // Create recording record in database
    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| format!("DB error: {}", e))?;

        diesel::sql_query(
            "INSERT INTO meeting_recordings (id, room_id, status, started_at, created_at)
             VALUES ($1, $2, 'recording', $3, NOW())
             ON CONFLICT (room_id) WHERE status = 'recording' DO NOTHING"
        )
        .bind::<diesel::sql_types::Uuid, _>(recording_id)
        .bind::<diesel::sql_types::Uuid, _>(webinar_id)
        .bind::<diesel::sql_types::Timestamptz, _>(started_at)
        .execute(&mut conn)
        .map_err(|e| format!("Insert error: {}", e))?;

        Ok::<_, String>(recording_id)
    })
    .await;

    match result {
        Ok(Ok(id)) => Json(serde_json::json!({
            "status": "recording_started",
            "recording_id": id,
            "webinar_id": webinar_id,
            "started_at": started_at.to_rfc3339()
        })),
        Ok(Err(e)) => Json(serde_json::json!({
            "status": "error",
            "error": e
        })),
        Err(e) => Json(serde_json::json!({
            "status": "error",
            "error": format!("Task error: {}", e)
        })),
    }
}

async fn stop_recording_handler(
    State(state): State<Arc<AppState>>,
    Path(webinar_id): Path<Uuid>,
) -> impl IntoResponse {
    let pool = state.conn.clone();
    let stopped_at = chrono::Utc::now();

    // Update recording record to stopped status
    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| format!("DB error: {}", e))?;

        // Get the active recording and calculate duration
        let recording: Result<(Uuid, chrono::DateTime<chrono::Utc>), _> = diesel::sql_query(
            "SELECT id, started_at FROM meeting_recordings
             WHERE room_id = $1 AND status = 'recording'
             LIMIT 1"
        )
        .bind::<diesel::sql_types::Uuid, _>(webinar_id)
        .get_result::<RecordingRow>(&mut conn)
        .map(|r| (r.id, r.started_at));

        if let Ok((recording_id, started_at)) = recording {
            let duration_secs = (stopped_at - started_at).num_seconds();

            diesel::sql_query(
                "UPDATE meeting_recordings
                 SET status = 'stopped', stopped_at = $1, duration_seconds = $2, updated_at = NOW()
                 WHERE id = $3"
            )
            .bind::<diesel::sql_types::Timestamptz, _>(stopped_at)
            .bind::<diesel::sql_types::BigInt, _>(duration_secs)
            .bind::<diesel::sql_types::Uuid, _>(recording_id)
            .execute(&mut conn)
            .map_err(|e| format!("Update error: {}", e))?;

            Ok::<_, String>((recording_id, duration_secs))
        } else {
            Err("No active recording found".to_string())
        }
    })
    .await;

    match result {
        Ok(Ok((id, duration))) => Json(serde_json::json!({
            "status": "recording_stopped",
            "recording_id": id,
            "webinar_id": webinar_id,
            "stopped_at": stopped_at.to_rfc3339(),
            "duration_seconds": duration
        })),
        Ok(Err(e)) => Json(serde_json::json!({
            "status": "error",
            "error": e
        })),
        Err(e) => Json(serde_json::json!({
            "status": "error",
            "error": format!("Task error: {}", e)
        })),
    }
}

#[derive(diesel::QueryableByName)]
struct RecordingRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    started_at: chrono::DateTime<chrono::Utc>,
}

async fn create_webinar_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateWebinarRequest>,
) -> Result<Json<Webinar>, WebinarError> {
    let service = WebinarService::new(Arc::new(state.conn.clone()));
    let organization_id = Uuid::nil();
    let host_id = Uuid::nil();
    let webinar = service.create_webinar(organization_id, host_id, request).await?;
    Ok(Json(webinar))
}

async fn get_webinar_handler(
    State(state): State<Arc<AppState>>,
    Path(webinar_id): Path<Uuid>,
) -> Result<Json<Webinar>, WebinarError> {
    let service = WebinarService::new(Arc::new(state.conn.clone()));
    let webinar = service.get_webinar(webinar_id).await?;
    Ok(Json(webinar))
}

async fn start_webinar_handler(
    State(state): State<Arc<AppState>>,
    Path(webinar_id): Path<Uuid>,
) -> Result<Json<Webinar>, WebinarError> {
    let service = WebinarService::new(Arc::new(state.conn.clone()));
    let host_id = Uuid::nil();
    let webinar = service.start_webinar(webinar_id, host_id).await?;
    Ok(Json(webinar))
}

async fn end_webinar_handler(
    State(state): State<Arc<AppState>>,
    Path(webinar_id): Path<Uuid>,
) -> Result<Json<Webinar>, WebinarError> {
    let service = WebinarService::new(Arc::new(state.conn.clone()));
    let host_id = Uuid::nil();
    let webinar = service.end_webinar(webinar_id, host_id).await?;
    Ok(Json(webinar))
}

async fn register_handler(
    State(state): State<Arc<AppState>>,
    Path(webinar_id): Path<Uuid>,
    Json(request): Json<RegisterRequest>,
) -> Result<Json<WebinarRegistration>, WebinarError> {
    let service = WebinarService::new(Arc::new(state.conn.clone()));
    let registration = service.register_attendee(webinar_id, request).await?;
    Ok(Json(registration))
}

async fn join_handler(
    State(state): State<Arc<AppState>>,
    Path(webinar_id): Path<Uuid>,
) -> Result<Json<WebinarParticipant>, WebinarError> {
    let service = WebinarService::new(Arc::new(state.conn.clone()));
    let participant_id = Uuid::nil();
    let participant = service.join_webinar(webinar_id, participant_id).await?;
    Ok(Json(participant))
}

async fn raise_hand_handler(
    State(state): State<Arc<AppState>>,
    Path(webinar_id): Path<Uuid>,
) -> Result<StatusCode, WebinarError> {
    let service = WebinarService::new(Arc::new(state.conn.clone()));
    let participant_id = Uuid::nil();
    service.raise_hand(webinar_id, participant_id).await?;
    Ok(StatusCode::OK)
}

async fn lower_hand_handler(
    State(state): State<Arc<AppState>>,
    Path(webinar_id): Path<Uuid>,
) -> Result<StatusCode, WebinarError> {
    let service = WebinarService::new(Arc::new(state.conn.clone()));
    let participant_id = Uuid::nil();
    service.lower_hand(webinar_id, participant_id).await?;
    Ok(StatusCode::OK)
}

async fn get_raised_hands_handler(
    State(state): State<Arc<AppState>>,
    Path(webinar_id): Path<Uuid>,
) -> Result<Json<Vec<WebinarParticipant>>, WebinarError> {
    let service = WebinarService::new(Arc::new(state.conn.clone()));
    let hands = service.get_raised_hands(webinar_id).await?;
    Ok(Json(hands))
}

async fn get_questions_handler(
    State(state): State<Arc<AppState>>,
    Path(webinar_id): Path<Uuid>,
) -> Result<Json<Vec<QAQuestion>>, WebinarError> {
    let service = WebinarService::new(Arc::new(state.conn.clone()));
    let questions = service.get_questions(webinar_id, false).await?;
    Ok(Json(questions))
}

async fn submit_question_handler(
    State(state): State<Arc<AppState>>,
    Path(webinar_id): Path<Uuid>,
    Json(request): Json<SubmitQuestionRequest>,
) -> Result<Json<QAQuestion>, WebinarError> {
    let service = WebinarService::new(Arc::new(state.conn.clone()));
    let asker_id: Option<Uuid> = None;
    let question = service.submit_question(webinar_id, asker_id, "Anonymous".to_string(), request).await?;
    Ok(Json(question))
}

async fn answer_question_handler(
    State(state): State<Arc<AppState>>,
    Path((webinar_id, question_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<AnswerQuestionRequest>,
) -> Result<Json<QAQuestion>, WebinarError> {
    log::debug!("Answering question {question_id} in webinar {webinar_id}");
    let service = WebinarService::new(Arc::new(state.conn.clone()));
    let answerer_id = Uuid::nil();
    let question = service.answer_question(question_id, answerer_id, request).await?;
    Ok(Json(question))
}

async fn upvote_question_handler(
    State(state): State<Arc<AppState>>,
    Path((webinar_id, question_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<QAQuestion>, WebinarError> {
    log::debug!("Upvoting question {question_id} in webinar {webinar_id}");
    let service = WebinarService::new(Arc::new(state.conn.clone()));
    let voter_id = Uuid::nil();
    let question = service.upvote_question(question_id, voter_id).await?;
    Ok(Json(question))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webinar_status_display() {
        assert_eq!(WebinarStatus::Draft.to_string(), "draft");
        assert_eq!(WebinarStatus::Live.to_string(), "live");
        assert_eq!(WebinarStatus::Ended.to_string(), "ended");
    }

    #[test]
    fn test_participant_role_can_present() {
        assert!(ParticipantRole::Host.can_present());
        assert!(ParticipantRole::Presenter.can_present());
        assert!(!ParticipantRole::Attendee.can_present());
    }
}

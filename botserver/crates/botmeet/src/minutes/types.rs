use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetRecording {
    pub id: Uuid,
    pub bot_id: Uuid,
    pub meeting_id: Option<Uuid>,
    pub title: String,
    pub recording_path: String,
    pub duration_seconds: Option<i32>,
    pub file_size: Option<i64>,
    pub language: String,
    pub status: RecordingStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RecordingStatus {
    Recorded,
    Transcribing,
    Transcribed,
    Generating,
    Ready,
    Failed,
}

impl std::fmt::Display for RecordingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Recorded => write!(f, "recorded"),
            Self::Transcribing => write!(f, "transcribing"),
            Self::Transcribed => write!(f, "transcribed"),
            Self::Generating => write!(f, "generating"),
            Self::Ready => write!(f, "ready"),
            Self::Failed => write!(f, "failed"),
        }
    }
}

impl std::str::FromStr for RecordingStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "recorded" => Ok(Self::Recorded),
            "transcribing" => Ok(Self::Transcribing),
            "transcribed" => Ok(Self::Transcribed),
            "generating" => Ok(Self::Generating),
            "ready" => Ok(Self::Ready),
            "failed" => Ok(Self::Failed),
            _ => Err(format!("Invalid recording status: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transcription {
    pub id: Uuid,
    pub recording_id: Uuid,
    pub full_text: String,
    pub segments: Vec<TranscriptionSegment>,
    pub speakers: Vec<SpeakerEntry>,
    pub language: String,
    pub confidence: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionSegment {
    pub start: f64,
    pub end: f64,
    pub speaker: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeakerEntry {
    pub id: String,
    pub name: String,
    pub segments_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingMinute {
    pub id: Uuid,
    pub bot_id: Uuid,
    pub recording_id: Option<Uuid>,
    pub meeting_id: Option<Uuid>,
    pub title: String,
    pub summary: String,
    pub key_points: Vec<String>,
    pub action_items: Vec<MinuteActionItem>,
    pub decisions: Vec<String>,
    pub attendees: Vec<AttendeeEntry>,
    pub duration_minutes: Option<i32>,
    pub status: MinuteStatus,
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MinuteStatus {
    Draft,
    Final,
    Signed,
}

impl std::fmt::Display for MinuteStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Draft => write!(f, "draft"),
            Self::Final => write!(f, "final"),
            Self::Signed => write!(f, "signed"),
        }
    }
}

impl std::str::FromStr for MinuteStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "draft" => Ok(Self::Draft),
            "final" => Ok(Self::Final),
            "signed" => Ok(Self::Signed),
            _ => Err(format!("Invalid minute status: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinuteActionItem {
    pub task: String,
    pub assignee: Option<String>,
    pub due_date: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttendeeEntry {
    pub name: String,
    pub role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinuteSignature {
    pub id: Uuid,
    pub minute_id: Uuid,
    pub user_id: Uuid,
    pub signature_id: Option<Uuid>,
    pub signed_hash: String,
    pub signed_at: DateTime<Utc>,
    pub ip_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRecordingRequest {
    pub title: String,
    pub meeting_id: Option<Uuid>,
    pub language: Option<String>,
    pub duration_seconds: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateMinutesRequest {
    pub title: Option<String>,
    pub attendees: Option<Vec<AttendeeEntry>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignMinutesRequest {
    pub signature_id: Option<Uuid>,
    pub ip_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMinutesRequest {
    pub title: Option<String>,
    pub summary: Option<String>,
    pub key_points: Option<Vec<String>>,
    pub action_items: Option<Vec<MinuteActionItem>>,
    pub decisions: Option<Vec<String>>,
    pub attendees: Option<Vec<AttendeeEntry>>,
    pub duration_minutes: Option<i32>,
}

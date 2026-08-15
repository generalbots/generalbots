//! Request/response types shared by the meet conversation handlers and the
//! DB-backed conversation store modules.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateConversationRequest {
    pub name: String,
    pub description: Option<String>,
    pub conversation_type: Option<String>,
    pub participants: Vec<Uuid>,
    pub is_private: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct JoinConversationRequest {
    pub user_id: Uuid,
    pub display_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LeaveConversationRequest {
    pub user_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub content: String,
    pub message_type: Option<String>,
    pub reply_to: Option<Uuid>,
    pub attachments: Option<Vec<String>>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct EditMessageRequest {
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct ReactToMessageRequest {
    pub reaction: String,
}

#[derive(Debug, Deserialize)]
pub struct SearchMessagesQuery {
    pub query: String,
    pub from_date: Option<String>,
    pub to_date: Option<String>,
    pub user_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct StartCallRequest {
    pub call_type: String,
    pub participants: Option<Vec<Uuid>>,
}

#[derive(Debug, Deserialize)]
pub struct ScreenShareRequest {
    pub quality: Option<String>,
    pub audio_included: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct ConversationResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub conversation_type: String,
    pub is_private: bool,
    pub participant_count: u32,
    pub unread_count: u32,
    pub created_by: Uuid,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
    pub last_message: Option<MessageSummary>,
}

#[derive(Debug, Serialize)]
pub struct MessageSummary {
    pub id: Uuid,
    pub sender_id: Uuid,
    pub content: String,
    pub timestamp: chrono::DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageResponse {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub sender_id: Uuid,
    pub sender_name: String,
    pub content: String,
    pub message_type: String,
    pub reply_to: Option<Uuid>,
    pub attachments: Vec<String>,
    pub reactions: Vec<ReactionResponse>,
    pub is_pinned: bool,
    pub is_edited: bool,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReactionResponse {
    pub user_id: Uuid,
    pub reaction: String,
    pub timestamp: chrono::DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ParticipantResponse {
    pub user_id: Uuid,
    pub username: String,
    pub display_name: Option<String>,
    pub role: String,
    pub status: String,
    pub joined_at: chrono::DateTime<Utc>,
    pub is_typing: bool,
}

#[derive(Debug, Serialize)]
pub struct CallResponse {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub call_type: String,
    pub status: String,
    pub started_by: Uuid,
    pub participants: Vec<CallParticipant>,
    pub started_at: chrono::DateTime<Utc>,
    pub ended_at: Option<chrono::DateTime<Utc>>,
    pub duration_seconds: Option<i64>,
    pub recording_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CallParticipant {
    pub user_id: Uuid,
    pub username: String,
    pub status: String,
    pub is_muted: bool,
    pub is_video_enabled: bool,
    pub is_screen_sharing: bool,
    pub joined_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ScreenShareResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub conversation_id: Uuid,
    pub status: String,
    pub quality: String,
    pub audio_included: bool,
    pub started_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct WhiteboardResponse {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub name: String,
    pub created_by: Uuid,
    pub collaborators: Vec<Uuid>,
    pub content_url: String,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct SuccessResponse {
    pub success: bool,
    pub message: Option<String>,
}

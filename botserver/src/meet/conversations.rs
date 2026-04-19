use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::core::shared::state::AppState;

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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_message: Option<MessageSummary>,
}

#[derive(Debug, Serialize)]
pub struct MessageSummary {
    pub id: Uuid,
    pub sender_id: Uuid,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ReactionResponse {
    pub user_id: Uuid,
    pub reaction: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ParticipantResponse {
    pub user_id: Uuid,
    pub username: String,
    pub display_name: Option<String>,
    pub role: String,
    pub status: String,
    pub joined_at: DateTime<Utc>,
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
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub duration_seconds: Option<i64>,
    pub recording_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CallParticipant {
    pub user_id: Uuid,
    pub username: String,
    pub status: String,
    pub is_muted: bool,
    pub is_video_enabled: bool,
    pub is_screen_sharing: bool,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ScreenShareResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub conversation_id: Uuid,
    pub status: String,
    pub quality: String,
    pub audio_included: bool,
    pub started_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct WhiteboardResponse {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub name: String,
    pub created_by: Uuid,
    pub collaborators: Vec<Uuid>,
    pub content_url: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct SuccessResponse {
    pub success: bool,
    pub message: Option<String>,
}

pub async fn create_conversation(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<CreateConversationRequest>,
) -> Result<Json<ConversationResponse>, (StatusCode, Json<serde_json::Value>)> {
    let conversation_id = Uuid::new_v4();
    let now = Utc::now();
    let creator_id = Uuid::new_v4();

    let conversation = ConversationResponse {
        id: conversation_id,
        name: req.name,
        description: req.description,
        conversation_type: req.conversation_type.unwrap_or_else(|| "group".to_string()),
        is_private: req.is_private.unwrap_or(false),
        participant_count: req.participants.len() as u32,
        unread_count: 0,
        created_by: creator_id,
        created_at: now,
        updated_at: now,
        last_message: None,
    };

    Ok(Json(conversation))
}

pub async fn join_conversation(
    State(_state): State<Arc<AppState>>,
    Path(conversation_id): Path<Uuid>,
    Json(req): Json<JoinConversationRequest>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(SuccessResponse {
        success: true,
        message: Some(format!(
            "User {} joined conversation {}",
            req.user_id, conversation_id
        )),
    }))
}

pub async fn leave_conversation(
    State(_state): State<Arc<AppState>>,
    Path(conversation_id): Path<Uuid>,
    Json(req): Json<LeaveConversationRequest>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(SuccessResponse {
        success: true,
        message: Some(format!(
            "User {} left conversation {}",
            req.user_id, conversation_id
        )),
    }))
}

pub async fn get_conversation_members(
    State(_state): State<Arc<AppState>>,
    Path(_conversation_id): Path<Uuid>,
) -> Result<Json<Vec<ParticipantResponse>>, (StatusCode, Json<serde_json::Value>)> {
    let members = vec![ParticipantResponse {
        user_id: Uuid::new_v4(),
        username: "user1".to_string(),
        display_name: Some("User One".to_string()),
        role: "member".to_string(),
        status: "online".to_string(),
        joined_at: Utc::now(),
        is_typing: false,
    }];

    Ok(Json(members))
}

pub async fn get_conversation_messages(
    State(_state): State<Arc<AppState>>,
    Path(_conversation_id): Path<Uuid>,
) -> Result<Json<Vec<MessageResponse>>, (StatusCode, Json<serde_json::Value>)> {
    let messages = vec![];

    Ok(Json(messages))
}

pub async fn send_message(
    State(_state): State<Arc<AppState>>,
    Path(conversation_id): Path<Uuid>,
    Json(req): Json<SendMessageRequest>,
) -> Result<Json<MessageResponse>, (StatusCode, Json<serde_json::Value>)> {
    let message_id = Uuid::new_v4();
    let sender_id = Uuid::new_v4();
    let now = Utc::now();

    let message = MessageResponse {
        id: message_id,
        conversation_id,
        sender_id,
        sender_name: "User".to_string(),
        content: req.content,
        message_type: req.message_type.unwrap_or_else(|| "text".to_string()),
        reply_to: req.reply_to,
        attachments: req.attachments.unwrap_or_default(),
        reactions: vec![],
        is_pinned: false,
        is_edited: false,
        created_at: now,
        updated_at: now,
    };

    Ok(Json(message))
}

pub async fn edit_message(
    State(_state): State<Arc<AppState>>,
    Path((conversation_id, message_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<EditMessageRequest>,
) -> Result<Json<MessageResponse>, (StatusCode, Json<serde_json::Value>)> {
    let now = Utc::now();

    let message = MessageResponse {
        id: message_id,
        conversation_id,
        sender_id: Uuid::new_v4(),
        sender_name: "User".to_string(),
        content: req.content,
        message_type: "text".to_string(),
        reply_to: None,
        attachments: vec![],
        reactions: vec![],
        is_pinned: false,
        is_edited: true,
        created_at: now,
        updated_at: now,
    };

    Ok(Json(message))
}

pub async fn delete_message(
    State(_state): State<Arc<AppState>>,
    Path((_conversation_id, message_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(SuccessResponse {
        success: true,
        message: Some(format!("Message {} deleted", message_id)),
    }))
}

pub async fn react_to_message(
    State(_state): State<Arc<AppState>>,
    Path((_conversation_id, message_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<ReactToMessageRequest>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(SuccessResponse {
        success: true,
        message: Some(format!(
            "Reaction '{}' added to message {}",
            req.reaction, message_id
        )),
    }))
}

pub async fn pin_message(
    State(_state): State<Arc<AppState>>,
    Path((_conversation_id, message_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(SuccessResponse {
        success: true,
        message: Some(format!("Message {} pinned", message_id)),
    }))
}

pub async fn search_messages(
    State(_state): State<Arc<AppState>>,
    Path(_conversation_id): Path<Uuid>,
    Query(_params): Query<SearchMessagesQuery>,
) -> Result<Json<Vec<MessageResponse>>, (StatusCode, Json<serde_json::Value>)> {
    let messages = vec![];

    Ok(Json(messages))
}

pub async fn start_call(
    State(_state): State<Arc<AppState>>,
    Path(conversation_id): Path<Uuid>,
    Json(req): Json<StartCallRequest>,
) -> Result<Json<CallResponse>, (StatusCode, Json<serde_json::Value>)> {
    let call_id = Uuid::new_v4();
    let starter_id = Uuid::new_v4();
    let now = Utc::now();

    let call = CallResponse {
        id: call_id,
        conversation_id,
        call_type: req.call_type,
        status: "active".to_string(),
        started_by: starter_id,
        participants: vec![],
        started_at: now,
        ended_at: None,
        duration_seconds: None,
        recording_url: None,
    };

    Ok(Json(call))
}

pub async fn join_call(
    State(_state): State<Arc<AppState>>,
    Path(_conversation_id): Path<Uuid>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(SuccessResponse {
        success: true,
        message: Some("Joined call successfully".to_string()),
    }))
}

pub async fn leave_call(
    State(_state): State<Arc<AppState>>,
    Path(_conversation_id): Path<Uuid>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(SuccessResponse {
        success: true,
        message: Some("Left call successfully".to_string()),
    }))
}

pub async fn mute_call(
    State(_state): State<Arc<AppState>>,
    Path(_conversation_id): Path<Uuid>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(SuccessResponse {
        success: true,
        message: Some("Audio muted".to_string()),
    }))
}

pub async fn unmute_call(
    State(_state): State<Arc<AppState>>,
    Path(_conversation_id): Path<Uuid>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(SuccessResponse {
        success: true,
        message: Some("Audio unmuted".to_string()),
    }))
}

pub async fn start_screen_share(
    State(_state): State<Arc<AppState>>,
    Path(conversation_id): Path<Uuid>,
    Json(req): Json<ScreenShareRequest>,
) -> Result<Json<ScreenShareResponse>, (StatusCode, Json<serde_json::Value>)> {
    let share_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let now = Utc::now();

    let screen_share = ScreenShareResponse {
        id: share_id,
        user_id,
        conversation_id,
        status: "active".to_string(),
        quality: req.quality.unwrap_or_else(|| "high".to_string()),
        audio_included: req.audio_included.unwrap_or(false),
        started_at: now,
    };

    Ok(Json(screen_share))
}

pub async fn stop_screen_share(
    State(_state): State<Arc<AppState>>,
    Path(_conversation_id): Path<Uuid>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(SuccessResponse {
        success: true,
        message: Some("Screen sharing stopped".to_string()),
    }))
}

pub async fn start_recording(
    State(_state): State<Arc<AppState>>,
    Path(_conversation_id): Path<Uuid>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(SuccessResponse {
        success: true,
        message: Some("Recording started".to_string()),
    }))
}

pub async fn stop_recording(
    State(_state): State<Arc<AppState>>,
    Path(_conversation_id): Path<Uuid>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(SuccessResponse {
        success: true,
        message: Some("Recording stopped".to_string()),
    }))
}

pub async fn create_whiteboard(
    State(_state): State<Arc<AppState>>,
    Path(conversation_id): Path<Uuid>,
) -> Result<Json<WhiteboardResponse>, (StatusCode, Json<serde_json::Value>)> {
    let whiteboard_id = Uuid::new_v4();
    let creator_id = Uuid::new_v4();
    let now = Utc::now();

    let whiteboard = WhiteboardResponse {
        id: whiteboard_id,
        conversation_id,
        name: "New Whiteboard".to_string(),
        created_by: creator_id,
        collaborators: vec![creator_id],
        content_url: format!("/whiteboards/{}/content", whiteboard_id),
        created_at: now,
        updated_at: now,
    };

    Ok(Json(whiteboard))
}

pub async fn collaborate_whiteboard(
    State(_state): State<Arc<AppState>>,
    Path(_conversation_id): Path<Uuid>,
    Json(_data): Json<serde_json::Value>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(SuccessResponse {
        success: true,
        message: Some("Whiteboard collaboration started".to_string()),
    }))
}

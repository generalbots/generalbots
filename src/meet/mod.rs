use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use log::{error, info};
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;

use crate::shared::state::AppState;

pub mod service;
use service::{DefaultTranscriptionService, MeetingService};

pub async fn voice_start(
    State(data): State<Arc<AppState>>,
    Json(info): Json<Value>,
) -> impl IntoResponse {
    let session_id = info
        .get("session_id")
        .and_then(|s| s.as_str())
        .unwrap_or("");
    let user_id = info
        .get("user_id")
        .and_then(|u| u.as_str())
        .unwrap_or("user");

    info!(
        "Voice session start request - session: {}, user: {}",
        session_id, user_id
    );

    match data
        .voice_adapter
        .start_voice_session(session_id, user_id)
        .await
    {
        Ok(token) => {
            info!(
                "Voice session started successfully for session {}",
                session_id
            );
            (
                StatusCode::OK,
                Json(serde_json::json!({"token": token, "status": "started"})),
            )
        }
        Err(e) => {
            error!(
                "Failed to start voice session for session {}: {}",
                session_id, e
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
        }
    }
}

pub async fn voice_stop(
    State(data): State<Arc<AppState>>,
    Json(info): Json<Value>,
) -> impl IntoResponse {
    let session_id = info
        .get("session_id")
        .and_then(|s| s.as_str())
        .unwrap_or("");

    match data.voice_adapter.stop_voice_session(session_id).await {
        Ok(()) => {
            info!(
                "Voice session stopped successfully for session {}",
                session_id
            );
            (
                StatusCode::OK,
                Json(serde_json::json!({"status": "stopped"})),
            )
        }
        Err(e) => {
            error!(
                "Failed to stop voice session for session {}: {}",
                session_id, e
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
        }
    }
}

/// Create a new meeting room
pub async fn create_meeting(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateMeetingRequest>,
) -> impl IntoResponse {
    let transcription_service = Arc::new(DefaultTranscriptionService);
    let meeting_service = MeetingService::new(state.clone(), transcription_service);

    match meeting_service
        .create_room(payload.name, payload.created_by, payload.settings)
        .await
    {
        Ok(room) => {
            info!("Created meeting room: {}", room.id);
            (StatusCode::OK, Json(serde_json::json!(room)))
        }
        Err(e) => {
            error!("Failed to create meeting room: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
        }
    }
}

/// List all active meeting rooms
pub async fn list_rooms(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let transcription_service = Arc::new(DefaultTranscriptionService);
    let meeting_service = MeetingService::new(state.clone(), transcription_service);

    let rooms = meeting_service.rooms.read().await;
    let room_list: Vec<_> = rooms.values().cloned().collect();

    (StatusCode::OK, Json(serde_json::json!(room_list)))
}

/// Get a specific meeting room
pub async fn get_room(
    State(state): State<Arc<AppState>>,
    Path(room_id): Path<String>,
) -> impl IntoResponse {
    let transcription_service = Arc::new(DefaultTranscriptionService);
    let meeting_service = MeetingService::new(state.clone(), transcription_service);

    let rooms = meeting_service.rooms.read().await;
    match rooms.get(&room_id) {
        Some(room) => (StatusCode::OK, Json(serde_json::json!(room))),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Room not found"})),
        ),
    }
}

/// Join a meeting room
pub async fn join_room(
    State(state): State<Arc<AppState>>,
    Path(room_id): Path<String>,
    Json(payload): Json<JoinRoomRequest>,
) -> impl IntoResponse {
    let transcription_service = Arc::new(DefaultTranscriptionService);
    let meeting_service = MeetingService::new(state.clone(), transcription_service);

    match meeting_service
        .join_room(&room_id, payload.participant_name, payload.participant_id)
        .await
    {
        Ok(participant) => {
            info!("Participant {} joined room {}", participant.id, room_id);
            (StatusCode::OK, Json(serde_json::json!(participant)))
        }
        Err(e) => {
            error!("Failed to join room {}: {}", room_id, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
        }
    }
}

/// Start transcription for a meeting
pub async fn start_transcription(
    State(state): State<Arc<AppState>>,
    Path(room_id): Path<String>,
) -> impl IntoResponse {
    let transcription_service = Arc::new(DefaultTranscriptionService);
    let meeting_service = MeetingService::new(state.clone(), transcription_service);

    match meeting_service.start_transcription(&room_id).await {
        Ok(_) => {
            info!("Started transcription for room {}", room_id);
            (
                StatusCode::OK,
                Json(serde_json::json!({"status": "transcription_started"})),
            )
        }
        Err(e) => {
            error!("Failed to start transcription for room {}: {}", room_id, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
        }
    }
}

/// Get meeting token for WebRTC
pub async fn get_meeting_token(
    State(_state): State<Arc<AppState>>,
    Json(payload): Json<GetTokenRequest>,
) -> impl IntoResponse {
    // Generate a simple token (in production, use JWT or proper token service)
    let token = format!(
        "meet_token_{}_{}_{}",
        payload.room_id,
        payload.user_id,
        uuid::Uuid::new_v4()
    );

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "token": token,
            "room_id": payload.room_id,
            "user_id": payload.user_id
        })),
    )
}

/// Send meeting invites
pub async fn send_meeting_invites(
    State(_state): State<Arc<AppState>>,
    Json(payload): Json<SendInvitesRequest>,
) -> impl IntoResponse {
    info!("Sending meeting invites for room {}", payload.room_id);
    // In production, integrate with email service
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "invites_sent",
            "recipients": payload.emails
        })),
    )
}

/// WebSocket handler for real-time meeting communication
pub async fn meeting_websocket(
    ws: axum::extract::ws::WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_meeting_socket(socket, state))
}

async fn handle_meeting_socket(_socket: axum::extract::ws::WebSocket, _state: Arc<AppState>) {
    info!("Meeting WebSocket connection established");
    // Handle WebSocket messages for real-time meeting communication
    // This would integrate with WebRTC signaling
}

// Request/Response DTOs
#[derive(Debug, Deserialize)]
pub struct CreateMeetingRequest {
    pub name: String,
    pub created_by: String,
    pub settings: Option<service::MeetingSettings>,
}

#[derive(Debug, Deserialize)]
pub struct JoinRoomRequest {
    pub participant_name: String,
    pub participant_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetTokenRequest {
    pub room_id: String,
    pub user_id: String,
}

#[derive(Debug, Deserialize)]
pub struct SendInvitesRequest {
    pub room_id: String,
    pub emails: Vec<String>,
}

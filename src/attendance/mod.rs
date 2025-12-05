//! Attendance Module
//!
//! Provides attendance tracking and human handoff queue functionality.
//!
//! ## Features
//!
//! - **Queue System**: Human handoff for conversations that need human attention
//! - **Keyword Services**: Check-in/out, break/resume tracking via keywords
//! - **Drive Integration**: S3 storage for attendance records
//! - **WebSocket**: Real-time notifications for attendants
//! - **LLM Assist**: AI-powered tips, polishing, smart replies, summaries, and sentiment analysis
//!
//! ## LLM Assist Features (config.csv)
//!
//! ```csv
//! name,value
//! attendant-llm-tips,true
//! attendant-polish-message,true
//! attendant-smart-replies,true
//! attendant-auto-summary,true
//! attendant-sentiment-analysis,true
//! ```
//!
//! ## Usage
//!
//! Enable with the `attendance` feature flag in Cargo.toml:
//! ```toml
//! [features]
//! default = ["attendance"]
//! ```

pub mod drive;
pub mod keyword_services;
pub mod llm_assist;
pub mod queue;

// Re-export main types for convenience
pub use drive::{AttendanceDriveConfig, AttendanceDriveService, RecordMetadata, SyncResult};
pub use keyword_services::{
    AttendanceCommand, AttendanceRecord, AttendanceResponse, AttendanceService, KeywordConfig,
    KeywordParser, ParsedCommand,
};
pub use llm_assist::{
    AttendantTip, ConversationMessage, ConversationSummary, LlmAssistConfig, PolishRequest,
    PolishResponse, SentimentAnalysis, SentimentResponse, SmartRepliesRequest,
    SmartRepliesResponse, SmartReply, SummaryRequest, SummaryResponse, TipRequest, TipResponse,
    TipType,
};
pub use queue::{
    AssignRequest, AttendantStats, AttendantStatus, QueueFilters, QueueItem, QueueStatus,
    TransferRequest,
};

use crate::core::bot::channels::whatsapp::WhatsAppAdapter;
use crate::core::bot::channels::ChannelAdapter;
use crate::shared::models::{BotResponse, UserSession};
use crate::shared::state::{AppState, AttendantNotification};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, Query, State,
    },
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use diesel::prelude::*;
use futures::{SinkExt, StreamExt};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;
use uuid::Uuid;

/// Configure attendance routes
pub fn configure_attendance_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Queue management endpoints
        .route("/api/attendance/queue", get(queue::list_queue))
        .route("/api/attendance/attendants", get(queue::list_attendants))
        .route("/api/attendance/assign", post(queue::assign_conversation))
        .route(
            "/api/attendance/transfer",
            post(queue::transfer_conversation),
        )
        .route(
            "/api/attendance/resolve/{session_id}",
            post(queue::resolve_conversation),
        )
        .route("/api/attendance/insights", get(queue::get_insights))
        // Attendant response endpoint
        .route("/api/attendance/respond", post(attendant_respond))
        // WebSocket for real-time notifications
        .route("/ws/attendant", get(attendant_websocket_handler))
        // LLM Assist endpoints - AI-powered attendant assistance
        .route("/api/attendance/llm/tips", post(llm_assist::generate_tips))
        .route(
            "/api/attendance/llm/polish",
            post(llm_assist::polish_message),
        )
        .route(
            "/api/attendance/llm/smart-replies",
            post(llm_assist::generate_smart_replies),
        )
        .route(
            "/api/attendance/llm/summary/{session_id}",
            get(llm_assist::generate_summary),
        )
        .route(
            "/api/attendance/llm/sentiment",
            post(llm_assist::analyze_sentiment),
        )
        .route(
            "/api/attendance/llm/config/{bot_id}",
            get(llm_assist::get_llm_config),
        )
}

/// Attendant response request
#[derive(Debug, Deserialize)]
pub struct AttendantRespondRequest {
    pub session_id: String,
    pub message: String,
    pub attendant_id: String,
}

/// Attendant response result
#[derive(Debug, Serialize)]
pub struct AttendantRespondResponse {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Handle attendant response - routes back to the customer's channel
pub async fn attendant_respond(
    State(state): State<Arc<AppState>>,
    Json(request): Json<AttendantRespondRequest>,
) -> impl IntoResponse {
    info!(
        "Attendant {} responding to session {}",
        request.attendant_id, request.session_id
    );

    let session_id = match Uuid::parse_str(&request.session_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(AttendantRespondResponse {
                    success: false,
                    message: "Invalid session ID".to_string(),
                    error: Some("Could not parse session ID as UUID".to_string()),
                }),
            )
        }
    };

    // Get session details
    let conn = state.conn.clone();
    let session_result = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().ok()?;
        use crate::shared::models::schema::user_sessions;
        user_sessions::table
            .find(session_id)
            .first::<UserSession>(&mut db_conn)
            .ok()
    })
    .await
    .ok()
    .flatten();

    let session = match session_result {
        Some(s) => s,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(AttendantRespondResponse {
                    success: false,
                    message: "Session not found".to_string(),
                    error: Some("No session with that ID exists".to_string()),
                }),
            )
        }
    };

    // Get channel from session context
    let channel = session
        .context_data
        .get("channel")
        .and_then(|v| v.as_str())
        .unwrap_or("web");

    // Get recipient (phone number for WhatsApp, user_id for web)
    let recipient = session
        .context_data
        .get("phone")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // Save attendant message to history
    if let Err(e) = save_message_to_history(&state, &session, &request.message, "attendant").await {
        error!("Failed to save attendant message: {}", e);
    }

    // Send to appropriate channel
    match channel {
        "whatsapp" => {
            if recipient.is_empty() {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(AttendantRespondResponse {
                        success: false,
                        message: "No phone number found".to_string(),
                        error: Some("Session has no phone number in context".to_string()),
                    }),
                );
            }

            let adapter = WhatsAppAdapter::new(state.conn.clone(), session.bot_id);
            let response = BotResponse {
                bot_id: session.bot_id.to_string(),
                session_id: session.id.to_string(),
                user_id: recipient.to_string(),
                channel: "whatsapp".to_string(),
                content: request.message.clone(),
                message_type: crate::shared::models::message_types::MessageType::BOT_RESPONSE,
                stream_token: None,
                is_complete: true,
                suggestions: vec![],
                context_name: None,
            };

            match adapter.send_message(response).await {
                Ok(_) => {
                    // Notify other attendants about the response
                    broadcast_attendant_action(&state, &session, &request, "attendant_response")
                        .await;

                    (
                        StatusCode::OK,
                        Json(AttendantRespondResponse {
                            success: true,
                            message: "Response sent to WhatsApp".to_string(),
                            error: None,
                        }),
                    )
                }
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(AttendantRespondResponse {
                        success: false,
                        message: "Failed to send WhatsApp message".to_string(),
                        error: Some(e.to_string()),
                    }),
                ),
            }
        }
        "web" | _ => {
            // For web and other channels, send via WebSocket if connected
            let sent = if let Some(tx) = state
                .response_channels
                .lock()
                .await
                .get(&session.id.to_string())
            {
                let response = BotResponse {
                    bot_id: session.bot_id.to_string(),
                    session_id: session.id.to_string(),
                    user_id: session.user_id.to_string(),
                    channel: channel.to_string(),
                    content: request.message.clone(),
                    message_type: crate::shared::models::message_types::MessageType::BOT_RESPONSE,
                    stream_token: None,
                    is_complete: true,
                    suggestions: vec![],
                    context_name: None,
                };
                tx.send(response).await.is_ok()
            } else {
                false
            };

            // Notify other attendants
            broadcast_attendant_action(&state, &session, &request, "attendant_response").await;

            if sent {
                (
                    StatusCode::OK,
                    Json(AttendantRespondResponse {
                        success: true,
                        message: "Response sent via WebSocket".to_string(),
                        error: None,
                    }),
                )
            } else {
                // Message saved but couldn't be delivered in real-time
                (
                    StatusCode::OK,
                    Json(AttendantRespondResponse {
                        success: true,
                        message: "Response saved (customer not connected)".to_string(),
                        error: None,
                    }),
                )
            }
        }
    }
}

/// Save message to conversation history
async fn save_message_to_history(
    state: &Arc<AppState>,
    session: &UserSession,
    content: &str,
    sender: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let conn = state.conn.clone();
    let session_id = session.id;
    let content_clone = content.to_string();
    let sender_clone = sender.to_string();

    tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| format!("DB error: {}", e))?;

        use crate::shared::models::schema::message_history;

        diesel::insert_into(message_history::table)
            .values((
                message_history::id.eq(Uuid::new_v4()),
                message_history::session_id.eq(session_id),
                message_history::role.eq(sender_clone),
                message_history::content.eq(content_clone),
                message_history::created_at.eq(diesel::dsl::now),
            ))
            .execute(&mut db_conn)
            .map_err(|e| format!("Insert error: {}", e))?;

        Ok::<(), String>(())
    })
    .await
    .map_err(|e| format!("Task error: {}", e))??;

    Ok(())
}

/// Broadcast attendant action to other connected attendants
async fn broadcast_attendant_action(
    state: &Arc<AppState>,
    session: &UserSession,
    request: &AttendantRespondRequest,
    action_type: &str,
) {
    if let Some(broadcast_tx) = state.attendant_broadcast.as_ref() {
        let notification = AttendantNotification {
            notification_type: action_type.to_string(),
            session_id: session.id.to_string(),
            user_id: session.user_id.to_string(),
            user_name: session
                .context_data
                .get("name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            user_phone: session
                .context_data
                .get("phone")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            channel: session
                .context_data
                .get("channel")
                .and_then(|v| v.as_str())
                .unwrap_or("web")
                .to_string(),
            content: request.message.clone(),
            timestamp: Utc::now().to_rfc3339(),
            assigned_to: Some(request.attendant_id.clone()),
            priority: 0,
        };

        if let Err(e) = broadcast_tx.send(notification) {
            debug!("No attendants listening for broadcast: {}", e);
        }
    }
}

/// WebSocket handler for attendant real-time notifications
pub async fn attendant_websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let attendant_id = params.get("attendant_id").cloned();

    if attendant_id.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "attendant_id is required" })),
        )
            .into_response();
    }

    let attendant_id = attendant_id.unwrap();
    info!(
        "Attendant WebSocket connection request from: {}",
        attendant_id
    );

    ws.on_upgrade(move |socket| handle_attendant_websocket(socket, state, attendant_id))
        .into_response()
}

/// Handle attendant WebSocket connection
async fn handle_attendant_websocket(socket: WebSocket, state: Arc<AppState>, attendant_id: String) {
    let (mut sender, mut receiver) = socket.split();

    info!("Attendant WebSocket connected: {}", attendant_id);

    // Send welcome message
    let welcome = serde_json::json!({
        "type": "connected",
        "attendant_id": attendant_id,
        "message": "Connected to attendant notification service",
        "timestamp": Utc::now().to_rfc3339()
    });

    if let Ok(welcome_str) = serde_json::to_string(&welcome) {
        if sender
            .send(Message::Text(welcome_str.into()))
            .await
            .is_err()
        {
            error!("Failed to send welcome message to attendant");
            return;
        }
    }

    // Subscribe to broadcast channel
    let mut broadcast_rx = if let Some(broadcast_tx) = state.attendant_broadcast.as_ref() {
        broadcast_tx.subscribe()
    } else {
        warn!("No broadcast channel available for attendants");
        return;
    };

    // Task to forward broadcast messages to WebSocket
    let attendant_id_clone = attendant_id.clone();
    let mut send_task = tokio::spawn(async move {
        loop {
            match broadcast_rx.recv().await {
                Ok(notification) => {
                    // Check if this notification is relevant to this attendant
                    // Send all notifications for now (can filter by assigned_to later)
                    let should_send = notification.assigned_to.is_none()
                        || notification.assigned_to.as_ref() == Some(&attendant_id_clone);

                    if should_send {
                        if let Ok(json_str) = serde_json::to_string(&notification) {
                            debug!(
                                "Sending notification to attendant {}: {}",
                                attendant_id_clone, notification.notification_type
                            );
                            if sender.send(Message::Text(json_str.into())).await.is_err() {
                                error!("Failed to send notification to attendant WebSocket");
                                break;
                            }
                        }
                    }
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    warn!(
                        "Attendant {} lagged behind by {} messages",
                        attendant_id_clone, n
                    );
                }
                Err(broadcast::error::RecvError::Closed) => {
                    info!("Broadcast channel closed");
                    break;
                }
            }
        }
    });

    // Task to handle incoming messages from attendant (e.g., status updates, typing indicators)
    let state_clone = state.clone();
    let attendant_id_for_recv = attendant_id.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    debug!(
                        "Received message from attendant {}: {}",
                        attendant_id_for_recv, text
                    );

                    // Parse and handle attendant messages
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                        handle_attendant_message(&state_clone, &attendant_id_for_recv, parsed)
                            .await;
                    }
                }
                Message::Ping(data) => {
                    debug!("Received ping from attendant {}", attendant_id_for_recv);
                    // Pong is automatically sent by axum
                }
                Message::Close(_) => {
                    info!(
                        "Attendant {} WebSocket close requested",
                        attendant_id_for_recv
                    );
                    break;
                }
                _ => {}
            }
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = (&mut send_task) => {
            recv_task.abort();
        }
        _ = (&mut recv_task) => {
            send_task.abort();
        }
    }

    info!("Attendant WebSocket disconnected: {}", attendant_id);
}

/// Handle incoming messages from attendant WebSocket
async fn handle_attendant_message(
    state: &Arc<AppState>,
    attendant_id: &str,
    message: serde_json::Value,
) {
    let msg_type = message
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    match msg_type {
        "status_update" => {
            // Update attendant status (online, busy, away, offline)
            if let Some(status) = message.get("status").and_then(|v| v.as_str()) {
                info!("Attendant {} status update: {}", attendant_id, status);
                // Could update in database or broadcast to other attendants
            }
        }
        "typing" => {
            // Broadcast typing indicator to customer
            if let Some(session_id) = message.get("session_id").and_then(|v| v.as_str()) {
                debug!(
                    "Attendant {} typing in session {}",
                    attendant_id, session_id
                );
                // Could broadcast to customer's WebSocket
            }
        }
        "read" => {
            // Mark messages as read
            if let Some(session_id) = message.get("session_id").and_then(|v| v.as_str()) {
                debug!(
                    "Attendant {} marked session {} as read",
                    attendant_id, session_id
                );
            }
        }
        "respond" => {
            // Handle response message (alternative to REST API)
            if let (Some(session_id), Some(content)) = (
                message.get("session_id").and_then(|v| v.as_str()),
                message.get("content").and_then(|v| v.as_str()),
            ) {
                info!(
                    "Attendant {} responding to {} via WebSocket",
                    attendant_id, session_id
                );

                // Process response similar to REST endpoint
                let request = AttendantRespondRequest {
                    session_id: session_id.to_string(),
                    message: content.to_string(),
                    attendant_id: attendant_id.to_string(),
                };

                // Get session and send response
                if let Ok(uuid) = Uuid::parse_str(session_id) {
                    let conn = state.conn.clone();
                    if let Some(session) = tokio::task::spawn_blocking(move || {
                        let mut db_conn = conn.get().ok()?;
                        use crate::shared::models::schema::user_sessions;
                        user_sessions::table
                            .find(uuid)
                            .first::<UserSession>(&mut db_conn)
                            .ok()
                    })
                    .await
                    .ok()
                    .flatten()
                    {
                        // Save to history
                        let _ =
                            save_message_to_history(state, &session, content, "attendant").await;

                        // Send to channel
                        let channel = session
                            .context_data
                            .get("channel")
                            .and_then(|v| v.as_str())
                            .unwrap_or("web");

                        if channel == "whatsapp" {
                            if let Some(phone) =
                                session.context_data.get("phone").and_then(|v| v.as_str())
                            {
                                let adapter =
                                    WhatsAppAdapter::new(state.conn.clone(), session.bot_id);
                                let response = BotResponse {
                                    bot_id: session.bot_id.to_string(),
                                    session_id: session.id.to_string(),
                                    user_id: phone.to_string(),
                                    channel: "whatsapp".to_string(),
                                    content: content.to_string(),
                                    message_type:
                                        crate::shared::models::message_types::MessageType::BOT_RESPONSE,
                                    stream_token: None,
                                    is_complete: true,
                                    suggestions: vec![],
                                    context_name: None,
                                };
                                let _ = adapter.send_message(response).await;
                            }
                        }

                        // Broadcast to other attendants
                        broadcast_attendant_action(state, &session, &request, "attendant_response")
                            .await;
                    }
                }
            }
        }
        _ => {
            debug!(
                "Unknown message type from attendant {}: {}",
                attendant_id, msg_type
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports() {
        // Test that types are properly exported
        let _config = KeywordConfig::default();
        let _parser = KeywordParser::new();
    }

    #[test]
    fn test_respond_request_parse() {
        let json = r#"{
            "session_id": "123e4567-e89b-12d3-a456-426614174000",
            "message": "Hello, how can I help?",
            "attendant_id": "att-001"
        }"#;

        let request: AttendantRespondRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.attendant_id, "att-001");
        assert_eq!(request.message, "Hello, how can I help?");
    }
}

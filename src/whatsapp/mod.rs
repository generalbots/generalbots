//! WhatsApp Integration Module
//!
//! Handles incoming WhatsApp webhooks and routes messages to bot or human attendants.
//! Supports the full message flow:
//! 1. WhatsApp Cloud API webhook verification
//! 2. Incoming message processing
//! 3. Bot response or human handoff
//! 4. Attendant response routing back to WhatsApp
//!
//! ## Configuration
//!
//! Add to your bot's config.csv:
//! ```csv
//! whatsapp-api-key,your_access_token
//! whatsapp-phone-number-id,your_phone_number_id
//! whatsapp-verify-token,your_webhook_verify_token
//! whatsapp-business-account-id,your_business_account_id
//! ```

use crate::bot::BotOrchestrator;
use crate::core::bot::channels::whatsapp::WhatsAppAdapter;
use crate::core::bot::channels::ChannelAdapter;
use crate::shared::models::{BotResponse, UserMessage, UserSession};
use crate::shared::state::{AppState, AttendantNotification};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use botlib::MessageType;
use chrono::Utc;
use diesel::prelude::*;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;
use uuid::Uuid;

/// WebSocket broadcast channel for attendant notifications
pub type AttendantBroadcast = broadcast::Sender<AttendantNotification>;

/// WhatsApp webhook verification query parameters
#[derive(Debug, Deserialize)]
pub struct WebhookVerifyQuery {
    #[serde(rename = "hub.mode")]
    pub mode: Option<String>,
    #[serde(rename = "hub.verify_token")]
    pub verify_token: Option<String>,
    #[serde(rename = "hub.challenge")]
    pub challenge: Option<String>,
}

/// WhatsApp webhook payload
#[derive(Debug, Deserialize)]
pub struct WhatsAppWebhook {
    pub object: String,
    #[serde(default)]
    pub entry: Vec<WhatsAppEntry>,
}

#[derive(Debug, Deserialize)]
pub struct WhatsAppEntry {
    pub id: String,
    #[serde(default)]
    pub changes: Vec<WhatsAppChange>,
}

#[derive(Debug, Deserialize)]
pub struct WhatsAppChange {
    pub field: String,
    pub value: WhatsAppValue,
}

#[derive(Debug, Deserialize)]
pub struct WhatsAppValue {
    pub messaging_product: String,
    #[serde(default)]
    pub metadata: WhatsAppMetadata,
    #[serde(default)]
    pub contacts: Vec<WhatsAppContact>,
    #[serde(default)]
    pub messages: Vec<WhatsAppMessage>,
    #[serde(default)]
    pub statuses: Vec<WhatsAppStatus>,
}

#[derive(Debug, Default, Deserialize)]
pub struct WhatsAppMetadata {
    pub display_phone_number: Option<String>,
    pub phone_number_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct WhatsAppContact {
    pub wa_id: String,
    pub profile: WhatsAppProfile,
}

#[derive(Debug, Deserialize)]
pub struct WhatsAppProfile {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct WhatsAppMessage {
    pub id: String,
    pub from: String,
    pub timestamp: String,
    #[serde(rename = "type")]
    pub message_type: String,
    #[serde(default)]
    pub text: Option<WhatsAppText>,
    #[serde(default)]
    pub image: Option<WhatsAppMedia>,
    #[serde(default)]
    pub audio: Option<WhatsAppMedia>,
    #[serde(default)]
    pub video: Option<WhatsAppMedia>,
    #[serde(default)]
    pub document: Option<WhatsAppMedia>,
    #[serde(default)]
    pub location: Option<WhatsAppLocation>,
    #[serde(default)]
    pub interactive: Option<WhatsAppInteractive>,
    #[serde(default)]
    pub button: Option<WhatsAppButton>,
}

#[derive(Debug, Deserialize)]
pub struct WhatsAppText {
    pub body: String,
}

#[derive(Debug, Deserialize)]
pub struct WhatsAppMedia {
    pub id: String,
    #[serde(default)]
    pub mime_type: Option<String>,
    #[serde(default)]
    pub sha256: Option<String>,
    #[serde(default)]
    pub caption: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct WhatsAppLocation {
    pub latitude: f64,
    pub longitude: f64,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub address: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct WhatsAppInteractive {
    #[serde(rename = "type")]
    pub interactive_type: String,
    #[serde(default)]
    pub button_reply: Option<WhatsAppButtonReply>,
    #[serde(default)]
    pub list_reply: Option<WhatsAppListReply>,
}

#[derive(Debug, Deserialize)]
pub struct WhatsAppButtonReply {
    pub id: String,
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub struct WhatsAppListReply {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct WhatsAppButton {
    pub payload: String,
    pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct WhatsAppStatus {
    pub id: String,
    pub status: String,
    pub timestamp: String,
    pub recipient_id: String,
}

/// Configure WhatsApp routes
pub fn configure() -> Router<Arc<AppState>> {
    Router::new()
        .route("/webhook/whatsapp", get(verify_webhook))
        .route("/webhook/whatsapp", post(handle_webhook))
        .route("/api/whatsapp/send", post(send_message))
        .route("/api/attendance/respond", post(attendant_respond))
}

/// Verify WhatsApp webhook (GET request)
pub async fn verify_webhook(
    State(state): State<Arc<AppState>>,
    Query(params): Query<WebhookVerifyQuery>,
) -> impl IntoResponse {
    info!("WhatsApp webhook verification request received");

    let mode = params.mode.unwrap_or_default();
    let token = params.verify_token.unwrap_or_default();
    let challenge = params.challenge.unwrap_or_default();

    if mode != "subscribe" {
        warn!("Invalid webhook mode: {}", mode);
        return (StatusCode::FORBIDDEN, "Invalid mode".to_string());
    }

    // Get verify token from config
    let expected_token = get_verify_token(&state).await;

    if token == expected_token {
        info!("Webhook verification successful");
        (StatusCode::OK, challenge)
    } else {
        warn!("Invalid verify token");
        (StatusCode::FORBIDDEN, "Invalid verify token".to_string())
    }
}

/// Handle incoming WhatsApp webhook (POST request)
pub async fn handle_webhook(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<WhatsAppWebhook>,
) -> impl IntoResponse {
    info!("WhatsApp webhook received: {:?}", payload.object);

    if payload.object != "whatsapp_business_account" {
        return StatusCode::OK;
    }

    for entry in payload.entry {
        for change in entry.changes {
            if change.field == "messages" {
                // Get contact info
                let contact = change.value.contacts.first();
                let contact_name = contact.map(|c| c.profile.name.clone());
                let contact_phone = contact.map(|c| c.wa_id.clone());

                // Process messages
                for message in change.value.messages {
                    if let Err(e) = process_incoming_message(
                        state.clone(),
                        &message,
                        contact_name.clone(),
                        contact_phone.clone(),
                    )
                    .await
                    {
                        error!("Failed to process WhatsApp message: {}", e);
                    }
                }

                // Process status updates
                for status in change.value.statuses {
                    debug!(
                        "Message {} status: {} for {}",
                        status.id, status.status, status.recipient_id
                    );
                }
            }
        }
    }

    StatusCode::OK
}

/// Process an incoming WhatsApp message
async fn process_incoming_message(
    state: Arc<AppState>,
    message: &WhatsAppMessage,
    contact_name: Option<String>,
    contact_phone: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let phone = contact_phone
        .clone()
        .unwrap_or_else(|| message.from.clone());
    let name = contact_name.clone().unwrap_or_else(|| phone.clone());

    info!(
        "Processing WhatsApp message from {} ({}): type={}",
        name, phone, message.message_type
    );

    // Extract message content
    let content = extract_message_content(message);
    if content.is_empty() {
        debug!("Empty message content, skipping");
        return Ok(());
    }

    // Check if this is an attendant command (starts with /)
    if content.starts_with('/') {
        if let Some(response) = process_attendant_command(&state, &phone, &content).await {
            // Send response back to the attendant
            let adapter = WhatsAppAdapter::new(state.conn.clone(), Uuid::nil());
            let bot_response = BotResponse {
                bot_id: Uuid::nil().to_string(),
                session_id: Uuid::nil().to_string(),
                user_id: phone.clone(),
                channel: "whatsapp".to_string(),
                content: response,
                message_type: MessageType::BOT_RESPONSE,
                stream_token: None,
                is_complete: true,
                suggestions: vec![],
                context_name: None,
                context_length: 0,
                context_max_length: 0,
            };
            if let Err(e) = adapter.send_message(bot_response).await {
                error!("Failed to send attendant command response: {}", e);
            }
            return Ok(());
        }
    }

    // Find or create session for this user
    let (session, is_new) = find_or_create_session(&state, &phone, &name).await?;

    // Check if session needs human attention (transferred to human)
    let needs_human = check_needs_human(&session);

    if needs_human {
        // Route to human attendant
        route_to_attendant(&state, &session, &content, &name, &phone).await?;
    } else {
        // Route to bot
        route_to_bot(&state, &session, &content, is_new).await?;
    }

    Ok(())
}

/// Process attendant commands from WhatsApp
/// Returns Some(response) if this is an attendant command, None otherwise
async fn process_attendant_command(
    state: &Arc<AppState>,
    phone: &str,
    content: &str,
) -> Option<String> {
    // Check if this phone number belongs to an attendant
    let is_attendant = check_is_attendant(state, phone).await;

    if !is_attendant {
        return None;
    }

    // Get current session the attendant is handling (if any)
    let current_session = get_attendant_active_session(state, phone).await;

    // Process the command using llm_assist module (only if attendance feature is enabled)
    #[cfg(feature = "attendance")]
    {
        match crate::attendance::llm_assist::process_attendant_command(
            state,
            phone,
            content,
            current_session,
        )
        .await
        {
            Ok(response) => return Some(response),
            Err(e) => return Some(format!("❌ Error: {}", e)),
        }
    }

    #[cfg(not(feature = "attendance"))]
    {
        let _ = current_session; // Suppress unused warning
        Some(format!(
            "Attendance module not enabled. Message: {}",
            content
        ))
    }
}

/// Check if a phone number belongs to a registered attendant
async fn check_is_attendant(state: &Arc<AppState>, phone: &str) -> bool {
    let conn = state.conn.clone();
    let phone_clone = phone.to_string();

    tokio::task::spawn_blocking(move || {
        // Try to find attendant by phone in bot_configuration or a dedicated table
        // For now, check if phone is in attendant.csv context
        let work_path = std::env::var("WORK_PATH").unwrap_or_else(|_| "./work".to_string());

        // Read attendant.csv files from all bots
        if let Ok(entries) = std::fs::read_dir(&work_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() && path.to_string_lossy().ends_with(".gbai") {
                    let attendant_path = path.join("attendant.csv");
                    if attendant_path.exists() {
                        if let Ok(content) = std::fs::read_to_string(&attendant_path) {
                            for line in content.lines().skip(1) {
                                // Check if phone is in this line (could be in channel or preferences)
                                if line.to_lowercase().contains(&phone_clone.to_lowercase()) {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }
        false
    })
    .await
    .unwrap_or(false)
}

/// Get the active session an attendant is currently handling
async fn get_attendant_active_session(state: &Arc<AppState>, phone: &str) -> Option<Uuid> {
    let conn = state.conn.clone();
    let phone_clone = phone.to_string();

    tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().ok()?;

        use crate::shared::models::schema::user_sessions;

        // Find session assigned to this attendant phone
        let session: Option<UserSession> = user_sessions::table
            .filter(
                user_sessions::context_data
                    .retrieve_as_text("assigned_to_phone")
                    .eq(&phone_clone),
            )
            .filter(
                user_sessions::context_data
                    .retrieve_as_text("status")
                    .ne("resolved"),
            )
            .order(user_sessions::updated_at.desc())
            .first(&mut db_conn)
            .ok();

        session.map(|s| s.id)
    })
    .await
    .ok()
    .flatten()
}

/// Extract text content from different message types
fn extract_message_content(message: &WhatsAppMessage) -> String {
    match message.message_type.as_str() {
        "text" => message
            .text
            .as_ref()
            .map(|t| t.body.clone())
            .unwrap_or_default(),
        "interactive" => {
            if let Some(interactive) = &message.interactive {
                match interactive.interactive_type.as_str() {
                    "button_reply" => interactive
                        .button_reply
                        .as_ref()
                        .map(|b| b.title.clone())
                        .unwrap_or_default(),
                    "list_reply" => interactive
                        .list_reply
                        .as_ref()
                        .map(|l| l.title.clone())
                        .unwrap_or_default(),
                    _ => String::new(),
                }
            } else {
                String::new()
            }
        }
        "button" => message
            .button
            .as_ref()
            .map(|b| b.text.clone())
            .unwrap_or_default(),
        "image" | "audio" | "video" | "document" => {
            format!("[{} message]", message.message_type)
        }
        "location" => {
            if let Some(loc) = &message.location {
                format!(
                    "📍 Location: {}, {} ({})",
                    loc.latitude,
                    loc.longitude,
                    loc.name.as_deref().unwrap_or("Unknown")
                )
            } else {
                "[Location]".to_string()
            }
        }
        _ => String::new(),
    }
}

/// Find existing session or create new one for WhatsApp user
async fn find_or_create_session(
    state: &Arc<AppState>,
    phone: &str,
    name: &str,
) -> Result<(UserSession, bool), Box<dyn std::error::Error + Send + Sync>> {
    let conn = state.conn.clone();
    let phone_clone = phone.to_string();
    let name_clone = name.to_string();

    let result = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| format!("DB error: {}", e))?;

        use crate::shared::models::schema::{bots, user_sessions, users};

        // Find user by phone (stored in email field or context)
        let existing_user: Option<(Uuid, String)> = users::table
            .filter(users::email.eq(&phone_clone))
            .select((users::id, users::username))
            .first(&mut db_conn)
            .optional()
            .map_err(|e| format!("Query error: {}", e))?;

        let (user_id, _username) = if let Some((id, uname)) = existing_user {
            (id, uname)
        } else {
            // Create new user
            let new_user_id = Uuid::new_v4();
            diesel::insert_into(users::table)
                .values((
                    users::id.eq(new_user_id),
                    users::username.eq(&name_clone),
                    users::email.eq(&phone_clone),
                    users::password_hash.eq("whatsapp_user"),
                    users::created_at.eq(diesel::dsl::now),
                ))
                .execute(&mut db_conn)
                .map_err(|e| format!("Insert user error: {}", e))?;
            (new_user_id, name_clone.clone())
        };

        // Get default bot
        let bot_id: Uuid = bots::table
            .filter(bots::is_active.eq(true))
            .select(bots::id)
            .first(&mut db_conn)
            .map_err(|e| format!("No active bot found: {}", e))?;

        // Find active session for this user
        let existing_session: Option<UserSession> = user_sessions::table
            .filter(user_sessions::user_id.eq(user_id))
            .filter(user_sessions::bot_id.eq(bot_id))
            .order(user_sessions::created_at.desc())
            .first(&mut db_conn)
            .optional()
            .map_err(|e| format!("Session query error: {}", e))?;

        if let Some(session) = existing_session {
            // Check if session is recent (within 24 hours)
            let age = Utc::now() - session.updated_at;
            if age.num_hours() < 24 {
                return Ok::<(UserSession, bool), String>((session, false));
            }
        }

        // Create new session
        let new_session_id = Uuid::new_v4();
        let context_data = serde_json::json!({
            "channel": "whatsapp",
            "phone": phone_clone,
            "name": name_clone,
        });

        diesel::insert_into(user_sessions::table)
            .values((
                user_sessions::id.eq(new_session_id),
                user_sessions::user_id.eq(user_id),
                user_sessions::bot_id.eq(bot_id),
                user_sessions::context_data.eq(&context_data),
                user_sessions::created_at.eq(diesel::dsl::now),
                user_sessions::updated_at.eq(diesel::dsl::now),
            ))
            .execute(&mut db_conn)
            .map_err(|e| format!("Create session error: {}", e))?;

        let new_session: UserSession = user_sessions::table
            .find(new_session_id)
            .first(&mut db_conn)
            .map_err(|e| format!("Load session error: {}", e))?;

        Ok::<(UserSession, bool), String>((new_session, true))
    })
    .await
    .map_err(|e| format!("Task error: {}", e))??;

    Ok(result)
}

/// Check if session needs human attention
fn check_needs_human(session: &UserSession) -> bool {
    if let Some(needs_human) = session.context_data.get("needs_human") {
        return needs_human.as_bool().unwrap_or(false);
    }
    false
}

/// Route message to bot for processing
async fn route_to_bot(
    state: &Arc<AppState>,
    session: &UserSession,
    content: &str,
    is_new: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("Routing WhatsApp message to bot for session {}", session.id);

    let user_message = UserMessage {
        bot_id: session.bot_id.to_string(),
        user_id: session.user_id.to_string(),
        session_id: session.id.to_string(),
        channel: "whatsapp".to_string(),
        content: content.to_string(),
        message_type: MessageType::USER,
        media_url: None,
        timestamp: Utc::now(),
        context_name: None,
    };

    // Get WhatsApp adapter for sending responses
    let adapter = WhatsAppAdapter::new(state.conn.clone(), session.bot_id);

    // Process through bot orchestrator
    let orchestrator = BotOrchestrator::new(state.clone());

    // Create response channel
    let (tx, mut rx) = tokio::sync::mpsc::channel::<BotResponse>(100);

    // Spawn task to collect responses
    let phone = session
        .context_data
        .get("phone")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let phone_for_error = phone.clone(); // Clone for use in error handling after move
    let adapter_for_send = WhatsAppAdapter::new(state.conn.clone(), session.bot_id);

    tokio::spawn(async move {
        while let Some(response) = rx.recv().await {
            if !response.content.is_empty() {
                // Send response to WhatsApp
                let mut wa_response = response.clone();
                wa_response.user_id = phone.clone();
                wa_response.channel = "whatsapp".to_string();

                if let Err(e) = adapter_for_send.send_message(wa_response).await {
                    error!("Failed to send WhatsApp response: {}", e);
                }
            }
        }
    });

    // Process message using stream_response
    if let Err(e) = orchestrator.stream_response(user_message, tx).await {
        error!("Bot processing error: {}", e);

        // Send error message back
        let error_response = BotResponse {
            bot_id: session.bot_id.to_string(),
            session_id: session.id.to_string(),
            user_id: phone_for_error.clone(),
            channel: "whatsapp".to_string(),
            content: "Sorry, I encountered an error processing your message. Please try again."
                .to_string(),
            message_type: MessageType::BOT_RESPONSE,
            stream_token: None,
            is_complete: true,
            suggestions: vec![],
            context_name: None,
            context_length: 0,
            context_max_length: 0,
        };

        if let Err(e) = adapter.send_message(error_response).await {
            error!("Failed to send error response: {}", e);
        }
    }

    Ok(())
}

/// Route message to human attendant
async fn route_to_attendant(
    state: &Arc<AppState>,
    session: &UserSession,
    content: &str,
    user_name: &str,
    user_phone: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!(
        "Routing WhatsApp message to attendant for session {}",
        session.id
    );

    // Get assigned attendant info
    let assigned_to = session
        .context_data
        .get("assigned_to")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let priority = session
        .context_data
        .get("transfer_priority")
        .and_then(|v| v.as_i64())
        .unwrap_or(1) as i32;

    // Save message to history
    save_message_to_history(state, session, content, "customer").await?;

    // Create notification for attendants
    let notification = AttendantNotification {
        notification_type: "new_message".to_string(),
        session_id: session.id.to_string(),
        user_id: session.user_id.to_string(),
        user_name: Some(user_name.to_string()),
        user_phone: Some(user_phone.to_string()),
        channel: "whatsapp".to_string(),
        content: content.to_string(),
        timestamp: Utc::now().to_rfc3339(),
        assigned_to,
        priority,
    };

    // Broadcast to attendant WebSocket connections
    if let Some(broadcast_tx) = state.attendant_broadcast.as_ref() {
        if let Err(e) = broadcast_tx.send(notification.clone()) {
            debug!("No attendants listening: {}", e);
        } else {
            info!("Notification sent to attendants");
        }
    }

    // Also update queue status
    update_queue_item(state, session, content).await?;

    Ok(())
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
    let user_id = session.user_id; // Get the actual user_id from the session
    let content_clone = content.to_string();
    let sender_clone = sender.to_string();

    tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| format!("DB error: {}", e))?;

        use crate::shared::models::schema::message_history;

        diesel::insert_into(message_history::table)
            .values((
                message_history::id.eq(Uuid::new_v4()),
                message_history::session_id.eq(session_id),
                message_history::user_id.eq(user_id), // User associated with the message (has mobile field)
                message_history::role.eq(if sender_clone == "user" { 1 } else { 2 }),
                message_history::content_encrypted.eq(content_clone),
                message_history::message_type.eq(1),
                message_history::message_index.eq(0i64),
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

/// Update queue item with latest message
async fn update_queue_item(
    state: &Arc<AppState>,
    session: &UserSession,
    last_message: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let conn = state.conn.clone();
    let session_id = session.id;
    let last_msg = last_message.to_string();

    tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| format!("DB error: {}", e))?;

        use crate::shared::models::schema::user_sessions;

        // Update session's context_data with last message
        let current: UserSession = user_sessions::table
            .find(session_id)
            .first(&mut db_conn)
            .map_err(|e| format!("Find error: {}", e))?;

        let mut ctx = current.context_data.clone();
        ctx["last_message"] = serde_json::json!(last_msg);
        ctx["last_message_time"] = serde_json::json!(Utc::now().to_rfc3339());

        diesel::update(user_sessions::table.filter(user_sessions::id.eq(session_id)))
            .set((
                user_sessions::context_data.eq(&ctx),
                user_sessions::updated_at.eq(diesel::dsl::now),
            ))
            .execute(&mut db_conn)
            .map_err(|e| format!("Update error: {}", e))?;

        Ok::<(), String>(())
    })
    .await
    .map_err(|e| format!("Task error: {}", e))??;

    Ok(())
}

/// Send request body
#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub to: String,
    pub message: String,
    #[serde(default)]
    pub template: Option<String>,
}

/// Send a message to WhatsApp
pub async fn send_message(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SendMessageRequest>,
) -> impl IntoResponse {
    info!("Sending WhatsApp message to {}", request.to);

    // Get default bot for adapter config
    let bot_id = get_default_bot_id(&state).await;
    let adapter = WhatsAppAdapter::new(state.conn.clone(), bot_id);

    let response = BotResponse {
        bot_id: bot_id.to_string(),
        session_id: Uuid::new_v4().to_string(),
        user_id: request.to.clone(),
        channel: "whatsapp".to_string(),
        content: request.message.clone(),
        message_type: MessageType::EXTERNAL,
        stream_token: None,
        is_complete: true,
        suggestions: vec![],
        context_name: None,
        context_length: 0,
        context_max_length: 0,
    };

    match adapter.send_message(response).await {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "message": "Message sent"
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "success": false,
                "error": e.to_string()
            })),
        ),
    }
}

/// Attendant response request
#[derive(Debug, Deserialize)]
pub struct AttendantRespondRequest {
    pub session_id: String,
    pub message: String,
    pub attendant_id: String,
}

/// Handle attendant response - routes back to WhatsApp
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
                Json(serde_json::json!({
                    "success": false,
                    "error": "Invalid session ID"
                })),
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
                Json(serde_json::json!({
                    "success": false,
                    "error": "Session not found"
                })),
            )
        }
    };

    // Get channel from session
    let channel = session
        .context_data
        .get("channel")
        .and_then(|v| v.as_str())
        .unwrap_or("web");

    // Get recipient
    let recipient = session
        .context_data
        .get("phone")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if recipient.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "success": false,
                "error": "No recipient found for session"
            })),
        );
    }

    // Save attendant message to history
    if let Err(e) = save_message_to_history(&state, &session, &request.message, "attendant").await {
        error!("Failed to save attendant message: {}", e);
    }

    // Send to appropriate channel
    match channel {
        "whatsapp" => {
            let adapter = WhatsAppAdapter::new(state.conn.clone(), session.bot_id);
            let response = BotResponse {
                bot_id: session.bot_id.to_string(),
                session_id: session.id.to_string(),
                user_id: recipient.to_string(),
                channel: "whatsapp".to_string(),
                content: request.message.clone(),
                message_type: MessageType::BOT_RESPONSE,
                stream_token: None,
                is_complete: true,
                suggestions: vec![],
                context_name: None,
                context_length: 0,
                context_max_length: 0,
            };

            match adapter.send_message(response).await {
                Ok(_) => (
                    StatusCode::OK,
                    Json(serde_json::json!({
                        "success": true,
                        "message": "Response sent to WhatsApp"
                    })),
                ),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "success": false,
                        "error": e.to_string()
                    })),
                ),
            }
        }
        _ => {
            // For web and other channels, broadcast via WebSocket
            if let Some(broadcast_tx) = state.attendant_broadcast.as_ref() {
                let notification = AttendantNotification {
                    notification_type: "attendant_response".to_string(),
                    session_id: session.id.to_string(),
                    user_id: session.user_id.to_string(),
                    user_name: None,
                    user_phone: None,
                    channel: channel.to_string(),
                    content: request.message.clone(),
                    timestamp: Utc::now().to_rfc3339(),
                    assigned_to: Some(request.attendant_id.clone()),
                    priority: 0,
                };

                let _ = broadcast_tx.send(notification);
            }

            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "success": true,
                    "message": "Response sent"
                })),
            )
        }
    }
}

/// Get verify token from config (from Vault)
async fn get_verify_token(_state: &Arc<AppState>) -> String {
    // Get verify token from Vault - stored at gbo/whatsapp
    use crate::core::secrets::SecretsManager;

    match SecretsManager::from_env() {
        Ok(secrets) => {
            match secrets.get_value("gbo/whatsapp", "verify_token").await {
                Ok(token) => token,
                Err(_) => "webhook_verify".to_string(), // Default for initial setup
            }
        }
        Err(_) => "webhook_verify".to_string(), // Default if Vault not configured
    }
}

/// Get default bot ID
async fn get_default_bot_id(state: &Arc<AppState>) -> Uuid {
    let conn = state.conn.clone();

    tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().ok()?;
        use crate::shared::models::schema::bots;
        bots::table
            .filter(bots::is_active.eq(true))
            .select(bots::id)
            .first::<Uuid>(&mut db_conn)
            .ok()
    })
    .await
    .ok()
    .flatten()
    .unwrap_or_else(Uuid::nil)
}

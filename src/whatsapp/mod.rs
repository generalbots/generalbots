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
use std::sync::Arc;
use tokio::sync::broadcast;
use uuid::Uuid;

pub type AttendantBroadcast = broadcast::Sender<AttendantNotification>;

#[derive(Debug, Deserialize)]
pub struct WebhookVerifyQuery {
    #[serde(rename = "hub.mode")]
    pub mode: Option<String>,
    #[serde(rename = "hub.verify_token")]
    pub verify_token: Option<String>,
    #[serde(rename = "hub.challenge")]
    pub challenge: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WhatsAppWebhook {
    pub object: String,
    #[serde(default)]
    pub entry: Vec<WhatsAppEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WhatsAppEntry {
    pub id: String,
    #[serde(default)]
    pub changes: Vec<WhatsAppChange>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WhatsAppChange {
    pub field: String,
    pub value: WhatsAppValue,
}

#[derive(Debug, Deserialize, Serialize)]
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

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct WhatsAppMetadata {
    pub display_phone_number: Option<String>,
    pub phone_number_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WhatsAppContact {
    pub wa_id: String,
    pub profile: WhatsAppProfile,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WhatsAppProfile {
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
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

#[derive(Debug, Deserialize, Serialize)]
pub struct WhatsAppText {
    pub body: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WhatsAppMedia {
    pub id: String,
    #[serde(default)]
    pub mime_type: Option<String>,
    #[serde(default)]
    pub sha256: Option<String>,
    #[serde(default)]
    pub caption: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WhatsAppLocation {
    pub latitude: f64,
    pub longitude: f64,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub address: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WhatsAppInteractive {
    #[serde(rename = "type")]
    pub interactive_type: String,
    #[serde(default)]
    pub button_reply: Option<WhatsAppButtonReply>,
    #[serde(default)]
    pub list_reply: Option<WhatsAppListReply>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WhatsAppButtonReply {
    pub id: String,
    pub title: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WhatsAppListReply {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WhatsAppButton {
    pub payload: String,
    pub text: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WhatsAppStatus {
    pub id: String,
    pub status: String,
    pub timestamp: String,
    pub recipient_id: String,
}

pub fn configure() -> Router<Arc<AppState>> {
    Router::new()
        .route("/webhook/whatsapp", get(verify_webhook))
        .route("/webhook/whatsapp", post(handle_webhook))
        .route("/api/whatsapp/send", post(send_message))
        .route("/api/attendance/respond", post(attendant_respond))
}

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

    let expected_token = get_verify_token(&state).await;

    if token == expected_token {
        info!("Webhook verification successful");
        (StatusCode::OK, challenge)
    } else {
        warn!("Invalid verify token");
        (StatusCode::FORBIDDEN, "Invalid verify token".to_string())
    }
}

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
                let contact = change.value.contacts.first();
                let contact_name = contact.map(|c| c.profile.name.clone());
                let contact_phone = contact.map(|c| c.wa_id.clone());

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

    let content = extract_message_content(message);
    if content.is_empty() {
        debug!("Empty message content, skipping");
        return Ok(());
    }

    if content.starts_with('/') {
        if let Some(response) = process_attendant_command(&state, &phone, &content).await {
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

    let (session, is_new) = find_or_create_session(&state, &phone, &name).await?;

    let needs_human = check_needs_human(&session);

    if needs_human {
        route_to_attendant(&state, &session, &content, &name, &phone).await?;
    } else {
        route_to_bot(&state, &session, &content, is_new).await?;
    }

    Ok(())
}

async fn process_attendant_command(
    state: &Arc<AppState>,
    phone: &str,
    content: &str,
) -> Option<String> {
    let is_attendant = check_is_attendant(state, phone).await;

    if !is_attendant {
        return None;
    }

    let current_session = get_attendant_active_session(state, phone).await;

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
        let _ = current_session;
        Some(format!(
            "Attendance module not enabled. Message: {}",
            content
        ))
    }
}

async fn check_is_attendant(state: &Arc<AppState>, phone: &str) -> bool {
    let _conn = state.conn.clone();
    let phone_clone = phone.to_string();

    tokio::task::spawn_blocking(move || {
        let work_path = std::env::var("WORK_PATH").unwrap_or_else(|_| "./work".to_string());

        if let Ok(entries) = std::fs::read_dir(&work_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() && path.to_string_lossy().ends_with(".gbai") {
                    let attendant_path = path.join("attendant.csv");
                    if attendant_path.exists() {
                        if let Ok(content) = std::fs::read_to_string(&attendant_path) {
                            for line in content.lines().skip(1) {
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

async fn get_attendant_active_session(state: &Arc<AppState>, phone: &str) -> Option<Uuid> {
    let conn = state.conn.clone();
    let phone_clone = phone.to_string();

    tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().ok()?;

        use crate::shared::models::schema::user_sessions;

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

        let existing_user: Option<(Uuid, String)> = users::table
            .filter(users::email.eq(&phone_clone))
            .select((users::id, users::username))
            .first(&mut db_conn)
            .optional()
            .map_err(|e| format!("Query error: {}", e))?;

        let (user_id, _username) = if let Some((id, uname)) = existing_user {
            (id, uname)
        } else {
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

        let bot_id: Uuid = bots::table
            .filter(bots::is_active.eq(true))
            .select(bots::id)
            .first(&mut db_conn)
            .map_err(|e| format!("No active bot found: {}", e))?;

        let existing_session: Option<UserSession> = user_sessions::table
            .filter(user_sessions::user_id.eq(user_id))
            .filter(user_sessions::bot_id.eq(bot_id))
            .order(user_sessions::created_at.desc())
            .first(&mut db_conn)
            .optional()
            .map_err(|e| format!("Session query error: {}", e))?;

        if let Some(session) = existing_session {
            let age = Utc::now() - session.updated_at;
            if age.num_hours() < 24 {
                return Ok::<(UserSession, bool), String>((session, false));
            }
        }

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

fn check_needs_human(session: &UserSession) -> bool {
    if let Some(needs_human) = session.context_data.get("needs_human") {
        return needs_human.as_bool().unwrap_or(false);
    }
    false
}

async fn route_to_bot(
    state: &Arc<AppState>,
    session: &UserSession,
    content: &str,
    _is_new: bool,
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

    let adapter = WhatsAppAdapter::new(state.conn.clone(), session.bot_id);

    let orchestrator = BotOrchestrator::new(state.clone());

    let (tx, mut rx) = tokio::sync::mpsc::channel::<BotResponse>(100);

    let phone = session
        .context_data
        .get("phone")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let phone_for_error = phone.clone();
    let adapter_for_send = WhatsAppAdapter::new(state.conn.clone(), session.bot_id);

    tokio::spawn(async move {
        while let Some(response) = rx.recv().await {
            if !response.content.is_empty() {
                let mut wa_response = response;
                wa_response.user_id.clone_from(&phone);
                wa_response.channel = "whatsapp".to_string();

                if let Err(e) = adapter_for_send.send_message(wa_response).await {
                    error!("Failed to send WhatsApp response: {}", e);
                }
            }
        }
    });

    if let Err(e) = orchestrator.stream_response(user_message, tx).await {
        error!("Bot processing error: {}", e);

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

    save_message_to_history(state, session, content, "customer").await?;

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

    if let Some(broadcast_tx) = state.attendant_broadcast.as_ref() {
        if let Err(e) = broadcast_tx.send(notification.clone()) {
            debug!("No attendants listening: {}", e);
        } else {
            info!("Notification sent to attendants");
        }
    }

    update_queue_item(state, session, content).await?;

    Ok(())
}

async fn save_message_to_history(
    state: &Arc<AppState>,
    session: &UserSession,
    content: &str,
    sender: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let conn = state.conn.clone();
    let session_id = session.id;
    let user_id = session.user_id;
    let content_clone = content.to_string();
    let sender_clone = sender.to_string();

    tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| format!("DB error: {}", e))?;

        use crate::shared::models::schema::message_history;

        diesel::insert_into(message_history::table)
            .values((
                message_history::id.eq(Uuid::new_v4()),
                message_history::session_id.eq(session_id),
                message_history::user_id.eq(user_id),
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

        let current: UserSession = user_sessions::table
            .find(session_id)
            .first(&mut db_conn)
            .map_err(|e| format!("Find error: {}", e))?;

        let mut ctx = current.context_data;
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

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub to: String,
    pub message: String,
    #[serde(default)]
    pub template: Option<String>,
}

pub async fn send_message(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SendMessageRequest>,
) -> impl IntoResponse {
    info!("Sending WhatsApp message to {}", request.to);

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

#[derive(Debug, Deserialize)]
pub struct AttendantRespondRequest {
    pub session_id: String,
    pub message: String,
    pub attendant_id: String,
}

pub async fn attendant_respond(
    State(state): State<Arc<AppState>>,
    Json(request): Json<AttendantRespondRequest>,
) -> impl IntoResponse {
    info!(
        "Attendant {} responding to session {}",
        request.attendant_id, request.session_id
    );

    let Ok(session_id) = Uuid::parse_str(&request.session_id) else {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "success": false,
                "error": "Invalid session ID"
            })),
        );
    };

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

    let Some(session) = session_result else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "success": false,
                "error": "Session not found"
            })),
        );
    };

    let channel = session
        .context_data
        .get("channel")
        .and_then(|v| v.as_str())
        .unwrap_or("web");

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

    if let Err(e) = save_message_to_history(&state, &session, &request.message, "attendant").await {
        error!("Failed to save attendant message: {}", e);
    }

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

async fn get_verify_token(_state: &Arc<AppState>) -> String {
    use crate::core::secrets::SecretsManager;

    match SecretsManager::from_env() {
        Ok(secrets) => match secrets.get_value("gbo/whatsapp", "verify_token").await {
            Ok(token) => token,
            Err(_) => "webhook_verify".to_string(),
        },
        Err(_) => "webhook_verify".to_string(),
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_text_message() {
        let message = WhatsAppMessage {
            id: "msg123".to_string(),
            from: "+1234567890".to_string(),
            timestamp: "1234567890".to_string(),
            message_type: "text".to_string(),
            text: Some(WhatsAppText {
                body: "Hello, world!".to_string(),
            }),
            image: None,
            audio: None,
            video: None,
            document: None,
            location: None,
            interactive: None,
            button: None,
        };

        let content = extract_message_content(&message);
        assert_eq!(content, "Hello, world!");
    }

    #[test]
    fn test_extract_interactive_button() {
        let message = WhatsAppMessage {
            id: "msg123".to_string(),
            from: "+1234567890".to_string(),
            timestamp: "1234567890".to_string(),
            message_type: "interactive".to_string(),
            text: None,
            image: None,
            audio: None,
            video: None,
            document: None,
            location: None,
            interactive: Some(WhatsAppInteractive {
                interactive_type: "button_reply".to_string(),
                button_reply: Some(WhatsAppButtonReply {
                    id: "btn1".to_string(),
                    title: "Yes".to_string(),
                }),
                list_reply: None,
            }),
            button: None,
        };

        let content = extract_message_content(&message);
        assert_eq!(content, "Yes");
    }

    #[test]
    fn test_extract_list_reply() {
        let message = WhatsAppMessage {
            id: "msg123".to_string(),
            from: "+1234567890".to_string(),
            timestamp: "1234567890".to_string(),
            message_type: "interactive".to_string(),
            text: None,
            image: None,
            audio: None,
            video: None,
            document: None,
            location: None,
            interactive: Some(WhatsAppInteractive {
                interactive_type: "list_reply".to_string(),
                button_reply: None,
                list_reply: Some(WhatsAppListReply {
                    id: "list1".to_string(),
                    title: "Option A".to_string(),
                    description: Some("First option".to_string()),
                }),
            }),
            button: None,
        };

        let content = extract_message_content(&message);
        assert_eq!(content, "Option A");
    }

    #[test]
    fn test_extract_button_message() {
        let message = WhatsAppMessage {
            id: "msg123".to_string(),
            from: "+1234567890".to_string(),
            timestamp: "1234567890".to_string(),
            message_type: "button".to_string(),
            text: None,
            image: None,
            audio: None,
            video: None,
            document: None,
            location: None,
            interactive: None,
            button: Some(WhatsAppButton {
                payload: "btn_payload".to_string(),
                text: "Click me".to_string(),
            }),
        };

        let content = extract_message_content(&message);
        assert_eq!(content, "Click me");
    }

    #[test]
    fn test_extract_location_message() {
        let message = WhatsAppMessage {
            id: "msg123".to_string(),
            from: "+1234567890".to_string(),
            timestamp: "1234567890".to_string(),
            message_type: "location".to_string(),
            text: None,
            image: None,
            audio: None,
            video: None,
            document: None,
            location: Some(WhatsAppLocation {
                latitude: 40.7128,
                longitude: -74.0060,
                name: Some("New York".to_string()),
                address: None,
            }),
            interactive: None,
            button: None,
        };

        let content = extract_message_content(&message);
        assert!(content.contains("40.7128"));
        assert!(content.contains("-74.006"));
        assert!(content.contains("New York"));
    }

    #[test]
    fn test_extract_media_message() {
        let message = WhatsAppMessage {
            id: "msg123".to_string(),
            from: "+1234567890".to_string(),
            timestamp: "1234567890".to_string(),
            message_type: "image".to_string(),
            text: None,
            image: Some(WhatsAppMedia {
                id: "media123".to_string(),
                mime_type: Some("image/jpeg".to_string()),
                sha256: None,
                caption: Some("My photo".to_string()),
            }),
            audio: None,
            video: None,
            document: None,
            location: None,
            interactive: None,
            button: None,
        };

        let content = extract_message_content(&message);
        assert_eq!(content, "[image message]");
    }

    // Additional tests from bottest/mocks/whatsapp.rs

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct SentMessage {
        pub id: String,
        pub to: String,
        pub message_type: MessageType,
        pub content: MessageContent,
        pub timestamp: chrono::DateTime<chrono::Utc>,
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum MessageType {
        Text,
        Template,
        Image,
        Document,
        Audio,
        Video,
        Location,
        Contacts,
        Interactive,
        Reaction,
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    #[serde(untagged)]
    pub enum MessageContent {
        Text {
            body: String,
        },
        Template {
            name: String,
            language: String,
            components: Vec<serde_json::Value>,
        },
        Media {
            url: Option<String>,
            caption: Option<String>,
        },
        Location {
            latitude: f64,
            longitude: f64,
            name: Option<String>,
        },
        Interactive {
            r#type: String,
            body: serde_json::Value,
        },
        Reaction {
            message_id: String,
            emoji: String,
        },
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    struct ErrorResponseTest {
        error: ErrorDetailTest,
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    struct ErrorDetailTest {
        message: String,
        #[serde(rename = "type")]
        error_type: String,
        code: i32,
        fbtrace_id: String,
    }

    #[test]
    fn test_message_type_serialization() {
        let msg_type = MessageType::Template;
        let json = serde_json::to_string(&msg_type).unwrap();
        assert_eq!(json, "\"template\"");
    }

    #[test]
    fn test_webhook_event_serialization() {
        let event = WhatsAppWebhook {
            object: "whatsapp_business_account".to_string(),
            entry: vec![],
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("whatsapp_business_account"));
    }

    #[test]
    fn test_incoming_message_text_full() {
        let msg = WhatsAppMessage {
            id: "wamid.123".to_string(),
            from: "15551234567".to_string(),
            timestamp: "1234567890".to_string(),
            message_type: "text".to_string(),
            text: Some(WhatsAppText {
                body: "Hello!".to_string(),
            }),
            image: None,
            audio: None,
            video: None,
            document: None,
            location: None,
            interactive: None,
            button: None,
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("Hello!"));
        assert!(json.contains("15551234567"));
    }

    #[test]
    fn test_message_status_serialization() {
        let status = WhatsAppStatus {
            id: "wamid.123".to_string(),
            status: "delivered".to_string(),
            timestamp: "1234567890".to_string(),
            recipient_id: "15551234567".to_string(),
        };

        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("delivered"));
    }

    #[test]
    fn test_error_response_whatsapp() {
        let error = ErrorResponseTest {
            error: ErrorDetailTest {
                message: "Test error".to_string(),
                error_type: "OAuthException".to_string(),
                code: 100,
                fbtrace_id: "trace123".to_string(),
            },
        };

        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("Test error"));
        assert!(json.contains("100"));
    }

    #[test]
    fn test_whatsapp_webhook_deserialization() {
        let json = r#"{
            "object": "whatsapp_business_account",
            "entry": [{
                "id": "123456789",
                "changes": [{
                    "field": "messages",
                    "value": {
                        "messaging_product": "whatsapp",
                        "metadata": {
                            "display_phone_number": "15551234567",
                            "phone_number_id": "987654321"
                        }
                    }
                }]
            }]
        }"#;

        let webhook: WhatsAppWebhook = serde_json::from_str(json).unwrap();
        assert_eq!(webhook.object, "whatsapp_business_account");
        assert_eq!(webhook.entry.len(), 1);
        assert_eq!(webhook.entry[0].id, "123456789");
    }

    #[test]
    fn test_whatsapp_contact_profile() {
        let json = r#"{
            "wa_id": "15551234567",
            "profile": {
                "name": "John Doe"
            }
        }"#;

        let contact: WhatsAppContact = serde_json::from_str(json).unwrap();
        assert_eq!(contact.wa_id, "15551234567");
        assert_eq!(contact.profile.name, "John Doe");
    }

    #[test]
    fn test_whatsapp_media_with_caption() {
        let media = WhatsAppMedia {
            id: "media123".to_string(),
            mime_type: Some("image/jpeg".to_string()),
            sha256: Some("abc123hash".to_string()),
            caption: Some("My vacation photo".to_string()),
        };

        let json = serde_json::to_string(&media).unwrap();
        assert!(json.contains("media123"));
        assert!(json.contains("image/jpeg"));
        assert!(json.contains("My vacation photo"));
    }

    #[test]
    fn test_whatsapp_location_with_address() {
        let location = WhatsAppLocation {
            latitude: 37.7749,
            longitude: -122.4194,
            name: Some("San Francisco".to_string()),
            address: Some("California, USA".to_string()),
        };

        let json = serde_json::to_string(&location).unwrap();
        assert!(json.contains("37.7749"));
        assert!(json.contains("-122.4194"));
        assert!(json.contains("San Francisco"));
        assert!(json.contains("California, USA"));
    }

    #[test]
    fn test_whatsapp_list_reply_with_description() {
        let list_reply = WhatsAppListReply {
            id: "list_option_1".to_string(),
            title: "Option 1".to_string(),
            description: Some("This is the first option".to_string()),
        };

        let json = serde_json::to_string(&list_reply).unwrap();
        assert!(json.contains("list_option_1"));
        assert!(json.contains("Option 1"));
        assert!(json.contains("This is the first option"));
    }

    #[test]
    fn test_extract_audio_message() {
        let message = WhatsAppMessage {
            id: "msg123".to_string(),
            from: "+1234567890".to_string(),
            timestamp: "1234567890".to_string(),
            message_type: "audio".to_string(),
            text: None,
            image: None,
            audio: Some(WhatsAppMedia {
                id: "audio123".to_string(),
                mime_type: Some("audio/ogg".to_string()),
                sha256: None,
                caption: None,
            }),
            video: None,
            document: None,
            location: None,
            interactive: None,
            button: None,
        };

        let content = extract_message_content(&message);
        assert_eq!(content, "[audio message]");
    }

    #[test]
    fn test_extract_video_message() {
        let message = WhatsAppMessage {
            id: "msg123".to_string(),
            from: "+1234567890".to_string(),
            timestamp: "1234567890".to_string(),
            message_type: "video".to_string(),
            text: None,
            image: None,
            audio: None,
            video: Some(WhatsAppMedia {
                id: "video123".to_string(),
                mime_type: Some("video/mp4".to_string()),
                sha256: None,
                caption: Some("Check this out!".to_string()),
            }),
            document: None,
            location: None,
            interactive: None,
            button: None,
        };

        let content = extract_message_content(&message);
        assert_eq!(content, "[video message]");
    }

    #[test]
    fn test_extract_document_message() {
        let message = WhatsAppMessage {
            id: "msg123".to_string(),
            from: "+1234567890".to_string(),
            timestamp: "1234567890".to_string(),
            message_type: "document".to_string(),
            text: None,
            image: None,
            audio: None,
            video: None,
            document: Some(WhatsAppMedia {
                id: "doc123".to_string(),
                mime_type: Some("application/pdf".to_string()),
                sha256: None,
                caption: Some("Invoice".to_string()),
            }),
            location: None,
            interactive: None,
            button: None,
        };

        let content = extract_message_content(&message);
        assert_eq!(content, "[document message]");
    }

    #[test]
    fn test_whatsapp_value_with_statuses() {
        let json = r#"{
            "messaging_product": "whatsapp",
            "metadata": {
                "display_phone_number": "15551234567",
                "phone_number_id": "987654321"
            },
            "statuses": [{
                "id": "wamid.123",
                "status": "sent",
                "timestamp": "1234567890",
                "recipient_id": "15559876543"
            }]
        }"#;

        let value: WhatsAppValue = serde_json::from_str(json).unwrap();
        assert_eq!(value.messaging_product, "whatsapp");
        assert!(!value.statuses.is_empty());
        assert_eq!(value.statuses.len(), 1);
        assert_eq!(value.statuses[0].status, "sent");
    }
}

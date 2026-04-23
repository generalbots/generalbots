use crate::core::bot::{BotOrchestrator, get_default_bot};
use crate::multimodal::BotModelsClient;
use crate::core::bot::channels::whatsapp::WhatsAppAdapter;
use crate::core::bot::channels::ChannelAdapter;
use crate::core::config::ConfigManager;
use crate::core::shared::models::{BotResponse, UserMessage, UserSession};
use crate::core::shared::state::{AppState, AttendantNotification};
use axum::{
    extract::{Path, Query, State},
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
        .route("/webhook/whatsapp/:bot_id", get(verify_webhook))
        .route("/webhook/whatsapp/:bot_id", post(handle_webhook))
        .route("/api/whatsapp/send", post(send_message))
}



/// Resolve bot_id string to Uuid.
/// - "default" → returns UUID of the default bot
/// - Valid UUID string → returns the UUID
/// - Otherwise → returns error response
async fn resolve_bot_id(
    bot_id_str: &str,
    state: &Arc<AppState>,
) -> Result<Uuid, (StatusCode, String)> {
    if bot_id_str == "default" {
        let conn = state.conn.clone();
        let bot_id = tokio::task::spawn_blocking(move || {
            let mut db_conn = conn.get().ok()?;
            let (id, _) = get_default_bot(&mut db_conn);
            Some(id)
        })
        .await
        .ok()
        .flatten()
        .unwrap_or_else(Uuid::nil);

        if bot_id.is_nil() {
            return Err((StatusCode::NOT_FOUND, "Default bot not found".to_string()));
        }
        info!("Resolved 'default' to bot_id: {}", bot_id);
        Ok(bot_id)
    } else {
        Uuid::parse_str(bot_id_str)
            .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid bot ID: {}", e)))
    }
}

pub async fn verify_webhook(
    State(state): State<Arc<AppState>>,
    Path(bot_id_str): Path<String>,
    Query(params): Query<WebhookVerifyQuery>,
) -> impl IntoResponse {
    let bot_id = match resolve_bot_id(&bot_id_str, &state).await {
        Ok(id) => id,
        Err(err) => return err,
    };

    info!("WhatsApp webhook verification request received for bot {} (input: {})", bot_id, bot_id_str);

    let mode = params.mode.unwrap_or_default();
    let token = params.verify_token.unwrap_or_default();
    let challenge = params.challenge.unwrap_or_default();

    if mode != "subscribe" {
        warn!("Invalid webhook mode: {}", mode);
        return (StatusCode::FORBIDDEN, "Invalid mode".to_string());
    }

    let expected_token = get_verify_token_for_bot(&state, &bot_id).await;

    if token == expected_token {
        info!("Webhook verification successful for bot {}", bot_id);
        (StatusCode::OK, challenge)
    } else {
        warn!("Invalid verify token for bot {}", bot_id);
        (StatusCode::FORBIDDEN, "Invalid verify token".to_string())
    }
}

pub async fn handle_webhook(
    State(state): State<Arc<AppState>>,
    Path(bot_id_str): Path<String>,
    body: axum::body::Bytes,
) -> impl IntoResponse {
    let bot_id = match resolve_bot_id(&bot_id_str, &state).await {
        Ok(id) => id,
        Err(err) => return err.0,
    };

    log::info!("[BASIC_EXEC] WhatsApp webhook received for bot_id={}", bot_id);
    debug!("Raw webhook body: {}", String::from_utf8_lossy(&body));

    let payload: WhatsAppWebhook = match serde_json::from_slice(&body) {
        Ok(p) => p,
        Err(e) => {
            error!("Failed to deserialize WhatsApp webhook: {}", e);
            return StatusCode::BAD_REQUEST;
        }
    };

    info!("WhatsApp webhook received for bot {}: {:?}", bot_id, payload.object);
    debug!("Webhook entry count: {}", payload.entry.len());

    if payload.object != "whatsapp_business_account" {
        return StatusCode::OK;
    }

    for entry in payload.entry {
        debug!("Entry changes count: {}", entry.changes.len());
        for change in entry.changes {
            debug!("Change field: {}", change.field);
            if change.field == "messages" {
                // ==================== Phone Number ID Based Routing ====================
                // Try to resolve bot by phone_number_id from webhook metadata
                let effective_bot_id = if let Some(ref phone_id) = change.value.metadata.phone_number_id {
                    if let Some(routed_bot_id) = resolve_bot_by_phone_number_id(&state, phone_id).await {
                        info!("Phone number ID routing: {} -> bot {}", phone_id, routed_bot_id);
                        routed_bot_id
                    } else {
                        debug!("No bot found for phone_number_id {}, using default", phone_id);
                        bot_id
                    }
                } else {
                    bot_id
                };

                debug!("Processing 'messages' field change for bot {}", effective_bot_id);
                debug!("Contacts count: {}", change.value.contacts.len());
                let contact = change.value.contacts.first();
                let contact_name = contact.map(|c| c.profile.name.clone());
                let contact_phone = contact.map(|c| c.wa_id.clone());

                debug!("Number of messages in webhook: {}", change.value.messages.len());
                for message in change.value.messages {
                    debug!("Message ID: {}, Type: {}, From: {}", message.id, message.message_type, message.from);
                    if let Err(e) = process_incoming_message(
                        state.clone(),
                        &effective_bot_id,
                        &message,
                        contact_name.clone(),
                        contact_phone.clone(),
                    )
                    .await
                    {
                        error!("Failed to process WhatsApp message for bot {}: {}", effective_bot_id, e);
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

// ==================== Phone Number ID → Bot Routing ====================

/// Resolve bot by whatsapp-phone-number-id from webhook metadata.
/// This allows automatic routing based on which WhatsApp phone number received the message.
async fn resolve_bot_by_phone_number_id(
    state: &Arc<AppState>,
    phone_number_id: &str,
) -> Option<Uuid> {
    if phone_number_id.is_empty() {
        return None;
    }

    let conn = state.conn.clone();
    let search_id = phone_number_id.to_string();

    let result = tokio::task::spawn_blocking(move || {
        use crate::core::shared::models::schema::{bots, bot_configuration};
        use diesel::prelude::*;

        let mut db_conn = conn.get().ok()?;

        // Find bot with matching whatsapp-phone-number-id
        let bot_id: Option<Uuid> = bot_configuration::table
            .inner_join(bots::table.on(bot_configuration::bot_id.eq(bots::id)))
            .filter(bots::is_active.eq(true))
            .filter(bot_configuration::config_key.eq("whatsapp-phone-number-id"))
            .filter(bot_configuration::config_value.eq(&search_id))
            .select(bot_configuration::bot_id)
            .first(&mut db_conn)
            .ok();

        bot_id
    })
    .await
    .ok()?;

    if let Some(bot_id) = result {
        info!("Resolved phone_number_id {} to bot {}", phone_number_id, bot_id);
        return Some(bot_id);
    }
    None
}

// ==================== Phone → Bot Routing Cache Functions ====================

/// Get the cached bot_id for a phone number from the routing cache
async fn get_cached_bot_for_phone(state: &Arc<AppState>, phone: &str) -> Option<Uuid> {
    let cache = state.cache.as_ref()?;
    let mut conn = cache.get_multiplexed_async_connection().await.ok()?;
    let key = format!("wa_phone_bot:{}", phone);

    let bot_id_str: Option<String> = redis::cmd("GET")
        .arg(&key)
        .query_async(&mut conn)
        .await
        .ok()
        .flatten();

    if let Some(bot_id_str) = bot_id_str {
        if let Ok(bot_id) = Uuid::parse_str(&bot_id_str) {
            debug!("Found cached bot {} for phone {}", bot_id, phone);
            return Some(bot_id);
        }
    }
    None
}

/// Set the bot_id for a phone number in the routing cache
async fn set_cached_bot_for_phone(state: &Arc<AppState>, phone: &str, bot_id: Uuid) {
    if let Some(cache) = &state.cache {
        if let Ok(mut conn) = cache.get_multiplexed_async_connection().await {
            let key = format!("wa_phone_bot:{}", phone);
            // Cache for 24 hours (86400 seconds)
            let result: Result<(), _> = redis::cmd("SET")
                .arg(&key)
                .arg(bot_id.to_string())
                .arg("EX")
                .arg(86400)
                .query_async(&mut conn)
                .await;
            if let Err(e) = result {
                error!("Failed to cache bot for phone {}: {}", phone, e);
            } else {
                info!("Cached bot {} for phone {}", bot_id, phone);
            }
        }
    }
}

// ==================== WhatsApp ID Routing Functions ====================

/// Check if the message text is a whatsapp-id routing command.
/// Returns the bot_id if a matching bot is found with that whatsapp-id.
async fn check_whatsapp_id_routing(
    state: &Arc<AppState>,
    message_text: &str,
) -> Option<Uuid> {
    let text = message_text.trim().to_lowercase();

    // Skip empty messages or messages that are too long (whatsapp-id should be short)
    if text.is_empty() || text.len() > 50 {
        return None;
    }

    // Skip messages that look like regular sentences (contain spaces or common punctuation)
    if text.contains(' ') || text.contains('.') || text.contains('?') || text.contains('!') {
        return None;
    }

    // Search for a bot with matching whatsapp-id in config
    let conn = state.conn.clone();
    let search_text = text.clone();

    let result = tokio::task::spawn_blocking(move || {
        use crate::core::shared::models::schema::{bots, bot_configuration};
        use diesel::prelude::*;

        let mut db_conn = conn.get().ok()?;

        // Find all active bots with whatsapp-id config
        let bot_ids_with_whatsapp_id: Vec<(Uuid, String)> = bot_configuration::table
            .inner_join(bots::table.on(bot_configuration::bot_id.eq(bots::id)))
            .filter(bots::is_active.eq(true))
            .filter(bot_configuration::config_key.eq("whatsapp-id"))
            .select((bot_configuration::bot_id, bot_configuration::config_value))
            .load::<(Uuid, String)>(&mut db_conn)
            .ok()?;

        // Find matching bot
        for (bot_id, whatsapp_id) in bot_ids_with_whatsapp_id {
            if whatsapp_id.to_lowercase() == search_text {
                return Some(bot_id);
            }
        }
        None
    })
    .await
    .ok()
    .flatten();

    if let Some(bot_id) = result {
        info!("Found bot {} matching whatsapp-id: {}", bot_id, text);
    }

    result
}

async fn process_incoming_message(
    state: Arc<AppState>,
    bot_id: &Uuid,
    message: &WhatsAppMessage,
    contact_name: Option<String>,
    contact_phone: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Deduplicate messages using cache to prevent processing the same message twice
    // WhatsApp may retry webhook delivery, causing duplicate processing
    let message_id_key = format!("wa_msg_processed:{}", message.id);
    if let Some(cache) = &state.cache {
        if let Ok(mut conn) = cache.get_multiplexed_async_connection().await {
            // SETNX returns true (1) if key was set (first time), false (0) if key existed (duplicate)
            let is_new_message: bool = redis::cmd("SET")
                .arg(&message_id_key)
                .arg("1")
                .arg("NX") // Only set if not exists
                .arg("EX")
                .arg("300") // 5 minutes TTL
                .query_async(&mut conn)
                .await
                .unwrap_or(false);

            if !is_new_message {
                info!("Skipping duplicate WhatsApp message ID: {}", message.id);
                return Ok(());
            }
        }
    }

    let phone = contact_phone
        .clone()
        .unwrap_or_else(|| message.from.clone());
    let name = contact_name.clone().unwrap_or_else(|| phone.clone());

    let mut content = extract_message_content(message);
    
    // Auto-transcribe audio if BotModels is enabled
    if message.message_type == "audio" {
        if let Some(audio) = &message.audio {
            info!("Received audio message {}, attempting transcription for bot {}", audio.id, bot_id);
            match process_audio_message(&state, bot_id, audio).await {
                Ok(transcription) => {
                    info!("Audio transcription successful: '{}'", transcription);
                    content = transcription;
                },
                Err(e) => {
                    error!("Audio transcription failed: {}. Continuing with placeholder.", e);
                    content = "[Áudio]".to_string();
                }
            }
        }
    }

    debug!("Final WhatsApp message content: '{}'", content);

    if content.is_empty() {
        warn!("Empty content after processing WhatsApp message type {}", message.message_type);
        return Ok(());
    }

    // ==================== Dynamic Bot Routing ====================
    // Check if this is a whatsapp-id routing command (e.g., "cristo", "salesianos")
    let mut effective_bot_id = *bot_id;

    if let Some(routed_bot_id) = check_whatsapp_id_routing(&state, &content).await {
        // User typed a whatsapp-id command - switch to that bot
        info!(
            "Routing WhatsApp user {} from bot {} to bot {} (whatsapp-id: {})",
            phone, bot_id, routed_bot_id, content
        );
        effective_bot_id = routed_bot_id;
        set_cached_bot_for_phone(&state, &phone, routed_bot_id).await;

        // Get or create session for the new bot
        let (session, is_new) = match find_or_create_session(&state, &effective_bot_id, &phone, &name).await {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to create session for routed bot: {}", e);
                return Ok(());
            }
        };

        // Clear start.bas execution flag for new bot's session
        if let Some(cache) = &state.cache {
            if let Ok(mut conn) = cache.get_multiplexed_async_connection().await {
                let key = format!("start_bas_executed:{}", session.id);
                let _: Result<(), redis::RedisError> = redis::cmd("DEL")
                    .arg(&key)
                    .query_async(&mut conn)
                    .await;
                debug!("Cleared start.bas flag {} after bot switch to {}", key, routed_bot_id);
            }
        }

        // Send confirmation message
        let adapter = WhatsAppAdapter::new(&state, effective_bot_id);
        let bot_response = BotResponse {
            bot_id: effective_bot_id.to_string(),
            session_id: session.id.to_string(),
            user_id: phone.clone(),
            channel: "whatsapp".to_string(),
            content: format!("✅ Bot alterado para: {}", content),
            message_type: MessageType::BOT_RESPONSE,
            stream_token: None,
            is_complete: true,
        suggestions: vec![],
        switchers: Vec::new(),
        context_name: None,
        context_length: 0,
        context_max_length: 0,
    };
    if let Err(e) = adapter.send_message(bot_response).await {
        error!("Failed to send routing confirmation: {}", e);
    }

        // Execute start.bas immediately by calling route_to_bot
        info!("Executing start.bas for bot '{}' via route_to_bot", routed_bot_id);
        if let Err(e) = route_to_bot(&state, &session, "", is_new).await {
            error!("Failed to execute start.bas for bot switch: {}", e);
        }

        return Ok(());
    }

    // Check if there's a cached bot for this phone number
    if let Some(cached_bot_id) = get_cached_bot_for_phone(&state, &phone).await {
        if cached_bot_id != *bot_id {
            info!(
                "Using cached bot {} for phone {} (webhook bot: {})",
                cached_bot_id, phone, bot_id
            );
            effective_bot_id = cached_bot_id;
        }
    }

    info!(
        "Processing WhatsApp message from {} ({}) for bot {}: type={}",
        name, phone, effective_bot_id, message.message_type
    );

    // Handle /clear command - available to all users
    if content.trim().to_lowercase() == "/clear" {
        let adapter = WhatsAppAdapter::new(&state, effective_bot_id);

        // Find and clear the user's session
        match find_or_create_session(&state, &effective_bot_id, &phone, &name).await {
            Ok((session, _)) => {
                // Clear message history for this session
                if let Err(e) = clear_session_history(&state, &session.id).await {
                    error!("Failed to clear session history: {}", e);
                }

                let bot_response = BotResponse {
                    bot_id: effective_bot_id.to_string(),
                    session_id: session.id.to_string(),
                    user_id: phone.clone(),
                    channel: "whatsapp".to_string(),
                    content: "🧹 Histórico de conversa limpo! Posso ajudar com algo novo?".to_string(),
                    message_type: MessageType::BOT_RESPONSE,
                    stream_token: None,
                    is_complete: true,
        suggestions: vec![],
        switchers: Vec::new(),
        context_name: None,
        context_length: 0,
        context_max_length: 0,
    };
    if let Err(e) = adapter.send_message(bot_response).await {
        error!("Failed to send clear confirmation: {}", e);
    }
                info!("Cleared conversation history for WhatsApp user {}", phone);
            }
            Err(e) => {
                error!("Failed to get session for /clear: {}", e);
            }
        }
        return Ok(());
    }

    if content.starts_with('/') {
        if let Some(response) = process_attendant_command(&state, &phone, &content).await {
            let adapter = WhatsAppAdapter::new(&state, effective_bot_id);
            let bot_response = BotResponse {
                bot_id: effective_bot_id.to_string(),
                session_id: Uuid::nil().to_string(),
                user_id: phone.clone(),
                channel: "whatsapp".to_string(),
                content: response,
                message_type: MessageType::BOT_RESPONSE,
                stream_token: None,
                is_complete: true,
        suggestions: vec![],
        switchers: Vec::new(),
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

    let (session, is_new) = find_or_create_session(&state, &effective_bot_id, &phone, &name).await?;

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

    #[cfg(feature = "attendant")]
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

    #[cfg(not(feature = "attendant"))]
    {
        let _ = current_session;
        Some(format!(
            "Attendance module not enabled. Message: {}",
            content
        ))
    }
}

async fn check_is_attendant(_state: &Arc<AppState>, phone: &str) -> bool {
    let phone_clone = phone.to_string();

    tokio::task::spawn_blocking(move || {
        let work_path = crate::core::shared::utils::get_work_path();

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

        use crate::core::shared::models::schema::user_sessions;

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
    bot_id: &Uuid,
    phone: &str,
    name: &str,
) -> Result<(UserSession, bool), Box<dyn std::error::Error + Send + Sync>> {
    let conn = state.conn.clone();
    let phone_clone = phone.to_string();
    let name_clone = name.to_string();
    let bot_id_clone = *bot_id;

    let result = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| format!("DB error: {}", e))?;

        use crate::core::shared::models::schema::{user_sessions, users};

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

        let existing_session: Option<UserSession> = user_sessions::table
            .filter(user_sessions::user_id.eq(user_id))
            .filter(user_sessions::bot_id.eq(bot_id_clone))
            .order(user_sessions::created_at.desc())
            .first(&mut db_conn)
            .optional()
            .map_err(|e| format!("Session query error: {}", e))?;

        if let Some(session) = existing_session {
            diesel::update(user_sessions::table.filter(user_sessions::id.eq(session.id)))
                .set(user_sessions::updated_at.eq(diesel::dsl::now))
                .execute(&mut db_conn)
                .map_err(|e| format!("Update session error: {}", e))?;
            return Ok::<(UserSession, bool), String>((session, false));
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
                user_sessions::bot_id.eq(bot_id_clone),
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

async fn clear_session_history(
    state: &Arc<AppState>,
    session_id: &Uuid,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let conn = state.conn.clone();
    let session_id_copy = *session_id;

    tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| format!("DB error: {}", e))?;

        use crate::core::shared::models::schema::message_history::dsl::*;

        diesel::delete(message_history.filter(session_id.eq(session_id_copy)))
            .execute(&mut db_conn)
            .map_err(|e| format!("Delete messages error: {}", e))?;

        info!("Cleared message history for session {}", session_id_copy);
        Ok::<(), String>(())
    })
    .await
    .map_err(|e| format!("Task error: {}", e))??;

    Ok(())
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

    let adapter = WhatsAppAdapter::new(&state, session.bot_id);

    let orchestrator = BotOrchestrator::new(state.clone());

    let (tx, mut rx) = tokio::sync::mpsc::channel::<BotResponse>(100);

    let phone = session
        .context_data
        .get("phone")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let phone_for_error = phone.clone();
    let adapter_for_send = WhatsAppAdapter::new(&state, session.bot_id);
    let bot_id_for_voice = session.bot_id;
    let state_clone = state.clone();

    tokio::spawn(async move {
        let mut buffer = String::new();
        const MAX_WHATSAPP_LENGTH: usize = 4000;
        const MIN_FLUSH_PARAGRAPHS: usize = 3;

        /// Check if a line is a list item (numbered: "1. ", "10. ", etc.)
        fn is_numbered_list_item(line: &str) -> bool {
            let trimmed = line.trim();
            // Must start with digit(s) followed by '.' or ')' and then space or end
            let chars: Vec<char> = trimmed.chars().collect();
            let mut i = 0;
            // Skip digits
            while i < chars.len() && chars[i].is_numeric() {
                i += 1;
            }
            // Must have at least one digit and be followed by '.' or ')' then space
            i > 0 && i < chars.len() && (chars[i] == '.' || chars[i] == ')')
                && (i + 1 >= chars.len() || chars[i + 1] == ' ')
        }

        /// Check if a line is a bullet list item
        fn is_bullet_list_item(line: &str) -> bool {
            let trimmed = line.trim();
            trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("• ")
        }

        /// Check if a line is any type of list item
        fn is_list_item(line: &str) -> bool {
            is_numbered_list_item(line) || is_bullet_list_item(line)
        }

        /// Check if buffer contains a list (any line starting with list marker)
        fn contains_list(text: &str) -> bool {
            text.lines().any(is_list_item)
        }

        /// Check if buffer looks like it might be starting a list
        /// (header followed by blank line, or ends with partial list item)
        fn looks_like_list_start(text: &str) -> bool {
            let lines: Vec<&str> = text.lines().collect();
            if lines.len() < 2 {
                return false;
            }
            // Check if last non-empty line looks like a header (short, ends with ':')
            let last_content = lines.iter().rev().find(|l| !l.trim().is_empty());
            if let Some(line) = last_content {
                let trimmed = line.trim();
                // Header pattern: short line ending with ':'
                if trimmed.len() < 50 && trimmed.ends_with(':') {
                    return true;
                }
                // Partial list item: starts with number but incomplete
                if trimmed.chars().next().map(|c| c.is_numeric()).unwrap_or(false) {
                    return true;
                }
            }
            false
        }

        /// Check if a list appears to have ended (had list items, but last lines are not list items)
        fn looks_like_list_end(text: &str) -> bool {
            let lines: Vec<&str> = text.lines().collect();
            if lines.len() < 3 {
                return false;
            }

            // Check if there's at least one list item in the text
            let has_list_items = lines.iter().any(|l| is_list_item(l));
            if !has_list_items {
                return false;
            }

            // Check if the last 2+ non-empty lines are NOT list items
            let non_empty_lines: Vec<&str> = lines.iter().rev()
                .copied()
                .filter(|l| !l.trim().is_empty())
                .take(2)
                .collect();

            if non_empty_lines.len() < 2 {
                return false;
            }

            // If the last 2 non-empty lines are not list items, the list has likely ended
            non_empty_lines.iter().all(|l| !is_list_item(l))
        }

        /// Split text into (before_list, list_and_after)
        /// Returns everything before the first list item, and everything from the list item onwards
        fn split_text_before_list(text: &str) -> (String, String) {
            let lines: Vec<&str> = text.lines().collect();
            let mut list_start_idx = None;

            // Find the first list item
            for (idx, line) in lines.iter().enumerate() {
                if is_list_item(line) {
                    list_start_idx = Some(idx);
                    break;
                }
            }

            match list_start_idx {
                Some(idx) => {
                    let before = lines[..idx].join("\n");
                    let rest = lines[idx..].join("\n");
                    (before, rest)
                }
                None => (text.to_string(), String::new())
            }
        }

        /// Split text into (list, after_list)
        /// Extracts the list portion and any text after it
        fn split_list_from_text(text: &str) -> (String, String) {
            let lines: Vec<&str> = text.lines().collect();
            let mut list_end_idx = lines.len();

            // Find where the list ends (first non-list item after list starts)
            let mut found_list = false;
            for (idx, line) in lines.iter().enumerate() {
                if is_list_item(line) {
                    found_list = true;
                } else if found_list && !line.trim().is_empty() {
                    // Found non-empty, non-list line after list items
                    list_end_idx = idx;
                    break;
                }
            }

            let list = lines[..list_end_idx].join("\n");
            let after = lines[list_end_idx..].join("\n");
            (list, after)
        }

        /// Send a WhatsApp message part
        async fn send_part(
            adapter: &crate::core::bot::channels::whatsapp::WhatsAppAdapter,
            phone: &str,
            content: String,
            is_final: bool,
        ) {
            if content.trim().is_empty() {
                return;
            }
            let wa_response = crate::core::shared::models::BotResponse {
                bot_id: String::new(),
                user_id: phone.to_string(),
                session_id: String::new(),
                channel: "whatsapp".to_string(),
                content,
                message_type: crate::core::shared::models::MessageType::BOT_RESPONSE,
                stream_token: None,
                is_complete: is_final,
            suggestions: vec![],
            switchers: Vec::new(),
            context_name: None,
            context_length: 0,
            context_max_length: 0,
        };

        if let Err(e) = adapter.send_message(wa_response).await {
                log::error!("Failed to send WhatsApp response part: {}", e);
            }
            // Rate limiting is handled by WhatsAppAdapter::send_whatsapp_message
        }

        // Use the shared LLM hallucination detector (simple: 50+ repetitions = hallucination)
        let detector = crate::llm::hallucination_detector::HallucinationDetector::default();

        while let Some(response) = rx.recv().await {
            let is_final = response.is_complete;

            if !response.content.is_empty() {
                // Check for hallucination (50+ repetitions of same pattern)
                if detector.check(&response.content).await {
                    warn!("WA hallucination detected: {:?}, stopping stream", response.content);
                    // Send what we have and stop
                    if !buffer.trim().is_empty() {
                        let clean_buffer = buffer.trim_end();
                        send_part(&adapter_for_send, &phone, clean_buffer.to_string(), true).await;
                    }
                    break;
                }

                // Add response content to buffer
                buffer.push_str(&response.content);
            }

            // IMPROVED LOGIC:
            // 1. If buffer contains a list OR looks like list is starting, wait for final/too long
            // 2. Otherwise, use normal paragraph-based flushing

            let has_list = contains_list(&buffer);
            let maybe_list_start = !has_list && looks_like_list_start(&buffer);
            let list_ended = has_list && looks_like_list_end(&buffer);

            info!(
                "WA stream: is_final={}, has_list={}, maybe_start={}, list_ended={}, len={}, preview={:?}",
                is_final, has_list, maybe_list_start, list_ended, buffer.len(), &buffer.chars().take(80).collect::<String>()
            );

            if has_list || maybe_list_start {
                // With lists: isolate them as separate messages
                if list_ended {
                    info!("WA list ended, isolating list message");

                    // Step 1: Split text before list
                    let (text_before, rest) = split_text_before_list(&buffer);

                    // Step 2: Send text before list (if not empty)
                    if !text_before.trim().is_empty() {
                        info!("WA sending text before list, len={}", text_before.len());
                        send_part(&adapter_for_send, &phone, text_before, false).await;
                    }

                    // Step 3: Split list from text after
                    let (list, text_after) = split_list_from_text(&rest);

                    // Step 4: Send list (isolated)
                    if !list.trim().is_empty() {
                        info!("WA sending isolated list, len={}", list.len());
                        send_part(&adapter_for_send, &phone, list, false).await;
                    }

                    // Step 5: Keep text after in buffer
                    buffer = text_after;

                    if !buffer.trim().is_empty() {
                        debug!("WA keeping text after list in buffer, len={}", buffer.len());
                    }
                } else if is_final || buffer.len() >= MAX_WHATSAPP_LENGTH {
                    // Final message or buffer too long - send everything
                    info!("WA sending list message (final/overflow), len={}, has_list={}", buffer.len(), has_list);
                    if buffer.len() > MAX_WHATSAPP_LENGTH {
                        let parts = adapter_for_send.split_message_smart(&buffer, MAX_WHATSAPP_LENGTH);
                        for part in parts {
                            send_part(&adapter_for_send, &phone, part, is_final).await;
                        }
                    } else {
                        send_part(&adapter_for_send, &phone, buffer.clone(), is_final).await;
                    }
                    buffer.clear();
                } else {
                    debug!("WA waiting for more list content (buffer len={})", buffer.len());
                }
            } else {
                // No list: use normal paragraph-based flushing
                let paragraph_count = buffer
                    .split("\n\n")
                    .filter(|p| !p.trim().is_empty())
                    .count();

                let ends_with_paragraph = buffer.ends_with("\n\n") ||
                    (buffer.ends_with('\n') && buffer.len() > 1 && !buffer[..buffer.len()-1].ends_with('\n'));

                let should_flush = buffer.len() >= MAX_WHATSAPP_LENGTH ||
                    (paragraph_count >= MIN_FLUSH_PARAGRAPHS && ends_with_paragraph) ||
                    (is_final && !buffer.is_empty());

                if should_flush {
                    info!("WA sending non-list message, len={}, paragraphs={}", buffer.len(), paragraph_count);
                    if buffer.len() > MAX_WHATSAPP_LENGTH {
                        let parts = adapter_for_send.split_message_smart(&buffer, MAX_WHATSAPP_LENGTH);
                        for part in parts {
                            send_part(&adapter_for_send, &phone, part, is_final).await;
                        }
                    } else {
                        send_part(&adapter_for_send, &phone, buffer.clone(), is_final).await;
                    }
                    buffer.clear();
                }
            }

            if is_final && !buffer.trim().is_empty() {
                let final_text = buffer.trim().to_string();
                let state_for_voice = state_clone.clone();
                let phone_for_voice = phone.clone();

                let config_manager = ConfigManager::new(state_for_voice.conn.clone().into());
                let voice_response = config_manager
                    .get_config(&bot_id_for_voice, "whatsapp-voice-response", Some("false"))
                    .unwrap_or_else(|_| "false".to_string())
                    .to_lowercase()
                    == "true";

                if voice_response && !final_text.is_empty() {
                    info!("Voice response enabled, generating TTS for: {}", &final_text.chars().take(50).collect::<String>());

                    let client = BotModelsClient::from_state(&state_for_voice, &bot_id_for_voice);

                    if !client.is_enabled() {
                        warn!("BotModels not enabled, skipping voice response");
                    } else {
                        match client.generate_audio(&final_text, None, None).await {
                            Ok(audio_url) => {
                                info!("TTS generated: {}", audio_url);
                                let wa_adapter = WhatsAppAdapter::new(&state_for_voice, bot_id_for_voice);
                                if let Err(e) = wa_adapter.send_voice_message(&phone_for_voice, &audio_url).await {
                                    error!("Failed to send voice message: {}", e);
                                } else {
                                    info!("Voice message sent successfully to {}", phone_for_voice);
                                }
                            }
                            Err(e) => {
                                error!("Failed to generate TTS: {}", e);
                            }
                        }
                    }
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
        switchers: Vec::new(),
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

        use crate::core::shared::models::schema::message_history;

        diesel::insert_into(message_history::table)
            .values((
                message_history::id.eq(Uuid::new_v4()),
                message_history::session_id.eq(session_id),
                message_history::user_id.eq(user_id),
                message_history::role.eq(if sender_clone == "user" { 1 } else { 2 }),
                message_history::content_encrypted.eq(content_clone),
                message_history::message_type.eq(1),
                message_history::message_index.eq(0i32),
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

        use crate::core::shared::models::schema::user_sessions;

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
    let adapter = WhatsAppAdapter::new(&state, bot_id);

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
        switchers: Vec::new(),
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
        use crate::core::shared::models::schema::user_sessions;
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
            let adapter = WhatsAppAdapter::new(&state, session.bot_id);
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
        switchers: Vec::new(),
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

async fn get_verify_token_for_bot(state: &Arc<AppState>, bot_id: &Uuid) -> String {
    let config_manager = ConfigManager::new(state.conn.clone());
    let bot_id_clone = *bot_id;

    tokio::task::spawn_blocking(move || {
        config_manager
            .get_config(&bot_id_clone, "whatsapp-verify-token", None)
            .unwrap_or_else(|_| "webhook_verify".to_string())
    })
    .await
    .unwrap_or_else(|_| "webhook_verify".to_string())
}



async fn get_default_bot_id(state: &Arc<AppState>) -> Uuid {
    let conn = state.conn.clone();

    tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().ok()?;
        use crate::core::shared::models::schema::bots;
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

async fn process_audio_message(
    state: &Arc<AppState>,
    bot_id: &Uuid,
    audio: &WhatsAppMedia,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let adapter = WhatsAppAdapter::new(&state, *bot_id);
    let binary = adapter.download_media(&audio.id).await?;
    
    let bot_models = BotModelsClient::from_state(state, bot_id);
    if !bot_models.is_enabled() {
        return Err("BotModels not enabled for transcription".into());
    }
    
    // Save to temp file
    let temp_name = format!("/tmp/whatsapp_audio_{}.ogg", audio.id);
    tokio::fs::write(&temp_name, binary).await?;
    
    info!("Sending WhatsApp audio {} to BotModels for transcription", audio.id);
    let transcription = bot_models.speech_to_text(&temp_name).await?;
    
    // Clean up
    let _ = tokio::fs::remove_file(&temp_name).await;
    
    Ok(transcription)
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
    fn test_sent_message_serialization() {
        let sent = SentMessage {
            id: "wamid.test123".to_string(),
            to: "+15551234567".to_string(),
            message_type: MessageType::Text,
            content: MessageContent::Text {
                body: "Hello from bot".to_string(),
            },
            timestamp: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&sent).unwrap();
        assert!(json.contains("wamid.test123"));
        assert!(json.contains("Hello from bot"));
    }

    #[test]
    fn test_message_content_variants() {
        let text = MessageContent::Text {
            body: "Hello".to_string(),
        };
        let template = MessageContent::Template {
            name: "welcome".to_string(),
            language: "en".to_string(),
            components: vec![],
        };
        let media = MessageContent::Media {
            url: Some("https://example.com/image.jpg".to_string()),
            caption: Some("A photo".to_string()),
        };
        let location = MessageContent::Location {
            latitude: 40.7128,
            longitude: -74.0060,
            name: Some("New York".to_string()),
        };

        let text_json = serde_json::to_string(&text).unwrap();
        let template_json = serde_json::to_string(&template).unwrap();
        let media_json = serde_json::to_string(&media).unwrap();
        let location_json = serde_json::to_string(&location).unwrap();

        assert!(text_json.contains("Hello"));
        assert!(template_json.contains("welcome"));
        assert!(media_json.contains("image.jpg"));
        assert!(location_json.contains("40.7128"));
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

    // ==================== List Detection Tests ====================

    /// Helper function to test numbered list item detection
    fn is_numbered_list_item(line: &str) -> bool {
        let trimmed = line.trim();
        let chars: Vec<char> = trimmed.chars().collect();
        let mut i = 0;
        while i < chars.len() && chars[i].is_numeric() {
            i += 1;
        }
        i > 0 && i < chars.len() && (chars[i] == '.' || chars[i] == ')')
            && (i + 1 >= chars.len() || chars[i + 1] == ' ')
    }

    fn is_bullet_list_item(line: &str) -> bool {
        let trimmed = line.trim();
        trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("• ")
    }

    fn is_list_item(line: &str) -> bool {
        is_numbered_list_item(line) || is_bullet_list_item(line)
    }

    fn contains_list(text: &str) -> bool {
        text.lines().any(is_list_item)
    }

    fn looks_like_list_start(text: &str) -> bool {
        let lines: Vec<&str> = text.lines().collect();
        if lines.len() < 2 {
            return false;
        }
        let last_content = lines.iter().rev().find(|l| !l.trim().is_empty());
        if let Some(line) = last_content {
            let trimmed = line.trim();
            if trimmed.len() < 50 && trimmed.ends_with(':') {
                return true;
            }
            if trimmed.chars().next().map(|c| c.is_numeric()).unwrap_or(false) {
                return true;
            }
        }
        false
    }

    fn looks_like_list_end(text: &str) -> bool {
        let lines: Vec<&str> = text.lines().collect();
        if lines.len() < 3 {
            return false;
        }

        // Check if there's at least one list item in the text
        let has_list_items = lines.iter().any(|l| is_list_item(l));
        if !has_list_items {
            return false;
        }

        // Check if the last 2+ non-empty lines are NOT list items
        let non_empty_lines: Vec<&str> = lines.iter().rev()
            .copied()
            .filter(|l| !l.trim().is_empty())
            .take(2)
            .collect();

        if non_empty_lines.len() < 2 {
            return false;
        }

        // If the last 2 non-empty lines are not list items, the list has likely ended
        non_empty_lines.iter().all(|l| !is_list_item(l))
    }

    /// Split text into (before_list, list_and_after)
    /// Returns everything before the first list item, and everything from the list item onwards
    fn split_text_before_list(text: &str) -> (String, String) {
        let lines: Vec<&str> = text.lines().collect();
        let mut list_start_idx = None;

        // Find the first list item
        for (idx, line) in lines.iter().enumerate() {
            if is_list_item(line) {
                list_start_idx = Some(idx);
                break;
            }
        }

        match list_start_idx {
            Some(idx) => {
                let before = lines[..idx].join("\n");
                let rest = lines[idx..].join("\n");
                (before, rest)
            }
            None => (text.to_string(), String::new())
        }
    }

    /// Split text into (list, after_list)
    /// Extracts the list portion and any text after it
    fn split_list_from_text(text: &str) -> (String, String) {
        let lines: Vec<&str> = text.lines().collect();
        let mut list_end_idx = lines.len();

        // Find where the list ends (first non-list item after list starts)
        let mut found_list = false;
        for (idx, line) in lines.iter().enumerate() {
            if is_list_item(line) {
                found_list = true;
            } else if found_list && !line.trim().is_empty() {
                // Found non-empty, non-list line after list items
                list_end_idx = idx;
                break;
            }
        }

        let list = lines[..list_end_idx].join("\n");
        let after = lines[list_end_idx..].join("\n");
        (list, after)
    }

    #[test]
    fn test_numbered_list_detection() {
        // Valid numbered list items
        assert!(is_numbered_list_item("1. Item"));
        assert!(is_numbered_list_item("1. Item with text"));
        assert!(is_numbered_list_item("10. Tenth item"));
        assert!(is_numbered_list_item("1) Item with parenthesis"));
        assert!(is_numbered_list_item("  1. Indented item")); // trim works

        // Invalid - not numbered list items
        assert!(!is_numbered_list_item("Item 1")); // number at end
        assert!(!is_numbered_list_item("2024 was a year")); // year in sentence
        assert!(!is_numbered_list_item("1.Item")); // no space after dot
        assert!(!is_numbered_list_item("Item")); // no number
        assert!(!is_numbered_list_item("")); // empty
    }

    #[test]
    fn test_bullet_list_detection() {
        // Valid bullet list items
        assert!(is_bullet_list_item("- Item"));
        assert!(is_bullet_list_item("* Item"));
        assert!(is_bullet_list_item("• Item"));
        assert!(is_bullet_list_item("  - Indented item"));

        // Invalid
        assert!(!is_bullet_list_item("Item - with dash"));
        assert!(!is_bullet_list_item("-Item")); // no space after dash
    }

    #[test]
    fn test_contains_list() {
        // Contains numbered list
        assert!(contains_list("Some text\n1. First item\n2. Second item"));

        // Contains bullet list
        assert!(contains_list("- Item 1\n- Item 2"));

        // No list
        assert!(!contains_list("Just regular text"));
        assert!(!contains_list("2024 was a great year")); // year should not trigger

        // Mixed content with list
        assert!(contains_list("Here are the options:\n\n1. Option A\n2. Option B"));
    }

    #[test]
    fn test_looks_like_list_start() {
        // Header followed by content looks like list start
        assert!(looks_like_list_start("Aulas Disponíveis:\n\n"));
        assert!(looks_like_list_start("Options:\n\nSome content"));

        // Number at start looks like potential list
        assert!(looks_like_list_start("Some text\n1"));

        // Regular text doesn't look like list start
        assert!(!looks_like_list_start("Just regular text"));
        assert!(!looks_like_list_start("Single line"));
    }

    #[test]
    fn test_full_list_scenario() {
        // Simulate the exact scenario from the bug report
        let content = r#"Aulas Disponíveis:

1. Aula de Violão - Aprenda a tocar violão do básico ao avançado
2. Aula de Piano - Desenvolva suas habilidades no piano
3. Aula de Canto - Técnicas vocais para todos os níveis
4. Aula de Teatro - Expressão corporal e interpretação
5. Aula de Dança - Diversos estilos de dança
6. Aula de Desenho - Técnicas de desenho e pintura
7. Aula de Inglês - Aprenda inglês de forma dinâmica
8. Aula de Robótica - Introdução à robótica e programação

Estou à disposição para ajudar com mais informações!"#;

        // Should detect list
        assert!(contains_list(content), "Should detect numbered list in content");

        // Count list items
        let list_items: Vec<&str> = content.lines().filter(|l| is_list_item(l)).collect();
        assert_eq!(list_items.len(), 8, "Should detect all 8 list items");

        // Verify each item is detected
        for (i, item) in list_items.iter().enumerate() {
            assert!(item.starts_with(&format!("{}.", i + 1)),
                "Item {} should start with '{}.'", i + 1, i + 1);
        }

        // NEW: Should detect that list has ended (content after list)
        assert!(looks_like_list_end(content), "Should detect list has ended");
    }

    #[test]
    fn test_looks_like_list_end() {
        // List with content after - should detect as ended
        let with_content_after = r#"Cursos disponíveis:

1. Ensino Fundamental I
2. Ensino Fundamental II
3. Ensino Médio

Entre em contato para mais informações."#;
        assert!(looks_like_list_end(with_content_after), "Should detect list ended with content after");

        // List with multiple paragraphs after
        let with_multiple_after = r#"Opções:

1. Opção A
2. Opção B
3. Opção C

Texto adicional aqui.

Mais um parágrafo."#;
        assert!(looks_like_list_end(with_multiple_after), "Should detect list ended with multiple paragraphs");

        // List still in progress - should NOT detect as ended
        let in_progress = r#"Cursos:

1. Curso A
2. Curso B
3."#;
        assert!(!looks_like_list_end(in_progress), "Should NOT detect list ended (still in progress)");

        // List with only one line after (need 2+ to confirm end)
        let one_line_after = r#"Cursos:

1. Curso A
2. Curso B

Uma linha apenas."#;
        // This has 2 non-empty lines after (empty line + text), so it should detect as ended
        assert!(looks_like_list_end(one_line_after), "Should detect list ended with 2+ non-empty lines after");

        // No list at all
        let no_list = "Apenas texto normal sem lista.";
        assert!(!looks_like_list_end(no_list), "Should NOT detect list ended (no list present)");

        // List with blank lines after (but no content)
        let list_with_blanks = r#"Lista:

1. Item 1
2. Item 2


"#;
        assert!(!looks_like_list_end(list_with_blanks), "Should NOT detect list ended (only blank lines after)");
    }

    #[test]
    fn test_split_text_before_list() {
        // Text with list in the middle
        let text1 = "Texto antes da lista\n\n1. Primeiro item\n2. Segundo item\n\nTexto depois";
        let (before, rest) = split_text_before_list(text1);
        assert_eq!(before, "Texto antes da lista\n");
        assert!(rest.starts_with("1. Primeiro item"));

        // List at the start (no text before)
        let text2 = "1. Item 1\n2. Item 2";
        let (before, rest) = split_text_before_list(text2);
        assert_eq!(before, "");
        assert_eq!(rest, "1. Item 1\n2. Item 2");

        // No list at all
        let text3 = "Apenas texto sem lista";
        let (before, rest) = split_text_before_list(text3);
        assert_eq!(before, "Apenas texto sem lista");
        assert_eq!(rest, "");

        // Multiple paragraphs before list
        let text4 = "Parágrafo 1\n\nParágrafo 2\n\n1. Item";
        let (before, rest) = split_text_before_list(text4);
        assert_eq!(before, "Parágrafo 1\n\nParágrafo 2\n");
        assert_eq!(rest, "1. Item");

        // Bullet list
        let text5 = "Introdução\n- Item 1\n- Item 2";
        let (before, rest) = split_text_before_list(text5);
        assert_eq!(before, "Introdução");
        assert!(rest.starts_with("- Item 1"));
    }

    #[test]
    fn test_split_list_from_text() {
        // List with text after
        let text1 = "1. Primeiro item\n2. Segundo item\n\nTexto depois da lista";
        let (list, after) = split_list_from_text(text1);
        assert_eq!(list, "1. Primeiro item\n2. Segundo item\n");
        assert_eq!(after, "Texto depois da lista");

        // List at the end (no text after)
        let text2 = "1. Item 1\n2. Item 2";
        let (list, after) = split_list_from_text(text2);
        assert_eq!(list, "1. Item 1\n2. Item 2");
        assert_eq!(after, "");

        // List only
        let text3 = "1. Item";
        let (list, after) = split_list_from_text(text3);
        assert_eq!(list, "1. Item");
        assert_eq!(after, "");

        // List with blank lines after
        let text4 = "1. Item 1\n2. Item 2\n\n\nTexto";
        let (list, after) = split_list_from_text(text4);
        assert_eq!(list, "1. Item 1\n2. Item 2\n\n");
        assert_eq!(after, "Texto");

        // Bullet list with text after
        let text5 = "- Item 1\n- Item 2\n\nConclusão";
        let (list, after) = split_list_from_text(text5);
        assert_eq!(list, "- Item 1\n- Item 2\n");
        assert_eq!(after, "Conclusão");

        // Multiple paragraphs after list
        let text6 = "1. Item\n\nTexto 1\n\nTexto 2";
        let (list, after) = split_list_from_text(text6);
        assert_eq!(list, "1. Item\n");
        assert_eq!(after, "Texto 1\n\nTexto 2");
    }

    #[test]
    fn test_list_isolation_scenario() {
        // Test the complete scenario from zap.md example
        let full_text = r#"Olá! 😊

Infelizmente, não tenho a informação específica sobre o horário de funcionamento da Escola Salesiana no momento.

Para obter essa informação, você pode:
1. *Entrar em contato com a secretaria* - Posso te ajudar
2. *Agendar uma visita* - Assim você conhece a escola

Gostaria que eu te ajudasse?"#;

        // Step 1: Split text before list
        let (text_before, rest) = split_text_before_list(full_text);
        assert!(text_before.contains("Olá!"));
        assert!(text_before.contains("Para obter essa informação, você pode:"));
        assert!(rest.starts_with("1. *Entrar em contato"));

        // Step 2: Split list from text after
        let (list, text_after) = split_list_from_text(&rest);
        assert!(list.starts_with("1. *Entrar em contato"));
        assert!(list.contains("2. *Agendar uma visita"));
        assert!(text_after.contains("Gostaria que eu te ajudasse?"));
    }

    #[test]
    fn test_partial_list_streaming() {
        // Simulate streaming chunks arriving
        let chunks = vec![
            "Aulas Disponíveis:\n\n",
            "1. Aula de Violão\n\n",
            "2. Aula de Piano\n\n",
            "3. Aula de Canto\n\n",
        ];

        let mut buffer = String::new();
        for (i, chunk) in chunks.iter().enumerate() {
            buffer.push_str(chunk);

            // After first chunk, should detect potential list start
            if i == 0 {
                assert!(looks_like_list_start(&buffer) || contains_list(&buffer),
                    "After chunk 0, should detect list start or contain list");
            }

            // After second chunk onwards, should detect list
            if i >= 1 {
                assert!(contains_list(&buffer),
                    "After chunk {}, should detect list", i);
            }
        }
    }
}

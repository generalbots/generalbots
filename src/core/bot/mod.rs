use crate::core::config::ConfigManager;
use crate::drive::drive_monitor::DriveMonitor;
use crate::llm::OpenAIClient;
use crate::shared::models::{BotResponse, UserMessage, UserSession};
use crate::shared::state::AppState;
use axum::extract::ws::{Message, WebSocket};
use axum::{
    extract::{ws::WebSocketUpgrade, Extension, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use diesel::PgConnection;
use futures::{sink::SinkExt, stream::StreamExt};
use log::{error, info, trace, warn};
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::Mutex as AsyncMutex;
use uuid::Uuid;

pub mod channels;
pub mod multimedia;

/// Retrieves the default bot (first active bot) from the database.
pub fn get_default_bot(conn: &mut PgConnection) -> (Uuid, String) {
    use crate::shared::models::schema::bots::dsl::*;
    use diesel::prelude::*;
    match bots
        .filter(is_active.eq(true))
        .select((id, name))
        .first::<(Uuid, String)>(conn)
        .optional()
    {
        Ok(Some((bot_id, bot_name))) => (bot_id, bot_name),
        Ok(None) => {
            warn!("No active bots found, using nil UUID");
            (Uuid::nil(), "default".to_string())
        }
        Err(e) => {
            error!("Failed to query default bot: {}", e);
            (Uuid::nil(), "default".to_string())
        }
    }
}

#[derive(Debug)]
pub struct BotOrchestrator {
    pub state: Arc<AppState>,
    pub mounted_bots: Arc<AsyncMutex<HashMap<String, Arc<DriveMonitor>>>>,
}

impl BotOrchestrator {
    pub fn new(state: Arc<AppState>) -> Self {
        Self {
            state,
            mounted_bots: Arc::new(AsyncMutex::new(HashMap::new())),
        }
    }

    // ... (All existing methods unchanged) ...

    pub async fn mount_all_bots(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // No-op: bot mounting is handled elsewhere
        info!("mount_all_bots called (no-op)");
        Ok(())
    }

    // Stream response to user via LLM
    pub async fn stream_response(
        &self,
        message: UserMessage,
        response_tx: mpsc::Sender<BotResponse>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        trace!(
            "Streaming response for user: {}, session: {}",
            message.user_id,
            message.session_id
        );

        let user_id = Uuid::parse_str(&message.user_id)?;
        let session_id = Uuid::parse_str(&message.session_id)?;
        let bot_id = Uuid::parse_str(&message.bot_id).unwrap_or_default();

        // All database operations in one blocking section
        let (session, context_data, history, model, key, _bot_id_from_config, cache_enabled) = {
            let state_clone = self.state.clone();
            tokio::task::spawn_blocking(
                move || -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
                    // Get session
                    let session = {
                        let mut sm = state_clone.session_manager.blocking_lock();
                        sm.get_session_by_id(session_id)?
                    }
                    .ok_or_else(|| "Session not found")?;

                    // Save user message
                    {
                        let mut sm = state_clone.session_manager.blocking_lock();
                        sm.save_message(session.id, user_id, 1, &message.content, 1)?;
                    }

                    // Get context and history
                    let context_data = {
                        let sm = state_clone.session_manager.blocking_lock();
                        let rt = tokio::runtime::Handle::current();
                        rt.block_on(async {
                            sm.get_session_context_data(&session.id, &session.user_id)
                                .await
                        })?
                    };

                    let history = {
                        let mut sm = state_clone.session_manager.blocking_lock();
                        sm.get_conversation_history(session.id, user_id)?
                    };

                    // Get model config
                    let config_manager = ConfigManager::new(state_clone.conn.clone());
                    let model = config_manager
                        .get_config(&bot_id, "llm-model", Some("gpt-3.5-turbo"))
                        .unwrap_or_else(|_| "gpt-3.5-turbo".to_string());
                    let key = config_manager
                        .get_config(&bot_id, "llm-key", Some(""))
                        .unwrap_or_default();

                    // Check if llm-cache is enabled for this bot
                    let cache_enabled = config_manager
                        .get_config(&bot_id, "llm-cache", Some("true"))
                        .unwrap_or_else(|_| "true".to_string());

                    Ok((
                        session,
                        context_data,
                        history,
                        model,
                        key,
                        bot_id,
                        cache_enabled,
                    ))
                },
            )
            .await??
        };

        // Build messages with bot_id for cache
        let system_prompt = std::env::var("SYSTEM_PROMPT")
            .unwrap_or_else(|_| "You are a helpful assistant.".to_string());
        let mut messages = OpenAIClient::build_messages(&system_prompt, &context_data, &history);

        // Add bot_id and cache config to messages for the cache layer
        if let serde_json::Value::Object(ref mut map) = messages {
            map.insert("bot_id".to_string(), serde_json::json!(bot_id.to_string()));
            map.insert("llm_cache".to_string(), serde_json::json!(cache_enabled));
        } else if let serde_json::Value::Array(_) = messages {
            // If messages is an array, wrap it in an object
            let messages_array = messages.clone();
            messages = serde_json::json!({
                "messages": messages_array,
                "bot_id": bot_id.to_string(),
                "llm_cache": cache_enabled
            });
        }

        // Stream from LLM
        let (stream_tx, mut stream_rx) = mpsc::channel::<String>(100);
        let llm = self.state.llm_provider.clone();

        tokio::spawn(async move {
            if let Err(e) = llm
                .generate_stream("", &messages, stream_tx, &model, &key)
                .await
            {
                error!("LLM streaming error: {}", e);
            }
        });

        let mut full_response = String::new();
        let mut chunk_count = 0;

        while let Some(chunk) = stream_rx.recv().await {
            chunk_count += 1;
            info!("Received LLM chunk #{}: {:?}", chunk_count, chunk);
            full_response.push_str(&chunk);

            let response = BotResponse {
                bot_id: message.bot_id.clone(),
                user_id: message.user_id.clone(),
                session_id: message.session_id.clone(),
                channel: message.channel.clone(),
                content: chunk,
                message_type: 2,
                stream_token: None,
                is_complete: false,
                suggestions: Vec::new(),
                context_name: None,
                context_length: 0,
                context_max_length: 0,
            };

            info!("Sending streaming chunk to WebSocket");
            if let Err(e) = response_tx.send(response).await {
                error!("Failed to send streaming chunk: {}", e);
                break;
            }
        }

        info!(
            "LLM streaming complete, received {} chunks, total length: {}",
            chunk_count,
            full_response.len()
        );

        // Send final complete response
        let final_response = BotResponse {
            bot_id: message.bot_id.clone(),
            user_id: message.user_id.clone(),
            session_id: message.session_id.clone(),
            channel: message.channel.clone(),
            content: full_response.clone(),
            message_type: 2,
            stream_token: None,
            is_complete: true,
            suggestions: Vec::new(),
            context_name: None,
            context_length: 0,
            context_max_length: 0,
        };

        info!("Sending final complete response to WebSocket");
        response_tx.send(final_response).await?;
        info!("Final response sent successfully");

        // Save bot response in blocking context
        let state_for_save = self.state.clone();
        let full_response_clone = full_response.clone();
        tokio::task::spawn_blocking(
            move || -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
                let mut sm = state_for_save.session_manager.blocking_lock();
                sm.save_message(session.id, user_id, 2, &full_response_clone, 2)?;
                Ok(())
            },
        )
        .await??;

        Ok(())
    }

    // ... (Other methods unchanged) ...

    pub async fn get_user_sessions(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<UserSession>, Box<dyn std::error::Error + Send + Sync>> {
        let mut session_manager = self.state.session_manager.lock().await;
        let sessions = session_manager.get_user_sessions(user_id)?;
        Ok(sessions)
    }

    pub async fn get_conversation_history(
        &self,
        session_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<(String, String)>, Box<dyn std::error::Error + Send + Sync>> {
        let mut session_manager = self.state.session_manager.lock().await;
        let history = session_manager.get_conversation_history(session_id, user_id)?;
        Ok(history)
    }

    // ... (Remaining BotOrchestrator methods unchanged) ...
}

/* Axum handlers – placeholders that delegate to BotOrchestrator where appropriate */

/// WebSocket handler that upgrades HTTP connection to WebSocket
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let session_id = params
        .get("session_id")
        .and_then(|s| Uuid::parse_str(s).ok());
    let user_id = params.get("user_id").and_then(|s| Uuid::parse_str(s).ok());

    if session_id.is_none() || user_id.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "session_id and user_id are required" })),
        )
            .into_response();
    }

    ws.on_upgrade(move |socket| {
        handle_websocket(socket, state, session_id.unwrap(), user_id.unwrap())
    })
    .into_response()
}

/// Handles an individual WebSocket connection
async fn handle_websocket(
    socket: WebSocket,
    state: Arc<AppState>,
    session_id: Uuid,
    user_id: Uuid,
) {
    let (mut sender, mut receiver) = socket.split();

    // Create a channel for this WebSocket connection
    let (tx, mut rx) = mpsc::channel::<BotResponse>(100);

    // Register this connection with the web adapter
    state
        .web_adapter
        .add_connection(session_id.to_string(), tx.clone())
        .await;

    // Also register in response_channels for BotOrchestrator
    {
        let mut channels = state.response_channels.lock().await;
        channels.insert(session_id.to_string(), tx.clone());
    }

    info!(
        "WebSocket connected for session: {}, user: {}",
        session_id, user_id
    );

    // Execute start.bas if it exists
    let state_for_start = state.clone();
    let session_for_start = {
        let mut sm = state.session_manager.lock().await;
        sm.get_session_by_id(session_id).ok().and_then(|opt| opt)
    };

    if let Some(session_clone) = session_for_start {
        tokio::task::spawn_blocking(move || {
            use crate::basic::ScriptService;

            let bot_name = "default"; // TODO: Get from session
            let start_script_path =
                format!("./work/{}.gbai/{}.gbdialog/start.bas", bot_name, bot_name);

            if let Ok(start_content) = std::fs::read_to_string(&start_script_path) {
                info!("Executing start.bas for session {}", session_id);
                let script_service = ScriptService::new(state_for_start, session_clone);
                match script_service.compile(&start_content) {
                    Ok(ast) => {
                        if let Err(e) = script_service.run(&ast) {
                            error!("Failed to execute start.bas: {}", e);
                        } else {
                            info!("start.bas executed successfully for session {}", session_id);
                        }
                    }
                    Err(e) => {
                        error!("Failed to compile start.bas: {}", e);
                    }
                }
            } else {
                info!("No start.bas found for bot {}", bot_name);
            }
        });
    }

    // Send initial welcome message
    let welcome = serde_json::json!({
        "type": "connected",
        "session_id": session_id,
        "user_id": user_id,
        "message": "Connected to bot server"
    });

    if let Ok(welcome_str) = serde_json::to_string(&welcome) {
        info!("Sending welcome message to session {}", session_id);
        if let Err(e) = sender.send(Message::Text(welcome_str.into())).await {
            error!("Failed to send welcome message: {}", e);
        }
    }

    // Spawn task to send messages from the channel to the WebSocket
    let mut send_task = tokio::spawn(async move {
        while let Some(response) = rx.recv().await {
            if let Ok(json_str) = serde_json::to_string(&response) {
                if sender.send(Message::Text(json_str.into())).await.is_err() {
                    break;
                }
            }
        }
    });

    // Handle incoming messages from the WebSocket
    let state_clone = state.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            info!("WebSocket received raw message type: {:?}", msg);
            match msg {
                Message::Text(text) => {
                    info!(
                        "Received WebSocket text message (length {}): {}",
                        text.len(),
                        text
                    );
                    match serde_json::from_str::<UserMessage>(&text) {
                        Ok(user_msg) => {
                            info!(
                                "Successfully parsed user message from session: {}, content: {}",
                                session_id, user_msg.content
                            );
                            // Process the message through the bot system
                            if let Err(e) = process_user_message(
                                state_clone.clone(),
                                session_id,
                                user_id,
                                user_msg,
                            )
                            .await
                            {
                                error!("Error processing user message: {}", e);
                            }
                        }
                        Err(e) => {
                            error!(
                                "Failed to parse user message from session {}: {} - Parse error: {}",
                                session_id, text, e
                            );
                        }
                    }
                }
                Message::Close(_) => {
                    info!(
                        "WebSocket close message received for session: {}",
                        session_id
                    );
                    break;
                }
                Message::Ping(_data) => {
                    // Pings are automatically handled by axum
                }
                Message::Pong(_) => {
                    // Pongs are automatically handled by axum
                }
                _ => {}
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = (&mut send_task) => {
            recv_task.abort();
        }
        _ = (&mut recv_task) => {
            send_task.abort();
        }
    }

    // Clean up: remove the connection from the adapter
    state
        .web_adapter
        .remove_connection(&session_id.to_string())
        .await;

    // Also remove from response_channels
    {
        let mut channels = state.response_channels.lock().await;
        channels.remove(&session_id.to_string());
    }

    info!("WebSocket disconnected for session: {}", session_id);
}

/// Process a user message received via WebSocket
async fn process_user_message(
    state: Arc<AppState>,
    session_id: Uuid,
    user_id: Uuid,
    user_msg: UserMessage,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!(
        "Processing message from user {} in session {}: {}",
        user_id, session_id, user_msg.content
    );

    // Get the response channel for this session
    let tx = {
        let channels = state.response_channels.lock().await;
        channels.get(&session_id.to_string()).cloned()
    };

    if let Some(response_tx) = tx {
        // Use BotOrchestrator to stream the response
        let orchestrator = BotOrchestrator::new(state.clone());
        if let Err(e) = orchestrator.stream_response(user_msg, response_tx).await {
            error!("Failed to stream response: {}", e);
        }
    } else {
        error!("No response channel found for session {}", session_id);
    }

    Ok(())
}

/// Create a new bot (placeholder implementation)
pub async fn create_bot_handler(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<HashMap<String, String>>,
) -> impl IntoResponse {
    let bot_name = payload
        .get("bot_name")
        .cloned()
        .unwrap_or_else(|| "default".to_string());

    // Use state to create the bot in the database
    let mut conn = match state.conn.get() {
        Ok(conn) => conn,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Database error: {}", e) })),
            )
        }
    };

    use crate::shared::models::schema::bots::dsl::*;
    use diesel::prelude::*;

    let new_bot = (
        name.eq(&bot_name),
        description.eq(format!("Bot created via API: {}", bot_name)),
        llm_provider.eq("openai"),
        llm_config.eq(serde_json::json!({"model": "gpt-4"})),
        context_provider.eq("none"),
        context_config.eq(serde_json::json!({})),
        is_active.eq(true),
    );

    match diesel::insert_into(bots)
        .values(&new_bot)
        .execute(&mut conn)
    {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": format!("bot '{}' created successfully", bot_name),
                "bot_name": bot_name
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Failed to create bot: {}", e) })),
        ),
    }
}

/// Mount an existing bot (placeholder implementation)
pub async fn mount_bot_handler(
    Extension(_state): Extension<Arc<AppState>>,
    Json(payload): Json<HashMap<String, String>>,
) -> impl IntoResponse {
    let bot_guid = payload.get("bot_guid").cloned().unwrap_or_default();
    (
        StatusCode::OK,
        Json(serde_json::json!({ "status": format!("bot '{}' mounted", bot_guid) })),
    )
}

/// Handle user input for a bot (placeholder implementation)
pub async fn handle_user_input_handler(
    Extension(_state): Extension<Arc<AppState>>,
    Json(payload): Json<HashMap<String, String>>,
) -> impl IntoResponse {
    let session_id = payload.get("session_id").cloned().unwrap_or_default();
    let user_input = payload.get("input").cloned().unwrap_or_default();
    (
        StatusCode::OK,
        Json(
            serde_json::json!({ "status": format!("input '{}' processed for session {}", user_input, session_id) }),
        ),
    )
}

/// Retrieve user sessions (placeholder implementation)
pub async fn get_user_sessions_handler(
    Extension(_state): Extension<Arc<AppState>>,
    Json(_payload): Json<HashMap<String, String>>,
) -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({ "sessions": [] })))
}

/// Retrieve conversation history (placeholder implementation)
pub async fn get_conversation_history_handler(
    Extension(_state): Extension<Arc<AppState>>,
    Json(_payload): Json<HashMap<String, String>>,
) -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({ "history": [] })))
}

/// Send warning (placeholder implementation)
pub async fn send_warning_handler(
    Extension(_state): Extension<Arc<AppState>>,
    Json(_payload): Json<HashMap<String, String>>,
) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(serde_json::json!({ "status": "warning acknowledged" })),
    )
}

use crate::config::ConfigManager;
use crate::drive_monitor::DriveMonitor;
use crate::llm::OpenAIClient;
use crate::llm_models;
use crate::nvidia::get_system_metrics;
use crate::shared::models::{BotResponse, Suggestion, UserMessage, UserSession};
use crate::shared::state::AppState;
use axum::extract::ws::{Message, WebSocket};
use axum::{
    extract::{ws::WebSocketUpgrade, Extension, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use chrono::Utc;
use diesel::PgConnection;
use futures::{sink::SinkExt, stream::StreamExt};
use log::{error, info, trace, warn};
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::Mutex as AsyncMutex;
use tokio::time::Instant;
use uuid::Uuid;

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

    // Placeholder for stream_response used by UI
    pub async fn stream_response(
        &self,
        _user_message: UserMessage,
        _response_tx: mpsc::Sender<BotResponse>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // No-op placeholder
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
                            let state_for_task = state_clone.clone();
                            tokio::spawn(async move {
                                if let Err(e) = process_user_message(
                                    state_for_task,
                                    session_id,
                                    user_id,
                                    user_msg,
                                )
                                .await
                                {
                                    error!("Error processing user message: {}", e);
                                }
                            });
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

    // Get the session from the session manager
    let session = {
        let mut sm = state.session_manager.lock().await;
        sm.get_session_by_id(session_id)
            .map_err(|e| format!("Session error: {}", e))?
            .ok_or("Session not found")?
    };

    let content = user_msg.content.clone();
    let bot_id = session.bot_id;

    info!("Sending message to LLM for processing");

    // Call the LLM to generate a response
    let messages = serde_json::json!([{"role": "user", "content": content}]);
    let llm_response = match state
        .llm_provider
        .generate(&content, &messages, "gpt-3.5-turbo", "")
        .await
    {
        Ok(response) => response,
        Err(e) => {
            error!("LLM generation failed: {}", e);
            format!(
                "I'm sorry, I encountered an error processing your message: {}",
                e
            )
        }
    };

    info!("LLM response received: {}", llm_response);

    // Create and send the bot response
    let response = BotResponse {
        bot_id: bot_id.to_string(),
        user_id: user_id.to_string(),
        session_id: session_id.to_string(),
        channel: "web".to_string(),
        content: llm_response,
        message_type: 2,
        stream_token: None,
        is_complete: true,
        suggestions: Vec::new(),
        context_name: None,
        context_length: 0,
        context_max_length: 0,
    };

    // Send response back through WebSocket
    info!("Sending response to WebSocket session {}", session_id);
    if let Err(e) = state
        .web_adapter
        .send_message_to_session(&session_id.to_string(), response)
        .await
    {
        error!("Failed to send LLM response: {:?}", e);
    } else {
        info!("Response sent successfully to session {}", session_id);
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
    (
        StatusCode::OK,
        Json(serde_json::json!({ "status": format!("bot '{}' created", bot_name) })),
    )
}

/// Mount an existing bot (placeholder implementation)
pub async fn mount_bot_handler(
    Extension(state): Extension<Arc<AppState>>,
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
    Extension(state): Extension<Arc<AppState>>,
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
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<HashMap<String, String>>,
) -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({ "sessions": [] })))
}

/// Retrieve conversation history (placeholder implementation)
pub async fn get_conversation_history_handler(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<HashMap<String, String>>,
) -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({ "history": [] })))
}

/// Send warning (placeholder implementation)
pub async fn send_warning_handler(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<HashMap<String, String>>,
) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(serde_json::json!({ "status": "warning acknowledged" })),
    )
}

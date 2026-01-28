#[cfg(any(feature = "research", feature = "llm"))]
pub mod kb_context;
#[cfg(feature = "llm")]
use crate::core::config::ConfigManager;

#[cfg(feature = "drive")]
use crate::drive::drive_monitor::DriveMonitor;
#[cfg(feature = "llm")]
use crate::llm::llm_models;
#[cfg(feature = "llm")]
use crate::llm::OpenAIClient;
#[cfg(feature = "nvidia")]
use crate::nvidia::get_system_metrics;
use crate::shared::message_types::MessageType;
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
#[cfg(feature = "llm")]
use log::trace;
use log::{error, info, warn};
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::Mutex as AsyncMutex;
use uuid::Uuid;

pub mod channels;
pub mod multimedia;

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

    pub fn mount_all_bots(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("mount_all_bots called");
        Ok(())
    }

    #[cfg(feature = "llm")]
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

        let (session, context_data, history, model, key) = {
            let state_clone = self.state.clone();
            tokio::task::spawn_blocking(
                move || -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
                    let session = {
                        let mut sm = state_clone.session_manager.blocking_lock();
                        sm.get_session_by_id(session_id)?
                    }
                    .ok_or("Session not found")?;

                    {
                        let mut sm = state_clone.session_manager.blocking_lock();
                        sm.save_message(session.id, user_id, 1, &message.content, 1)?;
                    }

                    let context_data = {
                        let sm = state_clone.session_manager.blocking_lock();
                        sm.get_session_context_data(&session.id, &session.user_id)?
                    };

                    let history = {
                        let mut sm = state_clone.session_manager.blocking_lock();
                        sm.get_conversation_history(session.id, user_id)?
                    };

                    let config_manager = ConfigManager::new(state_clone.conn.clone());
                    let model = config_manager
                        .get_config(&session.bot_id, "llm-model", Some("gpt-3.5-turbo"))
                        .unwrap_or_else(|_| "gpt-3.5-turbo".to_string());
                    let key = config_manager
                        .get_config(&session.bot_id, "llm-key", Some(""))
                        .unwrap_or_default();

                    Ok((session, context_data, history, model, key))
                },
            )
            .await??
        };

        let system_prompt = "You are a helpful assistant.".to_string();
        let messages = OpenAIClient::build_messages(&system_prompt, &context_data, &history);

        let (stream_tx, mut stream_rx) = mpsc::channel::<String>(100);
        let llm = self.state.llm_provider.clone();

        let model_clone = model.clone();
        let key_clone = key.clone();
        let messages_clone = messages.clone();
        tokio::spawn(async move {
            if let Err(e) = llm
                .generate_stream("", &messages_clone, stream_tx, &model_clone, &key_clone)
                .await
            {
                error!("LLM streaming error: {}", e);
            }
        });

        let mut full_response = String::new();
        let mut analysis_buffer = String::new();
        let mut in_analysis = false;
        let handler = llm_models::get_handler(&model);

        trace!("Using model handler for {}", model);

        #[cfg(feature = "nvidia")]
        {
            let initial_tokens = crate::shared::utils::estimate_token_count(&context_data);
            let config_manager = ConfigManager::new(self.state.conn.clone());
            let max_context_size = config_manager
                .get_config(&session.bot_id, "llm-server-ctx-size", None)
                .unwrap_or_default()
                .parse::<usize>()
                .unwrap_or(0);

            if let Ok(metrics) = get_system_metrics() {
                eprintln!(
                    "\nNVIDIA: {:.1}% | CPU: {:.1}% | Tokens: {}/{}",
                    metrics.gpu_usage.unwrap_or(0.0),
                    metrics.cpu_usage,
                    initial_tokens,
                    max_context_size
                );
            }
        }

        while let Some(chunk) = stream_rx.recv().await {
            trace!("Received LLM chunk: {:?}", chunk);

            analysis_buffer.push_str(&chunk);

            if !in_analysis && handler.has_analysis_markers(&analysis_buffer) {
                in_analysis = true;
                log::debug!(
                    "Detected start of thinking/analysis content for model {}",
                    model
                );

                let processed = handler.process_content(&analysis_buffer);
                if !processed.is_empty() && processed != analysis_buffer {
                    full_response.push_str(&processed);

                    let response = BotResponse {
                        bot_id: message.bot_id.clone(),
                        user_id: message.user_id.clone(),
                        session_id: message.session_id.clone(),
                        channel: message.channel.clone(),
                        content: processed,
                        message_type: MessageType::BOT_RESPONSE,
                        stream_token: None,
                        is_complete: false,
                        suggestions: Vec::new(),
                        context_name: None,
                        context_length: 0,
                        context_max_length: 0,
                    };

                    if response_tx.send(response).await.is_err() {
                        warn!("Response channel closed");
                        break;
                    }
                }
                continue;
            }

            if in_analysis && handler.is_analysis_complete(&analysis_buffer) {
                in_analysis = false;
                log::debug!(
                    "Detected end of thinking/analysis content for model {}",
                    model
                );

                let processed = handler.process_content(&analysis_buffer);
                if !processed.is_empty() {
                    full_response.push_str(&processed);

                    let response = BotResponse {
                        bot_id: message.bot_id.clone(),
                        user_id: message.user_id.clone(),
                        session_id: message.session_id.clone(),
                        channel: message.channel.clone(),
                        content: processed,
                        message_type: MessageType::BOT_RESPONSE,
                        stream_token: None,
                        is_complete: false,
                        suggestions: Vec::new(),
                        context_name: None,
                        context_length: 0,
                        context_max_length: 0,
                    };

                    if response_tx.send(response).await.is_err() {
                        warn!("Response channel closed");
                        break;
                    }
                }

                analysis_buffer.clear();
                continue;
            }

            if in_analysis {
                trace!("Accumulating thinking content, not sending to user");
                continue;
            }

            if !in_analysis {
                full_response.push_str(&chunk);

                let response = BotResponse {
                    bot_id: message.bot_id.clone(),
                    user_id: message.user_id.clone(),
                    session_id: message.session_id.clone(),
                    channel: message.channel.clone(),
                    content: chunk,
                    message_type: MessageType::BOT_RESPONSE,
                    stream_token: None,
                    is_complete: false,
                    suggestions: Vec::new(),
                    context_name: None,
                    context_length: 0,
                    context_max_length: 0,
                };

                if response_tx.send(response).await.is_err() {
                    warn!("Response channel closed");
                    break;
                }
            }
        }

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

        let final_response = BotResponse {
            bot_id: message.bot_id,
            user_id: message.user_id,
            session_id: message.session_id,
            channel: message.channel,
            content: full_response,
            message_type: MessageType::BOT_RESPONSE,
            stream_token: None,
            is_complete: true,
            suggestions: Vec::new(),
            context_name: None,
            context_length: 0,
            context_max_length: 0,
        };

        response_tx.send(final_response).await?;
        Ok(())
    }

    #[cfg(not(feature = "llm"))]
    pub async fn stream_response(
        &self,
        message: UserMessage,
        response_tx: mpsc::Sender<BotResponse>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        warn!("LLM feature not enabled, cannot stream response");

        let error_response = BotResponse {
            bot_id: message.bot_id,
            user_id: message.user_id,
            session_id: message.session_id,
            channel: message.channel,
            content: "LLM feature is not enabled in this build".to_string(),
            message_type: MessageType::BOT_RESPONSE,
            stream_token: None,
            is_complete: true,
            suggestions: Vec::new(),
            context_name: None,
            context_length: 0,
            context_max_length: 0,
        };

        response_tx.send(error_response).await?;
        Ok(())
    }

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
}

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let session_id = params
        .get("session_id")
        .and_then(|s| Uuid::parse_str(s).ok());
    let user_id = params.get("user_id").and_then(|s| Uuid::parse_str(s).ok());
    let bot_name = params
        .get("bot_name")
        .cloned()
        .unwrap_or_else(|| "default".to_string());

    if session_id.is_none() || user_id.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "session_id and user_id are required" })),
        )
            .into_response();
    }

    let session_id = session_id.unwrap_or_default();
    let user_id = user_id.unwrap_or_default();

    // Look up bot_id from bot_name
    let bot_id = {
        let conn = state.conn.get().ok();
        if let Some(mut db_conn) = conn {
            use crate::shared::models::schema::bots::dsl::*;
            let result: Result<Uuid, _> = bots
                .filter(name.eq(&bot_name))
                .select(id)
                .first(&mut db_conn);
            result.unwrap_or_else(|_| {
                log::warn!("Bot not found: {}, using nil bot_id", bot_name);
                Uuid::nil()
            })
        } else {
            log::warn!("Could not get database connection, using nil bot_id");
            Uuid::nil()
        }
    };

    ws.on_upgrade(move |socket| handle_websocket(socket, state, session_id, user_id, bot_id))
        .into_response()
}

async fn handle_websocket(
    socket: WebSocket,
    state: Arc<AppState>,
    session_id: Uuid,
    user_id: Uuid,
    bot_id: Uuid,
) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::channel::<BotResponse>(100);

    state
        .web_adapter
        .add_connection(session_id.to_string(), tx.clone())
        .await;

    {
        let mut channels = state.response_channels.lock().await;
        channels.insert(session_id.to_string(), tx.clone());
    }

    info!(
        "WebSocket connected for session: {}, user: {}",
        session_id, user_id
    );

    let welcome = serde_json::json!({
        "type": "connected",
        "session_id": session_id,
        "user_id": user_id,
        "bot_id": bot_id,
        "message": "Connected to bot server"
    });

    if let Ok(welcome_str) = serde_json::to_string(&welcome) {
        if sender.send(Message::Text(welcome_str)).await.is_err() {
            error!("Failed to send welcome message");
        }
    }

    let mut send_task = tokio::spawn(async move {
        while let Some(response) = rx.recv().await {
            if let Ok(json_str) = serde_json::to_string(&response) {
                if sender.send(Message::Text(json_str)).await.is_err() {
                    break;
                }
            }
        }
    });

    let state_clone = state.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    info!("Received WebSocket message: {}", text);
                    if let Ok(user_msg) = serde_json::from_str::<UserMessage>(&text) {
                        let orchestrator = BotOrchestrator::new(state_clone.clone());
                        if let Some(tx_clone) = state_clone
                            .response_channels
                            .lock()
                            .await
                            .get(&session_id.to_string())
                        {
                            // Use bot_id from WebSocket connection instead of from message
                            let corrected_msg = UserMessage {
                                bot_id: bot_id.to_string(),
                                ..user_msg
                            };
                            if let Err(e) = orchestrator
                                .stream_response(corrected_msg, tx_clone.clone())
                                .await
                            {
                                error!("Failed to stream response: {}", e);
                            }
                        }
                    }
                }
                Message::Close(_) => {
                    info!("WebSocket close message received");
                    break;
                }
                _ => {}
            }
        }
    });

    tokio::select! {
        _ = (&mut send_task) => { recv_task.abort(); }
        _ = (&mut recv_task) => { send_task.abort(); }
    }

    state
        .web_adapter
        .remove_connection(&session_id.to_string())
        .await;

    {
        let mut channels = state.response_channels.lock().await;
        channels.remove(&session_id.to_string());
    }

    info!("WebSocket disconnected for session: {}", session_id);
}

pub fn create_bot_handler(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<HashMap<String, String>>,
) -> impl IntoResponse {
    let bot_name = payload
        .get("bot_name")
        .cloned()
        .unwrap_or_else(|| "default".to_string());

    let orchestrator = BotOrchestrator::new(state);
    if let Err(e) = orchestrator.mount_all_bots() {
        error!("Failed to mount bots: {}", e);
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({ "status": format!("bot '{}' created", bot_name) })),
    )
}

pub fn mount_bot_handler(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<HashMap<String, String>>,
) -> impl IntoResponse {
    let bot_guid = payload.get("bot_guid").cloned().unwrap_or_default();

    let orchestrator = BotOrchestrator::new(state);
    if let Err(e) = orchestrator.mount_all_bots() {
        error!("Failed to mount bot: {}", e);
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({ "status": format!("bot '{}' mounted", bot_guid) })),
    )
}

pub async fn handle_user_input_handler(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<HashMap<String, String>>,
) -> impl IntoResponse {
    let session_id = payload.get("session_id").cloned().unwrap_or_default();
    let user_input = payload.get("input").cloned().unwrap_or_default();

    info!(
        "Processing user input: {} for session: {}",
        user_input, session_id
    );

    let orchestrator = BotOrchestrator::new(state);
    if let Ok(sessions) = orchestrator.get_user_sessions(Uuid::nil()).await {
        info!("Found {} sessions", sessions.len());
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({ "status": format!("processed: {}", user_input) })),
    )
}

pub async fn get_user_sessions_handler(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<HashMap<String, String>>,
) -> impl IntoResponse {
    let user_id = payload
        .get("user_id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .unwrap_or_else(Uuid::nil);

    let orchestrator = BotOrchestrator::new(state);
    match orchestrator.get_user_sessions(user_id).await {
        Ok(sessions) => (
            StatusCode::OK,
            Json(serde_json::json!({ "sessions": sessions })),
        ),
        Err(e) => {
            error!("Failed to get sessions: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        }
    }
}

pub async fn get_conversation_history_handler(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<HashMap<String, String>>,
) -> impl IntoResponse {
    let session_id = payload
        .get("session_id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .unwrap_or_else(Uuid::nil);
    let user_id = payload
        .get("user_id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .unwrap_or_else(Uuid::nil);

    let orchestrator = BotOrchestrator::new(state);
    match orchestrator
        .get_conversation_history(session_id, user_id)
        .await
    {
        Ok(history) => (
            StatusCode::OK,
            Json(serde_json::json!({ "history": history })),
        ),
        Err(e) => {
            error!("Failed to get history: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        }
    }
}

pub async fn send_warning_handler(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<HashMap<String, String>>,
) -> impl IntoResponse {
    let message = payload
        .get("message")
        .cloned()
        .unwrap_or_else(|| "Warning".to_string());
    let session_id = payload.get("session_id").cloned().unwrap_or_default();

    warn!("Warning for session {}: {}", session_id, message);

    let orchestrator = BotOrchestrator::new(state);
    info!("Orchestrator created for warning");

    if let Ok(sessions) = orchestrator.get_user_sessions(Uuid::nil()).await {
        info!("Current active sessions: {}", sessions.len());
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({ "status": "warning sent", "message": message })),
    )
}

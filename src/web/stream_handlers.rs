//! Server-Sent Events (SSE) streaming handlers for chat responses
//!
//! This module provides real-time streaming of LLM responses using SSE,
//! enabling token-by-token delivery to the client for a responsive chat experience.

use axum::{
    extract::{Query, State},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
    Json,
};
use futures::stream::Stream;
use log::{error, info, trace};
use serde::{Deserialize, Serialize};
use std::{convert::Infallible, sync::Arc, time::Duration};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use uuid::Uuid;

use crate::llm::{LLMProvider, OpenAIClient};
use crate::shared::state::AppState;

/// Request payload for streaming chat
#[derive(Debug, Deserialize)]
pub struct StreamChatRequest {
    /// Session ID
    pub session_id: String,
    /// User message content
    pub message: String,
    /// Optional system prompt override
    pub system_prompt: Option<String>,
    /// Optional model name override
    pub model: Option<String>,
    /// Optional bot ID
    pub bot_id: Option<String>,
}

/// Query parameters for SSE connection
#[derive(Debug, Deserialize)]
pub struct StreamQuery {
    pub session_id: String,
}

/// SSE event types
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum StreamEvent {
    /// Token chunk
    Token { content: String },
    /// Thinking/reasoning content (for models that support it)
    Thinking { content: String },
    /// Tool call request
    ToolCall { name: String, arguments: String },
    /// Error occurred
    Error { message: String },
    /// Stream completed
    Done { total_tokens: Option<u32> },
    /// Stream started
    Start { session_id: String, model: String },
    /// Metadata update
    Meta { key: String, value: String },
}

impl StreamEvent {
    pub fn to_sse_event(&self) -> Result<Event, serde_json::Error> {
        let event_type = match self {
            StreamEvent::Token { .. } => "token",
            StreamEvent::Thinking { .. } => "thinking",
            StreamEvent::ToolCall { .. } => "tool_call",
            StreamEvent::Error { .. } => "error",
            StreamEvent::Done { .. } => "done",
            StreamEvent::Start { .. } => "start",
            StreamEvent::Meta { .. } => "meta",
        };

        let data = serde_json::to_string(self)?;
        Ok(Event::default().event(event_type).data(data))
    }
}

/// Stream a chat response using SSE
pub async fn stream_chat_response(
    State(state): State<AppState>,
    Json(payload): Json<StreamChatRequest>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let (tx, rx) = mpsc::channel::<Result<Event, Infallible>>(100);

    // Clone state for the spawned task
    let state_clone = state.clone();
    let session_id = payload.session_id.clone();
    let message = payload.message.clone();
    let model = payload.model.clone();
    let system_prompt = payload.system_prompt.clone();
    let bot_id = payload
        .bot_id
        .as_ref()
        .and_then(|id| Uuid::parse_str(id).ok());

    // Spawn the streaming task
    tokio::spawn(async move {
        if let Err(e) = handle_stream_generation(
            state_clone,
            tx.clone(),
            session_id,
            message,
            model,
            system_prompt,
            bot_id,
        )
        .await
        {
            error!("Stream generation error: {}", e);
            let error_event = StreamEvent::Error {
                message: e.to_string(),
            };
            if let Ok(event) = error_event.to_sse_event() {
                let _ = tx.send(Ok(event)).await;
            }
        }

        // Send done event
        let done_event = StreamEvent::Done { total_tokens: None };
        if let Ok(event) = done_event.to_sse_event() {
            let _ = tx.send(Ok(event)).await;
        }
    });

    Sse::new(ReceiverStream::new(rx)).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    )
}

/// Handle the actual stream generation
async fn handle_stream_generation(
    state: AppState,
    tx: mpsc::Sender<Result<Event, Infallible>>,
    session_id: String,
    message: String,
    model_override: Option<String>,
    system_prompt_override: Option<String>,
    bot_id: Option<Uuid>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Get LLM configuration
    let (llm_url, llm_model, llm_key) = get_llm_config(&state, bot_id).await?;

    let model = model_override.unwrap_or(llm_model);

    // Send start event
    let start_event = StreamEvent::Start {
        session_id: session_id.clone(),
        model: model.clone(),
    };
    if let Ok(event) = start_event.to_sse_event() {
        tx.send(Ok(event)).await?;
    }

    info!(
        "Starting SSE stream for session: {}, model: {}",
        session_id, model
    );

    // Build messages
    let system_prompt = system_prompt_override.unwrap_or_else(|| {
        "You are a helpful AI assistant powered by General Bots.".to_string()
    });

    let messages = OpenAIClient::build_messages(&system_prompt, "", &[("user".to_string(), message)]);

    // Create LLM client
    let client = OpenAIClient::new(llm_key.clone(), Some(llm_url.clone()));

    // Create channel for token streaming
    let (token_tx, mut token_rx) = mpsc::channel::<String>(100);

    // Spawn LLM streaming task
    let client_clone = client;
    let messages_clone = messages.clone();
    let model_clone = model.clone();
    let key_clone = llm_key.clone();

    tokio::spawn(async move {
        if let Err(e) = client_clone
            .generate_stream(&"", &messages_clone, token_tx, &model_clone, &key_clone)
            .await
        {
            error!("LLM stream error: {}", e);
        }
    });

    // Forward tokens as SSE events
    while let Some(token) = token_rx.recv().await {
        trace!("Streaming token: {}", token);

        let token_event = StreamEvent::Token {
            content: token.clone(),
        };

        if let Ok(event) = token_event.to_sse_event() {
            if tx.send(Ok(event)).await.is_err() {
                // Client disconnected
                info!("Client disconnected from SSE stream");
                break;
            }
        }
    }

    Ok(())
}

/// Get LLM configuration for a bot
async fn get_llm_config(
    state: &AppState,
    bot_id: Option<Uuid>,
) -> Result<(String, String, String), Box<dyn std::error::Error + Send + Sync>> {
    use diesel::prelude::*;

    let mut conn = state
        .conn
        .get()
        .map_err(|e| format!("Failed to acquire database connection: {}", e))?;

    let target_bot_id = bot_id.unwrap_or(Uuid::nil());

    #[derive(QueryableByName)]
    struct ConfigRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        config_key: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        config_value: String,
    }

    let configs: Vec<ConfigRow> = diesel::sql_query(
        "SELECT config_key, config_value FROM bot_configuration \
         WHERE bot_id = $1 AND config_key IN ('llm-url', 'llm-model', 'llm-key')",
    )
    .bind::<diesel::sql_types::Uuid, _>(target_bot_id)
    .load(&mut conn)
    .unwrap_or_default();

    let mut llm_url = "http://localhost:8081".to_string();
    let mut llm_model = "default".to_string();
    let mut llm_key = "none".to_string();

    for config in configs {
        match config.config_key.as_str() {
            "llm-url" => llm_url = config.config_value,
            "llm-model" => llm_model = config.config_value,
            "llm-key" => llm_key = config.config_value,
            _ => {}
        }
    }

    Ok((llm_url, llm_model, llm_key))
}

/// Create routes for streaming endpoints
pub fn routes() -> axum::Router<AppState> {
    use axum::routing::post;

    axum::Router::new()
        .route("/api/chat/stream", post(stream_chat_response))
        .route("/api/v1/stream", post(stream_chat_response))
}

/// Streaming chat with conversation history
#[derive(Debug, Deserialize)]
pub struct StreamChatWithHistoryRequest {
    pub session_id: String,
    pub message: String,
    pub history: Option<Vec<HistoryMessage>>,
    pub system_prompt: Option<String>,
    pub model: Option<String>,
    pub bot_id: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HistoryMessage {
    pub role: String,
    pub content: String,
}

/// Stream chat with full conversation history
pub async fn stream_chat_with_history(
    State(state): State<AppState>,
    Json(payload): Json<StreamChatWithHistoryRequest>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let (tx, rx) = mpsc::channel::<Result<Event, Infallible>>(100);

    let state_clone = state.clone();

    tokio::spawn(async move {
        if let Err(e) =
            handle_stream_with_history(state_clone, tx.clone(), payload).await
        {
            error!("Stream with history error: {}", e);
            let error_event = StreamEvent::Error {
                message: e.to_string(),
            };
            if let Ok(event) = error_event.to_sse_event() {
                let _ = tx.send(Ok(event)).await;
            }
        }

        let done_event = StreamEvent::Done { total_tokens: None };
        if let Ok(event) = done_event.to_sse_event() {
            let _ = tx.send(Ok(event)).await;
        }
    });

    Sse::new(ReceiverStream::new(rx)).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    )
}

async fn handle_stream_with_history(
    state: AppState,
    tx: mpsc::Sender<Result<Event, Infallible>>,
    payload: StreamChatWithHistoryRequest,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let bot_id = payload
        .bot_id
        .as_ref()
        .and_then(|id| Uuid::parse_str(id).ok());

    let (llm_url, llm_model, llm_key) = get_llm_config(&state, bot_id).await?;
    let model = payload.model.unwrap_or(llm_model);

    // Send start event
    let start_event = StreamEvent::Start {
        session_id: payload.session_id.clone(),
        model: model.clone(),
    };
    if let Ok(event) = start_event.to_sse_event() {
        tx.send(Ok(event)).await?;
    }

    // Build history
    let history: Vec<(String, String)> = payload
        .history
        .unwrap_or_default()
        .into_iter()
        .map(|h| (h.role, h.content))
        .chain(std::iter::once(("user".to_string(), payload.message)))
        .collect();

    let system_prompt = payload.system_prompt.unwrap_or_else(|| {
        "You are a helpful AI assistant powered by General Bots.".to_string()
    });

    let messages = OpenAIClient::build_messages(&system_prompt, "", &history);

    let client = OpenAIClient::new(llm_key.clone(), Some(llm_url.clone()));

    let (token_tx, mut token_rx) = mpsc::channel::<String>(100);

    let client_clone = client;
    let messages_clone = messages.clone();
    let model_clone = model.clone();
    let key_clone = llm_key.clone();

    tokio::spawn(async move {
        if let Err(e) = client_clone
            .generate_stream(&"", &messages_clone, token_tx, &model_clone, &key_clone)
            .await
        {
            error!("LLM stream error: {}", e);
        }
    });

    while let Some(token) = token_rx.recv().await {
        let token_event = StreamEvent::Token {
            content: token.clone(),
        };

        if let Ok(event) = token_event.to_sse_event() {
            if tx.send(Ok(event)).await.is_err() {
                break;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_event_to_sse() {
        let event = StreamEvent::Token {
            content: "Hello".to_string(),
        };
        let sse = event.to_sse_event();
        assert!(sse.is_ok());
    }

    #[test]
    fn test_stream_event_done() {
        let event = StreamEvent::Done {
            total_tokens: Some(100),
        };
        let sse = event.to_sse_event();
        assert!(sse.is_ok());
    }

    #[test]
    fn test_stream_event_error() {
        let event = StreamEvent::Error {
            message: "Test error".to_string(),
        };
        let sse = event.to_sse_event();
        assert!(sse.is_ok());
    }

    #[test]
    fn test_history_message_serialization() {
        let msg = HistoryMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
        };
        let json = serde_json::to_string(&msg);
        assert!(json.is_ok());
    }
}

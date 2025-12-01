//! Chat module with Askama templates and business logic migrated from chat.js

use askama::Template;
use askama_axum::IntoResponse;
use axum::{
    extract::{Path, Query, State, WebSocketUpgrade},
    response::Response,
    routing::{get, post},
    Json, Router,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

use crate::shared::state::AppState;

/// Chat page template
#[derive(Template)]
#[template(path = "suite/chat.html")]
pub struct ChatTemplate {
    pub session_id: String,
}

/// Session list template
#[derive(Template)]
#[template(path = "suite/partials/sessions.html")]
struct SessionsTemplate {
    sessions: Vec<SessionItem>,
}

/// Message list template
#[derive(Template)]
#[template(path = "suite/partials/messages.html")]
struct MessagesTemplate {
    messages: Vec<Message>,
}

/// Suggestions template
#[derive(Template)]
#[template(path = "suite/partials/suggestions.html")]
struct SuggestionsTemplate {
    suggestions: Vec<String>,
}

/// Context selector template
#[derive(Template)]
#[template(path = "suite/partials/contexts.html")]
struct ContextsTemplate {
    contexts: Vec<Context>,
    current_context: Option<String>,
}

/// Session item
#[derive(Serialize, Deserialize, Clone)]
struct SessionItem {
    id: String,
    name: String,
    last_message: String,
    timestamp: String,
    active: bool,
}

/// Message
#[derive(Serialize, Deserialize, Clone)]
struct Message {
    id: String,
    session_id: String,
    sender: String,
    content: String,
    timestamp: String,
    is_user: bool,
}

/// Context
#[derive(Serialize, Deserialize, Clone)]
struct Context {
    id: String,
    name: String,
    description: String,
}

/// Chat state
pub struct ChatState {
    sessions: Arc<RwLock<Vec<SessionItem>>>,
    messages: Arc<RwLock<Vec<Message>>>,
    contexts: Arc<RwLock<Vec<Context>>>,
    current_context: Arc<RwLock<Option<String>>>,
    broadcast: broadcast::Sender<WsMessage>,
}

impl ChatState {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1000);
        Self {
            sessions: Arc::new(RwLock::new(vec![SessionItem {
                id: Uuid::new_v4().to_string(),
                name: "Default Session".to_string(),
                last_message: "Welcome to General Bots".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                active: true,
            }])),
            messages: Arc::new(RwLock::new(vec![])),
            contexts: Arc::new(RwLock::new(vec![
                Context {
                    id: "general".to_string(),
                    name: "General".to_string(),
                    description: "General conversation".to_string(),
                },
                Context {
                    id: "technical".to_string(),
                    name: "Technical".to_string(),
                    description: "Technical assistance".to_string(),
                },
                Context {
                    id: "creative".to_string(),
                    name: "Creative".to_string(),
                    description: "Creative writing and ideas".to_string(),
                },
            ])),
            current_context: Arc::new(RwLock::new(None)),
            broadcast: tx,
        }
    }
}

/// WebSocket message types
#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
enum WsMessage {
    Message(Message),
    Typing { session_id: String, user: String },
    StopTyping { session_id: String },
    ContextChanged { context: String },
    SessionSwitched { session_id: String },
}

/// Create chat routes
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/chat/messages", get(get_messages))
        .route("/api/chat/send", post(send_message))
        .route("/api/chat/sessions", get(get_sessions))
        .route("/api/chat/sessions/new", post(create_session))
        .route("/api/chat/sessions/:id", post(switch_session))
        .route("/api/chat/suggestions", get(get_suggestions))
        .route("/api/chat/contexts", get(get_contexts))
        .route("/api/chat/context", post(set_context))
        .route("/api/voice/toggle", post(toggle_voice))
}

/// Chat page handler
pub async fn chat_page(
    State(state): State<AppState>,
    crate::web::auth::AuthenticatedUser { claims }: crate::web::auth::AuthenticatedUser,
) -> impl IntoResponse {
    ChatTemplate {
        session_id: Uuid::new_v4().to_string(),
    }
}

/// Get messages for a session
async fn get_messages(
    Query(params): Query<GetMessagesParams>,
    State(state): State<AppState>,
    crate::web::auth::AuthenticatedUser { .. }: crate::web::auth::AuthenticatedUser,
) -> impl IntoResponse {
    let chat_state = state.extensions.get::<ChatState>().unwrap();
    let messages = chat_state.messages.read().await;

    let session_messages: Vec<Message> = messages
        .iter()
        .filter(|m| m.session_id == params.session_id)
        .cloned()
        .collect();

    MessagesTemplate {
        messages: session_messages,
    }
}

#[derive(Deserialize)]
struct GetMessagesParams {
    session_id: String,
}

/// Send a message
async fn send_message(
    State(state): State<AppState>,
    Json(payload): Json<SendMessagePayload>,
    crate::web::auth::AuthenticatedUser { claims }: crate::web::auth::AuthenticatedUser,
) -> impl IntoResponse {
    let chat_state = state.extensions.get::<ChatState>().unwrap();

    // Create user message
    let user_message = Message {
        id: Uuid::new_v4().to_string(),
        session_id: payload.session_id.clone(),
        sender: claims.name.clone(),
        content: payload.content.clone(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        is_user: true,
    };

    // Store message
    {
        let mut messages = chat_state.messages.write().await;
        messages.push(user_message.clone());
    }

    // Broadcast via WebSocket
    let _ = chat_state
        .broadcast
        .send(WsMessage::Message(user_message.clone()));

    // Simulate bot response (this would call actual LLM service)
    let bot_message = Message {
        id: Uuid::new_v4().to_string(),
        session_id: payload.session_id,
        sender: format!("Bot (for {})", claims.name),
        content: format!("I received: {}", payload.content),
        timestamp: chrono::Utc::now().to_rfc3339(),
        is_user: false,
    };

    // Store bot message
    {
        let mut messages = chat_state.messages.write().await;
        messages.push(bot_message.clone());
    }

    // Broadcast bot message
    let _ = chat_state
        .broadcast
        .send(WsMessage::Message(bot_message.clone()));

    // Return rendered messages
    MessagesTemplate {
        messages: vec![user_message, bot_message],
    }
}

#[derive(Deserialize)]
struct SendMessagePayload {
    session_id: String,
    content: String,
}

/// Get all sessions
async fn get_sessions(
    State(state): State<AppState>,
    crate::web::auth::AuthenticatedUser { .. }: crate::web::auth::AuthenticatedUser,
) -> impl IntoResponse {
    let chat_state = state.extensions.get::<ChatState>().unwrap();
    let sessions = chat_state.sessions.read().await;

    SessionsTemplate {
        sessions: sessions.clone(),
    }
}

/// Create new session
async fn create_session(
    State(state): State<AppState>,
    crate::web::auth::AuthenticatedUser { claims }: crate::web::auth::AuthenticatedUser,
) -> impl IntoResponse {
    let chat_state = state.extensions.get::<ChatState>().unwrap();

    let new_session = SessionItem {
        id: Uuid::new_v4().to_string(),
        name: format!("Chat {}", chrono::Utc::now().format("%H:%M")),
        last_message: String::new(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        active: true,
    };

    let mut sessions = chat_state.sessions.write().await;
    sessions.iter_mut().for_each(|s| s.active = false);
    sessions.insert(0, new_session.clone());

    // Return single session HTML
    format!(
        r##"<div class="session-item active"
             hx-post="/api/chat/sessions/{}"
             hx-target="#messages"
             hx-swap="innerHTML">
            <div class="session-name">{}</div>
            <div class="session-time">{}</div>
        </div>"##,
        new_session.id, new_session.name, new_session.timestamp
    )
}

/// Switch to a different session
async fn switch_session(
    Path(id): Path<String>,
    State(state): State<AppState>,
    crate::web::auth::AuthenticatedUser { .. }: crate::web::auth::AuthenticatedUser,
) -> impl IntoResponse {
    let chat_state = state.extensions.get::<ChatState>().unwrap();

    // Update active session
    {
        let mut sessions = chat_state.sessions.write().await;
        sessions.iter_mut().for_each(|s| {
            s.active = s.id == id;
        });
    }

    // Broadcast session switch
    let _ = chat_state.broadcast.send(WsMessage::SessionSwitched {
        session_id: id.clone(),
    });

    // Return messages for this session
    get_messages(Query(GetMessagesParams { session_id: id }), State(state)).await
}

/// Get suggestions
async fn get_suggestions(State(_state): State<AppState>) -> impl IntoResponse {
    SuggestionsTemplate {
        suggestions: vec![
            "What can you help me with?".to_string(),
            "Tell me about your capabilities".to_string(),
            "How do I get started?".to_string(),
            "Show me an example".to_string(),
        ],
    }
}

/// Get contexts
async fn get_contexts(State(state): State<AppState>) -> impl IntoResponse {
    let chat_state = state.extensions.get::<ChatState>().unwrap();
    let contexts = chat_state.contexts.read().await;
    let current = chat_state.current_context.read().await;

    ContextsTemplate {
        contexts: contexts.clone(),
        current_context: current.clone(),
    }
}

/// Set context
async fn set_context(
    State(state): State<AppState>,
    Json(payload): Json<SetContextPayload>,
) -> impl IntoResponse {
    let chat_state = state.extensions.get::<ChatState>().unwrap();

    {
        let mut current = chat_state.current_context.write().await;
        *current = Some(payload.context_id.clone());
    }

    // Broadcast context change
    let _ = chat_state.broadcast.send(WsMessage::ContextChanged {
        context: payload.context_id,
    });

    Response::builder()
        .header("HX-Trigger", "context-changed")
        .body("".to_string())
        .unwrap()
}

#[derive(Deserialize)]
struct SetContextPayload {
    context_id: String,
}

/// Toggle voice recording
async fn toggle_voice(State(_state): State<AppState>) -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "recording",
        "session_id": Uuid::new_v4().to_string()
    }))
}

/// WebSocket handler for real-time chat
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    crate::web::auth::AuthenticatedUser { claims }: crate::web::auth::AuthenticatedUser,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_chat_socket(socket, state, claims))
}

async fn handle_chat_socket(
    socket: axum::extract::ws::WebSocket,
    state: AppState,
    claims: crate::web::auth::Claims,
) {
    let (mut sender, mut receiver) = socket.split();
    let chat_state = state.extensions.get::<ChatState>().unwrap();
    let mut rx = chat_state.broadcast.subscribe();

    // Spawn task to forward broadcast messages to client
    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if let Ok(json) = serde_json::to_string(&msg) {
                if sender
                    .send(axum::extract::ws::Message::Text(json))
                    .await
                    .is_err()
                {
                    break;
                }
            }
        }
    });

    // Handle incoming messages
    while let Some(msg) = receiver.next().await {
        if let Ok(msg) = msg {
            match msg {
                axum::extract::ws::Message::Text(text) => {
                    // Parse and handle incoming message
                    if let Ok(parsed) = serde_json::from_str::<WsMessage>(&text) {
                        // Broadcast to other clients
                        let _ = chat_state.broadcast.send(parsed);
                    }
                }
                axum::extract::ws::Message::Close(_) => break,
                _ => {}
            }
        }
    }

    // Clean up
    send_task.abort();
}

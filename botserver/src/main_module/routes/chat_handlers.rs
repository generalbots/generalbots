use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::post,
    Form, Router,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use botcore::shared::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ChatMessageRequest {
    pub message: Option<String>,
    pub content: Option<String>,
    pub context: Option<String>,
    pub session_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct NewSessionRequest {
    pub title: Option<String>,
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn render_bot_message(text: &str) -> String {
    format!(
        r#"<div class="message bot"><div class="message-content"><span class="sender">Bot</span><span class="text">{}</span></div></div>"#,
        html_escape(text).replace('\n', "<br>"),
    )
}

pub async fn handle_chat_message(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ChatMessageRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let message = req.message.unwrap_or_default();
    let context = req.context.unwrap_or_default();

    let trimmed = message.trim().to_string();
    if trimmed.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Please enter a message.".to_string()));
    }

    let reply = if let Some(llm) = state.llm_provider.clone() {
        let prompt = if context.is_empty() {
            format!(
                "You are a helpful assistant inside a General Bots workspace. \
                 Answer concisely.\n\nUser: {trimmed}"
            )
        } else {
            format!(
                "You are a helpful assistant. Current app context: {context}. \
                 Answer concisely.\n\nUser: {trimmed}"
            )
        };
        match llm.generate_simple(&prompt).await {
            Ok(answer) => answer,
            Err(e) => {
                log::error!("Chat LLM error: {e}");
                "I'm sorry, I encountered an error while processing your message.".to_string()
            }
        }
    } else {
        format!(
            "I received your message: {trimmed}. Configure an LLM provider to enable AI replies."
        )
    };

    Ok(Json(serde_json::json!({ "reply": reply, "message": reply })))
}

pub async fn handle_chat_message_htmx(
    State(state): State<Arc<AppState>>,
    Form(payload): Form<ChatMessageRequest>,
) -> impl IntoResponse {
    let message = payload
        .content
        .or(payload.message)
        .unwrap_or_default()
        .trim()
        .to_string();

    if message.is_empty() {
        return Html(String::new());
    }

    let reply = if let Some(llm) = state.llm_provider.clone() {
        match llm
            .generate_simple(&format!(
                "You are a helpful assistant in a General Bots workspace. Answer concisely.\n\nUser: {message}"
            ))
            .await
        {
            Ok(answer) => answer,
            Err(e) => {
                log::error!("Chat LLM error: {e}");
                "I'm sorry, I could not process that right now.".to_string()
            }
        }
    } else {
        "No LLM provider configured. Please set one up in bot settings.".to_string()
    };

    Html(render_bot_message(&reply))
}

pub async fn handle_chat_context(
    Form(payload): Form<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let context_id = payload.get("context_id").cloned().unwrap_or_default();
    Html(format!(
        "<span class=\"context-active\" data-context-id=\"{}\">Context set</span>",
        html_escape(&context_id)
    ))
}

pub async fn handle_new_session(
    Form(payload): Form<NewSessionRequest>,
) -> impl IntoResponse {
    let id = Uuid::new_v4();
    let title = payload
        .title
        .unwrap_or_else(|| "New conversation".to_string());
    let html = format!(
        r#"<div class="session-item active" id="session-{id}">
            <div class="session-info"><div class="session-name">{title}</div></div>
            <div class="session-meta"><span class="session-time">just now</span></div>
        </div>"#,
        id = id,
        title = html_escape(&title),
    );
    Html(html)
}

pub async fn handle_sessions_current_message(
    State(state): State<Arc<AppState>>,
    Form(payload): Form<ChatMessageRequest>,
) -> impl IntoResponse {
    let message = payload
        .content
        .or(payload.message)
        .unwrap_or_default()
        .trim()
        .to_string();

    if message.is_empty() {
        return Html(String::new());
    }

    let reply = if let Some(llm) = state.llm_provider.clone() {
        match llm
            .generate_simple(&format!(
                "You are a helpful assistant in a General Bots workspace. Answer concisely.\n\nUser: {message}"
            ))
            .await
        {
            Ok(answer) => answer,
            Err(e) => {
                log::error!("Chat LLM error: {e}");
                "I'm sorry, I could not process that right now.".to_string()
            }
        }
    } else {
        "No LLM provider configured. Please set one up in bot settings.".to_string()
    };

    Html(render_bot_message(&reply))
}

pub fn configure_chat_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/chat", post(handle_chat_message))
        .route("/api/chat/message", post(handle_chat_message_htmx))
        .route("/api/chat/context", post(handle_chat_context))
        .route("/api/chat/sessions/new", post(handle_new_session))
        .route("/api/sessions/current/message", post(handle_sessions_current_message))
}

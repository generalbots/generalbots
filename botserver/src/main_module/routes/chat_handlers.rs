use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::{get, post},
    Form, Router,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use botcore::shared::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ChatModelsQuery {
    /// Bot name from the page the picker belongs to; the tier keys are per bot.
    pub bot: Option<String>,
}

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
        .route("/api/chat/models", get(handle_chat_models))
        .route("/api/chat/sessions/new", post(handle_new_session))
        .route("/api/sessions/current/message", post(handle_sessions_current_message))
}

/// #1637 — reports the model behind each picker tier so the UI can label the
/// options honestly ("Fast · gpt-4o-mini" or "Fast · uses the default model")
/// instead of promising a trade-off the bot has not configured.
pub async fn handle_chat_models(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ChatModelsQuery>,
) -> Json<serde_json::Value> {
    use botcore::shared::schema::bots;
    use diesel::prelude::*;

    let requested = query.bot.unwrap_or_default();
    let bot_uuid = if requested.trim().is_empty() {
        Uuid::nil()
    } else {
        match state.conn.get() {
            Ok(mut conn) => bots::table
                .filter(bots::name.eq(requested.trim()))
                .select(bots::id)
                .first::<Uuid>(&mut conn)
                .unwrap_or(Uuid::nil()),
            Err(e) => {
                log::warn!("chat models: database unavailable: {e}");
                Uuid::nil()
            }
        }
    };

    let cfg = botcore::config::ConfigManager::new(state.conn.clone());
    let mut default_model = cfg
        .get_config(&bot_uuid, "llm-model", Some(""))
        .unwrap_or_default();
    if default_model.is_empty() {
        default_model = std::env::var("LLM_MODEL").unwrap_or_default();
    }

    let tier = |id: &str, label: &str, tradeoff: &str| {
        let key = format!("llm-model-{id}");
        let tier_model = cfg
            .get_config(&bot_uuid, &key, Some(""))
            .unwrap_or_default();
        let inherits_default = tier_model.trim().is_empty();
        let model = if inherits_default {
            default_model.clone()
        } else {
            tier_model
        };
        // A tier is selectable when it has a model of its own OR can inherit
        // the bot's default model. Disabling every tier except "auto" made the
        // picker unusable on single-model deployments.
        let configured = !model.trim().is_empty();
        serde_json::json!({
            "id": id,
            "label": label,
            "model": model,
            "configured": configured,
            "inherits_default": inherits_default,
            "tradeoff": tradeoff,
        })
    };

    Json(serde_json::json!({
        "default_model": default_model,
        "tiers": [
            {
                "id": "auto",
                "label": "Auto",
                "model": default_model,
                "configured": true,
                "tradeoff": "Uses the bot's configured model."
            },
            tier("fast", "Fast", "cheapest, quick"),
            tier("balanced", "Balanced", "middle ground"),
            tier("quality", "Quality", "best, slower"),
        ],
    }))
}

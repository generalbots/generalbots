pub use crate::core::bot::channels::teams::TeamsAdapter;

use crate::core::bot::channels::ChannelAdapter;
use crate::shared::state::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::post, Json, Router};
use diesel::prelude::*;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct TeamsActivity {
    #[serde(rename = "type")]
    pub activity_type: String,
    pub id: String,
    pub timestamp: Option<String>,
    #[serde(rename = "serviceUrl")]
    pub service_url: Option<String>,
    #[serde(rename = "channelId")]
    pub channel_id: Option<String>,
    pub from: TeamsChannelAccount,
    pub conversation: TeamsConversationAccount,
    pub recipient: Option<TeamsChannelAccount>,
    pub text: Option<String>,
    pub value: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct TeamsChannelAccount {
    pub id: String,
    pub name: Option<String>,
    #[serde(rename = "aadObjectId")]
    pub aad_object_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TeamsConversationAccount {
    pub id: String,
    #[serde(rename = "conversationType")]
    pub conversation_type: Option<String>,
    #[serde(rename = "tenantId")]
    pub tenant_id: Option<String>,
    pub name: Option<String>,
}

pub fn configure() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/msteams/messages", post(handle_incoming))
        .route("/api/msteams/send", post(send_message))
}

async fn handle_incoming(
    State(state): State<Arc<AppState>>,
    Json(activity): Json<TeamsActivity>,
) -> impl IntoResponse {
    match activity.activity_type.as_str() {
        "message" => {
            if let Some(text) = &activity.text {
                log::info!(
                    "Teams message from={} conversation={} text={}",
                    activity.from.id,
                    activity.conversation.id,
                    text
                );
            }
            (StatusCode::OK, Json(serde_json::json!({})))
        }
        "conversationUpdate" => {
            log::info!("Teams conversation update id={}", activity.id);
            (StatusCode::OK, Json(serde_json::json!({})))
        }
        "invoke" => {
            log::info!("Teams invoke id={}", activity.id);
            (StatusCode::OK, Json(serde_json::json!({"status": 200})))
        }
        _ => (StatusCode::OK, Json(serde_json::json!({}))),
    }
}

async fn send_message(
    State(state): State<Arc<AppState>>,
    Json(request): Json<serde_json::Value>,
) -> impl IntoResponse {
    let bot_id = get_default_bot_id(&state).await;
    let adapter = TeamsAdapter::new(state.conn.clone(), bot_id);

    let conversation_id = request
        .get("conversation_id")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let message = request
        .get("message")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let response = crate::shared::models::BotResponse {
        bot_id: bot_id.to_string(),
        session_id: conversation_id.to_string(),
        user_id: conversation_id.to_string(),
        channel: "teams".to_string(),
        content: message.to_string(),
        message_type: botlib::MessageType::BOT_RESPONSE,
        stream_token: None,
        is_complete: true,
        suggestions: vec![],
        context_name: None,
        context_length: 0,
        context_max_length: 0,
    };

    match adapter.send_message(response).await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({"success": true}))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"success": false, "error": e.to_string()})),
        ),
    }
}

async fn get_default_bot_id(state: &Arc<AppState>) -> Uuid {
    let conn = state.conn.clone();

    tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().ok()?;
        use crate::shared::models::schema::bots;
        use diesel::prelude::*;

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

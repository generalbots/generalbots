use crate::state::ChannelState;
use crate::session::{find_or_create_session, route_to_attendant, route_to_bot};

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use log::info;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize, Serialize)]
pub struct TeamsActivity {
    #[serde(rename = "type")]
    pub activity_type: String,
    pub id: String,
    #[serde(default)]
    pub timestamp: Option<String>,
    #[serde(rename = "serviceUrl")]
    #[serde(default)]
    pub service_url: Option<String>,
    #[serde(rename = "channelId")]
    #[serde(default)]
    pub channel_id: Option<String>,
    pub from: TeamsChannelAccount,
    pub conversation: TeamsConversationAccount,
    #[serde(default)]
    pub recipient: Option<TeamsChannelAccount>,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub value: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TeamsChannelAccount {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(rename = "aadObjectId")]
    #[serde(default)]
    pub aad_object_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TeamsConversationAccount {
    pub id: String,
    #[serde(rename = "conversationType")]
    #[serde(default)]
    pub conversation_type: Option<String>,
    #[serde(rename = "tenantId")]
    #[serde(default)]
    pub tenant_id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
}

pub fn configure() -> Router<Arc<ChannelState>> {
    Router::new()
        .route("/api/msteams/messages", post(handle_incoming))
        .route("/api/msteams/send", post(crate::handlers::send_message))
}

pub async fn handle_incoming(
    State(state): State<Arc<ChannelState>>,
    Json(activity): Json<TeamsActivity>,
) -> impl IntoResponse {
    match activity.activity_type.as_str() {
        "message" => {
            if let Some(text) = &activity.text {
                let conversation_id = &activity.conversation.id;
                let user_name = activity.from.name.as_deref().unwrap_or("Unknown");

                info!(
                    "Teams message from={} conversation={} text={}",
                    activity.from.id, conversation_id, text
                );

                if let Ok(session) =
                    find_or_create_session(&state, conversation_id, user_name)
                {
                    let assigned_to = session
                        .context_data
                        .get("assigned_to")
                        .and_then(|v| v.as_str());

                    if assigned_to.is_some() {
                        let _ = route_to_attendant(
                            state,
                            &session,
                            text,
                            conversation_id,
                            user_name,
                        );
                    } else {
                        let _ = route_to_bot(state, &session, text, conversation_id).await;
                    }
                }
            }
            (StatusCode::OK, Json(serde_json::json!({})))
        }
        "conversationUpdate" => {
            info!("Teams conversation update id={}", activity.id);
            (StatusCode::OK, Json(serde_json::json!({})))
        }
        "invoke" => {
            info!("Teams invoke id={}", activity.id);
            (
                StatusCode::OK,
                Json(serde_json::json!({"status": 200})),
            )
        }
        _ => (StatusCode::OK, Json(serde_json::json!({}))),
    }
}

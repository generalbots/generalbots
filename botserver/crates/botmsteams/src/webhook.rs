use crate::auth::{self, TeamsCredentials};
use crate::state::ChannelState;
use crate::session::{find_or_create_session, route_to_attendant, route_to_bot};

use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
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

/// Reads the two inbound credentials from the bot configuration. Both are
/// optional in the sense that the gate decides what is required; an empty pair
/// rejects every delivery instead of accepting them (see `auth`).
fn teams_credentials(state: &Arc<ChannelState>) -> TeamsCredentials {
    let mut conn = match state.conn.get() {
        Ok(conn) => conn,
        Err(e) => {
            log::warn!("Teams credentials unavailable, no database connection: {e}");
            return TeamsCredentials::default();
        }
    };

    let bot_id = (state.get_default_bot)(&mut conn).0;
    drop(conn);

    let app_id = (state.get_config)(&bot_id, auth::APP_ID_KEY, None).unwrap_or_default();
    let security_token =
        (state.get_config)(&bot_id, auth::SECURITY_TOKEN_KEY, None).unwrap_or_default();

    TeamsCredentials::new(&app_id, &security_token)
}

pub async fn handle_incoming(
    State(state): State<Arc<ChannelState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    // The credential is checked before the body is parsed: an unsigned POST
    // must not reach the dispatcher at all (#2113). The body is kept as bytes
    // because the outgoing-webhook HMAC covers the exact payload.
    let credentials = teams_credentials(&state);
    if let Err(error) = auth::authorize(&headers, &body, &credentials).await {
        log::warn!("Teams activity rejected: {}", error.reason());
        return (
            error.status(),
            Json(serde_json::json!({ "error": "unauthorized" })),
        )
            .into_response();
    }

    let activity: TeamsActivity = match serde_json::from_slice(&body) {
        Ok(activity) => activity,
        Err(e) => {
            log::info!("Teams activity body is not a valid activity: {e}");
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "error": "invalid activity" })),
            )
                .into_response();
        }
    };

    let response = match activity.activity_type.as_str() {
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
    };

    response.into_response()
}

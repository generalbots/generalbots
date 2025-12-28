pub use crate::core::bot::channels::teams::TeamsAdapter;

use crate::core::bot::channels::ChannelAdapter;
use crate::shared::state::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::post, Json, Router};
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
    State(_state): State<Arc<AppState>>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;
    use std::collections::HashMap;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Activity {
        #[serde(rename = "type")]
        pub kind: String,
        pub id: String,
        pub timestamp: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub local_timestamp: Option<String>,
        #[serde(rename = "serviceUrl")]
        pub service_url: String,
        #[serde(rename = "channelId")]
        pub channel_id: String,
        pub from: ChannelAccount,
        pub conversation: ConversationAccount,
        pub recipient: ChannelAccount,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub text: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none", rename = "textFormat")]
        pub text_format: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub locale: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub attachments: Option<Vec<Attachment>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub entities: Option<Vec<Entity>>,
        #[serde(skip_serializing_if = "Option::is_none", rename = "channelData")]
        pub channel_data: Option<serde_json::Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub action: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none", rename = "replyToId")]
        pub reply_to_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub value: Option<serde_json::Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub name: Option<String>,
    }

    impl Default for Activity {
        fn default() -> Self {
            Self {
                kind: "message".to_string(),
                id: uuid::Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                local_timestamp: None,
                service_url: "https://smba.trafficmanager.net/teams/".to_string(),
                channel_id: "msteams".to_string(),
                from: ChannelAccount::default(),
                conversation: ConversationAccount::default(),
                recipient: ChannelAccount::default(),
                text: None,
                text_format: None,
                locale: None,
                attachments: None,
                entities: None,
                channel_data: None,
                action: None,
                reply_to_id: None,
                value: None,
                name: None,
            }
        }
    }

    #[derive(Debug, Clone, Default, Serialize, Deserialize)]
    pub struct ChannelAccount {
        pub id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none", rename = "aadObjectId")]
        pub aad_object_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub role: Option<String>,
    }

    #[derive(Debug, Clone, Default, Serialize, Deserialize)]
    pub struct ConversationAccount {
        pub id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none", rename = "conversationType")]
        pub conversation_type: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none", rename = "isGroup")]
        pub is_group: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none", rename = "tenantId")]
        pub tenant_id: Option<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Attachment {
        #[serde(rename = "contentType")]
        pub content_type: String,
        #[serde(skip_serializing_if = "Option::is_none", rename = "contentUrl")]
        pub content_url: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub content: Option<serde_json::Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none", rename = "thumbnailUrl")]
        pub thumbnail_url: Option<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Entity {
        #[serde(rename = "type")]
        pub kind: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub mentioned: Option<ChannelAccount>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub text: Option<String>,
        #[serde(flatten)]
        pub additional: HashMap<String, serde_json::Value>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ResourceResponse {
        pub id: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ErrorResponse {
        pub error: ErrorBody,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ErrorBody {
        pub code: String,
        pub message: String,
    }

    fn adaptive_card(content: serde_json::Value) -> Attachment {
        Attachment {
            content_type: "application/vnd.microsoft.card.adaptive".to_string(),
            content_url: None,
            content: Some(content),
            name: None,
            thumbnail_url: None,
        }
    }

    fn hero_card(title: &str, subtitle: Option<&str>, text: Option<&str>) -> Attachment {
        let mut content = serde_json::json!({
            "title": title
        });
        if let Some(s) = subtitle {
            content["subtitle"] = serde_json::json!(s);
        }
        if let Some(t) = text {
            content["text"] = serde_json::json!(t);
        }
        Attachment {
            content_type: "application/vnd.microsoft.card.hero".to_string(),
            content_url: None,
            content: Some(content),
            name: None,
            thumbnail_url: None,
        }
    }

    #[test]
    fn test_activity_default() {
        let activity = Activity::default();
        assert_eq!(activity.kind, "message");
        assert_eq!(activity.channel_id, "msteams");
        assert!(!activity.id.is_empty());
    }

    #[test]
    fn test_activity_serialization() {
        let activity = Activity {
            kind: "message".to_string(),
            id: "test-id".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            local_timestamp: None,
            service_url: "http://localhost".to_string(),
            channel_id: "msteams".to_string(),
            from: ChannelAccount {
                id: "user-1".to_string(),
                name: Some("Test User".to_string()),
                aad_object_id: None,
                role: None,
            },
            conversation: ConversationAccount {
                id: "conv-1".to_string(),
                name: None,
                conversation_type: Some("personal".to_string()),
                is_group: Some(false),
                tenant_id: Some("tenant-1".to_string()),
            },
            recipient: ChannelAccount::default(),
            text: Some("Hello!".to_string()),
            text_format: None,
            locale: None,
            attachments: None,
            entities: None,
            channel_data: None,
            action: None,
            reply_to_id: None,
            value: None,
            name: None,
        };

        let json = serde_json::to_string(&activity).unwrap();
        assert!(json.contains("Hello!"));
        assert!(json.contains("msteams"));
        assert!(json.contains("Test User"));
    }

    #[test]
    fn test_resource_response() {
        let response = ResourceResponse {
            id: "msg-123".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("msg-123"));
    }

    #[test]
    fn test_adaptive_card_helper() {
        let card = adaptive_card(serde_json::json!({
            "type": "AdaptiveCard",
            "body": [{"type": "TextBlock", "text": "Hello"}]
        }));

        assert_eq!(card.content_type, "application/vnd.microsoft.card.adaptive");
        assert!(card.content.is_some());
    }

    #[test]
    fn test_hero_card_helper() {
        let card = hero_card("Title", Some("Subtitle"), Some("Text"));

        assert_eq!(card.content_type, "application/vnd.microsoft.card.hero");
        let content = card.content.unwrap();
        assert_eq!(content["title"], "Title");
    }

    #[test]
    fn test_entity_mention() {
        let entity = Entity {
            kind: "mention".to_string(),
            mentioned: Some(ChannelAccount {
                id: "bot-id".to_string(),
                name: Some("Bot".to_string()),
                aad_object_id: None,
                role: None,
            }),
            text: Some("<at>Bot</at>".to_string()),
            additional: HashMap::new(),
        };

        let json = serde_json::to_string(&entity).unwrap();
        assert!(json.contains("mention"));
        assert!(json.contains("<at>Bot</at>"));
    }

    #[test]
    fn test_error_response() {
        let error = ErrorResponse {
            error: ErrorBody {
                code: "BadRequest".to_string(),
                message: "Invalid activity".to_string(),
            },
        };

        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("BadRequest"));
        assert!(json.contains("Invalid activity"));
    }

    #[test]
    fn test_teams_activity_deserialization() {
        let json = r#"{
            "type": "message",
            "id": "test-123",
            "timestamp": "2024-01-01T00:00:00Z",
            "serviceUrl": "https://smba.trafficmanager.net/teams/",
            "channelId": "msteams",
            "from": {
                "id": "user-1",
                "name": "Test User"
            },
            "conversation": {
                "id": "conv-1",
                "conversationType": "personal"
            },
            "text": "Hello bot!"
        }"#;

        let activity: TeamsActivity = serde_json::from_str(json).unwrap();
        assert_eq!(activity.activity_type, "message");
        assert_eq!(activity.text, Some("Hello bot!".to_string()));
        assert_eq!(activity.from.name, Some("Test User".to_string()));
    }

    #[test]
    fn test_teams_conversation_account() {
        let json = r#"{
            "id": "conv-123",
            "conversationType": "groupChat",
            "tenantId": "tenant-abc",
            "name": "Test Group"
        }"#;

        let conv: TeamsConversationAccount = serde_json::from_str(json).unwrap();
        assert_eq!(conv.id, "conv-123");
        assert_eq!(conv.conversation_type, Some("groupChat".to_string()));
        assert_eq!(conv.tenant_id, Some("tenant-abc".to_string()));
    }

    #[test]
    fn test_teams_channel_account() {
        let json = r#"{
            "id": "user-456",
            "name": "John Doe",
            "aadObjectId": "aad-789"
        }"#;

        let account: TeamsChannelAccount = serde_json::from_str(json).unwrap();
        assert_eq!(account.id, "user-456");
        assert_eq!(account.name, Some("John Doe".to_string()));
        assert_eq!(account.aad_object_id, Some("aad-789".to_string()));
    }

    #[test]
    fn test_invoke_activity() {
        let json = r#"{
            "type": "invoke",
            "id": "invoke-123",
            "serviceUrl": "https://smba.trafficmanager.net/teams/",
            "channelId": "msteams",
            "from": {"id": "user-1"},
            "conversation": {"id": "conv-1"},
            "value": {"action": "submit", "data": {"key": "value"}}
        }"#;

        let activity: TeamsActivity = serde_json::from_str(json).unwrap();
        assert_eq!(activity.activity_type, "invoke");
        assert!(activity.value.is_some());
        let value = activity.value.unwrap();
        assert_eq!(value["action"], "submit");
    }

    #[test]
    fn test_conversation_update_activity() {
        let json = r#"{
            "type": "conversationUpdate",
            "id": "update-123",
            "serviceUrl": "https://smba.trafficmanager.net/teams/",
            "channelId": "msteams",
            "from": {"id": "user-1"},
            "conversation": {"id": "conv-1"}
        }"#;

        let activity: TeamsActivity = serde_json::from_str(json).unwrap();
        assert_eq!(activity.activity_type, "conversationUpdate");
    }
}

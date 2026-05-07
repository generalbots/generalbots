pub mod ws;

pub use ws::handle_slides_websocket;

use crate::types::SlideMessage;
use axum::extract::Path;
use axum::response::IntoResponse;
use axum::Json;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;

pub type SlideChannels = Arc<tokio::sync::RwLock<HashMap<String, broadcast::Sender<SlideMessage>>>>;

static SLIDE_CHANNELS: std::sync::OnceLock<SlideChannels> = std::sync::OnceLock::new();

pub type PresenceMap = Arc<tokio::sync::RwLock<HashMap<String, Vec<UserPresence>>>>;

static PRESENCE: std::sync::OnceLock<PresenceMap> = std::sync::OnceLock::new();

pub type TypingMap = Arc<tokio::sync::RwLock<HashMap<String, Vec<TypingIndicator>>>>;

static TYPING: std::sync::OnceLock<TypingMap> = std::sync::OnceLock::new();

pub type SelectionMap = Arc<tokio::sync::RwLock<HashMap<String, Vec<SelectionInfo>>>>;

static SELECTIONS: std::sync::OnceLock<SelectionMap> = std::sync::OnceLock::new();

pub type MentionMap = Arc<tokio::sync::RwLock<HashMap<String, Vec<MentionNotification>>>>;

static MENTIONS: std::sync::OnceLock<MentionMap> = std::sync::OnceLock::new();

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPresence {
    pub user_id: String,
    pub user_name: String,
    pub user_color: String,
    pub current_slide: Option<usize>,
    pub current_element: Option<String>,
    pub last_active: chrono::DateTime<Utc>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypingIndicator {
    pub user_id: String,
    pub user_name: String,
    pub slide_index: usize,
    pub element_id: String,
    pub started_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionInfo {
    pub user_id: String,
    pub user_name: String,
    pub user_color: String,
    pub slide_index: usize,
    pub element_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MentionNotification {
    pub id: String,
    pub presentation_id: String,
    pub from_user_id: String,
    pub from_user_name: String,
    pub to_user_id: String,
    pub slide_index: usize,
    pub message: String,
    pub created_at: chrono::DateTime<Utc>,
    pub read: bool,
}

pub fn get_slide_channels() -> &'static SlideChannels {
    SLIDE_CHANNELS.get_or_init(|| Arc::new(tokio::sync::RwLock::new(HashMap::new())))
}

pub fn get_presence() -> &'static PresenceMap {
    PRESENCE.get_or_init(|| Arc::new(tokio::sync::RwLock::new(HashMap::new())))
}

pub fn get_typing() -> &'static TypingMap {
    TYPING.get_or_init(|| Arc::new(tokio::sync::RwLock::new(HashMap::new())))
}

pub fn get_selections() -> &'static SelectionMap {
    SELECTIONS.get_or_init(|| Arc::new(tokio::sync::RwLock::new(HashMap::new())))
}

pub fn get_mentions() -> &'static MentionMap {
    MENTIONS.get_or_init(|| Arc::new(tokio::sync::RwLock::new(HashMap::new())))
}

pub async fn handle_get_collaborators(Path(presentation_id): Path<String>) -> impl IntoResponse {
    let presence = get_presence().read().await;
    let users: Vec<&UserPresence> = presence
        .get(&presentation_id)
        .map(|v| v.iter().collect())
        .unwrap_or_default();

    Json(serde_json::json!({
        "count": users.len(),
        "users": users
    }))
}

pub async fn handle_get_presence(
    Path(presentation_id): Path<String>,
) -> impl IntoResponse {
    let presence = get_presence().read().await;
    let users = presence.get(&presentation_id).cloned().unwrap_or_default();
    Json(serde_json::json!({ "users": users }))
}

pub async fn handle_get_typing(
    Path(presentation_id): Path<String>,
) -> impl IntoResponse {
    let typing = get_typing().read().await;
    let indicators = typing.get(&presentation_id).cloned().unwrap_or_default();

    let now = Utc::now();
    let active: Vec<&TypingIndicator> = indicators
        .iter()
        .filter(|t| (now - t.started_at).num_seconds() < 5)
        .collect();

    Json(serde_json::json!({ "typing": active }))
}

pub async fn handle_get_selections(
    Path(presentation_id): Path<String>,
) -> impl IntoResponse {
    let selections = get_selections().read().await;
    let sels = selections.get(&presentation_id).cloned().unwrap_or_default();
    Json(serde_json::json!({ "selections": sels }))
}

pub async fn handle_get_mentions(
    Path(user_id): Path<String>,
) -> impl IntoResponse {
    let mentions = get_mentions().read().await;
    let user_mentions = mentions.get(&user_id).cloned().unwrap_or_default();
    Json(serde_json::json!({ "mentions": user_mentions }))
}

pub async fn broadcast_slide_change(
    presentation_id: &str,
    user_id: &str,
    user_name: &str,
    msg_type: &str,
    slide_index: Option<usize>,
    element_id: Option<&str>,
    data: Option<serde_json::Value>,
) {
    let channels = get_slide_channels().read().await;
    if let Some(tx) = channels.get(presentation_id) {
        let msg = SlideMessage {
            msg_type: msg_type.to_string(),
            presentation_id: presentation_id.to_string(),
            user_id: user_id.to_string(),
            user_name: user_name.to_string(),
            user_color: get_random_color(),
            slide_index,
            element_id: element_id.map(|s| s.to_string()),
            data,
            timestamp: Utc::now(),
        };
        let _ = tx.send(msg);
    }
}

pub async fn mark_mention_read(user_id: &str, mention_id: &str) {
    let mut mentions = get_mentions().write().await;
    if let Some(user_mentions) = mentions.get_mut(user_id) {
        for mention in user_mentions.iter_mut() {
            if mention.id == mention_id {
                mention.read = true;
            }
        }
    }
}

pub async fn clear_user_mentions(user_id: &str) {
    let mut mentions = get_mentions().write().await;
    mentions.remove(user_id);
}

fn get_random_color() -> String {
    use rand::Rng;
    let colors = [
        "#FF6B6B", "#4ECDC4", "#45B7D1", "#96CEB4", "#FFEAA7", "#DDA0DD", "#98D8C8", "#F7DC6F",
        "#BB8FCE", "#85C1E9", "#F1948A", "#82E0AA", "#F8C471", "#AED6F1", "#D7BDE2",
    ];
    let idx = rand::rng().random_range(0..colors.len());
    colors[idx].to_string()
}

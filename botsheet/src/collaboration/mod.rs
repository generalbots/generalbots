mod websocket;

pub use websocket::{handle_sheet_websocket, handle_sheet_connection};

use crate::types::CollabMessage;
use axum::{extract::Path, response::IntoResponse, Json};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;

pub type CollaborationChannels =
    Arc<tokio::sync::RwLock<HashMap<String, broadcast::Sender<CollabMessage>>>>;

static COLLAB_CHANNELS: std::sync::OnceLock<CollaborationChannels> = std::sync::OnceLock::new();

pub type PresenceMap = Arc<tokio::sync::RwLock<HashMap<String, Vec<UserPresence>>>>;

static PRESENCE: std::sync::OnceLock<PresenceMap> = std::sync::OnceLock::new();

pub type TypingMap = Arc<tokio::sync::RwLock<HashMap<String, Vec<TypingIndicator>>>>;

static TYPING: std::sync::OnceLock<TypingMap> = std::sync::OnceLock::new();

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPresence {
    pub user_id: String,
    pub user_name: String,
    pub user_color: String,
    pub current_cell: Option<String>,
    pub current_worksheet: Option<usize>,
    pub last_active: chrono::DateTime<Utc>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypingIndicator {
    pub user_id: String,
    pub user_name: String,
    pub cell: String,
    pub worksheet_index: usize,
    pub started_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionInfo {
    pub user_id: String,
    pub user_name: String,
    pub user_color: String,
    pub start_row: u32,
    pub start_col: u32,
    pub end_row: u32,
    pub end_col: u32,
    pub worksheet_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MentionNotification {
    pub id: String,
    pub sheet_id: String,
    pub from_user_id: String,
    pub from_user_name: String,
    pub to_user_id: String,
    pub cell: String,
    pub message: String,
    pub created_at: chrono::DateTime<Utc>,
    pub read: bool,
}

pub type SelectionMap = Arc<tokio::sync::RwLock<HashMap<String, Vec<SelectionInfo>>>>;

static SELECTIONS: std::sync::OnceLock<SelectionMap> = std::sync::OnceLock::new();

pub type MentionMap = Arc<tokio::sync::RwLock<HashMap<String, Vec<MentionNotification>>>>;

static MENTIONS: std::sync::OnceLock<MentionMap> = std::sync::OnceLock::new();

pub fn get_collab_channels() -> &'static CollaborationChannels {
    COLLAB_CHANNELS.get_or_init(|| Arc::new(tokio::sync::RwLock::new(HashMap::new())))
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

pub fn get_random_color() -> String {
    use rand::Rng;
    let colors = [
        "#FF6B6B", "#4ECDC4", "#45B7D1", "#96CEB4", "#FFEAA7", "#DDA0DD", "#98D8C8", "#F7DC6F",
        "#BB8FCE", "#85C1E9", "#F1948A", "#82E0AA", "#F8C471", "#AED6F1", "#D7BDE2",
    ];
    let idx = rand::rng().random_range(0..colors.len());
    colors[idx].to_string()
}

pub async fn handle_get_collaborators(
    Path(sheet_id): Path<String>,
) -> impl IntoResponse {
    let presence = get_presence().read().await;
    let users: Vec<&UserPresence> = presence
        .get(&sheet_id)
        .map(|v| v.iter().collect())
        .unwrap_or_default();

    Json(serde_json::json!({
        "count": users.len(),
        "users": users
    }))
}

pub async fn handle_get_presence(
    Path(sheet_id): Path<String>,
) -> impl IntoResponse {
    let presence = get_presence().read().await;
    let users = presence.get(&sheet_id).cloned().unwrap_or_default();
    Json(serde_json::json!({ "users": users }))
}

pub async fn handle_get_typing(
    Path(sheet_id): Path<String>,
) -> impl IntoResponse {
    let typing = get_typing().read().await;
    let indicators = typing.get(&sheet_id).cloned().unwrap_or_default();

    let now = Utc::now();
    let active: Vec<&TypingIndicator> = indicators
        .iter()
        .filter(|t| (now - t.started_at).num_seconds() < 5)
        .collect();

    Json(serde_json::json!({ "typing": active }))
}

pub async fn handle_get_selections(
    Path(sheet_id): Path<String>,
) -> impl IntoResponse {
    let selections = get_selections().read().await;
    let sels = selections.get(&sheet_id).cloned().unwrap_or_default();
    Json(serde_json::json!({ "selections": sels }))
}

pub async fn handle_get_mentions(
    Path(user_id): Path<String>,
) -> impl IntoResponse {
    let mentions = get_mentions().read().await;
    let user_mentions = mentions.get(&user_id).cloned().unwrap_or_default();
    Json(serde_json::json!({ "mentions": user_mentions }))
}

pub async fn broadcast_sheet_change(
    sheet_id: &str,
    user_id: &str,
    user_name: &str,
    row: u32,
    col: u32,
    value: &str,
    worksheet_index: usize,
) {
    let channels = get_collab_channels().read().await;
    if let Some(tx) = channels.get(sheet_id) {
        let msg = CollabMessage {
            msg_type: "cell_update".to_string(),
            sheet_id: sheet_id.to_string(),
            user_id: user_id.to_string(),
            user_name: user_name.to_string(),
            user_color: get_random_color(),
            row: Some(row),
            col: Some(col),
            value: Some(value.to_string()),
            worksheet_index: Some(worksheet_index),
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

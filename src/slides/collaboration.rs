use crate::shared::state::AppState;
use crate::slides::types::SlideMessage;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use log::{error, info};
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

pub async fn handle_slides_websocket(
    ws: WebSocketUpgrade,
    Path(presentation_id): Path<String>,
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_slides_connection(socket, presentation_id))
}

async fn handle_slides_connection(socket: WebSocket, presentation_id: String) {
    let (mut sender, mut receiver) = socket.split();

    let channels = get_slide_channels();
    let broadcast_tx = {
        let mut channels_write = channels.write().await;
        channels_write
            .entry(presentation_id.clone())
            .or_insert_with(|| broadcast::channel(100).0)
            .clone()
    };

    let mut broadcast_rx = broadcast_tx.subscribe();

    let user_id = uuid::Uuid::new_v4().to_string();
    let user_id_for_send = user_id.clone();
    let user_name = format!("User {}", &user_id[..8]);
    let user_color = get_random_color();

    {
        let mut presence = get_presence().write().await;
        let users = presence.entry(presentation_id.clone()).or_default();
        users.push(UserPresence {
            user_id: user_id.clone(),
            user_name: user_name.clone(),
            user_color: user_color.clone(),
            current_slide: Some(0),
            current_element: None,
            last_active: Utc::now(),
            status: "active".to_string(),
        });
    }

    let join_msg = SlideMessage {
        msg_type: "join".to_string(),
        presentation_id: presentation_id.clone(),
        user_id: user_id.clone(),
        user_name: user_name.clone(),
        user_color: user_color.clone(),
        slide_index: None,
        element_id: None,
        data: None,
        timestamp: Utc::now(),
    };

    if let Err(e) = broadcast_tx.send(join_msg) {
        error!("Failed to broadcast join: {}", e);
    }

    let broadcast_tx_clone = broadcast_tx.clone();
    let user_id_clone = user_id.clone();
    let presentation_id_clone = presentation_id.clone();
    let user_name_clone = user_name.clone();
    let user_color_clone = user_color.clone();

    let receive_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Ok(mut slide_msg) = serde_json::from_str::<SlideMessage>(&text) {
                        slide_msg.user_id = user_id_clone.clone();
                        slide_msg.user_name = user_name_clone.clone();
                        slide_msg.user_color = user_color_clone.clone();
                        slide_msg.presentation_id = presentation_id_clone.clone();
                        slide_msg.timestamp = Utc::now();

                        match slide_msg.msg_type.as_str() {
                            "slide_change" | "cursor" => {
                                let mut presence = get_presence().write().await;
                                if let Some(users) = presence.get_mut(&presentation_id_clone) {
                                    for user in users.iter_mut() {
                                        if user.user_id == user_id_clone {
                                            user.current_slide = slide_msg.slide_index;
                                            user.current_element = slide_msg.element_id.clone();
                                            user.last_active = Utc::now();
                                        }
                                    }
                                }
                            }
                            "typing_start" => {
                                if let (Some(slide_idx), Some(element_id)) =
                                    (slide_msg.slide_index, &slide_msg.element_id) {
                                    let mut typing = get_typing().write().await;
                                    let indicators = typing.entry(presentation_id_clone.clone()).or_default();
                                    indicators.retain(|t| t.user_id != user_id_clone);
                                    indicators.push(TypingIndicator {
                                        user_id: user_id_clone.clone(),
                                        user_name: user_name_clone.clone(),
                                        slide_index: slide_idx,
                                        element_id: element_id.clone(),
                                        started_at: Utc::now(),
                                    });
                                }
                            }
                            "typing_stop" => {
                                let mut typing = get_typing().write().await;
                                if let Some(indicators) = typing.get_mut(&presentation_id_clone) {
                                    indicators.retain(|t| t.user_id != user_id_clone);
                                }
                            }
                            "selection" => {
                                if let Some(data) = &slide_msg.data {
                                    let mut selections = get_selections().write().await;
                                    let sels = selections.entry(presentation_id_clone.clone()).or_default();
                                    sels.retain(|s| s.user_id != user_id_clone);

                                    if let (Some(slide_idx), Some(element_ids)) = (
                                        data.get("slide_index").and_then(|v| v.as_u64()),
                                        data.get("element_ids").and_then(|v| v.as_array()),
                                    ) {
                                        let ids: Vec<String> = element_ids
                                            .iter()
                                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                            .collect();
                                        sels.push(SelectionInfo {
                                            user_id: user_id_clone.clone(),
                                            user_name: user_name_clone.clone(),
                                            user_color: user_color_clone.clone(),
                                            slide_index: slide_idx as usize,
                                            element_ids: ids,
                                        });
                                    }
                                }
                            }
                            "mention" => {
                                if let Some(data) = &slide_msg.data {
                                    if let (Some(to_user), Some(message), Some(slide_idx)) = (
                                        data.get("to_user_id").and_then(|v| v.as_str()),
                                        data.get("message").and_then(|v| v.as_str()),
                                        data.get("slide_index").and_then(|v| v.as_u64()),
                                    ) {
                                        let mut mentions = get_mentions().write().await;
                                        let user_mentions = mentions.entry(to_user.to_string()).or_default();
                                        user_mentions.push(MentionNotification {
                                            id: uuid::Uuid::new_v4().to_string(),
                                            presentation_id: presentation_id_clone.clone(),
                                            from_user_id: user_id_clone.clone(),
                                            from_user_name: user_name_clone.clone(),
                                            to_user_id: to_user.to_string(),
                                            slide_index: slide_idx as usize,
                                            message: message.to_string(),
                                            created_at: Utc::now(),
                                            read: false,
                                        });
                                    }
                                }
                            }
                            _ => {}
                        }

                        if let Err(e) = broadcast_tx_clone.send(slide_msg) {
                            error!("Failed to broadcast message: {}", e);
                        }
                    }
                }
                Ok(Message::Close(_)) => break,
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }
    });

    let send_task = tokio::spawn(async move {
        while let Ok(msg) = broadcast_rx.recv().await {
            if msg.user_id == user_id_for_send {
                continue;
            }
            if let Ok(json) = serde_json::to_string(&msg) {
                if sender.send(Message::Text(json.into())).await.is_err() {
                    break;
                }
            }
        }
    });

    let presentation_id_leave = presentation_id.clone();
    let user_id_leave = user_id.clone();

    let leave_msg = SlideMessage {
        msg_type: "leave".to_string(),
        presentation_id: presentation_id.clone(),
        user_id: user_id.clone(),
        user_name,
        user_color,
        slide_index: None,
        element_id: None,
        data: None,
        timestamp: Utc::now(),
    };

    tokio::select! {
        _ = receive_task => {}
        _ = send_task => {}
    }

    {
        let mut presence = get_presence().write().await;
        if let Some(users) = presence.get_mut(&presentation_id_leave) {
            users.retain(|u| u.user_id != user_id_leave);
        }
    }

    {
        let mut typing = get_typing().write().await;
        if let Some(indicators) = typing.get_mut(&presentation_id_leave) {
            indicators.retain(|t| t.user_id != user_id_leave);
        }
    }

    {
        let mut selections = get_selections().write().await;
        if let Some(sels) = selections.get_mut(&presentation_id_leave) {
            sels.retain(|s| s.user_id != user_id_leave);
        }
    }

    if let Err(e) = broadcast_tx.send(leave_msg) {
        info!("User left (broadcast may have no receivers): {}", e);
    }
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

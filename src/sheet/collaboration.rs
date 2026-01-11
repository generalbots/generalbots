use crate::shared::state::AppState;
use crate::sheet::types::CollabMessage;
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

pub async fn handle_sheet_websocket(
    ws: WebSocketUpgrade,
    Path(sheet_id): Path<String>,
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_sheet_connection(socket, sheet_id))
}

async fn handle_sheet_connection(socket: WebSocket, sheet_id: String) {
    let (mut sender, mut receiver) = socket.split();

    let channels = get_collab_channels();
    let broadcast_tx = {
        let mut channels_write = channels.write().await;
        channels_write
            .entry(sheet_id.clone())
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
        let users = presence.entry(sheet_id.clone()).or_default();
        users.push(UserPresence {
            user_id: user_id.clone(),
            user_name: user_name.clone(),
            user_color: user_color.clone(),
            current_cell: None,
            current_worksheet: Some(0),
            last_active: Utc::now(),
            status: "active".to_string(),
        });
    }

    let join_msg = CollabMessage {
        msg_type: "join".to_string(),
        sheet_id: sheet_id.clone(),
        user_id: user_id.clone(),
        user_name: user_name.clone(),
        user_color: user_color.clone(),
        row: None,
        col: None,
        value: None,
        worksheet_index: None,
        timestamp: Utc::now(),
    };

    if let Err(e) = broadcast_tx.send(join_msg) {
        error!("Failed to broadcast join: {}", e);
    }

    let broadcast_tx_clone = broadcast_tx.clone();
    let user_id_clone = user_id.clone();
    let sheet_id_clone = sheet_id.clone();
    let user_name_clone = user_name.clone();
    let user_color_clone = user_color.clone();

    let receive_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Ok(mut collab_msg) = serde_json::from_str::<CollabMessage>(&text) {
                        collab_msg.user_id = user_id_clone.clone();
                        collab_msg.user_name = user_name_clone.clone();
                        collab_msg.user_color = user_color_clone.clone();
                        collab_msg.sheet_id = sheet_id_clone.clone();
                        collab_msg.timestamp = Utc::now();

                        match collab_msg.msg_type.as_str() {
                            "cursor" | "cell_select" => {
                                let mut presence = get_presence().write().await;
                                if let Some(users) = presence.get_mut(&sheet_id_clone) {
                                    for user in users.iter_mut() {
                                        if user.user_id == user_id_clone {
                                            if let (Some(row), Some(col)) = (collab_msg.row, collab_msg.col) {
                                                user.current_cell = Some(format!("{},{}", row, col));
                                            }
                                            user.current_worksheet = collab_msg.worksheet_index;
                                            user.last_active = Utc::now();
                                        }
                                    }
                                }
                            }
                            "typing_start" => {
                                if let (Some(row), Some(col), Some(ws_idx)) =
                                    (collab_msg.row, collab_msg.col, collab_msg.worksheet_index) {
                                    let mut typing = get_typing().write().await;
                                    let indicators = typing.entry(sheet_id_clone.clone()).or_default();
                                    indicators.retain(|t| t.user_id != user_id_clone);
                                    indicators.push(TypingIndicator {
                                        user_id: user_id_clone.clone(),
                                        user_name: user_name_clone.clone(),
                                        cell: format!("{},{}", row, col),
                                        worksheet_index: ws_idx,
                                        started_at: Utc::now(),
                                    });
                                }
                            }
                            "typing_stop" => {
                                let mut typing = get_typing().write().await;
                                if let Some(indicators) = typing.get_mut(&sheet_id_clone) {
                                    indicators.retain(|t| t.user_id != user_id_clone);
                                }
                            }
                            "selection" => {
                                if let Some(value) = &collab_msg.value {
                                    if let Ok(sel_data) = serde_json::from_str::<serde_json::Value>(value) {
                                        let mut selections = get_selections().write().await;
                                        let sels = selections.entry(sheet_id_clone.clone()).or_default();
                                        sels.retain(|s| s.user_id != user_id_clone);

                                        if let (Some(sr), Some(sc), Some(er), Some(ec), Some(ws)) = (
                                            sel_data.get("start_row").and_then(|v| v.as_u64()),
                                            sel_data.get("start_col").and_then(|v| v.as_u64()),
                                            sel_data.get("end_row").and_then(|v| v.as_u64()),
                                            sel_data.get("end_col").and_then(|v| v.as_u64()),
                                            sel_data.get("worksheet_index").and_then(|v| v.as_u64()),
                                        ) {
                                            sels.push(SelectionInfo {
                                                user_id: user_id_clone.clone(),
                                                user_name: user_name_clone.clone(),
                                                user_color: user_color_clone.clone(),
                                                start_row: sr as u32,
                                                start_col: sc as u32,
                                                end_row: er as u32,
                                                end_col: ec as u32,
                                                worksheet_index: ws as usize,
                                            });
                                        }
                                    }
                                }
                            }
                            "mention" => {
                                if let Some(value) = &collab_msg.value {
                                    if let Ok(mention_data) = serde_json::from_str::<serde_json::Value>(value) {
                                        if let (Some(to_user), Some(message), Some(cell)) = (
                                            mention_data.get("to_user_id").and_then(|v| v.as_str()),
                                            mention_data.get("message").and_then(|v| v.as_str()),
                                            mention_data.get("cell").and_then(|v| v.as_str()),
                                        ) {
                                            let mut mentions = get_mentions().write().await;
                                            let user_mentions = mentions.entry(to_user.to_string()).or_default();
                                            user_mentions.push(MentionNotification {
                                                id: uuid::Uuid::new_v4().to_string(),
                                                sheet_id: sheet_id_clone.clone(),
                                                from_user_id: user_id_clone.clone(),
                                                from_user_name: user_name_clone.clone(),
                                                to_user_id: to_user.to_string(),
                                                cell: cell.to_string(),
                                                message: message.to_string(),
                                                created_at: Utc::now(),
                                                read: false,
                                            });
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }

                        if let Err(e) = broadcast_tx_clone.send(collab_msg) {
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

    let sheet_id_leave = sheet_id.clone();
    let user_id_leave = user_id.clone();

    let leave_msg = CollabMessage {
        msg_type: "leave".to_string(),
        sheet_id: sheet_id.clone(),
        user_id: user_id.clone(),
        user_name,
        user_color,
        row: None,
        col: None,
        value: None,
        worksheet_index: None,
        timestamp: Utc::now(),
    };

    tokio::select! {
        _ = receive_task => {}
        _ = send_task => {}
    }

    {
        let mut presence = get_presence().write().await;
        if let Some(users) = presence.get_mut(&sheet_id_leave) {
            users.retain(|u| u.user_id != user_id_leave);
        }
    }

    {
        let mut typing = get_typing().write().await;
        if let Some(indicators) = typing.get_mut(&sheet_id_leave) {
            indicators.retain(|t| t.user_id != user_id_leave);
        }
    }

    {
        let mut selections = get_selections().write().await;
        if let Some(sels) = selections.get_mut(&sheet_id_leave) {
            sels.retain(|s| s.user_id != user_id_leave);
        }
    }

    if let Err(e) = broadcast_tx.send(leave_msg) {
        info!("User left (broadcast may have no receivers): {}", e);
    }
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

fn get_random_color() -> String {
    use rand::Rng;
    let colors = [
        "#FF6B6B", "#4ECDC4", "#45B7D1", "#96CEB4", "#FFEAA7", "#DDA0DD", "#98D8C8", "#F7DC6F",
        "#BB8FCE", "#85C1E9", "#F1948A", "#82E0AA", "#F8C471", "#AED6F1", "#D7BDE2",
    ];
    let idx = rand::rng().random_range(0..colors.len());
    colors[idx].to_string()
}

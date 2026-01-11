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
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;

pub type CollaborationChannels =
    Arc<tokio::sync::RwLock<HashMap<String, broadcast::Sender<CollabMessage>>>>;

static COLLAB_CHANNELS: std::sync::OnceLock<CollaborationChannels> = std::sync::OnceLock::new();

pub fn get_collab_channels() -> &'static CollaborationChannels {
    COLLAB_CHANNELS.get_or_init(|| Arc::new(tokio::sync::RwLock::new(HashMap::new())))
}

pub async fn handle_get_collaborators(
    Path(sheet_id): Path<String>,
) -> impl IntoResponse {
    let channels = get_collab_channels().read().await;
    let count = channels.get(&sheet_id).map(|s| s.receiver_count()).unwrap_or(0);
    Json(serde_json::json!({ "count": count }))
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

fn get_random_color() -> String {
    use rand::Rng;
    let colors = [
        "#FF6B6B", "#4ECDC4", "#45B7D1", "#96CEB4", "#FFEAA7", "#DDA0DD", "#98D8C8", "#F7DC6F",
        "#BB8FCE", "#85C1E9",
    ];
    let idx = rand::rng().random_range(0..colors.len());
    colors[idx].to_string()
}

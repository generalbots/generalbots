use super::*;
use crate::ws_auth::{extract_user_from_token, reason_code, unauthorized, WsAuthQuery};
use axum::extract::{
    ws::{Message, WebSocket, WebSocketUpgrade},
    Path, Query, State,
};
use axum::response::{IntoResponse, Response};
use futures_util::{SinkExt, StreamExt};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

static CHANNELS: Lazy<RwLock<HashMap<String, broadcast::Sender<CanvasCollabMessage>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

pub async fn handle_canvas_websocket(
    ws: WebSocketUpgrade,
    Path(canvas_id): Path<String>,
    Query(q): Query<WsAuthQuery>,
    State(_state): State<Arc<CanvasState>>,
) -> Response {
    match extract_user_from_token(&q.token) {
        Ok(identity) => ws
            .on_upgrade(move |socket| handle_canvas_connection(socket, canvas_id, identity))
            .into_response(),
        Err(reason) => {
            log::warn!(
                "canvas ws refused for canvas {canvas_id}: {}",
                reason_code(&reason)
            );
            unauthorized(reason)
        }
    }
}

async fn handle_canvas_connection(
    socket: WebSocket,
    canvas_id: String,
    identity: (String, String),
) {
    let (mut sender, mut receiver) = socket.split();

    let broadcast_tx = {
        let mut channels = CHANNELS.write().await;
        channels
            .entry(canvas_id.clone())
            .or_insert_with(|| broadcast::channel(100).0)
            .clone()
    };
    let mut broadcast_rx = broadcast_tx.subscribe();

    let (user_id, user_name) = identity;
    let user_id_for_send = user_id.clone();
    let user_color = "#4ECDC4".to_string();

    let join_msg = CanvasCollabMessage {
        msg_type: "join".to_string(),
        canvas_id: canvas_id.clone(),
        user_id: user_id.clone(),
        user_name: user_name.clone(),
        user_color: user_color.clone(),
        elements: None,
        timestamp: chrono::Utc::now(),
    };
    let _ = broadcast_tx.send(join_msg);

    let broadcast_tx_clone = broadcast_tx.clone();
    let user_id_clone = user_id.clone();
    let user_name_clone = user_name.clone();
    let user_color_clone = user_color.clone();
    let canvas_id_clone = canvas_id.clone();

    let receive_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Ok(mut collab_msg) = serde_json::from_str::<CanvasCollabMessage>(&text) {
                        collab_msg.user_id = user_id_clone.clone();
                        collab_msg.user_name = user_name_clone.clone();
                        collab_msg.user_color = user_color_clone.clone();
                        collab_msg.canvas_id = canvas_id_clone.clone();
                        collab_msg.timestamp = chrono::Utc::now();
                        let _ = broadcast_tx_clone.send(collab_msg);
                    }
                }
                Ok(Message::Close(_)) => break,
                Err(e) => {
                    log::error!("Canvas WebSocket error: {e}");
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
                if sender.send(Message::Text(json)).await.is_err() {
                    break;
                }
            }
        }
    });

    tokio::select! {
        _ = receive_task => {}
        _ = send_task => {}
    }

    let leave_msg = CanvasCollabMessage {
        msg_type: "leave".to_string(),
        canvas_id: canvas_id.clone(),
        user_id: user_id.clone(),
        user_name: user_name.clone(),
        user_color: user_color.clone(),
        elements: None,
        timestamp: chrono::Utc::now(),
    };
    let _ = broadcast_tx.send(leave_msg);
}

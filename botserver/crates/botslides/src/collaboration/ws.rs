use crate::collaboration::{
    get_mentions, get_presence, get_selections, get_slide_channels,
    get_typing, MentionNotification, SelectionInfo, TypingIndicator, UserPresence,
};
use crate::storage::DriveOps;
use crate::types::SlideMessage;
use crate::SlidesState;
use axum::extract::{Path, State, ws::{Message, WebSocket, WebSocketUpgrade}};
use axum::response::IntoResponse;
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;

pub async fn handle_slides_websocket<D: DriveOps>(
    ws: WebSocketUpgrade,
    Path(presentation_id): Path<String>,
    State(_state): State<Arc<SlidesState<D>>>,
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
            .or_insert_with(|| tokio::sync::broadcast::channel(100).0)
            .clone()
    };

    let mut broadcast_rx = broadcast_tx.subscribe();

    let user_id = uuid::Uuid::new_v4().to_string();
    let user_id_for_send = user_id.clone();
    let user_name = format!("User {}", &user_id[..8]);
    let user_color = {
        use rand::Rng;
        let colors = [
            "#FF6B6B", "#4ECDC4", "#45B7D1", "#96CEB4", "#FFEAA7", "#DDA0DD",
        ];
        let idx = rand::rng().random_range(0..colors.len());
        colors[idx].to_string()
    };

    {
        let mut presence = get_presence().write().await;
        let users = presence.entry(presentation_id.clone()).or_default();
        users.push(UserPresence {
            user_id: user_id.clone(),
            user_name: user_name.clone(),
            user_color: user_color.clone(),
            current_slide: Some(0),
            current_element: None,
            last_active: chrono::Utc::now(),
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
        timestamp: chrono::Utc::now(),
    };

    if let Err(e) = broadcast_tx.send(join_msg) {
        log::error!("Failed to broadcast join: {}", e);
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
                        slide_msg.timestamp = chrono::Utc::now();

                        match slide_msg.msg_type.as_str() {
                            "slide_change" | "cursor" => {
                                let mut presence = get_presence().write().await;
                                if let Some(users) = presence.get_mut(&presentation_id_clone) {
                                    for user in users.iter_mut() {
                                        if user.user_id == user_id_clone {
                                            user.current_slide = slide_msg.slide_index;
                                            user.current_element = slide_msg.element_id.clone();
                                            user.last_active = chrono::Utc::now();
                                        }
                                    }
                                }
                            }
                            "typing_start" => {
                                if let (Some(slide_idx), Some(element_id)) =
                                    (slide_msg.slide_index, &slide_msg.element_id)
                                {
                                    let mut typing = get_typing().write().await;
                                    let indicators =
                                        typing.entry(presentation_id_clone.clone()).or_default();
                                    indicators.retain(|t| t.user_id != user_id_clone);
                                    indicators.push(TypingIndicator {
                                        user_id: user_id_clone.clone(),
                                        user_name: user_name_clone.clone(),
                                        slide_index: slide_idx,
                                        element_id: element_id.clone(),
                                        started_at: chrono::Utc::now(),
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
                                    let mut sels_map = get_selections().write().await;
                                    let sels =
                                        sels_map.entry(presentation_id_clone.clone()).or_default();
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
                                        let user_mentions =
                                            mentions.entry(to_user.to_string()).or_default();
                                        user_mentions.push(MentionNotification {
                                            id: uuid::Uuid::new_v4().to_string(),
                                            presentation_id: presentation_id_clone.clone(),
                                            from_user_id: user_id_clone.clone(),
                                            from_user_name: user_name_clone.clone(),
                                            to_user_id: to_user.to_string(),
                                            slide_index: slide_idx as usize,
                                            message: message.to_string(),
                                            created_at: chrono::Utc::now(),
                                            read: false,
                                        });
                                    }
                                }
                            }
                            _ => {}
                        }

                        if let Err(e) = broadcast_tx_clone.send(slide_msg) {
                            log::error!("Failed to broadcast message: {}", e);
                        }
                    }
                }
                Ok(Message::Close(_)) => break,
                Err(e) => {
                    log::error!("WebSocket error: {}", e);
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
        timestamp: chrono::Utc::now(),
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
        let mut sels = get_selections().write().await;
        if let Some(s) = sels.get_mut(&presentation_id_leave) {
            s.retain(|s| s.user_id != user_id_leave);
        }
    }

    if let Err(e) = broadcast_tx.send(leave_msg) {
        log::info!("User left (broadcast may have no receivers): {}", e);
    }
}

use super::{
    ensure_sweep_started, get_collab_channels, get_presence, get_random_color, get_selections,
    get_typing, get_mentions, MentionNotification, SelectionInfo, TypingIndicator, UserPresence,
};
use crate::state::SheetState;
use crate::types::CollabMessage;
use std::sync::Arc;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, Query, State,
    },
    response::IntoResponse,
};
use botsheet_core::formulas::cell_to_a1;
use chrono::Utc;
use futures_util::{SinkExt, StreamExt};

use tokio::sync::broadcast;

mod websocket_auth;

use self::websocket_auth::{extract_user_from_token, WsAuthQuery};

pub async fn handle_sheet_websocket(
    ws: WebSocketUpgrade,
    State(state): State<Arc<SheetState>>,
    Path(sheet_id): Path<String>,
    Query(q): Query<WsAuthQuery>,
) -> impl IntoResponse {
    let auth = extract_user_from_token(&q.token);
    ws.on_upgrade(move |socket| handle_sheet_connection(socket, state, sheet_id, auth))
}

pub async fn handle_sheet_connection(
    socket: WebSocket,
    state: Arc<SheetState>,
    sheet_id: String,
    auth: Option<(String, String)>,
) {
    ensure_sweep_started();
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

    let (user_id, user_name) = match auth {
        Some((id, name)) => (id, name),
        None => {
            let id = uuid::Uuid::new_v4().to_string();
            (id.clone(), format!("Guest {}", &id[..8]))
        }
    };
    let user_id_for_send = user_id.clone();
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
        seq: None,
        timestamp: Utc::now(),
    };

    if let Err(e) = broadcast_tx.send(join_msg) {
        log::error!("Failed to broadcast join: {e}");
    }

    let broadcast_tx_clone = broadcast_tx.clone();
    let user_id_clone = user_id.clone();
    let sheet_id_clone = sheet_id.clone();
    let user_name_clone = user_name.clone();
    let user_color_clone = user_color.clone();
    let state_clone = Arc::clone(&state);
    
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
                                            if let (Some(row), Some(col)) =
                                                (collab_msg.row, collab_msg.col)
                                            {
                                                user.current_cell = cell_to_a1(row, col);
                                            }
                                            user.current_worksheet = collab_msg.worksheet_index;
                                            user.last_active = Utc::now();
                                        }
                                    }
                                }
                            }
                            "typing_start" => {
                                if let (Some(row), Some(col), Some(ws_idx)) = (
                                    collab_msg.row,
                                    collab_msg.col,
                                    collab_msg.worksheet_index,
                                ) {
                                    let mut typing = get_typing().write().await;
                                    let indicators =
                                        typing.entry(sheet_id_clone.clone()).or_default();
                                    indicators.retain(|t| t.user_id != user_id_clone);
                                    indicators.push(TypingIndicator {
                                        user_id: user_id_clone.clone(),
                                        user_name: user_name_clone.clone(),
                                        cell: cell_to_a1(row, col)
                                            .unwrap_or_else(|| format!("{row},{col}")),
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
                                    if let Ok(sel_data) =
                                        serde_json::from_str::<serde_json::Value>(value)
                                    {
                                        let mut selections = get_selections().write().await;
                                        let sels =
                                            selections.entry(sheet_id_clone.clone()).or_default();
                                        sels.retain(|s| s.user_id != user_id_clone);

                                        if let (Some(sr), Some(sc), Some(er), Some(ec), Some(ws)) = (
                                            sel_data.get("start_row").and_then(|v| v.as_u64()),
                                            sel_data.get("start_col").and_then(|v| v.as_u64()),
                                            sel_data.get("end_row").and_then(|v| v.as_u64()),
                                            sel_data.get("end_col").and_then(|v| v.as_u64()),
                                            sel_data
                                                .get("worksheet_index")
                                                .and_then(|v| v.as_u64()),
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
                                    if let Ok(mention_data) =
                                        serde_json::from_str::<serde_json::Value>(value)
                                    {
                                        if let (Some(to_user), Some(message), Some(cell)) = (
                                            mention_data
                                                .get("to_user_id")
                                                .and_then(|v| v.as_str()),
                                            mention_data
                                                .get("message")
                                                .and_then(|v| v.as_str()),
                                            mention_data
                                                .get("cell")
                                                .and_then(|v| v.as_str()),
                                        ) {
                                            let mut mentions = get_mentions().write().await;
                                            let user_mentions =
                                                mentions.entry(to_user.to_string()).or_default();
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
                            // Server-authoritative edit (#791): the client sends
                            // intent (row/col/content), the session applies it,
                            // records an oplog entry and stamps the message with
                            // the assigned sequence. Other clients order their
                            // local state by `seq` and use it as the catch-up
                            // cursor if they missed anything.
                            "edit" | "cell_update" => {
                                if let (Some(row), Some(col), Some(content)) =
                                    (collab_msg.row, collab_msg.col, collab_msg.value.as_ref())
                                {
                                    let session = match state_clone
                                        .sessions
                                        .get_or_load(&state_clone, &user_id_clone, &sheet_id_clone)
                                        .await
                                    {
                                        Ok(s) => s,
                                        Err(e) => {
                                            log::warn!(
                                                "collab edit on unknown sheet {}: {e}",
                                                sheet_id_clone
                                            );
                                            continue;
                                        }
                                    };
                                    {
                                        let mut sheet = session.sheet.write().await;
                                        let ws_idx = collab_msg
                                            .worksheet_index
                                            .unwrap_or(0)
                                            .min(sheet.worksheets.len().saturating_sub(1));
                                        let key = format!("{row},{col}");
                                        let entry = sheet.worksheets[ws_idx]
                                            .data
                                            .entry(key)
                                            .or_insert_with(|| crate::types::CellData {
                                                value: None,
                                                typed: None,
                                                formula: None,
                                                style: None,
                                                format: None,
                                                note: None,
                                                locked: None,
                                                has_comment: None,
                                                array_formula_id: None,
                                            });
                                        entry.value = Some(content.clone());
                                        entry.typed = Some(
                                            botsheet_core::engine::CellValue::parse(content),
                                        );
                                        sheet.updated_at = Utc::now();
                                    }
                                    // Recalculate dependents through the cached dependency graph
                                    // (#784). Everything happens under one
                                    // short blocking write guard — no awaits in
                                    // this path, so the spawn future stays Send.
                                    let changed = (row, col);
                                    {
                                        let mut graphs = match session.dep_graphs.lock() {
                                            Ok(guard) => guard,
                                            Err(poisoned) => poisoned.into_inner(),
                                        };
                                        let index = collab_msg
                                            .worksheet_index
                                            .unwrap_or(0)
                                            .min(graphs.len().saturating_sub(1));
                                        let mut sheet = session.sheet.blocking_write();
                                        let worksheet = &mut sheet.worksheets[index];
                                        graphs[index].on_edit(worksheet, &[changed]);
                                        graphs[index].recalc_cascade_typed(
                                            worksheet,
                                            changed,
                                            1000,
                                        );
                                    }
                                    let seq = state_clone.sessions.record(
                                        &session,
                                        &state_clone,
                                        &user_id_clone,
                                        "cell_update",
                                        serde_json::json!({
                                            "row": row,
                                            "col": col,
                                            "value": content,
                                            "worksheet_index":
                                                collab_msg.worksheet_index.unwrap_or(0),
                                        }),
                                    );
                                    collab_msg.msg_type = "cell_update".to_string();
                                    collab_msg.seq = Some(seq);
                                }
                            }
                            _ => {}
                        }

                        if let Err(e) = broadcast_tx_clone.send(collab_msg) {
                            log::error!("Failed to broadcast message: {e}");
                        }
                    }
                }
                Ok(Message::Close(_)) => break,
                Err(e) => {
                    log::error!("WebSocket error: {e}");
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
        seq: None,
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

    let _ = broadcast_tx.send(leave_msg);
}

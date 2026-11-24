use super::guard::{target_in_room, ParticipantIdentity};
use super::{
    MAX_PARTICIPANTS_PER_ROOM, OutgoingSignal, ParticipantEntry, ParticipantSocket, SignalMessage,
    SignalingRooms,
};
use axum::extract::ws::{Message, WebSocket};
use botcore::shared::state::AppState;
use futures::{SinkExt, StreamExt};
use log::{info, warn};
use std::sync::Arc;

fn serialize_signal(signal: &OutgoingSignal) -> String {
    serde_json::to_string(signal).unwrap_or_else(|e| {
        log::warn!("Failed to serialize signaling message: {e}");
        String::new()
    })
}

/// Backwards-compatible entry point retained for callers that do not yet
/// provide a verified identity. It is **not** routed: `configure.rs` uses
/// `authenticated_signaling_websocket`. With no identity, every `Join` is
/// refused, so this path can never grant room access.
pub async fn handle_signaling_socket(
    socket: WebSocket,
    room_id: String,
    state: Arc<AppState>,
    rooms: SignalingRooms,
) {
    handle_signaling_socket_inner(socket, room_id, state, rooms, None).await;
}

pub(crate) async fn handle_authenticated_signaling_socket(
    socket: WebSocket,
    room_id: String,
    state: Arc<AppState>,
    rooms: SignalingRooms,
    identity: ParticipantIdentity,
) {
    handle_signaling_socket_inner(socket, room_id, state, rooms, Some(identity)).await;
}

async fn handle_signaling_socket_inner(
    socket: WebSocket,
    room_id: String,
    state: Arc<AppState>,
    rooms: SignalingRooms,
    identity: Option<ParticipantIdentity>,
) {
    let (mut ws_sender, mut ws_receiver) = socket.split();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();
    let mut pid: Option<String> = None;
    let mut pname: Option<String> = None;

    let write_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    while let Some(msg_result) = ws_receiver.next().await {
        let text = match msg_result {
            Ok(Message::Text(t)) => t.to_string(),
            Ok(Message::Close(_)) => break,
            Err(e) => {
                warn!("WS error room {room_id}: {e}");
                break;
            }
            _ => continue,
        };

        let signal: SignalMessage = match serde_json::from_str(&text) {
            Ok(s) => s,
            Err(e) => {
                warn!("Invalid signal room {room_id}: {e}");
                if let Err(e) = tx.send(
                    serialize_signal(&OutgoingSignal::Error {
                        message: format!("Invalid: {e}"),
                    }),
                ) {
                    log::warn!("Signaling room {room_id}: failed to send error signal: {e}");
                }
                continue;
            }
        };

        match signal {
            SignalMessage::Join { room_id: jr, .. } => {
                if jr != room_id {
                    if let Err(e) = tx.send(
                        serialize_signal(&OutgoingSignal::Error {
                            message: "Room ID mismatch".into(),
                        }),
                    ) {
                        log::warn!("Signaling room {room_id}: failed to send room-mismatch error: {e}");
                    }
                    continue;
                }
                // The client-supplied participant_id/participant_name are
                // deliberately ignored (see `..` above): identity comes only
                // from the verified user bound at the WebSocket upgrade.
                let (new_pid, new_pname) = match &identity {
                    Some(identity) => (identity.id.clone(), identity.name.clone()),
                    None => {
                        warn!("Rejected Join on room {room_id}: no verified identity");
                        if let Err(e) = tx.send(
                            serialize_signal(&OutgoingSignal::Error {
                                message: "Unauthenticated signaling connection".into(),
                            }),
                        ) {
                            log::warn!("Signaling room {room_id}: failed to send auth error: {e}");
                        }
                        continue;
                    }
                };
                let mut guard = rooms.write().await;
                let room = guard.entry(room_id.clone()).or_default();
                if room.participants.len() >= MAX_PARTICIPANTS_PER_ROOM {
                    drop(guard);
                    if let Err(e) = tx.send(
                        serialize_signal(&OutgoingSignal::Error {
                            message: format!("Room full (max {MAX_PARTICIPANTS_PER_ROOM})"),
                        }),
                    ) {
                        log::warn!("Signaling room {room_id}: failed to send room-full error: {e}");
                    }
                    continue;
                }
                let existing: Vec<ParticipantEntry> = room
                    .participants
                    .values()
                    .map(|p| ParticipantEntry {
                        id: p.participant_id.clone(),
                        name: p.participant_name.clone(),
                        muted: p.muted,
                    })
                    .collect();
                room.participants.insert(
                    new_pid.clone(),
                    ParticipantSocket {
                        participant_id: new_pid.clone(),
                        participant_name: new_pname.clone(),
                        muted: false,
                        sender: tx.clone(),
                    },
                );
                drop(guard);

                for ep in &existing {
                    if let Err(e) = tx.send(
                        serialize_signal(&OutgoingSignal::UserJoined {
                            participant_id: ep.id.clone(),
                            participant_name: ep.name.clone(),
                        }),
                    ) {
                        log::warn!("Signaling room {room_id}: failed to send user-joined signal: {e}");
                    }
                }
                if let Err(e) = tx.send(
                    serialize_signal(&OutgoingSignal::ParticipantList {
                        participants: existing,
                    }),
                ) {
                    log::warn!("Signaling room {room_id}: failed to send participant list: {e}");
                }
                broadcast_to_room(
                    &rooms,
                    &room_id,
                    &new_pid,
                    &serialize_signal(&OutgoingSignal::UserJoined {
                        participant_id: new_pid.clone(),
                        participant_name: new_pname.clone(),
                    }),
                )
                .await;

                pid = Some(new_pid);
                pname = Some(new_pname);
            }
            SignalMessage::Offer { target_id, sdp } => {
                let from_id = match &pid {
                    Some(id) => id.clone(),
                    None => continue,
                };
                let _ = send_to_participant(
                    &rooms,
                    &room_id,
                    &target_id,
                    &serialize_signal(&OutgoingSignal::Offer { from_id, sdp }),
                )
                .await;
            }
            SignalMessage::Answer { target_id, sdp } => {
                let from_id = match &pid {
                    Some(id) => id.clone(),
                    None => continue,
                };
                let _ = send_to_participant(
                    &rooms,
                    &room_id,
                    &target_id,
                    &serialize_signal(&OutgoingSignal::Answer { from_id, sdp }),
                )
                .await;
            }
            SignalMessage::IceCandidate {
                target_id,
                candidate,
            } => {
                let from_id = match &pid {
                    Some(id) => id.clone(),
                    None => continue,
                };
                let _ = send_to_participant(
                    &rooms,
                    &room_id,
                    &target_id,
                    &serialize_signal(&OutgoingSignal::IceCandidate { from_id, candidate }),
                )
                .await;
            }
            SignalMessage::ChatMessage { content } => {
                let from_id = match &pid {
                    Some(id) => id.clone(),
                    None => continue,
                };
                let from_name = pname.clone().unwrap_or_else(|| "Anonymous".into());
                let ts = chrono::Utc::now().to_rfc3339();
                broadcast_to_room(
                    &rooms,
                    &room_id,
                    &from_id,
                    &serialize_signal(&OutgoingSignal::ChatMessage {
                        from_id: from_id.clone(),
                        from_name,
                        content,
                        timestamp: ts,
                    }),
                )
                .await;
            }
            SignalMessage::Mute { muted } => {
                if let Some(ref p) = pid {
                    let mut guard = rooms.write().await;
                    if let Some(room) = guard.get_mut(&room_id) {
                        if let Some(ps) = room.participants.get_mut(p) {
                            ps.muted = muted;
                        }
                    }
                    drop(guard);
                    broadcast_to_room(
                        &rooms,
                        &room_id,
                        p,
                        &serialize_signal(&OutgoingSignal::Muted {
                            participant_id: p.clone(),
                            muted,
                        }),
                    )
                    .await;
                }
            }
            SignalMessage::ScreenShareStart => {
                broadcast_event(&rooms, &room_id, &pid, |p| {
                    OutgoingSignal::ScreenShareStart { participant_id: p }
                })
                .await
            }
            SignalMessage::ScreenShareStop => {
                broadcast_event(&rooms, &room_id, &pid, |p| {
                    OutgoingSignal::ScreenShareStop { participant_id: p }
                })
                .await
            }
            SignalMessage::RaiseHand => {
                broadcast_event(&rooms, &room_id, &pid, |p| OutgoingSignal::RaiseHand {
                    participant_id: p,
                })
                .await
            }
            SignalMessage::Reaction { emoji } => {
                if let Some(ref p) = pid {
                    broadcast_to_room(
                        &rooms,
                        &room_id,
                        p,
                        &serialize_signal(&OutgoingSignal::Reaction {
                            participant_id: p.clone(),
                            emoji,
                        }),
                    )
                    .await;
                }
            }
            SignalMessage::Leave => break,
        }
    }

    if let Some(ref p) = pid {
        info!("Participant {p} leaving room {room_id}");
        let mut guard = rooms.write().await;
        if let Some(room) = guard.get_mut(&room_id) {
            room.participants.remove(p);
            if room.participants.is_empty() {
                guard.remove(&room_id);
                info!("Room {room_id} empty, removed");
            }
        }
        broadcast_to_room(
            &rooms,
            &room_id,
            p,
            &serialize_signal(&OutgoingSignal::UserLeft {
                participant_id: p.clone(),
            }),
        )
        .await;
    }
    drop(state);
    write_task.abort();
}

async fn broadcast_event<F>(
    rooms: &SignalingRooms,
    room_id: &str,
    pid: &Option<String>,
    make_signal: F,
) where
    F: FnOnce(String) -> OutgoingSignal,
{
    if let Some(ref p) = pid {
        broadcast_to_room(
            rooms,
            room_id,
            p,
            &serialize_signal(&make_signal(p.clone())),
        )
        .await;
    }
}

async fn broadcast_to_room(rooms: &SignalingRooms, room_id: &str, exclude_id: &str, message: &str) {
    let guard = rooms.read().await;
    if let Some(room) = guard.get(room_id) {
        for (pid, p) in &room.participants {
            if pid != exclude_id {
                if let Err(e) = p.sender.send(message.to_string()) {
                    log::warn!("Signaling room {room_id}: failed to send to participant {pid}: {e}");
                }
            }
        }
    }
}

async fn send_to_participant(
    rooms: &SignalingRooms,
    room_id: &str,
    target_id: &str,
    message: &str,
) -> bool {
    let guard = rooms.read().await;
    match guard.get(room_id) {
        Some(room) => {
            let participant_ids: Vec<String> = room.participants.keys().cloned().collect();
            if !target_in_room(&participant_ids, target_id) {
                warn!("Routing ignored: target {target_id} is not a member of room {room_id}");
                return false;
            }
            match room.participants.get(target_id) {
                Some(participant) => participant.sender.send(message.to_string()).is_ok(),
                None => false,
            }
        }
        None => false,
    }
}

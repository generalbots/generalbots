use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use botcore::shared::state::AppState;

pub const MAX_PARTICIPANTS_PER_ROOM: usize = 25;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SignalMessage {
    Join { room_id: String, participant_id: String, participant_name: String },
    Offer { target_id: String, sdp: serde_json::Value },
    Answer { target_id: String, sdp: serde_json::Value },
    IceCandidate { target_id: String, candidate: serde_json::Value },
    ChatMessage { content: String },
    Mute { muted: bool },
    ScreenShareStart,
    ScreenShareStop,
    RaiseHand,
    Reaction { emoji: String },
    Leave,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OutgoingSignal {
    UserJoined { participant_id: String, participant_name: String },
    UserLeft { participant_id: String },
    ParticipantList { participants: Vec<ParticipantEntry> },
    Offer { from_id: String, sdp: serde_json::Value },
    Answer { from_id: String, sdp: serde_json::Value },
    IceCandidate { from_id: String, candidate: serde_json::Value },
    ChatMessage { from_id: String, from_name: String, content: String, timestamp: String },
    Muted { participant_id: String, muted: bool },
    ScreenShareStart { participant_id: String },
    ScreenShareStop { participant_id: String },
    RaiseHand { participant_id: String },
    Reaction { participant_id: String, emoji: String },
    Error { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ParticipantEntry {
    pub id: String,
    pub name: String,
    pub muted: bool,
}

#[derive(Debug, Clone)]
pub struct ParticipantSocket {
    pub participant_id: String,
    pub participant_name: String,
    pub muted: bool,
    pub sender: tokio::sync::mpsc::UnboundedSender<String>,
}

#[derive(Debug, Default, Clone)]
pub struct RoomState {
    pub participants: HashMap<String, ParticipantSocket>,
}

pub type SignalingRooms = Arc<RwLock<HashMap<String, RoomState>>>;

pub fn new_signaling_rooms() -> SignalingRooms {
    Arc::new(RwLock::new(HashMap::new()))
}

pub async fn handle_signaling_socket(
    socket: WebSocket, room_id: String, state: Arc<AppState>, rooms: SignalingRooms,
) {
    let (mut ws_sender, mut ws_receiver) = socket.split();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();
    let mut pid: Option<String> = None;
    let mut pname: Option<String> = None;

    let write_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(Message::Text(msg)).await.is_err() { break; }
        }
    });

    while let Some(msg_result) = ws_receiver.next().await {
        let text = match msg_result {
            Ok(Message::Text(t)) => t.to_string(),
            Ok(Message::Close(_)) => break,
            Err(e) => { warn!("WS error room {room_id}: {e}"); break; }
            _ => continue,
        };

        let signal: SignalMessage = match serde_json::from_str(&text) {
            Ok(s) => s,
            Err(e) => {
                warn!("Invalid signal room {room_id}: {e}");
                let _ = tx.send(serde_json::to_string(&OutgoingSignal::Error { message: format!("Invalid: {e}") }).unwrap_or_default());
                continue;
            }
        };

        match signal {
            SignalMessage::Join { room_id: jr, participant_id: new_pid, participant_name: new_pname } => {
                if jr != room_id {
                    let _ = tx.send(serde_json::to_string(&OutgoingSignal::Error { message: "Room ID mismatch".into() }).unwrap_or_default());
                    continue;
                }
                let mut guard = rooms.write().await;
                let room = guard.entry(room_id.clone()).or_default();
                if room.participants.len() >= MAX_PARTICIPANTS_PER_ROOM {
                    drop(guard);
                    let _ = tx.send(serde_json::to_string(&OutgoingSignal::Error {
                        message: format!("Room full (max {MAX_PARTICIPANTS_PER_ROOM})"),
                    }).unwrap_or_default());
                    continue;
                }
                let existing: Vec<ParticipantEntry> = room.participants.values()
                    .map(|p| ParticipantEntry { id: p.participant_id.clone(), name: p.participant_name.clone(), muted: p.muted })
                    .collect();
                room.participants.insert(new_pid.clone(), ParticipantSocket {
                    participant_id: new_pid.clone(), participant_name: new_pname.clone(),
                    muted: false, sender: tx.clone(),
                });
                drop(guard);

                for ep in &existing {
                    let _ = tx.send(serde_json::to_string(&OutgoingSignal::UserJoined { participant_id: ep.id.clone(), participant_name: ep.name.clone() }).unwrap_or_default());
                }
                let _ = tx.send(serde_json::to_string(&OutgoingSignal::ParticipantList { participants: existing }).unwrap_or_default());
                broadcast_to_room(&rooms, &room_id, &new_pid, &serde_json::to_string(&OutgoingSignal::UserJoined { participant_id: new_pid.clone(), participant_name: new_pname.clone() }).unwrap_or_default()).await;

                pid = Some(new_pid);
                pname = Some(new_pname);
            }
            SignalMessage::Offer { target_id, sdp } => {
                let from_id = match &pid { Some(id) => id.clone(), None => continue };
                let _ = send_to_participant(&rooms, &room_id, &target_id, &serde_json::to_string(&OutgoingSignal::Offer { from_id, sdp }).unwrap_or_default()).await;
            }
            SignalMessage::Answer { target_id, sdp } => {
                let from_id = match &pid { Some(id) => id.clone(), None => continue };
                let _ = send_to_participant(&rooms, &room_id, &target_id, &serde_json::to_string(&OutgoingSignal::Answer { from_id, sdp }).unwrap_or_default()).await;
            }
            SignalMessage::IceCandidate { target_id, candidate } => {
                let from_id = match &pid { Some(id) => id.clone(), None => continue };
                let _ = send_to_participant(&rooms, &room_id, &target_id, &serde_json::to_string(&OutgoingSignal::IceCandidate { from_id, candidate }).unwrap_or_default()).await;
            }
            SignalMessage::ChatMessage { content } => {
                let from_id = match &pid { Some(id) => id.clone(), None => continue };
                let from_name = pname.clone().unwrap_or_else(|| "Anonymous".into());
                let ts = chrono::Utc::now().to_rfc3339();
                broadcast_to_room(&rooms, &room_id, &from_id, &serde_json::to_string(
                    &OutgoingSignal::ChatMessage { from_id: from_id.clone(), from_name, content, timestamp: ts },
                ).unwrap_or_default()).await;
            }
            SignalMessage::Mute { muted } => {
                if let Some(ref p) = pid {
                    let mut guard = rooms.write().await;
                    if let Some(room) = guard.get_mut(&room_id) {
                        if let Some(ps) = room.participants.get_mut(p) { ps.muted = muted; }
                    }
                    drop(guard);
                    broadcast_to_room(&rooms, &room_id, p, &serde_json::to_string(&OutgoingSignal::Muted { participant_id: p.clone(), muted }).unwrap_or_default()).await;
                }
            }
            SignalMessage::ScreenShareStart => broadcast_event(&rooms, &room_id, &pid, |p| OutgoingSignal::ScreenShareStart { participant_id: p }).await,
            SignalMessage::ScreenShareStop => broadcast_event(&rooms, &room_id, &pid, |p| OutgoingSignal::ScreenShareStop { participant_id: p }).await,
            SignalMessage::RaiseHand => broadcast_event(&rooms, &room_id, &pid, |p| OutgoingSignal::RaiseHand { participant_id: p }).await,
            SignalMessage::Reaction { emoji } => {
                if let Some(ref p) = pid {
                    broadcast_to_room(&rooms, &room_id, p, &serde_json::to_string(&OutgoingSignal::Reaction { participant_id: p.clone(), emoji }).unwrap_or_default()).await;
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
            if room.participants.is_empty() { guard.remove(&room_id); info!("Room {room_id} empty, removed"); }
        }
        broadcast_to_room(&rooms, &room_id, p, &serde_json::to_string(&OutgoingSignal::UserLeft { participant_id: p.clone() }).unwrap_or_default()).await;
    }
    drop(state);
    write_task.abort();
}

async fn broadcast_event<F>(rooms: &SignalingRooms, room_id: &str, pid: &Option<String>, make_signal: F)
where F: FnOnce(String) -> OutgoingSignal {
    if let Some(ref p) = pid {
        broadcast_to_room(rooms, room_id, p, &serde_json::to_string(&make_signal(p.clone())).unwrap_or_default()).await;
    }
}

async fn broadcast_to_room(rooms: &SignalingRooms, room_id: &str, exclude_id: &str, message: &str) {
    let guard = rooms.read().await;
    if let Some(room) = guard.get(room_id) {
        for (pid, p) in &room.participants {
            if pid != exclude_id { let _ = p.sender.send(message.to_string()); }
        }
    }
}

async fn send_to_participant(rooms: &SignalingRooms, room_id: &str, target_id: &str, message: &str) -> bool {
    let guard = rooms.read().await;
    if let Some(room) = guard.get(room_id) {
        if let Some(p) = room.participants.get(target_id) {
            return p.sender.send(message.to_string()).is_ok();
        }
        warn!("Participant {target_id} not found in room {room_id}");
    }
    false
}

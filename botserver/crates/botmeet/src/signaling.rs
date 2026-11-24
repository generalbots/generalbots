use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use botcore::shared::state::AppState;

mod guard;

pub use guard::authenticated_signaling_websocket;
use guard::{target_in_room, ParticipantIdentity};

pub const MAX_PARTICIPANTS_PER_ROOM: usize = 25;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SignalMessage {
    Join {
        room_id: String,
        participant_id: String,
        participant_name: String,
    },
    Offer {
        target_id: String,
        sdp: serde_json::Value,
    },
    Answer {
        target_id: String,
        sdp: serde_json::Value,
    },
    IceCandidate {
        target_id: String,
        candidate: serde_json::Value,
    },
    ChatMessage {
        content: String,
    },
    Mute {
        muted: bool,
    },
    ScreenShareStart,
    ScreenShareStop,
    RaiseHand,
    Reaction {
        emoji: String,
    },
    Leave,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OutgoingSignal {
    UserJoined {
        participant_id: String,
        participant_name: String,
    },
    UserLeft {
        participant_id: String,
    },
    ParticipantList {
        participants: Vec<ParticipantEntry>,
    },
    Offer {
        from_id: String,
        sdp: serde_json::Value,
    },
    Answer {
        from_id: String,
        sdp: serde_json::Value,
    },
    IceCandidate {
        from_id: String,
        candidate: serde_json::Value,
    },
    ChatMessage {
        from_id: String,
        from_name: String,
        content: String,
        timestamp: String,
    },
    Muted {
        participant_id: String,
        muted: bool,
    },
    ScreenShareStart {
        participant_id: String,
    },
    ScreenShareStop {
        participant_id: String,
    },
    RaiseHand {
        participant_id: String,
    },
    Reaction {
        participant_id: String,
        emoji: String,
    },
    Error {
        message: String,
    },
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

mod handler;

pub(crate) use handler::handle_authenticated_signaling_socket;
pub use handler::handle_signaling_socket;

use crate::shared::models::{BotResponse, UserMessage};
use crate::shared::state::AppState;
use anyhow::Result;
use async_trait::async_trait;
use axum::extract::ws::{Message, WebSocket};
use botlib::MessageType;
use futures::{SinkExt, StreamExt};
use log::{info, trace, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    pub id: String,
    pub name: String,
    pub email: Option<String>,
    pub role: ParticipantRole,
    pub is_bot: bool,
    pub joined_at: chrono::DateTime<chrono::Utc>,
    pub is_active: bool,
    pub has_video: bool,
    pub has_audio: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParticipantRole {
    Host,
    Moderator,
    Participant,
    Bot,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingRoom {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_by: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub participants: Vec<Participant>,
    pub is_recording: bool,
    pub is_transcribing: bool,
    pub max_participants: usize,
    pub settings: MeetingSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingSettings {
    pub enable_transcription: bool,
    pub enable_recording: bool,
    pub enable_chat: bool,
    pub enable_screen_share: bool,
    pub auto_admit: bool,
    pub waiting_room: bool,
    pub bot_enabled: bool,
    pub bot_id: Option<String>,
}

impl Default for MeetingSettings {
    fn default() -> Self {
        Self {
            enable_transcription: true,
            enable_recording: false,
            enable_chat: true,
            enable_screen_share: true,
            auto_admit: true,
            waiting_room: false,
            bot_enabled: true,
            bot_id: None,
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MeetingMessage {

    JoinMeeting {
        room_id: String,
        participant_name: String,
        participant_id: Option<String>,
    },

    LeaveMeeting {
        room_id: String,
        participant_id: String,
    },

    Transcription {
        room_id: String,
        participant_id: String,
        text: String,
        timestamp: chrono::DateTime<chrono::Utc>,
        confidence: f32,
        is_final: bool,
    },

    ChatMessage {
        room_id: String,
        participant_id: String,
        content: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    BotMessage {
        room_id: String,
        content: String,
        in_response_to: Option<String>,
        metadata: HashMap<String, String>,
    },

    ScreenShare {
        room_id: String,
        participant_id: String,
        is_sharing: bool,
        share_type: Option<ScreenShareType>,
    },

    StatusUpdate {
        room_id: String,
        status: MeetingStatus,
        details: Option<String>,
    },

    ParticipantUpdate {
        room_id: String,
        participant: Participant,
        action: ParticipantAction,
    },

    RecordingControl {
        room_id: String,
        action: RecordingAction,
        participant_id: String,
    },

    BotRequest {
        room_id: String,
        participant_id: String,
        command: String,
        parameters: HashMap<String, String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScreenShareType {
    Screen,
    Window,
    Tab,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MeetingStatus {
    Waiting,
    Active,
    Paused,
    Ended,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParticipantAction {
    Joined,
    Left,
    Updated,
    Muted,
    Unmuted,
    VideoOn,
    VideoOff,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordingAction {
    Start,
    Stop,
    Pause,
    Resume,
}


pub struct MeetingService {
    pub state: Arc<AppState>,
    pub rooms: Arc<RwLock<HashMap<String, MeetingRoom>>>,
    pub connections: Arc<RwLock<HashMap<String, mpsc::Sender<MeetingMessage>>>>,
    pub transcription_service: Arc<dyn TranscriptionService + Send + Sync>,
}

impl std::fmt::Debug for MeetingService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MeetingService")
            .field("state", &self.state)
            .field("rooms", &"Arc<RwLock<HashMap<String, MeetingRoom>>>")
            .field(
                "connections",
                &"Arc<RwLock<HashMap<String, mpsc::Sender<MeetingMessage>>>>",
            )
            .field("transcription_service", &"Arc<dyn TranscriptionService>")
            .finish()
    }
}

impl MeetingService {
    pub fn new(state: Arc<AppState>, transcription_service: Arc<dyn TranscriptionService>) -> Self {
        Self {
            state,
            rooms: Arc::new(RwLock::new(HashMap::new())),
            connections: Arc::new(RwLock::new(HashMap::new())),
            transcription_service,
        }
    }


    pub async fn create_room(
        &self,
        name: String,
        created_by: String,
        settings: Option<MeetingSettings>,
    ) -> Result<MeetingRoom> {
        let room_id = Uuid::new_v4().to_string();
        let room = MeetingRoom {
            id: room_id.clone(),
            name,
            description: None,
            created_by,
            created_at: chrono::Utc::now(),
            participants: Vec::new(),
            is_recording: false,
            is_transcribing: settings.as_ref().map_or(true, |s| s.enable_transcription),
            max_participants: 100,
            settings: settings.unwrap_or_default(),
        };

        self.rooms.write().await.insert(room_id, room.clone());


        if room.settings.bot_enabled {
            self.add_bot_to_room(&room.id).await?;
        }

        info!("Created meeting room: {} ({})", room.name, room.id);
        Ok(room)
    }


    pub async fn join_room(
        &self,
        room_id: &str,
        participant_name: String,
        participant_id: Option<String>,
    ) -> Result<Participant> {
        let mut rooms = self.rooms.write().await;
        let room = rooms
            .get_mut(room_id)
            .ok_or_else(|| anyhow::anyhow!("Room not found"))?;

        let participant = Participant {
            id: participant_id.unwrap_or_else(|| Uuid::new_v4().to_string()),
            name: participant_name,
            email: None,
            role: ParticipantRole::Participant,
            is_bot: false,
            joined_at: chrono::Utc::now(),
            is_active: true,
            has_video: false,
            has_audio: true,
        };

        room.participants.push(participant.clone());


        if room.is_transcribing && room.participants.iter().filter(|p| !p.is_bot).count() == 1 {
            self.start_transcription(room_id).await?;
        }

        info!(
            "Participant {} joined room {} ({})",
            participant.name, room.name, room.id
        );

        Ok(participant)
    }


    async fn add_bot_to_room(&self, room_id: &str) -> Result<()> {
        let bot_participant = Participant {
            id: format!("bot-{}", Uuid::new_v4()),
            name: "Meeting Assistant".to_string(),
            email: Some("bot@botserver.com".to_string()),
            role: ParticipantRole::Bot,
            is_bot: true,
            joined_at: chrono::Utc::now(),
            is_active: true,
            has_video: false,
            has_audio: true,
        };

        let mut rooms = self.rooms.write().await;
        if let Some(room) = rooms.get_mut(room_id) {
            room.participants.push(bot_participant);
            info!("Bot added to room: {}", room_id);
        }

        Ok(())
    }


    pub async fn start_transcription(&self, room_id: &str) -> Result<()> {
        info!("Starting transcription for room: {}", room_id);

        let rooms = self.rooms.read().await;
        if let Some(room) = rooms.get(room_id) {
            if room.is_transcribing {
                self.transcription_service
                    .start_transcription(room_id)
                    .await?;
            }
        }

        Ok(())
    }


    pub async fn handle_websocket(&self, socket: WebSocket, room_id: String) {
        let (mut sender, mut receiver) = socket.split();
        let (tx, mut rx) = mpsc::channel::<MeetingMessage>(100);


        self.connections
            .write()
            .await
            .insert(room_id.clone(), tx.clone());


        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if let Ok(json) = serde_json::to_string(&msg) {
                    if sender.send(Message::Text(json.into())).await.is_err() {
                        break;
                    }
                }
            }
        });


        while let Some(msg) = receiver.next().await {
            if let Ok(Message::Text(text)) = msg {
                if let Ok(meeting_msg) = serde_json::from_str::<MeetingMessage>(&text) {
                    self.handle_meeting_message(meeting_msg, &room_id).await;
                }
            }
        }


        self.connections.write().await.remove(&room_id);
    }


    async fn handle_meeting_message(&self, message: MeetingMessage, room_id: &str) {
        match message {
            MeetingMessage::Transcription {
                text,
                participant_id,
                is_final,
                ..
            } => {
                if is_final {
                    info!("Transcription from {}: {}", participant_id, text);


                    if let Some(room) = self.rooms.read().await.get(room_id) {
                        if room.settings.bot_enabled {
                            self.process_bot_command(&text, room_id, &participant_id)
                                .await;
                        }
                    }
                }
            }
            MeetingMessage::BotRequest {
                command,
                parameters,
                participant_id,
                ..
            } => {
                info!("Bot request from {}: {}", participant_id, command);
                self.handle_bot_request(&command, parameters, room_id, &participant_id)
                    .await;
            }
            MeetingMessage::ChatMessage { .. } => {

                self.broadcast_to_room(room_id, message.clone()).await;
            }
            _ => {
                trace!("Handling meeting message: {:?}", message);
            }
        }
    }


    async fn process_bot_command(&self, text: &str, room_id: &str, participant_id: &str) {

        if text.to_lowercase().contains("hey bot") || text.to_lowercase().contains("assistant") {

            let user_message = UserMessage {
                bot_id: "meeting-assistant".to_string(),
                user_id: participant_id.to_string(),
                session_id: room_id.to_string(),
                channel: "meeting".to_string(),
                content: text.to_string(),
                message_type: MessageType::USER,
                media_url: None,
                timestamp: chrono::Utc::now(),
                context_name: None,
            };


            if let Ok(response) = self.process_with_bot(user_message).await {
                let bot_msg = MeetingMessage::ChatMessage {
                    room_id: room_id.to_string(),
                    content: response.content,
                    participant_id: "bot".to_string(),
                    timestamp: chrono::Utc::now(),
                };

                self.broadcast_to_room(room_id, bot_msg).await;
            }
        }
    }


    async fn handle_bot_request(
        &self,
        command: &str,
        _parameters: HashMap<String, String>,
        room_id: &str,
        participant_id: &str,
    ) {
        match command {
            "summarize" => {
                let summary = "Meeting summary: Discussion about project updates and next steps.";
                let bot_msg = MeetingMessage::BotMessage {
                    room_id: room_id.to_string(),
                    content: summary.to_string(),
                    in_response_to: Some(participant_id.to_string()),
                    metadata: HashMap::from([("command".to_string(), "summarize".to_string())]),
                };
                self.broadcast_to_room(room_id, bot_msg).await;
            }
            "action_items" => {
                let actions = "Action items:\n1. Review documentation\n2. Schedule follow-up";
                let bot_msg = MeetingMessage::BotMessage {
                    room_id: room_id.to_string(),
                    content: actions.to_string(),
                    in_response_to: Some(participant_id.to_string()),
                    metadata: HashMap::from([("command".to_string(), "action_items".to_string())]),
                };
                self.broadcast_to_room(room_id, bot_msg).await;
            }
            _ => {
                warn!("Unknown bot command: {}", command);
            }
        }
    }


    async fn process_with_bot(&self, message: UserMessage) -> Result<BotResponse> {


        Ok(BotResponse {
            bot_id: message.bot_id,
            user_id: message.user_id,
            session_id: message.session_id,
            channel: "meeting".to_string(),
            content: format!("Processing: {}", message.content),
            message_type: MessageType::BOT_RESPONSE,
            stream_token: None,
            is_complete: true,
            suggestions: Vec::new(),
            context_name: None,
            context_length: 0,
            context_max_length: 0,
        })
    }


    async fn broadcast_to_room(&self, room_id: &str, message: MeetingMessage) {
        let connections = self.connections.read().await;
        if let Some(tx) = connections.get(room_id) {
            let _ = tx.send(message).await;
        }
    }


    pub async fn get_room(&self, room_id: &str) -> Option<MeetingRoom> {
        self.rooms.read().await.get(room_id).cloned()
    }


    pub async fn list_rooms(&self) -> Vec<MeetingRoom> {
        self.rooms.read().await.values().cloned().collect()
    }
}


#[async_trait]
pub trait TranscriptionService: Send + Sync {
    async fn start_transcription(&self, room_id: &str) -> Result<()>;
    async fn stop_transcription(&self, room_id: &str) -> Result<()>;
    async fn process_audio(&self, audio_data: Vec<u8>, room_id: &str) -> Result<String>;
}


#[derive(Debug)]
pub struct DefaultTranscriptionService;

#[async_trait]
impl TranscriptionService for DefaultTranscriptionService {
    async fn start_transcription(&self, room_id: &str) -> Result<()> {
        info!("Starting transcription for room: {}", room_id);
        Ok(())
    }

    async fn stop_transcription(&self, room_id: &str) -> Result<()> {
        info!("Stopping transcription for room: {}", room_id);
        Ok(())
    }

    async fn process_audio(&self, _audio_data: Vec<u8>, room_id: &str) -> Result<String> {

        Ok(format!("Transcribed text for room {}", room_id))
    }
}

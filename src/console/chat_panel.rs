use crate::shared::message_types::MessageType;
use crate::shared::models::BotResponse;
use crate::shared::state::AppState;
use color_eyre::Result;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;
pub struct ChatPanel {
    pub messages: Vec<String>,
    pub input_buffer: String,
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub response_rx: Option<mpsc::Receiver<BotResponse>>,
}

impl std::fmt::Debug for ChatPanel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChatPanel")
            .field("messages_count", &self.messages.len())
            .field("input_buffer_len", &self.input_buffer.len())
            .field("session_id", &self.session_id)
            .field("user_id", &self.user_id)
            .field("has_response_rx", &self.response_rx.is_some())
            .finish()
    }
}

impl ChatPanel {
    pub fn new(_app_state: Arc<AppState>) -> Self {
        Self {
            messages: vec!["Welcome to General Bots Console Chat!".to_string()],
            input_buffer: String::new(),
            session_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            response_rx: None,
        }
    }
    pub fn add_char(&mut self, c: char) {
        self.input_buffer.push(c);
    }
    pub fn backspace(&mut self) {
        self.input_buffer.pop();
    }
    pub async fn send_message(&mut self, bot_name: &str, app_state: &Arc<AppState>) -> Result<()> {
        if self.input_buffer.trim().is_empty() {
            return Ok(());
        }
        let message = self.input_buffer.clone();
        self.messages.push(format!("You: {}", message));
        self.input_buffer.clear();
        let bot_id = self.get_bot_id(bot_name, app_state).await?;
        let user_message = crate::shared::models::UserMessage {
            bot_id: bot_id.to_string(),
            user_id: self.user_id.to_string(),
            session_id: self.session_id.to_string(),
            channel: "console".to_string(),
            content: message,
            message_type: MessageType::USER,
            media_url: None,
            timestamp: chrono::Utc::now(),
            context_name: None,
        };
        let (tx, rx) = mpsc::channel::<BotResponse>(100);
        self.response_rx = Some(rx);
        let orchestrator = crate::bot::BotOrchestrator::new(app_state.clone());
        let _ = orchestrator.stream_response(user_message, tx).await;
        Ok(())
    }
    pub async fn poll_response(&mut self, _bot_name: &str) -> Result<()> {
        if let Some(rx) = &mut self.response_rx {
            while let Ok(response) = rx.try_recv() {
                if !response.content.is_empty() && !response.is_complete {
                    if let Some(last_msg) = self.messages.last_mut() {
                        if last_msg.starts_with("Bot: ") {
                            last_msg.push_str(&response.content);
                        } else {
                            self.messages.push(format!("Bot: {}", response.content));
                        }
                    } else {
                        self.messages.push(format!("Bot: {}", response.content));
                    }
                }
                if response.is_complete && response.content.is_empty() {
                    break;
                }
            }
        }
        Ok(())
    }
    async fn get_bot_id(&self, bot_name: &str, app_state: &Arc<AppState>) -> Result<Uuid> {
        use crate::shared::models::schema::bots::dsl::*;
        use diesel::prelude::*;
        let mut conn = app_state.conn.get().unwrap();
        let bot_id = bots
            .filter(name.eq(bot_name))
            .select(id)
            .first::<Uuid>(&mut *conn)?;
        Ok(bot_id)
    }
    pub fn render(&self) -> String {
        let mut lines = Vec::new();
        lines.push("╔═══════════════════════════════════════╗".to_string());
        lines.push("║         CONVERSATION                  ║".to_string());
        lines.push("╚═══════════════════════════════════════╝".to_string());
        lines.push("".to_string());
        let visible_start = if self.messages.len() > 15 {
            self.messages.len() - 15
        } else {
            0
        };
        for msg in &self.messages[visible_start..] {
            if msg.starts_with("You: ") {
                lines.push(format!(" {}", msg));
            } else if msg.starts_with("Bot: ") {
                lines.push(format!(" {}", msg));
            } else {
                lines.push(format!(" {}", msg));
            }
        }
        lines.push("".to_string());
        lines.push("─────────────────────────────────────────".to_string());
        lines.push(format!(" > {}_", self.input_buffer));
        lines.push("".to_string());
        lines.push(" Enter: Send | Tab: Switch Panel".to_string());
        lines.join("\n")
    }
}

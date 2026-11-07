use crate::core::bot::pipeline::{ChannelSink, PipelineError};
use crate::core::bot::runs::bus;
use uuid::Uuid;

/// Sink used by a background turn: frames are published to the thread channel
/// so every subscribed socket receives them, and the turn keeps running after
/// the socket that started it is gone.
pub(super) struct RunBusSink {
    pub(super) thread_id: String,
}

#[async_trait::async_trait]
impl ChannelSink for RunBusSink {
    async fn send_bot_response(
        &self,
        response: &botlib::models::BotResponse,
    ) -> Result<(), PipelineError> {
        let mut frame = serde_json::to_value(response)
            .map_err(|error| PipelineError::Transport(format!("run frame encode: {error}")))?;
        // #1756 — a completed assistant frame carries a non-empty id so tests
        // and clients can assert one message per turn without content dedupe.
        if response.is_complete && !response.content.is_empty() && frame.get("message_id").is_none()
        {
            if let Some(object) = frame.as_object_mut() {
                object.insert(
                    "message_id".to_string(),
                    serde_json::Value::String(Uuid::new_v4().to_string()),
                );
            }
        }
        bus::bus().publish(&self.thread_id, frame);
        Ok(())
    }

    async fn send_raw_json(&self, json: &serde_json::Value) -> Result<(), PipelineError> {
        bus::bus().publish(&self.thread_id, json.clone());
        Ok(())
    }

    async fn send_error(&self, session_id: &str, message: &str) -> Result<(), PipelineError> {
        let response = botlib::models::BotResponse::new("", session_id, "", message, "web");
        self.send_bot_response(&response).await
    }

    fn channel_type(&self) -> &str {
        "web"
    }

    fn supports_streaming(&self) -> bool {
        true
    }

    fn supports_suggestions(&self) -> bool {
        true
    }

    fn supports_raw_frames(&self) -> bool {
        true
    }
}

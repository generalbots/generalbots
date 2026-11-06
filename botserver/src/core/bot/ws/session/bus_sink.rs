use crate::core::bot::pipeline::{ChannelSink, PipelineError};
use crate::core::bot::runs::bus;

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
        let frame = serde_json::to_value(response)
            .map_err(|error| PipelineError::Transport(format!("run frame encode: {error}")))?;
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

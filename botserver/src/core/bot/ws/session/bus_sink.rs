use std::sync::Arc;

use botcore::shared::state::AppState;
use uuid::Uuid;

use crate::core::bot::pipeline::{ChannelSink, PipelineError};
use crate::core::bot::runs::bus;

/// Sink used by a background turn: frames are published to the thread channel
/// so every subscribed socket receives them, and the turn keeps running after
/// the socket that started it is gone.
pub(super) struct RunBusSink {
    thread_id: String,
    state: Arc<AppState>,
}

impl RunBusSink {
    /// Builds the sink from the session id with the same helper the socket
    /// subscribes with (#2084): the producer can no longer publish to a thread
    /// nobody listens on because of a differently formatted id.
    pub(super) fn new(session_id: Uuid, state: Arc<AppState>) -> Self {
        Self {
            thread_id: bus::thread_id(session_id),
            state,
        }
    }

    /// Keeps a finished answer that no socket received (#2084). The next
    /// connection of this session delivers it from `pending_stream_responses`
    /// instead of leaving the user on "Waiting for the model…" forever.
    async fn keep_for_reconnect(&self, response: &botlib::models::BotResponse) {
        if !response.is_complete || response.content.is_empty() {
            return;
        }
        let mut pending = self.state.pending_stream_responses.lock().await;
        pending.insert(self.thread_id.clone(), response.content.clone());
    }
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
        let delivered = bus::bus().publish(&self.thread_id, frame);
        if delivered == 0 && response.is_complete && !response.content.is_empty() {
            // The socket died (or is still reconnecting) during the turn: the
            // LLM answer used to be published into the void with no trace in
            // the log (#2084).
            botlog::safe_warn!(
                "run frame delivered to 0 sockets on thread {}; the reply is kept for the next connection",
                self.thread_id
            );
            self.keep_for_reconnect(response).await;
        }
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

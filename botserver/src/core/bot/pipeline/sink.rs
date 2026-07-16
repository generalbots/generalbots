use async_trait::async_trait;
use botlib::models::BotResponse;

use super::types::{PipelineError, PipelineResult};

#[async_trait]
pub trait ChannelSink: Send + Sync {
    async fn send_bot_response(&self, response: &BotResponse) -> PipelineResult<()>;

    async fn send_raw_json(&self, json: &serde_json::Value) -> PipelineResult<()> {
        let _ = json;
        Err(PipelineError::Transport("raw JSON not supported by this channel".into()))
    }

    async fn send_error(&self, session_id: &str, message: &str) -> PipelineResult<()>;

    fn channel_type(&self) -> &str;
    fn supports_streaming(&self) -> bool {
        true
    }
    fn supports_suggestions(&self) -> bool {
        true
    }
    fn supports_raw_frames(&self) -> bool {
        false
    }
}

pub struct MpscChannelSink(pub tokio::sync::mpsc::Sender<BotResponse>);

#[async_trait]
impl ChannelSink for MpscChannelSink {
    async fn send_bot_response(&self, response: &BotResponse) -> PipelineResult<()> {
        self.0.send(response.clone()).await
            .map_err(|_| PipelineError::Transport("mpsc send failed".into()))?;
        Ok(())
    }

    async fn send_error(&self, session_id: &str, message: &str) -> PipelineResult<()> {
        let resp = BotResponse::new("", session_id, "", message, "mpsc");
        self.send_bot_response(&resp).await
    }

    fn channel_type(&self) -> &str { "mpsc" }
    fn supports_streaming(&self) -> bool { true }
}
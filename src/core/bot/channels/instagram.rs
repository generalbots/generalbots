use crate::shared::models::BotResponse;
use async_trait::async_trait;
use log::info;

/// Instagram channel adapter for sending messages through Instagram
pub struct InstagramAdapter {
    // TODO: Add Instagram API client configuration
}

impl InstagramAdapter {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl super::ChannelAdapter for InstagramAdapter {
    async fn send_message(
        &self,
        response: BotResponse,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!(
            "Instagram message would be sent to {}: {}",
            response.user_id, response.content
        );
        // TODO: Implement actual Instagram API integration
        Ok(())
    }
}

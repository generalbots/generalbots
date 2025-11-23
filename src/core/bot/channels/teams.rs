use crate::shared::models::BotResponse;
use async_trait::async_trait;
use log::info;

/// Microsoft Teams channel adapter for sending messages through Teams
pub struct TeamsAdapter {
    // TODO: Add Teams API client configuration
}

impl TeamsAdapter {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl super::ChannelAdapter for TeamsAdapter {
    async fn send_message(
        &self,
        response: BotResponse,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!(
            "Teams message would be sent to {}: {}",
            response.user_id, response.content
        );
        // TODO: Implement actual Teams API integration
        Ok(())
    }
}

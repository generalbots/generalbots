use crate::shared::models::BotResponse;
use async_trait::async_trait;
use log::info;

/// WhatsApp channel adapter for sending messages through WhatsApp
pub struct WhatsAppAdapter {
    // TODO: Add WhatsApp API client configuration
}

impl WhatsAppAdapter {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl super::ChannelAdapter for WhatsAppAdapter {
    async fn send_message(
        &self,
        response: BotResponse,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!(
            "WhatsApp message would be sent to {}: {}",
            response.user_id, response.content
        );
        // TODO: Implement actual WhatsApp API integration
        Ok(())
    }
}

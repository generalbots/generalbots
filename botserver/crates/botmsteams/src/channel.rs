use botlib::models::BotResponse;
use async_trait::async_trait;

#[async_trait]
pub trait ChannelAdapter: Send + Sync {
    fn name(&self) -> &'static str {
        "Unknown"
    }

    fn is_configured(&self) -> bool {
        true
    }

    async fn send_message(
        &self,
        response: BotResponse,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    async fn receive_message(
        &self,
        _payload: serde_json::Value,
    ) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(None)
    }

    async fn get_user_info(
        &self,
        user_id: &str,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        Ok(serde_json::json!({
            "id": user_id,
            "platform": self.name()
        }))
    }
}

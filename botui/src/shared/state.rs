
use botlib::http_client::BotServerClient;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct AppState {
    pub client: Arc<BotServerClient>,
}

impl AppState {
    #[must_use]
    pub fn new() -> Self {
        let url = std::env::var("BOTSERVER_URL").ok();
        Self {
            client: Arc::new(BotServerClient::new(url)),
        }
    }

    pub async fn health_check(&self) -> bool {
        self.client.health_check().await
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

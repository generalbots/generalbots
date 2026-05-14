pub use botcorebot::*;
pub mod ws_handler;

pub use ws_handler::{websocket_handler, websocket_handler_with_bot};
use std::collections::HashMap;
use std::sync::Arc;
use axum::response::IntoResponse;

pub mod channels {
    pub use botlib::traits::ChannelAdapter;

    #[derive(Debug)]
    pub struct VoiceAdapter;

    #[derive(Debug)]
    pub struct WebChannelAdapter;

    impl Default for VoiceAdapter {
    fn default() -> Self { Self::new() }
}

impl VoiceAdapter {
        pub fn new() -> Self { Self }
    }

    impl Default for WebChannelAdapter {
    fn default() -> Self { Self::new() }
}

impl WebChannelAdapter {
        pub fn new() -> Self { Self }
    }

    impl ChannelAdapter for VoiceAdapter {
        fn channel_type(&self) -> &str { "voice" }
        fn send_message(&self, to: &str, _message: &str) -> Result<(), String> {
            log::warn!("VoiceAdapter::send_message stub: to={to}");
            Ok(())
        }
    }

    impl ChannelAdapter for WebChannelAdapter {
        fn channel_type(&self) -> &str { "web" }
        fn send_message(&self, to: &str, _message: &str) -> Result<(), String> {
            log::warn!("WebChannelAdapter::send_message stub: to={to}");
            Ok(())
        }
    }

    pub mod whatsapp {
        pub struct WhatsAppAdapter;
        impl WhatsAppAdapter {
            pub fn new(_state: &botcore::shared::state::AppState, _bot_id: uuid::Uuid) -> Self { Self }
            pub async fn send_message(&self, _response: botlib::models::BotResponse) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
                log::warn!("WhatsAppAdapter::send_message stub");
                Ok(())
            }
        }
    }
    pub mod instagram {
        pub struct InstagramAdapter;
    impl Default for InstagramAdapter {
    fn default() -> Self { Self::new() }
}

impl InstagramAdapter {
        pub fn new() -> Self { Self }
        pub async fn send_message(&self, _response: botlib::models::BotResponse) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            log::warn!("InstagramAdapter::send_message stub");
            Ok(())
        }
        pub async fn send_instagram_message(&self, _recipient: &str, _message: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            log::warn!("InstagramAdapter::send_instagram_message stub");
            Ok(())
        }
    }
    }
    pub mod teams {
        pub struct TeamsAdapter;
        impl Default for TeamsAdapter {
    fn default() -> Self { Self::new() }
}

impl TeamsAdapter {
            pub fn new() -> Self { Self }
            pub async fn send_message(&self, _response: botlib::models::BotResponse) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
                log::warn!("TeamsAdapter::send_message stub");
                Ok(())
            }
        }
    }
}

pub mod kb_context {
    pub struct KbContextManager;
    impl Default for KbContextManager {
    fn default() -> Self { Self::new() }
}

impl KbContextManager {
        pub fn new() -> Self { Self }
        pub fn search_active_kbs(&self, _session_id: &str, _bot_id: uuid::Uuid, _bot_name: &str, _query: &str, _limit: usize, _max_len: usize) -> Vec<String> {
            Vec::new()
        }
    }
}

pub struct BotOrchestrator;

impl BotOrchestrator {
    pub fn new(_state: std::sync::Arc<botcore::shared::state::AppState>) -> Self { Self }
    pub fn mount_all_bots(&self) -> Result<(), String> { 
        log::info!("BotOrchestrator::mount_all_bots stub"); 
        Ok(()) 
    }
}

pub fn get_default_bot() -> (String, String) { ("default".to_string(), "Default Bot".to_string()) }
pub async fn get_bot_config(
    axum::extract::State(state): axum::extract::State<Arc<botcore::shared::state::AppState>>,
    axum::extract::Query(_params): axum::extract::Query<HashMap<String, String>>,
) -> impl IntoResponse {
    use diesel::prelude::*;
    let mut conn = match state.conn.get() {
        Ok(c) => c,
        Err(_) => return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "db error".to_string()),
    };
    use botcore::shared::models::schema::bot_configuration::dsl::*;
    let rows: Vec<(String, String)> = match bot_configuration
        .select((config_key, config_value))
        .load(&mut conn)
    {
        Ok(r) => r,
        Err(_) => vec![],
    };
    let map: HashMap<String, String> = rows.into_iter().collect();
    (axum::http::StatusCode::OK, serde_json::to_string(&map).unwrap_or_default())
}

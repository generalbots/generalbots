pub use botcorebot::*;
pub mod ws;
pub mod manager;
pub mod manager_ops;
pub mod tool_context;
pub mod multimedia;

pub use ws::{websocket_handler, websocket_handler_with_bot};

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

pub mod answer_mode;
pub mod answer_mode_config;
pub mod answer_mode_ops;
pub mod kb_context;
pub mod pipeline;

pub struct BotOrchestrator;

impl BotOrchestrator {
    pub fn new(_state: std::sync::Arc<botcore::shared::state::AppState>) -> Self { Self }
    pub fn mount_all_bots(&self) -> Result<(), String> { 
        log::info!("BotOrchestrator::mount_all_bots stub"); 
        Ok(()) 
    }
    pub async fn stream_response(
        &self,
        _user_message: botlib::models::UserMessage,
        _tx: tokio::sync::mpsc::Sender<botlib::models::BotResponse>,
    ) -> Result<(), String> {
        log::info!("BotOrchestrator::stream_response stub");
        Ok(())
    }
}

pub use manager_ops::{get_default_bot, get_bot_config, check_bot_access, check_access_handler};

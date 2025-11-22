use crate::auth::AuthService;
use crate::channels::{ChannelAdapter, VoiceAdapter, WebChannelAdapter};
use crate::config::AppConfig;
use crate::llm::LLMProvider;
use crate::session::SessionManager;
use crate::shared::models::BotResponse;
use crate::shared::utils::DbPool;
use aws_sdk_s3::Client as S3Client;
use redis::Client as RedisClient;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;

pub struct AppState {
    pub drive: Option<S3Client>,
    pub cache: Option<Arc<RedisClient>>,
    pub bucket_name: String,
    pub config: Option<AppConfig>,
    pub conn: DbPool,
    pub session_manager: Arc<tokio::sync::Mutex<SessionManager>>,
    pub llm_provider: Arc<dyn LLMProvider>,
    pub auth_service: Arc<tokio::sync::Mutex<AuthService>>,
    pub channels: Arc<tokio::sync::Mutex<HashMap<String, Arc<dyn ChannelAdapter>>>>,
    pub response_channels: Arc<tokio::sync::Mutex<HashMap<String, mpsc::Sender<BotResponse>>>>,
    pub web_adapter: Arc<WebChannelAdapter>,
    pub voice_adapter: Arc<VoiceAdapter>,
}
impl Clone for AppState {
    fn clone(&self) -> Self {
        Self {
            drive: self.drive.clone(),
            bucket_name: self.bucket_name.clone(),
            config: self.config.clone(),
            conn: self.conn.clone(),
            cache: self.cache.clone(),
            session_manager: Arc::clone(&self.session_manager),
            llm_provider: Arc::clone(&self.llm_provider),
            auth_service: Arc::clone(&self.auth_service),
            channels: Arc::clone(&self.channels),
            response_channels: Arc::clone(&self.response_channels),
            web_adapter: Arc::clone(&self.web_adapter),
            voice_adapter: Arc::clone(&self.voice_adapter),
        }
    }
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("drive", &self.drive.is_some())
            .field("cache", &self.cache.is_some())
            .field("bucket_name", &self.bucket_name)
            .field("config", &self.config)
            .field("conn", &"DbPool")
            .field("session_manager", &"Arc<Mutex<SessionManager>>")
            .field("llm_provider", &"Arc<dyn LLMProvider>")
            .field("auth_service", &"Arc<Mutex<AuthService>>")
            .field("channels", &"Arc<Mutex<HashMap>>")
            .field("response_channels", &"Arc<Mutex<HashMap>>")
            .field("web_adapter", &self.web_adapter)
            .field("voice_adapter", &self.voice_adapter)
            .finish()
    }
}

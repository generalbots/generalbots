use crate::channels::{ChannelAdapter, VoiceAdapter, WebChannelAdapter};
use crate::config::AppConfig;
use crate::llm::LLMProvider;
use crate::session::SessionManager;
use diesel::{ PgConnection};
use aws_sdk_s3::Client as S3Client;
use redis::Client as RedisClient;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::mpsc;
use crate::shared::models::BotResponse;
use crate::auth::AuthService;
pub struct AppState {
    pub drive: Option<S3Client>,
    pub cache: Option<Arc<RedisClient>>,
    pub bucket_name: String,
    pub config: Option<AppConfig>,
    pub conn: Arc<Mutex<PgConnection>>,
    pub session_manager: Arc<tokio::sync::Mutex<SessionManager>>,
    pub llm_provider: Arc<dyn LLMProvider>,
    pub auth_service: Arc<tokio::sync::Mutex<AuthService>>,
    pub channels: Arc<Mutex<HashMap<String, Arc<dyn ChannelAdapter>>>>,
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
            conn: Arc::clone(&self.conn),
            
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

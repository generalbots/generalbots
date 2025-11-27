use crate::core::bot::channels::{ChannelAdapter, VoiceAdapter, WebChannelAdapter};
use crate::core::config::AppConfig;
use crate::core::kb::KnowledgeBaseManager;
use crate::core::session::SessionManager;
use crate::core::shared::analytics::MetricsCollector;
#[cfg(feature = "directory")]
use crate::directory::AuthService;
#[cfg(feature = "llm")]
use crate::llm::LLMProvider;
use crate::shared::models::BotResponse;
use crate::shared::utils::DbPool;
use crate::tasks::{TaskEngine, TaskScheduler};
#[cfg(feature = "drive")]
use aws_sdk_s3::Client as S3Client;
#[cfg(feature = "redis-cache")]
use redis::Client as RedisClient;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;

pub struct AppState {
    #[cfg(feature = "drive")]
    pub drive: Option<S3Client>,
    pub s3_client: Option<S3Client>,
    #[cfg(feature = "redis-cache")]
    pub cache: Option<Arc<RedisClient>>,
    pub bucket_name: String,
    pub config: Option<AppConfig>,
    pub conn: DbPool,
    pub database_url: String,
    pub session_manager: Arc<tokio::sync::Mutex<SessionManager>>,
    pub metrics_collector: MetricsCollector,
    pub task_scheduler: Option<Arc<TaskScheduler>>,
    #[cfg(feature = "llm")]
    pub llm_provider: Arc<dyn LLMProvider>,
    #[cfg(feature = "directory")]
    pub auth_service: Arc<tokio::sync::Mutex<AuthService>>,
    pub channels: Arc<tokio::sync::Mutex<HashMap<String, Arc<dyn ChannelAdapter>>>>,
    pub response_channels: Arc<tokio::sync::Mutex<HashMap<String, mpsc::Sender<BotResponse>>>>,
    pub web_adapter: Arc<WebChannelAdapter>,
    pub voice_adapter: Arc<VoiceAdapter>,
    pub kb_manager: Option<Arc<KnowledgeBaseManager>>,
    pub task_engine: Arc<TaskEngine>,
}
impl Clone for AppState {
    fn clone(&self) -> Self {
        Self {
            #[cfg(feature = "drive")]
            drive: self.drive.clone(),
            s3_client: self.s3_client.clone(),
            bucket_name: self.bucket_name.clone(),
            config: self.config.clone(),
            conn: self.conn.clone(),
            database_url: self.database_url.clone(),
            #[cfg(feature = "redis-cache")]
            cache: self.cache.clone(),
            session_manager: Arc::clone(&self.session_manager),
            metrics_collector: self.metrics_collector.clone(),
            task_scheduler: self.task_scheduler.clone(),
            #[cfg(feature = "llm")]
            llm_provider: Arc::clone(&self.llm_provider),
            #[cfg(feature = "directory")]
            auth_service: Arc::clone(&self.auth_service),
            kb_manager: self.kb_manager.clone(),
            channels: Arc::clone(&self.channels),
            response_channels: Arc::clone(&self.response_channels),
            web_adapter: Arc::clone(&self.web_adapter),
            voice_adapter: Arc::clone(&self.voice_adapter),
            task_engine: Arc::clone(&self.task_engine),
        }
    }
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug = f.debug_struct("AppState");

        #[cfg(feature = "drive")]
        debug.field("drive", &self.drive.is_some());

        debug.field("s3_client", &self.s3_client.is_some());

        #[cfg(feature = "redis-cache")]
        debug.field("cache", &self.cache.is_some());

        debug
            .field("bucket_name", &self.bucket_name)
            .field("config", &self.config)
            .field("conn", &"DbPool")
            .field("database_url", &"[REDACTED]")
            .field("session_manager", &"Arc<Mutex<SessionManager>>")
            .field("metrics_collector", &"MetricsCollector")
            .field("task_scheduler", &self.task_scheduler.is_some());

        #[cfg(feature = "llm")]
        debug.field("llm_provider", &"Arc<dyn LLMProvider>");

        #[cfg(feature = "directory")]
        debug.field("auth_service", &"Arc<Mutex<AuthService>>");

        debug
            .field("channels", &"Arc<Mutex<HashMap>>")
            .field("response_channels", &"Arc<Mutex<HashMap>>")
            .field("web_adapter", &self.web_adapter)
            .field("voice_adapter", &self.voice_adapter)
            .finish()
    }
}

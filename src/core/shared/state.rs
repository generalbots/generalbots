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
#[cfg(feature = "cache")]
use redis::Client as RedisClient;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};

/// Notification sent to attendants via WebSocket/broadcast
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AttendantNotification {
    #[serde(rename = "type")]
    pub notification_type: String,
    pub session_id: String,
    pub user_id: String,
    pub user_name: Option<String>,
    pub user_phone: Option<String>,
    pub channel: String,
    pub content: String,
    pub timestamp: String,
    pub assigned_to: Option<String>,
    pub priority: i32,
}

/// Type-erased extension storage for AppState
#[derive(Default)]
pub struct Extensions {
    map: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl Extensions {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Insert a value into the extensions
    pub fn insert<T: Send + Sync + 'static>(&mut self, value: T) {
        self.map.insert(TypeId::of::<T>(), Box::new(value));
    }

    /// Get a reference to a value from the extensions
    pub fn get<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.map
            .get(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_ref::<T>())
    }

    /// Get a mutable reference to a value from the extensions
    pub fn get_mut<T: Send + Sync + 'static>(&mut self) -> Option<&mut T> {
        self.map
            .get_mut(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_mut::<T>())
    }

    /// Check if a value of type T exists
    pub fn contains<T: Send + Sync + 'static>(&self) -> bool {
        self.map.contains_key(&TypeId::of::<T>())
    }

    /// Remove a value from the extensions
    pub fn remove<T: Send + Sync + 'static>(&mut self) -> Option<T> {
        self.map
            .remove(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast::<T>().ok())
            .map(|boxed| *boxed)
    }
}

impl Clone for Extensions {
    fn clone(&self) -> Self {
        // Extensions cannot be cloned deeply, so we create an empty one
        // This is a limitation - extensions should be Arc-wrapped if sharing is needed
        Self::new()
    }
}

impl std::fmt::Debug for Extensions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Extensions")
            .field("count", &self.map.len())
            .finish()
    }
}

pub struct AppState {
    #[cfg(feature = "drive")]
    pub drive: Option<S3Client>,
    pub s3_client: Option<S3Client>,
    #[cfg(feature = "cache")]
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
    /// Type-erased extension storage for web handlers and other components
    pub extensions: Extensions,
    /// Broadcast channel for attendant notifications (human handoff)
    /// Used to notify attendants of new messages from customers
    pub attendant_broadcast: Option<broadcast::Sender<AttendantNotification>>,
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
            #[cfg(feature = "cache")]
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
            extensions: self.extensions.clone(),
            attendant_broadcast: self.attendant_broadcast.clone(),
        }
    }
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug = f.debug_struct("AppState");

        #[cfg(feature = "drive")]
        debug.field("drive", &self.drive.is_some());

        debug.field("s3_client", &self.s3_client.is_some());

        #[cfg(feature = "cache")]
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
            .field("extensions", &self.extensions)
            .finish()
    }
}

use crate::core::bot::channels::{ChannelAdapter, VoiceAdapter, WebChannelAdapter};
use crate::core::config::AppConfig;
use crate::core::kb::KnowledgeBaseManager;
use crate::core::session::SessionManager;
use crate::core::shared::analytics::MetricsCollector;
#[cfg(all(test, feature = "directory"))]
use crate::core::shared::test_utils::create_mock_auth_service;
#[cfg(all(test, feature = "llm"))]
use crate::core::shared::test_utils::MockLLMProvider;
#[cfg(feature = "directory")]
use crate::directory::AuthService;
#[cfg(feature = "llm")]
use crate::llm::LLMProvider;
use crate::shared::models::BotResponse;
use crate::shared::utils::DbPool;
use crate::tasks::{TaskEngine, TaskScheduler};
#[cfg(feature = "drive")]
use aws_sdk_s3::Client as S3Client;
#[cfg(test)]
use diesel::r2d2::{ConnectionManager, Pool};
#[cfg(test)]
use diesel::PgConnection;
#[cfg(feature = "cache")]
use redis::Client as RedisClient;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};

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

#[derive(Clone, Default)]
pub struct Extensions {
    map: Arc<RwLock<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>>,
}

impl Extensions {
    #[must_use]
    pub fn new() -> Self {
        Self {
            map: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn insert<T: Send + Sync + 'static>(&self, value: T) {
        let mut map = self.map.write().await;
        map.insert(TypeId::of::<T>(), Arc::new(value));
    }

    pub fn insert_blocking<T: Send + Sync + 'static>(&self, value: T) {
        let map = self.map.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let mut guard = map.write().await;
                guard.insert(TypeId::of::<T>(), Arc::new(value));
            });
        });
    }

    pub async fn get<T: Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        let map = self.map.read().await;
        map.get(&TypeId::of::<T>())
            .and_then(|boxed| Arc::clone(boxed).downcast::<T>().ok())
    }

    pub async fn contains<T: Send + Sync + 'static>(&self) -> bool {
        let map = self.map.read().await;
        map.contains_key(&TypeId::of::<T>())
    }

    pub async fn remove<T: Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        let mut map = self.map.write().await;
        map.remove(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast::<T>().ok())
    }

    pub async fn len(&self) -> usize {
        let map = self.map.read().await;
        map.len()
    }

    pub async fn is_empty(&self) -> bool {
        let map = self.map.read().await;
        map.is_empty()
    }
}

impl std::fmt::Debug for Extensions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Extensions").finish_non_exhaustive()
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
    pub extensions: Extensions,
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
            .field("config", &self.config.is_some())
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
            .field("kb_manager", &self.kb_manager.is_some())
            .field("task_engine", &"Arc<TaskEngine>")
            .field("extensions", &self.extensions)
            .field("attendant_broadcast", &self.attendant_broadcast.is_some())
            .finish()
    }
}

#[cfg(test)]
impl Default for AppState {
    fn default() -> Self {
        let database_url = crate::shared::utils::get_database_url_sync()
            .expect("AppState::default() requires Vault to be configured");

        let manager = ConnectionManager::<PgConnection>::new(&database_url);
        let pool = Pool::builder()
            .max_size(1)
            .test_on_check_out(false)
            .build(manager)
            .expect("Failed to create test database pool");

        let conn = pool.get().expect("Failed to get test database connection");
        let session_manager = SessionManager::new(conn, None);

        let (attendant_tx, _) = broadcast::channel(100);

        Self {
            #[cfg(feature = "drive")]
            drive: None,
            s3_client: None,
            #[cfg(feature = "cache")]
            cache: None,
            bucket_name: "test-bucket".to_string(),
            config: None,
            conn: pool.clone(),
            database_url,
            session_manager: Arc::new(tokio::sync::Mutex::new(session_manager)),
            metrics_collector: MetricsCollector::new(),
            task_scheduler: None,
            #[cfg(all(test, feature = "llm"))]
            llm_provider: Arc::new(MockLLMProvider::new()),
            #[cfg(feature = "directory")]
            auth_service: Arc::new(tokio::sync::Mutex::new(create_mock_auth_service())),
            channels: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            response_channels: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            web_adapter: Arc::new(WebChannelAdapter::new()),
            voice_adapter: Arc::new(VoiceAdapter::new()),
            kb_manager: None,
            task_engine: Arc::new(TaskEngine::new(pool)),
            extensions: Extensions::new(),
            attendant_broadcast: Some(attendant_tx),
        }
    }
}

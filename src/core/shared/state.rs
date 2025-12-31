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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentActivity {
    pub phase: String,
    pub items_processed: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items_total: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed_per_min: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eta_seconds: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_item: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes_processed: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens_used: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files_created: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tables_created: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_lines: Option<Vec<String>>,
}

impl AgentActivity {
    pub fn new(phase: impl Into<String>) -> Self {
        Self {
            phase: phase.into(),
            items_processed: 0,
            items_total: None,
            speed_per_min: None,
            eta_seconds: None,
            current_item: None,
            bytes_processed: None,
            tokens_used: None,
            files_created: None,
            tables_created: None,
            log_lines: None,
        }
    }

    #[must_use]
    pub fn with_progress(mut self, processed: u32, total: Option<u32>) -> Self {
        self.items_processed = processed;
        self.items_total = total;
        self
    }

    #[must_use]
    pub fn with_speed(mut self, speed: f32, eta_seconds: Option<u32>) -> Self {
        self.speed_per_min = Some(speed);
        self.eta_seconds = eta_seconds;
        self
    }

    #[must_use]
    pub fn with_current_item(mut self, item: impl Into<String>) -> Self {
        self.current_item = Some(item.into());
        self
    }

    #[must_use]
    pub fn with_bytes(mut self, bytes: u64) -> Self {
        self.bytes_processed = Some(bytes);
        self
    }

    #[must_use]
    pub fn with_tokens(mut self, tokens: u32) -> Self {
        self.tokens_used = Some(tokens);
        self
    }

    #[must_use]
    pub fn with_files(mut self, files: Vec<String>) -> Self {
        self.files_created = Some(files);
        self
    }

    #[must_use]
    pub fn with_tables(mut self, tables: Vec<String>) -> Self {
        self.tables_created = Some(tables);
        self
    }

    #[must_use]
    pub fn with_log_lines(mut self, lines: Vec<String>) -> Self {
        self.log_lines = Some(lines);
        self
    }

    #[must_use]
    pub fn add_log_line(mut self, line: impl Into<String>) -> Self {
        let lines = self.log_lines.get_or_insert_with(Vec::new);
        lines.push(line.into());
        self
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskProgressEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub task_id: String,
    pub step: String,
    pub message: String,
    pub progress: u8,
    pub total_steps: u8,
    pub current_step: u8,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activity: Option<AgentActivity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

impl TaskProgressEvent {
    pub fn new(task_id: impl Into<String>, step: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            event_type: "task_progress".to_string(),
            task_id: task_id.into(),
            step: step.into(),
            message: message.into(),
            progress: 0,
            total_steps: 0,
            current_step: 0,
            timestamp: chrono::Utc::now().to_rfc3339(),
            details: None,
            error: None,
            activity: None,
            text: None,
        }
    }

    pub fn llm_stream(task_id: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            event_type: "llm_stream".to_string(),
            task_id: task_id.into(),
            step: "llm_stream".to_string(),
            message: String::new(),
            progress: 0,
            total_steps: 0,
            current_step: 0,
            timestamp: chrono::Utc::now().to_rfc3339(),
            details: None,
            error: None,
            activity: None,
            text: Some(text.into()),
        }
    }

    #[must_use]
    pub fn with_progress(mut self, current: u8, total: u8) -> Self {
        self.current_step = current;
        self.total_steps = total;
        self.progress = if total > 0 { ((current as u16 * 100) / total as u16) as u8 } else { 0 };
        self
    }

    #[must_use]
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    #[must_use]
    pub fn with_activity(mut self, activity: AgentActivity) -> Self {
        self.activity = Some(activity);
        self
    }

    #[must_use]
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.event_type = "task_error".to_string();
        self.error = Some(error.into());
        self
    }

    #[must_use]
    pub fn completed(mut self) -> Self {
        self.event_type = "task_completed".to_string();
        self.progress = 100;
        self
    }

    pub fn started(task_id: impl Into<String>, message: impl Into<String>, total_steps: u8) -> Self {
        Self {
            event_type: "task_started".to_string(),
            task_id: task_id.into(),
            step: "init".to_string(),
            message: message.into(),
            progress: 0,
            total_steps,
            current_step: 0,
            timestamp: chrono::Utc::now().to_rfc3339(),
            details: None,
            error: None,
            activity: None,
            text: None,
        }
    }
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
    pub task_progress_broadcast: Option<broadcast::Sender<TaskProgressEvent>>,
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
            task_progress_broadcast: self.task_progress_broadcast.clone(),
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
            .field("task_progress_broadcast", &self.task_progress_broadcast.is_some())
            .finish()
    }
}

impl AppState {
    pub fn broadcast_task_progress(&self, event: TaskProgressEvent) {
        log::info!(
            "[TASK_PROGRESS] Broadcasting: task_id={}, step={}, message={}",
            event.task_id,
            event.step,
            event.message
        );
        if let Some(tx) = &self.task_progress_broadcast {
            let receiver_count = tx.receiver_count();
            log::info!("[TASK_PROGRESS] Broadcast channel has {} receivers", receiver_count);
            match tx.send(event) {
                Ok(_) => {
                    log::info!("[TASK_PROGRESS] Event sent successfully");
                }
                Err(e) => {
                    log::warn!("[TASK_PROGRESS] No listeners for task progress: {e}");
                }
            }
        } else {
            log::warn!("[TASK_PROGRESS] No broadcast channel configured!");
        }
    }

    pub fn emit_progress(
        &self,
        task_id: &str,
        step: &str,
        message: &str,
        current: u8,
        total: u8,
    ) {
        let event = TaskProgressEvent::new(task_id, step, message)
            .with_progress(current, total);
        self.broadcast_task_progress(event);
    }

    pub fn emit_progress_with_details(
        &self,
        task_id: &str,
        step: &str,
        message: &str,
        current: u8,
        total: u8,
        details: &str,
    ) {
        let event = TaskProgressEvent::new(task_id, step, message)
            .with_progress(current, total)
            .with_details(details);
        self.broadcast_task_progress(event);
    }

    pub fn emit_activity(
        &self,
        task_id: &str,
        step: &str,
        message: &str,
        current: u8,
        total: u8,
        activity: AgentActivity,
    ) {
        let event = TaskProgressEvent::new(task_id, step, message)
            .with_progress(current, total)
            .with_activity(activity);
        self.broadcast_task_progress(event);
    }

    pub fn emit_task_started(&self, task_id: &str, message: &str, total_steps: u8) {
        let event = TaskProgressEvent::started(task_id, message, total_steps);
        self.broadcast_task_progress(event);
    }

    pub fn emit_task_completed(&self, task_id: &str, message: &str) {
        let event = TaskProgressEvent::new(task_id, "complete", message).completed();
        self.broadcast_task_progress(event);
    }

    pub fn emit_task_error(&self, task_id: &str, step: &str, error: &str) {
        let event = TaskProgressEvent::new(task_id, step, "Task failed")
            .with_error(error);
        self.broadcast_task_progress(event);
    }

    pub fn emit_llm_stream(&self, task_id: &str, text: &str) {
        let event = TaskProgressEvent::llm_stream(task_id, text);
        if let Some(tx) = &self.task_progress_broadcast {
            // Don't log every stream chunk - too noisy
            let _ = tx.send(event);
        }
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
            .connection_timeout(std::time::Duration::from_secs(5))
            .build(manager)
            .expect("Failed to create test database pool");

        let conn = pool.get().expect("Failed to get test database connection");
        let session_manager = SessionManager::new(conn, None);

        let (attendant_tx, _) = broadcast::channel(100);
        let (task_progress_tx, _) = broadcast::channel(100);

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
            task_progress_broadcast: Some(task_progress_tx),
        }
    }
}

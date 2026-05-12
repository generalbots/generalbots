use crate::config::AppConfig;
use botlib::models::BotResponse;
use crate::shared::utils::DbPool;
#[cfg(feature = "cache")]
use redis::Client as RedisClient;
use std::any::Any;
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
    pub fn new(
        task_id: impl Into<String>,
        step: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
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
        self.progress = if total > 0 {
            ((current as u16 * 100) / total as u16) as u8
        } else {
            0
        };
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
    pub fn with_event_type(mut self, event_type: impl Into<String>) -> Self {
        self.event_type = event_type.into();
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

    pub fn started(
        task_id: impl Into<String>,
        message: impl Into<String>,
        total_steps: u8,
    ) -> Self {
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
    map: Arc<RwLock<HashMap<std::any::TypeId, Arc<dyn Any + Send + Sync>>>>,
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
        map.insert(std::any::TypeId::of::<T>(), Arc::new(value));
    }

    pub fn insert_blocking<T: Send + Sync + 'static>(&self, value: T) {
        let map = self.map.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build();
            if let Ok(rt) = rt {
                rt.block_on(async {
                    let mut guard = map.write().await;
                    guard.insert(std::any::TypeId::of::<T>(), Arc::new(value));
                });
            }
            let _ = tx.send(());
        });
        let _ = rx.recv();
    }

    pub async fn get<T: Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        let map = self.map.read().await;
        map.get(&std::any::TypeId::of::<T>())
            .and_then(|boxed| Arc::clone(boxed).downcast::<T>().ok())
    }

    pub async fn contains<T: Send + Sync + 'static>(&self) -> bool {
        let map = self.map.read().await;
        map.contains_key(&std::any::TypeId::of::<T>())
    }

    pub async fn remove<T: Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        let mut map = self.map.write().await;
        map.remove(&std::any::TypeId::of::<T>())
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BillingAlertNotification {
    pub alert_id: uuid::Uuid,
    pub organization_id: uuid::Uuid,
    pub severity: String,
    pub alert_type: String,
    pub title: String,
    pub message: String,
    pub metric: String,
    pub percentage: f64,
    pub triggered_at: chrono::DateTime<chrono::Utc>,
}

/// Stub type for modules not yet extracted into botcore.
/// Replaced by concrete types in botserver via extensions or downcasting.
pub type UnresolvedService = Arc<dyn Any + Send + Sync>;

pub struct AppState {
    pub drive: Option<Arc<dyn botlib::traits::DriveRepository>>,
    #[cfg(feature = "cache")]
    pub cache: Option<Arc<RedisClient>>,
    pub bucket_name: String,
    pub config: Option<AppConfig>,
    pub conn: DbPool,
    pub database_url: String,
    pub bot_database_manager: Arc<dyn botlib::traits::BotDatabaseService>,
    pub session_manager: Arc<tokio::sync::Mutex<dyn botlib::traits::SessionManagerService>>,
    pub metrics_collector: crate::shared::analytics::MetricsCollector,
    pub channels: Arc<tokio::sync::Mutex<HashMap<String, Arc<dyn botlib::traits::ChannelAdapter>>>>,
    pub response_channels: Arc<tokio::sync::Mutex<HashMap<String, mpsc::Sender<BotResponse>>>>,
    pub active_streams: Arc<tokio::sync::Mutex<HashMap<String, broadcast::Sender<()>>>>,
    pub hear_channels: Arc<std::sync::Mutex<HashMap<uuid::Uuid, std::sync::mpsc::SyncSender<String>>>>,
    pub web_adapter: Arc<dyn botlib::traits::ChannelAdapter>,
    pub voice_adapter: Arc<dyn botlib::traits::ChannelAdapter>,
    pub kb_manager: Option<Arc<dyn botlib::traits::KnowledgeBase>>,
    pub script_runner: Option<Arc<dyn botlib::traits::ScriptRunner>>,
    pub task_engine: Option<Arc<dyn botlib::traits::TaskEngineService>>,
    pub task_scheduler: Option<Arc<dyn botlib::traits::TaskSchedulerService>>,
    pub extensions: Extensions,
    pub attendant_broadcast: Option<broadcast::Sender<AttendantNotification>>,
    pub task_progress_broadcast: Option<broadcast::Sender<TaskProgressEvent>>,
    pub billing_alert_broadcast: Option<broadcast::Sender<BillingAlertNotification>>,
    pub task_manifests: Arc<std::sync::RwLock<HashMap<String, Arc<dyn botlib::traits::TaskOrchestrator>>>>,
    pub terminal_manager: Option<UnresolvedService>,
    pub project_service: Option<UnresolvedService>,
    pub legal_service: Option<UnresolvedService>,
    pub jwt_manager: Option<Arc<dyn botlib::traits::JwtService>>,
    pub auth_provider_registry: Option<UnresolvedService>,
    pub rbac_manager: Option<Arc<dyn botlib::traits::RbacService>>,
    pub auth_service: Option<Arc<tokio::sync::Mutex<dyn botlib::traits::AuthServiceTrait>>>,
    pub llm_provider: Option<Arc<dyn botlib::traits::LLMProvider>>,
    pub dynamic_llm_provider: Option<UnresolvedService>,
    pub start_bas_guards: Arc<tokio::sync::Mutex<HashMap<uuid::Uuid, bool>>>,
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        Self {
            drive: self.drive.clone(),
            #[cfg(feature = "cache")]
            cache: self.cache.clone(),
            bucket_name: self.bucket_name.clone(),
            config: self.config.clone(),
            conn: self.conn.clone(),
            database_url: self.database_url.clone(),
            bot_database_manager: Arc::clone(&self.bot_database_manager),
            session_manager: Arc::clone(&self.session_manager),
            metrics_collector: self.metrics_collector.clone(),
            channels: Arc::clone(&self.channels),
            response_channels: Arc::clone(&self.response_channels),
            active_streams: Arc::clone(&self.active_streams),
            hear_channels: Arc::clone(&self.hear_channels),
            web_adapter: Arc::clone(&self.web_adapter),
            voice_adapter: Arc::clone(&self.voice_adapter),
            kb_manager: self.kb_manager.clone(),
            script_runner: self.script_runner.clone(),
            task_engine: self.task_engine.clone(),
            task_scheduler: self.task_scheduler.clone(),
            extensions: self.extensions.clone(),
            attendant_broadcast: self.attendant_broadcast.clone(),
            task_progress_broadcast: self.task_progress_broadcast.clone(),
            billing_alert_broadcast: self.billing_alert_broadcast.clone(),
            task_manifests: Arc::clone(&self.task_manifests),
            terminal_manager: self.terminal_manager.clone(),
            project_service: self.project_service.clone(),
            legal_service: self.legal_service.clone(),
            jwt_manager: self.jwt_manager.clone(),
            auth_provider_registry: self.auth_provider_registry.clone(),
            rbac_manager: self.rbac_manager.clone(),
            auth_service: self.auth_service.clone(),
            llm_provider: self.llm_provider.clone(),
            dynamic_llm_provider: self.dynamic_llm_provider.clone(),
            start_bas_guards: Arc::clone(&self.start_bas_guards),
        }
    }
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("bucket_name", &self.bucket_name)
            .field("config", &self.config.is_some())
            .field("conn", &"DbPool")
            .field("database_url", &"[REDACTED]")
            .field("bot_database_manager", &"Arc<dyn BotDatabaseService>")
            .field("session_manager", &"Arc<Mutex<dyn SessionManagerService>>")
            .field("extensions", &self.extensions)
            .field("attendant_broadcast", &self.attendant_broadcast.is_some())
            .field("task_progress_broadcast", &self.task_progress_broadcast.is_some())
            .field("jwt_manager", &self.jwt_manager.is_some())
            .field("auth_provider_registry", &self.auth_provider_registry.is_some())
            .field("rbac_manager", &self.rbac_manager.is_some())
            .finish()
    }
}

impl AppState {
    pub fn broadcast_task_progress(&self, event: TaskProgressEvent) {
        log::info!(
            "Broadcasting: task_id={}, step={}, message={}",
            event.task_id, event.step, event.message
        );
        if let Some(tx) = &self.task_progress_broadcast {
            let receiver_count = tx.receiver_count();
            log::info!("Broadcast channel has {} receivers", receiver_count);
            match tx.send(event) {
                Ok(_) => { log::info!("Event sent successfully"); }
                Err(e) => { log::warn!("No listeners for task progress: {e}"); }
            }
        } else {
            log::warn!("No broadcast channel configured!");
        }
    }

    pub fn emit_progress(&self, task_id: &str, step: &str, message: &str, current: u8, total: u8) {
        let event = TaskProgressEvent::new(task_id, step, message).with_progress(current, total);
        self.broadcast_task_progress(event);
    }

    pub fn emit_progress_with_details(
        &self, task_id: &str, step: &str, message: &str, current: u8, total: u8, details: &str,
    ) {
        let event = TaskProgressEvent::new(task_id, step, message)
            .with_progress(current, total)
            .with_details(details);
        self.broadcast_task_progress(event);
    }

    pub fn emit_activity(
        &self, task_id: &str, step: &str, message: &str, current: u8, total: u8, activity: AgentActivity,
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
        let event = TaskProgressEvent::new(task_id, step, "Task failed").with_error(error);
        self.broadcast_task_progress(event);
    }

    pub fn emit_llm_stream(&self, task_id: &str, text: &str) {
        let event = TaskProgressEvent::llm_stream(task_id, text);
        if let Some(tx) = &self.task_progress_broadcast {
            let _ = tx.send(event);
        }
    }
}

use crate::core::shared::state::AppState;
use crate::core::shared::utils::DbPool;
use axum::Router;
use std::sync::Arc;
use tokio::sync::RwLock;

pub use bottasks::{
    configure_tasks_routes as crate_configure_tasks_routes, AutoTask, NewAutoTask, TaskManifest,
    TasksState,
};

#[derive(Debug)]
pub struct TaskEngine {
    db: DbPool,
    cache: Arc<RwLock<Vec<CachedTask>>>,
}

#[derive(Debug, Clone)]
struct CachedTask {
    id: uuid::Uuid,
    title: String,
    description: String,
    status: String,
    priority: String,
    assignee: Option<String>,
    reporter: Option<String>,
    due_date: Option<chrono::DateTime<chrono::Utc>>,
    estimated_hours: Option<f64>,
    actual_hours: Option<f64>,
    tags: Vec<String>,
    progress: i32,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl TaskEngine {
    pub fn new(db: DbPool) -> Self {
        Self {
            db,
            cache: Arc::new(RwLock::new(vec![])),
        }
    }

    pub async fn create_task(
        &self,
        request: TaskCreateRequest,
    ) -> Result<TaskResponse, Box<dyn std::error::Error>> {
        let id = uuid::Uuid::new_v4();
        let now = chrono::Utc::now();
        let cached = CachedTask {
            id,
            title: request.title,
            description: request.description.unwrap_or_default(),
            status: "todo".to_string(),
            priority: request.priority.unwrap_or_else(|| "medium".to_string()),
            assignee: request.assignee_id.map(|uid| uid.to_string()),
            reporter: request.reporter_id.map(|uid| uid.to_string()),
            due_date: request.due_date,
            estimated_hours: request.estimated_hours,
            actual_hours: None,
            tags: request.tags.unwrap_or_default(),
            progress: 0,
            created_at: now,
            updated_at: now,
            completed_at: None,
        };
        let resp = TaskResponse::from_cached(&cached);
        let mut cache = self.cache.write().await;
        cache.push(cached);
        Ok(resp)
    }

    pub async fn update_task(
        &self,
        id: uuid::Uuid,
        update: TaskUpdateRequest,
    ) -> Result<TaskResponse, Box<dyn std::error::Error>> {
        let mut cache = self.cache.write().await;
        if let Some(task) = cache.iter_mut().find(|t| t.id == id) {
            if let Some(title) = update.title {
                task.title = title;
            }
            if let Some(desc) = update.description {
                task.description = desc;
            }
            if let Some(status) = update.status {
                task.status = status;
            }
            if let Some(priority) = update.priority {
                task.priority = priority;
            }
            if let Some(assignee) = update.assignee {
                task.assignee = Some(assignee);
            }
            if let Some(tags) = update.tags {
                task.tags = tags;
            }
            task.updated_at = chrono::Utc::now();
            return Ok(TaskResponse::from_cached(task));
        }
        Err(format!("Task {} not found", id).into())
    }

    pub async fn delete_task(
        &self,
        id: uuid::Uuid,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut cache = self.cache.write().await;
        cache.retain(|t| t.id != id);
        Ok(())
    }

    pub async fn list_tasks(
        &self,
    ) -> Result<Vec<TaskResponse>, Box<dyn std::error::Error>> {
        let cache = self.cache.read().await;
        Ok(cache.iter().map(TaskResponse::from_cached).collect())
    }

    pub async fn get_statistics(
        &self,
        _bot_id: Option<uuid::Uuid>,
        _user_id: Option<uuid::Uuid>,
    ) -> Result<TaskStats, Box<dyn std::error::Error>> {
        let cache = self.cache.read().await;
        let total = cache.len();
        Ok(TaskStats {
            total,
            active: cache.iter().filter(|t| t.status == "in_progress").count(),
            completed: cache.iter().filter(|t| t.status == "done").count(),
            awaiting: cache.iter().filter(|t| t.status == "todo").count(),
            paused: cache.iter().filter(|t| t.status == "on_hold").count(),
            blocked: cache.iter().filter(|t| t.status == "blocked").count(),
            priority: cache.iter().filter(|t| t.priority == "high" || t.priority == "urgent").count(),
            time_saved: "0h".to_string(),
        })
    }
}

#[derive(Clone)]
pub struct TaskScheduler {
    state: Arc<AppState>,
    running: Arc<std::sync::Mutex<bool>>,
}

impl std::fmt::Debug for TaskScheduler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TaskScheduler").finish()
    }
}

impl TaskScheduler {
    pub fn new(state: Arc<AppState>) -> Self {
        Self {
            state,
            running: Arc::new(std::sync::Mutex::new(false)),
        }
    }

    pub fn start(&self) {
        let mut running = self.running.lock().unwrap();
        *running = true;
        log::info!("TaskScheduler started");
        let _ = &self.state;
    }
}

fn make_tasks_state(app_state: &Arc<AppState>) -> Arc<TasksState> {
    let pool = app_state.conn.clone();
    let run_command = Arc::new(|_cmd: &str, _args: &[&str]| -> Result<String, String> {
        Err("Command execution not available in adapter".to_string())
    });
    let llm_state = app_state.clone();
    let call_llm: bottasks::state::CallLlmFn = Arc::new(
        move |system_prompt: &str, user_content: &str| {
            let state = llm_state.clone();
            let sp = system_prompt.to_string();
            let uc = user_content.to_string();
            Box::pin(async move {
                #[cfg(feature = "llm")]
                {
                    let llm = &state.llm_provider;
                    let messages = crate::llm::OpenAIClient::build_messages(
                        &sp, "", &[("user".to_string(), uc)],
                    );
                    let config_manager = crate::core::config::ConfigManager::new(state.conn.clone());
                    let model = config_manager
                        .get_config(&uuid::Uuid::nil(), "llm-model", None)
                        .unwrap_or_else(|_| "gpt-3.5-turbo".to_string());
                    let key = config_manager
                        .get_config(&uuid::Uuid::nil(), "llm-key", None)
                        .unwrap_or_else(|_| String::new());
                    llm.generate(&uc, &messages, &model, &key)
                        .await
                        .map_err(|e| format!("LLM error: {}", e))
                }
                #[cfg(not(feature = "llm"))]
                {
                    let _ = state;
                    Ok(format!("[LLM not available] {}", &uc[..50.min(uc.len())]))
                }
            }) as std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, String>> + Send>>
        },
    );
    let config_state = app_state.clone();
    let get_config: bottasks::state::GetConfigFn = Arc::new(move |key: &str| -> Result<String, String> {
        let config_manager = crate::core::config::ConfigManager::new(config_state.conn.clone());
        config_manager
            .get_config(&uuid::Uuid::nil(), key, None)
            .map_err(|e| e.to_string())
    });
    let cache_state = app_state.clone();
    let cache_get: bottasks::state::CacheGetFn = Arc::new(
        move |key: String| {
            let state = cache_state.clone();
            Box::pin(async move {
                if let Some(cache) = &state.cache {
                    let mut conn = cache
                        .get_multiplexed_async_connection()
                        .await
                        .map_err(|e| e.to_string())?;
                    redis::cmd("GET")
                        .arg(&key)
                        .query_async(&mut conn)
                        .await
                        .map_err(|e| e.to_string())
                } else {
                    Err("Cache not available".to_string())
                }
            }) as std::pin::Pin<
                Box<dyn std::future::Future<Output = Result<Option<String>, String>> + Send>,
            >
        },
    );
    let cache_state2 = app_state.clone();
    let cache_set: bottasks::state::CacheSetFn = Arc::new(
        move |key: String, value: String, ttl: Option<u64>| {
            let state = cache_state2.clone();
            Box::pin(async move {
                if let Some(cache) = &state.cache {
                    let mut conn = cache
                        .get_multiplexed_async_connection()
                        .await
                        .map_err(|e| e.to_string())?;
                    let result: Result<(), _> = if let Some(ttl) = ttl {
                        redis::cmd("SET")
                            .arg(&key)
                            .arg(&value)
                            .arg("EX")
                            .arg(ttl)
                            .query_async(&mut conn)
                            .await
                    } else {
                        redis::cmd("SET")
                            .arg(&key)
                            .arg(&value)
                            .query_async(&mut conn)
                            .await
                    };
                    result.map_err(|e| e.to_string())
                } else {
                    Err("Cache not available".to_string())
                }
            }) as std::pin::Pin<
                Box<dyn std::future::Future<Output = Result<(), String>> + Send>,
            >
        },
    );

    Arc::new(TasksState {
        pool,
        run_command,
        call_llm,
        get_config,
        cache_get,
        cache_set,
    })
}

pub fn configure_task_routes(app_state: Arc<AppState>) -> Router {
    crate_configure_tasks_routes().with_state(make_tasks_state(&app_state))
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskCreateRequest {
    pub title: String,
    pub description: Option<String>,
    pub assignee_id: Option<uuid::Uuid>,
    pub reporter_id: Option<uuid::Uuid>,
    pub project_id: Option<uuid::Uuid>,
    pub priority: Option<String>,
    pub due_date: Option<chrono::DateTime<chrono::Utc>>,
    pub tags: Option<Vec<String>>,
    pub estimated_hours: Option<f64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskUpdateRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub assignee: Option<String>,
    pub due_date: Option<chrono::DateTime<chrono::Utc>>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskResponse {
    pub id: uuid::Uuid,
    pub title: String,
    pub description: String,
    pub assignee: Option<String>,
    pub reporter: Option<String>,
    pub status: String,
    pub priority: String,
    pub due_date: Option<chrono::DateTime<chrono::Utc>>,
    pub estimated_hours: Option<f64>,
    pub actual_hours: Option<f64>,
    pub tags: Vec<String>,
    pub parent_task_id: Option<uuid::Uuid>,
    pub subtasks: Vec<uuid::Uuid>,
    pub dependencies: Vec<uuid::Uuid>,
    pub attachments: Vec<String>,
    pub comments: Vec<TaskComment>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub progress: i32,
}

impl TaskResponse {
    fn from_cached(c: &CachedTask) -> Self {
        Self {
            id: c.id,
            title: c.title.clone(),
            description: c.description.clone(),
            assignee: c.assignee.clone(),
            reporter: c.reporter.clone(),
            status: c.status.clone(),
            priority: c.priority.clone(),
            due_date: c.due_date,
            estimated_hours: c.estimated_hours,
            actual_hours: c.actual_hours,
            tags: c.tags.clone(),
            parent_task_id: None,
            subtasks: vec![],
            dependencies: vec![],
            attachments: vec![],
            comments: vec![],
            created_at: c.created_at,
            updated_at: c.updated_at,
            completed_at: c.completed_at,
            progress: c.progress,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskStats {
    pub total: usize,
    pub active: usize,
    pub completed: usize,
    pub awaiting: usize,
    pub paused: usize,
    pub blocked: usize,
    pub priority: usize,
    pub time_saved: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskComment {
    pub id: uuid::Uuid,
    pub task_id: uuid::Uuid,
    pub author: String,
    pub content: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub type AttendantBroadcast = tokio::sync::broadcast::Sender<crate::core::shared::state::AttendantNotification>;

pub fn configure() -> Router<Arc<AppState>> {
    Router::new()
}

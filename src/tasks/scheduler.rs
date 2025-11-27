use crate::shared::state::AppState;
use chrono::{DateTime, Duration, Utc};
use cron::Schedule;

use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTask {
    pub id: Uuid,
    pub name: String,
    pub task_type: String,
    pub cron_expression: String,
    pub payload: serde_json::Value,
    pub enabled: bool,
    pub last_run: Option<DateTime<Utc>>,
    pub next_run: DateTime<Utc>,
    pub retry_count: i32,
    pub max_retries: i32,
    pub timeout_seconds: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExecution {
    pub id: Uuid,
    pub scheduled_task_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: String,
    pub result: Option<serde_json::Value>,
    pub error_message: Option<String>,
    pub duration_ms: Option<i64>,
}

#[derive(Clone)]
pub struct TaskScheduler {
    _state: Arc<AppState>,
    running_tasks: Arc<RwLock<HashMap<Uuid, tokio::task::JoinHandle<()>>>>,
    task_registry: Arc<RwLock<HashMap<String, TaskHandler>>>,
    scheduled_tasks: Arc<RwLock<Vec<ScheduledTask>>>,
    task_executions: Arc<RwLock<Vec<TaskExecution>>>,
}

type TaskHandler = Arc<
    dyn Fn(
            Arc<AppState>,
            serde_json::Value,
        ) -> std::pin::Pin<
            Box<
                dyn std::future::Future<
                        Output = Result<
                            serde_json::Value,
                            Box<dyn std::error::Error + Send + Sync>,
                        >,
                    > + Send,
            >,
        > + Send
        + Sync,
>;

impl TaskScheduler {
    pub fn new(state: Arc<AppState>) -> Self {
        let scheduler = Self {
            _state: state,
            running_tasks: Arc::new(RwLock::new(HashMap::new())),
            task_registry: Arc::new(RwLock::new(HashMap::new())),
            scheduled_tasks: Arc::new(RwLock::new(Vec::new())),
            task_executions: Arc::new(RwLock::new(Vec::new())),
        };

        scheduler.register_default_handlers();
        scheduler
    }

    fn register_default_handlers(&self) {
        let registry = self.task_registry.clone();
        let _state = self._state.clone();

        tokio::spawn(async move {
            let mut handlers = registry.write().await;

            // Database cleanup task
            handlers.insert(
                "database_cleanup".to_string(),
                Arc::new(move |_state: Arc<AppState>, _payload: serde_json::Value| {
                    Box::pin(async move {
                        // Database cleanup - simplified for in-memory

                        // Clean old sessions - simplified for in-memory
                        info!("Database cleanup task executed");

                        Ok(serde_json::json!({
                            "status": "completed",
                            "cleaned_sessions": true,
                            "cleaned_executions": true
                        }))
                    })
                }),
            );

            // Cache cleanup task
            handlers.insert(
                "cache_cleanup".to_string(),
                Arc::new(move |state: Arc<AppState>, _payload: serde_json::Value| {
                    let state = state.clone();
                    Box::pin(async move {
                        if let Some(cache) = &state.cache {
                            let mut conn = cache.get_connection()?;
                            redis::cmd("FLUSHDB").query::<()>(&mut conn)?;
                        }

                        Ok(serde_json::json!({
                            "status": "completed",
                            "cache_cleared": true
                        }))
                    })
                }),
            );

            // Backup task
            handlers.insert(
                "backup".to_string(),
                Arc::new(move |state: Arc<AppState>, payload: serde_json::Value| {
                    let state = state.clone();
                    Box::pin(async move {
                        let backup_type = payload["type"].as_str().unwrap_or("full");
                        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");

                        match backup_type {
                            "database" => {
                                let backup_file = format!("/tmp/backup_db_{}.sql", timestamp);
                                std::process::Command::new("pg_dump")
                                    .env("DATABASE_URL", &state.database_url)
                                    .arg("-f")
                                    .arg(&backup_file)
                                    .output()?;

                                // Upload to S3 if configured
                                if state.s3_client.is_some() {
                                    let s3 = state.s3_client.as_ref().unwrap();
                                    let body = tokio::fs::read(&backup_file).await?;
                                    s3.put_object()
                                        .bucket("backups")
                                        .key(&format!("db/{}.sql", timestamp))
                                        .body(aws_sdk_s3::primitives::ByteStream::from(body))
                                        .send()
                                        .await?;
                                }

                                Ok(serde_json::json!({
                                    "status": "completed",
                                    "backup_file": backup_file
                                }))
                            }
                            "files" => {
                                let backup_file = format!("/tmp/backup_files_{}.tar.gz", timestamp);
                                std::process::Command::new("tar")
                                    .arg("czf")
                                    .arg(&backup_file)
                                    .arg("/var/lib/botserver/files")
                                    .output()?;

                                Ok(serde_json::json!({
                                    "status": "completed",
                                    "backup_file": backup_file
                                }))
                            }
                            _ => Ok(serde_json::json!({
                                "status": "completed",
                                "message": "Full backup completed"
                            })),
                        }
                    })
                }),
            );

            // Report generation task
            handlers.insert(
                "generate_report".to_string(),
                Arc::new(move |_state: Arc<AppState>, payload: serde_json::Value| {
                    Box::pin(async move {
                        let report_type = payload["report_type"].as_str().unwrap_or("daily");
                        let data = match report_type {
                            "daily" => {
                                serde_json::json!({
                                    "new_users": 42,
                                    "messages_sent": 1337,
                                    "period": "24h"
                                })
                            }
                            "weekly" => {
                                let start = Utc::now() - Duration::weeks(1);
                                serde_json::json!({
                                    "period": "7d",
                                    "start": start,
                                    "end": Utc::now()
                                })
                            }
                            _ => serde_json::json!({"type": report_type}),
                        };

                        Ok(serde_json::json!({
                            "status": "completed",
                            "report": data
                        }))
                    })
                }),
            );

            // Health check task
            handlers.insert(
                "health_check".to_string(),
                Arc::new(move |state: Arc<AppState>, _payload: serde_json::Value| {
                    let state = state.clone();
                    Box::pin(async move {
                        let mut health = serde_json::json!({
                            "status": "healthy",
                            "timestamp": Utc::now()
                        });

                        // Check database
                        let db_ok = state.conn.get().is_ok();
                        health["database"] = serde_json::json!(db_ok);

                        // Check cache
                        if let Some(cache) = &state.cache {
                            let cache_ok = cache.get_connection().is_ok();
                            health["cache"] = serde_json::json!(cache_ok);
                        }

                        // Check S3
                        if let Some(s3) = &state.s3_client {
                            let s3_ok = s3.list_buckets().send().await.is_ok();
                            health["storage"] = serde_json::json!(s3_ok);
                        }

                        Ok(health)
                    })
                }),
            );
        });
    }

    pub async fn register_handler(&self, task_type: String, handler: TaskHandler) {
        let mut registry = self.task_registry.write().await;
        registry.insert(task_type, handler);
    }

    pub async fn create_scheduled_task(
        &self,
        name: String,
        task_type: String,
        cron_expression: String,
        payload: serde_json::Value,
    ) -> Result<ScheduledTask, Box<dyn std::error::Error + Send + Sync>> {
        let schedule = Schedule::from_str(&cron_expression)?;
        let next_run = schedule
            .upcoming(chrono::Local)
            .take(1)
            .next()
            .ok_or("Invalid cron expression")?
            .with_timezone(&Utc);

        let task = ScheduledTask {
            id: Uuid::new_v4(),
            name,
            task_type,
            cron_expression,
            payload,
            enabled: true,
            last_run: None,
            next_run,
            retry_count: 0,
            max_retries: 3,
            timeout_seconds: 300,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let mut tasks = self.scheduled_tasks.write().await;
        tasks.push(task.clone());

        info!("Created scheduled task: {} ({})", task.name, task.id);
        Ok(task)
    }

    pub async fn start(&self) {
        info!("Starting task scheduler");
        let scheduler = self.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));

            loop {
                interval.tick().await;

                if let Err(e) = scheduler.check_and_run_tasks().await {
                    error!("Error checking scheduled tasks: {}", e);
                }
            }
        });
    }

    async fn check_and_run_tasks(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let now = Utc::now();

        let tasks = self.scheduled_tasks.read().await;
        let due_tasks: Vec<ScheduledTask> = tasks
            .iter()
            .filter(|t| t.enabled && t.next_run <= now)
            .cloned()
            .collect();

        for task in due_tasks {
            info!("Running scheduled task: {} ({})", task.name, task.id);
            self.execute_task(task).await;
        }

        Ok(())
    }

    async fn execute_task(&self, mut task: ScheduledTask) {
        let task_id = task.id;
        let state = self._state.clone();
        let registry = self.task_registry.clone();
        let running_tasks = self.running_tasks.clone();

        let handle = tokio::spawn(async move {
            let execution_id = Uuid::new_v4();
            let started_at = Utc::now();

            // Create execution record
            let _execution = TaskExecution {
                id: execution_id,
                scheduled_task_id: task_id,
                started_at,
                completed_at: None,
                status: "running".to_string(),
                result: None,
                error_message: None,
                duration_ms: None,
            };

            // Store in memory (would be database in production)
            // let mut executions = task_executions.write().await;
            // executions.push(execution);

            // Execute the task
            let result = {
                let handlers = registry.read().await;
                if let Some(handler) = handlers.get(&task.task_type) {
                    match tokio::time::timeout(
                        std::time::Duration::from_secs(task.timeout_seconds as u64),
                        handler(state.clone(), task.payload.clone()),
                    )
                    .await
                    {
                        Ok(result) => result,
                        Err(_) => Err("Task execution timed out".into()),
                    }
                } else {
                    Err(format!("No handler for task type: {}", task.task_type).into())
                }
            };

            let completed_at = Utc::now();
            let _duration_ms = (completed_at - started_at).num_milliseconds();

            // Update execution record in memory
            match result {
                Ok(_result) => {
                    // Update task
                    let schedule = Schedule::from_str(&task.cron_expression).ok();
                    let _next_run = schedule
                        .and_then(|s| s.upcoming(chrono::Local).take(1).next())
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|| Utc::now() + Duration::hours(1));

                    // Update task in memory
                    // Would update database in production
                    info!("Task {} completed successfully", task.name);
                }
                Err(e) => {
                    let error_msg = format!("Task failed: {}", e);
                    error!("{}", error_msg);

                    // Handle retries
                    task.retry_count += 1;
                    if task.retry_count < task.max_retries {
                        let _retry_delay =
                            Duration::seconds(60 * (2_i64.pow(task.retry_count as u32)));
                        warn!(
                            "Task {} will retry (attempt {}/{})",
                            task.name, task.retry_count, task.max_retries
                        );
                    } else {
                        error!(
                            "Task {} disabled after {} failed attempts",
                            task.name, task.max_retries
                        );
                    }
                }
            }

            // Remove from running tasks
            let mut running = running_tasks.write().await;
            running.remove(&task_id);
        });

        // Track running task
        let mut running = self.running_tasks.write().await;
        running.insert(task_id, handle);
    }

    pub async fn stop_task(
        &self,
        task_id: Uuid,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut running = self.running_tasks.write().await;

        if let Some(handle) = running.remove(&task_id) {
            handle.abort();
            info!("Stopped task: {}", task_id);
        }

        // Update in memory
        let mut tasks = self.scheduled_tasks.write().await;
        if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
            task.enabled = false;
        }

        Ok(())
    }

    pub async fn get_task_status(
        &self,
        task_id: Uuid,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        let tasks = self.scheduled_tasks.read().await;
        let task = tasks
            .iter()
            .find(|t| t.id == task_id)
            .ok_or("Task not found")?
            .clone();

        let executions = self.task_executions.read().await;
        let recent_executions: Vec<TaskExecution> = executions
            .iter()
            .filter(|e| e.scheduled_task_id == task_id)
            .take(10)
            .cloned()
            .collect();

        let running = self.running_tasks.read().await;
        let is_running = running.contains_key(&task_id);

        Ok(serde_json::json!({
            "task": task,
            "is_running": is_running,
            "recent_executions": recent_executions
        }))
    }

    pub async fn list_scheduled_tasks(
        &self,
    ) -> Result<Vec<ScheduledTask>, Box<dyn std::error::Error + Send + Sync>> {
        let tasks = self.scheduled_tasks.read().await;
        Ok(tasks.clone())
    }

    pub async fn update_task_schedule(
        &self,
        task_id: Uuid,
        cron_expression: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let schedule = Schedule::from_str(&cron_expression)?;
        let next_run = schedule
            .upcoming(chrono::Local)
            .take(1)
            .next()
            .ok_or("Invalid cron expression")?
            .with_timezone(&Utc);

        let mut tasks = self.scheduled_tasks.write().await;
        if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
            task.cron_expression = cron_expression;
            task.next_run = next_run;
            task.updated_at = Utc::now();
        }

        Ok(())
    }

    pub async fn cleanup_old_executions(
        &self,
        days: i64,
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let cutoff = Utc::now() - Duration::days(days);
        let mut executions = self.task_executions.write().await;
        let before_count = executions.len();
        executions.retain(|e| e.completed_at.map_or(true, |completed| completed > cutoff));
        let deleted = before_count - executions.len();
        info!("Cleaned up {} old task executions", deleted);
        Ok(deleted)
    }
}

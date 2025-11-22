use actix_web::{web, HttpResponse, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub assignee: Option<String>,
    pub reporter: String,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub due_date: Option<DateTime<Utc>>,
    pub estimated_hours: Option<f32>,
    pub actual_hours: Option<f32>,
    pub tags: Vec<String>,
    pub parent_task_id: Option<Uuid>,
    pub subtasks: Vec<Uuid>,
    pub dependencies: Vec<Uuid>,
    pub attachments: Vec<String>,
    pub comments: Vec<TaskComment>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Todo,
    InProgress,
    Review,
    Done,
    Blocked,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Urgent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskComment {
    pub id: Uuid,
    pub task_id: Uuid,
    pub author: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskTemplate {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub default_assignee: Option<String>,
    pub default_priority: TaskPriority,
    pub default_tags: Vec<String>,
    pub checklist: Vec<ChecklistItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistItem {
    pub id: Uuid,
    pub task_id: Uuid,
    pub description: String,
    pub completed: bool,
    pub completed_by: Option<String>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskBoard {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub columns: Vec<BoardColumn>,
    pub owner: String,
    pub members: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardColumn {
    pub id: Uuid,
    pub name: String,
    pub position: i32,
    pub status_mapping: TaskStatus,
    pub task_ids: Vec<Uuid>,
    pub wip_limit: Option<i32>,
}

pub struct TaskEngine {
    db: Arc<PgPool>,
    cache: Arc<RwLock<Vec<Task>>>,
}

impl TaskEngine {
    pub fn new(db: Arc<PgPool>) -> Self {
        Self {
            db,
            cache: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Create a new task
    pub async fn create_task(&self, task: Task) -> Result<Task, Box<dyn std::error::Error>> {
        let result = sqlx::query!(
            r#"
            INSERT INTO tasks
            (id, title, description, assignee, reporter, status, priority,
             due_date, estimated_hours, tags, parent_task_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING *
            "#,
            task.id,
            task.title,
            task.description,
            task.assignee,
            task.reporter,
            serde_json::to_value(&task.status)?,
            serde_json::to_value(&task.priority)?,
            task.due_date,
            task.estimated_hours,
            &task.tags[..],
            task.parent_task_id,
            task.created_at,
            task.updated_at
        )
        .fetch_one(self.db.as_ref())
        .await?;

        // Update cache
        let mut cache = self.cache.write().await;
        cache.push(task.clone());

        // Send notification to assignee if specified
        if let Some(assignee) = &task.assignee {
            self.notify_assignee(assignee, &task).await?;
        }

        Ok(task)
    }

    /// Update an existing task
    pub async fn update_task(
        &self,
        id: Uuid,
        updates: serde_json::Value,
    ) -> Result<Task, Box<dyn std::error::Error>> {
        let updated_at = Utc::now();

        // Check if status is changing to Done
        let completing = updates
            .get("status")
            .and_then(|v| v.as_str())
            .map(|s| s == "done")
            .unwrap_or(false);

        let completed_at = if completing {
            Some(Utc::now())
        } else {
            None
        };

        let result = sqlx::query!(
            r#"
            UPDATE tasks
            SET title = COALESCE($2, title),
                description = COALESCE($3, description),
                assignee = COALESCE($4, assignee),
                status = COALESCE($5, status),
                priority = COALESCE($6, priority),
                due_date = COALESCE($7, due_date),
                updated_at = $8,
                completed_at = COALESCE($9, completed_at)
            WHERE id = $1
            RETURNING *
            "#,
            id,
            updates.get("title").and_then(|v| v.as_str()),
            updates.get("description").and_then(|v| v.as_str()),
            updates.get("assignee").and_then(|v| v.as_str()),
            updates.get("status").and_then(|v| serde_json::to_value(v).ok()),
            updates.get("priority").and_then(|v| serde_json::to_value(v).ok()),
            updates
                .get("due_date")
                .and_then(|v| DateTime::parse_from_rfc3339(v.as_str()?).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            updated_at,
            completed_at
        )
        .fetch_one(self.db.as_ref())
        .await?;

        self.refresh_cache().await?;

        Ok(serde_json::from_value(serde_json::to_value(result)?)?)
    }

    /// Delete a task
    pub async fn delete_task(&self, id: Uuid) -> Result<bool, Box<dyn std::error::Error>> {
        // First, check for dependencies
        let dependencies = self.get_task_dependencies(id).await?;
        if !dependencies.is_empty() {
            return Err("Cannot delete task with dependencies".into());
        }

        let result = sqlx::query!("DELETE FROM tasks WHERE id = $1", id)
            .execute(self.db.as_ref())
            .await?;

        self.refresh_cache().await?;

        Ok(result.rows_affected() > 0)
    }

    /// Get tasks for a specific user
    pub async fn get_user_tasks(
        &self,
        user_id: &str,
    ) -> Result<Vec<Task>, Box<dyn std::error::Error>> {
        let results = sqlx::query!(
            r#"
            SELECT * FROM tasks
            WHERE assignee = $1 OR reporter = $1
            ORDER BY priority DESC, due_date ASC
            "#,
            user_id
        )
        .fetch_all(self.db.as_ref())
        .await?;

        Ok(results
            .into_iter()
            .map(|r| serde_json::from_value(serde_json::to_value(r).unwrap()).unwrap())
            .collect())
    }

    /// Get tasks by status
    pub async fn get_tasks_by_status(
        &self,
        status: TaskStatus,
    ) -> Result<Vec<Task>, Box<dyn std::error::Error>> {
        let results = sqlx::query!(
            r#"
            SELECT * FROM tasks
            WHERE status = $1
            ORDER BY priority DESC, created_at ASC
            "#,
            serde_json::to_value(&status)?
        )
        .fetch_all(self.db.as_ref())
        .await?;

        Ok(results
            .into_iter()
            .map(|r| serde_json::from_value(serde_json::to_value(r).unwrap()).unwrap())
            .collect())
    }

    /// Get overdue tasks
    pub async fn get_overdue_tasks(&self) -> Result<Vec<Task>, Box<dyn std::error::Error>> {
        let now = Utc::now();
        let results = sqlx::query!(
            r#"
            SELECT * FROM tasks
            WHERE due_date < $1 AND status != 'done' AND status != 'cancelled'
            ORDER BY due_date ASC
            "#,
            now
        )
        .fetch_all(self.db.as_ref())
        .await?;

        Ok(results
            .into_iter()
            .map(|r| serde_json::from_value(serde_json::to_value(r).unwrap()).unwrap())
            .collect())
    }

    /// Add a comment to a task
    pub async fn add_comment(
        &self,
        task_id: Uuid,
        author: &str,
        content: &str,
    ) -> Result<TaskComment, Box<dyn std::error::Error>> {
        let comment = TaskComment {
            id: Uuid::new_v4(),
            task_id,
            author: author.to_string(),
            content: content.to_string(),
            created_at: Utc::now(),
            updated_at: None,
        };

        sqlx::query!(
            r#"
            INSERT INTO task_comments (id, task_id, author, content, created_at)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            comment.id,
            comment.task_id,
            comment.author,
            comment.content,
            comment.created_at
        )
        .execute(self.db.as_ref())
        .await?;

        Ok(comment)
    }

    /// Create a subtask
    pub async fn create_subtask(
        &self,
        parent_id: Uuid,
        subtask: Task,
    ) -> Result<Task, Box<dyn std::error::Error>> {
        let mut subtask = subtask;
        subtask.parent_task_id = Some(parent_id);

        let created = self.create_task(subtask).await?;

        // Update parent's subtasks list
        sqlx::query!(
            r#"
            UPDATE tasks
            SET subtasks = array_append(subtasks, $1)
            WHERE id = $2
            "#,
            created.id,
            parent_id
        )
        .execute(self.db.as_ref())
        .await?;

        Ok(created)
    }

    /// Get task dependencies
    pub async fn get_task_dependencies(
        &self,
        task_id: Uuid,
    ) -> Result<Vec<Task>, Box<dyn std::error::Error>> {
        let task = self.get_task(task_id).await?;
        let mut dependencies = Vec::new();

        for dep_id in task.dependencies {
            if let Ok(dep_task) = self.get_task(dep_id).await {
                dependencies.push(dep_task);
            }
        }

        Ok(dependencies)
    }

    /// Get a single task by ID
    pub async fn get_task(&self, id: Uuid) -> Result<Task, Box<dyn std::error::Error>> {
        let result = sqlx::query!("SELECT * FROM tasks WHERE id = $1", id)
            .fetch_one(self.db.as_ref())
            .await?;

        Ok(serde_json::from_value(serde_json::to_value(result)?)?)
    }

    /// Calculate task progress (percentage)
    pub async fn calculate_progress(&self, task_id: Uuid) -> Result<f32, Box<dyn std::error::Error>> {
        let task = self.get_task(task_id).await?;

        if task.subtasks.is_empty() {
            // No subtasks, progress based on status
            return Ok(match task.status {
                TaskStatus::Todo => 0.0,
                TaskStatus::InProgress => 50.0,
                TaskStatus::Review => 75.0,
                TaskStatus::Done => 100.0,
                TaskStatus::Blocked => task.actual_hours.unwrap_or(0.0) / task.estimated_hours.unwrap_or(1.0) * 100.0,
                TaskStatus::Cancelled => 0.0,
            });
        }

        // Has subtasks, calculate based on subtask completion
        let total = task.subtasks.len() as f32;
        let mut completed = 0.0;

        for subtask_id in task.subtasks {
            if let Ok(subtask) = self.get_task(subtask_id).await {
                if matches!(subtask.status, TaskStatus::Done) {
                    completed += 1.0;
                }
            }
        }

        Ok((completed / total) * 100.0)
    }

    /// Create a task from template
    pub async fn create_from_template(
        &self,
        template_id: Uuid,
        assignee: Option<String>,
    ) -> Result<Task, Box<dyn std::error::Error>> {
        let template = sqlx::query!(
            "SELECT * FROM task_templates WHERE id = $1",
            template_id
        )
        .fetch_one(self.db.as_ref())
        .await?;

        let template: TaskTemplate = serde_json::from_value(serde_json::to_value(template)?)?;

        let task = Task {
            id: Uuid::new_v4(),
            title: template.name,
            description: Some(template.description),
            assignee: assignee.or(template.default_assignee),
            reporter: "system".to_string(),
            status: TaskStatus::Todo,
            priority: template.default_priority,
            due_date: None,
            estimated_hours: None,
            actual_hours: None,
            tags: template.default_tags,
            parent_task_id: None,
            subtasks: Vec::new(),
            dependencies: Vec::new(),
            attachments: Vec::new(),
            comments: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            completed_at: None,
        };

        let created = self.create_task(task).await?;

        // Create checklist items
        for item in template.checklist {
            let checklist_item = ChecklistItem {
                id: Uuid::new_v4(),
                task_id: created.id,
                description: item.description,
                completed: false,
                completed_by: None,
                completed_at: None,
            };

            sqlx::query!(
                r#"
                INSERT INTO task_checklists (id, task_id, description, completed)
                VALUES ($1, $2, $3, $4)
                "#,
                checklist_item.id,
                checklist_item.task_id,
                checklist_item.description,
                checklist_item.completed
            )
            .execute(self.db.as_ref())
            .await?;
        }

        Ok(created)
    }

    /// Send notification to assignee
    async fn notify_assignee(
        &self,
        assignee: &str,
        task: &Task,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // This would integrate with your notification system
        // For now, just log it
        log::info!(
            "Notifying {} about new task assignment: {}",
            assignee,
            task.title
        );
        Ok(())
    }

    /// Refresh the cache from database
    async fn refresh_cache(&self) -> Result<(), Box<dyn std::error::Error>> {
        let results = sqlx::query!("SELECT * FROM tasks ORDER BY created_at DESC")
            .fetch_all(self.db.as_ref())
            .await?;

        let tasks: Vec<Task> = results
            .into_iter()
            .map(|r| serde_json::from_value(serde_json::to_value(r).unwrap()).unwrap())
            .collect();

        let mut cache = self.cache.write().await;
        *cache = tasks;

        Ok(())
    }

    /// Get task statistics for reporting
    pub async fn get_statistics(
        &self,
        user_id: Option<&str>,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let base_query = if let Some(uid) = user_id {
            format!("WHERE assignee = '{}' OR reporter = '{}'", uid, uid)
        } else {
            String::new()
        };

        let stats = sqlx::query(&format!(
            r#"
            SELECT
                COUNT(*) FILTER (WHERE status = 'todo') as todo_count,
                COUNT(*) FILTER (WHERE status = 'inprogress') as in_progress_count,
                COUNT(*) FILTER (WHERE status = 'done') as done_count,
                COUNT(*) FILTER (WHERE status = 'blocked') as blocked_count,
                COUNT(*) FILTER (WHERE due_date < NOW() AND status != 'done') as overdue_count,
                AVG(EXTRACT(EPOCH FROM (completed_at - created_at))/3600) FILTER (WHERE completed_at IS NOT NULL) as avg_completion_hours
            FROM tasks
            {}
            "#,
            base_query
        ))
        .fetch_one(self.db.as_ref())
        .await?;

        Ok(serde_json::to_value(stats)?)
    }
}

/// HTTP API handlers
pub mod handlers {
    use super::*;

    pub async fn create_task_handler(
        engine: web::Data<TaskEngine>,
        task: web::Json<Task>,
    ) -> Result<HttpResponse> {
        match engine.create_task(task.into_inner()).await {
            Ok(created) => Ok(HttpResponse::Ok().json(created)),
            Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e.to_string()
            }))),
        }
    }

    pub async fn get_tasks_handler(
        engine: web::Data<TaskEngine>,
        query: web::Query<serde_json::Value>,
    ) -> Result<HttpResponse> {
        if let Some(user_id) = query.get("user_id").and_then(|v| v.as_str()) {
            match engine.get_user_tasks(user_id).await {
                Ok(tasks) => Ok(HttpResponse::Ok().json(tasks)),
                Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": e.to_string()
                }))),
            }
        } else if let Some(status) = query.get("status").and_then(|v| v.as_str()) {
            let status = serde_json::from_value(serde_json::json!(status)).unwrap_or(TaskStatus::Todo);
            match engine.get_tasks_by_status(status).await {
                Ok(tasks) => Ok(HttpResponse::Ok().json(tasks)),
                Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": e.to_string()
                }))),
            }
        } else {
            Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Missing user_id or status parameter"
            })))
        }
    }

    pub async fn update_task_handler(
        engine: web::Data<TaskEngine>,
        path: web::Path<Uuid>,
        updates: web::Json<serde_json::Value>,
    ) -> Result<HttpResponse> {
        match engine.update_task(path.into_inner(), updates.into_inner()).await {
            Ok(updated) => Ok(HttpResponse::Ok().json(updated)),
            Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e.to_string()
            }))),
        }
    }

    pub async fn get_statistics_handler(
        engine: web::Data<TaskEngine>,
        query: web::Query<serde_json::Value>,
    ) -> Result<HttpResponse> {
        let user_id = query.get("user_id").and_then(|v| v.as_str());

        match engine.get_statistics(user_id).await {
            Ok(stats) => Ok(HttpResponse::Ok().json(stats)),
            Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e.to_string()
            }))),
        }
    }
}

/// Configure task engine routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/tasks")
            .route("", web::post().to(handlers::create_task_handler))
            .route("", web::get().to(handlers::get_tasks_handler))
            .route("/{id}", web::put().to(handlers::update_task_handler))
            .route("/statistics", web::get().to(handlers::get_statistics_handler)),
    );
}

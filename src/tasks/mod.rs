use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::shared::utils::DbPool;

// TODO: Replace sqlx queries with Diesel queries

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskUpdate {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<TaskStatus>,
    pub priority: Option<TaskPriority>,
    pub assignee: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
    pub tags: Option<Vec<String>>,
}

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
    pub description: Option<String>,
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
    db: Arc<DbPool>,
    cache: Arc<RwLock<Vec<Task>>>,
}

impl TaskEngine {
    pub fn new(db: Arc<DbPool>) -> Self {
        Self {
            db,
            cache: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Create a new task
    pub async fn create_task(&self, task: Task) -> Result<Task, Box<dyn std::error::Error>> {
        // TODO: Implement with Diesel
        /*
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

        let created_task: Task = serde_json::from_value(serde_json::to_value(result)?)?;
        */

        let created_task = task.clone();

        // Update cache
        let mut cache = self.cache.write().await;
        cache.push(created_task.clone());

        Ok(created_task)
    }

    /// Update an existing task
    pub async fn update_task(
        &self,
        id: Uuid,
        updates: TaskUpdate,
    ) -> Result<Task, Box<dyn std::error::Error>> {
        let updated_at = Utc::now();

        // Check if status is changing to Done
        let completing = updates.status
            .as_ref()
            .map(|s| matches!(s, TaskStatus::Done))
            .unwrap_or(false);

        let completed_at = if completing {
            Some(Utc::now())
        } else {
            None
        };

        // TODO: Implement with Diesel
        /*
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

        let updated_task: Task = serde_json::from_value(serde_json::to_value(result)?)?;
        */

        // Create a dummy updated task for now
        let updated_task = Task {
            id,
            title: updates.title.unwrap_or_else(|| "Updated Task".to_string()),
            description: updates.description,
            assignee: updates.assignee,
            reporter: "system".to_string(),
            status: updates.status.unwrap_or(TaskStatus::Todo),
            priority: updates.priority.unwrap_or(TaskPriority::Medium),
            due_date: updates.due_date,
            estimated_hours: None,
            actual_hours: None,
            tags: updates.tags.unwrap_or_default(),
            parent_task_id: None,
            subtasks: Vec::new(),
            dependencies: Vec::new(),
            attachments: Vec::new(),
            comments: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            completed_at,
        };
        self.refresh_cache().await?;

        Ok(updated_task)
    }

    /// Delete a task
    pub async fn delete_task(&self, id: Uuid) -> Result<bool, Box<dyn std::error::Error>> {
        // First, check for dependencies
        let dependencies = self.get_task_dependencies(id).await?;
        if !dependencies.is_empty() {
            return Err("Cannot delete task with dependencies".into());
        }

        // TODO: Implement with Diesel
        /*
        let result = sqlx::query!("DELETE FROM tasks WHERE id = $1", id)
            .execute(self.db.as_ref())
            .await?;
        */

        self.refresh_cache().await?;
        Ok(false)
    }

    /// Get tasks for a specific user
    pub async fn get_user_tasks(
        &self,
        _user_id: &str,
    ) -> Result<Vec<Task>, Box<dyn std::error::Error>> {
        // TODO: Implement with Diesel
        /*
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
        */
        Ok(vec![])
    }

    /// Get tasks by status
    pub async fn get_tasks_by_status(
        &self,
        _status: TaskStatus,
    ) -> Result<Vec<Task>, Box<dyn std::error::Error>> {
        // TODO: Implement with Diesel
        /*
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
        */
        Ok(vec![])
    }

    /// Get overdue tasks
    pub async fn get_overdue_tasks(&self) -> Result<Vec<Task>, Box<dyn std::error::Error>> {
        // TODO: Implement with Diesel
        /*
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
        */
        Ok(vec![])
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

        // TODO: Implement with Diesel
        /*
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
        */

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
        // TODO: Implement with Diesel
        /*
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
        */

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
    pub async fn get_task(&self, _id: Uuid) -> Result<Task, Box<dyn std::error::Error>> {
        // TODO: Implement with Diesel
        /*
        let result = sqlx::query!("SELECT * FROM tasks WHERE id = $1", id)
            .fetch_one(self.db.as_ref())
            .await?;

        Ok(serde_json::from_value(serde_json::to_value(result)?)?)
        */
        Err("Not implemented".into())
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
        _template_id: Uuid,
        assignee: Option<String>,
    ) -> Result<Task, Box<dyn std::error::Error>> {
        // TODO: Implement with Diesel
        /*
        let template = sqlx::query!(
            "SELECT * FROM task_templates WHERE id = $1",
            template_id
        )
        .fetch_one(self.db.as_ref())
        .await?;

        let template: TaskTemplate = serde_json::from_value(serde_json::to_value(template)?)?;
        */

        let template = TaskTemplate {
            id: Uuid::new_v4(),
            name: "Default Template".to_string(),
            description: Some("Default template".to_string()),
            default_assignee: None,
            default_priority: TaskPriority::Medium,
            default_tags: vec![],
            checklist: vec![],
        };

        let task = Task {
            id: Uuid::new_v4(),
            title: template.name,
            description: template.description,
            assignee: assignee,
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
            let _checklist_item = ChecklistItem {
                id: Uuid::new_v4(),
                task_id: created.id,
                description: item.description,
                completed: false,
                completed_by: None,
                completed_at: None,
            };

            // TODO: Implement with Diesel
            /*
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
            */
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
        // TODO: Implement with Diesel
        /*
        let results = sqlx::query!("SELECT * FROM tasks ORDER BY created_at DESC")
            .fetch_all(self.db.as_ref())
            .await?;

        let tasks: Vec<Task> = results
            .into_iter()
            .map(|r| serde_json::from_value(serde_json::to_value(r).unwrap()).unwrap())
            .collect();
        */

        let tasks: Vec<Task> = vec![];

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

        // TODO: Implement with Diesel
        /*
        let stats = sqlx::query(&format!(
            r#"
            SELECT
                COUNT(*) FILTER (WHERE status = 'todo') as todo_count,
                COUNT(*) FILTER (WHERE status = 'in_progress') as in_progress_count,
                COUNT(*) FILTER (WHERE status = 'done') as done_count,
                COUNT(*) FILTER (WHERE due_date < NOW() AND status != 'done') as overdue_count,
                AVG(actual_hours / NULLIF(estimated_hours, 0)) as avg_completion_ratio
            FROM tasks
            {}
            "#,
            base_query
        ))
        .fetch_one(self.db.as_ref())
        .await?;
        */

        // Return empty stats for now
        Ok(serde_json::json!({
            "todo_count": 0,
            "in_progress_count": 0,
            "done_count": 0,
            "overdue_count": 0,
            "avg_completion_ratio": null
        }))
    }
}

/// HTTP API handlers
pub mod handlers {
    use super::*;
    use axum::extract::{State as AxumState, Query as AxumQuery, Path as AxumPath};
    use axum::response::{Json as AxumJson, IntoResponse};
    use axum::http::StatusCode;

    pub async fn create_task_handler<S>(
        AxumState(_engine): AxumState<S>,
        AxumJson(task): AxumJson<Task>,
    ) -> impl IntoResponse {
        // TODO: Implement with actual engine
        let created = task;
        (StatusCode::OK, AxumJson(serde_json::json!(created)))

    }

    pub async fn get_tasks_handler<S>(
        AxumState(_engine): AxumState<S>,
        AxumQuery(_query): AxumQuery<serde_json::Value>,
    ) -> impl IntoResponse {
        // TODO: Implement with actual engine
        let tasks: Vec<Task> = vec![];
        (StatusCode::OK, AxumJson(serde_json::json!(tasks)))
    }

    pub async fn update_task_handler<S>(
        AxumState(_engine): AxumState<S>,
        AxumPath(_id): AxumPath<Uuid>,
        AxumJson(_updates): AxumJson<TaskUpdate>,
    ) -> impl IntoResponse {
        // TODO: Implement with actual engine
        let updated = serde_json::json!({"message": "Task updated"});
        (StatusCode::OK, AxumJson(updated))
    }

    pub async fn get_statistics_handler<S>(
        AxumState(_engine): AxumState<S>,
        AxumQuery(_query): AxumQuery<serde_json::Value>,
    ) -> impl IntoResponse {
        // TODO: Implement with actual engine
        let stats = serde_json::json!({
            "todo_count": 0,
            "in_progress_count": 0,
            "done_count": 0,
            "overdue_count": 0
        });
        (StatusCode::OK, AxumJson(stats))
    }
}

/// Configure task engine routes
pub fn configure<S>(router: Router<S>) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    use axum::routing::{get, post, put};

    router
        .route("/api/tasks", post(handlers::create_task_handler::<S>))
        .route("/api/tasks", get(handlers::get_tasks_handler::<S>))
        .route("/api/tasks/:id", put(handlers::update_task_handler::<S>))
        .route("/api/tasks/statistics", get(handlers::get_statistics_handler::<S>))
}

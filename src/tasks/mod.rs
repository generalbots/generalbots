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

use crate::shared::state::AppState;
use crate::shared::utils::DbPool;

// TODO: Replace sqlx queries with Diesel queries

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskUpdate {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub assignee: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
    pub tags: Option<Vec<String>>,
}

// Database model - matches schema exactly
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = crate::core::shared::models::schema::tasks)]
pub struct Task {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: String,            // Changed to String to match schema
    pub priority: String,          // Changed to String to match schema
    pub assignee_id: Option<Uuid>, // Changed to match schema
    pub reporter_id: Option<Uuid>, // Changed to match schema
    pub project_id: Option<Uuid>,  // Added to match schema
    pub due_date: Option<DateTime<Utc>>,
    pub tags: Vec<String>,
    pub dependencies: Vec<Uuid>,
    pub estimated_hours: Option<f64>, // Changed to f64 to match Float8
    pub actual_hours: Option<f64>,    // Changed to f64 to match Float8
    pub progress: i32,                // Added to match schema
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

// API request/response model - includes additional fields for convenience
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResponse {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub assignee: Option<String>, // Converted from assignee_id
    pub reporter: String,         // Converted from reporter_id
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub due_date: Option<DateTime<Utc>>,
    pub estimated_hours: Option<f64>,
    pub actual_hours: Option<f64>,
    pub tags: Vec<String>,
    pub parent_task_id: Option<Uuid>, // For subtask relationships
    pub subtasks: Vec<Uuid>,          // List of subtask IDs
    pub dependencies: Vec<Uuid>,
    pub attachments: Vec<String>,   // File paths/URLs
    pub comments: Vec<TaskComment>, // Embedded comments
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub progress: i32,
}

// Convert database Task to API TaskResponse
impl From<Task> for TaskResponse {
    fn from(task: Task) -> Self {
        TaskResponse {
            id: task.id,
            title: task.title,
            description: task.description,
            assignee: task.assignee_id.map(|id| id.to_string()),
            reporter: task
                .reporter_id
                .map(|id| id.to_string())
                .unwrap_or_default(),
            status: match task.status.as_str() {
                "todo" => TaskStatus::Todo,
                "in_progress" | "in-progress" => TaskStatus::InProgress,
                "completed" | "done" => TaskStatus::Completed,
                "on_hold" | "on-hold" => TaskStatus::OnHold,
                "review" => TaskStatus::Review,
                "blocked" => TaskStatus::Blocked,
                "cancelled" => TaskStatus::Cancelled,
                _ => TaskStatus::Todo,
            },
            priority: match task.priority.as_str() {
                "low" => TaskPriority::Low,
                "medium" => TaskPriority::Medium,
                "high" => TaskPriority::High,
                "urgent" => TaskPriority::Urgent,
                _ => TaskPriority::Medium,
            },
            due_date: task.due_date,
            estimated_hours: task.estimated_hours,
            actual_hours: task.actual_hours,
            tags: task.tags,
            parent_task_id: None, // Would need separate query
            subtasks: vec![],     // Would need separate query
            dependencies: task.dependencies,
            attachments: vec![], // Would need separate query
            comments: vec![],    // Would need separate query
            created_at: task.created_at,
            updated_at: task.updated_at,
            completed_at: task.completed_at,
            progress: task.progress,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    Todo,
    InProgress,
    Completed,
    OnHold,
    Review,
    Blocked,
    Cancelled,
    Done,
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
    db: DbPool,
    cache: Arc<RwLock<Vec<Task>>>,
}

impl TaskEngine {
    pub fn new(db: DbPool) -> Self {
        Self {
            db,
            cache: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Create a new task
    pub async fn create_task(&self, task: Task) -> Result<Task, Box<dyn std::error::Error>> {
        use crate::core::shared::models::schema::tasks::dsl;
        let conn = &mut self.db.get()?;

        diesel::insert_into(dsl::tasks)
            .values(&task)
            .execute(conn)?;

        Ok(task)
    }

    pub async fn create_task_old(&self, task: Task) -> Result<Task, Box<dyn std::error::Error>> {
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
            task.assignee_id.map(|id| id.to_string()),
            task.reporter_id.map(|id| id.to_string()),
            serde_json::to_value(&task.status)?,
            serde_json::to_value(&task.priority)?,
            task.due_date,
            task.estimated_hours,
            &task.tags[..],
            None, // parent_task_id field doesn't exist in Task struct
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
        // use crate::core::shared::models::schema::tasks::dsl;
        let conn = &mut self.db.get()?;
        let updated_at = Utc::now();

        // Check if status is changing to Done
        let completing = updates
            .status
            .as_ref()
            .map(|s| s == "completed")
            .unwrap_or(false);

        let completed_at = if completing { Some(Utc::now()) } else { None };

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
            status: updates.status.unwrap_or("todo".to_string()),
            priority: updates.priority.unwrap_or("medium".to_string()),
            assignee_id: updates
                .assignee
                .and_then(|s| uuid::Uuid::parse_str(&s).ok()),
            reporter_id: Some(uuid::Uuid::new_v4()),
            project_id: None,
            due_date: updates.due_date,
            tags: updates.tags.unwrap_or_default(),
            dependencies: Vec::new(),
            estimated_hours: None,
            actual_hours: None,
            progress: 0,
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
        status: String,
    ) -> Result<Vec<Task>, Box<dyn std::error::Error>> {
        use crate::core::shared::models::schema::tasks::dsl;
        let conn = &mut self.db.get()?;

        let tasks = dsl::tasks
            .filter(dsl::status.eq(status))
            .order(dsl::created_at.desc())
            .load::<Task>(conn)?;

        Ok(tasks)
    }

    /// Get overdue tasks
    pub async fn get_overdue_tasks(&self) -> Result<Vec<Task>, Box<dyn std::error::Error>> {
        use crate::core::shared::models::schema::tasks::dsl;
        let conn = &mut self.db.get()?;
        let now = Utc::now();

        let tasks = dsl::tasks
            .filter(dsl::due_date.lt(Some(now)))
            .filter(dsl::status.ne("completed"))
            .filter(dsl::status.ne("cancelled"))
            .order(dsl::due_date.asc())
            .load::<Task>(conn)?;

        Ok(tasks)
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
        // For subtasks, we store parent relationship separately
        // or in a separate junction table
        let created = self.create_task(subtask).await?;

        // Update parent's subtasks list
        // TODO: Implement with Diesel
        /*
        sqlx::query!(
            r#"
            -- Update parent's subtasks would be done via a separate junction table
            -- This is a placeholder query
            SELECT 1
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
    pub async fn get_task(&self, id: Uuid) -> Result<Task, Box<dyn std::error::Error>> {
        use crate::core::shared::models::schema::tasks::dsl;
        let conn = &mut self.db.get()?;

        let task = dsl::tasks.filter(dsl::id.eq(id)).first::<Task>(conn)?;

        Ok(task)
    }

    /// Get all tasks
    pub async fn get_all_tasks(&self) -> Result<Vec<Task>, Box<dyn std::error::Error>> {
        use crate::core::shared::models::schema::tasks::dsl;
        let conn = &mut self.db.get()?;

        let tasks = dsl::tasks
            .order(dsl::created_at.desc())
            .load::<Task>(conn)?;

        Ok(tasks)
    }

    /// Assign a task to a user
    pub async fn assign_task(
        &self,
        id: Uuid,
        assignee: String,
    ) -> Result<Task, Box<dyn std::error::Error>> {
        use crate::core::shared::models::schema::tasks::dsl;
        let conn = &mut self.db.get()?;

        let assignee_id = Uuid::parse_str(&assignee).ok();
        let updated_at = Utc::now();

        diesel::update(dsl::tasks.filter(dsl::id.eq(id)))
            .set((
                dsl::assignee_id.eq(assignee_id),
                dsl::updated_at.eq(updated_at),
            ))
            .execute(conn)?;

        self.get_task(id).await
    }

    /// Set task dependencies
    pub async fn set_dependencies(
        &self,
        id: Uuid,
        dependencies: Vec<Uuid>,
    ) -> Result<Task, Box<dyn std::error::Error>> {
        use crate::core::shared::models::schema::tasks::dsl;
        let conn = &mut self.db.get()?;

        let updated_at = Utc::now();

        diesel::update(dsl::tasks.filter(dsl::id.eq(id)))
            .set((
                dsl::dependencies.eq(dependencies),
                dsl::updated_at.eq(updated_at),
            ))
            .execute(conn)?;

        self.get_task(id).await
    }

    /// Calculate task progress (percentage)
    pub async fn calculate_progress(
        &self,
        task_id: Uuid,
    ) -> Result<f32, Box<dyn std::error::Error>> {
        let task = self.get_task(task_id).await?;

        // Calculate progress based on status
        Ok(match task.status.as_str() {
            "todo" => 0.0,
            "in_progress" | "in-progress" => 50.0,
            "review" => 75.0,
            "completed" | "done" => 100.0,
            "blocked" => {
                (task.actual_hours.unwrap_or(0.0) / task.estimated_hours.unwrap_or(1.0) * 100.0)
                    as f32
            }
            "cancelled" => 0.0,
            _ => 0.0,
        })
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
            status: "todo".to_string(),
            priority: "medium".to_string(),
            assignee_id: assignee.and_then(|s| uuid::Uuid::parse_str(&s).ok()),
            reporter_id: Some(uuid::Uuid::new_v4()),
            project_id: None,
            due_date: None,
            estimated_hours: None,
            actual_hours: None,
            tags: template.default_tags,

            dependencies: Vec::new(),
            progress: 0,
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
        let _base_query = if let Some(uid) = user_id {
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
    use axum::extract::{Path as AxumPath, Query as AxumQuery, State as AxumState};
    use axum::http::StatusCode;
    use axum::response::{IntoResponse, Json as AxumJson};

    pub async fn create_task_handler<S>(
        AxumState(_engine): AxumState<S>,
        AxumJson(task): AxumJson<TaskResponse>,
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
        let tasks: Vec<TaskResponse> = vec![];
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

pub async fn handle_task_create(
    State(state): State<Arc<AppState>>,
    Json(mut task): Json<Task>,
) -> Result<Json<TaskResponse>, StatusCode> {
    task.id = Uuid::new_v4();
    task.created_at = Utc::now();
    task.updated_at = Utc::now();

    match state.task_engine.create_task(task).await {
        Ok(created) => Ok(Json(created.into())),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn handle_task_update(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(updates): Json<TaskUpdate>,
) -> Result<Json<TaskResponse>, StatusCode> {
    match state.task_engine.update_task(id, updates).await {
        Ok(updated) => Ok(Json(updated.into())),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn handle_task_delete(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match state.task_engine.delete_task(id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn handle_task_list(
    State(state): State<Arc<AppState>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<TaskResponse>>, StatusCode> {
    let tasks = if let Some(user_id) = params.get("user_id") {
        state.task_engine.get_user_tasks(user_id).await
    } else if let Some(status_str) = params.get("status") {
        let status = match status_str.as_str() {
            "todo" => "todo",
            "in_progress" => "in_progress",
            "review" => "review",
            "done" => "completed",
            "blocked" => "blocked",
            "cancelled" => "cancelled",
            _ => "todo",
        };
        state
            .task_engine
            .get_tasks_by_status(status.to_string())
            .await
    } else {
        state.task_engine.get_all_tasks().await
    };

    match tasks {
        Ok(task_list) => Ok(Json(
            task_list
                .into_iter()
                .map(|t| t.into())
                .collect::<Vec<TaskResponse>>(),
        )),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn handle_task_assign(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<TaskResponse>, StatusCode> {
    let assignee = payload["assignee"]
        .as_str()
        .ok_or(StatusCode::BAD_REQUEST)?;

    match state
        .task_engine
        .assign_task(id, assignee.to_string())
        .await
    {
        Ok(updated) => Ok(Json(updated.into())),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn handle_task_status_update(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<TaskResponse>, StatusCode> {
    let status_str = payload["status"].as_str().ok_or(StatusCode::BAD_REQUEST)?;
    let status = match status_str {
        "todo" => "todo",
        "in_progress" => "in_progress",
        "review" => "review",
        "done" => "completed",
        "blocked" => "blocked",
        "cancelled" => "cancelled",
        _ => return Err(StatusCode::BAD_REQUEST),
    };

    let updates = TaskUpdate {
        title: None,
        description: None,
        status: Some(status.to_string()),
        priority: None,
        assignee: None,
        due_date: None,
        tags: None,
    };

    match state.task_engine.update_task(id, updates).await {
        Ok(updated) => Ok(Json(updated.into())),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn handle_task_priority_set(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<TaskResponse>, StatusCode> {
    let priority_str = payload["priority"]
        .as_str()
        .ok_or(StatusCode::BAD_REQUEST)?;
    let priority = match priority_str {
        "low" => "low",
        "medium" => "medium",
        "high" => "high",
        "urgent" => "urgent",
        _ => return Err(StatusCode::BAD_REQUEST),
    };

    let updates = TaskUpdate {
        title: None,
        description: None,
        status: None,
        priority: Some(priority.to_string()),
        assignee: None,
        due_date: None,
        tags: None,
    };

    match state.task_engine.update_task(id, updates).await {
        Ok(updated) => Ok(Json(updated.into())),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn handle_task_set_dependencies(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<TaskResponse>, StatusCode> {
    let deps = payload["dependencies"]
        .as_array()
        .ok_or(StatusCode::BAD_REQUEST)?
        .iter()
        .filter_map(|v| v.as_str().and_then(|s| Uuid::parse_str(s).ok()))
        .collect::<Vec<_>>();

    match state.task_engine.set_dependencies(id, deps).await {
        Ok(updated) => Ok(Json(updated.into())),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Configure task engine routes
pub fn configure_task_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/tasks", post(handle_task_create))
        .route("/api/tasks", get(handle_task_list))
        .route("/api/tasks/:id", put(handle_task_update))
        .route("/api/tasks/:id", delete(handle_task_delete))
        .route("/api/tasks/:id/assign", post(handle_task_assign))
        .route("/api/tasks/:id/status", put(handle_task_status_update))
        .route("/api/tasks/:id/priority", put(handle_task_priority_set))
        .route(
            "/api/tasks/:id/dependencies",
            put(handle_task_set_dependencies),
        )
}

/// Configure task engine routes (legacy)
pub fn configure<S>(router: Router<S>) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    use axum::routing::{get, post, put};

    router
        .route("/api/tasks", post(handlers::create_task_handler::<S>))
        .route("/api/tasks", get(handlers::get_tasks_handler::<S>))
        .route("/api/tasks/:id", put(handlers::update_task_handler::<S>))
        .route(
            "/api/tasks/statistics",
            get(handlers::get_statistics_handler::<S>),
        )
}

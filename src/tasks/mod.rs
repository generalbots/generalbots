pub mod scheduler;

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

pub use scheduler::TaskScheduler;

// TODO: Replace sqlx queries with Diesel queries

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskRequest {
    pub title: String,
    pub description: Option<String>,
    pub assignee_id: Option<Uuid>,
    pub reporter_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub priority: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
    pub tags: Option<Vec<String>>,
    pub estimated_hours: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskFilters {
    pub status: Option<String>,
    pub priority: Option<String>,
    pub assignee: Option<String>,
    pub project_id: Option<Uuid>,
    pub tag: Option<String>,
    pub limit: Option<usize>,
}

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
    pub description: String,
    pub assignee: Option<String>, // Converted from assignee_id
    pub reporter: Option<String>, // Converted from reporter_id
    pub status: String,
    pub priority: String,
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
            description: task.description.unwrap_or_default(),
            assignee: task.assignee_id.map(|id| id.to_string()),
            reporter: task.reporter_id.map(|id| id.to_string()),
            status: task.status,
            priority: task.priority,
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
            cache: Arc::new(RwLock::new(vec![])),
        }
    }

    pub async fn create_task(
        &self,
        request: CreateTaskRequest,
    ) -> Result<TaskResponse, Box<dyn std::error::Error>> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        let task = Task {
            id,
            title: request.title,
            description: request.description,
            status: "todo".to_string(),
            priority: request.priority.unwrap_or("medium".to_string()),
            assignee_id: request.assignee_id,
            reporter_id: request.reporter_id,
            project_id: request.project_id,
            due_date: request.due_date,
            tags: request.tags.unwrap_or_default(),
            dependencies: vec![],
            estimated_hours: request.estimated_hours,
            actual_hours: None,
            progress: 0,
            created_at: now,
            updated_at: now,
            completed_at: None,
        };

        // Store in cache
        let mut cache = self.cache.write().await;
        cache.push(task.clone());

        Ok(task.into())
    }

    // Removed duplicate update_task - using database version below

    // Removed duplicate delete_task - using database version below

    // Removed duplicate get_task - using database version below

    pub async fn list_tasks(
        &self,
        filters: TaskFilters,
    ) -> Result<Vec<TaskResponse>, Box<dyn std::error::Error + Send + Sync>> {
        let cache = self.cache.read().await;

        let mut tasks: Vec<Task> = cache.clone();

        // Apply filters
        if let Some(status) = filters.status {
            tasks.retain(|t| t.status == status);
        }
        if let Some(priority) = filters.priority {
            tasks.retain(|t| t.priority == priority);
        }
        if let Some(assignee) = filters.assignee {
            if let Ok(assignee_id) = Uuid::parse_str(&assignee) {
                tasks.retain(|t| t.assignee_id == Some(assignee_id));
            }
        }
        if let Some(project_id) = filters.project_id {
            tasks.retain(|t| t.project_id == Some(project_id));
        }
        if let Some(tag) = filters.tag {
            tasks.retain(|t| t.tags.contains(&tag));
        }

        // Sort by creation date (newest first)
        tasks.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        // Apply limit
        if let Some(limit) = filters.limit {
            tasks.truncate(limit);
        }

        Ok(tasks.into_iter().map(|t| t.into()).collect())
    }

    // Removed duplicate - using database version below

    pub async fn update_status(
        &self,
        id: Uuid,
        status: String,
    ) -> Result<TaskResponse, Box<dyn std::error::Error + Send + Sync>> {
        let mut cache = self.cache.write().await;

        if let Some(task) = cache.iter_mut().find(|t| t.id == id) {
            task.status = status.clone();
            if status == "completed" || status == "done" {
                task.completed_at = Some(Utc::now());
                task.progress = 100;
            }
            task.updated_at = Utc::now();
            Ok(task.clone().into())
        } else {
            Err("Task not found".into())
        }
    }
}

// Task API handlers
pub async fn handle_task_create(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateTaskRequest>,
) -> Result<Json<TaskResponse>, StatusCode> {
    let task_engine = &state.task_engine;

    match task_engine.create_task(payload).await {
        Ok(task) => Ok(Json(task)),
        Err(e) => {
            log::error!("Failed to create task: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn handle_task_update(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<TaskUpdate>,
) -> Result<Json<TaskResponse>, StatusCode> {
    let task_engine = &state.task_engine;

    match task_engine.update_task(id, payload).await {
        Ok(task) => Ok(Json(task.into())),
        Err(e) => {
            log::error!("Failed to update task: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn handle_task_delete(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let task_engine = &state.task_engine;

    match task_engine.delete_task(id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            log::error!("Failed to delete task: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn handle_task_get(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<TaskResponse>, StatusCode> {
    let task_engine = &state.task_engine;

    match task_engine.get_task(id).await {
        Ok(task) => Ok(Json(task.into())),
        Err(e) => {
            log::error!("Failed to get task: {}", e);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

// Database operations for TaskEngine
impl TaskEngine {
    pub async fn create_task_with_db(
        &self,
        task: Task,
    ) -> Result<Task, Box<dyn std::error::Error>> {
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
    ) -> Result<Task, Box<dyn std::error::Error + Send + Sync>> {
        let updated_at = Utc::now();

        // Update task in memory cache
        let mut cache = self.cache.write().await;
        if let Some(task) = cache.iter_mut().find(|t| t.id == id) {
            task.updated_at = updated_at;

            // Apply updates
            if let Some(title) = updates.title {
                task.title = title;
            }
            if let Some(description) = updates.description {
                task.description = Some(description);
            }
            if let Some(status) = updates.status {
                task.status = status.clone();
                if status == "completed" || status == "done" {
                    task.completed_at = Some(Utc::now());
                    task.progress = 100;
                }
            }
            if let Some(priority) = updates.priority {
                task.priority = priority;
            }
            if let Some(assignee) = updates.assignee {
                task.assignee_id = Uuid::parse_str(&assignee).ok();
            }
            if let Some(due_date) = updates.due_date {
                task.due_date = Some(due_date);
            }
            if let Some(tags) = updates.tags {
                task.tags = tags;
            }

            return Ok(task.clone());
        }

        Err("Task not found".into())
    }

    /// Delete a task
    pub async fn delete_task(
        &self,
        id: Uuid,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // First, check for dependencies
        let dependencies = self.get_task_dependencies(id).await?;
        if !dependencies.is_empty() {
            return Err("Cannot delete task with dependencies".into());
        }

        // Delete from cache
        let mut cache = self.cache.write().await;
        cache.retain(|t| t.id != id);

        // Refresh cache
        self.refresh_cache()
            .await
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string(),
                ))
            })?;
        Ok(())
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
        status: TaskStatus,
    ) -> Result<Vec<Task>, Box<dyn std::error::Error + Send + Sync>> {
        let cache = self.cache.read().await;
        let status_str = format!("{:?}", status);
        let mut tasks: Vec<Task> = cache
            .iter()
            .filter(|t| t.status == status_str)
            .cloned()
            .collect();
        tasks.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(tasks)
    }

    /// Get overdue tasks
    pub async fn get_overdue_tasks(
        &self,
    ) -> Result<Vec<Task>, Box<dyn std::error::Error + Send + Sync>> {
        let now = Utc::now();
        let cache = self.cache.read().await;
        let mut tasks: Vec<Task> = cache
            .iter()
            .filter(|t| t.due_date.map_or(false, |due| due < now) && t.status != "completed")
            .cloned()
            .collect();
        tasks.sort_by(|a, b| a.due_date.cmp(&b.due_date));
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
        subtask_data: CreateTaskRequest,
    ) -> Result<Task, Box<dyn std::error::Error + Send + Sync>> {
        // Verify parent exists in cache
        {
            let cache = self.cache.read().await;
            if !cache.iter().any(|t| t.id == parent_id) {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Parent task not found",
                ))
                    as Box<dyn std::error::Error + Send + Sync>);
            }
        }

        // Create the subtask
        let subtask = self.create_task(subtask_data).await.map_err(
            |e| -> Box<dyn std::error::Error + Send + Sync> {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string(),
                ))
            },
        )?;

        // Convert TaskResponse back to Task for storage
        let created = Task {
            id: subtask.id,
            title: subtask.title,
            description: Some(subtask.description),
            status: subtask.status,
            priority: subtask.priority,
            assignee_id: subtask
                .assignee
                .as_ref()
                .and_then(|a| Uuid::parse_str(a).ok()),
            reporter_id: subtask
                .reporter
                .as_ref()
                .and_then(|r| Uuid::parse_str(r).ok()),
            project_id: None,
            due_date: subtask.due_date,
            tags: subtask.tags,
            dependencies: subtask.dependencies,
            estimated_hours: subtask.estimated_hours,
            actual_hours: subtask.actual_hours,
            progress: subtask.progress,
            created_at: subtask.created_at,
            updated_at: subtask.updated_at,
            completed_at: subtask.completed_at,
        };

        Ok(created)
    }

    /// Get task dependencies
    pub async fn get_task_dependencies(
        &self,
        task_id: Uuid,
    ) -> Result<Vec<Task>, Box<dyn std::error::Error + Send + Sync>> {
        let task = self.get_task(task_id).await?;
        let mut dependencies = Vec::new();

        for dep_id in task.dependencies {
            if let Ok(dep_task) = self.get_task(dep_id).await {
                // get_task already returns a Task, no conversion needed
                dependencies.push(dep_task);
            }
        }

        Ok(dependencies)
    }

    /// Get a single task by ID
    pub async fn get_task(
        &self,
        id: Uuid,
    ) -> Result<Task, Box<dyn std::error::Error + Send + Sync>> {
        let cache = self.cache.read().await;
        let task =
            cache.iter().find(|t| t.id == id).cloned().ok_or_else(|| {
                Box::<dyn std::error::Error + Send + Sync>::from("Task not found")
            })?;

        Ok(task)
    }

    /// Get all tasks
    pub async fn get_all_tasks(
        &self,
    ) -> Result<Vec<Task>, Box<dyn std::error::Error + Send + Sync>> {
        let cache = self.cache.read().await;
        let mut tasks: Vec<Task> = cache.clone();
        tasks.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(tasks)
    }

    /// Assign a task to a user
    pub async fn assign_task(
        &self,
        id: Uuid,
        assignee: String,
    ) -> Result<Task, Box<dyn std::error::Error + Send + Sync>> {
        let assignee_id = Uuid::parse_str(&assignee).ok();
        let updated_at = Utc::now();

        let mut cache = self.cache.write().await;
        if let Some(task) = cache.iter_mut().find(|t| t.id == id) {
            task.assignee_id = assignee_id;
            task.updated_at = updated_at;
            return Ok(task.clone());
        }

        Err("Task not found".into())
    }

    /// Set task dependencies
    pub async fn set_dependencies(
        &self,
        task_id: Uuid,
        dependency_ids: Vec<Uuid>,
    ) -> Result<TaskResponse, Box<dyn std::error::Error + Send + Sync>> {
        let mut cache = self.cache.write().await;
        if let Some(task) = cache.iter_mut().find(|t| t.id == task_id) {
            task.dependencies = dependency_ids;
            task.updated_at = Utc::now();
        }
        // Get the task and return as TaskResponse
        let task = self.get_task(task_id).await?;
        Ok(task.into())
    }

    /// Calculate task progress (percentage)
    pub async fn calculate_progress(
        &self,
        task_id: Uuid,
    ) -> Result<u8, Box<dyn std::error::Error + Send + Sync>> {
        let task = self.get_task(task_id).await?;

        // Calculate progress based on status
        Ok(match task.status.as_str() {
            "todo" => 0,
            "in_progress" | "in-progress" => 50,
            "review" => 75,
            "completed" | "done" => 100,
            "blocked" => {
                ((task.actual_hours.unwrap_or(0.0) / task.estimated_hours.unwrap_or(1.0)) * 100.0)
                    as u8
            }
            "cancelled" => 0,
            _ => 0,
        })
    }

    /// Create a task from template
    pub async fn create_from_template(
        &self,
        _template_id: Uuid,
        assignee_id: Option<Uuid>,
    ) -> Result<Task, Box<dyn std::error::Error + Send + Sync>> {
        // Create a task from template (simplified)

        let template = TaskTemplate {
            id: Uuid::new_v4(),
            name: "Default Template".to_string(),
            description: Some("Default template".to_string()),
            default_assignee: None,
            default_priority: TaskPriority::Medium,
            default_tags: vec![],
            checklist: vec![],
        };

        let now = Utc::now();
        let task = Task {
            id: Uuid::new_v4(),
            title: format!("Task from template: {}", template.name),
            description: template.description.clone(),
            status: "todo".to_string(),
            priority: "medium".to_string(),
            assignee_id: assignee_id,
            reporter_id: Some(Uuid::new_v4()),
            project_id: None,
            due_date: None,
            estimated_hours: None,
            actual_hours: None,
            tags: template.default_tags,
            dependencies: Vec::new(),
            progress: 0,
            created_at: now,
            updated_at: now,
            completed_at: None,
        };

        // Convert Task to CreateTaskRequest for create_task
        let task_request = CreateTaskRequest {
            title: task.title,
            description: task.description,
            assignee_id: task.assignee_id,
            reporter_id: task.reporter_id,
            project_id: task.project_id,
            priority: Some(task.priority),
            due_date: task.due_date,
            tags: Some(task.tags),
            estimated_hours: task.estimated_hours,
        };
        let created = self.create_task(task_request).await.map_err(
            |e| -> Box<dyn std::error::Error + Send + Sync> {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string(),
                ))
            },
        )?;

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

        // Convert TaskResponse to Task
        let task = Task {
            id: created.id,
            title: created.title,
            description: Some(created.description),
            status: created.status,
            priority: created.priority,
            assignee_id: created
                .assignee
                .as_ref()
                .and_then(|a| Uuid::parse_str(a).ok()),
            reporter_id: created.reporter.as_ref().and_then(|r| {
                if r == "system" {
                    None
                } else {
                    Uuid::parse_str(r).ok()
                }
            }),
            project_id: None,
            tags: created.tags,
            dependencies: created.dependencies,
            due_date: created.due_date,
            estimated_hours: created.estimated_hours,
            actual_hours: created.actual_hours,
            progress: created.progress,
            created_at: created.created_at,
            updated_at: created.updated_at,
            completed_at: created.completed_at,
        };
        Ok(task)
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
    async fn refresh_cache(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
        user_id: Option<Uuid>,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
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

// Duplicate handlers removed - using the ones defined above

pub async fn handle_task_list(
    State(state): State<Arc<AppState>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<TaskResponse>>, StatusCode> {
    let tasks = if let Some(user_id) = params.get("user_id") {
        match state.task_engine.get_user_tasks(user_id).await {
            Ok(tasks) => Ok(tasks),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }?
    } else if let Some(status_str) = params.get("status") {
        let status = match status_str.as_str() {
            "todo" => TaskStatus::Todo,
            "in_progress" => TaskStatus::InProgress,
            "review" => TaskStatus::Review,
            "done" => TaskStatus::Done,
            "blocked" => TaskStatus::Blocked,
            "completed" => TaskStatus::Completed,
            "cancelled" => TaskStatus::Cancelled,
            _ => TaskStatus::Todo,
        };
        match state.task_engine.get_tasks_by_status(status).await {
            Ok(tasks) => Ok(tasks),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }?
    } else {
        match state.task_engine.get_all_tasks().await {
            Ok(tasks) => Ok(tasks),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }?
    };

    Ok(Json(
        tasks
            .into_iter()
            .map(|t| t.into())
            .collect::<Vec<TaskResponse>>(),
    ))
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
        Ok(updated) => Ok(Json(updated)),
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

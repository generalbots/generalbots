//! Task engine - core task management logic
use crate::core::shared::utils::DbPool;
use crate::tasks::types::*;
use chrono::Utc;
use diesel::prelude::*;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug)]
pub struct TaskEngine {
    _db: DbPool,
    cache: Arc<RwLock<Vec<Task>>>,
}

impl TaskEngine {
    pub fn new(db: DbPool) -> Self {
        Self {
            _db: db,
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
            priority: request.priority.unwrap_or_else(|| "medium".to_string()),
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

        let created_task = self.create_task_with_db(task).await?;

        Ok(created_task.into())
    }

    pub async fn list_tasks(
        &self,
        filters: TaskFilters,
    ) -> Result<Vec<TaskResponse>, Box<dyn std::error::Error + Send + Sync>> {
        let cache = self.cache.read().await;
        let mut tasks: Vec<Task> = cache.clone();
        drop(cache);

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

        tasks.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        if let Some(limit) = filters.limit {
            tasks.truncate(limit);
        }

        Ok(tasks.into_iter().map(|t| t.into()).collect())
    }

    pub async fn update_status(
        &self,
        id: Uuid,
        status: String,
    ) -> Result<TaskResponse, Box<dyn std::error::Error + Send + Sync>> {
        let mut cache = self.cache.write().await;

        if let Some(task) = cache.iter_mut().find(|t| t.id == id) {
            task.status.clone_from(&status);
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

    pub async fn create_task_with_db(
        &self,
        task: Task,
    ) -> Result<Task, Box<dyn std::error::Error>> {
        use crate::core::shared::models::schema::tasks::dsl::*;
        use diesel::prelude::*;

        let conn = self._db.clone();
        let task_clone = task.clone();

        let created_task =
            tokio::task::spawn_blocking(move || -> Result<Task, diesel::result::Error> {
                let mut db_conn = conn.get().map_err(|e| {
                    diesel::result::Error::DatabaseError(
                        diesel::result::DatabaseErrorKind::UnableToSendCommand,
                        Box::new(e.to_string()),
                    )
                })?;

                diesel::insert_into(tasks)
                    .values(&task_clone)
                    .get_result(&mut db_conn)
            })
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

        let mut cache = self.cache.write().await;
        cache.push(created_task.clone());
        drop(cache);

        Ok(created_task)
    }

    pub async fn update_task(
        &self,
        id: Uuid,
        updates: TaskUpdate,
    ) -> Result<Task, Box<dyn std::error::Error + Send + Sync>> {
        let updated_at = Utc::now();

        let mut cache = self.cache.write().await;
        if let Some(task) = cache.iter_mut().find(|t| t.id == id) {
            task.updated_at = updated_at;

            if let Some(title) = updates.title {
                task.title = title;
            }
            if let Some(description) = updates.description {
                task.description = Some(description);
            }
            if let Some(status) = updates.status {
                task.status.clone_from(&status);
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

            let result = task.clone();
            drop(cache);
            return Ok(result);
        }
        drop(cache);

        Err("Task not found".into())
    }

    pub async fn delete_task(
        &self,
        id: Uuid,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let dependencies = self.get_task_dependencies(id).await?;
        if !dependencies.is_empty() {
            return Err("Cannot delete task with dependencies".into());
        }

        let mut cache = self.cache.write().await;
        cache.retain(|t| t.id != id);
        drop(cache);

        self.refresh_cache()
            .await
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
                Box::new(std::io::Error::other(e.to_string()))
            })?;
        Ok(())
    }

    pub async fn get_user_tasks(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<Task>, Box<dyn std::error::Error>> {
        let cache = self.cache.read().await;
        let user_tasks: Vec<Task> = cache
            .iter()
            .filter(|t| {
                t.assignee_id.map(|a| a == user_id).unwrap_or(false)
                    || t.reporter_id.map(|r| r == user_id).unwrap_or(false)
            })
            .cloned()
            .collect();
        drop(cache);

        Ok(user_tasks)
    }

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
        drop(cache);
        tasks.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(tasks)
    }

    pub async fn get_overdue_tasks(
        &self,
    ) -> Result<Vec<Task>, Box<dyn std::error::Error + Send + Sync>> {
        let now = Utc::now();
        let cache = self.cache.read().await;
        let mut tasks: Vec<Task> = cache
            .iter()
            .filter(|t| t.due_date.is_some_and(|due| due < now) && t.status != "completed")
            .cloned()
            .collect();
        drop(cache);
        tasks.sort_by(|a, b| a.due_date.cmp(&b.due_date));
        Ok(tasks)
    }

    pub fn add_comment(
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

        log::info!("Added comment to task {}: {}", task_id, content);

        Ok(comment)
    }

    pub async fn create_subtask(
        &self,
        parent_id: Uuid,
        subtask_data: CreateTaskRequest,
    ) -> Result<Task, Box<dyn std::error::Error + Send + Sync>> {
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

        let subtask = self.create_task(subtask_data).await.map_err(
            |e| -> Box<dyn std::error::Error + Send + Sync> {
                Box::new(std::io::Error::other(e.to_string()))
            },
        )?;

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

    pub async fn get_task_dependencies(
        &self,
        task_id: Uuid,
    ) -> Result<Vec<Task>, Box<dyn std::error::Error + Send + Sync>> {
        let task = self.get_task(task_id).await?;
        let mut dependencies = Vec::new();

        for dep_id in task.dependencies {
            if let Ok(dep_task) = self.get_task(dep_id).await {
                dependencies.push(dep_task);
            }
        }

        Ok(dependencies)
    }

    pub async fn get_task(
        &self,
        id: Uuid,
    ) -> Result<Task, Box<dyn std::error::Error + Send + Sync>> {
        let cache = self.cache.read().await;
        if let Some(task) = cache.iter().find(|t| t.id == id).cloned() {
            drop(cache);
            return Ok(task);
        }
        drop(cache);

        let conn = self._db.clone();
        let task_id = id;

        let task = tokio::task::spawn_blocking(move || {
            use crate::core::shared::models::schema::tasks::dsl::*;
            use diesel::prelude::*;

            let mut db_conn = conn.get().map_err(|e| {
                Box::<dyn std::error::Error + Send + Sync>::from(format!("DB error: {e}"))
            })?;

            tasks
                .filter(id.eq(task_id))
                .first::<Task>(&mut db_conn)
                .map_err(|e| {
                    Box::<dyn std::error::Error + Send + Sync>::from(format!("Task not found: {e}"))
                })
        })
        .await
        .map_err(|e| {
            Box::<dyn std::error::Error + Send + Sync>::from(format!("Task error: {e}"))
        })??;

        let mut cache = self.cache.write().await;
        cache.push(task.clone());
        drop(cache);

        Ok(task)
    }

    pub async fn get_all_tasks(
        &self,
    ) -> Result<Vec<Task>, Box<dyn std::error::Error + Send + Sync>> {
        let cache = self.cache.read().await;
        let mut tasks: Vec<Task> = cache.clone();
        drop(cache);
        tasks.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(tasks)
    }

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
            let result = task.clone();
            drop(cache);
            return Ok(result);
        }
        drop(cache);

        Err("Task not found".into())
    }

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

        let task = self.get_task(task_id).await?;
        Ok(task.into())
    }

    pub async fn calculate_progress(
        &self,
        task_id: Uuid,
    ) -> Result<u8, Box<dyn std::error::Error + Send + Sync>> {
        let task = self.get_task(task_id).await?;

        Ok(match task.status.as_str() {
            "in_progress" | "in-progress" => 50,
            "review" => 75,
            "completed" | "done" => 100,
            "blocked" => {
                ((task.actual_hours.unwrap_or(0.0) / task.estimated_hours.unwrap_or(1.0)) * 100.0)
                    as u8
            }
            // "todo", "cancelled", and any other status default to 0
            _ => 0,
        })
    }

    pub async fn create_from_template(
        &self,
        _template_id: Uuid,
        assignee_id: Option<Uuid>,
    ) -> Result<Task, Box<dyn std::error::Error + Send + Sync>> {
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
            assignee_id,
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
                Box::new(std::io::Error::other(e.to_string()))
            },
        )?;

        for item in template.checklist {
            let _checklist_item = ChecklistItem {
                id: Uuid::new_v4(),
                task_id: created.id,
                description: item.description.clone(),
                completed: false,
                completed_by: None,
                completed_at: None,
            };

            log::info!(
                "Added checklist item to task {}: {}",
                created.id,
                item.description
            );
        }

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

    fn _notify_assignee(assignee: &str, task: &Task) -> Result<(), Box<dyn std::error::Error>> {
        log::info!(
            "Notifying {} about new task assignment: {}",
            assignee,
            task.title
        );
        Ok(())
    }

    async fn refresh_cache(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use crate::core::shared::models::schema::tasks::dsl::*;
        use diesel::prelude::*;

        let conn = self._db.clone();

        let task_list = tokio::task::spawn_blocking(
            move || -> Result<Vec<Task>, Box<dyn std::error::Error + Send + Sync>> {
                let mut db_conn = conn.get()?;

                tasks
                    .order(created_at.desc())
                    .load::<Task>(&mut db_conn)
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
            },
        )
        .await??;

        let mut cache = self.cache.write().await;
        *cache = task_list;

        Ok(())
    }

    pub async fn get_statistics(
        &self,
        user_id: Option<Uuid>,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        use chrono::Utc;

        let cache = self.cache.read().await;
        let task_list = if let Some(uid) = user_id {
            cache
                .iter()
                .filter(|t| {
                    t.assignee_id.map(|a| a == uid).unwrap_or(false)
                        || t.reporter_id.map(|r| r == uid).unwrap_or(false)
                })
                .cloned()
                .collect()
        } else {
            cache.clone()
        };
        drop(cache);

        let mut todo_count = 0;
        let mut in_progress_count = 0;
        let mut done_count = 0;
        let mut overdue_count = 0;
        let mut total_completion_ratio = 0.0;
        let mut ratio_count = 0;

        let now = Utc::now();

        for task in &task_list {
            match task.status.as_str() {
                "todo" => todo_count += 1,
                "in_progress" => in_progress_count += 1,
                "done" => done_count += 1,
                _ => {}
            }

            if let Some(due) = task.due_date {
                if due < now && task.status != "done" {
                    overdue_count += 1;
                }
            }

            if let (Some(actual), Some(estimated)) = (task.actual_hours, task.estimated_hours) {
                if estimated > 0.0 {
                    total_completion_ratio += actual / estimated;
                    ratio_count += 1;
                }
            }
        }

        let avg_completion_ratio = if ratio_count > 0 {
            Some(total_completion_ratio / f64::from(ratio_count))
        } else {
            None
        };

        Ok(serde_json::json!({
            "todo_count": todo_count,
            "in_progress_count": in_progress_count,
            "done_count": done_count,
            "overdue_count": overdue_count,
            "avg_completion_ratio": avg_completion_ratio,
            "total_tasks": task_list.len()
        }))
    }
}

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

use crate::types::*;
use crate::service::ProjectService;

pub async fn create_project(
    State(service): State<Arc<ProjectService>>,
    Json(req): Json<CreateProjectRequest>,
) -> Result<Json<Project>, (StatusCode, Json<serde_json::Value>)> {
    let project = Project {
        id: Uuid::new_v4(),
        organization_id: Uuid::new_v4(),
        name: req.name,
        description: req.description,
        start_date: req.start_date,
        end_date: req.end_date,
        status: ProjectStatus::Planning,
        owner_id: Uuid::new_v4(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        settings: ProjectSettings::default(),
    };

    let created = service.create_project(project).await;
    Ok(Json(created))
}

pub async fn list_projects(
    State(service): State<Arc<ProjectService>>,
) -> Json<Vec<Project>> {
    let projects = service.get_all_projects().await;
    Json(projects)
}

pub async fn get_project(
    State(service): State<Arc<ProjectService>>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Project>, (StatusCode, Json<serde_json::Value>)> {
    match service.get_project(project_id).await {
        Some(project) => Ok(Json(project)),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Project not found"})),
        )),
    }
}

pub async fn delete_project(
    State(service): State<Arc<ProjectService>>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if service.delete_project(project_id).await {
        Ok(Json(serde_json::json!({"success": true})))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Project not found"})),
        ))
    }
}

/// Applies a partial update to a project (issue #873). Loads the project,
/// overlays the provided fields, refreshes `updated_at`, and persists.
pub async fn update_project(
    State(service): State<Arc<ProjectService>>,
    Path(project_id): Path<Uuid>,
    Json(req): Json<UpdateProjectRequest>,
) -> Result<Json<Project>, (StatusCode, Json<serde_json::Value>)> {
    let mut project = service
        .get_project(project_id)
        .await
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Project not found"})),
            )
        })?;

    if let Some(name) = req.name {
        if !name.trim().is_empty() {
            project.name = name;
        }
    }
    if let Some(description) = req.description {
        project.description = description;
    }
    if let Some(start_date) = req.start_date {
        project.start_date = start_date;
    }
    if let Some(end_date) = req.end_date {
        project.end_date = end_date;
    }
    if let Some(status) = req.status {
        project.status = status;
    }
    project.updated_at = Utc::now();

    service
        .update_project(project)
        .await
        .map(Json)
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Project not found"})),
            )
        })
}

/// Single-purpose lifecycle transition endpoint (issue #873).
pub async fn update_project_status(
    State(service): State<Arc<ProjectService>>,
    Path(project_id): Path<Uuid>,
    Json(req): Json<UpdateProjectStatusRequest>,
) -> Result<Json<Project>, (StatusCode, Json<serde_json::Value>)> {
    let mut project = service
        .get_project(project_id)
        .await
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Project not found"})),
            )
        })?;

    project.status = req.status;
    project.updated_at = Utc::now();

    service
        .update_project(project)
        .await
        .map(Json)
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Project not found"})),
            )
        })
}

pub async fn create_task(
    State(service): State<Arc<ProjectService>>,
    Path(project_id): Path<Uuid>,
    Json(req): Json<CreateTaskRequest>,
) -> Result<Json<ProjectTask>, (StatusCode, Json<serde_json::Value>)> {
    let end_date = req.start_date + chrono::Duration::days(req.duration_days as i64);

    let task = ProjectTask {
        id: Uuid::new_v4(),
        project_id,
        parent_id: req.parent_id,
        name: req.name,
        description: req.description,
        task_type: req.task_type.unwrap_or(TaskType::Task),
        start_date: req.start_date,
        end_date,
        duration_days: req.duration_days,
        percent_complete: 0,
        status: TaskStatus::NotStarted,
        priority: req.priority.unwrap_or(TaskPriority::Normal),
        assigned_to: Vec::new(),
        dependencies: Vec::new(),
        estimated_hours: None,
        actual_hours: None,
        cost: None,
        notes: None,
        wbs: "1".to_string(),
        outline_level: 1,
        is_milestone: req.is_milestone.unwrap_or(false),
        is_summary: false,
        is_critical: false,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let created = service.create_task(task).await;
    Ok(Json(created))
}

pub async fn get_tasks(
    State(service): State<Arc<ProjectService>>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<ProjectTask>>, (StatusCode, Json<serde_json::Value>)> {
    let tasks = service.get_tasks_for_project(project_id).await;
    Ok(Json(tasks))
}

pub async fn update_task_progress(
    State(service): State<Arc<ProjectService>>,
    Path(task_id): Path<Uuid>,
    Json(req): Json<UpdateProgressRequest>,
) -> Result<Json<ProjectTask>, (StatusCode, Json<serde_json::Value>)> {
    match service.update_task_progress(task_id, req.percent_complete).await {
        Some(task) => Ok(Json(task)),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Task not found"})),
        )),
    }
}

pub async fn add_dependency(
    State(service): State<Arc<ProjectService>>,
    Path(task_id): Path<Uuid>,
    Json(req): Json<AddDependencyRequest>,
) -> Result<Json<ProjectTask>, (StatusCode, Json<serde_json::Value>)> {
    match service
        .add_dependency(task_id, req.predecessor_id, req.dependency_type, req.lag_days.unwrap_or(0))
        .await
    {
        Some(task) => Ok(Json(task)),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Task not found"})),
        )),
    }
}

pub async fn get_gantt_chart(
    State(service): State<Arc<ProjectService>>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<GanttChartData>, (StatusCode, Json<serde_json::Value>)> {
    match service.get_gantt_chart_data(project_id).await {
        Some(data) => Ok(Json(data)),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Project not found"})),
        )),
    }
}

pub async fn get_timeline(
    State(service): State<Arc<ProjectService>>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<TimelineView>, (StatusCode, Json<serde_json::Value>)> {
    match service.get_timeline_view(project_id).await {
        Some(view) => Ok(Json(view)),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Project not found"})),
        )),
    }
}

pub async fn get_critical_path(
    State(service): State<Arc<ProjectService>>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<CriticalPathAnalysis>, (StatusCode, Json<serde_json::Value>)> {
    match service.calculate_critical_path_analysis(project_id).await {
        Some(analysis) => Ok(Json(analysis)),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Project not found or has no tasks"})),
        )),
    }
}

pub async fn delete_task(
    State(service): State<Arc<ProjectService>>,
    Path(task_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if service.delete_task(task_id).await {
        Ok(Json(serde_json::json!({"success": true})))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Task not found"})),
        ))
    }
}

/// Applies a partial update to a task (issue #872). Recomputes `end_date` when
/// `start_date`/`duration_days` change and refreshes `updated_at`.
pub async fn update_task(
    State(service): State<Arc<ProjectService>>,
    Path(task_id): Path<Uuid>,
    Json(req): Json<UpdateTaskRequest>,
) -> Result<Json<ProjectTask>, (StatusCode, Json<serde_json::Value>)> {
    let mut task = service.get_task(task_id).await.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Task not found"})),
        )
    })?;

    if let Some(name) = req.name {
        if !name.trim().is_empty() {
            task.name = name;
        }
    }
    if let Some(description) = req.description {
        task.description = description;
    }
    if let Some(task_type) = req.task_type {
        task.task_type = task_type;
    }
    if let Some(start_date) = req.start_date {
        task.start_date = start_date;
    }
    if let Some(duration_days) = req.duration_days {
        task.duration_days = duration_days;
        task.end_date = task.start_date + chrono::Duration::days(duration_days as i64);
    }
    if let Some(end_date) = req.end_date {
        task.end_date = end_date;
    }
    if let Some(status) = req.status {
        task.status = status;
    }
    if let Some(priority) = req.priority {
        task.priority = priority;
    }
    if let Some(assigned_to) = req.assigned_to {
        task.assigned_to = assigned_to;
    }
    if let Some(estimated_hours) = req.estimated_hours {
        task.estimated_hours = estimated_hours;
    }
    if let Some(actual_hours) = req.actual_hours {
        task.actual_hours = actual_hours;
    }
    if let Some(cost) = req.cost {
        task.cost = cost;
    }
    if let Some(notes) = req.notes {
        task.notes = notes;
    }
    if let Some(is_milestone) = req.is_milestone {
        task.is_milestone = is_milestone;
    }
    task.updated_at = Utc::now();

    service
        .update_task(task)
        .await
        .map(Json)
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Task not found"})),
            )
        })
}

pub async fn remove_dependency(
    State(service): State<Arc<ProjectService>>,
    Path(task_id): Path<Uuid>,
    Json(req): Json<RemoveDependencyRequest>,
) -> Result<Json<ProjectTask>, (StatusCode, Json<serde_json::Value>)> {
    service
        .remove_dependency(task_id, req.predecessor_id)
        .await
        .map(Json)
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Task not found"})),
            )
        })
}

pub async fn create_resource(
    State(service): State<Arc<ProjectService>>,
    Path(project_id): Path<Uuid>,
    Json(req): Json<CreateResourceRequest>,
) -> Json<Resource> {
    let resource = Resource {
        id: Uuid::new_v4(),
        project_id,
        user_id: req.user_id,
        name: req.name,
        resource_type: req.resource_type.unwrap_or(ResourceType::Work),
        email: req.email,
        max_units: req.max_units.unwrap_or(100.0),
        standard_rate: req.standard_rate,
        overtime_rate: req.overtime_rate,
        cost_per_use: req.cost_per_use,
        calendar_id: None,
        created_at: Utc::now(),
    };
    Json(service.create_resource(resource).await)
}

pub async fn list_resources(
    State(service): State<Arc<ProjectService>>,
    Path(project_id): Path<Uuid>,
) -> Json<Vec<Resource>> {
    Json(service.get_resources_for_project(project_id).await)
}

pub async fn delete_resource(
    State(service): State<Arc<ProjectService>>,
    Path(resource_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if service.delete_resource(resource_id).await {
        Ok(Json(serde_json::json!({"success": true})))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Resource not found"})),
        ))
    }
}

pub async fn assign_resource(
    State(service): State<Arc<ProjectService>>,
    Path(task_id): Path<Uuid>,
    Json(req): Json<AssignResourceRequest>,
) -> Result<Json<ResourceAssignment>, (StatusCode, Json<serde_json::Value>)> {
    service
        .assign_resource(task_id, req.resource_id, req.units, req.work_hours)
        .await
        .map(Json)
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Task or resource not found"})),
            )
        })
}

pub async fn list_assignments(
    State(service): State<Arc<ProjectService>>,
    Path(task_id): Path<Uuid>,
) -> Json<Vec<ResourceAssignment>> {
    Json(service.get_assignments_for_task(task_id).await)
}

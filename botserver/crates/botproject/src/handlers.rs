use axum::{
    extract::{Extension, Multipart, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use botsecurity_auth::auth_api::types::AuthenticatedUser;
use chrono::Utc;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::export;
use crate::import::{ImportFormat, ImportOptions, ProjectImportService};
use crate::service::ProjectService;
use crate::types::*;

type ApiError = (StatusCode, Json<serde_json::Value>);

fn forbidden() -> ApiError {
    (
        StatusCode::FORBIDDEN,
        Json(serde_json::json!({ "error": "Forbidden" })),
    )
}

fn not_found(what: &str) -> ApiError {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "error": what })),
    )
}

fn bad_request(what: &str) -> ApiError {
    (
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({ "error": what })),
    )
}

/// Detect the import format from a file extension, defaulting to MS Project
/// XML (the interchange format MS Project writes via File > Save As > XML).
fn format_from_filename(filename: &str) -> ImportFormat {
    let lower = filename.to_lowercase();
    if lower.ends_with(".mpp") {
        ImportFormat::MsProjectMpp
    } else if lower.ends_with(".csv") {
        ImportFormat::Csv
    } else if lower.ends_with(".json") {
        ImportFormat::Json
    } else {
        ImportFormat::MsProjectXml
    }
}

/// Effective tenant scope for a user. When the auth middleware has not
/// attached an organization (service accounts, some legacy sessions) we fall
/// back to the user's own id so a user never sees another user's data.
fn user_org(user: &AuthenticatedUser) -> Uuid {
    user.organization_id.unwrap_or(user.user_id)
}

/// Projects are tenant-scoped by `organization_id`. Admins can read across
/// tenants; everyone else is confined to their own organization.
fn can_access_project(user: &AuthenticatedUser, project: &Project) -> bool {
    user.is_admin() || project.organization_id == user_org(user)
}

/// Load a project and verify the caller may access it, or return 403/404.
async fn require_project(
    service: &ProjectService,
    user: &AuthenticatedUser,
    project_id: Uuid,
) -> Result<Project, ApiError> {
    let project = service
        .get_project(project_id)
        .await
        .ok_or_else(|| not_found("Project not found"))?;
    if !can_access_project(user, &project) {
        return Err(forbidden());
    }
    Ok(project)
}

/// Load a task and verify the caller may access its owning project.
async fn require_task(
    service: &ProjectService,
    user: &AuthenticatedUser,
    task_id: Uuid,
) -> Result<ProjectTask, ApiError> {
    let task = service
        .get_task(task_id)
        .await
        .ok_or_else(|| not_found("Task not found"))?;
    let project = service
        .get_project(task.project_id)
        .await
        .ok_or_else(|| not_found("Project not found"))?;
    if !can_access_project(user, &project) {
        return Err(forbidden());
    }
    Ok(task)
}

/// Load a resource and verify the caller may access its owning project.
async fn require_resource(
    service: &ProjectService,
    user: &AuthenticatedUser,
    resource_id: Uuid,
) -> Result<Resource, ApiError> {
    let resource = service
        .get_resource(resource_id)
        .await
        .ok_or_else(|| not_found("Resource not found"))?;
    let project = service
        .get_project(resource.project_id)
        .await
        .ok_or_else(|| not_found("Project not found"))?;
    if !can_access_project(user, &project) {
        return Err(forbidden());
    }
    Ok(resource)
}

pub async fn create_project(
    State(service): State<Arc<ProjectService>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<CreateProjectRequest>,
) -> Result<Json<Project>, ApiError> {
    if user.user_id.is_nil() {
        return Err(forbidden());
    }

    let project = Project {
        id: Uuid::new_v4(),
        organization_id: user_org(&user),
        name: req.name,
        description: req.description,
        start_date: req.start_date,
        end_date: req.end_date,
        status: ProjectStatus::Planning,
        owner_id: user.user_id,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        settings: ProjectSettings::default(),
    };

    let created = service.create_project(project).await;
    Ok(Json(created))
}

pub async fn list_projects(
    State(service): State<Arc<ProjectService>>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Json<Vec<Project>> {
    if user.is_admin() {
        Json(service.get_all_projects().await)
    } else {
        Json(service.get_projects_for_organization(user_org(&user)).await)
    }
}

pub async fn get_project(
    State(service): State<Arc<ProjectService>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Project>, ApiError> {
    let project = require_project(&service, &user, project_id).await?;
    Ok(Json(project))
}

pub async fn delete_project(
    State(service): State<Arc<ProjectService>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    require_project(&service, &user, project_id).await?;
    if service.delete_project(project_id).await {
        Ok(Json(serde_json::json!({ "success": true })))
    } else {
        Err(not_found("Project not found"))
    }
}

/// Applies a partial update to a project (issue #873). Loads the project,
/// overlays the provided fields, refreshes `updated_at`, and persists.
pub async fn update_project(
    State(service): State<Arc<ProjectService>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(project_id): Path<Uuid>,
    Json(req): Json<UpdateProjectRequest>,
) -> Result<Json<Project>, ApiError> {
    let mut project = require_project(&service, &user, project_id).await?;

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
        .ok_or_else(|| not_found("Project not found"))
}

/// Single-purpose lifecycle transition endpoint (issue #873).
pub async fn update_project_status(
    State(service): State<Arc<ProjectService>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(project_id): Path<Uuid>,
    Json(req): Json<UpdateProjectStatusRequest>,
) -> Result<Json<Project>, ApiError> {
    let mut project = require_project(&service, &user, project_id).await?;

    project.status = req.status;
    project.updated_at = Utc::now();

    service
        .update_project(project)
        .await
        .map(Json)
        .ok_or_else(|| not_found("Project not found"))
}

pub async fn create_task(
    State(service): State<Arc<ProjectService>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(project_id): Path<Uuid>,
    Json(req): Json<CreateTaskRequest>,
) -> Result<Json<ProjectTask>, ApiError> {
    require_project(&service, &user, project_id).await?;

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
    Extension(user): Extension<AuthenticatedUser>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<ProjectTask>>, ApiError> {
    require_project(&service, &user, project_id).await?;
    let tasks = service.get_tasks_for_project(project_id).await;
    Ok(Json(tasks))
}

pub async fn update_task_progress(
    State(service): State<Arc<ProjectService>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(task_id): Path<Uuid>,
    Json(req): Json<UpdateProgressRequest>,
) -> Result<Json<ProjectTask>, ApiError> {
    require_task(&service, &user, task_id).await?;
    match service.update_task_progress(task_id, req.percent_complete).await {
        Some(task) => Ok(Json(task)),
        None => Err(not_found("Task not found")),
    }
}

pub async fn add_dependency(
    State(service): State<Arc<ProjectService>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(task_id): Path<Uuid>,
    Json(req): Json<AddDependencyRequest>,
) -> Result<Json<ProjectTask>, ApiError> {
    require_task(&service, &user, task_id).await?;
    match service
        .add_dependency(task_id, req.predecessor_id, req.dependency_type, req.lag_days.unwrap_or(0))
        .await
    {
        Some(task) => Ok(Json(task)),
        None => Err(not_found("Task not found")),
    }
}

pub async fn get_gantt_chart(
    State(service): State<Arc<ProjectService>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<GanttChartData>, ApiError> {
    require_project(&service, &user, project_id).await?;
    match service.get_gantt_chart_data(project_id).await {
        Some(data) => Ok(Json(data)),
        None => Err(not_found("Project not found")),
    }
}

pub async fn get_timeline(
    State(service): State<Arc<ProjectService>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<TimelineView>, ApiError> {
    require_project(&service, &user, project_id).await?;
    match service.get_timeline_view(project_id).await {
        Some(view) => Ok(Json(view)),
        None => Err(not_found("Project not found")),
    }
}

pub async fn get_critical_path(
    State(service): State<Arc<ProjectService>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<CriticalPathAnalysis>, ApiError> {
    require_project(&service, &user, project_id).await?;
    match service.calculate_critical_path_analysis(project_id).await {
        Some(analysis) => Ok(Json(analysis)),
        None => Err(not_found("Project not found or has no tasks")),
    }
}

pub async fn delete_task(
    State(service): State<Arc<ProjectService>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(task_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    require_task(&service, &user, task_id).await?;
    if service.delete_task(task_id).await {
        Ok(Json(serde_json::json!({ "success": true })))
    } else {
        Err(not_found("Task not found"))
    }
}

/// Applies a partial update to a task (issue #872). Recomputes `end_date` when
/// `start_date`/`duration_days` change and refreshes `updated_at`.
pub async fn update_task(
    State(service): State<Arc<ProjectService>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(task_id): Path<Uuid>,
    Json(req): Json<UpdateTaskRequest>,
) -> Result<Json<ProjectTask>, ApiError> {
    let mut task = require_task(&service, &user, task_id).await?;

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
        .ok_or_else(|| not_found("Task not found"))
}

pub async fn remove_dependency(
    State(service): State<Arc<ProjectService>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(task_id): Path<Uuid>,
    Json(req): Json<RemoveDependencyRequest>,
) -> Result<Json<ProjectTask>, ApiError> {
    require_task(&service, &user, task_id).await?;
    service
        .remove_dependency(task_id, req.predecessor_id)
        .await
        .map(Json)
        .ok_or_else(|| not_found("Task not found"))
}

pub async fn create_resource(
    State(service): State<Arc<ProjectService>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(project_id): Path<Uuid>,
    Json(req): Json<CreateResourceRequest>,
) -> Result<Json<Resource>, ApiError> {
    require_project(&service, &user, project_id).await?;

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
    Ok(Json(service.create_resource(resource).await))
}

pub async fn list_resources(
    State(service): State<Arc<ProjectService>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<Resource>>, ApiError> {
    require_project(&service, &user, project_id).await?;
    Ok(Json(service.get_resources_for_project(project_id).await))
}

pub async fn delete_resource(
    State(service): State<Arc<ProjectService>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(resource_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    require_resource(&service, &user, resource_id).await?;
    if service.delete_resource(resource_id).await {
        Ok(Json(serde_json::json!({ "success": true })))
    } else {
        Err(not_found("Resource not found"))
    }
}

pub async fn assign_resource(
    State(service): State<Arc<ProjectService>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(task_id): Path<Uuid>,
    Json(req): Json<AssignResourceRequest>,
) -> Result<Json<ResourceAssignment>, ApiError> {
    require_task(&service, &user, task_id).await?;
    service
        .assign_resource(task_id, req.resource_id, req.units, req.work_hours)
        .await
        .map(Json)
        .ok_or_else(|| not_found("Task or resource not found"))
}

pub async fn list_assignments(
    State(service): State<Arc<ProjectService>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(task_id): Path<Uuid>,
) -> Result<Json<Vec<ResourceAssignment>>, ApiError> {
    require_task(&service, &user, task_id).await?;
    Ok(Json(service.get_assignments_for_task(task_id).await))
}

/// Import a project from an uploaded file (MS Project XML/MPP, CSV, JSON).
/// Multipart fields: `file` (required) and optional `format` override.
pub async fn import_project(
    State(service): State<Arc<ProjectService>>,
    Extension(user): Extension<AuthenticatedUser>,
    mut multipart: Multipart,
) -> Result<Json<crate::import::ImportResult>, ApiError> {
    if user.user_id.is_nil() {
        return Err(forbidden());
    }

    let mut file_bytes: Option<Vec<u8>> = None;
    let mut filename = "import.xml".to_string();
    let mut explicit_format: Option<String> = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        match field.name() {
            Some("file") => {
                if let Some(name) = field.file_name() {
                    filename = name.to_string();
                }
                match field.bytes().await {
                    Ok(bytes) => file_bytes = Some(bytes.to_vec()),
                    Err(e) => return Err(bad_request(&format!("Failed to read file: {e}"))),
                }
            }
            Some("format") => {
                if let Ok(text) = field.text().await {
                    explicit_format = Some(text);
                }
            }
            _ => {}
        }
    }

    let bytes = file_bytes.ok_or_else(|| bad_request("No file uploaded"))?;

    let format = match explicit_format.as_deref().map(str::to_lowercase).as_deref() {
        Some("xml") | Some("ms_project_xml") => ImportFormat::MsProjectXml,
        Some("mpp") | Some("ms_project_mpp") => ImportFormat::MsProjectMpp,
        Some("csv") => ImportFormat::Csv,
        Some("json") => ImportFormat::Json,
        _ => format_from_filename(&filename),
    };

    let options = ImportOptions {
        format,
        organization_id: user_org(&user),
        owner_id: user.user_id,
        ..ImportOptions::default()
    };

    let importer = ProjectImportService::new();
    let result = importer
        .import(&bytes[..], options)
        .map_err(|e| bad_request(&e))?;

    let result = service.import_data(result).await;
    Ok(Json(result))
}

#[derive(Deserialize)]
pub struct ExportQuery {
    pub format: Option<String>,
}

/// Export a project to MS Project XML (default), CSV, or JSON as a file
/// download.
pub async fn export_project(
    State(service): State<Arc<ProjectService>>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(project_id): Path<Uuid>,
    Query(query): Query<ExportQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let project = require_project(&service, &user, project_id).await?;
    let tasks = service.get_tasks_for_project(project_id).await;
    let resources = service.get_resources_for_project(project_id).await;
    let assignments = service.get_assignments_for_project(project_id).await;

    let format = query.format.unwrap_or_else(|| "xml".to_string()).to_lowercase();

    let (content_type, body) = match format.as_str() {
        "xml" | "mpp" => (
            "application/xml; charset=utf-8",
            export::export_ms_project_xml(&project, &tasks, &resources, &assignments),
        ),
        "csv" => ("text/csv; charset=utf-8", export::export_csv(&project, &tasks)),
        "json" => (
            "application/json; charset=utf-8",
            export::export_json(&project, &tasks),
        ),
        _ => return Err(bad_request("Unsupported export format (use xml, csv, or json)")),
    };

    let filename = format!(
        "{}.{}",
        sanitize_filename(&project.name),
        if format.as_str() == "mpp" { "xml" } else { &format }
    );

    Ok((
        [
            (axum::http::header::CONTENT_TYPE, content_type.to_string()),
            (
                axum::http::header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{filename}\""),
            ),
        ],
        body,
    ))
}

fn sanitize_filename(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect();
    if cleaned.is_empty() {
        "project".to_string()
    } else {
        cleaned
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use botsecurity_auth::auth_api::types::Role;

    fn test_project(org: Uuid) -> Project {
        Project {
            id: Uuid::new_v4(),
            organization_id: org,
            name: "p".to_string(),
            description: None,
            start_date: chrono::NaiveDate::from_ymd_opt(2026, 8, 16).unwrap(),
            end_date: None,
            status: ProjectStatus::Planning,
            owner_id: Uuid::new_v4(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            settings: ProjectSettings::default(),
        }
    }

    #[test]
    fn user_org_uses_organization_when_present() {
        let org = Uuid::new_v4();
        let user = AuthenticatedUser::new(Uuid::new_v4(), "tester".into()).with_organization(org);
        assert_eq!(user_org(&user), org);
    }

    #[test]
    fn user_org_falls_back_to_user_id() {
        let user = AuthenticatedUser::new(Uuid::new_v4(), "tester".into());
        assert_eq!(user_org(&user), user.user_id);
    }

    #[test]
    fn project_access_is_scoped_to_organization() {
        let org = Uuid::new_v4();
        let user = AuthenticatedUser::new(Uuid::new_v4(), "tester".into()).with_organization(org);

        // Same organization -> allowed.
        assert!(can_access_project(&user, &test_project(org)));
        // Different organization -> denied (the IDOR / cross-tenant leak guard).
        assert!(!can_access_project(&user, &test_project(Uuid::new_v4())));
    }

    #[test]
    fn admin_can_access_any_organization() {
        let admin = AuthenticatedUser::new(Uuid::new_v4(), "admin".into()).with_role(Role::Admin);
        assert!(can_access_project(&admin, &test_project(Uuid::new_v4())));
    }
}

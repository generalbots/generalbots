use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post, put},
    Json, Router,
};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::core::shared::state::AppState;

pub mod import;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub status: ProjectStatus,
    pub owner_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub settings: ProjectSettings,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProjectStatus {
    Planning,
    Active,
    OnHold,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSettings {
    pub working_days: Vec<Weekday>,
    pub hours_per_day: f32,
    pub default_task_duration_days: u32,
    pub auto_schedule: bool,
    pub show_critical_path: bool,
    pub currency: String,
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            working_days: vec![
                Weekday::Monday,
                Weekday::Tuesday,
                Weekday::Wednesday,
                Weekday::Thursday,
                Weekday::Friday,
            ],
            hours_per_day: 8.0,
            default_task_duration_days: 1,
            auto_schedule: true,
            show_critical_path: true,
            currency: "USD".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectTask {
    pub id: Uuid,
    pub project_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub task_type: TaskType,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub duration_days: u32,
    pub percent_complete: u8,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub assigned_to: Vec<Uuid>,
    pub dependencies: Vec<TaskDependency>,
    pub estimated_hours: Option<f32>,
    pub actual_hours: Option<f32>,
    pub cost: Option<f64>,
    pub notes: Option<String>,
    pub wbs: String,
    pub outline_level: u32,
    pub is_milestone: bool,
    pub is_summary: bool,
    pub is_critical: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    Task,
    Milestone,
    Summary,
    Form,
    Site,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    NotStarted,
    InProgress,
    Completed,
    OnHold,
    Cancelled,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskPriority {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDependency {
    pub predecessor_id: Uuid,
    pub dependency_type: DependencyType,
    pub lag_days: i32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DependencyType {
    FinishToStart,
    StartToStart,
    FinishToFinish,
    StartToFinish,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub id: Uuid,
    pub project_id: Uuid,
    pub user_id: Option<Uuid>,
    pub name: String,
    pub resource_type: ResourceType,
    pub email: Option<String>,
    pub max_units: f32,
    pub standard_rate: Option<f64>,
    pub overtime_rate: Option<f64>,
    pub cost_per_use: Option<f64>,
    pub calendar_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResourceType {
    Work,
    Material,
    Cost,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAssignment {
    pub id: Uuid,
    pub task_id: Uuid,
    pub resource_id: Uuid,
    pub units: f32,
    pub work_hours: f32,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GanttChartData {
    pub project: Project,
    pub tasks: Vec<GanttTask>,
    pub milestones: Vec<GanttMilestone>,
    pub critical_path: Vec<Uuid>,
    pub date_range: DateRange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GanttTask {
    pub id: Uuid,
    pub name: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub percent_complete: u8,
    pub is_critical: bool,
    pub is_summary: bool,
    pub outline_level: u32,
    pub dependencies: Vec<Uuid>,
    pub assigned_resources: Vec<String>,
    pub bar_color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GanttMilestone {
    pub id: Uuid,
    pub name: String,
    pub date: NaiveDate,
    pub is_completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub start: NaiveDate,
    pub end: NaiveDate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineView {
    pub project_id: Uuid,
    pub project_name: String,
    pub items: Vec<TimelineItem>,
    pub date_range: DateRange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineItem {
    pub id: Uuid,
    pub name: String,
    pub item_type: TimelineItemType,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub percent_complete: u8,
    pub color: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TimelineItemType {
    Task,
    Milestone,
    Phase,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriticalPathAnalysis {
    pub project_id: Uuid,
    pub critical_path_tasks: Vec<Uuid>,
    pub total_duration_days: u32,
    pub float_analysis: Vec<TaskFloat>,
    pub calculated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskFloat {
    pub task_id: Uuid,
    pub task_name: String,
    pub early_start: NaiveDate,
    pub early_finish: NaiveDate,
    pub late_start: NaiveDate,
    pub late_finish: NaiveDate,
    pub total_float_days: i32,
    pub free_float_days: i32,
    pub is_critical: bool,
}

pub struct ProjectService {
    projects: Arc<RwLock<HashMap<Uuid, Project>>>,
    tasks: Arc<RwLock<HashMap<Uuid, ProjectTask>>>,
    resources: Arc<RwLock<HashMap<Uuid, Resource>>>,
    assignments: Arc<RwLock<HashMap<Uuid, ResourceAssignment>>>,
}

impl ProjectService {
    pub fn new() -> Self {
        Self {
            projects: Arc::new(RwLock::new(HashMap::new())),
            tasks: Arc::new(RwLock::new(HashMap::new())),
            resources: Arc::new(RwLock::new(HashMap::new())),
            assignments: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_project(&self, project: Project) -> Project {
        let mut projects = self.projects.write().await;
        projects.insert(project.id, project.clone());
        project
    }

    pub async fn get_project(&self, project_id: Uuid) -> Option<Project> {
        let projects = self.projects.read().await;
        projects.get(&project_id).cloned()
    }

    pub async fn get_projects_for_organization(&self, org_id: Uuid) -> Vec<Project> {
        let projects = self.projects.read().await;
        projects
            .values()
            .filter(|p| p.organization_id == org_id)
            .cloned()
            .collect()
    }

    pub async fn update_project(&self, project: Project) -> Option<Project> {
        let mut projects = self.projects.write().await;
        if projects.contains_key(&project.id) {
            projects.insert(project.id, project.clone());
            Some(project)
        } else {
            None
        }
    }

    pub async fn delete_project(&self, project_id: Uuid) -> bool {
        let mut projects = self.projects.write().await;
        let mut tasks = self.tasks.write().await;
        let mut resources = self.resources.write().await;

        tasks.retain(|_, t| t.project_id != project_id);
        resources.retain(|_, r| r.project_id != project_id);
        projects.remove(&project_id).is_some()
    }

    pub async fn create_task(&self, task: ProjectTask) -> ProjectTask {
        let mut tasks = self.tasks.write().await;
        tasks.insert(task.id, task.clone());
        task
    }

    pub async fn get_task(&self, task_id: Uuid) -> Option<ProjectTask> {
        let tasks = self.tasks.read().await;
        tasks.get(&task_id).cloned()
    }

    pub async fn get_tasks_for_project(&self, project_id: Uuid) -> Vec<ProjectTask> {
        let tasks = self.tasks.read().await;
        let mut project_tasks: Vec<ProjectTask> = tasks
            .values()
            .filter(|t| t.project_id == project_id)
            .cloned()
            .collect();
        project_tasks.sort_by(|a, b| a.wbs.cmp(&b.wbs));
        project_tasks
    }

    pub async fn update_task(&self, task: ProjectTask) -> Option<ProjectTask> {
        let mut tasks = self.tasks.write().await;
        if tasks.contains_key(&task.id) {
            tasks.insert(task.id, task.clone());
            Some(task)
        } else {
            None
        }
    }

    pub async fn delete_task(&self, task_id: Uuid) -> bool {
        let mut tasks = self.tasks.write().await;
        let mut assignments = self.assignments.write().await;

        assignments.retain(|_, a| a.task_id != task_id);
        tasks.remove(&task_id).is_some()
    }

    pub async fn add_dependency(
        &self,
        task_id: Uuid,
        predecessor_id: Uuid,
        dependency_type: DependencyType,
        lag_days: i32,
    ) -> Option<ProjectTask> {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(&task_id) {
            let dependency = TaskDependency {
                predecessor_id,
                dependency_type,
                lag_days,
            };
            task.dependencies.push(dependency);
            task.updated_at = Utc::now();
            return Some(task.clone());
        }
        None
    }

    pub async fn remove_dependency(&self, task_id: Uuid, predecessor_id: Uuid) -> Option<ProjectTask> {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(&task_id) {
            task.dependencies.retain(|d| d.predecessor_id != predecessor_id);
            task.updated_at = Utc::now();
            return Some(task.clone());
        }
        None
    }

    pub async fn create_resource(&self, resource: Resource) -> Resource {
        let mut resources = self.resources.write().await;
        resources.insert(resource.id, resource.clone());
        resource
    }

    pub async fn get_resources_for_project(&self, project_id: Uuid) -> Vec<Resource> {
        let resources = self.resources.read().await;
        resources
            .values()
            .filter(|r| r.project_id == project_id)
            .cloned()
            .collect()
    }

    pub async fn assign_resource(
        &self,
        task_id: Uuid,
        resource_id: Uuid,
        units: f32,
        work_hours: f32,
    ) -> Option<ResourceAssignment> {
        let tasks = self.tasks.read().await;
        let resources = self.resources.read().await;

        let task = tasks.get(&task_id)?;
        let resource = resources.get(&resource_id)?;

        let cost = work_hours * resource.standard_rate.unwrap_or(0.0) as f32;

        let assignment = ResourceAssignment {
            id: Uuid::new_v4(),
            task_id,
            resource_id,
            units,
            work_hours,
            start_date: task.start_date,
            end_date: task.end_date,
            cost: cost as f64,
        };

        drop(tasks);
        drop(resources);

        let mut assignments = self.assignments.write().await;
        assignments.insert(assignment.id, assignment.clone());
        Some(assignment)
    }

    pub async fn get_gantt_chart_data(&self, project_id: Uuid) -> Option<GanttChartData> {
        let projects = self.projects.read().await;
        let project = projects.get(&project_id)?.clone();
        drop(projects);

        let tasks = self.get_tasks_for_project(project_id).await;
        let resources = self.resources.read().await;
        let assignments = self.assignments.read().await;

        let mut gantt_tasks = Vec::new();
        let mut milestones = Vec::new();
        let mut min_date = project.start_date;
        let mut max_date = project.end_date.unwrap_or(project.start_date);

        for task in &tasks {
            if task.start_date < min_date {
                min_date = task.start_date;
            }
            if task.end_date > max_date {
                max_date = task.end_date;
            }

            let assigned_resources: Vec<String> = assignments
                .values()
                .filter(|a| a.task_id == task.id)
                .filter_map(|a| resources.get(&a.resource_id))
                .map(|r| r.name.clone())
                .collect();

            if task.is_milestone {
                milestones.push(GanttMilestone {
                    id: task.id,
                    name: task.name.clone(),
                    date: task.start_date,
                    is_completed: task.percent_complete == 100,
                });
            } else {
                gantt_tasks.push(GanttTask {
                    id: task.id,
                    name: task.name.clone(),
                    start_date: task.start_date,
                    end_date: task.end_date,
                    percent_complete: task.percent_complete,
                    is_critical: task.is_critical,
                    is_summary: task.is_summary,
                    outline_level: task.outline_level,
                    dependencies: task.dependencies.iter().map(|d| d.predecessor_id).collect(),
                    assigned_resources,
                    bar_color: if task.is_critical {
                        Some("#e53935".to_string())
                    } else {
                        None
                    },
                });
            }
        }

        let critical_path = self.calculate_critical_path(&tasks);

        Some(GanttChartData {
            project,
            tasks: gantt_tasks,
            milestones,
            critical_path,
            date_range: DateRange {
                start: min_date,
                end: max_date,
            },
        })
    }

    pub async fn get_timeline_view(&self, project_id: Uuid) -> Option<TimelineView> {
        let projects = self.projects.read().await;
        let project = projects.get(&project_id)?;
        let project_name = project.name.clone();
        drop(projects);

        let tasks = self.get_tasks_for_project(project_id).await;

        let mut items = Vec::new();
        let mut min_date = NaiveDate::MAX;
        let mut max_date = NaiveDate::MIN;

        for task in &tasks {
            if task.start_date < min_date {
                min_date = task.start_date;
            }
            if task.end_date > max_date {
                max_date = task.end_date;
            }

            let (item_type, color) = if task.is_milestone {
                (TimelineItemType::Milestone, "#9c27b0".to_string())
            } else if task.is_summary {
                (TimelineItemType::Phase, "#1976d2".to_string())
            } else {
                (TimelineItemType::Task, "#4caf50".to_string())
            };

            items.push(TimelineItem {
                id: task.id,
                name: task.name.clone(),
                item_type,
                start_date: task.start_date,
                end_date: if task.is_milestone {
                    None
                } else {
                    Some(task.end_date)
                },
                percent_complete: task.percent_complete,
                color,
            });
        }

        Some(TimelineView {
            project_id,
            project_name,
            items,
            date_range: DateRange {
                start: min_date,
                end: max_date,
            },
        })
    }

    pub async fn calculate_critical_path_analysis(&self, project_id: Uuid) -> Option<CriticalPathAnalysis> {
        let tasks = self.get_tasks_for_project(project_id).await;
        if tasks.is_empty() {
            return None;
        }

        let critical_path = self.calculate_critical_path(&tasks);
        let float_analysis = self.calculate_float(&tasks);

        let total_duration = tasks
            .iter()
            .filter(|t| critical_path.contains(&t.id))
            .map(|t| t.duration_days)
            .sum();

        Some(CriticalPathAnalysis {
            project_id,
            critical_path_tasks: critical_path,
            total_duration_days: total_duration,
            float_analysis,
            calculated_at: Utc::now(),
        })
    }

    fn calculate_critical_path(&self, tasks: &[ProjectTask]) -> Vec<Uuid> {
        if tasks.is_empty() {
            return Vec::new();
        }

        let task_map: HashMap<Uuid, &ProjectTask> = tasks.iter().map(|t| (t.id, t)).collect();
        let mut in_degree: HashMap<Uuid, usize> = HashMap::new();
        let mut successors: HashMap<Uuid, Vec<Uuid>> = HashMap::new();

        for task in tasks {
            in_degree.entry(task.id).or_insert(0);
            successors.entry(task.id).or_default();

            for dep in &task.dependencies {
                *in_degree.entry(task.id).or_insert(0) += 1;
                successors.entry(dep.predecessor_id).or_default().push(task.id);
            }
        }

        let mut early_start: HashMap<Uuid, i64> = HashMap::new();
        let mut early_finish: HashMap<Uuid, i64> = HashMap::new();
        let mut queue: Vec<Uuid> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&id, _)| id)
            .collect();

        for &task_id in &queue {
            early_start.insert(task_id, 0);
            if let Some(task) = task_map.get(&task_id) {
                early_finish.insert(task_id, task.duration_days as i64);
            }
        }

        let mut processed = HashSet::new();
        while let Some(task_id) = queue.pop() {
            if processed.contains(&task_id) {
                continue;
            }
            processed.insert(task_id);

            let ef = *early_finish.get(&task_id).unwrap_or(&0);

            if let Some(succs) = successors.get(&task_id) {
                for &succ_id in succs {
                    let current_es = *early_start.get(&succ_id).unwrap_or(&0);
                    if ef > current_es {
                        early_start.insert(succ_id, ef);
                        if let Some(task) = task_map.get(&succ_id) {
                            early_finish.insert(succ_id, ef + task.duration_days as i64);
                        }
                    }

                    if let Some(deg) = in_degree.get_mut(&succ_id) {
                        *deg = deg.saturating_sub(1);
                        if *deg == 0 {
                            queue.push(succ_id);
                        }
                    }
                }
            }
        }

        let project_duration = early_finish.values().max().copied().unwrap_or(0);

        let mut late_finish: HashMap<Uuid, i64> = HashMap::new();
        let mut late_start: HashMap<Uuid, i64> = HashMap::new();

        for task in tasks {
            late_finish.insert(task.id, project_duration);
            late_start.insert(
                task.id,
                project_duration - task.duration_days as i64,
            );
        }

        let mut critical_path = Vec::new();
        for task in tasks {
            let es = *early_start.get(&task.id).unwrap_or(&0);
            let ls = *late_start.get(&task.id).unwrap_or(&0);
            if es == ls {
                critical_path.push(task.id);
            }
        }

        critical_path
    }

    fn calculate_float(&self, tasks: &[ProjectTask]) -> Vec<TaskFloat> {
        let critical_path = self.calculate_critical_path(tasks);

        tasks
            .iter()
            .map(|task| {
                let is_critical = critical_path.contains(&task.id);
                TaskFloat {
                    task_id: task.id,
                    task_name: task.name.clone(),
                    early_start: task.start_date,
                    early_finish: task.end_date,
                    late_start: task.start_date,
                    late_finish: task.end_date,
                    total_float_days: if is_critical { 0 } else { 5 },
                    free_float_days: if is_critical { 0 } else { 3 },
                    is_critical,
                }
            })
            .collect()
    }

    pub async fn update_task_progress(&self, task_id: Uuid, percent_complete: u8) -> Option<ProjectTask> {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(&task_id) {
            task.percent_complete = percent_complete.min(100);
            task.status = if percent_complete == 0 {
                TaskStatus::NotStarted
            } else if percent_complete == 100 {
                TaskStatus::Completed
            } else {
                TaskStatus::InProgress
            };
            task.updated_at = Utc::now();
            return Some(task.clone());
        }
        None
    }
}

impl Default for ProjectService {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    pub name: String,
    pub description: Option<String>,
    pub task_type: Option<TaskType>,
    pub start_date: NaiveDate,
    pub duration_days: u32,
    pub parent_id: Option<Uuid>,
    pub priority: Option<TaskPriority>,
    pub is_milestone: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProgressRequest {
    pub percent_complete: u8,
}

#[derive(Debug, Deserialize)]
pub struct AddDependencyRequest {
    pub predecessor_id: Uuid,
    pub dependency_type: DependencyType,
    pub lag_days: Option<i32>,
}

async fn create_project(
    State(state): State<Arc<AppState>>,
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

    let service = state.project_service.read().await;
    let created = service.create_project(project).await;
    Ok(Json(created))
}

async fn get_project(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Project>, (StatusCode, Json<serde_json::Value>)> {
    let service = state.project_service.read().await;
    match service.get_project(project_id).await {
        Some(project) => Ok(Json(project)),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Project not found"})),
        )),
    }
}

async fn delete_project(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let service = state.project_service.read().await;
    if service.delete_project(project_id).await {
        Ok(Json(serde_json::json!({"success": true})))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Project not found"})),
        ))
    }
}

async fn create_task(
    State(state): State<Arc<AppState>>,
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

    let service = state.project_service.read().await;
    let created = service.create_task(task).await;
    Ok(Json(created))
}

async fn get_tasks(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<ProjectTask>>, (StatusCode, Json<serde_json::Value>)> {
    let service = state.project_service.read().await;
    let tasks = service.get_tasks_for_project(project_id).await;
    Ok(Json(tasks))
}

async fn update_task_progress(
    State(state): State<Arc<AppState>>,
    Path(task_id): Path<Uuid>,
    Json(req): Json<UpdateProgressRequest>,
) -> Result<Json<ProjectTask>, (StatusCode, Json<serde_json::Value>)> {
    let service = state.project_service.read().await;
    match service.update_task_progress(task_id, req.percent_complete).await {
        Some(task) => Ok(Json(task)),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Task not found"})),
        )),
    }
}

async fn add_dependency(
    State(state): State<Arc<AppState>>,
    Path(task_id): Path<Uuid>,
    Json(req): Json<AddDependencyRequest>,
) -> Result<Json<ProjectTask>, (StatusCode, Json<serde_json::Value>)> {
    let service = state.project_service.read().await;
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

async fn get_gantt_chart(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<GanttChartData>, (StatusCode, Json<serde_json::Value>)> {
    let service = state.project_service.read().await;
    match service.get_gantt_chart_data(project_id).await {
        Some(data) => Ok(Json(data)),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Project not found"})),
        )),
    }
}

async fn get_timeline(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<TimelineView>, (StatusCode, Json<serde_json::Value>)> {
    let service = state.project_service.read().await;
    match service.get_timeline_view(project_id).await {
        Some(view) => Ok(Json(view)),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Project not found"})),
        )),
    }
}

async fn get_critical_path(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<CriticalPathAnalysis>, (StatusCode, Json<serde_json::Value>)> {
    let service = state.project_service.read().await;
    match service.calculate_critical_path_analysis(project_id).await {
        Some(analysis) => Ok(Json(analysis)),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Project not found or has no tasks"})),
        )),
    }
}

async fn delete_task(
    State(state): State<Arc<AppState>>,
    Path(task_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let service = state.project_service.read().await;
    if service.delete_task(task_id).await {
        Ok(Json(serde_json::json!({"success": true})))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Task not found"})),
        ))
    }
}

pub fn configure() -> Router<Arc<AppState>> {
    Router::new()
        .route("/projects", post(create_project))
        .route("/projects/:project_id", get(get_project))
        .route("/projects/:project_id", delete(delete_project))
        .route("/projects/:project_id/tasks", post(create_task))
        .route("/projects/:project_id/tasks", get(get_tasks))
        .route("/projects/:project_id/gantt", get(get_gantt_chart))
        .route("/projects/:project_id/timeline", get(get_timeline))
        .route("/projects/:project_id/critical-path", get(get_critical_path))
        .route("/tasks/:task_id/progress", put(update_task_progress))
        .route("/tasks/:task_id/dependencies", post(add_dependency))
        .route("/tasks/:task_id", delete(delete_task))
}

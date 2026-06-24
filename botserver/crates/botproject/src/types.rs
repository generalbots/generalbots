use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

// Request types

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

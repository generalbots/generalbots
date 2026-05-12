use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskContact {
    pub id: Uuid,
    pub task_id: Uuid,
    pub contact_id: Uuid,
    pub role: TaskContactRole,
    pub assigned_at: DateTime<Utc>,
    pub assigned_by: Uuid,
    pub notified: bool,
    pub notified_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum TaskContactRole {
    #[default]
    Assignee,
    Reviewer,
    Stakeholder,
    Collaborator,
    Client,
    Vendor,
    Consultant,
    Approver,
}

impl std::fmt::Display for TaskContactRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskContactRole::Assignee => write!(f, "assignee"),
            TaskContactRole::Reviewer => write!(f, "reviewer"),
            TaskContactRole::Stakeholder => write!(f, "stakeholder"),
            TaskContactRole::Collaborator => write!(f, "collaborator"),
            TaskContactRole::Client => write!(f, "client"),
            TaskContactRole::Vendor => write!(f, "vendor"),
            TaskContactRole::Consultant => write!(f, "consultant"),
            TaskContactRole::Approver => write!(f, "approver"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignContactRequest {
    pub contact_id: Uuid,
    pub role: Option<TaskContactRole>,
    pub send_notification: Option<bool>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkAssignContactsRequest {
    pub assignments: Vec<ContactAssignment>,
    pub send_notification: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactAssignment {
    pub contact_id: Uuid,
    pub role: Option<TaskContactRole>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTaskContactRequest {
    pub role: Option<TaskContactRole>,
    pub notes: Option<String>,
}

pub struct TaskAssignmentParams<'a> {
    pub id: Uuid,
    pub task_id: Uuid,
    pub contact_id: Uuid,
    pub role: &'a TaskContactRole,
    pub assigned_by: Uuid,
    pub notes: Option<&'a str>,
    pub assigned_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskContactsQuery {
    pub role: Option<TaskContactRole>,
    pub include_contact_details: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactTasksQuery {
    pub status: Option<String>,
    pub priority: Option<String>,
    pub role: Option<TaskContactRole>,
    pub due_before: Option<DateTime<Utc>>,
    pub due_after: Option<DateTime<Utc>>,
    pub project_id: Option<Uuid>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub sort_by: Option<TaskSortField>,
    pub sort_order: Option<SortOrder>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ContactTaskPriority {
    Low,
    Normal,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum TaskSortField {
    #[default]
    DueDate,
    Priority,
    CreatedAt,
    UpdatedAt,
    Title,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum SortOrder {
    #[default]
    Asc,
    Desc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskContactWithDetails {
    pub task_contact: TaskContact,
    pub contact: ContactSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactSummary {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub company: Option<String>,
    pub job_title: Option<String>,
    pub avatar_url: Option<String>,
}

impl ContactSummary {
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name).trim().to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSummary {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: String,
    pub due_date: Option<DateTime<Utc>>,
    pub project_id: Option<Uuid>,
    pub project_name: Option<String>,
    pub progress: u8,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactTaskWithDetails {
    pub task_contact: TaskContact,
    pub task: TaskSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactTasksResponse {
    pub tasks: Vec<ContactTaskWithDetails>,
    pub total_count: u32,
    pub by_status: HashMap<String, u32>,
    pub by_priority: HashMap<String, u32>,
    pub overdue_count: u32,
    pub due_today_count: u32,
    pub due_this_week_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactTaskStats {
    pub contact_id: Uuid,
    pub total_tasks: u32,
    pub completed_tasks: u32,
    pub in_progress_tasks: u32,
    pub overdue_tasks: u32,
    pub completion_rate: f32,
    pub average_completion_time_days: Option<f32>,
    pub tasks_by_role: HashMap<String, u32>,
    pub recent_activity: Vec<TaskActivity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskActivity {
    pub id: Uuid,
    pub task_id: Uuid,
    pub task_title: String,
    pub activity_type: TaskActivityType,
    pub description: String,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskActivityType {
    Assigned,
    Unassigned,
    StatusChanged,
    Completed,
    Commented,
    Updated,
    DueDateChanged,
}

impl std::fmt::Display for TaskActivityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskActivityType::Assigned => write!(f, "assigned"),
            TaskActivityType::Unassigned => write!(f, "unassigned"),
            TaskActivityType::StatusChanged => write!(f, "status_changed"),
            TaskActivityType::Completed => write!(f, "completed"),
            TaskActivityType::Commented => write!(f, "commented"),
            TaskActivityType::Updated => write!(f, "updated"),
            TaskActivityType::DueDateChanged => write!(f, "due_date_changed"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedTaskContact {
    pub contact: ContactSummary,
    pub reason: TaskSuggestionReason,
    pub score: f32,
    pub workload: ContactWorkload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskSuggestionReason {
    PreviouslyAssigned,
    SameProject,
    SimilarTasks,
    TeamMember,
    ExpertInArea,
    LowWorkload,
    ClientContact,
}

impl std::fmt::Display for TaskSuggestionReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskSuggestionReason::PreviouslyAssigned => write!(f, "Previously assigned to similar tasks"),
            TaskSuggestionReason::SameProject => write!(f, "Assigned to same project"),
            TaskSuggestionReason::SimilarTasks => write!(f, "Completed similar tasks"),
            TaskSuggestionReason::TeamMember => write!(f, "Team member"),
            TaskSuggestionReason::ExpertInArea => write!(f, "Expert in this area"),
            TaskSuggestionReason::LowWorkload => write!(f, "Has capacity"),
            TaskSuggestionReason::ClientContact => write!(f, "Client contact"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactWorkload {
    pub active_tasks: u32,
    pub high_priority_tasks: u32,
    pub overdue_tasks: u32,
    pub due_this_week: u32,
    pub workload_level: WorkloadLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WorkloadLevel {
    Low,
    Medium,
    High,
    Overloaded,
}

impl std::fmt::Display for WorkloadLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkloadLevel::Low => write!(f, "low"),
            WorkloadLevel::Medium => write!(f, "medium"),
            WorkloadLevel::High => write!(f, "high"),
            WorkloadLevel::Overloaded => write!(f, "overloaded"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskForContactRequest {
    pub title: String,
    pub description: Option<String>,
    pub priority: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
    pub project_id: Option<Uuid>,
    pub tags: Option<Vec<String>>,
    pub role: Option<TaskContactRole>,
    pub send_notification: Option<bool>,
}

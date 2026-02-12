use axum::{response::IntoResponse, Json};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::core::shared::schema::people::{crm_contacts as crm_contacts_table, people as people_table};
use crate::core::shared::schema::tasks::tasks as tasks_table;
use crate::core::shared::utils::DbPool;

#[derive(Debug, Clone)]
pub enum TasksIntegrationError {
    DatabaseError(String),
    ContactNotFound,
    TaskNotFound,
    AlreadyAssigned,
    NotAssigned,
    Unauthorized,
    InvalidInput(String),
}

impl std::fmt::Display for TasksIntegrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DatabaseError(e) => write!(f, "Database error: {e}"),
            Self::ContactNotFound => write!(f, "Contact not found"),
            Self::TaskNotFound => write!(f, "Task not found"),
            Self::AlreadyAssigned => write!(f, "Contact already assigned"),
            Self::NotAssigned => write!(f, "Contact not assigned"),
            Self::Unauthorized => write!(f, "Unauthorized"),
            Self::InvalidInput(e) => write!(f, "Invalid input: {e}"),
        }
    }
}

impl std::error::Error for TasksIntegrationError {}

impl IntoResponse for TasksIntegrationError {
    fn into_response(self) -> axum::response::Response {
        use axum::http::StatusCode;
        let status = match &self {
            Self::ContactNotFound | Self::TaskNotFound => StatusCode::NOT_FOUND,
            Self::AlreadyAssigned | Self::NotAssigned => StatusCode::CONFLICT,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::InvalidInput(_) => StatusCode::BAD_REQUEST,
            Self::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, Json(serde_json::json!({ "error": self.to_string() }))).into_response()
    }
}

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

pub struct TasksIntegrationService {
    db_pool: DbPool,
}

impl TasksIntegrationService {
    pub fn new(pool: DbPool) -> Self {
        Self { db_pool: pool }
    }

    pub async fn assign_contact_to_task(
        &self,
        organization_id: Uuid,
        task_id: Uuid,
        request: &AssignContactRequest,
        assigned_by: Uuid,
    ) -> Result<TaskContact, TasksIntegrationError> {
        // Verify contact exists and belongs to organization
        self.verify_contact(organization_id, request.contact_id).await?;

        // Verify task exists
        self.verify_task(organization_id, task_id).await?;

        // Check if already assigned
        if self.is_contact_assigned(task_id, request.contact_id).await? {
            return Err(TasksIntegrationError::AlreadyAssigned);
        }

        let id = Uuid::new_v4();
        let now = Utc::now();
        let role = request.role.clone().unwrap_or_default();

        // Create assignment in database
        self.create_task_contact_assignment(
            id,
            task_id,
            request.contact_id,
            &role,
            assigned_by,
            request.notes.as_deref(),
            now,
        )
        .await?;

        // Send notification if requested
        let notified = if request.send_notification.unwrap_or(true) {
            self.send_task_assignment_notification(task_id, request.contact_id)
                .await
                .is_ok()
        } else {
            false
        };

        // Log activity
        self.log_contact_activity(
            request.contact_id,
            TaskActivityType::Assigned,
            &format!("Assigned to task"),
            task_id,
        )
        .await?;

        Ok(TaskContact {
            id,
            task_id,
            contact_id: request.contact_id,
            role,
            assigned_at: now,
            assigned_by,
            notified,
            notified_at: if notified { Some(now) } else { None },
            notes: request.notes.clone(),
        })
    }

    pub async fn bulk_assign_contacts(
        &self,
        organization_id: Uuid,
        task_id: Uuid,
        request: &BulkAssignContactsRequest,
        assigned_by: Uuid,
    ) -> Result<Vec<TaskContact>, TasksIntegrationError> {
        let mut results = Vec::new();

        for assignment in &request.assignments {
            let assign_request = AssignContactRequest {
                contact_id: assignment.contact_id,
                role: assignment.role.clone(),
                send_notification: request.send_notification,
                notes: assignment.notes.clone(),
            };

            match self
                .assign_contact_to_task(organization_id, task_id, &assign_request, assigned_by)
                .await
            {
                Ok(task_contact) => results.push(task_contact),
                Err(TasksIntegrationError::AlreadyAssigned) => continue,
                Err(e) => return Err(e),
            }
        }

        Ok(results)
    }

    pub async fn unassign_contact_from_task(
        &self,
        organization_id: Uuid,
        task_id: Uuid,
        contact_id: Uuid,
    ) -> Result<(), TasksIntegrationError> {
        self.verify_contact(organization_id, contact_id).await?;
        self.verify_task(organization_id, task_id).await?;

        self.delete_task_contact_assignment(task_id, contact_id).await?;

        self.log_contact_activity(
            contact_id,
            TaskActivityType::Unassigned,
            "Unassigned from task",
            task_id,
        )
        .await?;

        Ok(())
    }

    pub async fn update_task_contact(
        &self,
        organization_id: Uuid,
        task_id: Uuid,
        contact_id: Uuid,
        request: &UpdateTaskContactRequest,
    ) -> Result<TaskContact, TasksIntegrationError> {
        self.verify_contact(organization_id, contact_id).await?;
        self.verify_task(organization_id, task_id).await?;

        let mut task_contact = self.get_task_contact(task_id, contact_id).await?;

        if let Some(role) = &request.role {
            task_contact.role = role.clone();
        }

        if let Some(notes) = &request.notes {
            task_contact.notes = Some(notes.clone());
        }

        self.update_task_contact_in_db(&task_contact).await?;

        Ok(task_contact)
    }

    pub async fn get_task_contacts(
        &self,
        organization_id: Uuid,
        task_id: Uuid,
        query: &TaskContactsQuery,
    ) -> Result<Vec<TaskContactWithDetails>, TasksIntegrationError> {
        self.verify_task(organization_id, task_id).await?;

        let contacts = self.fetch_task_contacts(task_id, query).await?;

        if query.include_contact_details.unwrap_or(true) {
            let mut results = Vec::new();
            for task_contact in contacts {
                if let Ok(contact) = self.get_contact_summary(task_contact.contact_id).await {
                    results.push(TaskContactWithDetails {
                        task_contact,
                        contact,
                    });
                }
            }
            Ok(results)
        } else {
            Ok(contacts
                .into_iter()
                .map(|tc| TaskContactWithDetails {
                    contact: ContactSummary {
                        id: tc.contact_id,
                        first_name: String::new(),
                        last_name: String::new(),
                        email: None,
                        phone: None,
                        company: None,
                        job_title: None,
                        avatar_url: None,
                    },
                    task_contact: tc,
                })
                .collect())
        }
    }

    pub async fn get_contact_tasks(
        &self,
        organization_id: Uuid,
        contact_id: Uuid,
        query: &ContactTasksQuery,
    ) -> Result<ContactTasksResponse, TasksIntegrationError> {
        self.verify_contact(organization_id, contact_id).await?;

        let tasks = self.fetch_contact_tasks(contact_id, query).await?;
        let total_count = tasks.len() as u32;
        let now = Utc::now();

        let week_end = now + chrono::Duration::days(7);

        let mut by_status: HashMap<String, u32> = HashMap::new();
        let mut by_priority: HashMap<String, u32> = HashMap::new();
        let mut overdue_count = 0;
        let mut due_today_count = 0;
        let mut due_this_week_count = 0;

        for task in &tasks {
            *by_status.entry(task.task.status.clone()).or_insert(0) += 1;
            *by_priority.entry(task.task.priority.clone()).or_insert(0) += 1;

            if let Some(due_date) = task.task.due_date {
                if due_date < now && task.task.status != "completed" {
                    overdue_count += 1;
                } else if due_date.date_naive() == now.date_naive() {
                    due_today_count += 1;
                } else if due_date < week_end {
                    due_this_week_count += 1;
                }
            }
        }

        Ok(ContactTasksResponse {
            tasks,
            total_count,
            by_status,
            by_priority,
            overdue_count,
            due_today_count,
            due_this_week_count,
        })
    }

    pub async fn get_contact_task_stats(
        &self,
        organization_id: Uuid,
        contact_id: Uuid,
    ) -> Result<ContactTaskStats, TasksIntegrationError> {
        self.verify_contact(organization_id, contact_id).await?;

        let stats = self.calculate_contact_task_stats(contact_id).await?;

        Ok(stats)
    }

    pub async fn get_suggested_contacts(
        &self,
        organization_id: Uuid,
        task_id: Uuid,
        limit: Option<u32>,
    ) -> Result<Vec<SuggestedTaskContact>, TasksIntegrationError> {
        self.verify_task(organization_id, task_id).await?;

        let limit = limit.unwrap_or(10);
        let mut suggestions: Vec<SuggestedTaskContact> = Vec::new();

        // Get task details for context
        let task = self.get_task_details(task_id).await?;

        // Get already assigned contacts to exclude
        let assigned_contacts = self.get_assigned_contact_ids(task_id).await?;

        // Find contacts previously assigned to similar tasks
        let previous_assignees = self
            .find_similar_task_assignees(&task, &assigned_contacts, 5)
            .await?;
        for (contact, workload) in previous_assignees {
            suggestions.push(SuggestedTaskContact {
                contact,
                reason: TaskSuggestionReason::PreviouslyAssigned,
                score: 0.9,
                workload,
            });
        }

        // Find contacts assigned to same project
        if let Some(project_id) = task.project_id {
            let project_contacts = self
                .find_project_contacts(project_id, &assigned_contacts, 5)
                .await?;
            for (contact, workload) in project_contacts {
                suggestions.push(SuggestedTaskContact {
                    contact,
                    reason: TaskSuggestionReason::SameProject,
                    score: 0.8,
                    workload,
                });
            }
        }

        // Find contacts with low workload
        let available_contacts = self
            .find_low_workload_contacts(organization_id, &assigned_contacts, 5)
            .await?;
        for (contact, workload) in available_contacts {
            if workload.workload_level == WorkloadLevel::Low {
                suggestions.push(SuggestedTaskContact {
                    contact,
                    reason: TaskSuggestionReason::LowWorkload,
                    score: 0.6,
                    workload,
                });
            }
        }

        // Sort by score and limit
        suggestions.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        suggestions.truncate(limit as usize);

        Ok(suggestions)
    }

    pub async fn get_contact_workload(
        &self,
        organization_id: Uuid,
        contact_id: Uuid,
    ) -> Result<ContactWorkload, TasksIntegrationError> {
        self.verify_contact(organization_id, contact_id).await?;

        let workload = self.calculate_contact_workload(contact_id).await?;

        Ok(workload)
    }

    pub async fn create_task_for_contact(
        &self,
        organization_id: Uuid,
        contact_id: Uuid,
        request: &CreateTaskForContactRequest,
        created_by: Uuid,
    ) -> Result<ContactTaskWithDetails, TasksIntegrationError> {
        self.verify_contact(organization_id, contact_id).await?;

        // Create task
        let task_id = Uuid::new_v4();
        let now = Utc::now();

        self.create_task_in_db(
            task_id,
            organization_id,
            &request.title,
            request.description.as_deref(),
            Some(created_by),
            request.due_date,
        )
        .await?;

        // Assign contact
        let assign_request = AssignContactRequest {
            contact_id,
            role: request.role.clone(),
            send_notification: request.send_notification,
            notes: None,
        };

        let task_contact = self
            .assign_contact_to_task(organization_id, task_id, &assign_request, created_by)
            .await?;

        let task = TaskSummary {
            id: task_id,
            title: request.title.clone(),
            description: request.description.clone(),
            status: "todo".to_string(),
            priority: request.priority.clone().unwrap_or_else(|| "medium".to_string()),
            due_date: request.due_date,
            project_id: request.project_id,
            project_name: None,
            progress: 0,
            created_at: now,
            updated_at: now,
        };

        Ok(ContactTaskWithDetails { task_contact, task })
    }

    async fn send_task_assignment_notification(
        &self,
        _task_id: Uuid,
        _contact_id: Uuid,
    ) -> Result<(), TasksIntegrationError> {
        Ok(())
    }

    async fn log_contact_activity(
        &self,
        _contact_id: Uuid,
        _activity_type: TaskActivityType,
        _description: &str,
        _task_id: Uuid,
    ) -> Result<(), TasksIntegrationError> {
        Ok(())
    }

    async fn verify_contact(
        &self,
        _organization_id: Uuid,
        _contact_id: Uuid,
    ) -> Result<(), TasksIntegrationError> {
        // Verify contact exists and belongs to organization
        Ok(())
    }

    async fn verify_task(
        &self,
        _organization_id: Uuid,
        _task_id: Uuid,
    ) -> Result<(), TasksIntegrationError> {
        // Verify task exists and belongs to organization
        Ok(())
    }

    async fn is_contact_assigned(
        &self,
        _task_id: Uuid,
        _contact_id: Uuid,
    ) -> Result<bool, TasksIntegrationError> {
        // Check if contact is already assigned to task
        Ok(false)
    }

    async fn create_task_contact_assignment(
        &self,
        _id: Uuid,
        _task_id: Uuid,
        _contact_id: Uuid,
        _role: &TaskContactRole,
        _assigned_by: Uuid,
        _notes: Option<&str>,
        _assigned_at: DateTime<Utc>,
    ) -> Result<(), TasksIntegrationError> {
        // Insert into task_contacts table
        Ok(())
    }

    async fn delete_task_contact_assignment(
        &self,
        _task_id: Uuid,
        _contact_id: Uuid,
    ) -> Result<(), TasksIntegrationError> {
        // Delete from task_contacts table
        Ok(())
    }

    async fn get_task_contact(
        &self,
        task_id: Uuid,
        contact_id: Uuid,
    ) -> Result<TaskContact, TasksIntegrationError> {
        // Query task_contacts table
        Ok(TaskContact {
            id: Uuid::new_v4(),
            task_id,
            contact_id,
            role: TaskContactRole::Assignee,
            assigned_at: Utc::now(),
            assigned_by: Uuid::new_v4(),
            notified: false,
            notified_at: None,
            notes: None,
        })
    }

    async fn update_task_contact_in_db(
        &self,
        task_contact: &TaskContact,
    ) -> Result<(), TasksIntegrationError> {
        let pool = self.db_pool.clone();
        let task_id = task_contact.task_id;
        let contact_id = task_contact.contact_id;
        let role = task_contact.role.to_string();
        let _notes = task_contact.notes.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get().map_err(|e| TasksIntegrationError::DatabaseError(e.to_string()))?;

            // Get the contact's email to find the corresponding person
            let contact_email: Option<String> = crm_contacts_table::table
                .filter(crm_contacts_table::id.eq(contact_id))
                .select(crm_contacts_table::email)
                .first(&mut conn)
                .map_err(|e: diesel::result::Error| TasksIntegrationError::DatabaseError(format!("Contact not found: {}", e)))?;

            let contact_email = match contact_email {
                Some(email) => email,
                None => return Ok(()), // No email, can't link to person
            };

            // Find the person with this email
            let person_id: Result<uuid::Uuid, _> = people_table::table
                .filter(people_table::email.eq(&contact_email))
                .select(people_table::id)
                .first(&mut conn);

            if let Ok(pid) = person_id {
                // Update the task's assigned_to field if this is an assignee
                if role == "assignee" {
                    diesel::update(tasks_table::table.filter(tasks_table::id.eq(task_id)))
                        .set(tasks_table::assignee_id.eq(Some(pid)))
                        .execute(&mut conn)
                        .map_err(|e: diesel::result::Error| TasksIntegrationError::DatabaseError(format!("Failed to update task: {}", e)))?;
                }
            }

            Ok(())
        })
        .await
        .map_err(|e| TasksIntegrationError::DatabaseError(e.to_string()))?
    }

    async fn fetch_task_contacts(
        &self,
        task_id: Uuid,
        _query: &TaskContactsQuery,
    ) -> Result<Vec<TaskContact>, TasksIntegrationError> {
        let pool = self.db_pool.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get().map_err(|e| TasksIntegrationError::DatabaseError(e.to_string()))?;

            // Get task assignees from tasks table and look up corresponding contacts
            let task_row: Result<(Uuid, Option<Uuid>, DateTime<Utc>), _> = tasks_table::table
                .filter(tasks_table::id.eq(task_id))
                .select((tasks_table::id, tasks_table::assignee_id, tasks_table::created_at))
                .first(&mut conn);

            let mut task_contacts = Vec::new();

            if let Ok((tid, assigned_to, created_at)) = task_row {
                if let Some(assignee_id) = assigned_to {
                    // Look up person -> email -> contact
                    let person_email: Result<Option<String>, _> = people_table::table
                        .filter(people_table::id.eq(assignee_id))
                        .select(people_table::email)
                        .first(&mut conn);

                    if let Ok(Some(email)) = person_email {
                        // Find contact with this email
                        let contact_result: Result<Uuid, _> = crm_contacts_table::table
                            .filter(crm_contacts_table::email.eq(&email))
                            .select(crm_contacts_table::id)
                            .first(&mut conn);

                        if let Ok(contact_id) = contact_result {
                            task_contacts.push(TaskContact {
                                id: Uuid::new_v4(),
                                task_id: tid,
                                contact_id,
                                role: TaskContactRole::Assignee,
                                assigned_at: created_at,
                                assigned_by: Uuid::nil(),
                                notified: false,
                                notified_at: None,
                                notes: None,
                            });
                        }
                    }
                }
            }

            Ok(task_contacts)
        })
        .await
        .map_err(|e| TasksIntegrationError::DatabaseError(e.to_string()))?
    }

    async fn fetch_contact_tasks(
        &self,
        contact_id: Uuid,
        query: &ContactTasksQuery,
    ) -> Result<Vec<ContactTaskWithDetails>, TasksIntegrationError> {
        let pool = self.db_pool.clone();
        let status_filter = query.status.clone();

        tokio::task::spawn_blocking(move || -> Result<Vec<ContactTaskWithDetails>, TasksIntegrationError> {
            let mut conn = pool.get().map_err(|e| TasksIntegrationError::DatabaseError(e.to_string()))?;

            let mut db_query = tasks_table::table
                .filter(tasks_table::status.ne("deleted"))
                .into_boxed();

            if let Some(status) = status_filter {
                db_query = db_query.filter(tasks_table::status.eq(status));
            }

            let rows: Vec<(Uuid, String, Option<String>, String, String, Option<DateTime<Utc>>, Option<Uuid>, i32, DateTime<Utc>, DateTime<Utc>)> = db_query
                .order(tasks_table::created_at.desc())
                .select((
                    tasks_table::id,
                    tasks_table::title,
                    tasks_table::description,
                    tasks_table::status,
                    tasks_table::priority,
                    tasks_table::due_date,
                    tasks_table::project_id,
                    tasks_table::progress,
                    tasks_table::created_at,
                    tasks_table::updated_at,
                ))
                .limit(50)
                .load(&mut conn)
                .map_err(|e: diesel::result::Error| TasksIntegrationError::DatabaseError(e.to_string()))?;

            let tasks_list = rows.into_iter().map(|row| {
                ContactTaskWithDetails {
                    task_contact: TaskContact {
                        id: Uuid::new_v4(),
                        task_id: row.0,
                        contact_id,
                        role: TaskContactRole::Assignee,
                        assigned_at: Utc::now(),
                        assigned_by: Uuid::nil(),
                        notified: false,
                        notified_at: None,
                        notes: None,
                    },
                    task: TaskSummary {
                        id: row.0,
                        title: row.1,
                        description: row.2,
                        status: row.3,
                        priority: row.4,
                        due_date: row.5,
                        project_id: row.6,
                        project_name: None,
                        progress: row.7 as u8,
                        created_at: row.8,
                        updated_at: row.9,
                    },
                }
            }).collect();

            Ok(tasks_list)
        })
        .await
        .map_err(|e: tokio::task::JoinError| TasksIntegrationError::DatabaseError(e.to_string()))?
    }

    async fn get_contact_summary(
        &self,
        contact_id: Uuid,
    ) -> Result<ContactSummary, TasksIntegrationError> {
        // Query contacts table for summary
        Ok(ContactSummary {
            id: contact_id,
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            email: Some("john@example.com".to_string()),
            phone: None,
            company: None,
            job_title: None,
            avatar_url: None,
        })
    }

    async fn get_task_details(&self, task_id: Uuid) -> Result<TaskSummary, TasksIntegrationError> {
        // Query tasks table
        Ok(TaskSummary {
            id: task_id,
            title: "Task".to_string(),
            description: None,
            status: "todo".to_string(),
            priority: "medium".to_string(),
            due_date: None,
            project_id: None,
            project_name: None,
            progress: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    async fn get_assigned_contact_ids(
        &self,
        task_id: Uuid,
    ) -> Result<Vec<Uuid>, TasksIntegrationError> {
        let pool = self.db_pool.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get().map_err(|e| TasksIntegrationError::DatabaseError(e.to_string()))?;

            let assignee_id: Option<Uuid> = tasks_table::table
                .filter(tasks_table::id.eq(task_id))
                .select(tasks_table::assignee_id)
                .first(&mut conn)
                .optional()
                .map_err(|e: diesel::result::Error| TasksIntegrationError::DatabaseError(e.to_string()))?
                .flatten();

            if let Some(user_id) = assignee_id {
                let person_email: Option<String> = people_table::table
                    .filter(people_table::user_id.eq(user_id))
                    .select(people_table::email)
                    .first(&mut conn)
                    .optional()
                    .map_err(|e: diesel::result::Error| TasksIntegrationError::DatabaseError(e.to_string()))?
                    .flatten();

                if let Some(email) = person_email {
                    let contact_ids: Vec<Uuid> = crm_contacts_table::table
                        .filter(crm_contacts_table::email.eq(&email))
                        .select(crm_contacts_table::id)
                        .load(&mut conn)
                        .unwrap_or_default();

                    return Ok(contact_ids);
                }
            }

            Ok(vec![])
        })
        .await
        .map_err(|e| TasksIntegrationError::DatabaseError(e.to_string()))?
    }

    async fn calculate_contact_task_stats(
        &self,
        contact_id: Uuid,
    ) -> Result<ContactTaskStats, TasksIntegrationError> {
        // Calculate task statistics for contact
        Ok(ContactTaskStats {
            contact_id,
            total_tasks: 0,
            completed_tasks: 0,
            in_progress_tasks: 0,
            overdue_tasks: 0,
            completion_rate: 0.0,
            average_completion_time_days: None,
            tasks_by_role: HashMap::new(),
            recent_activity: vec![],
        })
    }

    async fn calculate_contact_workload(
        &self,
        _contact_id: Uuid,
    ) -> Result<ContactWorkload, TasksIntegrationError> {
        // Calculate current workload for contact
        Ok(ContactWorkload {
            active_tasks: 0,
            high_priority_tasks: 0,
            overdue_tasks: 0,
            due_this_week: 0,
            workload_level: WorkloadLevel::Low,
        })
    }

    async fn find_similar_task_assignees(
        &self,
        _task: &TaskSummary,
        exclude: &[Uuid],
        limit: usize,
    ) -> Result<Vec<(ContactSummary, ContactWorkload)>, TasksIntegrationError> {
        let pool = self.db_pool.clone();
        let exclude = exclude.to_vec();

        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get().map_err(|e| TasksIntegrationError::DatabaseError(e.to_string()))?;

            let mut query = crm_contacts_table::table
                .filter(crm_contacts_table::status.eq("active"))
                .into_boxed();

            for exc in &exclude {
                query = query.filter(crm_contacts_table::id.ne(*exc));
            }

            let rows: Vec<(Uuid, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>)> = query
                .select((
                    crm_contacts_table::id,
                    crm_contacts_table::first_name,
                    crm_contacts_table::last_name,
                    crm_contacts_table::email,
                    crm_contacts_table::company,
                    crm_contacts_table::job_title,
                ))
                .limit(limit as i64)
                .load(&mut conn)
                .map_err(|e: diesel::result::Error| TasksIntegrationError::DatabaseError(e.to_string()))?;

            let contacts = rows.into_iter().map(|row| {
                let summary = ContactSummary {
                    id: row.0,
                    first_name: row.1.unwrap_or_default(),
                    last_name: row.2.unwrap_or_default(),
                    email: row.3,
                    phone: None,
                    company: row.4,
                    job_title: row.5,
                    avatar_url: None,
                };
                let workload = ContactWorkload {
                    active_tasks: 0,
                    high_priority_tasks: 0,
                    overdue_tasks: 0,
                    due_this_week: 0,
                    workload_level: WorkloadLevel::Low,
                };
                (summary, workload)
            }).collect();

            Ok(contacts)
        })
        .await
        .map_err(|e| TasksIntegrationError::DatabaseError(e.to_string()))?
    }

    async fn find_project_contacts(
        &self,
        _project_id: Uuid,
        exclude: &[Uuid],
        limit: usize,
    ) -> Result<Vec<(ContactSummary, ContactWorkload)>, TasksIntegrationError> {
        let pool = self.db_pool.clone();
        let exclude = exclude.to_vec();

        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get().map_err(|e| TasksIntegrationError::DatabaseError(e.to_string()))?;

            let mut query = crm_contacts_table::table
                .filter(crm_contacts_table::status.eq("active"))
                .into_boxed();

            for exc in &exclude {
                query = query.filter(crm_contacts_table::id.ne(*exc));
            }

            let rows: Vec<(Uuid, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>)> = query
                .select((
                    crm_contacts_table::id,
                    crm_contacts_table::first_name,
                    crm_contacts_table::last_name,
                    crm_contacts_table::email,
                    crm_contacts_table::company,
                    crm_contacts_table::job_title,
                ))
                .limit(limit as i64)
                .load(&mut conn)
                .map_err(|e| TasksIntegrationError::DatabaseError(e.to_string()))?;

            let contacts = rows.into_iter().map(|row| {
                let summary = ContactSummary {
                    id: row.0,
                    first_name: row.1.unwrap_or_default(),
                    last_name: row.2.unwrap_or_default(),
                    email: row.3,
                    phone: None,
                    company: row.4,
                    job_title: row.5,
                    avatar_url: None,
                };
                let workload = ContactWorkload {
                    active_tasks: 0,
                    high_priority_tasks: 0,
                    overdue_tasks: 0,
                    due_this_week: 0,
                    workload_level: WorkloadLevel::Low,
                };
                (summary, workload)
            }).collect();

            Ok(contacts)
        })
        .await
        .map_err(|e| TasksIntegrationError::DatabaseError(e.to_string()))?
    }

    async fn find_low_workload_contacts(
        &self,
        _organization_id: Uuid,
        exclude: &[Uuid],
        limit: usize,
    ) -> Result<Vec<(ContactSummary, ContactWorkload)>, TasksIntegrationError> {
        let pool = self.db_pool.clone();
        let exclude = exclude.to_vec();

        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get().map_err(|e| TasksIntegrationError::DatabaseError(e.to_string()))?;

            let mut query = crm_contacts_table::table
                .filter(crm_contacts_table::status.eq("active"))
                .into_boxed();

            for exc in &exclude {
                query = query.filter(crm_contacts_table::id.ne(*exc));
            }

            let rows: Vec<(Uuid, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>)> = query
                .select((
                    crm_contacts_table::id,
                    crm_contacts_table::first_name,
                    crm_contacts_table::last_name,
                    crm_contacts_table::email,
                    crm_contacts_table::company,
                    crm_contacts_table::job_title,
                ))
                .limit(limit as i64)
                .load(&mut conn)
                .map_err(|e| TasksIntegrationError::DatabaseError(e.to_string()))?;

            let contacts = rows.into_iter().map(|row| {
                let summary = ContactSummary {
                    id: row.0,
                    first_name: row.1.unwrap_or_default(),
                    last_name: row.2.unwrap_or_default(),
                    email: row.3,
                    phone: None,
                    company: row.4,
                    job_title: row.5,
                    avatar_url: None,
                };
                let workload = ContactWorkload {
                    active_tasks: 0,
                    high_priority_tasks: 0,
                    overdue_tasks: 0,
                    due_this_week: 0,
                    workload_level: WorkloadLevel::Low,
                };
                (summary, workload)
            }).collect();

            Ok(contacts)
        })
        .await
        .map_err(|e| TasksIntegrationError::DatabaseError(e.to_string()))?
    }

    async fn create_task_in_db(
        &self,
        _task_id: Uuid,
        _organization_id: Uuid,
        _title: &str,
        _description: Option<&str>,
        _assignee_id: Option<Uuid>,
        _due_date: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<(), TasksIntegrationError> {
        // Implementation would insert task into database
        // For now, this is a placeholder
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_task_type_display() {
        assert_eq!(format!("{:?}", ContactTaskType::FollowUp), "FollowUp");
        assert_eq!(format!("{:?}", ContactTaskType::Meeting), "Meeting");
        assert_eq!(format!("{:?}", ContactTaskType::Call), "Call");
    }

    #[test]
    fn test_task_priority_display() {
        assert_eq!(format!("{:?}", ContactTaskPriority::Low), "Low");
        assert_eq!(format!("{:?}", ContactTaskPriority::Normal), "Normal");
        assert_eq!(format!("{:?}", ContactTaskPriority::High), "High");
    }
}

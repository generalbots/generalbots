use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use crate::schema::crm_contacts;
use crate::schema::people;
use crate::schema::tasks;

use super::tasks_service::TasksIntegrationError;
use super::tasks_types::*;

pub(crate) fn update_task_contact_db(
    pool: &crate::DbPool,
    task_contact: &TaskContact,
) -> Result<(), TasksIntegrationError> {
    let mut conn = pool.get().map_err(|e| TasksIntegrationError::DatabaseError(e.to_string()))?;

    let contact_email: Option<String> = crm_contacts::table
        .filter(crm_contacts::id.eq(task_contact.contact_id))
        .select(crm_contacts::email)
        .first(&mut conn)
        .optional()
        .map_err(|e: diesel::result::Error| TasksIntegrationError::DatabaseError(e.to_string()))?
        .flatten();

    let contact_email = match contact_email {
        Some(email) => email,
        None => return Ok(()),
    };

    let person_id: Option<Uuid> = people::table
        .filter(people::email.eq(&contact_email))
        .select(people::id)
        .first::<Uuid>(&mut conn)
        .optional()
        .map_err(|e: diesel::result::Error| TasksIntegrationError::DatabaseError(e.to_string()))?;

    if let Some(pid) = person_id {
        if task_contact.role == TaskContactRole::Assignee {
            diesel::update(tasks::table.filter(tasks::id.eq(task_contact.task_id)))
                .set(tasks::assignee_id.eq(Some(pid)))
                .execute(&mut conn)
                .map_err(|e: diesel::result::Error| TasksIntegrationError::DatabaseError(format!("Failed to update task: {e}")))?;
        }
    }

    Ok(())
}

pub(crate) fn fetch_task_contacts_db(
    pool: &crate::DbPool,
    task_id: Uuid,
) -> Result<Vec<TaskContact>, TasksIntegrationError> {
    let mut conn = pool.get().map_err(|e| TasksIntegrationError::DatabaseError(e.to_string()))?;

    let task_row = tasks::table
        .filter(tasks::id.eq(task_id))
        .select((tasks::id, tasks::assignee_id, tasks::created_at))
        .first::<(Uuid, Option<Uuid>, DateTime<Utc>)>(&mut conn)
        .optional()
        .map_err(|e: diesel::result::Error| TasksIntegrationError::DatabaseError(e.to_string()))?;

    let mut task_contacts = Vec::new();

    if let Some((tid, assignee_id, created_at)) = task_row {
        if let Some(aid) = assignee_id {
        let person_email: Option<String> = people::table
            .filter(people::id.eq(aid))
            .select(people::email)
            .first::<Option<String>>(&mut conn)
            .optional()
            .map_err(|e: diesel::result::Error| TasksIntegrationError::DatabaseError(e.to_string()))?
            .flatten();

        if let Some(email) = person_email {
                let contact_result = crm_contacts::table
                    .filter(crm_contacts::email.eq(&email))
                    .select(crm_contacts::id)
                    .first::<Uuid>(&mut conn)
                    .optional()
                    .unwrap_or(None);

                if let Some(contact_id) = contact_result {
                    task_contacts.push(TaskContact {
                        id: Uuid::new_v4(), task_id: tid, contact_id,
                        role: TaskContactRole::Assignee, assigned_at: created_at,
                        assigned_by: Uuid::nil(), notified: false, notified_at: None, notes: None,
                    });
                }
            }
        }
    }

    Ok(task_contacts)
}

pub(crate) fn fetch_contact_tasks_db(
    pool: &crate::DbPool,
    contact_id: Uuid,
    status_filter: Option<String>,
) -> Result<Vec<ContactTaskWithDetails>, TasksIntegrationError> {
    let mut conn = pool.get().map_err(|e| TasksIntegrationError::DatabaseError(e.to_string()))?;

    let mut db_query = tasks::table
        .filter(tasks::status.ne("deleted"))
        .into_boxed();

    if let Some(status) = status_filter {
        db_query = db_query.filter(tasks::status.eq(status));
    }

    #[derive(Queryable)]
    struct TaskRow {
        id: Uuid, title: String, description: Option<String>,
        status: String, priority: String, due_date: Option<DateTime<Utc>>,
        project_id: Option<Uuid>, progress: i32,
        created_at: DateTime<Utc>, updated_at: DateTime<Utc>,
    }

    let rows: Vec<TaskRow> = db_query
        .order(tasks::created_at.desc())
        .select((
            tasks::id, tasks::title, tasks::description, tasks::status,
            tasks::priority, tasks::due_date, tasks::project_id,
            tasks::progress, tasks::created_at, tasks::updated_at,
        ))
        .limit(50)
        .load(&mut conn)
        .map_err(|e: diesel::result::Error| TasksIntegrationError::DatabaseError(e.to_string()))?;

    Ok(rows.into_iter().map(|row| {
        ContactTaskWithDetails {
            task_contact: TaskContact {
                id: Uuid::new_v4(), task_id: row.id, contact_id,
                role: TaskContactRole::Assignee, assigned_at: Utc::now(),
                assigned_by: Uuid::nil(), notified: false, notified_at: None, notes: None,
            },
            task: TaskSummary {
                id: row.id, title: row.title, description: row.description,
                status: row.status, priority: row.priority, due_date: row.due_date,
                project_id: row.project_id, project_name: None,
                progress: row.progress as u8, created_at: row.created_at, updated_at: row.updated_at,
            },
        }
    }).collect())
}

pub(crate) fn get_assigned_contact_ids_db(
    pool: &crate::DbPool,
    task_id: Uuid,
) -> Result<Vec<Uuid>, TasksIntegrationError> {
    let mut conn = pool.get().map_err(|e| TasksIntegrationError::DatabaseError(e.to_string()))?;

    let assignee_id: Option<Uuid> = tasks::table
        .filter(tasks::id.eq(task_id))
        .select(tasks::assignee_id)
        .first::<Option<Uuid>>(&mut conn)
        .optional()
        .map_err(|e: diesel::result::Error| TasksIntegrationError::DatabaseError(e.to_string()))?
        .flatten();

    if let Some(user_id) = assignee_id {
        let person_email: Option<String> = people::table
            .filter(people::user_id.eq(user_id))
            .select(people::email)
            .first::<Option<String>>(&mut conn)
            .optional()
            .map_err(|e: diesel::result::Error| TasksIntegrationError::DatabaseError(e.to_string()))?
            .flatten();

        if let Some(email) = person_email {
            let contact_ids: Vec<Uuid> = crm_contacts::table
                .filter(crm_contacts::email.eq(&email))
                .select(crm_contacts::id)
                .load(&mut conn)
                .unwrap_or_default();
            return Ok(contact_ids);
        }
    }

    Ok(vec![])
}

pub(crate) fn query_contacts_for_suggestions(
    pool: &crate::DbPool,
    exclude: &[Uuid],
    limit: usize,
) -> Result<Vec<ContactSummary>, TasksIntegrationError> {
    let mut conn = pool.get().map_err(|e| TasksIntegrationError::DatabaseError(e.to_string()))?;

    let mut query = crm_contacts::table
        .filter(crm_contacts::status.eq("active"))
        .into_boxed();

    for exc in exclude {
        query = query.filter(crm_contacts::id.ne(*exc));
    }

    #[derive(Queryable)]
    struct Row {
        id: Uuid, first_name: Option<String>, last_name: Option<String>,
        email: Option<String>, company: Option<String>, job_title: Option<String>,
    }

    let rows: Vec<Row> = query
        .select((
            crm_contacts::id, crm_contacts::first_name, crm_contacts::last_name,
            crm_contacts::email, crm_contacts::company, crm_contacts::job_title,
        ))
        .limit(limit as i64)
        .load(&mut conn)
        .map_err(|e: diesel::result::Error| TasksIntegrationError::DatabaseError(e.to_string()))?;

    Ok(rows.into_iter().map(|row| {
        ContactSummary {
            id: row.id, first_name: row.first_name.unwrap_or_default(),
            last_name: row.last_name.unwrap_or_default(), email: row.email,
            phone: None, company: row.company, job_title: row.job_title, avatar_url: None,
        }
    }).collect())
}

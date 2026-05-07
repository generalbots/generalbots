use chrono::{DateTime, Utc};
use diesel::prelude::*;
use std::sync::Arc;
use uuid::Uuid;

use crate::schema::calendar_events;
use crate::schema::crm_contacts;

use super::calendar_types::*;

pub(crate) fn fetch_contact_events_db(
    pool: Arc<crate::DbPool>,
    contact_id: Uuid,
    from_date: Option<DateTime<Utc>>,
    to_date: Option<DateTime<Utc>>,
) -> Result<Vec<ContactEventWithDetails>, CalendarIntegrationError> {
    let mut conn = pool.get().map_err(|_| CalendarIntegrationError::DatabaseError)?;

    let rows: Vec<(Uuid, String, Option<String>, DateTime<Utc>, DateTime<Utc>, Option<String>)> =
        calendar_events::table
            .filter(calendar_events::start_time.ge(from_date.unwrap_or_else(Utc::now)))
            .filter(
                calendar_events::start_time
                    .le(to_date.unwrap_or_else(|| Utc::now() + chrono::Duration::days(30))),
            )
            .filter(calendar_events::status.ne("cancelled"))
            .order(calendar_events::start_time.asc())
            .select((
                calendar_events::id,
                calendar_events::title,
                calendar_events::description,
                calendar_events::start_time,
                calendar_events::end_time,
                calendar_events::location,
            ))
            .limit(50)
            .load(&mut conn)
            .map_err(|_| CalendarIntegrationError::DatabaseError)?;

    Ok(rows
        .into_iter()
        .map(|row| ContactEventWithDetails {
            event_contact: EventContact {
                id: Uuid::new_v4(),
                event_id: row.0,
                contact_id,
                role: EventContactRole::Attendee,
                response_status: ResponseStatus::Accepted,
                notified: false,
                notified_at: None,
                created_at: Utc::now(),
            },
            event: EventSummary {
                id: row.0,
                title: row.1,
                description: row.2,
                start_time: row.3,
                end_time: row.4,
                location: row.5,
                is_recurring: false,
                organizer_name: None,
            },
        })
        .collect())
}

pub(crate) fn query_contacts_excluding(
    pool: &crate::DbPool,
    exclude_contact_id: Option<Uuid>,
    exclude_ids: &[Uuid],
    limit: usize,
    filter_company: bool,
) -> Result<Vec<ContactSummary>, CalendarIntegrationError> {
    let mut conn = pool.get().map_err(|_| CalendarIntegrationError::DatabaseError)?;

    let mut query = crm_contacts::table
        .filter(crm_contacts::status.eq("active"))
        .into_boxed();

    if filter_company {
        query = query.filter(crm_contacts::company.is_not_null());
    }

    if let Some(cid) = exclude_contact_id {
        query = query.filter(crm_contacts::id.ne(cid));
    }

    for exc in exclude_ids {
        query = query.filter(crm_contacts::id.ne(*exc));
    }

    let rows: Vec<(
        Uuid,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    )> = query
        .select((
            crm_contacts::id,
            crm_contacts::first_name,
            crm_contacts::last_name,
            crm_contacts::email,
            crm_contacts::company,
            crm_contacts::job_title,
        ))
        .limit(limit as i64)
        .load(&mut conn)
        .map_err(|_| CalendarIntegrationError::DatabaseError)?;

    Ok(rows
        .into_iter()
        .map(|row| ContactSummary {
            id: row.0,
            first_name: row.1.unwrap_or_default(),
            last_name: row.2.unwrap_or_default(),
            email: row.3,
            phone: None,
            company: row.4,
            job_title: row.5,
            avatar_url: None,
        })
        .collect())
}

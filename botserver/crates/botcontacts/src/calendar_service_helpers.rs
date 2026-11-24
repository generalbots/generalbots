use chrono::{DateTime, Utc};
use diesel::prelude::*;
use std::sync::Arc;
use uuid::Uuid;

use crate::schema::calendar_events;
use crate::schema::crm_contacts;

use super::calendar_types::*;

/// Verified tenant scope for a request, resolved server-side from the caller's
/// JWT/session — never from client input (#2896). Both columns are carried so
/// every query can be narrowed to the workspace that actually owns the rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TenantScope {
    pub(crate) org_id: Uuid,
    pub(crate) branch_id: Uuid,
}

impl TenantScope {
    /// Pure predicate: does a row carrying `(row_org_id, row_branch_id)`
    /// belong to this tenant? The contact-ownership check and the query
    /// filters below use the same rule, so a bare id from the URL cannot open
    /// another tenant's data.
    pub(crate) fn owns(self, row_org_id: Uuid, row_branch_id: Uuid) -> bool {
        self.org_id == row_org_id && self.branch_id == row_branch_id
    }
}

/// Loads the scope columns of a contact so the caller can decide whether the
/// bare `contact_id` from the URL really belongs to its tenant. A missing row
/// is reported as "not in scope" and becomes a 404 in the service layer.
pub(crate) fn contact_in_scope_db(
    pool: &crate::DbPool,
    scope: TenantScope,
    contact_id: Uuid,
) -> Result<bool, CalendarIntegrationError> {
    let mut conn = pool.get().map_err(|_| CalendarIntegrationError::DatabaseError)?;

    let row: Option<(Uuid, Uuid)> = crm_contacts::table
        .filter(crm_contacts::id.eq(contact_id))
        .select((crm_contacts::org_id, crm_contacts::branch_id))
        .first(&mut conn)
        .optional()
        .map_err(|_| CalendarIntegrationError::DatabaseError)?;

    Ok(row.is_some_and(|(org_id, branch_id)| scope.owns(org_id, branch_id)))
}

pub(crate) fn fetch_contact_events_db(
    pool: Arc<crate::DbPool>,
    contact_id: Uuid,
    scope: TenantScope,
    from_date: Option<DateTime<Utc>>,
    to_date: Option<DateTime<Utc>>,
) -> Result<Vec<ContactEventWithDetails>, CalendarIntegrationError> {
    let mut conn = pool.get().map_err(|_| CalendarIntegrationError::DatabaseError)?;

    let rows: Vec<(Uuid, String, Option<String>, DateTime<Utc>, DateTime<Utc>, Option<String>)> =
        calendar_events::table
            .filter(calendar_events::org_id.eq(scope.org_id))
            .filter(calendar_events::branch_id.eq(scope.branch_id))
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
    scope: TenantScope,
    exclude_contact_id: Option<Uuid>,
    exclude_ids: &[Uuid],
    limit: usize,
    filter_company: bool,
) -> Result<Vec<ContactSummary>, CalendarIntegrationError> {
    let mut conn = pool.get().map_err(|_| CalendarIntegrationError::DatabaseError)?;

    let mut query = crm_contacts::table
        .filter(crm_contacts::org_id.eq(scope.org_id))
        .filter(crm_contacts::branch_id.eq(scope.branch_id))
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
            first_name: crate::value_or_default(row.1),
            last_name: crate::value_or_default(row.2),
            email: row.3,
            phone: None,
            company: row.4,
            job_title: row.5,
            avatar_url: None,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scope_owns_only_the_matching_tenant() {
        let tenant_a = TenantScope {
            org_id: Uuid::new_v4(),
            branch_id: Uuid::new_v4(),
        };
        let tenant_b = TenantScope {
            org_id: Uuid::new_v4(),
            branch_id: Uuid::new_v4(),
        };

        assert!(tenant_a.owns(tenant_a.org_id, tenant_a.branch_id));
        assert!(!tenant_a.owns(tenant_b.org_id, tenant_b.branch_id));
        assert!(!tenant_b.owns(tenant_a.org_id, tenant_a.branch_id));
    }

    #[test]
    fn scope_rejects_partial_scope_matches() {
        let org_id = Uuid::new_v4();
        let branch_id = Uuid::new_v4();
        let scope = TenantScope { org_id, branch_id };

        // Same org but a different workspace, and vice versa, are both foreign.
        assert!(!scope.owns(org_id, Uuid::new_v4()));
        assert!(!scope.owns(Uuid::new_v4(), branch_id));
    }

    #[test]
    fn nil_scope_is_not_owned_by_real_tenant_rows() {
        let scope = TenantScope {
            org_id: Uuid::nil(),
            branch_id: Uuid::nil(),
        };
        assert!(!scope.owns(Uuid::new_v4(), Uuid::new_v4()));
        assert!(scope.owns(Uuid::nil(), Uuid::nil()));
    }
}

use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use super::calendar_types::*;
use super::calendar_service_helpers;

pub struct CalendarIntegrationService {
    db_pool: Arc<crate::DbPool>,
}

impl CalendarIntegrationService {
    pub fn new(pool: Arc<crate::DbPool>) -> Self {
        Self { db_pool: pool }
    }

    pub async fn link_contact_to_event(
        &self,
        organization_id: Uuid,
        event_id: Uuid,
        request: &LinkContactRequest,
    ) -> Result<EventContact, CalendarIntegrationError> {
        self.verify_contact(organization_id, request.contact_id).await?;
        self.verify_event(organization_id, event_id).await?;

        if self.is_contact_linked(event_id, request.contact_id).await? {
            return Err(CalendarIntegrationError::AlreadyLinked);
        }

        let id = Uuid::new_v4();
        let now = Utc::now();
        let role = request.role.clone().unwrap_or_default();

        self.create_event_contact_link(id, event_id, request.contact_id, &role, now)
            .await?;

        let notified = if request.send_notification.unwrap_or(true) {
            self.send_event_invitation(event_id, request.contact_id)
                .await
                .is_ok()
        } else {
            false
        };

        self.log_contact_activity(
            request.contact_id,
            "linked_to_event",
            &format!("Linked to event {event_id}"),
            Some(event_id),
        )
        .await?;

        Ok(EventContact {
            id,
            event_id,
            contact_id: request.contact_id,
            role,
            response_status: ResponseStatus::NeedsAction,
            notified,
            notified_at: if notified { Some(now) } else { None },
            created_at: now,
        })
    }

    pub async fn bulk_link_contacts(
        &self,
        organization_id: Uuid,
        event_id: Uuid,
        request: &BulkLinkContactsRequest,
    ) -> Result<Vec<EventContact>, CalendarIntegrationError> {
        let mut results = Vec::new();
        for contact_id in &request.contact_ids {
            let link_request = LinkContactRequest {
                contact_id: *contact_id,
                role: request.role.clone(),
                send_notification: request.send_notification,
            };
            match self.link_contact_to_event(organization_id, event_id, &link_request).await {
                Ok(event_contact) => results.push(event_contact),
                Err(CalendarIntegrationError::AlreadyLinked) => continue,
                Err(e) => return Err(e),
            }
        }
        Ok(results)
    }

    pub async fn unlink_contact_from_event(
        &self,
        organization_id: Uuid,
        event_id: Uuid,
        contact_id: Uuid,
    ) -> Result<(), CalendarIntegrationError> {
        self.verify_contact(organization_id, contact_id).await?;
        self.verify_event(organization_id, event_id).await?;
        self.delete_event_contact_link(event_id, contact_id).await?;
        self.log_contact_activity(
            contact_id,
            "unlinked_from_event",
            &format!("Unlinked from event {event_id}"),
            Some(event_id),
        )
        .await?;
        Ok(())
    }

    pub async fn update_event_contact(
        &self,
        organization_id: Uuid,
        event_id: Uuid,
        contact_id: Uuid,
        request: &UpdateEventContactRequest,
    ) -> Result<EventContact, CalendarIntegrationError> {
        self.verify_contact(organization_id, contact_id).await?;
        self.verify_event(organization_id, event_id).await?;

        let mut event_contact = self.get_event_contact(event_id, contact_id).await?;
        if let Some(role) = &request.role {
            event_contact.role = role.clone();
        }
        if let Some(status) = &request.response_status {
            event_contact.response_status = status.clone();
        }
        self.update_event_contact_in_db(&event_contact).await?;
        Ok(event_contact)
    }

    pub async fn get_event_contacts(
        &self,
        organization_id: Uuid,
        event_id: Uuid,
        query: &EventContactsQuery,
    ) -> Result<Vec<EventContactWithDetails>, CalendarIntegrationError> {
        self.verify_event(organization_id, event_id).await?;
        let contacts = self.fetch_event_contacts(event_id, query).await?;

        if query.include_contact_details.unwrap_or(true) {
            let mut results = Vec::new();
            for event_contact in contacts {
                if let Ok(contact) = self.get_contact_summary(event_contact.contact_id).await {
                    results.push(EventContactWithDetails {
                        event_contact,
                        contact,
                    });
                }
            }
            Ok(results)
        } else {
            Ok(contacts
                .into_iter()
                .map(|ec| EventContactWithDetails {
                    contact: ContactSummary {
                        id: ec.contact_id,
                        first_name: String::new(),
                        last_name: String::new(),
                        email: None,
                        phone: None,
                        company: None,
                        job_title: None,
                        avatar_url: None,
                    },
                    event_contact: ec,
                })
                .collect())
        }
    }

    pub async fn get_contact_events(
        &self,
        organization_id: Uuid,
        contact_id: Uuid,
        query: &ContactEventsQuery,
    ) -> Result<ContactEventsResponse, CalendarIntegrationError> {
        self.verify_contact(organization_id, contact_id).await?;
        let events = self.fetch_contact_events(contact_id, query).await?;
        let total_count = events.len() as u32;
        let now = Utc::now();
        let upcoming_count = events.iter().filter(|e| e.event.start_time > now).count() as u32;
        let past_count = total_count - upcoming_count;

        Ok(ContactEventsResponse {
            events,
            total_count,
            upcoming_count,
            past_count,
        })
    }

    pub async fn get_suggested_contacts(
        &self,
        organization_id: Uuid,
        event_id: Uuid,
        limit: Option<u32>,
    ) -> Result<Vec<SuggestedContact>, CalendarIntegrationError> {
        self.verify_event(organization_id, event_id).await?;
        let limit = limit.unwrap_or(10);
        let mut suggestions: Vec<SuggestedContact> = Vec::new();

        if let Some(organizer_id) = self.get_event_organizer_contact_id(event_id).await? {
            let linked = self.get_linked_contact_ids(event_id).await?;
            let collaborators = self.find_frequent_collaborators(organizer_id, &linked, 5).await?;
            for contact in collaborators {
                suggestions.push(SuggestedContact {
                    contact,
                    reason: SuggestionReason::FrequentCollaborator,
                    score: 0.9,
                });
            }
        }

        let linked = self.get_linked_contact_ids(event_id).await?;
        let company_contacts = self.find_same_company_contacts(event_id, &linked, 5).await?;
        for contact in company_contacts {
            suggestions.push(SuggestedContact {
                contact,
                reason: SuggestionReason::SameCompany,
                score: 0.7,
            });
        }

        suggestions.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        suggestions.truncate(limit as usize);
        Ok(suggestions)
    }

    pub async fn find_contacts_for_event(
        &self,
        organization_id: Uuid,
        emails: &[String],
    ) -> Result<HashMap<String, Option<ContactSummary>>, CalendarIntegrationError> {
        let mut results = HashMap::new();
        for email in emails {
            let contact = self.find_contact_by_email(organization_id, email).await.ok();
            results.insert(email.clone(), contact);
        }
        Ok(results)
    }

    pub async fn create_contacts_from_attendees(
        &self,
        organization_id: Uuid,
        user_id: Uuid,
        attendees: &[AttendeeInfo],
    ) -> Result<Vec<ContactSummary>, CalendarIntegrationError> {
        let mut created = Vec::new();
        for attendee in attendees {
            if self.find_contact_by_email(organization_id, &attendee.email).await.is_ok() {
                continue;
            }
            let contact_id = Uuid::new_v4();
            let now = Utc::now();
            self.create_contact_from_attendee(contact_id, organization_id, user_id, attendee, now)
                .await?;
            created.push(ContactSummary {
                id: contact_id,
                first_name: attendee.name.split_whitespace().next().unwrap_or("").to_string(),
                last_name: attendee.name.split_whitespace().skip(1).collect::<Vec<_>>().join(" "),
                email: Some(attendee.email.clone()),
                phone: None,
                company: attendee.company.clone(),
                job_title: None,
                avatar_url: None,
            });
        }
        Ok(created)
    }

    async fn verify_contact(&self, _org: Uuid, _cid: Uuid) -> Result<(), CalendarIntegrationError> {
        Ok(())
    }

    async fn verify_event(&self, _org: Uuid, _eid: Uuid) -> Result<(), CalendarIntegrationError> {
        Ok(())
    }

    async fn is_contact_linked(&self, _eid: Uuid, _cid: Uuid) -> Result<bool, CalendarIntegrationError> {
        Ok(false)
    }

    async fn create_event_contact_link(
        &self, _id: Uuid, _eid: Uuid, _cid: Uuid, _role: &EventContactRole, _at: DateTime<Utc>,
    ) -> Result<(), CalendarIntegrationError> {
        Ok(())
    }

    async fn delete_event_contact_link(&self, _eid: Uuid, _cid: Uuid) -> Result<(), CalendarIntegrationError> {
        Ok(())
    }

    async fn get_event_contact(
        &self, event_id: Uuid, contact_id: Uuid,
    ) -> Result<EventContact, CalendarIntegrationError> {
        Ok(EventContact {
            id: Uuid::new_v4(), event_id, contact_id,
            role: EventContactRole::Attendee,
            response_status: ResponseStatus::NeedsAction,
            notified: false, notified_at: None, created_at: Utc::now(),
        })
    }

    async fn update_event_contact_in_db(&self, _ec: &EventContact) -> Result<(), CalendarIntegrationError> {
        Ok(())
    }

    async fn fetch_event_contacts(
        &self, event_id: Uuid, _q: &EventContactsQuery,
    ) -> Result<Vec<EventContact>, CalendarIntegrationError> {
        Ok(vec![EventContact {
            id: Uuid::new_v4(), event_id, contact_id: Uuid::new_v4(),
            role: EventContactRole::Attendee,
            response_status: ResponseStatus::Accepted,
            notified: true, notified_at: Some(Utc::now()), created_at: Utc::now(),
        }])
    }

    async fn fetch_contact_events(
        &self, contact_id: Uuid, query: &ContactEventsQuery,
    ) -> Result<Vec<ContactEventWithDetails>, CalendarIntegrationError> {
        let pool = self.db_pool.clone();
        let from_date = query.from_date;
        let to_date = query.to_date;
        tokio::task::spawn_blocking(move || {
            calendar_service_helpers::fetch_contact_events_db(pool, contact_id, from_date, to_date)
        })
        .await
        .map_err(|e: tokio::task::JoinError| {
            log::error!("Spawn blocking error: {e}");
            CalendarIntegrationError::DatabaseError
        })?
    }

    async fn get_contact_summary(&self, cid: Uuid) -> Result<ContactSummary, CalendarIntegrationError> {
        Ok(ContactSummary {
            id: cid, first_name: String::new(), last_name: String::new(),
            email: None, phone: None, company: None, job_title: None, avatar_url: None,
        })
    }

    async fn get_linked_contact_ids(&self, _eid: Uuid) -> Result<Vec<Uuid>, CalendarIntegrationError> {
        Ok(vec![])
    }

    async fn get_event_organizer_contact_id(&self, _eid: Uuid) -> Result<Option<Uuid>, CalendarIntegrationError> {
        Ok(None)
    }

    async fn find_frequent_collaborators(
        &self, contact_id: Uuid, exclude: &[Uuid], limit: usize,
    ) -> Result<Vec<ContactSummary>, CalendarIntegrationError> {
        let pool = self.db_pool.clone();
        let exclude = exclude.to_vec();
        tokio::task::spawn_blocking(move || {
            calendar_service_helpers::query_contacts_excluding(&pool, Some(contact_id), &exclude, limit, false)
        })
        .await
        .map_err(|e: tokio::task::JoinError| {
            log::error!("Spawn blocking error: {e}");
            CalendarIntegrationError::DatabaseError
        })?
    }

    async fn find_same_company_contacts(
        &self, _eid: Uuid, exclude: &[Uuid], limit: usize,
    ) -> Result<Vec<ContactSummary>, CalendarIntegrationError> {
        let pool = self.db_pool.clone();
        let exclude = exclude.to_vec();
        tokio::task::spawn_blocking(move || {
            calendar_service_helpers::query_contacts_excluding(&pool, None, &exclude, limit, true)
        })
        .await
        .map_err(|_| CalendarIntegrationError::DatabaseError)?
    }

    async fn find_contact_by_email(&self, _org: Uuid, _email: &str) -> Result<ContactSummary, CalendarIntegrationError> {
        Err(CalendarIntegrationError::ContactNotFound)
    }

    async fn create_contact_from_attendee(
        &self, _cid: Uuid, _org: Uuid, _uid: Uuid, _att: &AttendeeInfo, _at: DateTime<Utc>,
    ) -> Result<(), CalendarIntegrationError> {
        Ok(())
    }

    async fn send_event_invitation(&self, _eid: Uuid, _cid: Uuid) -> Result<(), CalendarIntegrationError> {
        Ok(())
    }

    async fn log_contact_activity(
        &self, _cid: Uuid, _ty: &str, _desc: &str, _rid: Option<Uuid>,
    ) -> Result<(), CalendarIntegrationError> {
        Ok(())
    }
}

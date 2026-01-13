use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::core::shared::schema::{calendar_events, crm_contacts};
use crate::shared::state::AppState;
use crate::shared::utils::DbPool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventContact {
    pub id: Uuid,
    pub event_id: Uuid,
    pub contact_id: Uuid,
    pub role: EventContactRole,
    pub response_status: ResponseStatus,
    pub notified: bool,
    pub notified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum EventContactRole {
    #[default]
    Attendee,
    Organizer,
    OptionalAttendee,
    Resource,
    Speaker,
    Host,
}

impl std::fmt::Display for EventContactRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventContactRole::Attendee => write!(f, "attendee"),
            EventContactRole::Organizer => write!(f, "organizer"),
            EventContactRole::OptionalAttendee => write!(f, "optional"),
            EventContactRole::Resource => write!(f, "resource"),
            EventContactRole::Speaker => write!(f, "speaker"),
            EventContactRole::Host => write!(f, "host"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum ResponseStatus {
    #[default]
    NeedsAction,
    Accepted,
    Declined,
    Tentative,
    Delegated,
}

impl std::fmt::Display for ResponseStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResponseStatus::NeedsAction => write!(f, "needs_action"),
            ResponseStatus::Accepted => write!(f, "accepted"),
            ResponseStatus::Declined => write!(f, "declined"),
            ResponseStatus::Tentative => write!(f, "tentative"),
            ResponseStatus::Delegated => write!(f, "delegated"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkContactRequest {
    pub contact_id: Uuid,
    pub role: Option<EventContactRole>,
    pub send_notification: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkLinkContactsRequest {
    pub contact_ids: Vec<Uuid>,
    pub role: Option<EventContactRole>,
    pub send_notification: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateEventContactRequest {
    pub role: Option<EventContactRole>,
    pub response_status: Option<ResponseStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventContactsQuery {
    pub role: Option<EventContactRole>,
    pub response_status: Option<ResponseStatus>,
    pub include_contact_details: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactEventsQuery {
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
    pub role: Option<EventContactRole>,
    pub response_status: Option<ResponseStatus>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventContactWithDetails {
    pub event_contact: EventContact,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSummary {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub location: Option<String>,
    pub is_recurring: bool,
    pub organizer_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactEventWithDetails {
    pub event_contact: EventContact,
    pub event: EventSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactEventsResponse {
    pub events: Vec<ContactEventWithDetails>,
    pub total_count: u32,
    pub upcoming_count: u32,
    pub past_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedContact {
    pub contact: ContactSummary,
    pub reason: SuggestionReason,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestionReason {
    FrequentCollaborator,
    SameCompany,
    PreviousAttendee,
    RelatedProject,
    OrganizationMember,
    RecentlyContacted,
}

impl std::fmt::Display for SuggestionReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SuggestionReason::FrequentCollaborator => write!(f, "Frequent collaborator"),
            SuggestionReason::SameCompany => write!(f, "Same company"),
            SuggestionReason::PreviousAttendee => write!(f, "Previously attended similar events"),
            SuggestionReason::RelatedProject => write!(f, "Related to project"),
            SuggestionReason::OrganizationMember => write!(f, "Organization member"),
            SuggestionReason::RecentlyContacted => write!(f, "Recently contacted"),
        }
    }
}

pub struct CalendarIntegrationService {}

impl CalendarIntegrationService {
    pub fn new(_pool: DbPool) -> Self {
        Self {}
    }

    pub async fn link_contact_to_event(
        &self,
        organization_id: Uuid,
        event_id: Uuid,
        request: &LinkContactRequest,
    ) -> Result<EventContact, CalendarIntegrationError> {
        // Verify contact exists and belongs to organization
        self.verify_contact(organization_id, request.contact_id).await?;

        // Verify event exists
        self.verify_event(organization_id, event_id).await?;

        // Check if already linked
        if self.is_contact_linked(event_id, request.contact_id).await? {
            return Err(CalendarIntegrationError::AlreadyLinked);
        }

        let id = Uuid::new_v4();
        let now = Utc::now();
        let role = request.role.clone().unwrap_or_default();

        // Create link in database
        self.create_event_contact_link(id, event_id, request.contact_id, &role, now)
            .await?;

        // Send notification if requested
        let notified = if request.send_notification.unwrap_or(true) {
            self.send_event_invitation(event_id, request.contact_id).await.is_ok()
        } else {
            false
        };

        // Log activity
        self.log_contact_activity(
            request.contact_id,
            "linked_to_event",
            &format!("Linked to event {}", event_id),
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
                Err(CalendarIntegrationError::AlreadyLinked) => {
                    // Skip already linked contacts
                    continue;
                }
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
        // Verify ownership
        self.verify_contact(organization_id, contact_id).await?;
        self.verify_event(organization_id, event_id).await?;

        // Delete link
        self.delete_event_contact_link(event_id, contact_id).await?;

        // Log activity
        self.log_contact_activity(
            contact_id,
            "unlinked_from_event",
            &format!("Unlinked from event {}", event_id),
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

        let upcoming_count = events
            .iter()
            .filter(|e| e.event.start_time > now)
            .count() as u32;
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

        // Get event details for context
        let event = self.get_event_details(event_id).await?;

        // Get already linked contacts to exclude
        let linked_contacts = self.get_linked_contact_ids(event_id).await?;

        // Find frequent collaborators of the organizer
        if let Some(organizer_id) = self.get_event_organizer_contact_id(event_id).await? {
            let collaborators = self
                .find_frequent_collaborators(organizer_id, &linked_contacts, 5)
                .await?;
            for contact in collaborators {
                suggestions.push(SuggestedContact {
                    contact,
                    reason: SuggestionReason::FrequentCollaborator,
                    score: 0.9,
                });
            }
        }

        // Find contacts from same company as existing attendees
        let company_contacts = self
            .find_same_company_contacts(event_id, &linked_contacts, 5)
            .await?;
        for contact in company_contacts {
            suggestions.push(SuggestedContact {
                contact,
                reason: SuggestionReason::SameCompany,
                score: 0.7,
            });
        }

        // Find contacts who attended similar events
        let similar_attendees = self
            .find_similar_event_attendees(&event.title, &linked_contacts, 5)
            .await?;
        for contact in similar_attendees {
            suggestions.push(SuggestedContact {
                contact,
                reason: SuggestionReason::PreviousAttendee,
                score: 0.6,
            });
        }

        // Sort by score and limit
        suggestions.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
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
        let mut created_contacts = Vec::new();

        for attendee in attendees {
            // Check if contact already exists
            if self
                .find_contact_by_email(organization_id, &attendee.email)
                .await
                .is_ok()
            {
                continue;
            }

            // Create new contact
            let contact_id = Uuid::new_v4();
            let now = Utc::now();

            self.create_contact_from_attendee(
                contact_id,
                organization_id,
                user_id,
                attendee,
                now,
            )
            .await?;

            created_contacts.push(ContactSummary {
                id: contact_id,
                first_name: attendee.name.split_whitespace().next().unwrap_or("").to_string(),
                last_name: attendee
                    .name
                    .split_whitespace()
                    .skip(1)
                    .collect::<Vec<_>>()
                    .join(" "),
                email: Some(attendee.email.clone()),
                phone: None,
                company: attendee.company.clone(),
                job_title: None,
                avatar_url: None,
            });
        }

        Ok(created_contacts)
    }

    // Helper methods (database operations - stubs for implementation)

    async fn verify_contact(
        &self,
        _organization_id: Uuid,
        _contact_id: Uuid,
    ) -> Result<(), CalendarIntegrationError> {
        // Verify contact exists and belongs to organization
        Ok(())
    }

    async fn verify_event(
        &self,
        _organization_id: Uuid,
        _event_id: Uuid,
    ) -> Result<(), CalendarIntegrationError> {
        // Verify event exists and belongs to organization
        Ok(())
    }

    async fn is_contact_linked(
        &self,
        _event_id: Uuid,
        _contact_id: Uuid,
    ) -> Result<bool, CalendarIntegrationError> {
        // Check if contact is already linked to event
        Ok(false)
    }

    async fn create_event_contact_link(
        &self,
        _id: Uuid,
        _event_id: Uuid,
        _contact_id: Uuid,
        _role: &EventContactRole,
        _created_at: DateTime<Utc>,
    ) -> Result<(), CalendarIntegrationError> {
        // Insert into event_contacts table
        Ok(())
    }

    async fn delete_event_contact_link(
        &self,
        _event_id: Uuid,
        _contact_id: Uuid,
    ) -> Result<(), CalendarIntegrationError> {
        // Delete from event_contacts table
        Ok(())
    }

    async fn get_event_contact(
        &self,
        event_id: Uuid,
        contact_id: Uuid,
    ) -> Result<EventContact, CalendarIntegrationError> {
        // Query event_contacts table
        Ok(EventContact {
            id: Uuid::new_v4(),
            event_id,
            contact_id,
            role: EventContactRole::Attendee,
            response_status: ResponseStatus::NeedsAction,
            notified: false,
            notified_at: None,
            created_at: Utc::now(),
        })
    }

    async fn update_event_contact_in_db(
        &self,
        _event_contact: &EventContact,
    ) -> Result<(), CalendarIntegrationError> {
        // Update event_contacts table
        Ok(())
    }

    async fn fetch_event_contacts(
        &self,
        event_id: Uuid,
        _query: &EventContactsQuery,
    ) -> Result<Vec<EventContact>, CalendarIntegrationError> {
        // Return mock data for contacts linked to this event
        // In production, this would query an event_contacts junction table
        Ok(vec![
            EventContact {
                id: Uuid::new_v4(),
                event_id,
                contact_id: Uuid::new_v4(),
                role: EventContactRole::Attendee,
                response_status: ResponseStatus::Accepted,
                notified: true,
                notified_at: Some(Utc::now()),
                created_at: Utc::now(),
            }
        ])
    }

    async fn fetch_contact_events(
        &self,
        contact_id: Uuid,
        query: &ContactEventsQuery,
    ) -> Result<Vec<ContactEventWithDetails>, CalendarIntegrationError> {
        let pool = self.pool.clone();
        let from_date = query.from_date;
        let to_date = query.to_date;

        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get().map_err(|e| CalendarIntegrationError::DatabaseError(e.to_string()))?;

            // Get events for the contact's organization in the date range
            let rows: Vec<(Uuid, String, Option<String>, DateTime<Utc>, DateTime<Utc>, Option<String>)> = calendar_events::table
                .filter(calendar_events::start_time.ge(from_date.unwrap_or(Utc::now())))
                .filter(calendar_events::start_time.le(to_date.unwrap_or(Utc::now() + chrono::Duration::days(30))))
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
                .map_err(|e| CalendarIntegrationError::DatabaseError(e.to_string()))?;

            let events = rows.into_iter().map(|row| {
                ContactEventWithDetails {
                    link: EventContact {
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
                }
            }).collect();

            Ok(events)
        })
        .await
        .map_err(|e| CalendarIntegrationError::DatabaseError(e.to_string()))?
    }

    async fn get_contact_summary(
        &self,
        contact_id: Uuid,
    ) -> Result<ContactSummary, CalendarIntegrationError> {
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

    async fn get_event_details(
        &self,
        event_id: Uuid,
    ) -> Result<EventSummary, CalendarIntegrationError> {
        // Query events table
        Ok(EventSummary {
            id: event_id,
            title: "Meeting".to_string(),
            description: None,
            start_time: Utc::now(),
            end_time: Utc::now(),
            location: None,
            is_recurring: false,
            organizer_name: None,
        })
    }

    async fn get_linked_contact_ids(
        &self,
        event_id: Uuid,
    ) -> Result<Vec<Uuid>, CalendarIntegrationError> {
        // In production, query event_contacts junction table
        // For now return empty - would need junction table to be created
        let _ = event_id;
        Ok(vec![])
    }

    async fn get_event_organizer_contact_id(
        &self,
        _event_id: Uuid,
    ) -> Result<Option<Uuid>, CalendarIntegrationError> {
        // Get organizer's contact ID if exists
        Ok(None)
    }

    async fn find_frequent_collaborators(
        &self,
        contact_id: Uuid,
        exclude: &[Uuid],
        limit: usize,
    ) -> Result<Vec<ContactSummary>, CalendarIntegrationError> {
        let pool = self.pool.clone();
        let exclude = exclude.to_vec();

        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get().map_err(|e| CalendarIntegrationError::DatabaseError(e.to_string()))?;

            // Find other contacts in the same organization, excluding specified ones
            let mut query = crm_contacts::table
                .filter(crm_contacts::id.ne(contact_id))
                .filter(crm_contacts::status.eq("active"))
                .into_boxed();

            for exc in &exclude {
                query = query.filter(crm_contacts::id.ne(*exc));
            }

            let rows: Vec<(Uuid, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>)> = query
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
                .map_err(|e| CalendarIntegrationError::DatabaseError(e.to_string()))?;

            let contacts = rows.into_iter().map(|row| {
                ContactSummary {
                    id: row.0,
                    first_name: row.1.unwrap_or_default(),
                    last_name: row.2.unwrap_or_default(),
                    email: row.3,
                    phone: None,
                    company: row.4,
                    job_title: row.5,
                    avatar_url: None,
                }
            }).collect();

            Ok(contacts)
        })
        .await
        .map_err(|e| CalendarIntegrationError::DatabaseError(e.to_string()))?
    }

    async fn find_same_company_contacts(
        &self,
        _event_id: Uuid,
        exclude: &[Uuid],
        limit: usize,
    ) -> Result<Vec<ContactSummary>, CalendarIntegrationError> {
        let pool = self.pool.clone();
        let exclude = exclude.to_vec();

        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get().map_err(|e| CalendarIntegrationError::DatabaseError(e.to_string()))?;

            // Find contacts with company field set
            let mut query = crm_contacts::table
                .filter(crm_contacts::company.is_not_null())
                .filter(crm_contacts::status.eq("active"))
                .into_boxed();

            for exc in &exclude {
                query = query.filter(crm_contacts::id.ne(*exc));
            }

            let rows: Vec<(Uuid, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>)> = query
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
                .map_err(|e| CalendarIntegrationError::DatabaseError(e.to_string()))?;

            let contacts = rows.into_iter().map(|row| {
                ContactSummary {
                    id: row.0,
                    first_name: row.1.unwrap_or_default(),
                    last_name: row.2.unwrap_or_default(),
                    email: row.3,
                    phone: None,
                    company: row.4,
                    job_title: row.5,
                    avatar_url: None,
                }
            }).collect();

            Ok(contacts)
        })
        .await
        .map_err(|e| CalendarIntegrationError::DatabaseError(e.to_string()))?
    }

    async fn find_similar_event_attendees(
        &self,
        _event_title: &str,
        exclude: &[Uuid],
        limit: usize,
    ) -> Result<Vec<ContactSummary>, CalendarIntegrationError> {
        let pool = self.pool.clone();
        let exclude = exclude.to_vec();

        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get().map_err(|e| CalendarIntegrationError::DatabaseError(e.to_string()))?;

            // Find active contacts
            let mut query = crm_contacts::table
                .filter(crm_contacts::status.eq("active"))
                .into_boxed();

            for exc in &exclude {
                query = query.filter(crm_contacts::id.ne(*exc));
            }

            let rows: Vec<(Uuid, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>)> = query
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
                .map_err(|e| CalendarIntegrationError::DatabaseError(e.to_string()))?;

            let contacts = rows.into_iter().map(|row| {
                ContactSummary {
                    id: row.0,
                    first_name: row.1.unwrap_or_default(),
                    last_name: row.2.unwrap_or_default(),
                    email: row.3,
                    phone: None,
                    company: row.4,
                    job_title: row.5,
                    avatar_url: None,
                }
            }).collect();

            Ok(contacts)
        })
        .await
        .map_err(|e| CalendarIntegrationError::DatabaseError(e.to_string()))?
    }

    async fn find_contact_by_email(
        &self,
        _organization_id: Uuid,
        _email: &str,
    ) -> Result<ContactSummary, CalendarIntegrationError> {
        Err(CalendarIntegrationError::ContactNotFound)
    }

    async fn create_contact_from_attendee(
        &self,
        _contact_id: Uuid,
        _organization_id: Uuid,
        _user_id: Uuid,
        _attendee: &AttendeeInfo,
        _created_at: DateTime<Utc>,
    ) -> Result<(), CalendarIntegrationError> {
        // Insert into contacts table
        Ok(())
    }

    async fn send_event_invitation(
        &self,
        _event_id: Uuid,
        _contact_id: Uuid,
    ) -> Result<(), CalendarIntegrationError> {
        // Send email invitation
        Ok(())
    }

    async fn log_contact_activity(
        &self,
        _contact_id: Uuid,
        _activity_type: &str,
        _description: &str,
        _related_id: Option<Uuid>,
    ) -> Result<(), CalendarIntegrationError> {
        // Log activity to contact_activities table
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttendeeInfo {
    pub email: String,
    pub name: String,
    pub company: Option<String>,
}

#[derive(Debug, Clone)]
pub enum CalendarIntegrationError {
    DatabaseError,
    ContactNotFound,
    EventNotFound,
    AlreadyLinked,
    NotLinked,
    Unauthorized,
    InvalidInput(String),
}

impl std::fmt::Display for CalendarIntegrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CalendarIntegrationError::DatabaseError => write!(f, "Database error"),
            CalendarIntegrationError::ContactNotFound => write!(f, "Contact not found"),
            CalendarIntegrationError::EventNotFound => write!(f, "Event not found"),
            CalendarIntegrationError::AlreadyLinked => {
                write!(f, "Contact is already linked to this event")
            }
            CalendarIntegrationError::NotLinked => {
                write!(f, "Contact is not linked to this event")
            }
            CalendarIntegrationError::Unauthorized => write!(f, "Unauthorized"),
            CalendarIntegrationError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
        }
    }
}

impl std::error::Error for CalendarIntegrationError {}

impl IntoResponse for CalendarIntegrationError {
    fn into_response(self) -> axum::response::Response {
        use axum::http::StatusCode;

        let (status, message) = match self {
            CalendarIntegrationError::DatabaseError => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
            CalendarIntegrationError::ContactNotFound => (StatusCode::NOT_FOUND, self.to_string()),
            CalendarIntegrationError::EventNotFound => (StatusCode::NOT_FOUND, self.to_string()),
            CalendarIntegrationError::AlreadyLinked => (StatusCode::CONFLICT, self.to_string()),
            CalendarIntegrationError::NotLinked => (StatusCode::NOT_FOUND, self.to_string()),
            CalendarIntegrationError::Unauthorized => (StatusCode::FORBIDDEN, self.to_string()),
            CalendarIntegrationError::InvalidInput(_) => {
                (StatusCode::BAD_REQUEST, self.to_string())
            }
        };

        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

pub fn create_calendar_integration_tables_migration() -> String {
    r#"
    CREATE TABLE IF NOT EXISTS event_contacts (
        id UUID PRIMARY KEY,
        event_id UUID NOT NULL REFERENCES calendar_events(id) ON DELETE CASCADE,
        contact_id UUID NOT NULL REFERENCES contacts(id) ON DELETE CASCADE,
        role VARCHAR(50) NOT NULL DEFAULT 'attendee',
        response_status VARCHAR(50) NOT NULL DEFAULT 'needs_action',
        notified BOOLEAN NOT NULL DEFAULT FALSE,
        notified_at TIMESTAMP WITH TIME ZONE,
        created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

        UNIQUE(event_id, contact_id)
    );

    CREATE INDEX IF NOT EXISTS idx_event_contacts_event_id ON event_contacts(event_id);
    CREATE INDEX IF NOT EXISTS idx_event_contacts_contact_id ON event_contacts(contact_id);
    CREATE INDEX IF NOT EXISTS idx_event_contacts_role ON event_contacts(role);
    CREATE INDEX IF NOT EXISTS idx_event_contacts_response_status ON event_contacts(response_status);
    "#
    .to_string()
}

pub fn calendar_integration_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Event contacts
        .route("/events/:event_id/contacts", get(get_event_contacts_handler))
        .route("/events/:event_id/contacts", post(link_contact_handler))
        .route(
            "/events/:event_id/contacts/bulk",
            post(bulk_link_contacts_handler),
        )
        .route(
            "/events/:event_id/contacts/:contact_id",
            delete(unlink_contact_handler),
        )
        .route(
            "/events/:event_id/contacts/:contact_id",
            post(update_event_contact_handler),
        )
        .route(
            "/events/:event_id/contacts/suggestions",
            get(get_suggestions_handler),
        )
        // Contact events
        .route("/contacts/:contact_id/events", get(get_contact_events_handler))
        // Utilities
        .route("/events/:event_id/find-contacts", post(find_contacts_handler))
        .route(
            "/events/:event_id/create-contacts",
            post(create_contacts_from_attendees_handler),
        )
}

// Route handlers

async fn get_event_contacts_handler(
    State(state): State<Arc<AppState>>,
    Path(event_id): Path<Uuid>,
    Query(query): Query<EventContactsQuery>,
) -> impl IntoResponse {
    let service = CalendarIntegrationService::new(state.conn.clone());
    let org_id = Uuid::new_v4();

    match service.get_event_contacts(org_id, event_id, &query).await {
        Ok(contacts) => Json(contacts).into_response(),
        Err(e) => e.into_response(),
    }
}

async fn link_contact_handler(
    State(state): State<Arc<AppState>>,
    Path(event_id): Path<Uuid>,
    Json(request): Json<LinkContactRequest>,
) -> impl IntoResponse {
    let service = CalendarIntegrationService::new(state.conn.clone());
    let org_id = Uuid::new_v4();

    match service.link_contact_to_event(org_id, event_id, &request).await {
        Ok(event_contact) => Json(event_contact).into_response(),
        Err(e) => e.into_response(),
    }
}

async fn bulk_link_contacts_handler(
    State(state): State<Arc<AppState>>,
    Path(event_id): Path<Uuid>,
    Json(request): Json<BulkLinkContactsRequest>,
) -> impl IntoResponse {
    let service = CalendarIntegrationService::new(state.conn.clone());
    let org_id = Uuid::new_v4();

    match service.bulk_link_contacts(org_id, event_id, &request).await {
        Ok(contacts) => Json(contacts).into_response(),
        Err(e) => e.into_response(),
    }
}

async fn unlink_contact_handler(
    State(state): State<Arc<AppState>>,
    Path((event_id, contact_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let service = CalendarIntegrationService::new(state.conn.clone());
    let org_id = Uuid::new_v4();

    match service.unlink_contact_from_event(org_id, event_id, contact_id).await {
        Ok(_) => Json(serde_json::json!({ "success": true })).into_response(),
        Err(e) => e.into_response(),
    }
}

async fn update_event_contact_handler(
    State(state): State<Arc<AppState>>,
    Path((event_id, contact_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<UpdateEventContactRequest>,
) -> impl IntoResponse {
    let service = CalendarIntegrationService::new(state.conn.clone());
    let org_id = Uuid::new_v4();

    match service.update_event_contact(org_id, event_id, contact_id, &request).await {
        Ok(contact) => Json(contact).into_response(),
        Err(e) => e.into_response(),
    }
}

async fn get_suggestions_handler(
    State(state): State<Arc<AppState>>,
    Path(event_id): Path<Uuid>,
) -> impl IntoResponse {
    let service = CalendarIntegrationService::new(state.conn.clone());
    let org_id = Uuid::new_v4();

    match service.get_suggested_contacts(org_id, event_id, None).await {
        Ok(suggestions) => Json(suggestions).into_response(),
        Err(e) => e.into_response(),
    }
}

async fn get_contact_events_handler(
    State(state): State<Arc<AppState>>,
    Path(contact_id): Path<Uuid>,
    Query(query): Query<ContactEventsQuery>,
) -> impl IntoResponse {
    let service = CalendarIntegrationService::new(state.conn.clone());
    let org_id = Uuid::new_v4();

    match service.get_contact_events(org_id, contact_id, &query).await {
        Ok(events) => Json(events).into_response(),
        Err(e) => e.into_response(),
    }
}

async fn find_contacts_handler(
    State(state): State<Arc<AppState>>,
    Path(event_id): Path<Uuid>,
) -> impl IntoResponse {
    log::debug!("Finding contacts for event {event_id}");
    let service = CalendarIntegrationService::new(state.conn.clone());
    let org_id = Uuid::new_v4();

    match service.find_contacts_for_event(org_id, &[]).await {
        Ok(contacts) => Json(contacts).into_response(),
        Err(e) => e.into_response(),
    }
}

async fn create_contacts_from_attendees_handler(
    State(state): State<Arc<AppState>>,
    Path(event_id): Path<Uuid>,
    Json(attendees): Json<Vec<AttendeeInfo>>,
) -> impl IntoResponse {
    let service = CalendarIntegrationService::new(state.conn.clone());
    let org_id = Uuid::new_v4();

    match service.create_contacts_from_attendees(org_id, event_id, &attendees).await {
        Ok(contacts) => Json(contacts).into_response(),
        Err(e) => e.into_response(),
    }
}

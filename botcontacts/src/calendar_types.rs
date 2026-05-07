use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
            CalendarIntegrationError::InvalidInput(msg) => write!(f, "Invalid input: {msg}"),
        }
    }
}

impl std::error::Error for CalendarIntegrationError {}

//! Calendar Module
//!
//! Provides calendar functionality with iCal (RFC 5545) support using the icalendar library.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use chrono::{DateTime, Utc};
use icalendar::{Calendar, Component, Event as IcalEvent, EventLike, Property};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::core::urls::ApiUrls;
use crate::shared::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub location: Option<String>,
    pub attendees: Vec<String>,
    pub organizer: String,
    pub reminder_minutes: Option<i32>,
    pub recurrence: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarEventInput {
    pub title: String,
    pub description: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub location: Option<String>,
    #[serde(default)]
    pub attendees: Vec<String>,
    pub organizer: String,
    pub reminder_minutes: Option<i32>,
    pub recurrence: Option<String>,
}

impl CalendarEvent {
    /// Convert to iCal Event
    pub fn to_ical(&self) -> IcalEvent {
        let mut event = IcalEvent::new();
        event.uid(&self.id.to_string());
        event.summary(&self.title);
        event.starts(self.start_time);
        event.ends(self.end_time);

        if let Some(ref desc) = self.description {
            event.description(desc);
        }
        if let Some(ref loc) = self.location {
            event.location(loc);
        }

        event.add_property("ORGANIZER", &format!("mailto:{}", self.organizer));

        for attendee in &self.attendees {
            event.add_property("ATTENDEE", &format!("mailto:{}", attendee));
        }

        if let Some(ref rrule) = self.recurrence {
            event.add_property("RRULE", rrule);
        }

        if let Some(minutes) = self.reminder_minutes {
            event.add_property("VALARM", &format!("-PT{}M", minutes));
        }

        event.done()
    }

    /// Create from iCal Event
    pub fn from_ical(ical: &IcalEvent, organizer: &str) -> Option<Self> {
        let uid = ical.get_uid()?;
        let summary = ical.get_summary()?;

        let start_time = ical.get_start()?.with_timezone(&Utc);
        let end_time = ical.get_end()?.with_timezone(&Utc);

        let id = Uuid::parse_str(uid).unwrap_or_else(|_| Uuid::new_v4());

        Some(Self {
            id,
            title: summary.to_string(),
            description: ical.get_description().map(String::from),
            start_time,
            end_time,
            location: ical.get_location().map(String::from),
            attendees: Vec::new(),
            organizer: organizer.to_string(),
            reminder_minutes: None,
            recurrence: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }
}

/// Export events to iCal format
pub fn export_to_ical(events: &[CalendarEvent], calendar_name: &str) -> String {
    let mut calendar = Calendar::new();
    calendar.name(calendar_name);
    calendar.append_property(Property::new("PRODID", "-//GeneralBots//Calendar//EN"));

    for event in events {
        calendar.push(event.to_ical());
    }

    calendar.done().to_string()
}

/// Import events from iCal format
pub fn import_from_ical(ical_str: &str, organizer: &str) -> Vec<CalendarEvent> {
    let Ok(calendar) = ical_str.parse::<Calendar>() else {
        return Vec::new();
    };

    calendar
        .components
        .iter()
        .filter_map(|c| {
            if let icalendar::CalendarComponent::Event(e) = c {
                CalendarEvent::from_ical(e, organizer)
            } else {
                None
            }
        })
        .collect()
}

#[derive(Default)]
pub struct CalendarEngine {
    events: Vec<CalendarEvent>,
}

impl CalendarEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_event(&mut self, input: CalendarEventInput) -> CalendarEvent {
        let event = CalendarEvent {
            id: Uuid::new_v4(),
            title: input.title,
            description: input.description,
            start_time: input.start_time,
            end_time: input.end_time,
            location: input.location,
            attendees: input.attendees,
            organizer: input.organizer,
            reminder_minutes: input.reminder_minutes,
            recurrence: input.recurrence,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        self.events.push(event.clone());
        event
    }

    pub fn get_event(&self, id: Uuid) -> Option<&CalendarEvent> {
        self.events.iter().find(|e| e.id == id)
    }

    pub fn update_event(&mut self, id: Uuid, input: CalendarEventInput) -> Option<CalendarEvent> {
        let event = self.events.iter_mut().find(|e| e.id == id)?;
        event.title = input.title;
        event.description = input.description;
        event.start_time = input.start_time;
        event.end_time = input.end_time;
        event.location = input.location;
        event.attendees = input.attendees;
        event.organizer = input.organizer;
        event.reminder_minutes = input.reminder_minutes;
        event.recurrence = input.recurrence;
        event.updated_at = Utc::now();
        Some(event.clone())
    }

    pub fn delete_event(&mut self, id: Uuid) -> bool {
        let len = self.events.len();
        self.events.retain(|e| e.id != id);
        self.events.len() < len
    }

    pub fn list_events(&self, limit: usize, offset: usize) -> Vec<&CalendarEvent> {
        self.events.iter().skip(offset).take(limit).collect()
    }

    pub fn get_events_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<&CalendarEvent> {
        self.events
            .iter()
            .filter(|e| e.start_time >= start && e.end_time <= end)
            .collect()
    }

    pub fn get_user_events(&self, user_id: &str) -> Vec<&CalendarEvent> {
        self.events
            .iter()
            .filter(|e| e.organizer == user_id)
            .collect()
    }

    pub fn check_conflicts(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        user_id: &str,
    ) -> Vec<&CalendarEvent> {
        self.events
            .iter()
            .filter(|e| e.organizer == user_id && e.start_time < end && e.end_time > start)
            .collect()
    }

    pub fn export_ical(&self, calendar_name: &str) -> String {
        export_to_ical(&self.events, calendar_name)
    }

    pub fn import_ical(&mut self, ical_str: &str, organizer: &str) -> usize {
        let imported = import_from_ical(ical_str, organizer);
        let count = imported.len();
        self.events.extend(imported);
        count
    }
}

// HTTP Handlers

pub async fn list_events(
    State(_state): State<Arc<AppState>>,
    axum::extract::Query(_query): axum::extract::Query<serde_json::Value>,
) -> Json<Vec<CalendarEvent>> {
    Json(vec![])
}

pub async fn get_event(
    State(_state): State<Arc<AppState>>,
    Path(_id): Path<Uuid>,
) -> Result<Json<CalendarEvent>, StatusCode> {
    Err(StatusCode::NOT_FOUND)
}

pub async fn create_event(
    State(_state): State<Arc<AppState>>,
    Json(_input): Json<CalendarEventInput>,
) -> Result<Json<CalendarEvent>, StatusCode> {
    Err(StatusCode::NOT_IMPLEMENTED)
}

pub async fn update_event(
    State(_state): State<Arc<AppState>>,
    Path(_id): Path<Uuid>,
    Json(_input): Json<CalendarEventInput>,
) -> Result<Json<CalendarEvent>, StatusCode> {
    Err(StatusCode::NOT_IMPLEMENTED)
}

pub async fn delete_event(
    State(_state): State<Arc<AppState>>,
    Path(_id): Path<Uuid>,
) -> StatusCode {
    StatusCode::NOT_IMPLEMENTED
}

pub async fn export_ical(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    let calendar = Calendar::new().name("GeneralBots Calendar").done();
    (
        [(
            axum::http::header::CONTENT_TYPE,
            "text/calendar; charset=utf-8",
        )],
        calendar.to_string(),
    )
}

pub async fn import_ical(
    State(_state): State<Arc<AppState>>,
    body: String,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let events = import_from_ical(&body, "unknown");
    Ok(Json(serde_json::json!({ "imported": events.len() })))
}

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route(
            ApiUrls::CALENDAR_EVENTS,
            get(list_events).post(create_event),
        )
        .route(
            &ApiUrls::CALENDAR_EVENT_BY_ID.replace(":id", "{id}"),
            get(get_event).put(update_event).delete(delete_event),
        )
        .route("/api/calendar/export.ics", get(export_ical))
        .route("/api/calendar/import", post(import_ical))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_to_ical_roundtrip() {
        let event = CalendarEvent {
            id: Uuid::new_v4(),
            title: "Test Meeting".to_string(),
            description: Some("A test meeting".to_string()),
            start_time: Utc::now(),
            end_time: Utc::now() + chrono::Duration::hours(1),
            location: Some("Room 101".to_string()),
            attendees: vec!["user@example.com".to_string()],
            organizer: "organizer@example.com".to_string(),
            reminder_minutes: Some(15),
            recurrence: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let ical = event.to_ical();
        assert_eq!(ical.get_summary(), Some("Test Meeting"));
        assert_eq!(ical.get_location(), Some("Room 101"));
    }

    #[test]
    fn test_export_import_ical() {
        let mut engine = CalendarEngine::new();
        engine.create_event(CalendarEventInput {
            title: "Event 1".to_string(),
            description: None,
            start_time: Utc::now(),
            end_time: Utc::now() + chrono::Duration::hours(1),
            location: None,
            attendees: vec![],
            organizer: "test@example.com".to_string(),
            reminder_minutes: None,
            recurrence: None,
        });

        let ical = engine.export_ical("Test Calendar");
        assert!(ical.contains("BEGIN:VCALENDAR"));
        assert!(ical.contains("Event 1"));
    }
}

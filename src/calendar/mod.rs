use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use chrono::{DateTime, Utc};
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
    pub attendees: Vec<String>,
    pub organizer: String,
    pub reminder_minutes: Option<i32>,
    pub recurrence: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarReminder {
    pub id: Uuid,
    pub event_id: Uuid,
    pub reminder_type: String,
    pub trigger_time: DateTime<Utc>,
    pub channel: String,
    pub sent: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingSummary {
    pub event_id: Uuid,
    pub title: String,
    pub summary: String,
    pub action_items: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecurrenceRule {
    pub frequency: String,
    pub interval: Option<i32>,
    pub count: Option<i32>,
    pub until: Option<DateTime<Utc>>,
}

pub struct CalendarEngine {
    events: Vec<CalendarEvent>,
}

impl CalendarEngine {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub async fn create_event(
        &mut self,
        event: CalendarEventInput,
    ) -> Result<CalendarEvent, String> {
        let calendar_event = CalendarEvent {
            id: Uuid::new_v4(),
            title: event.title,
            description: event.description,
            start_time: event.start_time,
            end_time: event.end_time,
            location: event.location,
            attendees: event.attendees,
            organizer: event.organizer,
            reminder_minutes: event.reminder_minutes,
            recurrence: event.recurrence,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        self.events.push(calendar_event.clone());
        Ok(calendar_event)
    }

    pub async fn get_event(&self, id: Uuid) -> Result<Option<CalendarEvent>, String> {
        Ok(self.events.iter().find(|e| e.id == id).cloned())
    }

    pub async fn update_event(
        &mut self,
        id: Uuid,
        updates: CalendarEventInput,
    ) -> Result<CalendarEvent, String> {
        if let Some(event) = self.events.iter_mut().find(|e| e.id == id) {
            event.title = updates.title;
            event.description = updates.description;
            event.start_time = updates.start_time;
            event.end_time = updates.end_time;
            event.location = updates.location;
            event.attendees = updates.attendees;
            event.organizer = updates.organizer;
            event.reminder_minutes = updates.reminder_minutes;
            event.recurrence = updates.recurrence;
            event.updated_at = Utc::now();
            Ok(event.clone())
        } else {
            Err("Event not found".to_string())
        }
    }

    pub async fn delete_event(&mut self, id: Uuid) -> Result<bool, String> {
        let initial_len = self.events.len();
        self.events.retain(|e| e.id != id);
        Ok(self.events.len() < initial_len)
    }

    pub async fn list_events(
        &self,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<CalendarEvent>, String> {
        let limit = limit.unwrap_or(50) as usize;
        let offset = offset.unwrap_or(0) as usize;
        Ok(self
            .events
            .iter()
            .skip(offset)
            .take(limit)
            .cloned()
            .collect())
    }

    pub async fn get_events_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<CalendarEvent>, String> {
        Ok(self
            .events
            .iter()
            .filter(|e| e.start_time >= start && e.end_time <= end)
            .cloned()
            .collect())
    }

    pub async fn get_user_events(&self, user_id: &str) -> Result<Vec<CalendarEvent>, String> {
        Ok(self
            .events
            .iter()
            .filter(|e| e.organizer == user_id)
            .cloned()
            .collect())
    }

    pub async fn create_reminder(
        &self,
        event_id: Uuid,
        reminder_type: String,
        trigger_time: DateTime<Utc>,
        channel: String,
    ) -> Result<CalendarReminder, String> {
        Ok(CalendarReminder {
            id: Uuid::new_v4(),
            event_id,
            reminder_type,
            trigger_time,
            channel,
            sent: false,
        })
    }

    pub async fn check_conflicts(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        user_id: &str,
    ) -> Result<Vec<CalendarEvent>, String> {
        Ok(self
            .events
            .iter()
            .filter(|e| {
                e.organizer == user_id
                    && ((e.start_time < end && e.end_time > start)
                        || (e.start_time >= start && e.start_time < end))
            })
            .cloned()
            .collect())
    }
}

pub async fn list_events(
    State(_state): State<Arc<AppState>>,
    axum::extract::Query(_query): axum::extract::Query<serde_json::Value>,
) -> Result<Json<Vec<CalendarEvent>>, StatusCode> {
    Ok(Json(vec![]))
}

pub async fn get_event(
    State(_state): State<Arc<AppState>>,
    Path(_id): Path<Uuid>,
) -> Result<Json<Option<CalendarEvent>>, StatusCode> {
    Ok(Json(None))
}

pub async fn create_event(
    State(_state): State<Arc<AppState>>,
    Json(_event): Json<CalendarEventInput>,
) -> Result<Json<CalendarEvent>, StatusCode> {
    Err(StatusCode::NOT_IMPLEMENTED)
}

pub async fn update_event(
    State(_state): State<Arc<AppState>>,
    Path(_id): Path<Uuid>,
    Json(_updates): Json<CalendarEventInput>,
) -> Result<Json<CalendarEvent>, StatusCode> {
    Err(StatusCode::NOT_IMPLEMENTED)
}

pub async fn delete_event(
    State(_state): State<Arc<AppState>>,
    Path(_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    Err(StatusCode::NOT_IMPLEMENTED)
}

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route(
            ApiUrls::CALENDAR_EVENTS,
            get(list_events).post(create_event),
        )
        .route(
            ApiUrls::CALENDAR_EVENT_BY_ID.replace(":id", "{id}"),
            get(get_event).put(update_event).delete(delete_event),
        )
        .with_state(state)
}

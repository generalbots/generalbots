use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::shared::utils::DbPool;
use tokio::sync::RwLock;
use uuid::Uuid;
use crate::shared::state::AppState;
use diesel::sql_query;
use diesel::sql_types::{Text, Timestamptz, Integer, Jsonb};

#[derive(Debug, Clone, Serialize, Deserialize, QueryableByName)]
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
    pub recurrence_rule: Option<String>,
    pub status: EventStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EventStatus {
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meeting {
    pub id: Uuid,
    pub event_id: Uuid,
    pub meeting_url: Option<String>,
    pub meeting_id: Option<String>,
    pub platform: MeetingPlatform,
    pub recording_url: Option<String>,
    pub notes: Option<String>,
    pub action_items: Vec<ActionItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MeetingPlatform {
    Zoom,
    Teams,
    Meet,
    Internal,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionItem {
    pub id: Uuid,
    pub description: String,
    pub assignee: String,
    pub due_date: Option<DateTime<Utc>>,
    pub completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarReminder {
    pub id: Uuid,
    pub event_id: Uuid,
    pub remind_at: DateTime<Utc>,
    pub message: String,
    pub channel: ReminderChannel,
    pub sent: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReminderChannel {
    Email,
    Sms,
    Push,
    InApp,
}

// API Request/Response structs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEventRequest {
    pub title: String,
    pub description: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub location: Option<String>,
    pub attendees: Option<Vec<String>>,
    pub organizer: String,
    pub reminder_minutes: Option<i32>,
    pub recurrence_rule: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateEventRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub location: Option<String>,
    pub status: Option<EventStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleMeetingRequest {
    pub title: String,
    pub description: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub location: Option<String>,
    pub attendees: Vec<String>,
    pub organizer: String,
    pub reminder_minutes: Option<i32>,
    pub meeting_url: Option<String>,
    pub meeting_id: Option<String>,
    pub platform: Option<MeetingPlatform>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetReminderRequest {
    pub event_id: Uuid,
    pub remind_at: DateTime<Utc>,
    pub message: String,
    pub channel: ReminderChannel,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventListQuery {
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventSearchQuery {
    pub query: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckAvailabilityQuery {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
}

#[derive(Clone)]
pub struct CalendarEngine {
    db: Arc<DbPool>,
    cache: Arc<RwLock<Vec<CalendarEvent>>>,
}

impl CalendarEngine {
    pub fn new(db: Arc<PgPool>) -> Self {
        Self {
            db,
            cache: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn create_event(
        &self,
        event: CalendarEvent,
    ) -> Result<CalendarEvent, Box<dyn std::error::Error>> {
        let mut conn = self.db.get().map_err(|e| format!("DB connection error: {}", e))?;

        let attendees_json = serde_json::to_value(&event.attendees)?;
        let recurrence_json = event.recurrence_rule.as_ref().map(|r| serde_json::to_value(r).ok()).flatten();

        diesel::sql_query(
            "INSERT INTO calendar_events
             (id, title, description, start_time, end_time, location, attendees, organizer,
              reminder_minutes, recurrence_rule, status, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
             RETURNING *"
        )
            event.id,
            event.title,
            event.description,
            event.start_time,
            event.end_time,
            event.location,
            &event.attendees[..],
            event.organizer,
            event.reminder_minutes,
            event.recurrence_rule,
            serde_json::to_value(&event.status)?,
            event.created_at,
            event.updated_at
        )
        .fetch_one(self.db.as_ref())
        .await?;
        */

        self.refresh_cache().await?;

        Ok(event)
        Ok(event)
    }

    pub async fn update_event(
        &self,
        id: Uuid,
        updates: serde_json::Value,
    ) -> Result<CalendarEvent, Box<dyn std::error::Error>> {
        let updated_at = Utc::now();

        let result = sqlx::query!(
            r#"
            UPDATE calendar_events
            SET title = COALESCE($2, title),
                description = COALESCE($3, description),
                start_time = COALESCE($4, start_time),
                end_time = COALESCE($5, end_time),
                location = COALESCE($6, location),
                updated_at = $7
            WHERE id = $1
            RETURNING *
            "#,
            id,
            updates.get("title").and_then(|v| v.as_str()),
            updates.get("description").and_then(|v| v.as_str()),
            updates
                .get("start_time")
                .and_then(|v| DateTime::parse_from_rfc3339(v.as_str()?).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            updates
                .get("end_time")
                .and_then(|v| DateTime::parse_from_rfc3339(v.as_str()?).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            updates.get("location").and_then(|v| v.as_str()),
            updated_at
        )
        .fetch_one(self.db.as_ref())
        .await?;

        self.refresh_cache().await?;

        Ok(serde_json::from_value(serde_json::to_value(result)?)?)
    }

    pub async fn delete_event(&self, id: Uuid) -> Result<bool, Box<dyn std::error::Error>> {
        let mut conn = self.db.get().map_err(|e| format!("DB connection error: {}", e))?;

        let rows_affected = diesel::sql_query("DELETE FROM calendar_events WHERE id = $1")
            .bind::<diesel::sql_types::Uuid, _>(&id)
            .execute(&mut conn)?;

        self.refresh_cache().await?;

        Ok(rows_affected > 0)
    }

    pub async fn get_events_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<CalendarEvent>, Box<dyn std::error::Error>> {
        let mut conn = self.db.get().map_err(|e| format!("DB connection error: {}", e))?;

        let results = diesel::sql_query(
            "SELECT * FROM calendar_events
             WHERE start_time >= $1 AND end_time <= $2
             ORDER BY start_time ASC"
        )
        .bind::<Timestamptz, _>(&start)
            end
        )
        .fetch_all(self.db.as_ref())
        .await?;
        */

        Ok(vec![])
    }

    pub async fn get_user_events(
        &self,
        user_id: &str,
    ) -> Result<Vec<CalendarEvent>, Box<dyn std::error::Error>> {
        let mut conn = self.db.get().map_err(|e| format!("DB connection error: {}", e))?;

        let results = diesel::sql_query(
            "SELECT * FROM calendar_events
             WHERE organizer = $1 OR $1::text = ANY(SELECT jsonb_array_elements_text(attendees))
             ORDER BY start_time ASC"
        )
        .bind::<Text, _>(&user_id)
        .fetch_all(self.db.as_ref())
        .await?;

        Ok(results
            .into_iter()
            .map(|r| serde_json::from_value(serde_json::to_value(r).unwrap()).unwrap())
            .collect())
        */
        Ok(vec![])
    }

    pub async fn create_meeting(
        &self,
        event_id: Uuid,
        platform: MeetingPlatform,
    ) -> Result<Meeting, Box<dyn std::error::Error>> {
        let meeting = Meeting {
            id: Uuid::new_v4(),
            event_id,
            meeting_url: None,
            meeting_id: None,
            platform,
            recording_url: None,
            notes: None,
            action_items: Vec::new(),
        };

        let mut conn = self.db.get().map_err(|e| format!("DB connection error: {}", e))?;

        diesel::sql_query(
            r#"
            INSERT INTO meetings (id, event_id, platform, created_at)
            VALUES ($1, $2, $3, $4)
            "#,
            meeting.id,
            meeting.event_id,
            meeting.platform,
            meeting.created_at
        )
        .execute(self.db.as_ref())
        .await?;
        */

        Ok(meeting)
    }

    pub async fn schedule_reminder(
        &self,
        event_id: Uuid,
        minutes_before: i32,
        channel: ReminderChannel,
    ) -> Result<CalendarReminder, Box<dyn std::error::Error>> {
        let event = self.get_event(event_id).await?;
        let remind_at = event.start_time - chrono::Duration::minutes(minutes_before as i64);

        let reminder = CalendarReminder {
            id: Uuid::new_v4(),
            event_id,
            remind_at,
            message: format!(
                "Reminder: {} starts in {} minutes",
                event.title, minutes_before
            ),
            channel,
            sent: false,
        };

        let mut conn = self.db.get().map_err(|e| format!("DB connection error: {}", e))?;

        diesel::sql_query(
            r#"
            INSERT INTO calendar_reminders (id, event_id, remind_at, message, channel, sent)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            reminder.id,
            reminder.event_id,
            reminder.remind_at,
            reminder.message,
            reminder.channel,
            reminder.sent
        )
        .execute(self.db.as_ref())
        .await?;
        */

        Ok(reminder)
    }

    pub async fn get_event(&self, id: Uuid) -> Result<CalendarEvent, Box<dyn std::error::Error>> {
        let mut conn = self.db.get().map_err(|e| format!("DB connection error: {}", e))?;

        let result = diesel::sql_query("SELECT * FROM calendar_events WHERE id = $1")
            .bind::<diesel::sql_types::Uuid, _>(&id)
            .get_result::<CalendarEvent>(&mut conn)?;

        Ok(result)
    }

    pub async fn check_conflicts(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        user_id: &str,
    ) -> Result<Vec<CalendarEvent>, Box<dyn std::error::Error>> {
        let mut conn = self.db.get().map_err(|e| format!("DB connection error: {}", e))?;

        let results = diesel::sql_query(
            "SELECT * FROM calendar_events
             WHERE (organizer = $1 OR $1::text = ANY(SELECT jsonb_array_elements_text(attendees)))
             AND NOT (end_time <= $2 OR start_time >= $3)"
        )
        .bind::<Text, _>(&user_id)
        .bind::<Timestamptz, _>(&start)
            end
        )
        .fetch_all(self.db.as_ref())
        .await?;

        Ok(results
            .into_iter()
            .map(|r| serde_json::from_value(serde_json::to_value(r).unwrap()).unwrap())
            .collect())
        */
        Ok(vec![])
    }
    pub async fn create_event(&self, event: CreateEventRequest) -> Result<CalendarEvent, Box<dyn std::error::Error>> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        let calendar_event = CalendarEvent {
            id,
            title: event.title,
            description: event.description,
            start_time: event.start_time,
            end_time: event.end_time,
            location: event.location,
            attendees: event.attendees.unwrap_or_default(),
            organizer: event.organizer,
            reminder_minutes: event.reminder_minutes,
            recurrence_rule: event.recurrence_rule,
            status: EventStatus::Scheduled,
            created_at: now,
            updated_at: now,
        };

        // Store in cache
        self.cache.write().await.push(calendar_event.clone());

        Ok(calendar_event)
    }

    pub async fn update_event(&self, id: Uuid, update: UpdateEventRequest) -> Result<CalendarEvent, Box<dyn std::error::Error>> {
        let mut cache = self.cache.write().await;

        if let Some(event) = cache.iter_mut().find(|e| e.id == id) {
            if let Some(title) = update.title {
                event.title = title;
            }
            if let Some(description) = update.description {
                event.description = Some(description);
            }
            if let Some(start_time) = update.start_time {
                event.start_time = start_time;
            }
            if let Some(end_time) = update.end_time {
                event.end_time = end_time;
            }
            if let Some(location) = update.location {
                event.location = Some(location);
            }
            if let Some(status) = update.status {
                event.status = status;
            }
            event.updated_at = Utc::now();

            Ok(event.clone())
        } else {
            Err("Event not found".into())
        }
    }

    pub async fn delete_event(&self, id: Uuid) -> Result<(), Box<dyn std::error::Error>> {
        let mut cache = self.cache.write().await;
        cache.retain(|e| e.id != id);
        Ok(())
    }

    pub async fn list_events(&self, start_date: Option<DateTime<Utc>>, end_date: Option<DateTime<Utc>>) -> Result<Vec<CalendarEvent>, Box<dyn std::error::Error>> {
        let cache = self.cache.read().await;

        let events: Vec<CalendarEvent> = if let (Some(start), Some(end)) = (start_date, end_date) {
            cache.iter()
                .filter(|e| e.start_time >= start && e.start_time <= end)
                .cloned()
                .collect()
        } else {
            cache.clone()
        };

        Ok(events)
    }

    pub async fn search_events(&self, query: &str) -> Result<Vec<CalendarEvent>, Box<dyn std::error::Error>> {
        let cache = self.cache.read().await;
        let query_lower = query.to_lowercase();

        let events: Vec<CalendarEvent> = cache
            .iter()
            .filter(|e| {
                e.title.to_lowercase().contains(&query_lower) ||
                e.description.as_ref().map_or(false, |d| d.to_lowercase().contains(&query_lower))
            })
            .cloned()
            .collect();

        Ok(events)
    }

    pub async fn check_availability(&self, start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> Result<bool, Box<dyn std::error::Error>> {
        let cache = self.cache.read().await;

        let has_conflict = cache.iter().any(|event| {
            (event.start_time < end_time && event.end_time > start_time) &&
            event.status != EventStatus::Cancelled
        });

        Ok(!has_conflict)
    }

    pub async fn schedule_meeting(&self, meeting: ScheduleMeetingRequest) -> Result<Meeting, Box<dyn std::error::Error>> {
        // First create the calendar event
        let event = self.create_event(CreateEventRequest {
            title: meeting.title.clone(),
            description: meeting.description.clone(),
            start_time: meeting.start_time,
            end_time: meeting.end_time,
            location: meeting.location.clone(),
            attendees: Some(meeting.attendees.clone()),
            organizer: meeting.organizer.clone(),
            reminder_minutes: meeting.reminder_minutes,
            recurrence_rule: None,
        }).await?;

        // Create meeting record
        let meeting_record = Meeting {
            id: Uuid::new_v4(),
            event_id: event.id,
            meeting_url: meeting.meeting_url,
            meeting_id: meeting.meeting_id,
            platform: meeting.platform.unwrap_or(MeetingPlatform::Internal),
            recording_url: None,
            notes: None,
            action_items: vec![],
        };

        Ok(meeting_record)
    }

    pub async fn set_reminder(&self, reminder: SetReminderRequest) -> Result<CalendarReminder, Box<dyn std::error::Error>> {
        let reminder_record = CalendarReminder {
            id: Uuid::new_v4(),
            event_id: reminder.event_id,
            remind_at: reminder.remind_at,
            message: reminder.message,
            channel: reminder.channel,
            sent: false,
        };

        Ok(reminder_record)
    }

    async fn refresh_cache(&self) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: Implement with Diesel
        /*
        let results = sqlx::query!("SELECT * FROM calendar_events ORDER BY start_time ASC")
            .load::<CalendarEvent>(&mut conn)?;
        let events: Vec<CalendarEvent> = vec![];
        let mut cache = self.cache.write().await;
        *cache = events;

        Ok(())
    }
}

// Calendar API handlers
pub async fn handle_event_create(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateEventRequest>,
) -> Result<Json<CalendarEvent>, StatusCode> {
    let calendar = state.calendar_engine.as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    match calendar.create_event(payload).await {
        Ok(event) => Ok(Json(event)),
        Err(e) => {
            log::error!("Failed to create event: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn handle_event_update(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateEventRequest>,
) -> Result<Json<CalendarEvent>, StatusCode> {
    let calendar = state.calendar_engine.as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    match calendar.update_event(id, payload).await {
        Ok(event) => Ok(Json(event)),
        Err(e) => {
            log::error!("Failed to update event: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn handle_event_delete(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let calendar = state.calendar_engine.as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    match calendar.delete_event(id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            log::error!("Failed to delete event: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn handle_events_list(
    State(state): State<Arc<AppState>>,
    Query(query): Query<EventListQuery>,
) -> Result<Json<Vec<CalendarEvent>>, StatusCode> {
    let calendar = state.calendar_engine.as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    match calendar.list_events(query.start_date, query.end_date).await {
        Ok(events) => Ok(Json(events)),
        Err(e) => {
            log::error!("Failed to list events: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn handle_events_search(
    State(state): State<Arc<AppState>>,
    Query(query): Query<EventSearchQuery>,
) -> Result<Json<Vec<CalendarEvent>>, StatusCode> {
    let calendar = state.calendar_engine.as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    match calendar.search_events(&query.query).await {
        Ok(events) => Ok(Json(events)),
        Err(e) => {
            log::error!("Failed to search events: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn handle_check_availability(
    State(state): State<Arc<AppState>>,
    Query(query): Query<CheckAvailabilityQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let calendar = state.calendar_engine.as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    match calendar.check_availability(query.start_time, query.end_time).await {
        Ok(available) => Ok(Json(serde_json::json!({ "available": available }))),
        Err(e) => {
            log::error!("Failed to check availability: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn handle_schedule_meeting(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ScheduleMeetingRequest>,
) -> Result<Json<Meeting>, StatusCode> {
    let calendar = state.calendar_engine.as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    match calendar.schedule_meeting(payload).await {
        Ok(meeting) => Ok(Json(meeting)),
        Err(e) => {
            log::error!("Failed to schedule meeting: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn handle_set_reminder(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SetReminderRequest>,
) -> Result<Json<CalendarReminder>, StatusCode> {
    let calendar = state.calendar_engine.as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    match calendar.set_reminder(payload).await {
        Ok(reminder) => Ok(Json(reminder)),
        Err(e) => {
            log::error!("Failed to set reminder: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Configure calendar routes
pub fn configure_calendar_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/calendar/events", post(handle_event_create))
        .route("/api/calendar/events", get(handle_events_list))
        .route("/api/calendar/events/:id", put(handle_event_update))
        .route("/api/calendar/events/:id", delete(handle_event_delete))
        .route("/api/calendar/events/search", get(handle_events_search))
        .route("/api/calendar/availability", get(handle_check_availability))
        .route("/api/calendar/meetings", post(handle_schedule_meeting))
        .route("/api/calendar/reminders", post(handle_set_reminder))
}

#[derive(Deserialize)]
pub struct EventQuery {
    pub start: Option<String>,
    pub end: Option<String>,
    pub user_id: Option<String>,
}

#[derive(Deserialize)]
pub struct MeetingRequest {
    pub event_id: Uuid,
    pub platform: MeetingPlatform,

    /// Process due reminders
    pub async fn process_reminders(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let now = Utc::now();
        let mut conn = self.db.get().map_err(|e| format!("DB connection error: {}", e))?;

        // Find events that need reminders sent
        let events = diesel::sql_query(
            "SELECT * FROM calendar_events
             WHERE reminder_minutes IS NOT NULL
             AND start_time - INTERVAL '1 minute' * reminder_minutes <= $1
             AND start_time > $1
             AND reminder_sent = false
             ORDER BY start_time ASC"
        )
        .bind::<Timestamptz, _>(&now)
        .load::<CalendarEvent>(&mut conn)?;

        let mut notifications = Vec::new();

        for event in events {
            // Send reminder notification
            let message = format!(
                "Reminder: {} starting at {}",
                event.title,
                event.start_time.format("%H:%M")
            );

            // Mark reminder as sent
            diesel::sql_query(
                "UPDATE calendar_events SET reminder_sent = true WHERE id = $1"
            )
            .bind::<diesel::sql_types::Uuid, _>(&event.id)
            .execute(&mut conn)?;

            notifications.push(message);
        }

        Ok(notifications)
    }
}

/// CalDAV Server implementation
pub mod caldav {
    use super::*;
    use axum::{
        body::Body,
        extract::{Path, State, Query},
        http::{Method, StatusCode, header},
        response::{Response, IntoResponse},
        routing::{get, put, delete, any},
        Router,
    };
    use std::sync::Arc;

    pub fn create_caldav_router(calendar_engine: Arc<CalendarEngine>) -> Router {
        Router::new()
            .route("/.well-known/caldav", get(caldav_redirect))
            .route("/caldav/:user/", any(caldav_propfind))
            .route("/caldav/:user/calendar/", any(caldav_calendar_handler))
            .route("/caldav/:user/calendar/:event_uid.ics",
                get(caldav_get_event)
                .put(caldav_put_event)
                .delete(caldav_delete_event))
            .with_state(calendar_engine)
    }

    async fn caldav_redirect() -> impl IntoResponse {
        Response::builder()
            .status(StatusCode::MOVED_PERMANENTLY)
            .header(header::LOCATION, "/caldav/")
            .body(Body::empty())
            .unwrap()
    }

    async fn caldav_propfind(
        Path(user): Path<String>,
        State(engine): State<Arc<CalendarEngine>>,
    ) -> impl IntoResponse {
        let xml = format!(r#"<?xml version="1.0" encoding="utf-8"?>
<D:multistatus xmlns:D="DAV:" xmlns:C="urn:ietf:params:xml:ns:caldav">
  <D:response>
    <D:href>/caldav/{}/</D:href>
    <D:propstat>
      <D:prop>
        <D:resourcetype>
          <D:collection/>
          <C:calendar/>
        </D:resourcetype>
        <D:displayname>{}'s Calendar</D:displayname>
        <C:supported-calendar-component-set>
          <C:comp name="VEVENT"/>
        </C:supported-calendar-component-set>
      </D:prop>
      <D:status>HTTP/1.1 200 OK</D:status>
    </D:propstat>
  </D:response>
</D:multistatus>"#, user, user);

        Response::builder()
            .status(StatusCode::MULTI_STATUS)
            .header(header::CONTENT_TYPE, "application/xml; charset=utf-8")
            .body(Body::from(xml))
            .unwrap()
    }

    async fn caldav_calendar_handler(
        Path(user): Path<String>,
        State(engine): State<Arc<CalendarEngine>>,
        method: Method,
    ) -> impl IntoResponse {
        match method {
            Method::GET => {
                // Return calendar collection
                let events = engine.get_user_events(&user).await.unwrap_or_default();
                let ics = events_to_icalendar(&events, &user);

                Response::builder()
                    .status(StatusCode::OK)
                    .header(header::CONTENT_TYPE, "text/calendar; charset=utf-8")
                    .body(Body::from(ics))
                    .unwrap()
            },
            _ => caldav_propfind(Path(user), State(engine)).await.into_response(),
        }
    }

    async fn caldav_get_event(
        Path((user, event_uid)): Path<(String, String)>,
        State(engine): State<Arc<CalendarEngine>>,
    ) -> impl IntoResponse {
        let event_id = event_uid.trim_end_matches(".ics");

        match Uuid::parse_str(event_id) {
            Ok(id) => {
                match engine.get_event(id).await {
                    Ok(event) => {
                        let ics = event_to_icalendar(&event);
                        Response::builder()
                            .status(StatusCode::OK)
                            .header(header::CONTENT_TYPE, "text/calendar; charset=utf-8")
                            .body(Body::from(ics))
                            .unwrap()
                    },
                    Err(_) => Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .body(Body::empty())
                        .unwrap(),
                }
            },
            Err(_) => Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::empty())
                .unwrap(),
        }
    }

    async fn caldav_put_event(
        Path((user, event_uid)): Path<(String, String)>,
        State(engine): State<Arc<CalendarEngine>>,
        body: String,
    ) -> impl IntoResponse {
        // Parse iCalendar data and create/update event
        // This is a simplified implementation
        StatusCode::CREATED
    }

    async fn caldav_delete_event(
        Path((user, event_uid)): Path<(String, String)>,
        State(engine): State<Arc<CalendarEngine>>,
    ) -> impl IntoResponse {
        let event_id = event_uid.trim_end_matches(".ics");

        match Uuid::parse_str(event_id) {
            Ok(id) => {
                match engine.delete_event(id).await {
                    Ok(true) => StatusCode::NO_CONTENT,
                    Ok(false) => StatusCode::NOT_FOUND,
                    Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
                }
            },
            Err(_) => StatusCode::BAD_REQUEST,
        }
    }

    fn events_to_icalendar(events: &[CalendarEvent], user: &str) -> String {
        let mut ics = String::from("BEGIN:VCALENDAR\r\n");
        ics.push_str("VERSION:2.0\r\n");
        ics.push_str(&format!("PRODID:-//BotServer//Calendar {}//EN\r\n", user));

        for event in events {
            ics.push_str(&event_to_icalendar(event));
        }

        ics.push_str("END:VCALENDAR\r\n");
        ics
    }

    fn event_to_icalendar(event: &CalendarEvent) -> String {
        let mut vevent = String::from("BEGIN:VEVENT\r\n");
        vevent.push_str(&format!("UID:{}\r\n", event.id));
        vevent.push_str(&format!("SUMMARY:{}\r\n", event.title));

        if let Some(desc) = &event.description {
            vevent.push_str(&format!("DESCRIPTION:{}\r\n", desc));
        }

        if let Some(loc) = &event.location {
            vevent.push_str(&format!("LOCATION:{}\r\n", loc));
        }

        vevent.push_str(&format!("DTSTART:{}\r\n", event.start_time.format("%Y%m%dT%H%M%SZ")));
        vevent.push_str(&format!("DTEND:{}\r\n", event.end_time.format("%Y%m%dT%H%M%SZ")));
        vevent.push_str(&format!("STATUS:{}\r\n", event.status.to_uppercase()));

        for attendee in &event.attendees {
            vevent.push_str(&format!("ATTENDEE:mailto:{}\r\n", attendee));
        }

        vevent.push_str("END:VEVENT\r\n");
        vevent
    }
}

/// Reminder job service
pub async fn start_reminder_job(engine: Arc<CalendarEngine>) {
    use tokio::time::{interval, Duration};

    let mut ticker = interval(Duration::from_secs(60)); // Check every minute

    loop {
        ticker.tick().await;

        match engine.process_reminders().await {
            Ok(notifications) => {
                for message in notifications {
                    log::info!("Calendar reminder: {}", message);
                    // Here you would send actual notifications via email, push, etc.
                }
            },
            Err(e) => {
                log::error!("Failed to process calendar reminders: {}", e);
            }
        }
    }
}


async fn create_event_handler(
    State(engine): State<Arc<CalendarEngine>>,
    Json(event): Json<CalendarEvent>,
) -> Result<Json<CalendarEvent>, StatusCode> {
    match engine.create_event(event).await {
        Ok(created) => Ok(Json(created)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_events_handler(
    State(engine): State<Arc<CalendarEngine>>,
    Query(params): Query<EventQuery>,
) -> Result<Json<Vec<CalendarEvent>>, StatusCode> {
    if let (Some(start), Some(end)) = (params.start, params.end) {
        let start = DateTime::parse_from_rfc3339(&start)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
        let end = DateTime::parse_from_rfc3339(&end)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now() + chrono::Duration::days(30));

        match engine.get_events_range(start, end).await {
            Ok(events) => Ok(Json(events)),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    } else if let Some(user_id) = params.user_id {
        match engine.get_user_events(&user_id).await {
            Ok(events) => Ok(Json(events)),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

async fn update_event_handler(
    State(engine): State<Arc<CalendarEngine>>,
    Path(id): Path<Uuid>,
    Json(updates): Json<serde_json::Value>,
) -> Result<Json<CalendarEvent>, StatusCode> {
    match engine.update_event(id, updates).await {
        Ok(updated) => Ok(Json(updated)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn delete_event_handler(
    State(engine): State<Arc<CalendarEngine>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match engine.delete_event(id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn schedule_meeting_handler(
    State(engine): State<Arc<CalendarEngine>>,
    Json(req): Json<MeetingRequest>,
) -> Result<Json<Meeting>, StatusCode> {
    match engine.create_meeting(req.event_id, req.platform).await {
        Ok(meeting) => Ok(Json(meeting)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub fn routes(engine: Arc<CalendarEngine>) -> Router {
    Router::new()
        .route(
            "/events",
            post(create_event_handler).get(get_events_handler),
        )
        .route(
            "/events/:id",
            put(update_event_handler).delete(delete_event_handler),
        )
        .route("/meetings", post(schedule_meeting_handler))
        .with_state(engine)
}

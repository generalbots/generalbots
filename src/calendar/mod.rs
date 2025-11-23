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

// TODO: Replace sqlx queries with Diesel queries

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
        // TODO: Implement with Diesel
        /*
        let result = sqlx::query!(
            r#"
            INSERT INTO calendar_events
            (id, title, description, start_time, end_time, location, attendees, organizer,
             reminder_minutes, recurrence_rule, status, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING *
            "#,
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

    pub async fn delete_event(&self, _id: Uuid) -> Result<bool, Box<dyn std::error::Error>> {
        // TODO: Implement with Diesel
        /*
        let result = sqlx::query!("DELETE FROM calendar_events WHERE id = $1", id)
            .execute(self.db.as_ref())
            .await?;
        */

        self.refresh_cache().await?;

        Ok(false)
    }

    pub async fn get_events_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<CalendarEvent>, Box<dyn std::error::Error>> {
        // TODO: Implement with Diesel
        /*
        let results = sqlx::query_as!(
            CalendarEvent,
            r#"
            SELECT * FROM calendar_events
            WHERE start_time >= $1 AND end_time <= $2
            ORDER BY start_time ASC
            "#,
            start,
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
        // TODO: Implement with Diesel
        /*
        let results = sqlx::query!(
            r#"
            SELECT * FROM calendar_events
            WHERE organizer = $1 OR $1 = ANY(attendees)
            ORDER BY start_time ASC
            "#,
            user_id
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

        // TODO: Implement with Diesel
        /*
        sqlx::query!(
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

        // TODO: Implement with Diesel
        /*
        sqlx::query!(
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

    pub async fn get_event(&self, _id: Uuid) -> Result<CalendarEvent, Box<dyn std::error::Error>> {
        // TODO: Implement with Diesel
        /*
        let result = sqlx::query!("SELECT * FROM calendar_events WHERE id = $1", id)
            .fetch_one(self.db.as_ref())
            .await?;

        Ok(serde_json::from_value(serde_json::to_value(result)?)?)
        */
        Err("Not implemented".into())
    }

    pub async fn check_conflicts(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        user_id: &str,
    ) -> Result<Vec<CalendarEvent>, Box<dyn std::error::Error>> {
        // TODO: Implement with Diesel
        /*
        let results = sqlx::query!(
            r#"
            SELECT * FROM calendar_events
            WHERE (organizer = $1 OR $1 = ANY(attendees))
            AND NOT (end_time <= $2 OR start_time >= $3)
            "#,
            user_id,
            start,
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

    async fn refresh_cache(&self) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: Implement with Diesel
        /*
        let results = sqlx::query!("SELECT * FROM calendar_events ORDER BY start_time ASC")
            .fetch_all(self.db.as_ref())
            .await?;

        let events: Vec<CalendarEvent> = results
            .into_iter()
            .map(|r| serde_json::from_value(serde_json::to_value(r).unwrap()).unwrap())
            .collect();
        */

        let events: Vec<CalendarEvent> = vec![];
        let mut cache = self.cache.write().await;
        *cache = events;

        Ok(())
    }
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

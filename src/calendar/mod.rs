use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::{get, post},
    Router,
};
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use diesel::prelude::*;
use icalendar::{
    Calendar, CalendarDateTime, Component, DatePerhapsTime, Event as IcalEvent, EventLike, Property,
};
use log::info;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::core::shared::schema::{calendar_event_attendees, calendar_events, calendar_shares, calendars};
use crate::core::urls::ApiUrls;
use crate::shared::state::AppState;

pub mod caldav;
pub mod ui;

fn get_bot_context() -> (Uuid, Uuid) {
    let org_id = std::env::var("DEFAULT_ORG_ID")
        .ok()
        .and_then(|s| Uuid::parse_str(&s).ok())
        .unwrap_or_else(Uuid::nil);
    let bot_id = std::env::var("DEFAULT_BOT_ID")
        .ok()
        .and_then(|s| Uuid::parse_str(&s).ok())
        .unwrap_or_else(Uuid::nil);
    (org_id, bot_id)
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = calendars)]
pub struct CalendarRecord {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub owner_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub timezone: Option<String>,
    pub is_primary: bool,
    pub is_visible: bool,
    pub is_shared: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = calendar_events)]
pub struct CalendarEventRecord {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub calendar_id: Uuid,
    pub owner_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub all_day: bool,
    pub recurrence_rule: Option<String>,
    pub recurrence_id: Option<Uuid>,
    pub color: Option<String>,
    pub status: String,
    pub visibility: String,
    pub busy_status: String,
    pub reminders: serde_json::Value,
    pub attendees: serde_json::Value,
    pub conference_data: Option<serde_json::Value>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable)]
#[diesel(table_name = calendar_event_attendees)]
pub struct EventAttendeeRecord {
    pub id: Uuid,
    pub event_id: Uuid,
    pub email: String,
    pub name: Option<String>,
    pub status: String,
    pub role: String,
    pub rsvp_time: Option<DateTime<Utc>>,
    pub comment: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable)]
#[diesel(table_name = calendar_shares)]
pub struct CalendarShareRecord {
    pub id: Uuid,
    pub calendar_id: Uuid,
    pub shared_with_user_id: Option<Uuid>,
    pub shared_with_email: Option<String>,
    pub permission: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub id: Uuid,
    pub calendar_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub location: Option<String>,
    pub attendees: Vec<String>,
    pub organizer: String,
    pub reminder_minutes: Option<i32>,
    pub recurrence: Option<String>,
    pub all_day: bool,
    pub status: String,
    pub color: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarEventInput {
    pub calendar_id: Option<Uuid>,
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
    #[serde(default)]
    pub all_day: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCalendarRequest {
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub timezone: Option<String>,
    #[serde(default)]
    pub is_primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCalendarRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub color: Option<String>,
    pub timezone: Option<String>,
    pub is_visible: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EventQuery {
    pub calendar_id: Option<Uuid>,
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareCalendarRequest {
    pub user_id: Option<Uuid>,
    pub email: Option<String>,
    pub permission: String,
}

impl CalendarEvent {
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

        event.add_property("ORGANIZER", format!("mailto:{}", self.organizer));

        for attendee in &self.attendees {
            event.add_property("ATTENDEE", format!("mailto:{attendee}"));
        }

        if let Some(ref rrule) = self.recurrence {
            event.add_property("RRULE", rrule);
        }

        if let Some(minutes) = self.reminder_minutes {
            event.add_property("VALARM", format!("-PT{minutes}M"));
        }

        event.done()
    }

    pub fn from_ical(ical: &IcalEvent, organizer: &str, calendar_id: Uuid) -> Option<Self> {
        let uid = ical.get_uid()?;
        let summary = ical.get_summary()?;

        let start_time = date_perhaps_time_to_utc(ical.get_start()?)?;
        let end_time = date_perhaps_time_to_utc(ical.get_end()?)?;

        let id = Uuid::parse_str(uid).unwrap_or_else(|_| Uuid::new_v4());

        Some(Self {
            id,
            calendar_id,
            title: summary.to_string(),
            description: ical.get_description().map(String::from),
            start_time,
            end_time,
            location: ical.get_location().map(String::from),
            attendees: Vec::new(),
            organizer: organizer.to_string(),
            reminder_minutes: None,
            recurrence: None,
            all_day: false,
            status: "confirmed".to_string(),
            color: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }
}

fn date_perhaps_time_to_utc(dpt: DatePerhapsTime) -> Option<DateTime<Utc>> {
    match dpt {
        DatePerhapsTime::DateTime(cal_dt) => match cal_dt {
            CalendarDateTime::Utc(dt) => Some(dt),
            CalendarDateTime::Floating(naive) => Some(Utc.from_utc_datetime(&naive)),
            CalendarDateTime::WithTimezone { date_time, .. } => {
                Some(Utc.from_utc_datetime(&date_time))
            }
        },
        DatePerhapsTime::Date(date) => {
            let naive = NaiveDateTime::new(date, chrono::NaiveTime::from_hms_opt(0, 0, 0)?);
            Some(Utc.from_utc_datetime(&naive))
        }
    }
}

fn record_to_event(record: CalendarEventRecord) -> CalendarEvent {
    let attendees: Vec<String> = serde_json::from_value(record.attendees.clone()).unwrap_or_default();
    let reminders: Vec<serde_json::Value> = serde_json::from_value(record.reminders.clone()).unwrap_or_default();
    let reminder_minutes = reminders.first()
        .and_then(|r| r.get("minutes_before"))
        .and_then(|m| m.as_i64())
        .map(|m| m as i32);

    CalendarEvent {
        id: record.id,
        calendar_id: record.calendar_id,
        title: record.title,
        description: record.description,
        start_time: record.start_time,
        end_time: record.end_time,
        location: record.location,
        attendees,
        organizer: record.owner_id.to_string(),
        reminder_minutes,
        recurrence: record.recurrence_rule,
        all_day: record.all_day,
        status: record.status,
        color: record.color,
        created_at: record.created_at,
        updated_at: record.updated_at,
    }
}

pub fn export_to_ical(events: &[CalendarEvent], calendar_name: &str) -> String {
    let mut calendar = Calendar::new();
    calendar.name(calendar_name);
    calendar.append_property(Property::new("PRODID", "-//GeneralBots//Calendar//EN"));

    for event in events {
        calendar.push(event.to_ical());
    }

    calendar.done().to_string()
}

pub fn import_from_ical(ical_str: &str, organizer: &str, calendar_id: Uuid) -> Vec<CalendarEvent> {
    let Ok(calendar) = ical_str.parse::<Calendar>() else {
        return Vec::new();
    };

    calendar
        .components
        .iter()
        .filter_map(|c| {
            if let icalendar::CalendarComponent::Event(e) = c {
                CalendarEvent::from_ical(e, organizer, calendar_id)
            } else {
                None
            }
        })
        .collect()
}

pub async fn create_calendar(
    State(state): State<Arc<AppState>>,
    Json(input): Json<CreateCalendarRequest>,
) -> Result<Json<CalendarRecord>, StatusCode> {
    let pool = state.conn.clone();
    let (org_id, bot_id) = get_bot_context();
    let owner_id = Uuid::nil();
    let now = Utc::now();

    let new_calendar = CalendarRecord {
        id: Uuid::new_v4(),
        org_id,
        bot_id,
        owner_id,
        name: input.name,
        description: input.description,
        color: input.color.or(Some("#3b82f6".to_string())),
        timezone: input.timezone.or(Some("UTC".to_string())),
        is_primary: input.is_primary,
        is_visible: true,
        is_shared: false,
        created_at: now,
        updated_at: now,
    };

    let calendar = new_calendar.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
        diesel::insert_into(calendars::table)
            .values(&new_calendar)
            .execute(&mut conn)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok::<_, StatusCode>(())
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    result?;
    info!("Created calendar: {} ({})", calendar.name, calendar.id);
    Ok(Json(calendar))
}

pub async fn list_calendars_db(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<CalendarRecord>>, StatusCode> {
    let pool = state.conn.clone();
    let (org_id, bot_id) = get_bot_context();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
        calendars::table
            .filter(calendars::org_id.eq(org_id))
            .filter(calendars::bot_id.eq(bot_id))
            .order(calendars::created_at.desc())
            .load::<CalendarRecord>(&mut conn)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(result?))
}

pub async fn get_calendar(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<CalendarRecord>, StatusCode> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
        calendars::table
            .find(id)
            .first::<CalendarRecord>(&mut conn)
            .optional()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    result?.ok_or(StatusCode::NOT_FOUND).map(Json)
}

pub async fn update_calendar(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateCalendarRequest>,
) -> Result<Json<CalendarRecord>, StatusCode> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;

        let mut calendar = calendars::table
            .find(id)
            .first::<CalendarRecord>(&mut conn)
            .optional()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;

        if let Some(name) = input.name {
            calendar.name = name;
        }
        if let Some(description) = input.description {
            calendar.description = Some(description);
        }
        if let Some(color) = input.color {
            calendar.color = Some(color);
        }
        if let Some(timezone) = input.timezone {
            calendar.timezone = Some(timezone);
        }
        if let Some(is_visible) = input.is_visible {
            calendar.is_visible = is_visible;
        }
        calendar.updated_at = Utc::now();

        diesel::update(calendars::table.find(id))
            .set(&calendar)
            .execute(&mut conn)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok::<_, StatusCode>(calendar)
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(result?))
}

pub async fn delete_calendar(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> StatusCode {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
        let deleted = diesel::delete(calendars::table.find(id))
            .execute(&mut conn)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if deleted > 0 {
            Ok::<_, StatusCode>(StatusCode::NO_CONTENT)
        } else {
            Ok(StatusCode::NOT_FOUND)
        }
    })
    .await;

    match result {
        Ok(Ok(status)) => status,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn list_events(
    State(state): State<Arc<AppState>>,
    Query(query): Query<EventQuery>,
) -> Result<Json<Vec<CalendarEvent>>, StatusCode> {
    let pool = state.conn.clone();
    let (org_id, bot_id) = get_bot_context();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;

        let mut db_query = calendar_events::table
            .filter(calendar_events::org_id.eq(org_id))
            .filter(calendar_events::bot_id.eq(bot_id))
            .into_boxed();

        if let Some(calendar_id) = query.calendar_id {
            db_query = db_query.filter(calendar_events::calendar_id.eq(calendar_id));
        }
        if let Some(start) = query.start {
            db_query = db_query.filter(calendar_events::start_time.ge(start));
        }
        if let Some(end) = query.end {
            db_query = db_query.filter(calendar_events::end_time.le(end));
        }

        db_query = db_query.order(calendar_events::start_time.asc());

        if let Some(limit) = query.limit {
            db_query = db_query.limit(limit);
        }
        if let Some(offset) = query.offset {
            db_query = db_query.offset(offset);
        }

        db_query
            .load::<CalendarEventRecord>(&mut conn)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let records = result?;
    let events: Vec<CalendarEvent> = records.into_iter().map(record_to_event).collect();
    Ok(Json(events))
}

pub async fn get_event(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<CalendarEvent>, StatusCode> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
        calendar_events::table
            .find(id)
            .first::<CalendarEventRecord>(&mut conn)
            .optional()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    result?.map(record_to_event).ok_or(StatusCode::NOT_FOUND).map(Json)
}

pub async fn create_event(
    State(state): State<Arc<AppState>>,
    Json(input): Json<CalendarEventInput>,
) -> Result<Json<CalendarEvent>, StatusCode> {
    let pool = state.conn.clone();
    let (org_id, bot_id) = get_bot_context();
    let owner_id = Uuid::nil();
    let now = Utc::now();

    let calendar_id = input.calendar_id.unwrap_or_else(Uuid::nil);

    let reminders = if let Some(minutes) = input.reminder_minutes {
        serde_json::json!([{"minutes_before": minutes, "type": "notification"}])
    } else {
        serde_json::json!([])
    };

    let new_event = CalendarEventRecord {
        id: Uuid::new_v4(),
        org_id,
        bot_id,
        calendar_id,
        owner_id,
        title: input.title.clone(),
        description: input.description.clone(),
        location: input.location.clone(),
        start_time: input.start_time,
        end_time: input.end_time,
        all_day: input.all_day,
        recurrence_rule: input.recurrence.clone(),
        recurrence_id: None,
        color: None,
        status: "confirmed".to_string(),
        visibility: "default".to_string(),
        busy_status: "busy".to_string(),
        reminders,
        attendees: serde_json::to_value(&input.attendees).unwrap_or(serde_json::json!([])),
        conference_data: None,
        metadata: serde_json::json!({}),
        created_at: now,
        updated_at: now,
    };

    let event_record = new_event.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
        diesel::insert_into(calendar_events::table)
            .values(&new_event)
            .execute(&mut conn)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok::<_, StatusCode>(())
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    result?;

    let event = record_to_event(event_record);
    info!("Created calendar event: {} ({})", event.title, event.id);
    Ok(Json(event))
}

pub async fn update_event(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(input): Json<CalendarEventInput>,
) -> Result<Json<CalendarEvent>, StatusCode> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;

        let mut event = calendar_events::table
            .find(id)
            .first::<CalendarEventRecord>(&mut conn)
            .optional()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;

        event.title = input.title;
        event.description = input.description;
        event.location = input.location;
        event.start_time = input.start_time;
        event.end_time = input.end_time;
        event.all_day = input.all_day;
        event.recurrence_rule = input.recurrence;
        event.attendees = serde_json::to_value(&input.attendees).unwrap_or(serde_json::json!([]));
        if let Some(minutes) = input.reminder_minutes {
            event.reminders = serde_json::json!([{"minutes_before": minutes, "type": "notification"}]);
        }
        event.updated_at = Utc::now();

        if let Some(calendar_id) = input.calendar_id {
            event.calendar_id = calendar_id;
        }

        diesel::update(calendar_events::table.find(id))
            .set(&event)
            .execute(&mut conn)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok::<_, StatusCode>(event)
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let event = record_to_event(result?);
    info!("Updated calendar event: {} ({})", event.title, event.id);
    Ok(Json(event))
}

pub async fn delete_event(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> StatusCode {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
        let deleted = diesel::delete(calendar_events::table.find(id))
            .execute(&mut conn)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if deleted > 0 {
            info!("Deleted calendar event: {id}");
            Ok::<_, StatusCode>(StatusCode::NO_CONTENT)
        } else {
            Ok(StatusCode::NOT_FOUND)
        }
    })
    .await;

    match result {
        Ok(Ok(status)) => status,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn share_calendar(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(input): Json<ShareCalendarRequest>,
) -> Result<Json<CalendarShareRecord>, StatusCode> {
    let pool = state.conn.clone();

    let new_share = CalendarShareRecord {
        id: Uuid::new_v4(),
        calendar_id: id,
        shared_with_user_id: input.user_id,
        shared_with_email: input.email,
        permission: input.permission,
        created_at: Utc::now(),
    };

    let share = new_share.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
        diesel::insert_into(calendar_shares::table)
            .values(&new_share)
            .execute(&mut conn)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok::<_, StatusCode>(())
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    result?;
    Ok(Json(share))
}

pub async fn export_ical(
    State(state): State<Arc<AppState>>,
    Path(calendar_id): Path<Uuid>,
) -> impl IntoResponse {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;

        let calendar = calendars::table
            .find(calendar_id)
            .first::<CalendarRecord>(&mut conn)
            .optional()
            .ok()??;

        let events = calendar_events::table
            .filter(calendar_events::calendar_id.eq(calendar_id))
            .load::<CalendarEventRecord>(&mut conn)
            .ok()?;

        let event_list: Vec<CalendarEvent> = events.into_iter().map(record_to_event).collect();
        Some(export_to_ical(&event_list, &calendar.name))
    })
    .await;

    match result {
        Ok(Some(ical)) => (
            StatusCode::OK,
            [(axum::http::header::CONTENT_TYPE, "text/calendar; charset=utf-8")],
            ical,
        ).into_response(),
        _ => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn import_ical(
    State(state): State<Arc<AppState>>,
    Path(calendar_id): Path<Uuid>,
    body: String,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let pool = state.conn.clone();
    let (org_id, bot_id) = get_bot_context();
    let owner_id = Uuid::nil();

    let events = import_from_ical(&body, &owner_id.to_string(), calendar_id);
    let count = events.len();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
        let now = Utc::now();

        for event in events {
            let record = CalendarEventRecord {
                id: event.id,
                org_id,
                bot_id,
                calendar_id,
                owner_id,
                title: event.title,
                description: event.description,
                location: event.location,
                start_time: event.start_time,
                end_time: event.end_time,
                all_day: event.all_day,
                recurrence_rule: event.recurrence,
                recurrence_id: None,
                color: event.color,
                status: event.status,
                visibility: "default".to_string(),
                busy_status: "busy".to_string(),
                reminders: serde_json::json!([]),
                attendees: serde_json::to_value(&event.attendees).unwrap_or(serde_json::json!([])),
                conference_data: None,
                metadata: serde_json::json!({}),
                created_at: now,
                updated_at: now,
            };

            diesel::insert_into(calendar_events::table)
                .values(&record)
                .execute(&mut conn)
                .ok();
        }

        Ok::<_, StatusCode>(())
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    result?;
    Ok(Json(serde_json::json!({ "imported": count })))
}

pub async fn list_calendars_api(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let pool = state.conn.clone();
    let (org_id, bot_id) = get_bot_context();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        calendars::table
            .filter(calendars::org_id.eq(org_id))
            .filter(calendars::bot_id.eq(bot_id))
            .load::<CalendarRecord>(&mut conn)
            .ok()
    })
    .await;

    match result {
        Ok(Some(cals)) => {
            let calendar_list: Vec<serde_json::Value> = cals.iter().map(|c| {
                serde_json::json!({
                    "id": c.id,
                    "name": c.name,
                    "color": c.color,
                    "visible": c.is_visible
                })
            }).collect();
            Json(serde_json::json!({ "calendars": calendar_list }))
        }
        _ => Json(serde_json::json!({
            "calendars": [{
                "id": "default",
                "name": "My Calendar",
                "color": "#3b82f6",
                "visible": true
            }]
        })),
    }
}

pub async fn list_calendars_html(State(state): State<Arc<AppState>>) -> Html<String> {
    let pool = state.conn.clone();
    let (org_id, bot_id) = get_bot_context();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        calendars::table
            .filter(calendars::org_id.eq(org_id))
            .filter(calendars::bot_id.eq(bot_id))
            .load::<CalendarRecord>(&mut conn)
            .ok()
    })
    .await;

    match result {
        Ok(Some(cals)) if !cals.is_empty() => {
            let html: String = cals.iter().map(|c| {
                let color = c.color.as_deref().unwrap_or("#3b82f6");
                let checked = if c.is_visible { "checked" } else { "" };
                format!(
                    r#"<div class="calendar-item" data-calendar-id="{}">
                        <span class="calendar-checkbox {}" style="background: {};" onclick="toggleCalendar(this)"></span>
                        <span class="calendar-name">{}</span>
                    </div>"#,
                    c.id, checked, color, c.name
                )
            }).collect();
            Html(html)
        }
        _ => Html(r#"
            <div class="calendar-item" data-calendar-id="default">
                <span class="calendar-checkbox checked" style="background: #3b82f6;" onclick="toggleCalendar(this)"></span>
                <span class="calendar-name">My Calendar</span>
            </div>
        "#.to_string()),
    }
}

pub async fn upcoming_events_api(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let pool = state.conn.clone();
    let (org_id, bot_id) = get_bot_context();
    let now = Utc::now();
    let end = now + chrono::Duration::days(7);

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        calendar_events::table
            .filter(calendar_events::org_id.eq(org_id))
            .filter(calendar_events::bot_id.eq(bot_id))
            .filter(calendar_events::start_time.ge(now))
            .filter(calendar_events::start_time.le(end))
            .order(calendar_events::start_time.asc())
            .limit(10)
            .load::<CalendarEventRecord>(&mut conn)
            .ok()
    })
    .await;

    match result {
        Ok(Some(events)) => {
            let event_list: Vec<serde_json::Value> = events.iter().map(|e| {
                serde_json::json!({
                    "id": e.id,
                    "title": e.title,
                    "start_time": e.start_time,
                    "end_time": e.end_time,
                    "location": e.location
                })
            }).collect();
            Json(serde_json::json!({ "events": event_list }))
        }
        _ => Json(serde_json::json!({
            "events": [],
            "message": "No upcoming events"
        })),
    }
}

pub async fn upcoming_events_html(State(state): State<Arc<AppState>>) -> Html<String> {
    let pool = state.conn.clone();
    let (org_id, bot_id) = get_bot_context();
    let now = Utc::now();
    let end = now + chrono::Duration::days(7);

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        calendar_events::table
            .filter(calendar_events::org_id.eq(org_id))
            .filter(calendar_events::bot_id.eq(bot_id))
            .filter(calendar_events::start_time.ge(now))
            .filter(calendar_events::start_time.le(end))
            .order(calendar_events::start_time.asc())
            .limit(5)
            .load::<CalendarEventRecord>(&mut conn)
            .ok()
    })
    .await;

    match result {
        Ok(Some(events)) if !events.is_empty() => {
            let html: String = events.iter().map(|e| {
                let color = e.color.as_deref().unwrap_or("#3b82f6");
                let time = e.start_time.format("%b %d, %H:%M").to_string();
                format!(
                    r#"<div class="upcoming-event">
                        <div class="upcoming-color" style="background: {};"></div>
                        <div class="upcoming-info">
                            <span class="upcoming-title">{}</span>
                            <span class="upcoming-time">{}</span>
                        </div>
                    </div>"#,
                    color, e.title, time
                )
            }).collect();
            Html(html)
        }
        _ => Html(r#"
            <div class="upcoming-event">
                <div class="upcoming-color" style="background: #3b82f6;"></div>
                <div class="upcoming-info">
                    <span class="upcoming-title">No upcoming events</span>
                    <span class="upcoming-time">Create your first event</span>
                </div>
            </div>
        "#.to_string()),
    }
}

pub async fn new_event_form() -> Html<String> {
    Html(r#"
        <div class="event-form-content">
            <p>Create a new event using the form on the right panel.</p>
        </div>
    "#.to_string())
}

pub async fn new_calendar_form() -> Html<String> {
    Html(r##"
        <form class="calendar-form" hx-post="/api/calendar/calendars" hx-swap="none">
            <div class="form-group">
                <label>Calendar Name</label>
                <input type="text" name="name" placeholder="My Calendar" required />
            </div>
            <div class="form-group">
                <label>Color</label>
                <div class="color-options">
                    <label><input type="radio" name="color" value="#3b82f6" checked /><span class="color-dot" style="background:#3b82f6"></span></label>
                    <label><input type="radio" name="color" value="#22c55e" /><span class="color-dot" style="background:#22c55e"></span></label>
                    <label><input type="radio" name="color" value="#f59e0b" /><span class="color-dot" style="background:#f59e0b"></span></label>
                    <label><input type="radio" name="color" value="#ef4444" /><span class="color-dot" style="background:#ef4444"></span></label>
                    <label><input type="radio" name="color" value="#8b5cf6" /><span class="color-dot" style="background:#8b5cf6"></span></label>
                </div>
            </div>
            <div class="form-actions">
                <button type="button" class="btn-secondary" onclick="this.closest('.modal').classList.add('hidden')">Cancel</button>
                <button type="submit" class="btn-primary">Create Calendar</button>
            </div>
        </form>
    "##.to_string())
}

pub fn configure_calendar_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/calendar/calendars", get(list_calendars_db).post(create_calendar))
        .route("/api/calendar/calendars/:id", get(get_calendar).put(update_calendar).delete(delete_calendar))
        .route("/api/calendar/calendars/:id/share", post(share_calendar))
        .route("/api/calendar/calendars/:id/export", get(export_ical))
        .route("/api/calendar/calendars/:id/import", post(import_ical))
        .route(ApiUrls::CALENDAR_EVENTS, get(list_events).post(create_event))
        .route(ApiUrls::CALENDAR_EVENT_BY_ID, get(get_event).put(update_event).delete(delete_event))
        .route(ApiUrls::CALENDAR_UPCOMING_JSON, get(upcoming_events_api))
        .route(ApiUrls::CALENDAR_CALENDARS, get(list_calendars_html))
        .route(ApiUrls::CALENDAR_UPCOMING, get(upcoming_events_html))
        .route(ApiUrls::CALENDAR_NEW_EVENT_FORM, get(new_event_form))
        .route(ApiUrls::CALENDAR_NEW_CALENDAR_FORM, get(new_calendar_form))
}

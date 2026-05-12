use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use chrono::{DateTime, Datelike, Duration, NaiveDate, NaiveDateTime, TimeZone, Utc};
use diesel::prelude::*;
use icalendar::{
    Calendar, CalendarDateTime, Component, DatePerhapsTime, Event as IcalEvent, EventLike,
    Property,
};
use log::info;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

pub type DbPool = diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>>;

pub trait SecretsProvider: Send + Sync + 'static {
    fn get_value(&self, path: &str, key: &str) -> Option<String>;
}

pub trait DefaultBotProvider: Send + Sync + 'static {
    fn get_default_bot(&self, conn: &mut diesel::PgConnection) -> (Uuid, String);
}

pub trait CalendarEngineProvider: Send + Sync + 'static {}

diesel::table! {
    calendars (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        owner_id -> Uuid,
        name -> Varchar,
        description -> Nullable<Text>,
        color -> Nullable<Varchar>,
        timezone -> Nullable<Varchar>,
        is_primary -> Bool,
        is_visible -> Bool,
        is_shared -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    calendar_events (id) {
        id -> Uuid,
        org_id -> Uuid,
        bot_id -> Uuid,
        calendar_id -> Uuid,
        owner_id -> Uuid,
        title -> Varchar,
        description -> Nullable<Text>,
        location -> Nullable<Varchar>,
        start_time -> Timestamptz,
        end_time -> Timestamptz,
        all_day -> Bool,
        recurrence_rule -> Nullable<Text>,
        recurrence_id -> Nullable<Uuid>,
        color -> Nullable<Varchar>,
        status -> Varchar,
        visibility -> Varchar,
        busy_status -> Varchar,
        reminders -> Jsonb,
        attendees -> Jsonb,
        conference_data -> Nullable<Jsonb>,
        metadata -> Jsonb,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    calendar_event_attendees (id) {
        id -> Uuid,
        event_id -> Uuid,
        email -> Varchar,
        name -> Nullable<Varchar>,
        status -> Varchar,
        role -> Varchar,
        rsvp_time -> Nullable<Timestamptz>,
        comment -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    calendar_shares (id) {
        id -> Uuid,
        calendar_id -> Uuid,
        shared_with_user_id -> Nullable<Uuid>,
        shared_with_email -> Nullable<Varchar>,
        permission -> Varchar,
        created_at -> Timestamptz,
    }
}

diesel::joinable!(calendar_events -> calendars (calendar_id));
diesel::joinable!(calendar_event_attendees -> calendar_events (event_id));
diesel::joinable!(calendar_shares -> calendars (calendar_id));

diesel::allow_tables_to_appear_in_same_query!(
    calendars,
    calendar_events,
    calendar_event_attendees,
    calendar_shares,
);

const API_CALENDAR_EVENTS: &str = "/api/calendar/events";
const API_CALENDAR_EVENT_BY_ID: &str = "/api/calendar/events/:id";
const API_CALENDAR_UPCOMING_JSON: &str = "/api/calendar/events/upcoming";

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
    let attendees: Vec<String> =
        serde_json::from_value(record.attendees.clone()).unwrap_or_default();
    let reminders: Vec<serde_json::Value> =
        serde_json::from_value(record.reminders.clone()).unwrap_or_default();
    let reminder_minutes = reminders
        .first()
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

pub fn get_bot_context_from_secrets(secrets: &dyn SecretsProvider) -> (Uuid, Uuid) {
    let org_id = secrets
        .get_value("gbo/analytics", "default_org_id")
        .unwrap_or_else(|| "system".to_string());
    let bot_id = secrets
        .get_value("gbo/analytics", "default_bot_id")
        .unwrap_or_else(|| "system".to_string());
    (
        Uuid::parse_str(&org_id).unwrap_or_else(|_| Uuid::nil()),
        Uuid::parse_str(&bot_id).unwrap_or_else(|_| Uuid::nil()),
    )
}

pub async fn create_calendar(
    State(state): State<Arc<DbPool>>,
    Json(input): Json<CreateCalendarRequest>,
) -> Result<Json<CalendarRecord>, StatusCode> {
    let pool = state.clone();
    let (org_id, bot_id) = (Uuid::nil(), Uuid::nil());
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
    State(state): State<Arc<DbPool>>,
) -> Result<Json<Vec<CalendarRecord>>, StatusCode> {
    let pool = state.clone();
    let (org_id, bot_id) = (Uuid::nil(), Uuid::nil());

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
    State(state): State<Arc<DbPool>>,
    Path(id): Path<Uuid>,
) -> Result<Json<CalendarRecord>, StatusCode> {
    let pool = state.clone();

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
    State(state): State<Arc<DbPool>>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateCalendarRequest>,
) -> Result<Json<CalendarRecord>, StatusCode> {
    let pool = state.clone();

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
    State(state): State<Arc<DbPool>>,
    Path(id): Path<Uuid>,
) -> StatusCode {
    let pool = state.clone();

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
    State(state): State<Arc<DbPool>>,
    Query(query): Query<EventQuery>,
) -> Result<Json<Vec<CalendarEvent>>, StatusCode> {
    let pool = state.clone();
    let (org_id, bot_id) = (Uuid::nil(), Uuid::nil());

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
    State(state): State<Arc<DbPool>>,
    Path(id): Path<Uuid>,
) -> Result<Json<CalendarEvent>, StatusCode> {
    let pool = state.clone();

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

    result?
        .map(record_to_event)
        .ok_or(StatusCode::NOT_FOUND)
        .map(Json)
}

pub async fn create_event(
    State(state): State<Arc<DbPool>>,
    Json(input): Json<CalendarEventInput>,
) -> Result<Json<CalendarEvent>, StatusCode> {
    let pool = state.clone();
    let (org_id, bot_id) = (Uuid::nil(), Uuid::nil());
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
    State(state): State<Arc<DbPool>>,
    Path(id): Path<Uuid>,
    Json(input): Json<CalendarEventInput>,
) -> Result<Json<CalendarEvent>, StatusCode> {
    let pool = state.clone();

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
        event.attendees =
            serde_json::to_value(&input.attendees).unwrap_or(serde_json::json!([]));
        if let Some(minutes) = input.reminder_minutes {
            event.reminders =
                serde_json::json!([{"minutes_before": minutes, "type": "notification"}]);
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
    State(state): State<Arc<DbPool>>,
    Path(id): Path<Uuid>,
) -> StatusCode {
    let pool = state.clone();

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
    State(state): State<Arc<DbPool>>,
    Path(id): Path<Uuid>,
    Json(input): Json<ShareCalendarRequest>,
) -> Result<Json<CalendarShareRecord>, StatusCode> {
    let pool = state.clone();

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
    State(state): State<Arc<DbPool>>,
    Path(calendar_id): Path<Uuid>,
) -> impl IntoResponse {
    let pool = state.clone();

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
        )
            .into_response(),
        _ => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn import_ical(
    State(state): State<Arc<DbPool>>,
    Path(calendar_id): Path<Uuid>,
    body: String,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let pool = state.clone();
    let (org_id, bot_id) = (Uuid::nil(), Uuid::nil());
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
                attendees: serde_json::to_value(&event.attendees)
                    .unwrap_or(serde_json::json!([])),
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

pub async fn list_calendars_api(State(state): State<Arc<DbPool>>) -> Json<serde_json::Value> {
    let pool = state.clone();
    let (org_id, bot_id) = (Uuid::nil(), Uuid::nil());

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
            let calendar_list: Vec<serde_json::Value> = cals
                .iter()
                .map(|c| {
                    serde_json::json!({
                        "id": c.id,
                        "name": c.name,
                        "color": c.color,
                        "visible": c.is_visible
                    })
                })
                .collect();
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

pub async fn list_calendars_html(State(state): State<Arc<DbPool>>) -> Html<String> {
    let pool = state.clone();
    let (org_id, bot_id) = (Uuid::nil(), Uuid::nil());

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
            let html: String = cals
                .iter()
                .map(|c| {
                    let color = c.color.as_deref().unwrap_or("#3b82f6");
                    let checked = if c.is_visible { "checked" } else { "" };
                    format!(
                        r#"<div class="calendar-item" data-calendar-id="{}">
<span class="calendar-checkbox {}" style="background: {};" onclick="toggleCalendar(this)"></span>
<span class="calendar-name">{}</span>
</div>"#,
                        c.id, checked, color, c.name
                    )
                })
                .collect();
            Html(html)
        }
        _ => Html(
            r#"
<div class="calendar-item" data-calendar-id="default">
<span class="calendar-checkbox checked" style="background: #3b82f6;" onclick="toggleCalendar(this)"></span>
<span class="calendar-name">My Calendar</span>
</div>
"#
            .to_string(),
        ),
    }
}

pub async fn upcoming_events_api(State(state): State<Arc<DbPool>>) -> Json<serde_json::Value> {
    let pool = state.clone();
    let (org_id, bot_id) = (Uuid::nil(), Uuid::nil());
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
            let event_list: Vec<serde_json::Value> = events
                .iter()
                .map(|e| {
                    serde_json::json!({
                        "id": e.id,
                        "title": e.title,
                        "start_time": e.start_time,
                        "end_time": e.end_time,
                        "location": e.location
                    })
                })
                .collect();
            Json(serde_json::json!({ "events": event_list }))
        }
        _ => Json(serde_json::json!({
            "events": [],
            "message": "No upcoming events"
        })),
    }
}

pub async fn upcoming_events_html(State(state): State<Arc<DbPool>>) -> Html<String> {
    let pool = state.clone();
    let (org_id, bot_id) = (Uuid::nil(), Uuid::nil());
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
            let html: String = events
                .iter()
                .map(|e| {
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
                })
                .collect();
            Html(html)
        }
        _ => Html(
            r#"
<div class="upcoming-event">
<div class="upcoming-color" style="background: #3b82f6;"></div>
<div class="upcoming-info">
<span class="upcoming-title">No upcoming events</span>
<span class="upcoming-time">Create your first event</span>
</div>
</div>
"#
            .to_string(),
        ),
    }
}

pub async fn new_event_form() -> Html<String> {
    Html(
        r#"
<div class="event-form-content">
<p>Create a new event using the form on the right panel.</p>
</div>
"#
        .to_string(),
    )
}

pub async fn new_calendar_form() -> Html<String> {
    Html(
        r##"<form class="calendar-form" hx-post="/api/calendar/calendars" hx-swap="none">
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
</form>"##
            .to_string(),
    )
}

pub fn configure_calendar_routes() -> Router<Arc<DbPool>> {
    Router::new()
        .route("/api/calendar/calendars", get(list_calendars_db).post(create_calendar))
        .route(
            "/api/calendar/calendars/:id",
            get(get_calendar).put(update_calendar).delete(delete_calendar),
        )
        .route("/api/calendar/calendars/:id/share", post(share_calendar))
        .route("/api/calendar/calendars/:id/export", get(export_ical))
        .route("/api/calendar/calendars/:id/import", post(import_ical))
        .route(API_CALENDAR_EVENTS, get(list_events).post(create_event))
        .route(
            API_CALENDAR_EVENT_BY_ID,
            get(get_event).put(update_event).delete(delete_event),
        )
        .route(API_CALENDAR_UPCOMING_JSON, get(upcoming_events_api))
}

pub fn create_caldav_router() -> Router<Arc<DbPool>> {
    Router::new()
        .route("/caldav", get(caldav_root))
        .route("/caldav/principals", get(caldav_principals))
        .route("/caldav/calendars", get(caldav_calendars))
        .route("/caldav/calendars/:calendar_id", get(caldav_calendar))
        .route(
            "/caldav/calendars/:calendar_id/:event_id.ics",
            get(caldav_event).put(caldav_put_event),
        )
}

async fn caldav_root() -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::OK)
        .header("DAV", "1, 2, calendar-access")
        .header("Content-Type", "application/xml; charset=utf-8")
        .body(
            r#"<?xml version="1.0" encoding="utf-8"?>
<D:multistatus xmlns:D="DAV:" xmlns:C="urn:ietf:params:xml:ns:caldav">
<D:response>
<D:href>/caldav/</D:href>
<D:propstat>
<D:prop>
<D:resourcetype>
<D:collection/>
</D:resourcetype>
<D:displayname>GeneralBots CalDAV Server</D:displayname>
</D:prop>
<D:status>HTTP/1.1 200 OK</D:status>
</D:propstat>
</D:response>
</D:multistatus>"#
                .to_string(),
        )
        .unwrap_or_default()
}

async fn caldav_principals() -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/xml; charset=utf-8")
        .body(
            r#"<?xml version="1.0" encoding="utf-8"?>
<D:multistatus xmlns:D="DAV:" xmlns:C="urn:ietf:params:xml:ns:caldav">
<D:response>
<D:href>/caldav/principals/</D:href>
<D:propstat>
<D:prop>
<D:resourcetype>
<D:collection/>
<D:principal/>
</D:resourcetype>
<C:calendar-home-set>
<D:href>/caldav/calendars/</D:href>
</C:calendar-home-set>
</D:prop>
<D:status>HTTP/1.1 200 OK</D:status>
</D:propstat>
</D:response>
</D:multistatus>"#
                .to_string(),
        )
        .unwrap_or_default()
}

async fn caldav_calendars() -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/xml; charset=utf-8")
        .body(
            r#"<?xml version="1.0" encoding="utf-8"?>
<D:multistatus xmlns:D="DAV:" xmlns:C="urn:ietf:params:xml:ns:caldav">
<D:response>
<D:href>/caldav/calendars/</D:href>
<D:propstat>
<D:prop>
<D:resourcetype>
<D:collection/>
</D:resourcetype>
<D:displayname>Calendars</D:displayname>
</D:prop>
<D:status>HTTP/1.1 200 OK</D:status>
</D:propstat>
</D:response>
<D:response>
<D:href>/caldav/calendars/default/</D:href>
<D:propstat>
<D:prop>
<D:resourcetype>
<D:collection/>
<C:calendar/>
</D:resourcetype>
<D:displayname>Default Calendar</D:displayname>
<C:supported-calendar-component-set>
<C:comp name="VEVENT"/>
<C:comp name="VTODO"/>
</C:supported-calendar-component-set>
</D:prop>
<D:status>HTTP/1.1 200 OK</D:status>
</D:propstat>
</D:response>
</D:multistatus>"#
                .to_string(),
        )
        .unwrap_or_default()
}

async fn caldav_calendar() -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/xml; charset=utf-8")
        .body(
            r#"<?xml version="1.0" encoding="utf-8"?>
<D:multistatus xmlns:D="DAV:" xmlns:C="urn:ietf:params:xml:ns:caldav">
<D:response>
<D:href>/caldav/calendars/default/</D:href>
<D:propstat>
<D:prop>
<D:resourcetype>
<D:collection/>
<C:calendar/>
</D:resourcetype>
<D:displayname>Default Calendar</D:displayname>
</D:prop>
<D:status>HTTP/1.1 200 OK</D:status>
</D:propstat>
</D:response>
</D:multistatus>"#
                .to_string(),
        )
        .unwrap_or_default()
}

async fn caldav_event() -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/calendar; charset=utf-8")
        .body(
            r"BEGIN:VCALENDAR
VERSION:2.0
PRODID:-
BEGIN:VEVENT
UID:placeholder@generalbots.com
DTSTAMP:20240101T000000Z
DTSTART:20240101T090000Z
DTEND:20240101T100000Z
SUMMARY:Placeholder Event
END:VEVENT
END:VCALENDAR"
            .to_string(),
        )
        .unwrap_or_default()
}

async fn caldav_put_event() -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::CREATED)
        .header("ETag", "\"placeholder-etag\"")
        .body(String::new())
        .unwrap_or_default()
}

#[derive(Debug, Deserialize, Default)]
pub struct EventsQuery {
    pub calendar_id: Option<Uuid>,
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTime<Utc>>,
    pub view: Option<String>,
}

pub async fn ui_events_list(
    State(state): State<Arc<DbPool>>,
    Query(query): Query<EventsQuery>,
) -> Html<String> {
    let pool = state.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;

        let now = Utc::now();
        let start = query.start.unwrap_or(now);
        let end = query.end.unwrap_or(now + Duration::days(30));

        let mut db_query = calendar_events::table
            .filter(calendar_events::start_time.ge(start))
            .filter(calendar_events::start_time.le(end))
            .into_boxed();

        if let Some(calendar_id) = query.calendar_id {
            db_query = db_query.filter(calendar_events::calendar_id.eq(calendar_id));
        }

        db_query = db_query.order(calendar_events::start_time.asc());

        db_query
            .select((
                calendar_events::id,
                calendar_events::title,
                calendar_events::description,
                calendar_events::location,
                calendar_events::start_time,
                calendar_events::end_time,
                calendar_events::all_day,
                calendar_events::color,
                calendar_events::status,
            ))
            .load::<(
                Uuid,
                String,
                Option<String>,
                Option<String>,
                DateTime<Utc>,
                DateTime<Utc>,
                bool,
                Option<String>,
                String,
            )>(&mut conn)
            .ok()
    })
    .await
    .ok()
    .flatten();

    match result {
        Some(events) if !events.is_empty() => {
            let items: String = events
                .iter()
                .map(|(id, title, _desc, location, start, end, all_day, color, _status)| {
                    let event_color = color.clone().unwrap_or_else(|| "#3b82f6".to_string());
                    let location_text = location.clone().unwrap_or_default();
                    let time_str = if *all_day {
                        "All day".to_string()
                    } else {
                        format!("{} - {}", start.format("%H:%M"), end.format("%H:%M"))
                    };
                    let date_str = start.format("%b %d").to_string();

                    format!(
                        r##"<div class="event-item" data-id="{}" style="border-left: 4px solid {};"
hx-get="/api/ui/calendar/events/{}" hx-target="#event-detail" hx-swap="innerHTML">
<div class="event-date">{}</div>
<div class="event-content">
<span class="event-title">{}</span>
<span class="event-time">{}</span>
{}</div>
</div>"##,
                        id,
                        event_color,
                        id,
                        date_str,
                        title,
                        time_str,
                        if location_text.is_empty() {
                            String::new()
                        } else {
                            format!(r##"<span class="event-location">{}</span>"##, location_text)
                        }
                    )
                })
                .collect();

            Html(format!(r##"<div class="events-list">{}</div>"##, items))
        }
        _ => Html(
            r##"<div class="empty-state">
<p>No events found</p>
<button class="btn btn-primary" hx-get="/api/ui/calendar/new-event" hx-target="#modal-content">
Create Event
</button>
</div>"##
            .to_string(),
        ),
    }
}

pub async fn ui_event_detail(
    State(state): State<Arc<DbPool>>,
    Path(id): Path<Uuid>,
) -> Html<String> {
    let pool = state.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;

        calendar_events::table
            .find(id)
            .select((
                calendar_events::id,
                calendar_events::title,
                calendar_events::description,
                calendar_events::location,
                calendar_events::start_time,
                calendar_events::end_time,
                calendar_events::all_day,
                calendar_events::color,
                calendar_events::status,
                calendar_events::attendees,
            ))
            .first::<(
                Uuid,
                String,
                Option<String>,
                Option<String>,
                DateTime<Utc>,
                DateTime<Utc>,
                bool,
                Option<String>,
                String,
                serde_json::Value,
            )>(&mut conn)
            .ok()
    })
    .await
    .ok()
    .flatten();

    match result {
        Some((id, title, desc, location, start, end, all_day, color, status, attendees)) => {
            let description = desc.unwrap_or_else(|| "No description".to_string());
            let location_text = location.unwrap_or_else(|| "No location".to_string());
            let event_color = color.unwrap_or_else(|| "#3b82f6".to_string());

            let time_str = if all_day {
                format!("{} (All day)", start.format("%B %d, %Y"))
            } else {
                format!(
                    "{} - {}",
                    start.format("%B %d, %Y %H:%M"),
                    end.format("%H:%M")
                )
            };

            let attendees_list: Vec<String> =
                serde_json::from_value(attendees).unwrap_or_default();
            let attendees_html = if attendees_list.is_empty() {
                "<p>No attendees</p>".to_string()
            } else {
                attendees_list
                    .iter()
                    .map(|a| format!(r##"<span class="attendee-badge">{}</span>"##, a))
                    .collect::<Vec<_>>()
                    .join("")
            };

            Html(format!(
                r##"<div class="event-detail-card">
<div class="detail-header" style="border-left: 4px solid {};">
<h3>{}</h3>
<span class="status-badge status-{}">{}</span>
</div>
<div class="detail-section">
<div class="detail-item">
<label>When</label>
<span>{}</span>
</div>
<div class="detail-item">
<label>Where</label>
<span>{}</span>
</div>
</div>
<div class="detail-section">
<h4>Description</h4>
<p>{}</p>
</div>
<div class="detail-section">
<h4>Attendees</h4>
<div class="attendees-list">{}</div>
</div>
<div class="detail-actions">
<button class="btn btn-primary" hx-get="/api/ui/calendar/events/{}/edit" hx-target="#modal-content">Edit</button>
<button class="btn btn-danger" hx-delete="/api/calendar/events/{}" hx-swap="none" hx-confirm="Delete this event?">Delete</button>
</div>
</div>"##,
                event_color,
                title,
                status,
                status,
                time_str,
                location_text,
                description,
                attendees_html,
                id,
                id
            ))
        }
        None => Html(
            r##"<div class="empty-state">
<p>Event not found</p>
</div>"##
            .to_string(),
        ),
    }
}

pub async fn ui_calendars_sidebar(State(state): State<Arc<DbPool>>) -> Html<String> {
    let pool = state.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;

        calendars::table
            .order(calendars::is_primary.desc())
            .select((
                calendars::id,
                calendars::name,
                calendars::color,
                calendars::is_visible,
                calendars::is_primary,
            ))
            .load::<(Uuid, String, Option<String>, bool, bool)>(&mut conn)
            .ok()
    })
    .await
    .ok()
    .flatten();

    match result {
        Some(cals) if !cals.is_empty() => {
            let items: String = cals
                .iter()
                .map(|(id, name, color, visible, primary)| {
                    let cal_color = color.clone().unwrap_or_else(|| "#3b82f6".to_string());
                    let checked = if *visible { "checked" } else { "" };
                    let primary_badge = if *primary {
                        r##"<span class="primary-badge">Primary</span>"##
                    } else {
                        ""
                    };

                    format!(
                        r##"<div class="calendar-item" data-calendar-id="{}">
<input type="checkbox" class="calendar-checkbox" {}
hx-put="/api/calendar/calendars/{}"
hx-vals='{{"is_visible": {}}}'
hx-swap="none" />
<span class="calendar-color" style="background: {};"></span>
<span class="calendar-name">{}</span>
{}
</div>"##,
                        id, checked, id, !visible, cal_color, name, primary_badge
                    )
                })
                .collect();

            Html(format!(
                r##"<div class="calendars-sidebar">
<div class="sidebar-header">
<h4>My Calendars</h4>
<button class="btn-icon" hx-get="/api/ui/calendar/new-calendar" hx-target="#modal-content">+</button>
</div>
<div class="calendars-list">{}</div>
</div>"##,
                items
            ))
        }
        _ => Html(
            r##"<div class="calendars-sidebar">
<div class="sidebar-header">
<h4>My Calendars</h4>
<button class="btn-icon" hx-get="/api/ui/calendar/new-calendar" hx-target="#modal-content">+</button>
</div>
<div class="empty-state">
<p>No calendars yet</p>
</div>
</div>"##
            .to_string(),
        ),
    }
}

pub async fn ui_upcoming_events(State(state): State<Arc<DbPool>>) -> Html<String> {
    let pool = state.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;

        let now = Utc::now();
        let end = now + Duration::days(7);

        calendar_events::table
            .filter(calendar_events::start_time.ge(now))
            .filter(calendar_events::start_time.le(end))
            .order(calendar_events::start_time.asc())
            .limit(5)
            .select((
                calendar_events::id,
                calendar_events::title,
                calendar_events::start_time,
                calendar_events::color,
            ))
            .load::<(Uuid, String, DateTime<Utc>, Option<String>)>(&mut conn)
            .ok()
    })
    .await
    .ok()
    .flatten();

    match result {
        Some(events) if !events.is_empty() => {
            let items: String = events
                .iter()
                .map(|(id, title, start, color)| {
                    let event_color = color.clone().unwrap_or_else(|| "#3b82f6".to_string());
                    let time_str = start.format("%b %d, %H:%M").to_string();

                    format!(
                        r##"<div class="upcoming-event" hx-get="/api/ui/calendar/events/{}" hx-target="#event-detail">
<div class="upcoming-color" style="background: {};"></div>
<div class="upcoming-info">
<span class="upcoming-title">{}</span>
<span class="upcoming-time">{}</span>
</div>
</div>"##,
                        id, event_color, title, time_str
                    )
                })
                .collect();

            Html(format!(r##"<div class="upcoming-list">{}</div>"##, items))
        }
        _ => Html(
            r##"<div class="upcoming-event">
<div class="upcoming-color" style="background: #94a3b8;"></div>
<div class="upcoming-info">
<span class="upcoming-title">No upcoming events</span>
<span class="upcoming-time">Create your first event</span>
</div>
</div>"##
            .to_string(),
        ),
    }
}

pub async fn ui_events_count(State(state): State<Arc<DbPool>>) -> Html<String> {
    let pool = state.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;

        calendar_events::table
            .count()
            .get_result::<i64>(&mut conn)
            .ok()
    })
    .await
    .ok()
    .flatten();

    Html(result.unwrap_or(0).to_string())
}

pub async fn ui_today_events_count(State(state): State<Arc<DbPool>>) -> Html<String> {
    let pool = state.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;

        let today = Utc::now().date_naive();
        let today_start = today.and_hms_opt(0, 0, 0)?;
        let today_end = today.and_hms_opt(23, 59, 59)?;

        calendar_events::table
            .filter(calendar_events::start_time.ge(today_start))
            .filter(calendar_events::start_time.le(today_end))
            .count()
            .get_result::<i64>(&mut conn)
            .ok()
    })
    .await
    .ok()
    .flatten();

    Html(result.unwrap_or(0).to_string())
}

#[derive(Debug, Deserialize, Default)]
pub struct MonthQuery {
    pub year: Option<i32>,
    pub month: Option<u32>,
}

pub async fn ui_month_view(
    State(state): State<Arc<DbPool>>,
    Query(query): Query<MonthQuery>,
) -> Html<String> {
    let pool = state.clone();
    let now = Utc::now();
    let year = query.year.unwrap_or(now.year());
    let month = query.month.unwrap_or(now.month());

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;

        let first_day = NaiveDate::from_ymd_opt(year, month, 1)?;
        let last_day = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1)?
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1)?
        }
        .pred_opt()?;

        let start = first_day.and_hms_opt(0, 0, 0)?;
        let end = last_day.and_hms_opt(23, 59, 59)?;

        let events = calendar_events::table
            .filter(calendar_events::start_time.ge(start))
            .filter(calendar_events::start_time.le(end))
            .select((
                calendar_events::id,
                calendar_events::title,
                calendar_events::start_time,
                calendar_events::color,
            ))
            .load::<(Uuid, String, DateTime<Utc>, Option<String>)>(&mut conn)
            .ok()?;

        Some((first_day, last_day, events))
    })
    .await
    .ok()
    .flatten();

    match result {
        Some((first_day, last_day, events)) => {
            let month_name = first_day.format("%B %Y").to_string();
            let start_weekday = first_day.weekday().num_days_from_sunday();

            let mut days_html = String::new();

            for _ in 0..start_weekday {
                days_html.push_str(r#"<div class="calendar-day empty"></div>"#);
            }

            let mut current = first_day;
            while current <= last_day {
                let day_num = current.day();
                let day_events: Vec<_> = events
                    .iter()
                    .filter(|(_, _, start, _)| start.date_naive() == current)
                    .collect();

                let events_dots: String = day_events
                    .iter()
                    .take(3)
                    .map(|(_, _, _, color)| {
                        let c = color.clone().unwrap_or_else(|| "#3b82f6".to_string());
                        format!(r##"<span class="event-dot" style="background: {};"></span>"##, c)
                    })
                    .collect();

                let is_today = current == Utc::now().date_naive();
                let today_class = if is_today { "today" } else { "" };

                days_html.push_str(&format!(
                    r##"<div class="calendar-day {}" data-date="{}"
hx-get="/api/ui/calendar/day?date={}" hx-target="#day-events">
<span class="day-number">{}</span>
<div class="day-events">{}</div>
</div>"##,
                    today_class, current, current, day_num, events_dots
                ));

                current = current.succ_opt().unwrap_or(current);
            }

            let prev_month = if month == 1 { 12 } else { month - 1 };
            let prev_year = if month == 1 { year - 1 } else { year };
            let next_month = if month == 12 { 1 } else { month + 1 };
            let next_year = if month == 12 { year + 1 } else { year };

            Html(format!(
                r##"<div class="calendar-month">
<div class="month-header">
<button class="btn-icon" hx-get="/api/ui/calendar/month?year={}&month={}" hx-target="#calendar-view">&lt;</button>
<h3>{}</h3>
<button class="btn-icon" hx-get="/api/ui/calendar/month?year={}&month={}" hx-target="#calendar-view">&gt;</button>
</div>
<div class="weekdays">
<div>Sun</div><div>Mon</div><div>Tue</div><div>Wed</div><div>Thu</div><div>Fri</div><div>Sat</div>
</div>
<div class="days-grid">{}</div>
</div>"##,
                prev_year, prev_month, month_name, next_year, next_month, days_html
            ))
        }
        None => Html(
            r##"<div class="empty-state">
<p>Could not load calendar</p>
</div>"##
            .to_string(),
        ),
    }
}

#[derive(Debug, Deserialize)]
pub struct DayQuery {
    pub date: NaiveDate,
}

pub async fn ui_day_events(
    State(state): State<Arc<DbPool>>,
    Query(query): Query<DayQuery>,
) -> Html<String> {
    let pool = state.clone();
    let date = query.date;

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;

        let start = date.and_hms_opt(0, 0, 0)?;
        let end = date.and_hms_opt(23, 59, 59)?;

        calendar_events::table
            .filter(calendar_events::start_time.ge(start))
            .filter(calendar_events::start_time.le(end))
            .order(calendar_events::start_time.asc())
            .select((
                calendar_events::id,
                calendar_events::title,
                calendar_events::start_time,
                calendar_events::end_time,
                calendar_events::color,
                calendar_events::all_day,
            ))
            .load::<(Uuid, String, DateTime<Utc>, DateTime<Utc>, Option<String>, bool)>(
                &mut conn,
            )
            .ok()
    })
    .await
    .ok()
    .flatten();

    let date_str = date.format("%A, %B %d, %Y").to_string();

    match result {
        Some(events) if !events.is_empty() => {
            let items: String = events
                .iter()
                .map(|(id, title, start, end, color, all_day)| {
                    let event_color = color.clone().unwrap_or_else(|| "#3b82f6".to_string());
                    let time_str = if *all_day {
                        "All day".to_string()
                    } else {
                        format!("{} - {}", start.format("%H:%M"), end.format("%H:%M"))
                    };

                    format!(
                        r##"<div class="day-event-item" style="border-left: 4px solid {};"
hx-get="/api/ui/calendar/events/{}" hx-target="#event-detail">
<span class="event-time">{}</span>
<span class="event-title">{}</span>
</div>"##,
                        event_color, id, time_str, title
                    )
                })
                .collect();

            Html(format!(
                r##"<div class="day-events-panel">
<h4>{}</h4>
<div class="events-list">{}</div>
</div>"##,
                date_str, items
            ))
        }
        _ => Html(format!(
            r##"<div class="day-events-panel">
<h4>{}</h4>
<div class="empty-state">
<p>No events on this day</p>
</div>
</div>"##,
            date_str
        )),
    }
}

pub async fn ui_new_event_form() -> Html<String> {
    let now = Utc::now();
    let date = now.format("%Y-%m-%d").to_string();
    let time = now.format("%H:00").to_string();
    let end_time = (now + Duration::hours(1)).format("%H:00").to_string();

    Html(format!(
        r##"<div class="modal-header">
<h3>New Event</h3>
<button class="btn-close" onclick="closeModal()">&times;</button>
</div>
<form class="event-form" hx-post="/api/calendar/events" hx-swap="none" hx-on::after-request="closeModal(); htmx.trigger('#calendar-view', 'refresh')">
<div class="form-group">
<label>Title</label>
<input type="text" name="title" placeholder="Event title" required />
</div>
<div class="form-row">
<div class="form-group">
<label>Date</label>
<input type="date" name="date" value="{}" required />
</div>
<div class="form-group">
<label>All Day</label>
<input type="checkbox" name="all_day" onchange="toggleTimeInputs(this)" />
</div>
</div>
<div class="form-row time-inputs">
<div class="form-group">
<label>Start Time</label>
<input type="time" name="start_time" value="{}" />
</div>
<div class="form-group">
<label>End Time</label>
<input type="time" name="end_time" value="{}" />
</div>
</div>
<div class="form-group">
<label>Location</label>
<input type="text" name="location" placeholder="Add location" />
</div>
<div class="form-group">
<label>Description</label>
<textarea name="description" rows="3" placeholder="Add description"></textarea>
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
<button type="button" class="btn btn-secondary" onclick="closeModal()">Cancel</button>
<button type="submit" class="btn btn-primary">Create Event</button>
</div>
</form>"##,
        date, time, end_time
    ))
}

pub async fn ui_new_calendar_form() -> Html<String> {
    Html(
        r##"<div class="modal-header">
<h3>New Calendar</h3>
<button class="btn-close" onclick="closeModal()">&times;</button>
</div>
<form class="calendar-form" hx-post="/api/calendar/calendars" hx-swap="none" hx-on::after-request="closeModal(); htmx.trigger('#calendars-sidebar', 'refresh')">
<div class="form-group">
<label>Calendar Name</label>
<input type="text" name="name" placeholder="My Calendar" required />
</div>
<div class="form-group">
<label>Description</label>
<textarea name="description" rows="2" placeholder="Calendar description"></textarea>
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
<div class="form-group">
<label>Timezone</label>
<select name="timezone">
<option value="UTC">UTC</option>
<option value="America/New_York">Eastern Time</option>
<option value="America/Chicago">Central Time</option>
<option value="America/Denver">Mountain Time</option>
<option value="America/Los_Angeles">Pacific Time</option>
<option value="Europe/London">London</option>
<option value="Europe/Paris">Paris</option>
<option value="Asia/Tokyo">Tokyo</option>
</select>
</div>
<div class="form-actions">
<button type="button" class="btn btn-secondary" onclick="closeModal()">Cancel</button>
<button type="submit" class="btn btn-primary">Create Calendar</button>
</div>
</form>"##
        .to_string(),
    )
}

pub fn configure_calendar_ui_routes() -> Router<Arc<DbPool>> {
    Router::new()
        .route("/api/ui/calendar/events", get(ui_events_list))
        .route("/api/ui/calendar/events/count", get(ui_events_count))
        .route("/api/ui/calendar/events/today", get(ui_today_events_count))
        .route("/api/ui/calendar/events/:id", get(ui_event_detail))
        .route("/api/ui/calendar/calendars", get(ui_calendars_sidebar))
        .route("/api/ui/calendar/upcoming", get(ui_upcoming_events))
        .route("/api/ui/calendar/month", get(ui_month_view))
        .route("/api/ui/calendar/day", get(ui_day_events))
        .route("/api/ui/calendar/new-event", get(ui_new_event_form))
        .route("/api/ui/calendar/new-calendar", get(ui_new_calendar_form))
}

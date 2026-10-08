use super::*;

pub(crate) const API_CALENDAR_EVENTS: &str = "/api/calendar/events";
// The router runs on axum 0.7, whose path parameters are written `:name`.
// `{name}` is axum 0.8 syntax and is treated as a literal segment here, so a
// route declared that way never matches a real identifier — the request fell
// through to the desktop-shell fallback and answered `200 text/html`, which is
// how an unreachable endpoint stayed invisible. RBAC accepts both spellings, so
// only the router strings had to change.
pub(crate) const API_CALENDAR_EVENT_BY_ID: &str = "/api/calendar/events/:id";
pub(crate) const API_CALENDAR_UPCOMING_JSON: &str = "/api/calendar/events/upcoming";

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = calendars)]
pub struct CalendarRecord {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub branch_id: Uuid,
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
    pub branch_id: Uuid,
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


use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::{Bool, Float8, Int4, Int8, Nullable, Text, Timestamptz};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use botcore::shared::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentRoom {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub duration_minutes: i32,
    pub status: String,
    pub is_recording: bool,
    pub is_transcribing: bool,
    pub settings: MeetingSettingsJson,
    pub recording_url: Option<String>,
    pub transcription_url: Option<String>,
    pub ended_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct MeetingSettingsJson {
    pub enable_transcription: bool,
    pub enable_recording: bool,
    pub enable_chat: bool,
    pub enable_screen_share: bool,
    pub auto_admit: bool,
    pub waiting_room: bool,
    pub bot_enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct ScheduleMeetingRequest {
    pub title: String,
    pub description: Option<String>,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub duration_minutes: Option<i32>,
    pub created_by: Option<String>,
    pub settings: Option<MeetingSettingsJson>,
}

#[derive(Debug, Deserialize)]
pub struct DashboardQuery {
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct DashboardStats {
    pub total_meetings: i64,
    pub live_meetings: i64,
    pub scheduled_meetings: i64,
    pub completed_meetings: i64,
    pub total_participants: i64,
    pub total_recording_hours: f64,
}

#[derive(Debug, Serialize)]
pub struct MeetingListItem {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub duration_minutes: i32,
    pub created_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub is_recording: bool,
    pub participant_count: i64,
}

#[derive(QueryableByName, Debug)]
struct CountRow {
    #[diesel(sql_type = Int8)]
    count: i64,
}

#[derive(QueryableByName, Debug)]
struct HoursRow {
    #[diesel(sql_type = Nullable<Float8>)]
    count: Option<f64>,
}

#[derive(QueryableByName, Debug)]
struct MeetingRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: Uuid,
    #[diesel(sql_type = Text)]
    title: String,
    #[diesel(sql_type = Nullable<Text>)]
    description: Option<String>,
    #[diesel(sql_type = Text)]
    status: String,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    scheduled_at: Option<DateTime<Utc>>,
    #[diesel(sql_type = Int4)]
    duration_minutes: i32,
    #[diesel(sql_type = Timestamptz)]
    created_at: DateTime<Utc>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    ended_at: Option<DateTime<Utc>>,
    #[diesel(sql_type = Bool)]
    is_recording: bool,
}

pub async fn schedule_meeting(
    state: &AppState,
    request: ScheduleMeetingRequest,
) -> Result<PersistentRoom, String> {
    let room_id = Uuid::new_v4();
    let now = Utc::now();
    let created_by_uuid = request
        .created_by
        .and_then(|s| Uuid::parse_str(&s).ok())
        .unwrap_or(Uuid::nil());

    let settings = request.settings.unwrap_or_default();
    let settings_json = serde_json::to_value(&settings)
        .map_err(|e| format!("Failed to serialize settings: {e}"))?;

    let duration = request.duration_minutes.unwrap_or(60);

    let pool = state.conn.clone();
    let title = request.title.clone();
    let description = request.description.clone();
    let scheduled_at = request.scheduled_at;

    tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .map_err(|e| format!("Failed to get DB connection: {e}"))?;

        diesel::sql_query(
            "INSERT INTO meeting_rooms (id, organization_id, title, description, created_by, created_at, scheduled_at, duration_minutes, status, is_recording, is_transcribing, settings, metadata) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12::jsonb)"
        )
        .bind::<diesel::sql_types::Uuid, _>(room_id)
        .bind::<diesel::sql_types::Uuid, _>(Uuid::nil())
        .bind::<Text, _>(&title)
        .bind::<Nullable<Text>, _>(&description)
        .bind::<diesel::sql_types::Uuid, _>(created_by_uuid)
        .bind::<Timestamptz, _>(now)
        .bind::<Nullable<Timestamptz>, _>(scheduled_at)
        .bind::<Int4, _>(duration)
        .bind::<Text, _>("scheduled")
        .bind::<Bool, _>(false)
        .bind::<Bool, _>(false)
        .bind::<diesel::sql_types::Jsonb, _>(settings_json)
        .bind::<diesel::sql_types::Jsonb, _>(serde_json::json!({}))
        .execute(&mut conn)
        .map_err(|e| format!("Failed to insert meeting: {e}"))?;

        Ok(PersistentRoom {
            id: room_id,
            organization_id: Uuid::nil(),
            title,
            description,
            created_by: created_by_uuid,
            created_at: now,
            scheduled_at: request.scheduled_at,
            duration_minutes: duration,
            status: "scheduled".to_string(),
            is_recording: false,
            is_transcribing: settings.enable_transcription,
            settings,
            recording_url: None,
            transcription_url: None,
            ended_at: None,
        })
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

pub async fn get_dashboard_stats(state: &AppState) -> Result<DashboardStats, String> {
    let pool = state.conn.clone();

    tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .map_err(|e| format!("Failed to get DB connection: {e}"))?;

        let total: i64 = sql_query("SELECT COUNT(*)::bigint AS count FROM meeting_rooms")
            .get_result::<CountRow>(&mut conn)
            .map(|r| r.count)
            .unwrap_or(0);

        let live: i64 = sql_query("SELECT COUNT(*)::bigint AS count FROM meeting_rooms WHERE status = 'live'")
            .get_result::<CountRow>(&mut conn)
            .map(|r| r.count)
            .unwrap_or(0);

        let scheduled: i64 = sql_query("SELECT COUNT(*)::bigint AS count FROM meeting_rooms WHERE status = 'scheduled'")
            .get_result::<CountRow>(&mut conn)
            .map(|r| r.count)
            .unwrap_or(0);

        let completed: i64 = sql_query("SELECT COUNT(*)::bigint AS count FROM meeting_rooms WHERE status = 'ended'")
            .get_result::<CountRow>(&mut conn)
            .map(|r| r.count)
            .unwrap_or(0);

        let participants: i64 = sql_query("SELECT COUNT(*)::bigint AS count FROM meeting_participants WHERE left_at IS NULL")
            .get_result::<CountRow>(&mut conn)
            .map(|r| r.count)
            .unwrap_or(0);

        let recording_hours: f64 = sql_query("SELECT COALESCE(SUM(duration_seconds)::float8 / 3600.0, 0.0) AS count FROM meeting_recordings WHERE status IN ('ready', 'recording')")
            .get_result::<HoursRow>(&mut conn)
            .map(|r| r.count.unwrap_or(0.0))
            .unwrap_or(0.0);

        Ok(DashboardStats {
            total_meetings: total,
            live_meetings: live,
            scheduled_meetings: scheduled,
            completed_meetings: completed,
            total_participants: participants,
            total_recording_hours: recording_hours,
        })
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

pub async fn list_meetings(
    state: &AppState,
    query: DashboardQuery,
) -> Result<Vec<MeetingListItem>, String> {
    let pool = state.conn.clone();
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .map_err(|e| format!("Failed to get DB connection: {e}"))?;

        let rows: Vec<MeetingRow> = sql_query(
            "SELECT id, title, description, status, scheduled_at, duration_minutes, created_at, ended_at, is_recording FROM meeting_rooms ORDER BY created_at DESC LIMIT $1 OFFSET $2"
        )
        .bind::<Int8, _>(limit)
        .bind::<Int8, _>(offset)
        .get_results(&mut conn)
        .map_err(|e| format!("Failed to list meetings: {e}"))?;

        let items = rows
            .into_iter()
            .map(|row| MeetingListItem {
                id: row.id,
                title: row.title,
                description: row.description,
                status: row.status,
                scheduled_at: row.scheduled_at,
                duration_minutes: row.duration_minutes,
                created_at: row.created_at,
                ended_at: row.ended_at,
                is_recording: row.is_recording,
                participant_count: 0,
            })
            .collect();

        Ok(items)
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

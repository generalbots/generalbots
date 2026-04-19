pub mod ui;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, post, put},
    Json, Router,
};

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::core::bot::get_default_bot;
use crate::core::shared::schema::{
    attendant_agent_status, attendant_canned_responses, attendant_queue_agents, attendant_queues,
    attendant_session_messages, attendant_sessions, attendant_tags, attendant_transfers,
    attendant_wrap_up_codes,
};
use crate::core::shared::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = attendant_queues)]
pub struct AttendantQueue {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub priority: i32,
    pub max_wait_minutes: i32,
    pub auto_assign: bool,
    pub working_hours: serde_json::Value,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = attendant_sessions)]
pub struct AttendantSession {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub session_number: String,
    pub channel: String,
    pub customer_id: Option<Uuid>,
    pub customer_name: Option<String>,
    pub customer_email: Option<String>,
    pub customer_phone: Option<String>,
    pub status: String,
    pub priority: i32,
    pub agent_id: Option<Uuid>,
    pub queue_id: Option<Uuid>,
    pub subject: Option<String>,
    pub initial_message: Option<String>,
    pub started_at: DateTime<Utc>,
    pub assigned_at: Option<DateTime<Utc>>,
    pub first_response_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub wait_time_seconds: Option<i32>,
    pub handle_time_seconds: Option<i32>,
    pub satisfaction_rating: Option<i32>,
    pub satisfaction_comment: Option<String>,
    pub tags: Vec<String>,
    pub metadata: serde_json::Value,
    pub notes: Option<String>,
    pub transfer_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = attendant_session_messages)]
pub struct SessionMessage {
    pub id: Uuid,
    pub session_id: Uuid,
    pub sender_type: String,
    pub sender_id: Option<Uuid>,
    pub sender_name: Option<String>,
    pub content: String,
    pub content_type: String,
    pub attachments: serde_json::Value,
    pub is_internal: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = attendant_queue_agents)]
pub struct QueueAgent {
    pub id: Uuid,
    pub queue_id: Uuid,
    pub agent_id: Uuid,
    pub max_concurrent: i32,
    pub priority: i32,
    pub skills: Vec<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = attendant_agent_status)]
pub struct AgentStatus {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub agent_id: Uuid,
    pub status: String,
    pub status_message: Option<String>,
    pub current_sessions: i32,
    pub max_sessions: i32,
    pub last_activity_at: DateTime<Utc>,
    pub break_started_at: Option<DateTime<Utc>>,
    pub break_reason: Option<String>,
    pub available_since: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = attendant_transfers)]
pub struct SessionTransfer {
    pub id: Uuid,
    pub session_id: Uuid,
    pub from_agent_id: Option<Uuid>,
    pub to_agent_id: Option<Uuid>,
    pub to_queue_id: Option<Uuid>,
    pub reason: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = attendant_canned_responses)]
pub struct CannedResponse {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub title: String,
    pub content: String,
    pub shortcut: Option<String>,
    pub category: Option<String>,
    pub queue_id: Option<Uuid>,
    pub is_active: bool,
    pub usage_count: i32,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = attendant_tags)]
pub struct AttendantTag {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub name: String,
    pub color: Option<String>,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = attendant_wrap_up_codes)]
pub struct WrapUpCode {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub requires_notes: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateQueueRequest {
    pub name: String,
    pub description: Option<String>,
    pub priority: Option<i32>,
    pub max_wait_minutes: Option<i32>,
    pub auto_assign: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    pub channel: String,
    pub customer_name: Option<String>,
    pub customer_email: Option<String>,
    pub customer_phone: Option<String>,
    pub customer_id: Option<Uuid>,
    pub queue_id: Option<Uuid>,
    pub subject: Option<String>,
    pub initial_message: Option<String>,
    pub priority: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct AssignSessionRequest {
    pub agent_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct TransferSessionRequest {
    pub to_agent_id: Option<Uuid>,
    pub to_queue_id: Option<Uuid>,
    pub reason: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct EndSessionRequest {
    pub wrap_up_code: Option<String>,
    pub notes: Option<String>,
    pub follow_up_required: Option<bool>,
    pub follow_up_date: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RateSessionRequest {
    pub rating: i32,
    pub comment: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub content: String,
    pub content_type: Option<String>,
    pub is_internal: Option<bool>,
    pub sender_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAgentStatusRequest {
    pub status: String,
    pub status_message: Option<String>,
    pub break_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddQueueAgentRequest {
    pub agent_id: Uuid,
    pub max_concurrent: Option<i32>,
    pub priority: Option<i32>,
    pub skills: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCannedResponseRequest {
    pub title: String,
    pub content: String,
    pub shortcut: Option<String>,
    pub category: Option<String>,
    pub queue_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub status: Option<String>,
    pub queue_id: Option<Uuid>,
    pub agent_id: Option<Uuid>,
    pub channel: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct AttendantStats {
    pub total_sessions_today: i64,
    pub active_sessions: i64,
    pub waiting_sessions: i64,
    pub avg_wait_time_seconds: i64,
    pub avg_handle_time_seconds: i64,
    pub agents_online: i64,
    pub agents_on_break: i64,
    pub satisfaction_avg: f64,
}

#[derive(Debug, Serialize)]
pub struct SessionWithMessages {
    pub session: AttendantSession,
    pub messages: Vec<SessionMessage>,
}

#[derive(Debug, Serialize)]
pub struct QueueWithStats {
    pub queue: AttendantQueue,
    pub waiting_count: i64,
    pub active_count: i64,
    pub agents_count: i64,
}

fn get_bot_context(state: &AppState) -> (Uuid, Uuid) {
    let Ok(mut conn) = state.conn.get() else {
        return (Uuid::nil(), Uuid::nil());
    };
    let (bot_id, _bot_name) = get_default_bot(&mut conn);
    let org_id = Uuid::nil();
    (org_id, bot_id)
}

fn generate_session_number(conn: &mut diesel::PgConnection, org_id: Uuid) -> String {
    let count: i64 = attendant_sessions::table
        .filter(attendant_sessions::org_id.eq(org_id))
        .count()
        .get_result(conn)
        .unwrap_or(0);
    format!("SES-{:06}", count + 1)
}

pub async fn create_queue(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateQueueRequest>,
) -> Result<Json<AttendantQueue>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);
    let id = Uuid::new_v4();
    let now = Utc::now();

    let queue = AttendantQueue {
        id,
        org_id,
        bot_id,
        name: req.name,
        description: req.description,
        priority: req.priority.unwrap_or(0),
        max_wait_minutes: req.max_wait_minutes.unwrap_or(30),
        auto_assign: req.auto_assign.unwrap_or(true),
        working_hours: serde_json::json!({}),
        is_active: true,
        created_at: now,
        updated_at: now,
    };

    diesel::insert_into(attendant_queues::table)
        .values(&queue)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert error: {e}")))?;

    Ok(Json(queue))
}

pub async fn list_queues(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<AttendantQueue>>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);

    let queues: Vec<AttendantQueue> = attendant_queues::table
        .filter(attendant_queues::org_id.eq(org_id))
        .filter(attendant_queues::bot_id.eq(bot_id))
        .filter(attendant_queues::is_active.eq(true))
        .order(attendant_queues::priority.desc())
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    Ok(Json(queues))
}

pub async fn get_queue(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<QueueWithStats>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let queue: AttendantQueue = attendant_queues::table
        .filter(attendant_queues::id.eq(id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Queue not found".to_string()))?;

    let waiting_count: i64 = attendant_sessions::table
        .filter(attendant_sessions::queue_id.eq(id))
        .filter(attendant_sessions::status.eq("waiting"))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    let active_count: i64 = attendant_sessions::table
        .filter(attendant_sessions::queue_id.eq(id))
        .filter(attendant_sessions::status.eq("active"))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    let agents_count: i64 = attendant_queue_agents::table
        .filter(attendant_queue_agents::queue_id.eq(id))
        .filter(attendant_queue_agents::is_active.eq(true))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    Ok(Json(QueueWithStats {
        queue,
        waiting_count,
        active_count,
        agents_count,
    }))
}

pub async fn delete_queue(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    diesel::delete(attendant_queues::table.filter(attendant_queues::id.eq(id)))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Delete error: {e}")))?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn add_queue_agent(
    State(state): State<Arc<AppState>>,
    Path(queue_id): Path<Uuid>,
    Json(req): Json<AddQueueAgentRequest>,
) -> Result<Json<QueueAgent>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let id = Uuid::new_v4();
    let now = Utc::now();

    let agent = QueueAgent {
        id,
        queue_id,
        agent_id: req.agent_id,
        max_concurrent: req.max_concurrent.unwrap_or(3),
        priority: req.priority.unwrap_or(0),
        skills: req.skills.unwrap_or_default(),
        is_active: true,
        created_at: now,
    };

    diesel::insert_into(attendant_queue_agents::table)
        .values(&agent)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert error: {e}")))?;

    Ok(Json(agent))
}

pub async fn remove_queue_agent(
    State(state): State<Arc<AppState>>,
    Path((queue_id, agent_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    diesel::delete(
        attendant_queue_agents::table
            .filter(attendant_queue_agents::queue_id.eq(queue_id))
            .filter(attendant_queue_agents::agent_id.eq(agent_id)),
    )
    .execute(&mut conn)
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Delete error: {e}")))?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn create_session(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateSessionRequest>,
) -> Result<Json<AttendantSession>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);
    let id = Uuid::new_v4();
    let now = Utc::now();
    let session_number = generate_session_number(&mut conn, org_id);

    let session = AttendantSession {
        id,
        org_id,
        bot_id,
        session_number,
        channel: req.channel,
        customer_id: req.customer_id,
        customer_name: req.customer_name,
        customer_email: req.customer_email,
        customer_phone: req.customer_phone,
        status: "waiting".to_string(),
        priority: req.priority.unwrap_or(0),
        agent_id: None,
        queue_id: req.queue_id,
        subject: req.subject,
        initial_message: req.initial_message,
        started_at: now,
        assigned_at: None,
        first_response_at: None,
        ended_at: None,
        wait_time_seconds: None,
        handle_time_seconds: None,
        satisfaction_rating: None,
        satisfaction_comment: None,
        tags: vec![],
        metadata: serde_json::json!({}),
        notes: None,
        transfer_count: 0,
        created_at: now,
    };

    diesel::insert_into(attendant_sessions::table)
        .values(&session)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert error: {e}")))?;

    Ok(Json(session))
}

pub async fn list_sessions(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<AttendantSession>>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    let mut q = attendant_sessions::table
        .filter(attendant_sessions::org_id.eq(org_id))
        .filter(attendant_sessions::bot_id.eq(bot_id))
        .into_boxed();

    if let Some(status) = query.status {
        if status != "all" {
            q = q.filter(attendant_sessions::status.eq(status));
        }
    }

    if let Some(queue_id) = query.queue_id {
        q = q.filter(attendant_sessions::queue_id.eq(queue_id));
    }

    if let Some(agent_id) = query.agent_id {
        q = q.filter(attendant_sessions::agent_id.eq(agent_id));
    }

    if let Some(channel) = query.channel {
        q = q.filter(attendant_sessions::channel.eq(channel));
    }

    let sessions: Vec<AttendantSession> = q
        .order(attendant_sessions::created_at.desc())
        .limit(limit)
        .offset(offset)
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    Ok(Json(sessions))
}

pub async fn get_session(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<SessionWithMessages>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let session: AttendantSession = attendant_sessions::table
        .filter(attendant_sessions::id.eq(id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Session not found".to_string()))?;

    let messages: Vec<SessionMessage> = attendant_session_messages::table
        .filter(attendant_session_messages::session_id.eq(id))
        .order(attendant_session_messages::created_at.asc())
        .load(&mut conn)
        .unwrap_or_default();

    Ok(Json(SessionWithMessages { session, messages }))
}

pub async fn assign_session(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<AssignSessionRequest>,
) -> Result<Json<AttendantSession>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let now = Utc::now();

    let session: AttendantSession = attendant_sessions::table
        .filter(attendant_sessions::id.eq(id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Session not found".to_string()))?;

    let wait_time = (now - session.started_at).num_seconds() as i32;

    diesel::update(attendant_sessions::table.filter(attendant_sessions::id.eq(id)))
        .set((
            attendant_sessions::agent_id.eq(Some(req.agent_id)),
            attendant_sessions::status.eq("active"),
            attendant_sessions::assigned_at.eq(Some(now)),
            attendant_sessions::wait_time_seconds.eq(Some(wait_time)),
        ))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;

    let updated: AttendantSession = attendant_sessions::table
        .filter(attendant_sessions::id.eq(id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Session not found".to_string()))?;

    Ok(Json(updated))
}

pub async fn transfer_session(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<TransferSessionRequest>,
) -> Result<Json<AttendantSession>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let now = Utc::now();

    let session: AttendantSession = attendant_sessions::table
        .filter(attendant_sessions::id.eq(id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Session not found".to_string()))?;

    let transfer = SessionTransfer {
        id: Uuid::new_v4(),
        session_id: id,
        from_agent_id: session.agent_id,
        to_agent_id: req.to_agent_id,
        to_queue_id: req.to_queue_id,
        reason: req.reason,
        notes: req.notes,
        created_at: now,
    };

    diesel::insert_into(attendant_transfers::table)
        .values(&transfer)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert error: {e}")))?;

    if let Some(to_agent_id) = req.to_agent_id {
        diesel::update(attendant_sessions::table.filter(attendant_sessions::id.eq(id)))
            .set((
                attendant_sessions::agent_id.eq(Some(to_agent_id)),
                attendant_sessions::transfer_count.eq(session.transfer_count + 1),
            ))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    } else if let Some(to_queue_id) = req.to_queue_id {
        diesel::update(attendant_sessions::table.filter(attendant_sessions::id.eq(id)))
            .set((
                attendant_sessions::agent_id.eq(None::<Uuid>),
                attendant_sessions::queue_id.eq(Some(to_queue_id)),
                attendant_sessions::status.eq("waiting"),
                attendant_sessions::transfer_count.eq(session.transfer_count + 1),
            ))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    let updated: AttendantSession = attendant_sessions::table
        .filter(attendant_sessions::id.eq(id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Session not found".to_string()))?;

    Ok(Json(updated))
}

pub async fn end_session(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(_req): Json<EndSessionRequest>,
) -> Result<Json<AttendantSession>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let now = Utc::now();

    let session: AttendantSession = attendant_sessions::table
        .filter(attendant_sessions::id.eq(id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Session not found".to_string()))?;

    let handle_time = session.assigned_at.map(|assigned| {
        (now - assigned).num_seconds() as i32
    });

    diesel::update(attendant_sessions::table.filter(attendant_sessions::id.eq(id)))
        .set((
            attendant_sessions::status.eq("ended"),
            attendant_sessions::ended_at.eq(Some(now)),
            attendant_sessions::handle_time_seconds.eq(handle_time),
        ))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;

    let updated: AttendantSession = attendant_sessions::table
        .filter(attendant_sessions::id.eq(id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Session not found".to_string()))?;

    Ok(Json(updated))
}

pub async fn rate_session(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<RateSessionRequest>,
) -> Result<Json<AttendantSession>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    diesel::update(attendant_sessions::table.filter(attendant_sessions::id.eq(id)))
        .set((
            attendant_sessions::satisfaction_rating.eq(Some(req.rating)),
            attendant_sessions::satisfaction_comment.eq(req.comment),
        ))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;

    let session: AttendantSession = attendant_sessions::table
        .filter(attendant_sessions::id.eq(id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Session not found".to_string()))?;

    Ok(Json(session))
}

pub async fn send_message(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<Uuid>,
    Json(req): Json<SendMessageRequest>,
) -> Result<Json<SessionMessage>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let id = Uuid::new_v4();
    let now = Utc::now();

    let message = SessionMessage {
        id,
        session_id,
        sender_type: "agent".to_string(),
        sender_id: None,
        sender_name: req.sender_name,
        content: req.content,
        content_type: req.content_type.unwrap_or_else(|| "text".to_string()),
        attachments: serde_json::json!([]),
        is_internal: req.is_internal.unwrap_or(false),
        created_at: now,
    };

    diesel::insert_into(attendant_session_messages::table)
        .values(&message)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert error: {e}")))?;

    let session: AttendantSession = attendant_sessions::table
        .filter(attendant_sessions::id.eq(session_id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Session not found".to_string()))?;

    if session.first_response_at.is_none() && !message.is_internal {
        diesel::update(attendant_sessions::table.filter(attendant_sessions::id.eq(session_id)))
            .set(attendant_sessions::first_response_at.eq(Some(now)))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    Ok(Json(message))
}

pub async fn update_agent_status(
    State(state): State<Arc<AppState>>,
    Path(agent_id): Path<Uuid>,
    Json(req): Json<UpdateAgentStatusRequest>,
) -> Result<Json<AgentStatus>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);
    let now = Utc::now();

    let existing: Option<AgentStatus> = attendant_agent_status::table
        .filter(attendant_agent_status::org_id.eq(org_id))
        .filter(attendant_agent_status::agent_id.eq(agent_id))
        .first(&mut conn)
        .ok();

    let break_started = if req.status == "break" {
        Some(now)
    } else {
        None
    };

    let available_since = if req.status == "online" {
        Some(now)
    } else {
        None
    };

    if let Some(_) = existing {
        diesel::update(
            attendant_agent_status::table
                .filter(attendant_agent_status::org_id.eq(org_id))
                .filter(attendant_agent_status::agent_id.eq(agent_id)),
        )
        .set((
            attendant_agent_status::status.eq(&req.status),
            attendant_agent_status::status_message.eq(&req.status_message),
            attendant_agent_status::break_started_at.eq(break_started),
            attendant_agent_status::break_reason.eq(&req.break_reason),
            attendant_agent_status::available_since.eq(available_since),
            attendant_agent_status::last_activity_at.eq(now),
            attendant_agent_status::updated_at.eq(now),
        ))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    } else {
        let status = AgentStatus {
            id: Uuid::new_v4(),
            org_id,
            bot_id,
            agent_id,
            status: req.status.clone(),
            status_message: req.status_message.clone(),
            current_sessions: 0,
            max_sessions: 5,
            last_activity_at: now,
            break_started_at: break_started,
            break_reason: req.break_reason.clone(),
            available_since,
            created_at: now,
            updated_at: now,
        };

        diesel::insert_into(attendant_agent_status::table)
            .values(&status)
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert error: {e}")))?;
    }

    let agent_status: AgentStatus = attendant_agent_status::table
        .filter(attendant_agent_status::org_id.eq(org_id))
        .filter(attendant_agent_status::agent_id.eq(agent_id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Agent status not found".to_string()))?;

    Ok(Json(agent_status))
}

pub async fn list_agent_statuses(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<AgentStatus>>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);

    let statuses: Vec<AgentStatus> = attendant_agent_status::table
        .filter(attendant_agent_status::org_id.eq(org_id))
        .filter(attendant_agent_status::bot_id.eq(bot_id))
        .order(attendant_agent_status::status.asc())
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    Ok(Json(statuses))
}

pub async fn list_canned_responses(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<CannedResponse>>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);

    let responses: Vec<CannedResponse> = attendant_canned_responses::table
        .filter(attendant_canned_responses::org_id.eq(org_id))
        .filter(attendant_canned_responses::bot_id.eq(bot_id))
        .filter(attendant_canned_responses::is_active.eq(true))
        .order(attendant_canned_responses::title.asc())
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    Ok(Json(responses))
}

pub async fn create_canned_response(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateCannedResponseRequest>,
) -> Result<Json<CannedResponse>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);
    let id = Uuid::new_v4();
    let now = Utc::now();

    let response = CannedResponse {
        id,
        org_id,
        bot_id,
        title: req.title,
        content: req.content,
        shortcut: req.shortcut,
        category: req.category,
        queue_id: req.queue_id,
        is_active: true,
        usage_count: 0,
        created_by: None,
        created_at: now,
        updated_at: now,
    };

    diesel::insert_into(attendant_canned_responses::table)
        .values(&response)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert error: {e}")))?;

    Ok(Json(response))
}

pub async fn list_tags(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<AttendantTag>>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);

    let tags: Vec<AttendantTag> = attendant_tags::table
        .filter(attendant_tags::org_id.eq(org_id))
        .filter(attendant_tags::bot_id.eq(bot_id))
        .filter(attendant_tags::is_active.eq(true))
        .order(attendant_tags::name.asc())
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    Ok(Json(tags))
}

pub async fn list_wrap_up_codes(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<WrapUpCode>>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);

    let codes: Vec<WrapUpCode> = attendant_wrap_up_codes::table
        .filter(attendant_wrap_up_codes::org_id.eq(org_id))
        .filter(attendant_wrap_up_codes::bot_id.eq(bot_id))
        .filter(attendant_wrap_up_codes::is_active.eq(true))
        .order(attendant_wrap_up_codes::name.asc())
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    Ok(Json(codes))
}

pub async fn get_attendant_stats(
    State(state): State<Arc<AppState>>,
) -> Result<Json<AttendantStats>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, bot_id) = get_bot_context(&state);
    let today = Utc::now().date_naive().and_hms_opt(0, 0, 0).unwrap_or_else(|| {
        // Fallback to midnight (0,0,0 should always be valid)
        chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap_or_else(|| chrono::NaiveTime::MIN)
    });
    let today_utc = DateTime::<Utc>::from_naive_utc_and_offset(today, Utc);

    let total_sessions_today: i64 = attendant_sessions::table
        .filter(attendant_sessions::org_id.eq(org_id))
        .filter(attendant_sessions::bot_id.eq(bot_id))
        .filter(attendant_sessions::created_at.ge(today_utc))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    let active_sessions: i64 = attendant_sessions::table
        .filter(attendant_sessions::org_id.eq(org_id))
        .filter(attendant_sessions::bot_id.eq(bot_id))
        .filter(attendant_sessions::status.eq("active"))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    let waiting_sessions: i64 = attendant_sessions::table
        .filter(attendant_sessions::org_id.eq(org_id))
        .filter(attendant_sessions::bot_id.eq(bot_id))
        .filter(attendant_sessions::status.eq("waiting"))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    let agents_online: i64 = attendant_agent_status::table
        .filter(attendant_agent_status::org_id.eq(org_id))
        .filter(attendant_agent_status::bot_id.eq(bot_id))
        .filter(attendant_agent_status::status.eq("online"))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    let agents_on_break: i64 = attendant_agent_status::table
        .filter(attendant_agent_status::org_id.eq(org_id))
        .filter(attendant_agent_status::bot_id.eq(bot_id))
        .filter(attendant_agent_status::status.eq("break"))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);

    let stats = AttendantStats {
        total_sessions_today,
        active_sessions,
        waiting_sessions,
        avg_wait_time_seconds: 0,
        avg_handle_time_seconds: 0,
        agents_online,
        agents_on_break,
        satisfaction_avg: 0.0,
    };

    Ok(Json(stats))
}

pub fn configure_attendant_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/attendant/queues", get(list_queues).post(create_queue))
        .route("/api/attendant/queues/:id", get(get_queue).delete(delete_queue))
        .route("/api/attendant/queues/:id/agents", post(add_queue_agent))
        .route("/api/attendant/queues/:queue_id/agents/:agent_id", delete(remove_queue_agent))
        .route("/api/attendant/sessions", get(list_sessions).post(create_session))
        .route("/api/attendant/sessions/:id", get(get_session))
        .route("/api/attendant/sessions/:id/assign", put(assign_session))
        .route("/api/attendant/sessions/:id/transfer", put(transfer_session))
        .route("/api/attendant/sessions/:id/end", put(end_session))
        .route("/api/attendant/sessions/:id/rate", put(rate_session))
        .route("/api/attendant/sessions/:id/messages", post(send_message))
        .route("/api/attendant/agents", get(list_agent_statuses))
        .route("/api/attendant/agents/:id/status", put(update_agent_status))
        .route("/api/attendant/canned", get(list_canned_responses).post(create_canned_response))
        .route("/api/attendant/tags", get(list_tags))
        .route("/api/attendant/wrap-up-codes", get(list_wrap_up_codes))
        .route("/api/attendant/stats", get(get_attendant_stats))
}

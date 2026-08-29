use axum::extract::{Json, Path};
use axum::http::{HeaderMap, StatusCode};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use diesel::RunQueryDsl;
use diesel::OptionalExtension;

use crate::db;
use crate::storage::ensure_schema_sync;

use botcore::shared::tenant::branch_from_claims;

fn resolve_branch(headers: &HeaderMap) -> Uuid {
    branch_from_claims(headers).unwrap_or_else(Uuid::nil)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueEntry {
    pub id: Uuid,
    pub session_id: Uuid,
    pub user_name: String,
    pub channel: String,
    pub priority: String,
    pub waiting_since: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffAnalytics {
    pub period: String,
    pub total_transfers: i64,
    pub avg_wait_seconds: i64,
    pub avg_handle_seconds: i64,
    pub satisfaction_avg: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: Uuid,
    pub name: String,
    pub kind: String,
    pub status: String,
    pub active_agents: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsatEntry {
    pub id: Uuid,
    pub session_id: Uuid,
    pub rating: i32,
    pub comment: Option<String>,
    pub submitted_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct TransferRequest {
    pub agent_id: Uuid,
    pub notes: Option<String>,
}

pub async fn list_queue(headers: HeaderMap) -> Result<Json<Vec<QueueEntry>>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Uuid)] session_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)] user_name: String,
        #[diesel(sql_type = diesel::sql_types::Text)] channel: String,
        #[diesel(sql_type = diesel::sql_types::Text)] priority: String,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)] waiting_since: chrono::DateTime<Utc>,
    }
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT id, session_id, user_name, channel, priority, waiting_since
         FROM handoff_queue WHERE branch_id = $1 ORDER BY priority DESC, waiting_since ASC LIMIT 500",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .load(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Json(rows.into_iter().map(|r| QueueEntry {
        id: r.id, session_id: r.session_id, user_name: r.user_name, channel: r.channel,
        priority: r.priority, waiting_since: r.waiting_since,
    }).collect()))
}

pub async fn transfer_item(
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(req): Json<TransferRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let branch = resolve_branch(&headers);
    let parsed = Uuid::parse_str(&id)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid queue id '{id}': {e}")))?;
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)] session_id: Uuid,
    }
    let entry: Option<Uuid> = diesel::sql_query("SELECT session_id FROM handoff_queue WHERE id = $1 AND branch_id = $2")
        .bind::<diesel::sql_types::Uuid, _>(parsed)
        .bind::<diesel::sql_types::Uuid, _>(branch)
        .get_result::<Row>(&mut conn)
        .optional()
        .map_err(db::map_diesel_err)?
        .map(|r| r.session_id);
    let session_id = entry.ok_or((StatusCode::NOT_FOUND, format!("Queue entry {id} not found")))?;
    let n = diesel::sql_query("DELETE FROM handoff_queue WHERE id = $1 AND branch_id = $2")
        .bind::<diesel::sql_types::Uuid, _>(parsed)
        .bind::<diesel::sql_types::Uuid, _>(branch)
        .execute(&mut conn)
        .map_err(db::map_diesel_err)?;
    if n == 0 { return Err((StatusCode::NOT_FOUND, format!("Queue entry {id} not found"))); }
    Ok(Json(serde_json::json!({
        "transferred": true,
        "session_id": session_id,
        "agent_id": req.agent_id,
        "notes": req.notes
    })))
}

pub async fn get_analytics(headers: HeaderMap) -> Result<Json<Vec<HandoffAnalytics>>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Text)] period: String,
        #[diesel(sql_type = diesel::sql_types::BigInt)] total_transfers: i64,
        #[diesel(sql_type = diesel::sql_types::BigInt)] avg_wait_seconds: i64,
        #[diesel(sql_type = diesel::sql_types::BigInt)] avg_handle_seconds: i64,
        #[diesel(sql_type = diesel::sql_types::Numeric)] satisfaction_avg: rust_decimal::Decimal,
    }
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT to_char(date_trunc('day', a.started_at), 'YYYY-MM-DD') AS period,
                COUNT(*)::BIGINT AS total_transfers,
                COALESCE(AVG(EXTRACT(EPOCH FROM (a.ended_at - a.started_at)))::BIGINT, 0) AS avg_wait_seconds,
                COALESCE(AVG(a.duration_seconds), 0)::BIGINT AS avg_handle_seconds,
                COALESCE((SELECT AVG(r.rating) FROM conversation_ratings r WHERE r.branch_id = $1), 0)::NUMERIC AS satisfaction_avg
         FROM conversation_analytics a
         WHERE a.branch_id = $1
         GROUP BY date_trunc('day', a.started_at)
         ORDER BY period DESC LIMIT 100",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .load(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Json(rows.into_iter().map(|r| HandoffAnalytics {
        period: r.period, total_transfers: r.total_transfers,
        avg_wait_seconds: r.avg_wait_seconds, avg_handle_seconds: r.avg_handle_seconds,
        satisfaction_avg: r.satisfaction_avg.to_string(),
    }).collect()))
}

pub async fn list_channels(headers: HeaderMap) -> Result<Json<Vec<Channel>>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)] name: String,
        #[diesel(sql_type = diesel::sql_types::Text)] kind: String,
        #[diesel(sql_type = diesel::sql_types::Text)] status: String,
        #[diesel(sql_type = diesel::sql_types::BigInt)] active_agents: i64,
    }
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT id, name, kind, status, active_agents FROM handoff_channels WHERE branch_id = $1 ORDER BY name ASC LIMIT 500",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .load(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Json(rows.into_iter().map(|r| Channel {
        id: r.id, name: r.name, kind: r.kind, status: r.status, active_agents: r.active_agents,
    }).collect()))
}

pub async fn list_csat(headers: HeaderMap) -> Result<Json<Vec<CsatEntry>>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Uuid)] session_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Integer)] rating: i32,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)] comment: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)] submitted_at: chrono::DateTime<Utc>,
    }
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT id, session_id, rating, comment, submitted_at FROM conversation_ratings
         WHERE branch_id = $1 ORDER BY submitted_at DESC LIMIT 500",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .load(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Json(rows.into_iter().map(|r| CsatEntry {
        id: r.id, session_id: r.session_id, rating: r.rating, comment: r.comment, submitted_at: r.submitted_at,
    }).collect()))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub status: String,
    pub active_chats: i64,
    pub handled_today: i64,
    pub avg_handling_seconds: i64,
    pub csat: String,
    pub skills: String,
}

pub async fn list_agents(headers: HeaderMap) -> Result<Json<Vec<Agent>>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)] name: String,
        #[diesel(sql_type = diesel::sql_types::Text)] email: String,
        #[diesel(sql_type = diesel::sql_types::Text)] status: String,
        #[diesel(sql_type = diesel::sql_types::BigInt)] active_chats: i64,
        #[diesel(sql_type = diesel::sql_types::BigInt)] handled_today: i64,
        #[diesel(sql_type = diesel::sql_types::BigInt)] avg_handling_seconds: i64,
        #[diesel(sql_type = diesel::sql_types::Numeric)] csat: rust_decimal::Decimal,
        #[diesel(sql_type = diesel::sql_types::Text)] skills: String,
    }
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT id, name, email, status, active_chats, handled_today, avg_handling_seconds, csat, skills
         FROM handoff_agents WHERE branch_id = $1 ORDER BY name ASC LIMIT 500",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .load(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Json(rows.into_iter().map(|r| Agent {
        id: r.id, name: r.name, email: r.email, status: r.status,
        active_chats: r.active_chats, handled_today: r.handled_today,
        avg_handling_seconds: r.avg_handling_seconds, csat: r.csat.to_string(), skills: r.skills,
    }).collect()))
}

pub async fn create_agent(headers: HeaderMap, Json(item): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    let id = Uuid::new_v4();
    let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let email = item.get("email").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let skills = item.get("skills").and_then(|v| v.as_str()).unwrap_or("").to_string();
    diesel::sql_query(
        "INSERT INTO handoff_agents (id, name, email, status, active_chats, handled_today, avg_handling_seconds, csat, skills, branch_id)
         VALUES ($1, $2, $3, 'available', 0, 0, 0, 0, $4, $5)",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .bind::<diesel::sql_types::Text, _>(&name)
    .bind::<diesel::sql_types::Text, _>(&email)
    .bind::<diesel::sql_types::Text, _>(&skills)
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Json(serde_json::json!({"item": {"id": id, "name": name, "email": email, "status": "available", "skills": skills}})))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transcript {
    pub id: Uuid,
    pub customer: String,
    pub agent: String,
    pub channel: String,
    pub duration_seconds: i64,
    pub messages: i64,
    pub outcome: String,
    pub created_at: chrono::DateTime<Utc>,
}

pub async fn list_transcripts(headers: HeaderMap) -> Result<Json<Vec<Transcript>>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)] customer: String,
        #[diesel(sql_type = diesel::sql_types::Text)] agent: String,
        #[diesel(sql_type = diesel::sql_types::Text)] channel: String,
        #[diesel(sql_type = diesel::sql_types::BigInt)] duration_seconds: i64,
        #[diesel(sql_type = diesel::sql_types::BigInt)] messages: i64,
        #[diesel(sql_type = diesel::sql_types::Text)] outcome: String,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)] created_at: chrono::DateTime<Utc>,
    }
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT id, customer, agent, channel, duration_seconds, messages, outcome, created_at
         FROM handoff_transcripts WHERE branch_id = $1 ORDER BY created_at DESC LIMIT 500",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .load(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Json(rows.into_iter().map(|r| Transcript {
        id: r.id, customer: r.customer, agent: r.agent, channel: r.channel,
        duration_seconds: r.duration_seconds, messages: r.messages,
        outcome: r.outcome, created_at: r.created_at,
    }).collect()))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaReport {
    pub first_response_pct: i64,
    pub resolution_pct: i64,
    pub csat_pct: i64,
    pub breaches: Vec<serde_json::Value>,
}

pub async fn get_sla(headers: HeaderMap) -> Result<Json<SlaReport>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;

    #[derive(diesel::QueryableByName)]
    struct PctRow { #[diesel(sql_type = diesel::sql_types::BigInt)] c: i64 }
    #[derive(diesel::QueryableByName)]
    struct TotRow { #[diesel(sql_type = diesel::sql_types::BigInt)] c: i64 }

    let first_ok: i64 = diesel::sql_query(
        "SELECT COUNT(*) AS c FROM conversation_analytics WHERE branch_id = $1 AND avg_wait_seconds <= 30",
    ).bind::<diesel::sql_types::Uuid, _>(branch).get_result::<PctRow>(&mut conn).map_err(db::map_diesel_err)?.c;
    let first_total: i64 = diesel::sql_query(
        "SELECT COUNT(*) AS c FROM conversation_analytics WHERE branch_id = $1",
    ).bind::<diesel::sql_types::Uuid, _>(branch).get_result::<TotRow>(&mut conn).map_err(db::map_diesel_err)?.c;
    let first_response_pct = if first_total > 0 { first_ok * 100 / first_total } else { 0 };

    let res_ok: i64 = diesel::sql_query(
        "SELECT COUNT(*) AS c FROM conversation_analytics WHERE branch_id = $1 AND avg_handle_seconds <= 14400",
    ).bind::<diesel::sql_types::Uuid, _>(branch).get_result::<PctRow>(&mut conn).map_err(db::map_diesel_err)?.c;
    let resolution_pct = if first_total > 0 { res_ok * 100 / first_total } else { 0 };

    let csat_ok: i64 = diesel::sql_query(
        "SELECT COUNT(*) AS c FROM conversation_ratings WHERE branch_id = $1 AND rating >= 4",
    ).bind::<diesel::sql_types::Uuid, _>(branch).get_result::<PctRow>(&mut conn).map_err(db::map_diesel_err)?.c;
    let csat_total: i64 = diesel::sql_query(
        "SELECT COUNT(*) AS c FROM conversation_ratings WHERE branch_id = $1",
    ).bind::<diesel::sql_types::Uuid, _>(branch).get_result::<TotRow>(&mut conn).map_err(db::map_diesel_err)?.c;
    let csat_pct = if csat_total > 0 { csat_ok * 100 / csat_total } else { 0 };

    #[derive(diesel::QueryableByName)]
    struct BreachRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)] user_name: String,
        #[diesel(sql_type = diesel::sql_types::Text)] channel: String,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)] waiting_since: chrono::DateTime<Utc>,
    }
    let breaches: Vec<serde_json::Value> = diesel::sql_query(
        "SELECT id, user_name, channel, waiting_since FROM handoff_queue
         WHERE branch_id = $1 AND waiting_since < NOW() - INTERVAL '30 seconds'
         ORDER BY waiting_since ASC LIMIT 100",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .load::<BreachRow>(&mut conn)
    .map_err(db::map_diesel_err)?
    .into_iter()
    .map(|b| serde_json::json!({
        "id": b.id, "user_name": b.user_name, "channel": b.channel,
        "waiting_since": b.waiting_since, "elapsed_seconds": (Utc::now() - b.waiting_since).num_seconds()
    }))
    .collect();

    Ok(Json(SlaReport { first_response_pct, resolution_pct, csat_pct, breaches }))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeflectionReason {
    pub reason: String,
    pub count: i64,
}

pub async fn list_deflection(headers: HeaderMap) -> Result<Json<Vec<DeflectionReason>>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Text)] reason: String,
        #[diesel(sql_type = diesel::sql_types::BigInt)] count: i64,
    }
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT COALESCE(reason, 'uncategorized') AS reason, COUNT(*) AS count
         FROM handoff_queue WHERE branch_id = $1 AND reason IS NOT NULL AND reason <> ''
         GROUP BY reason ORDER BY count DESC LIMIT 20",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .load(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Json(rows.into_iter().map(|r| DeflectionReason { reason: r.reason, count: r.count }).collect()))
}

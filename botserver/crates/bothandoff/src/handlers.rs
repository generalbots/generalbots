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
        "SELECT period, total_transfers, avg_wait_seconds, avg_handle_seconds, satisfaction_avg
         FROM conversation_analytics WHERE branch_id = $1 ORDER BY period DESC LIMIT 100",
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

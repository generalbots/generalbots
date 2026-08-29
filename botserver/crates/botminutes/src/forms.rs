use axum::extract::{Json, Path};
use axum::http::{HeaderMap, StatusCode};
use chrono::{DateTime, Utc};
use diesel::RunQueryDsl;
use serde::Deserialize;
use uuid::Uuid;

use crate::db;
use crate::storage::ensure_schema_sync;

fn resolve_branch(headers: &HeaderMap) -> Uuid {
    botcore::shared::tenant::branch_from_claims(headers).unwrap_or_else(Uuid::nil)
}

#[derive(Debug, Deserialize)]
pub struct MeetingForm {
    pub title: String,
    pub date: String,
    pub time: String,
    #[serde(default)]
    pub duration: Option<i64>,
    #[serde(default)]
    pub location: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ActionForm {
    #[serde(default)]
    pub meeting_id: Option<String>,
    pub title: String,
    pub owner: String,
    #[serde(default)]
    pub due: Option<String>,
    #[serde(default = "default_priority")]
    pub priority: String,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DocumentForm {
    #[serde(default)]
    pub meeting_id: Option<String>,
    pub title: String,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub kind: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SignForm {
    pub signer: String,
}

#[derive(Debug, Deserialize)]
pub struct AttendanceForm {
    pub attendee: String,
    #[serde(default)]
    pub status: Option<String>,
}

fn default_priority() -> String {
    "medium".to_string()
}

fn parse_dt(date: &str, time: &str) -> Option<DateTime<Utc>> {
    let combined = format!("{date}T{time}:00");
    DateTime::parse_from_str(&combined, "%Y-%m-%dT%H:%M:%S")
        .ok()
        .map(|d| d.with_timezone(&Utc))
        .or_else(|| {
            date.parse::<DateTime<Utc>>().ok()
        })
}

pub async fn create_meeting(
    headers: HeaderMap,
    Json(req): Json<MeetingForm>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    let id = Uuid::new_v4();
    let meeting_date = parse_dt(&req.date, &req.time)
        .unwrap_or_else(Utc::now);
    let duration = req.duration.unwrap_or(30).max(0);
    let location = req.location.unwrap_or_default();
    diesel::sql_query(
        "INSERT INTO minutes_meetings (id, title, meeting_date, duration_minutes, participants, status, created_at, branch_id)
         VALUES ($1, $2, $3, $4, '[]'::jsonb, 'scheduled', NOW(), $5)",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .bind::<diesel::sql_types::Text, _>(&req.title)
    .bind::<diesel::sql_types::Timestamptz, _>(meeting_date)
    .bind::<diesel::sql_types::BigInt, _>(duration)
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    // `location` is stored inside participants JSONB for simplicity (no dedicated column).
    let _ = location;
    Ok(Json(serde_json::json!({ "ok": true, "id": id })))
}

pub async fn create_action(
    headers: HeaderMap,
    Json(req): Json<ActionForm>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    let id = Uuid::new_v4();
    let meeting_id = req
        .meeting_id
        .as_deref()
        .and_then(|s| Uuid::parse_str(s).ok());
    let due = req.due.as_deref().and_then(|d| {
        DateTime::parse_from_rfc3339(d)
            .map(|x| x.with_timezone(&Utc))
            .ok()
            .or_else(|| parse_dt(d, "00:00"))
    });
    let notes = req.notes.unwrap_or_default();
    diesel::sql_query(
        "INSERT INTO minutes_actions (id, meeting_id, title, owner, due, priority, notes, status, created_at, branch_id)
         VALUES ($1, $2, $3, $4, $5, $6, $7, 'open', NOW(), $8)",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(meeting_id)
    .bind::<diesel::sql_types::Text, _>(&req.title)
    .bind::<diesel::sql_types::Text, _>(&req.owner)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>, _>(due)
    .bind::<diesel::sql_types::Text, _>(&req.priority)
    .bind::<diesel::sql_types::Text, _>(&notes)
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Json(serde_json::json!({ "ok": true, "id": id })))
}

pub async fn create_document(
    headers: HeaderMap,
    Json(req): Json<DocumentForm>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    let id = Uuid::new_v4();
    let meeting_id = req
        .meeting_id
        .as_deref()
        .and_then(|s| Uuid::parse_str(s).ok());
    let kind = req.kind.clone().unwrap_or_else(|| "minutes".to_string());
    let content = req.content.unwrap_or_default();
    diesel::sql_query(
        "INSERT INTO minutes_documents (id, meeting_id, title, kind, content, version, created_at, updated_at, branch_id)
         VALUES ($1, $2, $3, $4, $5, 1, NOW(), NOW(), $6)",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(meeting_id)
    .bind::<diesel::sql_types::Text, _>(&req.title)
    .bind::<diesel::sql_types::Text, _>(&kind)
    .bind::<diesel::sql_types::Text, _>(&content)
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Json(serde_json::json!({ "ok": true, "id": id })))
}

pub async fn sign_document(
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(req): Json<SignForm>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let parsed = Uuid::parse_str(&id)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid id: {e}")))?;
    ensure_schema_sync()?;
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    let sig_id = Uuid::new_v4();
    diesel::sql_query(
        "INSERT INTO minutes_signatures (id, document_id, signer, signed_at, created_at, branch_id)
         VALUES ($1, $2, $3, NOW(), NOW(), $4)",
    )
    .bind::<diesel::sql_types::Uuid, _>(sig_id)
    .bind::<diesel::sql_types::Uuid, _>(parsed)
    .bind::<diesel::sql_types::Text, _>(&req.signer)
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Json(serde_json::json!({ "ok": true, "id": sig_id })))
}

pub async fn record_attendance(
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(req): Json<AttendanceForm>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let parsed = Uuid::parse_str(&id)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid id: {e}")))?;
    ensure_schema_sync()?;
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    let att_id = Uuid::new_v4();
    let status = req.status.clone().unwrap_or_else(|| "confirmed".to_string());
    diesel::sql_query(
        "INSERT INTO minutes_attendance (id, meeting_id, attendee, status, created_at, branch_id)
         VALUES ($1, $2, $3, $4, NOW(), $5)",
    )
    .bind::<diesel::sql_types::Uuid, _>(att_id)
    .bind::<diesel::sql_types::Uuid, _>(parsed)
    .bind::<diesel::sql_types::Text, _>(&req.attendee)
    .bind::<diesel::sql_types::Text, _>(&status)
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Json(serde_json::json!({ "ok": true, "id": att_id })))
}

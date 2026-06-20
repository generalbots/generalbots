use axum::extract::{Json, Path};
use axum::http::StatusCode;
use chrono::Utc;
use diesel::RunQueryDsl;
use uuid::Uuid;

use crate::db;
use crate::storage::ensure_schema_sync;

pub async fn list_meetings() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)] title: String,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)] meeting_date: chrono::DateTime<Utc>,
        #[diesel(sql_type = diesel::sql_types::BigInt)] duration_minutes: i64,
        #[diesel(sql_type = diesel::sql_types::Jsonb)] participants: serde_json::Value,
        #[diesel(sql_type = diesel::sql_types::Text)] status: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)] transcript_id: Option<Uuid>,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)] created_at: chrono::DateTime<Utc>,
    }
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT id, title, meeting_date, duration_minutes, participants, status, transcript_id, created_at
         FROM minutes_meetings ORDER BY meeting_date DESC LIMIT 500",
    ).load(&mut conn).map_err(db::map_diesel_err)?;
    let items: Vec<serde_json::Value> = rows.into_iter().map(|r| serde_json::json!({
        "id": r.id, "title": r.title, "date": r.meeting_date, "duration_minutes": r.duration_minutes,
        "participants": r.participants, "status": r.status, "transcript_id": r.transcript_id, "created_at": r.created_at,
    })).collect();
    Ok(Json(serde_json::json!({"items": items})))
}

pub async fn list_transcripts() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Uuid)] meeting_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)] content: String,
        #[diesel(sql_type = diesel::sql_types::Text)] language: String,
        #[diesel(sql_type = diesel::sql_types::BigInt)] word_count: i64,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)] created_at: chrono::DateTime<Utc>,
    }
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT id, meeting_id, content, language, word_count, created_at
         FROM minutes_transcripts ORDER BY created_at DESC LIMIT 500",
    ).load(&mut conn).map_err(db::map_diesel_err)?;
    let items: Vec<serde_json::Value> = rows.into_iter().map(|r| serde_json::json!({
        "id": r.id, "meeting_id": r.meeting_id, "content": r.content, "language": r.language,
        "word_count": r.word_count, "created_at": r.created_at,
    })).collect();
    Ok(Json(serde_json::json!({"items": items})))
}

pub async fn list_documents() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Uuid)] meeting_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)] title: String,
        #[diesel(sql_type = diesel::sql_types::Text)] kind: String,
        #[diesel(sql_type = diesel::sql_types::Text)] content: String,
        #[diesel(sql_type = diesel::sql_types::BigInt)] version: i64,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)] created_at: chrono::DateTime<Utc>,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)] updated_at: chrono::DateTime<Utc>,
    }
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT id, meeting_id, title, kind, content, version, created_at, updated_at
         FROM minutes_documents ORDER BY updated_at DESC LIMIT 500",
    ).load(&mut conn).map_err(db::map_diesel_err)?;
    let items: Vec<serde_json::Value> = rows.into_iter().map(|r| serde_json::json!({
        "id": r.id, "meeting_id": r.meeting_id, "title": r.title, "kind": r.kind,
        "content": r.content, "version": r.version, "created_at": r.created_at, "updated_at": r.updated_at,
    })).collect();
    Ok(Json(serde_json::json!({"items": items})))
}

pub async fn update_document(Path(id): Path<String>, Json(item): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let parsed = Uuid::parse_str(&id).map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid id: {e}")))?;
    ensure_schema_sync()?;
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    if let Some(title) = item.get("title").and_then(|v| v.as_str()) {
        diesel::sql_query("UPDATE minutes_documents SET title = $1, version = version + 1, updated_at = NOW() WHERE id = $2")
            .bind::<diesel::sql_types::Text, _>(title)
            .bind::<diesel::sql_types::Uuid, _>(parsed)
            .execute(&mut conn).map_err(db::map_diesel_err)?;
    }
    if let Some(content) = item.get("content").and_then(|v| v.as_str()) {
        diesel::sql_query("UPDATE minutes_documents SET content = $1, version = version + 1, updated_at = NOW() WHERE id = $2")
            .bind::<diesel::sql_types::Text, _>(content)
            .bind::<diesel::sql_types::Uuid, _>(parsed)
            .execute(&mut conn).map_err(db::map_diesel_err)?;
    }
    Ok(Json(serde_json::json!({"success": true, "id": id})))
}

pub async fn start_meeting(Path(id): Path<String>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    let parsed = Uuid::parse_str(&id).map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid id: {e}")))?;
    let now = Utc::now();
    diesel::sql_query(
        "INSERT INTO minutes_meetings (id, title, meeting_date, duration_minutes, participants, status, created_at)
         VALUES ($1, $2, $3, 0, '[]'::jsonb, 'in_progress', $4)
         ON CONFLICT (id) DO UPDATE SET status = 'in_progress', meeting_date = $3",
    )
    .bind::<diesel::sql_types::Uuid, _>(parsed)
    .bind::<diesel::sql_types::Text, _>(format!("Meeting {id}"))
    .bind::<diesel::sql_types::Timestamptz, _>(now)
    .bind::<diesel::sql_types::Timestamptz, _>(now)
    .execute(&mut conn).map_err(db::map_diesel_err)?;
    Ok(Json(serde_json::json!({"success": true, "id": id})))
}

pub async fn update_meeting(Path(id): Path<String>, Json(item): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let parsed = Uuid::parse_str(&id).map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid id: {e}")))?;
    ensure_schema_sync()?;
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    if let Some(title) = item.get("title").and_then(|v| v.as_str()) {
        diesel::sql_query("UPDATE minutes_meetings SET title = $1 WHERE id = $2")
            .bind::<diesel::sql_types::Text, _>(title)
            .bind::<diesel::sql_types::Uuid, _>(parsed)
            .execute(&mut conn).map_err(db::map_diesel_err)?;
    }
    if let Some(status) = item.get("status").and_then(|v| v.as_str()) {
        diesel::sql_query("UPDATE minutes_meetings SET status = $1 WHERE id = $2")
            .bind::<diesel::sql_types::Text, _>(status)
            .bind::<diesel::sql_types::Uuid, _>(parsed)
            .execute(&mut conn).map_err(db::map_diesel_err)?;
    }
    Ok(Json(serde_json::json!({"success": true, "id": id})))
}

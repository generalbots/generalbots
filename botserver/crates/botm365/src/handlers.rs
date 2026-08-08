use axum::extract::Json;
use axum::http::{HeaderMap, StatusCode};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use diesel::RunQueryDsl;
use diesel::OptionalExtension;

use crate::db;
use crate::storage;

/// Resolves the caller's tenant branch from the server-minted JWT claims
/// (issue #734). Falls back to the global nil branch for anonymous/system
/// callers; every query is still bounded by the resolved branch.
fn resolve_branch(headers: &HeaderMap) -> Uuid {
    botcore::shared::tenant::branch_from_claims(headers).unwrap_or_else(Uuid::nil)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharePointItem {
    pub id: Uuid,
    pub site_name: String,
    pub list_name: String,
    pub item_count: i64,
    pub last_modified: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub id: Uuid,
    pub subject: String,
    pub start: chrono::DateTime<Utc>,
    pub end: chrono::DateTime<Utc>,
    pub location: Option<String>,
    pub attendees: Vec<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OneDriveFile {
    pub id: Uuid,
    pub name: String,
    pub path: String,
    pub size_bytes: i64,
    pub last_modified: chrono::DateTime<Utc>,
    pub author: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct M365Settings {
    pub tenant_id: String,
    pub client_id: String,
    pub connected: bool,
    pub scopes: Vec<String>,
    pub last_sync: Option<chrono::DateTime<Utc>>,
}

pub async fn list_sharepoint(headers: HeaderMap) -> Result<Json<Vec<SharePointItem>>, (StatusCode, String)> {
    storage::ensure_schema_sync()?;
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)] site_name: String,
        #[diesel(sql_type = diesel::sql_types::Text)] list_name: String,
        #[diesel(sql_type = diesel::sql_types::BigInt)] item_count: i64,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)] last_modified: chrono::DateTime<Utc>,
    }
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT id, COALESCE(site_id, '') AS site_name, COALESCE(list_id, '') AS list_name, \
         (CASE WHEN fields IS NULL THEN 0 ELSE 1 END)::bigint AS item_count, \
         COALESCE(modified_at, synced_at) AS last_modified
         FROM m365_sharepoint_items WHERE branch_id = $1 ORDER BY synced_at DESC LIMIT 500",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .load(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Json(rows.into_iter().map(|r| SharePointItem {
        id: r.id, site_name: r.site_name, list_name: r.list_name,
        item_count: r.item_count, last_modified: r.last_modified,
    }).collect()))
}

pub async fn list_calendar(headers: HeaderMap) -> Result<Json<Vec<CalendarEvent>>, (StatusCode, String)> {
    storage::ensure_schema_sync()?;
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)] subject: String,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)] start_time: chrono::DateTime<Utc>,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)] end_time: chrono::DateTime<Utc>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)] location: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Jsonb)] attendees: serde_json::Value,
        #[diesel(sql_type = diesel::sql_types::Text)] status: String,
    }
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT id, subject, start_time, end_time, location, attendees, status
         FROM m365_calendar_events WHERE branch_id = $1 ORDER BY start_time DESC LIMIT 500",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .load(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Json(rows.into_iter().map(|r| CalendarEvent {
        id: r.id, subject: r.subject, start: r.start_time, end: r.end_time,
        location: r.location,
        attendees: r.attendees.as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect()).unwrap_or_default(),
        status: r.status,
    }).collect()))
}

pub async fn list_onedrive(headers: HeaderMap) -> Result<Json<Vec<OneDriveFile>>, (StatusCode, String)> {
    storage::ensure_schema_sync()?;
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)] name: String,
        #[diesel(sql_type = diesel::sql_types::Text)] path: String,
        #[diesel(sql_type = diesel::sql_types::BigInt)] size_bytes: i64,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)] last_modified: chrono::DateTime<Utc>,
        #[diesel(sql_type = diesel::sql_types::Text)] author: String,
    }
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT id, name, path, size_bytes, last_modified, author
         FROM m365_onedrive_files WHERE branch_id = $1 ORDER BY last_modified DESC LIMIT 500",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .load(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Json(rows.into_iter().map(|r| OneDriveFile {
        id: r.id, name: r.name, path: r.path, size_bytes: r.size_bytes,
        last_modified: r.last_modified, author: r.author,
    }).collect()))
}

pub async fn get_settings(headers: HeaderMap) -> Result<Json<Option<M365Settings>>, (StatusCode, String)> {
    storage::ensure_schema_sync()?;
    let branch = resolve_branch(&headers);
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Text)] tenant_id: String,
        #[diesel(sql_type = diesel::sql_types::Text)] client_id: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)] last_sync: Option<chrono::DateTime<Utc>>,
    }
    let row: Option<Row> = diesel::sql_query(
        "SELECT tenant_id, client_id, last_sync FROM oauth_microsoft_settings \
         WHERE branch_id = $1 ORDER BY created_at DESC LIMIT 1",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .get_result(&mut conn)
    .optional()
    .map_err(db::map_diesel_err)?;
    Ok(Json(row.map(|r| M365Settings {
        tenant_id: r.tenant_id,
        client_id: r.client_id,
        connected: r.last_sync.is_some(),
        scopes: Vec::new(),
        last_sync: r.last_sync,
    })))
}

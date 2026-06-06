use axum::extract::Json;
use axum::http::StatusCode;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use diesel::RunQueryDsl;
use diesel::OptionalExtension;

use crate::db;

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

fn ensure_schema_sync() -> Result<(), (StatusCode, String)> {
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS m365_sharepoint_items (
            id UUID PRIMARY KEY,
            bot_id UUID NOT NULL DEFAULT gen_random_uuid(),
            site_name TEXT NOT NULL,
            list_name TEXT NOT NULL,
            item_count BIGINT NOT NULL DEFAULT 0,
            last_modified TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
    )
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS m365_calendar_events (
            id UUID PRIMARY KEY,
            bot_id UUID NOT NULL DEFAULT gen_random_uuid(),
            subject TEXT NOT NULL,
            start_time TIMESTAMPTZ NOT NULL,
            end_time TIMESTAMPTZ NOT NULL,
            location TEXT,
            attendees JSONB NOT NULL DEFAULT '[]'::jsonb,
            status VARCHAR(30) NOT NULL DEFAULT 'confirmed'
        )",
    )
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS m365_onedrive_files (
            id UUID PRIMARY KEY,
            bot_id UUID NOT NULL DEFAULT gen_random_uuid(),
            name TEXT NOT NULL,
            path TEXT NOT NULL,
            size_bytes BIGINT NOT NULL DEFAULT 0,
            last_modified TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            author TEXT NOT NULL DEFAULT ''
        )",
    )
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS oauth_microsoft_settings (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            tenant_id TEXT NOT NULL DEFAULT '',
            client_id TEXT NOT NULL DEFAULT '',
            client_secret_encrypted TEXT,
            scopes JSONB NOT NULL DEFAULT '[]'::jsonb,
            connected BOOLEAN NOT NULL DEFAULT false,
            last_sync_at TIMESTAMPTZ
        )",
    )
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(())
}

pub async fn list_sharepoint() -> Result<Json<Vec<SharePointItem>>, (StatusCode, String)> {
    ensure_schema_sync()?;
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
        "SELECT id, site_name, list_name, item_count, last_modified
         FROM m365_sharepoint_items ORDER BY last_modified DESC LIMIT 500",
    )
    .load(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Json(rows.into_iter().map(|r| SharePointItem {
        id: r.id, site_name: r.site_name, list_name: r.list_name,
        item_count: r.item_count, last_modified: r.last_modified,
    }).collect()))
}

pub async fn list_calendar() -> Result<Json<Vec<CalendarEvent>>, (StatusCode, String)> {
    ensure_schema_sync()?;
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
         FROM m365_calendar_events ORDER BY start_time DESC LIMIT 500",
    )
    .load(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Json(rows.into_iter().map(|r| CalendarEvent {
        id: r.id, subject: r.subject, start: r.start_time, end: r.end_time,
        location: r.location,
        attendees: r.attendees.as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect()).unwrap_or_default(),
        status: r.status,
    }).collect()))
}

pub async fn list_onedrive() -> Result<Json<Vec<OneDriveFile>>, (StatusCode, String)> {
    ensure_schema_sync()?;
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
         FROM m365_onedrive_files ORDER BY last_modified DESC LIMIT 500",
    )
    .load(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Json(rows.into_iter().map(|r| OneDriveFile {
        id: r.id, name: r.name, path: r.path, size_bytes: r.size_bytes,
        last_modified: r.last_modified, author: r.author,
    }).collect()))
}

pub async fn get_settings() -> Result<Json<Option<M365Settings>>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Text)] tenant_id: String,
        #[diesel(sql_type = diesel::sql_types::Text)] client_id: String,
        #[diesel(sql_type = diesel::sql_types::Jsonb)] scopes: serde_json::Value,
        #[diesel(sql_type = diesel::sql_types::Bool)] connected: bool,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)] last_sync_at: Option<chrono::DateTime<Utc>>,
    }
    let row: Option<Row> = diesel::sql_query(
        "SELECT tenant_id, client_id, scopes, connected, last_sync_at FROM oauth_microsoft_settings LIMIT 1",
    )
    .get_result(&mut conn)
    .optional()
    .map_err(db::map_diesel_err)?;
    Ok(Json(row.map(|r| M365Settings {
        tenant_id: r.tenant_id, client_id: r.client_id,
        connected: r.connected,
        scopes: r.scopes.as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect()).unwrap_or_default(),
        last_sync: r.last_sync_at,
    })))
}

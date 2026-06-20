use axum::extract::{Json, Path};
use axum::http::StatusCode;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use diesel::RunQueryDsl;
use diesel::OptionalExtension;

use crate::db;
use crate::storage::ensure_schema_sync;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Incident {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub severity: String,
    pub status: String,
    pub assignee: Option<String>,
    pub created_at: chrono::DateTime<Utc>,
    pub resolved_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRequest {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub category: String,
    pub status: String,
    pub requester: String,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CmdbItem {
    pub id: Uuid,
    pub name: String,
    pub kind: String,
    pub owner: String,
    pub status: String,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KbArticle {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub author: String,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct NewIncident {
    pub title: String,
    pub description: String,
    pub severity: String,
    pub assignee: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateIncident {
    pub title: Option<String>,
    pub description: Option<String>,
    pub severity: Option<String>,
    pub status: Option<String>,
    pub assignee: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct NewServiceRequest {
    pub title: String,
    pub description: String,
    pub category: String,
    pub requester: String,
}

#[derive(diesel::QueryableByName)]
struct IncidentRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)] title: String,
    #[diesel(sql_type = diesel::sql_types::Text)] description: String,
    #[diesel(sql_type = diesel::sql_types::Text)] severity: String,
    #[diesel(sql_type = diesel::sql_types::Text)] status: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)] assignee: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)] created_at: chrono::DateTime<Utc>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)] resolved_at: Option<chrono::DateTime<Utc>>,
}

pub async fn list_incidents() -> Result<Json<Vec<Incident>>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    let rows: Vec<IncidentRow> = diesel::sql_query(
        "SELECT id, title, description, severity, status, assignee, created_at, resolved_at
         FROM itsm_incidents ORDER BY created_at DESC LIMIT 500",
    )
    .load(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Json(rows.into_iter().map(|r| Incident {
        id: r.id, title: r.title, description: r.description, severity: r.severity,
        status: r.status, assignee: r.assignee, created_at: r.created_at, resolved_at: r.resolved_at,
    }).collect()))
}

pub async fn create_incident(Json(req): Json<NewIncident>) -> Result<Json<Incident>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    let id = Uuid::new_v4();
    let now = Utc::now();
    diesel::sql_query(
        "INSERT INTO itsm_incidents (id, title, description, severity, status, assignee, created_at)
         VALUES ($1, $2, $3, $4, 'open', $5, $6)",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .bind::<diesel::sql_types::Text, _>(&req.title)
    .bind::<diesel::sql_types::Text, _>(&req.description)
    .bind::<diesel::sql_types::Text, _>(&req.severity)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(req.assignee.as_deref())
    .bind::<diesel::sql_types::Timestamptz, _>(now)
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Json(Incident {
        id, title: req.title, description: req.description, severity: req.severity,
        status: "open".to_string(), assignee: req.assignee, created_at: now, resolved_at: None,
    }))
}

pub async fn update_incident(
    Path(id): Path<String>,
    Json(req): Json<UpdateIncident>,
) -> Result<Json<Incident>, (StatusCode, String)> {
    let parsed = Uuid::parse_str(&id)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid id '{id}': {e}")))?;
    ensure_schema_sync()?;
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    let existing: Option<IncidentRow> = diesel::sql_query(
        "SELECT id, title, description, severity, status, assignee, created_at, resolved_at
         FROM itsm_incidents WHERE id = $1",
    )
    .bind::<diesel::sql_types::Uuid, _>(parsed)
    .get_result(&mut conn)
    .optional()
    .map_err(db::map_diesel_err)?;
    let existing = existing.ok_or((StatusCode::NOT_FOUND, format!("Incident {id} not found")))?;
    let new_title = req.title.unwrap_or(existing.title);
    let new_desc = req.description.unwrap_or(existing.description);
    let new_sev = req.severity.unwrap_or(existing.severity);
    let new_status = req.status.clone().unwrap_or(existing.status);
    let new_assignee = req.assignee.or(existing.assignee);
    let resolved_at = if new_status == "resolved" && existing.resolved_at.is_none() {
        Some(Utc::now())
    } else {
        existing.resolved_at
    };
    diesel::sql_query(
        "UPDATE itsm_incidents SET title = $1, description = $2, severity = $3, status = $4, assignee = $5, resolved_at = $6 WHERE id = $7",
    )
    .bind::<diesel::sql_types::Text, _>(&new_title)
    .bind::<diesel::sql_types::Text, _>(&new_desc)
    .bind::<diesel::sql_types::Text, _>(&new_sev)
    .bind::<diesel::sql_types::Text, _>(&new_status)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(new_assignee.as_deref())
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>, _>(resolved_at)
    .bind::<diesel::sql_types::Uuid, _>(parsed)
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Json(Incident {
        id: parsed, title: new_title, description: new_desc, severity: new_sev,
        status: new_status, assignee: new_assignee, created_at: existing.created_at, resolved_at,
    }))
}

pub async fn list_requests() -> Result<Json<Vec<ServiceRequest>>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)] title: String,
        #[diesel(sql_type = diesel::sql_types::Text)] description: String,
        #[diesel(sql_type = diesel::sql_types::Text)] category: String,
        #[diesel(sql_type = diesel::sql_types::Text)] status: String,
        #[diesel(sql_type = diesel::sql_types::Text)] requester: String,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)] created_at: chrono::DateTime<Utc>,
    }
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT id, title, description, category, status, requester, created_at
         FROM itsm_service_requests ORDER BY created_at DESC LIMIT 500",
    )
    .load(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Json(rows.into_iter().map(|r| ServiceRequest {
        id: r.id, title: r.title, description: r.description, category: r.category,
        status: r.status, requester: r.requester, created_at: r.created_at,
    }).collect()))
}

pub async fn create_request(Json(req): Json<NewServiceRequest>) -> Result<Json<ServiceRequest>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    let id = Uuid::new_v4();
    let now = Utc::now();
    diesel::sql_query(
        "INSERT INTO itsm_service_requests (id, title, description, category, status, requester, created_at)
         VALUES ($1, $2, $3, $4, 'open', $5, $6)",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .bind::<diesel::sql_types::Text, _>(&req.title)
    .bind::<diesel::sql_types::Text, _>(&req.description)
    .bind::<diesel::sql_types::Text, _>(&req.category)
    .bind::<diesel::sql_types::Text, _>(&req.requester)
    .bind::<diesel::sql_types::Timestamptz, _>(now)
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Json(ServiceRequest {
        id, title: req.title, description: req.description, category: req.category,
        status: "open".to_string(), requester: req.requester, created_at: now,
    }))
}

pub async fn list_cmdb() -> Result<Json<Vec<CmdbItem>>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)] name: String,
        #[diesel(sql_type = diesel::sql_types::Text)] kind: String,
        #[diesel(sql_type = diesel::sql_types::Text)] owner: String,
        #[diesel(sql_type = diesel::sql_types::Text)] status: String,
        #[diesel(sql_type = diesel::sql_types::Jsonb)] dependencies: serde_json::Value,
    }
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT id, name, kind, owner, status, dependencies FROM itsm_cmdb ORDER BY name ASC LIMIT 500",
    )
    .load(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Json(rows.into_iter().map(|r| CmdbItem {
        id: r.id, name: r.name, kind: r.kind, owner: r.owner, status: r.status,
        dependencies: r.dependencies.as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect()).unwrap_or_default(),
    }).collect()))
}

pub async fn list_kb() -> Result<Json<Vec<KbArticle>>, (StatusCode, String)> {
    ensure_schema_sync()?;
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)] title: String,
        #[diesel(sql_type = diesel::sql_types::Text)] content: String,
        #[diesel(sql_type = diesel::sql_types::Jsonb)] tags: serde_json::Value,
        #[diesel(sql_type = diesel::sql_types::Text)] author: String,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)] created_at: chrono::DateTime<Utc>,
    }
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT id, title, content, tags, author, created_at FROM itsm_kb ORDER BY created_at DESC LIMIT 500",
    )
    .load(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Json(rows.into_iter().map(|r| KbArticle {
        id: r.id, title: r.title, content: r.content,
        tags: r.tags.as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect()).unwrap_or_default(),
        author: r.author, created_at: r.created_at,
    }).collect()))
}

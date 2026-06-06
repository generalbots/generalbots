use axum::extract::{Json, Path};
use axum::http::StatusCode;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Camera {
    pub id: Uuid,
    pub bot_id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub rtsp_url: String,
    pub location: Option<String>,
    pub zone: Option<String>,
    pub enabled: bool,
    pub status: String,
    pub last_frame_at: Option<chrono::DateTime<Utc>>,
    pub last_seen_at: Option<chrono::DateTime<Utc>>,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringEvent {
    pub id: Uuid,
    pub bot_id: Uuid,
    pub camera_id: Uuid,
    pub event_type: String,
    pub severity: String,
    pub confidence: String,
    pub description: Option<String>,
    pub snapshot_url: Option<String>,
    pub detected_at: chrono::DateTime<Utc>,
    pub acknowledged_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraAnalytics {
    pub camera_id: Option<Uuid>,
    pub total_cameras: i64,
    pub total_events: i64,
    pub total_alerts: i64,
    pub events_by_severity: serde_json::Value,
    pub period: String,
}

#[derive(Debug, Deserialize)]
pub struct NewCamera {
    pub bot_id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub rtsp_url: String,
    pub location: Option<String>,
    pub zone: Option<String>,
}

pub async fn list_cameras() -> Result<Json<Vec<Camera>>, (StatusCode, String)> {
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(db::map_diesel_err)?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Uuid)] bot_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Uuid)] organization_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)] name: String,
        #[diesel(sql_type = diesel::sql_types::Text)] rtsp_url: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)] location: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)] zone: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Bool)] enabled: bool,
        #[diesel(sql_type = diesel::sql_types::Text)] status: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)] last_frame_at: Option<chrono::DateTime<Utc>>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)] last_seen_at: Option<chrono::DateTime<Utc>>,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)] created_at: chrono::DateTime<Utc>,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)] updated_at: chrono::DateTime<Utc>,
    }
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT id, bot_id, organization_id, name, rtsp_url, location, zone, enabled, status,
                last_frame_at, last_seen_at, created_at, updated_at
         FROM monitoring_cameras ORDER BY name ASC LIMIT 500",
    )
    .load(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Json(rows.into_iter().map(|r| Camera {
        id: r.id, bot_id: r.bot_id, organization_id: r.organization_id,
        name: r.name, rtsp_url: r.rtsp_url, location: r.location, zone: r.zone,
        enabled: r.enabled, status: r.status, last_frame_at: r.last_frame_at,
        last_seen_at: r.last_seen_at, created_at: r.created_at, updated_at: r.updated_at,
    }).collect()))
}

pub async fn create_camera(Json(req): Json<NewCamera>) -> Result<Json<Camera>, (StatusCode, String)> {
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(db::map_diesel_err)?;
    let id = Uuid::new_v4();
    let now = Utc::now();
    diesel::sql_query(
        "INSERT INTO monitoring_cameras
            (id, bot_id, organization_id, name, rtsp_url, location, zone, enabled, status, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, true, 'offline', $8, $9)",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .bind::<diesel::sql_types::Uuid, _>(req.bot_id)
    .bind::<diesel::sql_types::Uuid, _>(req.organization_id)
    .bind::<diesel::sql_types::Text, _>(&req.name)
    .bind::<diesel::sql_types::Text, _>(&req.rtsp_url)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(req.location.as_deref())
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(req.zone.as_deref())
    .bind::<diesel::sql_types::Timestamptz, _>(now)
    .bind::<diesel::sql_types::Timestamptz, _>(now)
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Json(Camera {
        id, bot_id: req.bot_id, organization_id: req.organization_id,
        name: req.name, rtsp_url: req.rtsp_url, location: req.location, zone: req.zone,
        enabled: true, status: "offline".to_string(), last_frame_at: None,
        last_seen_at: None, created_at: now, updated_at: now,
    }))
}

pub async fn delete_camera(Path(id): Path<String>) -> Result<StatusCode, (StatusCode, String)> {
    let parsed = Uuid::parse_str(&id)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid id '{id}': {e}")))?;
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(db::map_diesel_err)?;
    let n = diesel::sql_query("DELETE FROM monitoring_cameras WHERE id = $1")
        .bind::<diesel::sql_types::Uuid, _>(parsed)
        .execute(&mut conn)
        .map_err(db::map_diesel_err)?;
    if n == 0 {
        return Err((StatusCode::NOT_FOUND, format!("Camera {id} not found")));
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_alerts() -> Result<Json<Vec<MonitoringEvent>>, (StatusCode, String)> {
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(db::map_diesel_err)?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Uuid)] bot_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Uuid)] camera_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)] event_type: String,
        #[diesel(sql_type = diesel::sql_types::Text)] severity: String,
        #[diesel(sql_type = diesel::sql_types::Numeric)] confidence: rust_decimal::Decimal,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)] description: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)] snapshot_url: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)] detected_at: chrono::DateTime<Utc>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)] acknowledged_at: Option<chrono::DateTime<Utc>>,
    }
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT id, bot_id, camera_id, event_type, severity, confidence, description, snapshot_url, detected_at, acknowledged_at
         FROM monitoring_events ORDER BY detected_at DESC LIMIT 500",
    )
    .load(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Json(rows.into_iter().map(|r| MonitoringEvent {
        id: r.id, bot_id: r.bot_id, camera_id: r.camera_id,
        event_type: r.event_type, severity: r.severity, confidence: r.confidence.to_string(),
        description: r.description, snapshot_url: r.snapshot_url,
        detected_at: r.detected_at, acknowledged_at: r.acknowledged_at,
    }).collect()))
}

pub async fn list_analytics() -> Result<Json<Vec<CameraAnalytics>>, (StatusCode, String)> {
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(db::map_diesel_err)?;
    #[derive(diesel::QueryableByName)]
    struct Count {
        #[diesel(sql_type = diesel::sql_types::BigInt)] total: i64,
    }
    let cameras: Count = diesel::sql_query("SELECT COUNT(*) AS total FROM monitoring_cameras")
        .get_result(&mut conn)
        .map_err(db::map_diesel_err)?;
    let events: Count = diesel::sql_query("SELECT COUNT(*) AS total FROM monitoring_events")
        .get_result(&mut conn)
        .map_err(db::map_diesel_err)?;
    let alerts: Count = diesel::sql_query(
        "SELECT COUNT(*) AS total FROM monitoring_events WHERE severity IN ('high','critical')",
    )
    .get_result(&mut conn)
    .map_err(db::map_diesel_err)?;
    let severity_map = serde_json::json!({
        "info": 0, "low": 0, "medium": 0, "high": alerts.total, "critical": 0,
    });
    Ok(Json(vec![CameraAnalytics {
        camera_id: None,
        total_cameras: cameras.total,
        total_events: events.total,
        total_alerts: alerts.total,
        events_by_severity: severity_map,
        period: "30d".to_string(),
    }]))
}

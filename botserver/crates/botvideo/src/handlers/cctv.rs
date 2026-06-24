use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::routes::AppState;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Camera {
    pub id: Uuid,
    pub name: String,
    pub stream_url: String,
    pub status: String,
    pub last_seen: String,
}

#[derive(Debug, Deserialize)]
pub struct AddCameraRequest {
    pub name: String,
    pub stream_url: String,
}

#[derive(Debug, Serialize)]
pub struct CameraListResponse {
    pub cameras: Vec<Camera>,
}

#[derive(Debug, Serialize)]
pub struct CameraCreateResponse {
    pub camera: Camera,
}

fn default_cameras() -> Vec<Camera> {
    vec![
        Camera {
            id: Uuid::nil(),
            name: "Main Entrance".to_string(),
            stream_url: "/api/video/stream/main".to_string(),
            status: "online".to_string(),
            last_seen: chrono::Utc::now().to_rfc3339(),
        },
    ]
}

pub async fn list_cameras(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<CameraListResponse>, (StatusCode, String)> {
    let cameras = default_cameras();
    Ok(Json(CameraListResponse { cameras }))
}

pub async fn add_camera(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<AddCameraRequest>,
) -> Result<Json<CameraCreateResponse>, (StatusCode, String)> {
    let camera = Camera {
        id: Uuid::new_v4(),
        name: req.name,
        stream_url: req.stream_url,
        status: "online".to_string(),
        last_seen: chrono::Utc::now().to_rfc3339(),
    };
    Ok(Json(CameraCreateResponse { camera }))
}

#[derive(Debug, Serialize, Clone)]
pub struct Alert {
    pub id: Uuid,
    pub camera_name: String,
    pub alert_type: String,
    pub severity: String,
    pub timestamp: String,
    pub acknowledged: bool,
}

#[derive(Debug, Serialize)]
pub struct AlertListResponse {
    pub alerts: Vec<Alert>,
}

pub async fn list_alerts(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<AlertListResponse>, (StatusCode, String)> {
    let alerts = vec![Alert {
        id: Uuid::nil(),
        camera_name: "Main Entrance".to_string(),
        alert_type: "motion_detected".to_string(),
        severity: "low".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        acknowledged: false,
    }];
    Ok(Json(AlertListResponse { alerts }))
}

#[derive(Debug, Serialize)]
pub struct AnalyticsSummary {
    pub cameras_active: u32,
    pub detections_today: u32,
    pub storage_used_gb: f64,
    pub uptime_pct: f64,
}

pub async fn get_analytics_summary(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<AnalyticsSummary>, (StatusCode, String)> {
    Ok(Json(AnalyticsSummary {
        cameras_active: 1,
        detections_today: 0,
        storage_used_gb: 0.5,
        uptime_pct: 99.9,
    }))
}

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Camera {
    pub id: Uuid,
    pub name: String,
    pub rtsp_url: String,
    pub location: String,
    pub status: String,
    pub last_alert_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: Uuid,
    pub camera_id: Uuid,
    pub camera_name: String,
    pub alert_type: String,
    pub severity: String,
    pub message: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoAnalytics {
    pub total_cameras: usize,
    pub online_cameras: usize,
    pub total_alerts: usize,
    pub critical_alerts: usize,
    pub high_alerts: usize,
    pub medium_alerts: usize,
    pub low_alerts: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCamera {
    pub name: String,
    pub rtsp_url: String,
    pub location: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAlert {
    pub camera_id: Uuid,
    pub alert_type: String,
    pub severity: String,
    pub message: String,
}

#[derive(Debug, Default)]
pub struct VideoState {
    pub cameras: HashMap<Uuid, Camera>,
    pub alerts: HashMap<Uuid, Alert>,
}



pub fn create_video_state() -> SharedVideoState {
    Arc::new(RwLock::new(VideoState::default()))
}

async fn list_cameras(
    State(state): State<SharedVideoState>,
) -> Result<Json<Vec<Camera>>, StatusCode> {
    let data = state.read().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(data.cameras.values().cloned().collect()))
}

async fn create_camera(
    State(state): State<SharedVideoState>,
    Json(input): Json<CreateCamera>,
) -> Result<(StatusCode, Json<Camera>), StatusCode> {
    let mut data = state.write().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let camera = Camera {
        id: Uuid::new_v4(),
        name: input.name,
        rtsp_url: input.rtsp_url,
        location: input.location,
        status: "online".to_string(),
        last_alert_at: None,
    };
    data.cameras.insert(camera.id, camera.clone());
    Ok((StatusCode::CREATED, Json(camera)))
}

async fn delete_camera(
    State(state): State<SharedVideoState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let mut data = state.write().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    data.cameras.remove(&id).ok_or(StatusCode::NOT_FOUND)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn list_alerts(
    State(state): State<SharedVideoState>,
) -> Result<Json<Vec<Alert>>, StatusCode> {
    let data = state.read().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(data.alerts.values().cloned().collect()))
}

async fn get_analytics(
    State(state): State<SharedVideoState>,
) -> Result<Json<VideoAnalytics>, StatusCode> {
    let data = state.read().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let total_cameras = data.cameras.len();
    let online_cameras = data
        .cameras
        .values()
        .filter(|c| c.status == "online")
        .count();
    let total_alerts = data.alerts.len();
    let critical_alerts = data
        .alerts
        .values()
        .filter(|a| a.severity == "critical")
        .count();
    let high_alerts = data
        .alerts
        .values()
        .filter(|a| a.severity == "high")
        .count();
    let medium_alerts = data
        .alerts
        .values()
        .filter(|a| a.severity == "medium")
        .count();
    let low_alerts = data
        .alerts
        .values()
        .filter(|a| a.severity == "low")
        .count();
    Ok(Json(VideoAnalytics {
        total_cameras,
        online_cameras,
        total_alerts,
        critical_alerts,
        high_alerts,
        medium_alerts,
        low_alerts,
    }))
}

pub fn routes() -> Router {
    let state = std::sync::Arc::new(std::sync::RwLock::new(Default::default()));
    Router::new()
        .route(
            "/api/video/cameras",
            get(list_cameras).post(create_camera),
        )
        .route("/api/video/cameras/{id}", delete(delete_camera))
        .route("/api/video/alerts", get(list_alerts))
        .route("/api/video/analytics", get(get_analytics))
        .with_state(state)
}

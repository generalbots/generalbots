use axum::{extract::{State, Json, Path}, routing::get, Router};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CameraStatus { Online, Offline, Maintenance }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity { Low, Medium, High, Critical }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Camera {
    pub id: Uuid,
    pub name: String,
    pub url: String,
    pub location: String,
    pub status: String,
    pub resolution: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VideoAlert {
    pub id: Uuid,
    pub camera_id: Uuid,
    pub alert_type: String,
    pub severity: String,
    pub message: String,
    pub acknowledged: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VideoAnalytic {
    pub id: Uuid,
    pub camera_id: Uuid,
    pub analytic_type: String,
    pub result: String,
    pub confidence: f64,
    pub metadata: String,
    pub created_at: String,
}

#[derive(Default)]
pub struct VideoState {
    pub cameras: HashMap<Uuid, Camera>,
    pub alerts: HashMap<Uuid, VideoAlert>,
    pub analytics: HashMap<Uuid, VideoAnalytic>,
}

pub fn routes() -> axum::Router {
    let state = Arc::new(RwLock::new(VideoState::default()));
    Router::new()
        .route("/api/video/cameras", get(list_cameras).post(create_camera))
        .route("/api/video/cameras/{id}", get(get_camera).put(update_camera).delete(delete_camera))
        .route("/api/video/alerts", get(list_alerts).post(create_alert))
        .route("/api/video/alerts/{id}", get(get_alert).put(update_alert).delete(delete_alert))
        .route("/api/video/analytics", get(list_analytics).post(create_analytic))
        .route("/api/video/analytics/{id}", get(get_analytic).delete(delete_analytic))
        .with_state(state)
}

async fn list_cameras(State(state): State<Arc<RwLock<VideoState>>>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    let items: Vec<&Camera> = s.cameras.values().collect();
    Json(serde_json::json!({"cameras": items}))
}

async fn create_camera(State(state): State<Arc<RwLock<VideoState>>>, Json(mut cam): Json<Camera>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    let id = Uuid::new_v4();
    cam.id = id;
    cam.status = "Online".to_string();
    cam.created_at = Utc::now().to_rfc3339();
    s.cameras.insert(id, cam.clone());
    Json(serde_json::json!({"camera": cam}))
}

async fn get_camera(State(state): State<Arc<RwLock<VideoState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    match s.cameras.get(&id) {
        Some(cam) => Json(serde_json::json!({"camera": cam})),
        None => Json(serde_json::json!({"error": "Camera not found"})),
    }
}

async fn update_camera(State(state): State<Arc<RwLock<VideoState>>>, Path(id): Path<Uuid>, Json(cam): Json<Camera>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    if let Some(existing) = s.cameras.get_mut(&id) {
        *existing = cam.clone();
        existing.id = id;
        Json(serde_json::json!({"camera": existing.clone()}))
    } else {
        Json(serde_json::json!({"error": "Camera not found"}))
    }
}

async fn delete_camera(State(state): State<Arc<RwLock<VideoState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    s.cameras.remove(&id);
    Json(serde_json::json!({"deleted": true}))
}

async fn list_alerts(State(state): State<Arc<RwLock<VideoState>>>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    let items: Vec<&VideoAlert> = s.alerts.values().collect();
    Json(serde_json::json!({"alerts": items}))
}

async fn create_alert(State(state): State<Arc<RwLock<VideoState>>>, Json(mut alert): Json<VideoAlert>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    let id = Uuid::new_v4();
    alert.id = id;
    alert.acknowledged = false;
    alert.created_at = Utc::now().to_rfc3339();
    s.alerts.insert(id, alert.clone());
    Json(serde_json::json!({"alert": alert}))
}

async fn get_alert(State(state): State<Arc<RwLock<VideoState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    match s.alerts.get(&id) {
        Some(alert) => Json(serde_json::json!({"alert": alert})),
        None => Json(serde_json::json!({"error": "Alert not found"})),
    }
}

async fn update_alert(State(state): State<Arc<RwLock<VideoState>>>, Path(id): Path<Uuid>, Json(alert): Json<VideoAlert>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    if let Some(existing) = s.alerts.get_mut(&id) {
        *existing = alert.clone();
        existing.id = id;
        Json(serde_json::json!({"alert": existing.clone()}))
    } else {
        Json(serde_json::json!({"error": "Alert not found"}))
    }
}

async fn delete_alert(State(state): State<Arc<RwLock<VideoState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    s.alerts.remove(&id);
    Json(serde_json::json!({"deleted": true}))
}

async fn list_analytics(State(state): State<Arc<RwLock<VideoState>>>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    let items: Vec<&VideoAnalytic> = s.analytics.values().collect();
    Json(serde_json::json!({"analytics": items}))
}

async fn create_analytic(State(state): State<Arc<RwLock<VideoState>>>, Json(mut analytic): Json<VideoAnalytic>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    let id = Uuid::new_v4();
    analytic.id = id;
    analytic.created_at = Utc::now().to_rfc3339();
    s.analytics.insert(id, analytic.clone());
    Json(serde_json::json!({"analytic": analytic}))
}

async fn get_analytic(State(state): State<Arc<RwLock<VideoState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    match s.analytics.get(&id) {
        Some(a) => Json(serde_json::json!({"analytic": a})),
        None => Json(serde_json::json!({"error": "Analytic not found"})),
    }
}

async fn delete_analytic(State(state): State<Arc<RwLock<VideoState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    s.analytics.remove(&id);
    Json(serde_json::json!({"deleted": true}))
}

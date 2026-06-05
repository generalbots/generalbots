use axum::extract::{Json, Path};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, OnceLock};
use axum::http::StatusCode;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Camera {
    pub id: String,
    pub name: String,
    pub url: String,
    pub location: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Alert {
    pub id: String,
    pub camera_id: String,
    pub kind: String,
    pub severity: String,
    pub message: String,
    pub triggered_at: String,
    pub acknowledged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Analytics {
    pub camera_id: String,
    pub period: String,
    pub detections: u64,
    pub alerts: u64,
    pub uptime_pct: f64,
}

#[derive(Default)]
struct AppState {
    cameras: HashMap<String, Camera>,
    alerts: Vec<Alert>,
    analytics: Vec<Analytics>,
}

fn state() -> &'static Arc<RwLock<AppState>> {
    static S: OnceLock<Arc<RwLock<AppState>>> = OnceLock::new();
    S.get_or_init(|| Arc::new(RwLock::new(AppState::default())))
}

pub async fn list_cameras() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let s = state().read().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {}", e)))?;
    let items: Vec<&Camera> = s.cameras.values().collect();
    Ok(Json(serde_json::json!({"items": items})))
}

pub async fn create_camera(Json(item): Json<Camera>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut s = state().write().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {}", e)))?;
    let id = uuid::Uuid::new_v4().to_string();
    let mut new_item = item;
    new_item.id = id.clone();
    new_item.created_at = chrono::Utc::now().to_rfc3339();
    new_item.status = "active".to_string();
    s.cameras.insert(id.clone(), new_item.clone());
    Ok(Json(serde_json::json!({"item": new_item})))
}

pub async fn delete_camera(Path(id): Path<String>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut s = state().write().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {}", e)))?;
    match s.cameras.remove(&id) {
        Some(_) => Ok(Json(serde_json::json!({"deleted": true}))),
        None => Err((StatusCode::NOT_FOUND, "Camera not found".to_string())),
    }
}

pub async fn list_alerts() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let s = state().read().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {}", e)))?;
    let items: Vec<&Alert> = s.alerts.iter().collect();
    Ok(Json(serde_json::json!({"items": items})))
}

pub async fn list_analytics() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let s = state().read().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {}", e)))?;
    let items: Vec<&Analytics> = s.analytics.iter().collect();
    Ok(Json(serde_json::json!({"items": items})))
}

use axum::extract::{Json, Path};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, OnceLock};
use axum::http::StatusCode;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClockEvent {
    pub id: String,
    pub employee_id: String,
    pub kind: String,
    pub timestamp: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TimeRecord {
    pub id: String,
    pub employee_id: String,
    pub date: String,
    pub clock_in: String,
    pub clock_out: Option<String>,
    pub hours_worked: f64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OvertimeRequest {
    pub id: String,
    pub employee_id: String,
    pub date: String,
    pub hours: f64,
    pub reason: String,
    pub status: String,
    pub approved_by: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Report {
    pub id: String,
    pub period: String,
    pub total_hours: f64,
    pub overtime_hours: f64,
    pub employees: u64,
    pub created_at: String,
}

#[derive(Default)]
struct AppState {
    clock_events: Vec<ClockEvent>,
    records: HashMap<String, TimeRecord>,
    overtime: HashMap<String, OvertimeRequest>,
    reports: HashMap<String, Report>,
}

fn state() -> &'static Arc<RwLock<AppState>> {
    static S: OnceLock<Arc<RwLock<AppState>>> = OnceLock::new();
    S.get_or_init(|| Arc::new(RwLock::new(AppState::default())))
}

pub async fn clock_in_out(Json(event): Json<ClockEvent>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut s = state().write().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {}", e)))?;
    let mut new_event = event;
    new_event.id = uuid::Uuid::new_v4().to_string();
    new_event.timestamp = chrono::Utc::now().to_rfc3339();
    s.clock_events.push(new_event.clone());
    Ok(Json(serde_json::json!({"event": new_event})))
}

pub async fn list_records() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let s = state().read().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {}", e)))?;
    let items: Vec<&TimeRecord> = s.records.values().collect();
    Ok(Json(serde_json::json!({"items": items})))
}

pub async fn list_overtime() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let s = state().read().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {}", e)))?;
    let items: Vec<&OvertimeRequest> = s.overtime.values().collect();
    Ok(Json(serde_json::json!({"items": items})))
}

pub async fn approve_overtime(Path(id): Path<String>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut s = state().write().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {}", e)))?;
    match s.overtime.get_mut(&id) {
        Some(req) => {
            req.status = "approved".to_string();
            req.approved_by = Some("system".to_string());
            Ok(Json(serde_json::json!({"item": req})))
        }
        None => Err((StatusCode::NOT_FOUND, "Overtime request not found".to_string())),
    }
}

pub async fn get_reports() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let s = state().read().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {}", e)))?;
    let items: Vec<&Report> = s.reports.values().collect();
    Ok(Json(serde_json::json!({"items": items})))
}

use axum::{extract::{State, Json, Path}, routing::{get, post}, Router};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClockEntry {
    pub id: Uuid,
    pub employee_id: String,
    pub clock_type: String,
    pub timestamp: String,
    pub location: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TimeRecord {
    pub id: Uuid,
    pub employee_id: String,
    pub date: String,
    pub clock_in: String,
    pub clock_out: Option<String>,
    pub total_hours: f64,
    pub regular_hours: f64,
    pub overtime_hours: f64,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OvertimeRequest {
    pub id: Uuid,
    pub employee_id: String,
    pub date: String,
    pub hours: f64,
    pub reason: String,
    pub status: String,
    pub approved_by: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TimeclockReport {
    pub id: Uuid,
    pub employee_id: String,
    pub period: String,
    pub total_hours: f64,
    pub regular_hours: f64,
    pub overtime_hours: f64,
    pub days_worked: u32,
    pub absences: u32,
    pub created_at: String,
}

#[derive(Default)]
pub struct TimeclockState {
    pub clock_entries: HashMap<Uuid, ClockEntry>,
    pub records: HashMap<Uuid, TimeRecord>,
    pub overtime_requests: HashMap<Uuid, OvertimeRequest>,
    pub reports: HashMap<Uuid, TimeclockReport>,
}

pub fn routes() -> axum::Router {
    let state = Arc::new(RwLock::new(TimeclockState::default()));
    Router::new()
        .route("/api/timeclock/clock", post(clock_in_out))
        .route("/api/timeclock/clock/{id}", get(get_clock_entry))
        .route("/api/timeclock/records", get(list_records).post(create_record))
        .route("/api/timeclock/records/{id}", get(get_record).put(update_record))
        .route("/api/timeclock/overtime", get(list_overtime).post(create_overtime))
        .route("/api/timeclock/overtime/{id}", get(get_overtime).put(update_overtime))
        .route("/api/timeclock/reports", get(list_reports).post(create_report))
        .route("/api/timeclock/reports/{id}", get(get_report))
        .with_state(state)
}

async fn clock_in_out(State(state): State<Arc<RwLock<TimeclockState>>>, Json(mut entry): Json<ClockEntry>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    let id = Uuid::new_v4();
    entry.id = id;
    entry.timestamp = Utc::now().to_rfc3339();
    entry.created_at = Utc::now().to_rfc3339();
    s.clock_entries.insert(id, entry.clone());
    Json(serde_json::json!({"clock_entry": entry}))
}

async fn get_clock_entry(State(state): State<Arc<RwLock<TimeclockState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    match s.clock_entries.get(&id) {
        Some(e) => Json(serde_json::json!({"clock_entry": e})),
        None => Json(serde_json::json!({"error": "Clock entry not found"})),
    }
}

async fn list_records(State(state): State<Arc<RwLock<TimeclockState>>>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    let items: Vec<&TimeRecord> = s.records.values().collect();
    Json(serde_json::json!({"records": items}))
}

async fn create_record(State(state): State<Arc<RwLock<TimeclockState>>>, Json(mut rec): Json<TimeRecord>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    let id = Uuid::new_v4();
    rec.id = id;
    rec.status = "Active".to_string();
    rec.created_at = Utc::now().to_rfc3339();
    s.records.insert(id, rec.clone());
    Json(serde_json::json!({"record": rec}))
}

async fn get_record(State(state): State<Arc<RwLock<TimeclockState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    match s.records.get(&id) {
        Some(r) => Json(serde_json::json!({"record": r})),
        None => Json(serde_json::json!({"error": "Record not found"})),
    }
}

async fn update_record(State(state): State<Arc<RwLock<TimeclockState>>>, Path(id): Path<Uuid>, Json(rec): Json<TimeRecord>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    if let Some(existing) = s.records.get_mut(&id) {
        *existing = rec.clone();
        existing.id = id;
        Json(serde_json::json!({"record": existing.clone()}))
    } else {
        Json(serde_json::json!({"error": "Record not found"}))
    }
}

async fn list_overtime(State(state): State<Arc<RwLock<TimeclockState>>>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    let items: Vec<&OvertimeRequest> = s.overtime_requests.values().collect();
    Json(serde_json::json!({"overtime_requests": items}))
}

async fn create_overtime(State(state): State<Arc<RwLock<TimeclockState>>>, Json(mut ot): Json<OvertimeRequest>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    let id = Uuid::new_v4();
    ot.id = id;
    ot.status = "Pending".to_string();
    ot.created_at = Utc::now().to_rfc3339();
    s.overtime_requests.insert(id, ot.clone());
    Json(serde_json::json!({"overtime_request": ot}))
}

async fn get_overtime(State(state): State<Arc<RwLock<TimeclockState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    match s.overtime_requests.get(&id) {
        Some(ot) => Json(serde_json::json!({"overtime_request": ot})),
        None => Json(serde_json::json!({"error": "Overtime request not found"})),
    }
}

async fn update_overtime(State(state): State<Arc<RwLock<TimeclockState>>>, Path(id): Path<Uuid>, Json(ot): Json<OvertimeRequest>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    if let Some(existing) = s.overtime_requests.get_mut(&id) {
        *existing = ot.clone();
        existing.id = id;
        Json(serde_json::json!({"overtime_request": existing.clone()}))
    } else {
        Json(serde_json::json!({"error": "Overtime request not found"}))
    }
}

async fn list_reports(State(state): State<Arc<RwLock<TimeclockState>>>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    let items: Vec<&TimeclockReport> = s.reports.values().collect();
    Json(serde_json::json!({"reports": items}))
}

async fn create_report(State(state): State<Arc<RwLock<TimeclockState>>>, Json(mut report): Json<TimeclockReport>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    let id = Uuid::new_v4();
    report.id = id;
    report.created_at = Utc::now().to_rfc3339();
    s.reports.insert(id, report.clone());
    Json(serde_json::json!({"report": report}))
}

async fn get_report(State(state): State<Arc<RwLock<TimeclockState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    match s.reports.get(&id) {
        Some(r) => Json(serde_json::json!({"report": r})),
        None => Json(serde_json::json!({"error": "Report not found"})),
    }
}

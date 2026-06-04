use axum::{
    extract::{Path, State as AxumState},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ClockStatus {
    Normal,
    Overtime,
    Absent,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OvertimeStatus {
    Pending,
    Approved,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClockEntry {
    pub id: Uuid,
    pub employee_name: String,
    pub clock_in: DateTime<Utc>,
    pub clock_out: Option<DateTime<Utc>>,
    pub hours_worked: Option<f64>,
    pub status: ClockStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OvertimeEntry {
    pub id: Uuid,
    pub employee_name: String,
    pub date: DateTime<Utc>,
    pub hours: f64,
    pub reason: String,
    pub status: OvertimeStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeReport {
    pub employee_name: String,
    pub total_hours: f64,
    pub total_overtime: f64,
    pub entries_count: u32,
}

#[derive(Debug, Deserialize)]
pub struct ClockInRequest {
    pub employee_name: String,
}

#[derive(Debug, Deserialize)]
pub struct ClockOutRequest {
    pub employee_name: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateOvertimeRequest {
    pub employee_name: String,
    pub hours: f64,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct ApproveOvertimeRequest {
    pub status: OvertimeStatus,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

#[derive(Clone)]
pub struct TimeclockState {
    pub clock_entries: Arc<RwLock<HashMap<String, ClockEntry>>>,
    pub overtime_entries: Arc<RwLock<Vec<OvertimeEntry>>>,
    pub reports: Arc<RwLock<HashMap<String, TimeReport>>>,
}

impl TimeclockState {
    pub fn new() -> Self {
        Self {
            clock_entries: Arc::new(RwLock::new(HashMap::new())),
            overtime_entries: Arc::new(RwLock::new(Vec::new())),
            reports: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

pub fn routes() -> Router {
    let state = TimeclockState::new();
    Router::new()
        .route("/api/timeclock/clock", post(clock_action))
        .route("/api/timeclock/records", get(list_records))
        .route("/api/timeclock/overtime", get(list_overtime).post(create_overtime))
        .route("/api/timeclock/overtime/:id/approve", post(approve_overtime))
        .route("/api/timeclock/reports", get(list_reports))
        .with_state(state)
}

async fn clock_action(
    AxumState(state): AxumState<TimeclockState>,
    Json(payload): Json<ClockInRequest>,
) -> Result<Json<ApiResponse<ClockEntry>>, StatusCode> {
    let mut entries = state
        .clock_entries
        .write()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if let Some(entry) = entries.get_mut(&payload.employee_name) {
        if entry.clock_out.is_none() {
            entry.clock_out = Some(Utc::now());
            let duration = Utc::now() - entry.clock_in;
            entry.hours_worked = Some(duration.num_seconds() as f64 / 3600.0);
            entry.status = if entry.hours_worked.unwrap_or(0.0) > 8.0 {
                ClockStatus::Overtime
            } else {
                ClockStatus::Normal
            };
            return Ok(Json(ApiResponse {
                success: true,
                data: Some(entry.clone()),
                error: None,
            }));
        }
    }
    let entry = ClockEntry {
        id: Uuid::new_v4(),
        employee_name: payload.employee_name.clone(),
        clock_in: Utc::now(),
        clock_out: None,
        hours_worked: None,
        status: ClockStatus::Normal,
    };
    entries.insert(payload.employee_name, entry.clone());
    Ok(Json(ApiResponse {
        success: true,
        data: Some(entry),
        error: None,
    }))
}

async fn list_records(
    AxumState(state): AxumState<TimeclockState>,
) -> Result<Json<ApiResponse<Vec<ClockEntry>>>, StatusCode> {
    let entries = state
        .clock_entries
        .read()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(ApiResponse {
        success: true,
        data: Some(entries.values().cloned().collect()),
        error: None,
    }))
}

async fn list_overtime(
    AxumState(state): AxumState<TimeclockState>,
) -> Result<Json<ApiResponse<Vec<OvertimeEntry>>>, StatusCode> {
    let entries = state
        .overtime_entries
        .read()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(ApiResponse {
        success: true,
        data: Some(entries.clone()),
        error: None,
    }))
}

async fn create_overtime(
    AxumState(state): AxumState<TimeclockState>,
    Json(payload): Json<CreateOvertimeRequest>,
) -> Result<(StatusCode, Json<ApiResponse<OvertimeEntry>>), StatusCode> {
    let entry = OvertimeEntry {
        id: Uuid::new_v4(),
        employee_name: payload.employee_name,
        date: Utc::now(),
        hours: payload.hours,
        reason: payload.reason,
        status: OvertimeStatus::Pending,
    };
    let mut entries = state
        .overtime_entries
        .write()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    entries.push(entry.clone());
    Ok((StatusCode::CREATED, Json(ApiResponse {
        success: true,
        data: Some(entry),
        error: None,
    })))
}

async fn approve_overtime(
    AxumState(state): AxumState<TimeclockState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ApproveOvertimeRequest>,
) -> Result<Json<ApiResponse<OvertimeEntry>>, StatusCode> {
    let mut entries = state
        .overtime_entries
        .write()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let entry = entries
        .iter_mut()
        .find(|e| e.id == id)
        .ok_or(StatusCode::NOT_FOUND)?;
    entry.status = payload.status;
    Ok(Json(ApiResponse {
        success: true,
        data: Some(entry.clone()),
        error: None,
    }))
}

async fn list_reports(
    AxumState(state): AxumState<TimeclockState>,
) -> Result<Json<ApiResponse<Vec<TimeReport>>>, StatusCode> {
    let reports = state
        .reports
        .read()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(ApiResponse {
        success: true,
        data: Some(reports.values().cloned().collect()),
        error: None,
    }))
}

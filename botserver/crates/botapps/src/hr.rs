use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use axum::extract::{Path, State as AxumState};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmployeeStatus {
    Active,
    OnLeave,
    Terminated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobStatus {
    Open,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Employee {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub department: String,
    pub role: String,
    pub status: EmployeeStatus,
    pub hired_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobPosting {
    pub id: Uuid,
    pub title: String,
    pub department: String,
    pub candidates_count: i32,
    pub status: JobStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttendanceRecord {
    pub employee_email: String,
    pub date: String,
    pub clock_in: String,
    pub clock_out: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEmployee {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub department: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateEmployee {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub department: Option<String>,
    pub role: Option<String>,
    pub status: Option<EmployeeStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateJobPosting {
    pub title: String,
    pub department: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClockInRequest {
    pub employee_email: String,
    pub date: String,
    pub clock_in: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClockOutRequest {
    pub employee_email: String,
    pub date: String,
    pub clock_out: String,
}

#[derive(Clone)]
pub struct HrState {
    pub employees: Arc<RwLock<HashMap<Uuid, Employee>>>,
    pub job_postings: Arc<RwLock<HashMap<Uuid, JobPosting>>>,
    pub attendance: Arc<RwLock<HashMap<String, AttendanceRecord>>>,
}

impl HrState {
    pub fn new() -> Self {
        Self {
            employees: Arc::new(RwLock::new(HashMap::new())),
            job_postings: Arc::new(RwLock::new(HashMap::new())),
            attendance: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: T,
}

async fn list_employees(AxumState(state): AxumState<HrState>) -> Json<ApiResponse<Vec<Employee>>> {
    let employees = state.employees.read().unwrap().values().cloned().collect();
    Json(ApiResponse { success: true, data: employees })
}

async fn create_employee(
    AxumState(state): AxumState<HrState>,
    Json(payload): Json<CreateEmployee>,
) -> Json<ApiResponse<Employee>> {
    let employee = Employee {
        id: Uuid::new_v4(),
        first_name: payload.first_name,
        last_name: payload.last_name,
        email: payload.email,
        department: payload.department,
        role: payload.role,
        status: EmployeeStatus::Active,
        hired_at: Utc::now(),
    };
    state.employees.write().unwrap().insert(employee.id, employee.clone());
    Json(ApiResponse { success: true, data: employee })
}

async fn update_employee(
    AxumState(state): AxumState<HrState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateEmployee>,
) -> Json<ApiResponse<Employee>> {
    let mut employees = state.employees.write().unwrap();
    let employee = employees.get_mut(&id).expect("Employee not found");
    if let Some(first_name) = payload.first_name { employee.first_name = first_name; }
    if let Some(last_name) = payload.last_name { employee.last_name = last_name; }
    if let Some(email) = payload.email { employee.email = email; }
    if let Some(department) = payload.department { employee.department = department; }
    if let Some(role) = payload.role { employee.role = role; }
    if let Some(status) = payload.status { employee.status = status; }
    Json(ApiResponse { success: true, data: employee.clone() })
}

async fn list_recruitment(AxumState(state): AxumState<HrState>) -> Json<ApiResponse<Vec<JobPosting>>> {
    let postings = state.job_postings.read().unwrap().values().cloned().collect();
    Json(ApiResponse { success: true, data: postings })
}

async fn create_job_posting(
    AxumState(state): AxumState<HrState>,
    Json(payload): Json<CreateJobPosting>,
) -> Json<ApiResponse<JobPosting>> {
    let posting = JobPosting {
        id: Uuid::new_v4(),
        title: payload.title,
        department: payload.department,
        candidates_count: 0,
        status: JobStatus::Open,
    };
    state.job_postings.write().unwrap().insert(posting.id, posting.clone());
    Json(ApiResponse { success: true, data: posting })
}

async fn list_attendance(AxumState(state): AxumState<HrState>) -> Json<ApiResponse<Vec<AttendanceRecord>>> {
    let records = state.attendance.read().unwrap().values().cloned().collect();
    Json(ApiResponse { success: true, data: records })
}

async fn clock_in(
    AxumState(state): AxumState<HrState>,
    Json(payload): Json<ClockInRequest>,
) -> Json<ApiResponse<AttendanceRecord>> {
    let key = format!("{}:{}", payload.employee_email, payload.date);
    let record = AttendanceRecord {
        employee_email: payload.employee_email,
        date: payload.date,
        clock_in: payload.clock_in,
        clock_out: None,
    };
    state.attendance.write().unwrap().insert(key, record.clone());
    Json(ApiResponse { success: true, data: record })
}

async fn clock_out(
    AxumState(state): AxumState<HrState>,
    Json(payload): Json<ClockOutRequest>,
) -> Json<ApiResponse<AttendanceRecord>> {
    let key = format!("{}:{}", payload.employee_email, payload.date);
    let mut attendance = state.attendance.write().unwrap();
    let record = attendance.get_mut(&key).expect("No clock-in record found");
    record.clock_out = Some(payload.clock_out);
    Json(ApiResponse { success: true, data: record.clone() })
}

pub fn routes() -> Router {
    let state = HrState::new();
    Router::new()
        .route("/api/hr/employees", get(list_employees).post(create_employee))
        .route("/api/hr/employees/{id}", put(update_employee))
        .route("/api/hr/recruitment", get(list_recruitment).post(create_job_posting))
        .route("/api/hr/attendance", get(list_attendance).post(clock_in))
        .route("/api/hr/attendance/clock-out", post(clock_out))
        .with_state(state)
}

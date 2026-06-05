use axum::{extract::{State, Json, Path}, routing::get, Router};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Employee {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub department: String,
    pub position: String,
    pub hire_date: String,
    pub status: String,
    pub manager_id: Option<Uuid>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Recruitment {
    pub id: Uuid,
    pub position: String,
    pub department: String,
    pub description: String,
    pub salary_range: String,
    pub status: String,
    pub candidates_count: u32,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Attendance {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub date: String,
    pub clock_in: String,
    pub clock_out: Option<String>,
    pub hours_worked: f64,
    pub overtime_hours: f64,
    pub status: String,
    pub created_at: String,
}

#[derive(Default)]
pub struct HrState {
    pub employees: HashMap<Uuid, Employee>,
    pub recruitments: HashMap<Uuid, Recruitment>,
    pub attendance: HashMap<Uuid, Attendance>,
}

pub fn routes() -> axum::Router {
    let state = Arc::new(RwLock::new(HrState::default()));
    Router::new()
        .route("/api/hr/employees", get(list_employees).post(create_employee))
        .route("/api/hr/employees/{id}", get(get_employee).put(update_employee).delete(delete_employee))
        .route("/api/hr/recruitment", get(list_recruitments).post(create_recruitment))
        .route("/api/hr/recruitment/{id}", get(get_recruitment).put(update_recruitment).delete(delete_recruitment))
        .route("/api/hr/attendance", get(list_attendance).post(create_attendance))
        .route("/api/hr/attendance/{id}", get(get_attendance).put(update_attendance))
        .with_state(state)
}

async fn list_employees(State(state): State<Arc<RwLock<HrState>>>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    let items: Vec<&Employee> = s.employees.values().collect();
    Json(serde_json::json!({"employees": items}))
}

async fn create_employee(State(state): State<Arc<RwLock<HrState>>>, Json(mut emp): Json<Employee>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    let id = Uuid::new_v4();
    emp.id = id;
    emp.status = "Active".to_string();
    emp.created_at = Utc::now().to_rfc3339();
    s.employees.insert(id, emp.clone());
    Json(serde_json::json!({"employee": emp}))
}

async fn get_employee(State(state): State<Arc<RwLock<HrState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    match s.employees.get(&id) {
        Some(e) => Json(serde_json::json!({"employee": e})),
        None => Json(serde_json::json!({"error": "Employee not found"})),
    }
}

async fn update_employee(State(state): State<Arc<RwLock<HrState>>>, Path(id): Path<Uuid>, Json(emp): Json<Employee>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    if let Some(existing) = s.employees.get_mut(&id) {
        *existing = emp.clone();
        existing.id = id;
        Json(serde_json::json!({"employee": existing.clone()}))
    } else {
        Json(serde_json::json!({"error": "Employee not found"}))
    }
}

async fn delete_employee(State(state): State<Arc<RwLock<HrState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    s.employees.remove(&id);
    Json(serde_json::json!({"deleted": true}))
}

async fn list_recruitments(State(state): State<Arc<RwLock<HrState>>>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    let items: Vec<&Recruitment> = s.recruitments.values().collect();
    Json(serde_json::json!({"recruitments": items}))
}

async fn create_recruitment(State(state): State<Arc<RwLock<HrState>>>, Json(mut rec): Json<Recruitment>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    let id = Uuid::new_v4();
    rec.id = id;
    rec.status = "Open".to_string();
    rec.candidates_count = 0;
    rec.created_at = Utc::now().to_rfc3339();
    s.recruitments.insert(id, rec.clone());
    Json(serde_json::json!({"recruitment": rec}))
}

async fn get_recruitment(State(state): State<Arc<RwLock<HrState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    match s.recruitments.get(&id) {
        Some(r) => Json(serde_json::json!({"recruitment": r})),
        None => Json(serde_json::json!({"error": "Recruitment not found"})),
    }
}

async fn update_recruitment(State(state): State<Arc<RwLock<HrState>>>, Path(id): Path<Uuid>, Json(rec): Json<Recruitment>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    if let Some(existing) = s.recruitments.get_mut(&id) {
        *existing = rec.clone();
        existing.id = id;
        Json(serde_json::json!({"recruitment": existing.clone()}))
    } else {
        Json(serde_json::json!({"error": "Recruitment not found"}))
    }
}

async fn delete_recruitment(State(state): State<Arc<RwLock<HrState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    s.recruitments.remove(&id);
    Json(serde_json::json!({"deleted": true}))
}

async fn list_attendance(State(state): State<Arc<RwLock<HrState>>>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    let items: Vec<&Attendance> = s.attendance.values().collect();
    Json(serde_json::json!({"attendance": items}))
}

async fn create_attendance(State(state): State<Arc<RwLock<HrState>>>, Json(mut att): Json<Attendance>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    let id = Uuid::new_v4();
    att.id = id;
    att.status = "Active".to_string();
    att.created_at = Utc::now().to_rfc3339();
    s.attendance.insert(id, att.clone());
    Json(serde_json::json!({"attendance": att}))
}

async fn get_attendance(State(state): State<Arc<RwLock<HrState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    match s.attendance.get(&id) {
        Some(a) => Json(serde_json::json!({"attendance": a})),
        None => Json(serde_json::json!({"error": "Attendance record not found"})),
    }
}

async fn update_attendance(State(state): State<Arc<RwLock<HrState>>>, Path(id): Path<Uuid>, Json(att): Json<Attendance>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    if let Some(existing) = s.attendance.get_mut(&id) {
        *existing = att.clone();
        existing.id = id;
        Json(serde_json::json!({"attendance": existing.clone()}))
    } else {
        Json(serde_json::json!({"error": "Attendance record not found"}))
    }
}

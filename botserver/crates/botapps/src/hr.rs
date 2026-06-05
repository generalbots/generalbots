use axum::extract::{Json, Path};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, OnceLock};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Employee {
    pub id: String,
    pub name: String,
    pub email: String,
    pub department: String,
    pub role: String,
    pub status: String,
    pub hired_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Recruitment {
    pub id: String,
    pub position: String,
    pub department: String,
    pub status: String,
    pub candidates: u64,
    pub opened_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Attendance {
    pub id: String,
    pub employee_id: String,
    pub date: String,
    pub clock_in: String,
    pub clock_out: Option<String>,
    pub hours_worked: f64,
}

#[derive(Default)]
struct AppState {
    employees: HashMap<String, Employee>,
    recruitment: HashMap<String, Recruitment>,
    attendance: HashMap<String, Attendance>,
}

fn state() -> &'static Arc<RwLock<AppState>> {
    static S: OnceLock<Arc<RwLock<AppState>>> = OnceLock::new();
    S.get_or_init(|| Arc::new(RwLock::new(AppState::default())))
}

pub async fn list_employees() -> Json<serde_json::Value> {
    let s = state().read().unwrap();
    let items: Vec<&Employee> = s.employees.values().collect();
    Json(serde_json::json!({"items": items}))
}

pub async fn create_employee(Json(item): Json<Employee>) -> Json<serde_json::Value> {
    let mut s = state().write().unwrap();
    let id = uuid::Uuid::new_v4().to_string();
    let mut new_item = item;
    new_item.id = id.clone();
    new_item.hired_at = chrono::Utc::now().to_rfc3339();
    new_item.status = "active".to_string();
    s.employees.insert(id.clone(), new_item.clone());
    Json(serde_json::json!({"item": new_item}))
}

pub async fn update_employee(Path(id): Path<String>, Json(item): Json<Employee>) -> Json<serde_json::Value> {
    let mut s = state().write().unwrap();
    if let Some(existing) = s.employees.get_mut(&id) {
        existing.name = item.name;
        existing.email = item.email;
        existing.department = item.department;
        existing.role = item.role;
        existing.status = item.status;
        Json(serde_json::json!({"item": existing}))
    } else {
        Json(serde_json::json!({"error": "Employee not found"}))
    }
}

pub async fn list_recruitment() -> Json<serde_json::Value> {
    let s = state().read().unwrap();
    let items: Vec<&Recruitment> = s.recruitment.values().collect();
    Json(serde_json::json!({"items": items}))
}

pub async fn list_attendance() -> Json<serde_json::Value> {
    let s = state().read().unwrap();
    let items: Vec<&Attendance> = s.attendance.values().collect();
    Json(serde_json::json!({"items": items}))
}

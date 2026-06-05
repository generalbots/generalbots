use axum::extract::{Json, Path};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, OnceLock};
use axum::http::StatusCode;

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

pub async fn list_employees() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let s = state().read().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {}", e)))?;
    let items: Vec<&Employee> = s.employees.values().collect();
    Ok(Json(serde_json::json!({"items": items})))
}

pub async fn create_employee(Json(item): Json<Employee>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut s = state().write().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {}", e)))?;
    let id = uuid::Uuid::new_v4().to_string();
    let mut new_item = item;
    new_item.id = id.clone();
    new_item.hired_at = chrono::Utc::now().to_rfc3339();
    new_item.status = "active".to_string();
    s.employees.insert(id.clone(), new_item.clone());
    Ok(Json(serde_json::json!({"item": new_item})))
}

pub async fn update_employee(Path(id): Path<String>, Json(item): Json<Employee>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut s = state().write().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {}", e)))?;
    if let Some(existing) = s.employees.get_mut(&id) {
        existing.name = item.name;
        existing.email = item.email;
        existing.department = item.department;
        existing.role = item.role;
        existing.status = item.status;
        Ok(Json(serde_json::json!({"item": existing})))
    } else {
        Err((StatusCode::NOT_FOUND, "Employee not found".to_string()))
    }
}

pub async fn list_recruitment() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let s = state().read().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {}", e)))?;
    let items: Vec<&Recruitment> = s.recruitment.values().collect();
    Ok(Json(serde_json::json!({"items": items})))
}

pub async fn list_attendance() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let s = state().read().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {}", e)))?;
    let items: Vec<&Attendance> = s.attendance.values().collect();
    Ok(Json(serde_json::json!({"items": items})))
}

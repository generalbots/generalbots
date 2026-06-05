use axum::extract::{Json, Path};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, OnceLock};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Incident {
    pub id: String,
    pub title: String,
    pub description: String,
    pub severity: String,
    pub status: String,
    pub assignee: Option<String>,
    pub created_at: String,
    pub resolved_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServiceRequest {
    pub id: String,
    pub title: String,
    pub description: String,
    pub category: String,
    pub status: String,
    pub requester: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CmdbItem {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub owner: String,
    pub status: String,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KbArticle {
    pub id: String,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub author: String,
    pub created_at: String,
}

#[derive(Default)]
struct AppState {
    incidents: HashMap<String, Incident>,
    requests: HashMap<String, ServiceRequest>,
    cmdb: HashMap<String, CmdbItem>,
    kb: HashMap<String, KbArticle>,
}

fn state() -> &'static Arc<RwLock<AppState>> {
    static S: OnceLock<Arc<RwLock<AppState>>> = OnceLock::new();
    S.get_or_init(|| Arc::new(RwLock::new(AppState::default())))
}

pub async fn list_incidents() -> Json<serde_json::Value> {
    let s = state().read().unwrap();
    let items: Vec<&Incident> = s.incidents.values().collect();
    Json(serde_json::json!({"items": items}))
}

pub async fn create_incident(Json(item): Json<Incident>) -> Json<serde_json::Value> {
    let mut s = state().write().unwrap();
    let id = uuid::Uuid::new_v4().to_string();
    let mut new_item = item;
    new_item.id = id.clone();
    new_item.created_at = chrono::Utc::now().to_rfc3339();
    new_item.status = "open".to_string();
    s.incidents.insert(id.clone(), new_item.clone());
    Json(serde_json::json!({"item": new_item}))
}

pub async fn update_incident(Path(id): Path<String>, Json(item): Json<Incident>) -> Json<serde_json::Value> {
    let mut s = state().write().unwrap();
    if let Some(existing) = s.incidents.get_mut(&id) {
        existing.title = item.title;
        existing.description = item.description;
        existing.severity = item.severity;
        existing.status = item.status;
        existing.assignee = item.assignee;
        Json(serde_json::json!({"item": existing}))
    } else {
        Json(serde_json::json!({"error": "Incident not found"}))
    }
}

pub async fn list_requests() -> Json<serde_json::Value> {
    let s = state().read().unwrap();
    let items: Vec<&ServiceRequest> = s.requests.values().collect();
    Json(serde_json::json!({"items": items}))
}

pub async fn create_request(Json(item): Json<ServiceRequest>) -> Json<serde_json::Value> {
    let mut s = state().write().unwrap();
    let id = uuid::Uuid::new_v4().to_string();
    let mut new_item = item;
    new_item.id = id.clone();
    new_item.created_at = chrono::Utc::now().to_rfc3339();
    new_item.status = "open".to_string();
    s.requests.insert(id.clone(), new_item.clone());
    Json(serde_json::json!({"item": new_item}))
}

pub async fn list_cmdb() -> Json<serde_json::Value> {
    let s = state().read().unwrap();
    let items: Vec<&CmdbItem> = s.cmdb.values().collect();
    Json(serde_json::json!({"items": items}))
}

pub async fn list_kb() -> Json<serde_json::Value> {
    let s = state().read().unwrap();
    let items: Vec<&KbArticle> = s.kb.values().collect();
    Json(serde_json::json!({"items": items}))
}

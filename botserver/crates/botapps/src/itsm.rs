use axum::{extract::{State, Json, Path}, routing::get, Router};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Incident {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub priority: String,
    pub severity: String,
    pub status: String,
    pub assigned_to: String,
    pub service: String,
    pub resolution: Option<String>,
    pub created_at: String,
    pub resolved_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServiceRequest {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub category: String,
    pub requested_by: String,
    pub status: String,
    pub approval_status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CmdbItem {
    pub id: Uuid,
    pub name: String,
    pub item_type: String,
    pub category: String,
    pub status: String,
    pub owner: String,
    pub location: String,
    pub attributes: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KbArticle {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub category: String,
    pub author: String,
    pub views: u64,
    pub helpful_count: u64,
    pub published: bool,
    pub created_at: String,
}

#[derive(Default)]
pub struct ItsmState {
    pub incidents: HashMap<Uuid, Incident>,
    pub requests: HashMap<Uuid, ServiceRequest>,
    pub cmdb_items: HashMap<Uuid, CmdbItem>,
    pub kb_articles: HashMap<Uuid, KbArticle>,
}

pub fn routes() -> axum::Router {
    let state = Arc::new(RwLock::new(ItsmState::default()));
    Router::new()
        .route("/api/itsm/incidents", get(list_incidents).post(create_incident))
        .route("/api/itsm/incidents/{id}", get(get_incident).put(update_incident).delete(delete_incident))
        .route("/api/itsm/requests", get(list_requests).post(create_request))
        .route("/api/itsm/requests/{id}", get(get_request).put(update_request).delete(delete_request))
        .route("/api/itsm/cmdb", get(list_cmdb).post(create_cmdb_item))
        .route("/api/itsm/cmdb/{id}", get(get_cmdb_item).put(update_cmdb_item).delete(delete_cmdb_item))
        .route("/api/itsm/kb", get(list_kb).post(create_kb_article))
        .route("/api/itsm/kb/{id}", get(get_kb_article).put(update_kb_article).delete(delete_kb_article))
        .with_state(state)
}

async fn list_incidents(State(state): State<Arc<RwLock<ItsmState>>>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    let items: Vec<&Incident> = s.incidents.values().collect();
    Json(serde_json::json!({"incidents": items}))
}

async fn create_incident(State(state): State<Arc<RwLock<ItsmState>>>, Json(mut inc): Json<Incident>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    let id = Uuid::new_v4();
    inc.id = id;
    inc.status = "Open".to_string();
    inc.created_at = Utc::now().to_rfc3339();
    s.incidents.insert(id, inc.clone());
    Json(serde_json::json!({"incident": inc}))
}

async fn get_incident(State(state): State<Arc<RwLock<ItsmState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    match s.incidents.get(&id) {
        Some(i) => Json(serde_json::json!({"incident": i})),
        None => Json(serde_json::json!({"error": "Incident not found"})),
    }
}

async fn update_incident(State(state): State<Arc<RwLock<ItsmState>>>, Path(id): Path<Uuid>, Json(inc): Json<Incident>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    if let Some(existing) = s.incidents.get_mut(&id) {
        *existing = inc.clone();
        existing.id = id;
        Json(serde_json::json!({"incident": existing.clone()}))
    } else {
        Json(serde_json::json!({"error": "Incident not found"}))
    }
}

async fn delete_incident(State(state): State<Arc<RwLock<ItsmState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    s.incidents.remove(&id);
    Json(serde_json::json!({"deleted": true}))
}

async fn list_requests(State(state): State<Arc<RwLock<ItsmState>>>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    let items: Vec<&ServiceRequest> = s.requests.values().collect();
    Json(serde_json::json!({"requests": items}))
}

async fn create_request(State(state): State<Arc<RwLock<ItsmState>>>, Json(mut req): Json<ServiceRequest>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    let id = Uuid::new_v4();
    req.id = id;
    req.status = "Submitted".to_string();
    req.approval_status = "Pending".to_string();
    req.created_at = Utc::now().to_rfc3339();
    s.requests.insert(id, req.clone());
    Json(serde_json::json!({"request": req}))
}

async fn get_request(State(state): State<Arc<RwLock<ItsmState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    match s.requests.get(&id) {
        Some(r) => Json(serde_json::json!({"request": r})),
        None => Json(serde_json::json!({"error": "Request not found"})),
    }
}

async fn update_request(State(state): State<Arc<RwLock<ItsmState>>>, Path(id): Path<Uuid>, Json(req): Json<ServiceRequest>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    if let Some(existing) = s.requests.get_mut(&id) {
        *existing = req.clone();
        existing.id = id;
        Json(serde_json::json!({"request": existing.clone()}))
    } else {
        Json(serde_json::json!({"error": "Request not found"}))
    }
}

async fn delete_request(State(state): State<Arc<RwLock<ItsmState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    s.requests.remove(&id);
    Json(serde_json::json!({"deleted": true}))
}

async fn list_cmdb(State(state): State<Arc<RwLock<ItsmState>>>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    let items: Vec<&CmdbItem> = s.cmdb_items.values().collect();
    Json(serde_json::json!({"cmdb_items": items}))
}

async fn create_cmdb_item(State(state): State<Arc<RwLock<ItsmState>>>, Json(mut item): Json<CmdbItem>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    let id = Uuid::new_v4();
    item.id = id;
    item.status = "Active".to_string();
    item.created_at = Utc::now().to_rfc3339();
    s.cmdb_items.insert(id, item.clone());
    Json(serde_json::json!({"cmdb_item": item}))
}

async fn get_cmdb_item(State(state): State<Arc<RwLock<ItsmState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    match s.cmdb_items.get(&id) {
        Some(i) => Json(serde_json::json!({"cmdb_item": i})),
        None => Json(serde_json::json!({"error": "CMDB item not found"})),
    }
}

async fn update_cmdb_item(State(state): State<Arc<RwLock<ItsmState>>>, Path(id): Path<Uuid>, Json(item): Json<CmdbItem>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    if let Some(existing) = s.cmdb_items.get_mut(&id) {
        *existing = item.clone();
        existing.id = id;
        Json(serde_json::json!({"cmdb_item": existing.clone()}))
    } else {
        Json(serde_json::json!({"error": "CMDB item not found"}))
    }
}

async fn delete_cmdb_item(State(state): State<Arc<RwLock<ItsmState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    s.cmdb_items.remove(&id);
    Json(serde_json::json!({"deleted": true}))
}

async fn list_kb(State(state): State<Arc<RwLock<ItsmState>>>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    let items: Vec<&KbArticle> = s.kb_articles.values().collect();
    Json(serde_json::json!({"kb_articles": items}))
}

async fn create_kb_article(State(state): State<Arc<RwLock<ItsmState>>>, Json(mut article): Json<KbArticle>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    let id = Uuid::new_v4();
    article.id = id;
    article.views = 0;
    article.helpful_count = 0;
    article.published = false;
    article.created_at = Utc::now().to_rfc3339();
    s.kb_articles.insert(id, article.clone());
    Json(serde_json::json!({"kb_article": article}))
}

async fn get_kb_article(State(state): State<Arc<RwLock<ItsmState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    match s.kb_articles.get(&id) {
        Some(a) => Json(serde_json::json!({"kb_article": a})),
        None => Json(serde_json::json!({"error": "KB article not found"})),
    }
}

async fn update_kb_article(State(state): State<Arc<RwLock<ItsmState>>>, Path(id): Path<Uuid>, Json(article): Json<KbArticle>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    if let Some(existing) = s.kb_articles.get_mut(&id) {
        *existing = article.clone();
        existing.id = id;
        Json(serde_json::json!({"kb_article": existing.clone()}))
    } else {
        Json(serde_json::json!({"error": "KB article not found"}))
    }
}

async fn delete_kb_article(State(state): State<Arc<RwLock<ItsmState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    s.kb_articles.remove(&id);
    Json(serde_json::json!({"deleted": true}))
}

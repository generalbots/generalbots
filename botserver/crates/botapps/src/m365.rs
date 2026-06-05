use axum::{extract::{State, Json, Path}, routing::get, Router};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SharePointSite {
    pub id: Uuid,
    pub name: String,
    pub url: String,
    pub description: String,
    pub owner: String,
    pub storage_used_mb: f64,
    pub storage_quota_mb: f64,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CalendarEvent {
    pub id: Uuid,
    pub subject: String,
    pub body: String,
    pub start_time: String,
    pub end_time: String,
    pub location: Option<String>,
    pub attendees: String,
    pub is_recurring: bool,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OneDriveFile {
    pub id: Uuid,
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub file_type: String,
    pub last_modified_at: String,
    pub last_modified_by: String,
    pub download_url: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct M365Settings {
    pub id: Uuid,
    pub tenant_id: String,
    pub client_id: String,
    pub scopes: String,
    pub sync_interval_minutes: u32,
    pub auto_sync: bool,
    pub status: String,
    pub last_sync_at: Option<String>,
    pub created_at: String,
}

#[derive(Default)]
pub struct M365State {
    pub sharepoint_sites: HashMap<Uuid, SharePointSite>,
    pub calendar_events: HashMap<Uuid, CalendarEvent>,
    pub onedrive_files: HashMap<Uuid, OneDriveFile>,
    pub settings: HashMap<Uuid, M365Settings>,
}

pub fn routes() -> axum::Router {
    let state = Arc::new(RwLock::new(M365State::default()));
    Router::new()
        .route("/api/m365/sharepoint", get(list_sharepoint).post(create_sharepoint))
        .route("/api/m365/sharepoint/{id}", get(get_sharepoint).put(update_sharepoint).delete(delete_sharepoint))
        .route("/api/m365/calendar", get(list_calendar).post(create_event))
        .route("/api/m365/calendar/{id}", get(get_event).put(update_event).delete(delete_event))
        .route("/api/m365/onedrive", get(list_onedrive).post(create_file))
        .route("/api/m365/onedrive/{id}", get(get_file).put(update_file).delete(delete_file))
        .route("/api/m365/settings", get(list_settings).post(create_settings))
        .route("/api/m365/settings/{id}", get(get_settings).put(update_settings).delete(delete_settings))
        .with_state(state)
}

async fn list_sharepoint(State(state): State<Arc<RwLock<M365State>>>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    let items: Vec<&SharePointSite> = s.sharepoint_sites.values().collect();
    Json(serde_json::json!({"sharepoint_sites": items}))
}

async fn create_sharepoint(State(state): State<Arc<RwLock<M365State>>>, Json(mut site): Json<SharePointSite>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    let id = Uuid::new_v4();
    site.id = id;
    site.status = "Active".to_string();
    site.created_at = Utc::now().to_rfc3339();
    s.sharepoint_sites.insert(id, site.clone());
    Json(serde_json::json!({"sharepoint_site": site}))
}

async fn get_sharepoint(State(state): State<Arc<RwLock<M365State>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    match s.sharepoint_sites.get(&id) {
        Some(site) => Json(serde_json::json!({"sharepoint_site": site})),
        None => Json(serde_json::json!({"error": "SharePoint site not found"})),
    }
}

async fn update_sharepoint(State(state): State<Arc<RwLock<M365State>>>, Path(id): Path<Uuid>, Json(site): Json<SharePointSite>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    if let Some(existing) = s.sharepoint_sites.get_mut(&id) {
        *existing = site.clone();
        existing.id = id;
        Json(serde_json::json!({"sharepoint_site": existing.clone()}))
    } else {
        Json(serde_json::json!({"error": "SharePoint site not found"}))
    }
}

async fn delete_sharepoint(State(state): State<Arc<RwLock<M365State>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    s.sharepoint_sites.remove(&id);
    Json(serde_json::json!({"deleted": true}))
}

async fn list_calendar(State(state): State<Arc<RwLock<M365State>>>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    let items: Vec<&CalendarEvent> = s.calendar_events.values().collect();
    Json(serde_json::json!({"calendar_events": items}))
}

async fn create_event(State(state): State<Arc<RwLock<M365State>>>, Json(mut event): Json<CalendarEvent>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    let id = Uuid::new_v4();
    event.id = id;
    event.status = "Confirmed".to_string();
    event.created_at = Utc::now().to_rfc3339();
    s.calendar_events.insert(id, event.clone());
    Json(serde_json::json!({"calendar_event": event}))
}

async fn get_event(State(state): State<Arc<RwLock<M365State>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    match s.calendar_events.get(&id) {
        Some(e) => Json(serde_json::json!({"calendar_event": e})),
        None => Json(serde_json::json!({"error": "Calendar event not found"})),
    }
}

async fn update_event(State(state): State<Arc<RwLock<M365State>>>, Path(id): Path<Uuid>, Json(event): Json<CalendarEvent>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    if let Some(existing) = s.calendar_events.get_mut(&id) {
        *existing = event.clone();
        existing.id = id;
        Json(serde_json::json!({"calendar_event": existing.clone()}))
    } else {
        Json(serde_json::json!({"error": "Calendar event not found"}))
    }
}

async fn delete_event(State(state): State<Arc<RwLock<M365State>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    s.calendar_events.remove(&id);
    Json(serde_json::json!({"deleted": true}))
}

async fn list_onedrive(State(state): State<Arc<RwLock<M365State>>>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    let items: Vec<&OneDriveFile> = s.onedrive_files.values().collect();
    Json(serde_json::json!({"onedrive_files": items}))
}

async fn create_file(State(state): State<Arc<RwLock<M365State>>>, Json(mut file): Json<OneDriveFile>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    let id = Uuid::new_v4();
    file.id = id;
    file.created_at = Utc::now().to_rfc3339();
    s.onedrive_files.insert(id, file.clone());
    Json(serde_json::json!({"onedrive_file": file}))
}

async fn get_file(State(state): State<Arc<RwLock<M365State>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    match s.onedrive_files.get(&id) {
        Some(f) => Json(serde_json::json!({"onedrive_file": f})),
        None => Json(serde_json::json!({"error": "OneDrive file not found"})),
    }
}

async fn update_file(State(state): State<Arc<RwLock<M365State>>>, Path(id): Path<Uuid>, Json(file): Json<OneDriveFile>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    if let Some(existing) = s.onedrive_files.get_mut(&id) {
        *existing = file.clone();
        existing.id = id;
        Json(serde_json::json!({"onedrive_file": existing.clone()}))
    } else {
        Json(serde_json::json!({"error": "OneDrive file not found"}))
    }
}

async fn delete_file(State(state): State<Arc<RwLock<M365State>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    s.onedrive_files.remove(&id);
    Json(serde_json::json!({"deleted": true}))
}

async fn list_settings(State(state): State<Arc<RwLock<M365State>>>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    let items: Vec<&M365Settings> = s.settings.values().collect();
    Json(serde_json::json!({"settings": items}))
}

async fn create_settings(State(state): State<Arc<RwLock<M365State>>>, Json(mut settings): Json<M365Settings>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    let id = Uuid::new_v4();
    settings.id = id;
    settings.auto_sync = true;
    settings.status = "Active".to_string();
    settings.created_at = Utc::now().to_rfc3339();
    s.settings.insert(id, settings.clone());
    Json(serde_json::json!({"settings": settings}))
}

async fn get_settings(State(state): State<Arc<RwLock<M365State>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    match s.settings.get(&id) {
        Some(set) => Json(serde_json::json!({"settings": set})),
        None => Json(serde_json::json!({"error": "Settings not found"})),
    }
}

async fn update_settings(State(state): State<Arc<RwLock<M365State>>>, Path(id): Path<Uuid>, Json(settings): Json<M365Settings>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    if let Some(existing) = s.settings.get_mut(&id) {
        *existing = settings.clone();
        existing.id = id;
        Json(serde_json::json!({"settings": existing.clone()}))
    } else {
        Json(serde_json::json!({"error": "Settings not found"}))
    }
}

async fn delete_settings(State(state): State<Arc<RwLock<M365State>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    s.settings.remove(&id);
    Json(serde_json::json!({"deleted": true}))
}

use axum::extract::Json;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, OnceLock};
use axum::http::StatusCode;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SharePointItem {
    pub id: String,
    pub site_name: String,
    pub list_name: String,
    pub item_count: u64,
    pub last_modified: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CalendarEvent {
    pub id: String,
    pub subject: String,
    pub start: String,
    pub end: String,
    pub location: Option<String>,
    pub attendees: Vec<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OneDriveFile {
    pub id: String,
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub last_modified: String,
    pub author: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct M365Settings {
    pub tenant_id: String,
    pub client_id: String,
    pub connected: bool,
    pub scopes: Vec<String>,
    pub last_sync: Option<String>,
}

#[derive(Default)]
struct AppState {
    sharepoint: HashMap<String, SharePointItem>,
    calendar: HashMap<String, CalendarEvent>,
    onedrive: HashMap<String, OneDriveFile>,
    settings: Option<M365Settings>,
}

fn state() -> &'static Arc<RwLock<AppState>> {
    static S: OnceLock<Arc<RwLock<AppState>>> = OnceLock::new();
    S.get_or_init(|| Arc::new(RwLock::new(AppState::default())))
}

pub async fn list_sharepoint() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let s = state().read().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {}", e)))?;
    let items: Vec<&SharePointItem> = s.sharepoint.values().collect();
    Ok(Json(serde_json::json!({"items": items})))
}

pub async fn list_calendar() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let s = state().read().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {}", e)))?;
    let items: Vec<&CalendarEvent> = s.calendar.values().collect();
    Ok(Json(serde_json::json!({"items": items})))
}

pub async fn list_onedrive() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let s = state().read().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {}", e)))?;
    let items: Vec<&OneDriveFile> = s.onedrive.values().collect();
    Ok(Json(serde_json::json!({"items": items})))
}

pub async fn get_settings() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let s = state().read().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {}", e)))?;
    match &s.settings {
        Some(settings) => Ok(Json(serde_json::json!({"settings": settings}))),
        None => Ok(Json(serde_json::json!({"settings": null}))),
    }
}

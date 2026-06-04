use axum::{
    extract::{State as AxumState},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use uuid::Uuid;

#[derive(Default)]
pub struct SharePointSite {
    pub id: Uuid,
    pub name: String,
    pub url: String,
    pub document_count: i32,
    pub last_modified: DateTime<Utc>,
}

#[derive(Default)]
pub struct M365CalendarEvent {
    pub id: Uuid,
    pub subject: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub organizer: String,
    pub location: String,
}

#[derive(Default)]
pub struct OneDriveFile {
    pub id: Uuid,
    pub name: String,
    pub path: String,
    pub size_bytes: i64,
    pub is_folder: bool,
    pub last_modified: DateTime<Utc>,
}

#[derive(Default)]
pub struct M365Settings {
    pub tenant_id: String,
    pub client_id: String,
    pub scopes: Vec<String>,
    pub connected: bool,
}

#[derive(Default)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

#[derive(Default)]
pub struct M365State {
    pub sharepoint_sites: Arc<RwLock<Vec<SharePointSite>>>,
    pub calendar_events: Arc<RwLock<Vec<M365CalendarEvent>>>,
    pub onedrive_files: Arc<RwLock<Vec<OneDriveFile>>>,
    pub settings: Arc<RwLock<M365Settings>>,
}

impl M365State {
    pub fn new() -> Self {
        Self {
            sharepoint_sites: Arc::new(RwLock::new(Vec::new())),
            calendar_events: Arc::new(RwLock::new(Vec::new())),
            onedrive_files: Arc::new(RwLock::new(Vec::new())),
            settings: Arc::new(RwLock::new(M365Settings {
                tenant_id: String::new(),
                client_id: String::new(),
                scopes: vec![
                    "Sites.Read.All".to_string(),
                    "Calendars.Read".to_string(),
                    "Files.Read.All".to_string(),
                ],
                connected: false,
            })),
        }
    }
}

pub fn routes() -> Router {
    let state = M365State::new();
    Router::new()
        .route("/api/m365/sharepoint", get(list_sharepoint_sites))
        .route("/api/m365/calendar", get(list_calendar_events))
        .route("/api/m365/onedrive", get(list_onedrive_files))
        .route("/api/m365/settings", get(get_settings))
        .with_state(state)
}

async fn list_sharepoint_sites(
    AxumState(state): AxumState<M365State>,
) -> Result<Json<ApiResponse<Vec<SharePointSite>>>, StatusCode> {
    let sites = state
        .sharepoint_sites
        .read()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(ApiResponse {
        success: true,
        data: Some(sites.clone()),
        error: None,
    }))
}

async fn list_calendar_events(
    AxumState(state): AxumState<M365State>,
) -> Result<Json<ApiResponse<Vec<M365CalendarEvent>>>, StatusCode> {
    let events = state
        .calendar_events
        .read()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(ApiResponse {
        success: true,
        data: Some(events.clone()),
        error: None,
    }))
}

async fn list_onedrive_files(
    AxumState(state): AxumState<M365State>,
) -> Result<Json<ApiResponse<Vec<OneDriveFile>>>, StatusCode> {
    let files = state
        .onedrive_files
        .read()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(ApiResponse {
        success: true,
        data: Some(files.clone()),
        error: None,
    }))
}

async fn get_settings(
    AxumState(state): AxumState<M365State>,
) -> Result<Json<ApiResponse<M365Settings>>, StatusCode> {
    let settings = state
        .settings
        .read()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(ApiResponse {
        success: true,
        data: Some(settings.clone()),
        error: None,
    }))
}

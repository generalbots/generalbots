use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::m365_auth::{
    AuthorizationUrlRequest, M365Credentials, M365OAuthClient, M365Token, OAuthFlow,
};
use super::outlook::{CalendarEvent, EmailMessage, EmailImportance, EmailRecipient, OutlookService};
use super::sharepoint::{parse_drive_response, SharePointClient, SharePointDrive, SharePointItem, SharePointSite};

mod outlook_handlers;

pub fn configure_m365_api_routes() -> Router<Arc<crate::AppState>> {
    Router::new()
        .route("/api/m365/auth/url", post(build_auth_url))
        .route("/api/m365/auth/exchange", post(exchange_code))
        .route("/api/m365/sites", get(list_sites))
        .route("/api/m365/drives/:site_id", get(list_drives))
        .route("/api/m365/items/:drive_id", get(list_items))
        .route("/api/m365/mail", get(list_messages))
        .route("/api/m365/calendar", get(list_calendar))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BuildAuthUrlRequest {
    pub credentials: M365Credentials,
    pub auth_request: AuthorizationUrlRequest,
    pub flow: OAuthFlow,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BuildAuthUrlResponse {
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExchangeCodeRequest {
    pub credentials: M365Credentials,
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListSitesRequest {
    pub access_token: String,
    pub tenant_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListDrivesRequest {
    pub access_token: String,
    pub tenant_id: String,
    pub site_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListItemsRequest {
    pub access_token: String,
    pub tenant_id: String,
    pub drive_id: String,
    pub path: Option<String>,
}

pub use outlook_handlers::{list_calendar, list_messages, ListCalendarRequest, ListMessagesRequest};

async fn build_auth_url(
    Json(req): Json<BuildAuthUrlRequest>,
) -> Result<Json<BuildAuthUrlResponse>, (StatusCode, String)> {
    let client = M365OAuthClient::new(req.credentials);
    let url = client.authorization_url(req.auth_request);
    Ok(Json(BuildAuthUrlResponse { url }))
}

async fn exchange_code(
    Json(req): Json<ExchangeCodeRequest>,
) -> Result<Json<M365Token>, (StatusCode, String)> {
    let client = M365OAuthClient::new(req.credentials);
    let token = tokio::task::spawn_blocking(move || client.exchange_code(&req.code))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .map_err(|e| (StatusCode::BAD_GATEWAY, e))?;
    Ok(Json(token))
}

async fn list_sites(
    Json(req): Json<ListSitesRequest>,
) -> Result<Json<Vec<SharePointSite>>, (StatusCode, String)> {
    let client = SharePointClient::new(req.tenant_id, req.access_token);
    let url = client.list_sites_url();
    let client_clone = client;
    let result = tokio::task::spawn_blocking(move || -> Result<Vec<SharePointSite>, String> {
        let resp = client_clone
            .http_client
            .get(&url)
            .header("Authorization", client_clone.build_auth_header())
            .send()
            .map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            return Err(format!("HTTP {}", resp.status()));
        }
        let body: serde_json::Value = resp.json().map_err(|e| e.to_string())?;
        let mut sites = Vec::new();
        if let Some(arr) = body.get("value").and_then(|v| v.as_array()) {
            for entry in arr {
                sites.push(SharePointSite {
                    id: entry
                        .get("id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    display_name: entry
                        .get("displayName")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    url: entry
                        .get("webUrl")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    description: entry
                        .get("description")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                    web_template: entry
                        .get("webTemplate")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                    last_modified: entry
                        .get("lastModifiedDateTime")
                        .and_then(|t| t.as_str())
                        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                        .map(|d| d.with_timezone(&chrono::Utc))
                        .unwrap_or_else(chrono::Utc::now),
                });
            }
        }
        Ok(sites)
    })
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .map_err(|e| (StatusCode::BAD_GATEWAY, e))?;
    Ok(Json(result))
}

async fn list_drives(
    Json(req): Json<ListDrivesRequest>,
) -> Result<Json<Vec<SharePointDrive>>, (StatusCode, String)> {
    let client = SharePointClient::new(req.tenant_id, req.access_token);
    let url = client.site_drives_url(&req.site_id);
    let result = tokio::task::spawn_blocking(move || -> Result<Vec<SharePointDrive>, String> {
        let resp = client
            .http_client
            .get(&url)
            .header("Authorization", client.build_auth_header())
            .send()
            .map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            return Err(format!("HTTP {}", resp.status()));
        }
        let body: serde_json::Value = resp.json().map_err(|e| e.to_string())?;
        let mut drives = Vec::new();
        if let Some(arr) = body.get("value").and_then(|v| v.as_array()) {
            for entry in arr {
                drives.push(SharePointDrive {
                    id: entry.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    name: entry.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    description: entry
                        .get("description")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                    drive_type: entry
                        .get("driveType")
                        .and_then(|v| v.as_str())
                        .unwrap_or("documentLibrary")
                        .to_string(),
                    quota_total_bytes: entry
                        .get("quota")
                        .and_then(|q| q.get("total"))
                        .and_then(|v| v.as_i64()),
                    quota_used_bytes: entry
                        .get("quota")
                        .and_then(|q| q.get("used"))
                        .and_then(|v| v.as_i64()),
                });
            }
        }
        Ok(drives)
    })
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .map_err(|e| (StatusCode::BAD_GATEWAY, e))?;
    Ok(Json(result))
}

async fn list_items(
    Json(req): Json<ListItemsRequest>,
) -> Result<Json<Vec<SharePointItem>>, (StatusCode, String)> {
    let client = SharePointClient::new(req.tenant_id, req.access_token);
    let url = client.drive_items_url(&req.drive_id, req.path.as_deref());
    let result = tokio::task::spawn_blocking(move || -> Result<Vec<SharePointItem>, String> {
        let resp = client
            .http_client
            .get(&url)
            .header("Authorization", client.build_auth_header())
            .send()
            .map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            return Err(format!("HTTP {}", resp.status()));
        }
        let body: serde_json::Value = resp.json().map_err(|e| e.to_string())?;
        Ok(parse_drive_response(&body))
    })
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .map_err(|e| (StatusCode::BAD_GATEWAY, e))?;
    Ok(Json(result))
}

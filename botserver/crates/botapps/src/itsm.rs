use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use axum::extract::{Path, State as AxumState};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IncidentStatus {
    Open,
    InProgress,
    Resolved,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequestStatus {
    Pending,
    Approved,
    Fulfilled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Incident {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub priority: Priority,
    pub status: IncidentStatus,
    pub assignee: Option<String>,
    pub sla_deadline: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRequest {
    pub id: Uuid,
    pub title: String,
    pub category: String,
    pub requester: String,
    pub status: RequestStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CmdbAsset {
    pub id: Uuid,
    pub name: String,
    pub asset_type: String,
    pub owner: String,
    pub location: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KbArticle {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateIncident {
    pub title: String,
    pub description: String,
    pub priority: Priority,
    pub assignee: Option<String>,
    pub sla_deadline: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateIncident {
    pub title: Option<String>,
    pub description: Option<String>,
    pub priority: Option<Priority>,
    pub status: Option<IncidentStatus>,
    pub assignee: Option<String>,
    pub sla_deadline: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateServiceRequest {
    pub title: String,
    pub category: String,
    pub requester: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCmdbAsset {
    pub name: String,
    pub asset_type: String,
    pub owner: String,
    pub location: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateKbArticle {
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
}

#[derive(Clone)]
pub struct ItsmState {
    pub incidents: Arc<RwLock<HashMap<Uuid, Incident>>>,
    pub service_requests: Arc<RwLock<HashMap<Uuid, ServiceRequest>>>,
    pub cmdb_assets: Arc<RwLock<HashMap<Uuid, CmdbAsset>>>,
    pub kb_articles: Arc<RwLock<Vec<KbArticle>>>,
}

impl ItsmState {
    pub fn new() -> Self {
        Self {
            incidents: Arc::new(RwLock::new(HashMap::new())),
            service_requests: Arc::new(RwLock::new(HashMap::new())),
            cmdb_assets: Arc::new(RwLock::new(HashMap::new())),
            kb_articles: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: T,
}

async fn list_incidents(AxumState(state): AxumState<ItsmState>) -> Json<ApiResponse<Vec<Incident>>> {
    let incidents = state.incidents.read().unwrap().values().cloned().collect();
    Json(ApiResponse { success: true, data: incidents })
}

async fn create_incident(
    AxumState(state): AxumState<ItsmState>,
    Json(payload): Json<CreateIncident>,
) -> Json<ApiResponse<Incident>> {
    let incident = Incident {
        id: Uuid::new_v4(),
        title: payload.title,
        description: payload.description,
        priority: payload.priority,
        status: IncidentStatus::Open,
        assignee: payload.assignee,
        sla_deadline: payload.sla_deadline,
        created_at: Utc::now(),
    };
    state.incidents.write().unwrap().insert(incident.id, incident.clone());
    Json(ApiResponse { success: true, data: incident })
}

async fn update_incident(
    AxumState(state): AxumState<ItsmState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateIncident>,
) -> Json<ApiResponse<Incident>> {
    let mut incidents = state.incidents.write().unwrap();
    let incident = incidents.get_mut(&id).expect("Incident not found");
    if let Some(title) = payload.title { incident.title = title; }
    if let Some(description) = payload.description { incident.description = description; }
    if let Some(priority) = payload.priority { incident.priority = priority; }
    if let Some(status) = payload.status { incident.status = status; }
    if let Some(assignee) = payload.assignee { incident.assignee = Some(assignee); }
    if let Some(sla_deadline) = payload.sla_deadline { incident.sla_deadline = Some(sla_deadline); }
    Json(ApiResponse { success: true, data: incident.clone() })
}

async fn list_requests(AxumState(state): AxumState<ItsmState>) -> Json<ApiResponse<Vec<ServiceRequest>>> {
    let requests = state.service_requests.read().unwrap().values().cloned().collect();
    Json(ApiResponse { success: true, data: requests })
}

async fn create_request(
    AxumState(state): AxumState<ItsmState>,
    Json(payload): Json<CreateServiceRequest>,
) -> Json<ApiResponse<ServiceRequest>> {
    let request = ServiceRequest {
        id: Uuid::new_v4(),
        title: payload.title,
        category: payload.category,
        requester: payload.requester,
        status: RequestStatus::Pending,
    };
    state.service_requests.write().unwrap().insert(request.id, request.clone());
    Json(ApiResponse { success: true, data: request })
}

async fn list_cmdb(AxumState(state): AxumState<ItsmState>) -> Json<ApiResponse<Vec<CmdbAsset>>> {
    let assets = state.cmdb_assets.read().unwrap().values().cloned().collect();
    Json(ApiResponse { success: true, data: assets })
}

async fn create_cmdb_asset(
    AxumState(state): AxumState<ItsmState>,
    Json(payload): Json<CreateCmdbAsset>,
) -> Json<ApiResponse<CmdbAsset>> {
    let asset = CmdbAsset {
        id: Uuid::new_v4(),
        name: payload.name,
        asset_type: payload.asset_type,
        owner: payload.owner,
        location: payload.location,
    };
    state.cmdb_assets.write().unwrap().insert(asset.id, asset.clone());
    Json(ApiResponse { success: true, data: asset })
}

async fn list_kb(AxumState(state): AxumState<ItsmState>) -> Json<ApiResponse<Vec<KbArticle>>> {
    let articles = state.kb_articles.read().unwrap().clone();
    Json(ApiResponse { success: true, data: articles })
}

async fn create_kb_article(
    AxumState(state): AxumState<ItsmState>,
    Json(payload): Json<CreateKbArticle>,
) -> Json<ApiResponse<KbArticle>> {
    let article = KbArticle {
        id: Uuid::new_v4(),
        title: payload.title,
        content: payload.content,
        tags: payload.tags,
        created_at: Utc::now(),
    };
    state.kb_articles.write().unwrap().push(article.clone());
    Json(ApiResponse { success: true, data: article })
}

pub fn routes() -> Router {
    let state = ItsmState::new();
    Router::new()
        .route("/api/itsm/incidents", get(list_incidents).post(create_incident))
        .route("/api/itsm/incidents/{id}", put(update_incident))
        .route("/api/itsm/requests", get(list_requests).post(create_request))
        .route("/api/itsm/cmdb", get(list_cmdb).post(create_cmdb_asset))
        .route("/api/itsm/kb", get(list_kb).post(create_kb_article))
        .with_state(state)
}

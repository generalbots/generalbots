use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, put},
    Json, Router,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::cmdb::{CmdbService, CreateCIRequest, CreateRelationshipRequest, UpdateCIRequest};
use crate::incident::{CreateIncidentRequest, IncidentQuery, IncidentService, UpdateIncidentRequest};
use crate::knowledge::{CreateKnowledgeArticleRequest, KnowledgeBase, SearchKnowledgeQuery, UpdateKnowledgeArticleRequest};

#[derive(Clone)]
pub struct ItsmState {
    pub incident_service: IncidentService,
    pub cmdb_service: CmdbService,
    pub knowledge_base: KnowledgeBase,
}

impl ItsmState {
    pub fn new() -> Self {
        ItsmState {
            incident_service: IncidentService::new(),
            cmdb_service: CmdbService::new(),
            knowledge_base: KnowledgeBase::new(),
        }
    }
}

// Incident handlers

async fn create_incident(
    State(state): State<Arc<ItsmState>>,
    Json(req): Json<CreateIncidentRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let incident = state
        .incident_service
        .create(req)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    Ok(Json(serde_json::to_value(&incident).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?))
}

async fn get_incident(
    State(state): State<Arc<ItsmState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let incident = state
        .incident_service
        .get(id)
        .map_err(|e| (StatusCode::NOT_FOUND, e))?;
    Ok(Json(serde_json::to_value(&incident).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?))
}

async fn list_incidents(
    State(state): State<Arc<ItsmState>>,
    Query(query): Query<IncidentQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let incidents = state
        .incident_service
        .list(query)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(serde_json::to_value(&incidents).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?))
}

async fn update_incident(
    State(state): State<Arc<ItsmState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateIncidentRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let incident = state
        .incident_service
        .update(id, req)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    Ok(Json(serde_json::to_value(&incident).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?))
}

async fn delete_incident(
    State(state): State<Arc<ItsmState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    state
        .incident_service
        .delete(id)
        .map_err(|e| (StatusCode::NOT_FOUND, e))?;
    Ok(StatusCode::NO_CONTENT)
}

async fn check_incident_sla(
    State(state): State<Arc<ItsmState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let (violated, deadline) = state
        .incident_service
        .check_sla_violation(id)
        .map_err(|e| (StatusCode::NOT_FOUND, e))?;
    Ok(Json(serde_json::json!({
        "incident_id": id,
        "sla_violated": violated,
        "sla_deadline": deadline,
    })))
}

// CMDB handlers

async fn create_ci(
    State(state): State<Arc<ItsmState>>,
    Json(req): Json<CreateCIRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let ci = state
        .cmdb_service
        .create_ci(req)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    Ok(Json(serde_json::to_value(&ci).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?))
}

async fn get_ci(
    State(state): State<Arc<ItsmState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let ci = state
        .cmdb_service
        .get_ci(id)
        .map_err(|e| (StatusCode::NOT_FOUND, e))?;
    Ok(Json(serde_json::to_value(&ci).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?))
}

async fn list_cis(
    State(state): State<Arc<ItsmState>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let cis = state
        .cmdb_service
        .list_cis()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(serde_json::to_value(&cis).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?))
}

async fn update_ci(
    State(state): State<Arc<ItsmState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateCIRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let ci = state
        .cmdb_service
        .update_ci(id, req)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    Ok(Json(serde_json::to_value(&ci).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?))
}

async fn delete_ci(
    State(state): State<Arc<ItsmState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    state
        .cmdb_service
        .delete_ci(id)
        .map_err(|e| (StatusCode::NOT_FOUND, e))?;
    Ok(StatusCode::NO_CONTENT)
}

async fn create_relationship(
    State(state): State<Arc<ItsmState>>,
    Json(req): Json<CreateRelationshipRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let rel = state
        .cmdb_service
        .create_relationship(req)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    Ok(Json(serde_json::to_value(&rel).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?))
}

async fn list_relationships(
    State(state): State<Arc<ItsmState>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let rels = state
        .cmdb_service
        .list_relationships()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(serde_json::to_value(&rels).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?))
}

async fn get_ci_relationships(
    State(state): State<Arc<ItsmState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let rels = state
        .cmdb_service
        .get_relationships_for_ci(id)
        .map_err(|e| (StatusCode::NOT_FOUND, e))?;
    Ok(Json(serde_json::to_value(&rels).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?))
}

async fn impact_analysis(
    State(state): State<Arc<ItsmState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let analysis = state
        .cmdb_service
        .impact_analysis(id)
        .map_err(|e| (StatusCode::NOT_FOUND, e))?;
    Ok(Json(serde_json::to_value(&analysis).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?))
}

// Knowledge base handlers

async fn create_article(
    State(state): State<Arc<ItsmState>>,
    Json(req): Json<CreateKnowledgeArticleRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let article = state
        .knowledge_base
        .create(req)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    Ok(Json(serde_json::to_value(&article).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?))
}

async fn get_article(
    State(state): State<Arc<ItsmState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let article = state
        .knowledge_base
        .get(id)
        .map_err(|e| (StatusCode::NOT_FOUND, e))?;
    Ok(Json(serde_json::to_value(&article).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?))
}

async fn update_article(
    State(state): State<Arc<ItsmState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateKnowledgeArticleRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let article = state
        .knowledge_base
        .update(id, req)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    Ok(Json(serde_json::to_value(&article).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?))
}

async fn approve_article(
    State(state): State<Arc<ItsmState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let article = state
        .knowledge_base
        .approve(id)
        .map_err(|e| (StatusCode::NOT_FOUND, e))?;
    Ok(Json(serde_json::to_value(&article).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?))
}

async fn search_articles(
    State(state): State<Arc<ItsmState>>,
    Query(query): Query<SearchKnowledgeQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let articles = state
        .knowledge_base
        .search(query)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(serde_json::to_value(&articles).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?))
}

pub fn configure_itsm_routes() -> Router<Arc<ItsmState>> {
    Router::new()
        // Incident routes
        .route("/api/itsm/incidents", get(list_incidents).post(create_incident))
        .route("/api/itsm/incidents/{id}", get(get_incident).put(update_incident).delete(delete_incident))
        .route("/api/itsm/incidents/{id}/sla", get(check_incident_sla))
        // CMDB routes
        .route("/api/itsm/cmdb", get(list_cis).post(create_ci))
        .route("/api/itsm/cmdb/{id}", get(get_ci).put(update_ci).delete(delete_ci))
        .route("/api/itsm/cmdb/{id}/relationships", get(get_ci_relationships))
        .route("/api/itsm/cmdb/{id}/impact", get(impact_analysis))
        .route("/api/itsm/cmdb/relationships", get(list_relationships).post(create_relationship))
        // Knowledge base routes
        .route("/api/itsm/knowledge", get(search_articles).post(create_article))
        .route("/api/itsm/knowledge/{id}", get(get_article).put(update_article))
        .route("/api/itsm/knowledge/{id}/approve", put(approve_article))
}

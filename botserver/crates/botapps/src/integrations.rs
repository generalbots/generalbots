use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connector {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub connected: bool,
    pub last_sync: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EtlJob {
    pub id: Uuid,
    pub name: String,
    pub source_connector: String,
    pub destination: String,
    pub schedule: String,
    pub last_run_status: Option<String>,
    pub last_run_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateConnector {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
}

#[derive(Debug, Default)]
pub struct IntegrationsState {
    pub connectors: HashMap<String, Connector>,
    pub etl_jobs: Vec<EtlJob>,
}



pub fn create_integrations_state() -> SharedIntegrationsState {
    Arc::new(RwLock::new(IntegrationsState::default()))
}

async fn list_connectors(
    State(state): State<SharedIntegrationsState>,
) -> Result<Json<Vec<Connector>>, StatusCode> {
    let data = state.read().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(data.connectors.values().cloned().collect()))
}

async fn connect_connector(
    State(state): State<SharedIntegrationsState>,
    Path(id): Path<String>,
) -> Result<Json<Connector>, StatusCode> {
    let mut data = state.write().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let connector = data.connectors.get_mut(&*id).ok_or(StatusCode::NOT_FOUND)?;
    connector.connected = true;
    connector.last_sync = Some(Utc::now().to_rfc3339());
    Ok(Json(connector.clone()))
}

async fn create_connector(
    State(state): State<SharedIntegrationsState>,
    Json(input): Json<CreateConnector>,
) -> Result<(StatusCode, Json<Connector>), StatusCode> {
    let mut data = state.write().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let connector = Connector {
        id: input.id.clone(),
        name: input.name,
        description: input.description,
        category: input.category,
        connected: false,
        last_sync: None,
    };
    data.connectors.insert(input.id, connector.clone());
    Ok((StatusCode::CREATED, Json(connector)))
}

async fn delete_connector(
    State(state): State<SharedIntegrationsState>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let mut data = state.write().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    data.connectors.remove(&*id).ok_or(StatusCode::NOT_FOUND)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn list_etl(
    State(state): State<SharedIntegrationsState>,
) -> Result<Json<Vec<EtlJob>>, StatusCode> {
    let data = state.read().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(data.etl_jobs.clone()))
}

pub fn routes() -> Router {
    let state = std::sync::Arc::new(std::sync::RwLock::new(Default::default()));
    Router::new()
        .route(
            "/api/integrations/connectors",
            get(list_connectors).post(create_connector),
        )
        .route(
            "/api/integrations/connectors/{id}/connect",
            post(connect_connector),
        )
        .route(
            "/api/integrations/connectors/{id}",
            delete(delete_connector),
        )
        .route("/api/integrations/etl", get(list_etl))
        .with_state(state)
}

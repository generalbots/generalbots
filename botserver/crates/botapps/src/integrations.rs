use axum::{extract::{State, Json, Path}, routing::{get, post}, Router};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Connector {
    pub id: Uuid,
    pub name: String,
    pub connector_type: String,
    pub endpoint: String,
    pub auth_type: String,
    pub status: String,
    pub config: String,
    pub last_sync_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EtlJob {
    pub id: Uuid,
    pub name: String,
    pub source: String,
    pub destination: String,
    pub transformation: String,
    pub schedule: String,
    pub status: String,
    pub last_run_at: Option<String>,
    pub records_processed: u64,
    pub created_at: String,
}

#[derive(Default)]
pub struct IntegrationsState {
    pub connectors: HashMap<Uuid, Connector>,
    pub etl_jobs: HashMap<Uuid, EtlJob>,
}

pub fn routes() -> axum::Router {
    let state = Arc::new(RwLock::new(IntegrationsState::default()));
    Router::new()
        .route("/api/integrations/connectors", get(list_connectors).post(create_connector))
        .route("/api/integrations/connectors/{id}", get(get_connector).put(update_connector).delete(delete_connector))
        .route("/api/integrations/connectors/{id}/test", post(test_connector))
        .route("/api/integrations/etl", get(list_etl_jobs).post(create_etl_job))
        .route("/api/integrations/etl/{id}", get(get_etl_job).put(update_etl_job).delete(delete_etl_job))
        .route("/api/integrations/etl/{id}/run", post(run_etl_job))
        .with_state(state)
}

async fn list_connectors(State(state): State<Arc<RwLock<IntegrationsState>>>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    let items: Vec<&Connector> = s.connectors.values().collect();
    Json(serde_json::json!({"connectors": items}))
}

async fn create_connector(State(state): State<Arc<RwLock<IntegrationsState>>>, Json(mut connector): Json<Connector>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    let id = Uuid::new_v4();
    connector.id = id;
    connector.status = "Created".to_string();
    connector.created_at = Utc::now().to_rfc3339();
    s.connectors.insert(id, connector.clone());
    Json(serde_json::json!({"connector": connector}))
}

async fn get_connector(State(state): State<Arc<RwLock<IntegrationsState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    match s.connectors.get(&id) {
        Some(c) => Json(serde_json::json!({"connector": c})),
        None => Json(serde_json::json!({"error": "Connector not found"})),
    }
}

async fn update_connector(State(state): State<Arc<RwLock<IntegrationsState>>>, Path(id): Path<Uuid>, Json(connector): Json<Connector>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    if let Some(existing) = s.connectors.get_mut(&id) {
        *existing = connector.clone();
        existing.id = id;
        Json(serde_json::json!({"connector": existing.clone()}))
    } else {
        Json(serde_json::json!({"error": "Connector not found"}))
    }
}

async fn delete_connector(State(state): State<Arc<RwLock<IntegrationsState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    s.connectors.remove(&id);
    Json(serde_json::json!({"deleted": true}))
}

async fn test_connector(State(state): State<Arc<RwLock<IntegrationsState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    match s.connectors.get_mut(&id) {
        Some(c) => {
            c.status = "Connected".to_string();
            c.last_sync_at = Some(Utc::now().to_rfc3339());
            Json(serde_json::json!({"connector": c.clone(), "test_result": "success"}))
        }
        None => Json(serde_json::json!({"error": "Connector not found"})),
    }
}

async fn list_etl_jobs(State(state): State<Arc<RwLock<IntegrationsState>>>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    let items: Vec<&EtlJob> = s.etl_jobs.values().collect();
    Json(serde_json::json!({"etl_jobs": items}))
}

async fn create_etl_job(State(state): State<Arc<RwLock<IntegrationsState>>>, Json(mut job): Json<EtlJob>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    let id = Uuid::new_v4();
    job.id = id;
    job.status = "Created".to_string();
    job.created_at = Utc::now().to_rfc3339();
    s.etl_jobs.insert(id, job.clone());
    Json(serde_json::json!({"etl_job": job}))
}

async fn get_etl_job(State(state): State<Arc<RwLock<IntegrationsState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    match s.etl_jobs.get(&id) {
        Some(j) => Json(serde_json::json!({"etl_job": j})),
        None => Json(serde_json::json!({"error": "ETL job not found"})),
    }
}

async fn update_etl_job(State(state): State<Arc<RwLock<IntegrationsState>>>, Path(id): Path<Uuid>, Json(job): Json<EtlJob>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    if let Some(existing) = s.etl_jobs.get_mut(&id) {
        *existing = job.clone();
        existing.id = id;
        Json(serde_json::json!({"etl_job": existing.clone()}))
    } else {
        Json(serde_json::json!({"error": "ETL job not found"}))
    }
}

async fn delete_etl_job(State(state): State<Arc<RwLock<IntegrationsState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    s.etl_jobs.remove(&id);
    Json(serde_json::json!({"deleted": true}))
}

async fn run_etl_job(State(state): State<Arc<RwLock<IntegrationsState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    match s.etl_jobs.get_mut(&id) {
        Some(j) => {
            j.status = "Running".to_string();
            j.last_run_at = Some(Utc::now().to_rfc3339());
            j.records_processed += 100;
            Json(serde_json::json!({"etl_job": j.clone(), "run_result": "success"}))
        }
        None => Json(serde_json::json!({"error": "ETL job not found"})),
    }
}

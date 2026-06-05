use axum::extract::{Json, Path};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, OnceLock};
use axum::http::StatusCode;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Connector {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub endpoint: String,
    pub status: String,
    pub config: Option<HashMap<String, String>>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EtlJob {
    pub id: String,
    pub name: String,
    pub source: String,
    pub target: String,
    pub schedule: String,
    pub status: String,
    pub last_run: Option<String>,
    pub created_at: String,
}

#[derive(Default)]
struct AppState {
    connectors: HashMap<String, Connector>,
    etl_jobs: HashMap<String, EtlJob>,
}

fn state() -> &'static Arc<RwLock<AppState>> {
    static S: OnceLock<Arc<RwLock<AppState>>> = OnceLock::new();
    S.get_or_init(|| Arc::new(RwLock::new(AppState::default())))
}

pub async fn list_connectors() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let s = state().read().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {}", e)))?;
    let items: Vec<&Connector> = s.connectors.values().collect();
    Ok(Json(serde_json::json!({"items": items})))
}

pub async fn connect_connector(Path(id): Path<String>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut s = state().write().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {}", e)))?;
    match s.connectors.get_mut(&id) {
        Some(item) => {
            item.status = "connected".to_string();
            Ok(Json(serde_json::json!({"item": item})))
        }
        None => Err((StatusCode::NOT_FOUND, "Connector not found".to_string())),
    }
}

pub async fn disconnect_connector(Path(id): Path<String>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut s = state().write().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {}", e)))?;
    match s.connectors.get_mut(&id) {
        Some(item) => {
            item.status = "disconnected".to_string();
            Ok(Json(serde_json::json!({"item": item})))
        }
        None => Err((StatusCode::NOT_FOUND, "Connector not found".to_string())),
    }
}

pub async fn list_etl() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let s = state().read().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {}", e)))?;
    let items: Vec<&EtlJob> = s.etl_jobs.values().collect();
    Ok(Json(serde_json::json!({"items": items})))
}

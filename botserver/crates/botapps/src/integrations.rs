use axum::extract::{Json, Path};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, OnceLock};

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

pub async fn list_connectors() -> Json<serde_json::Value> {
    let s = state().read().unwrap();
    let items: Vec<&Connector> = s.connectors.values().collect();
    Json(serde_json::json!({"items": items}))
}

pub async fn connect_connector(Path(id): Path<String>) -> Json<serde_json::Value> {
    let mut s = state().write().unwrap();
    match s.connectors.get_mut(&id) {
        Some(item) => {
            item.status = "connected".to_string();
            Json(serde_json::json!({"item": item}))
        }
        None => Json(serde_json::json!({"error": "Connector not found"})),
    }
}

pub async fn disconnect_connector(Path(id): Path<String>) -> Json<serde_json::Value> {
    let mut s = state().write().unwrap();
    match s.connectors.get_mut(&id) {
        Some(item) => {
            item.status = "disconnected".to_string();
            Json(serde_json::json!({"item": item}))
        }
        None => Json(serde_json::json!({"error": "Connector not found"})),
    }
}

pub async fn list_etl() -> Json<serde_json::Value> {
    let s = state().read().unwrap();
    let items: Vec<&EtlJob> = s.etl_jobs.values().collect();
    Json(serde_json::json!({"items": items}))
}

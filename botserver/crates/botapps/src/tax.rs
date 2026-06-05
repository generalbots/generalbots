use axum::extract::{Json, Path};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, OnceLock};
use axum::http::StatusCode;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NFe {
    pub id: String,
    pub number: String,
    pub series: String,
    pub emitter_cnpj: String,
    pub recipient_cnpj: String,
    pub total: f64,
    pub status: String,
    pub created_at: String,
    pub authorized_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NFSe {
    pub id: String,
    pub number: String,
    pub service_code: String,
    pub provider_cnpj: String,
    pub total: f64,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CTe {
    pub id: String,
    pub number: String,
    pub sender_cnpj: String,
    pub recipient_cnpj: String,
    pub modality: String,
    pub total: f64,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Sped {
    pub id: String,
    pub period: String,
    pub kind: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Default)]
struct AppState {
    nfe: HashMap<String, NFe>,
    nfse: HashMap<String, NFSe>,
    cte: HashMap<String, CTe>,
    sped: HashMap<String, Sped>,
}

fn state() -> &'static Arc<RwLock<AppState>> {
    static S: OnceLock<Arc<RwLock<AppState>>> = OnceLock::new();
    S.get_or_init(|| Arc::new(RwLock::new(AppState::default())))
}

pub async fn list_nfe() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let s = state().read().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {}", e)))?;
    let items: Vec<&NFe> = s.nfe.values().collect();
    Ok(Json(serde_json::json!({"items": items})))
}

pub async fn create_nfe(Json(item): Json<NFe>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut s = state().write().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {}", e)))?;
    let id = uuid::Uuid::new_v4().to_string();
    let mut new_item = item;
    new_item.id = id.clone();
    new_item.created_at = chrono::Utc::now().to_rfc3339();
    new_item.status = "pending".to_string();
    s.nfe.insert(id.clone(), new_item.clone());
    Ok(Json(serde_json::json!({"item": new_item})))
}

pub async fn authorize_nfe(Path(id): Path<String>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut s = state().write().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {}", e)))?;
    match s.nfe.get_mut(&id) {
        Some(item) => {
            item.status = "authorized".to_string();
            item.authorized_at = Some(chrono::Utc::now().to_rfc3339());
            Ok(Json(serde_json::json!({"item": item})))
        }
        None => Err((StatusCode::NOT_FOUND, "NFe not found".to_string())),
    }
}

pub async fn list_nfse() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let s = state().read().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {}", e)))?;
    let items: Vec<&NFSe> = s.nfse.values().collect();
    Ok(Json(serde_json::json!({"items": items})))
}

pub async fn create_nfse(Json(item): Json<NFSe>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut s = state().write().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {}", e)))?;
    let id = uuid::Uuid::new_v4().to_string();
    let mut new_item = item;
    new_item.id = id.clone();
    new_item.created_at = chrono::Utc::now().to_rfc3339();
    new_item.status = "pending".to_string();
    s.nfse.insert(id.clone(), new_item.clone());
    Ok(Json(serde_json::json!({"item": new_item})))
}

pub async fn list_cte() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let s = state().read().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {}", e)))?;
    let items: Vec<&CTe> = s.cte.values().collect();
    Ok(Json(serde_json::json!({"items": items})))
}

pub async fn create_cte(Json(item): Json<CTe>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut s = state().write().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {}", e)))?;
    let id = uuid::Uuid::new_v4().to_string();
    let mut new_item = item;
    new_item.id = id.clone();
    new_item.created_at = chrono::Utc::now().to_rfc3339();
    new_item.status = "pending".to_string();
    s.cte.insert(id.clone(), new_item.clone());
    Ok(Json(serde_json::json!({"item": new_item})))
}

pub async fn list_sped() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let s = state().read().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {}", e)))?;
    let items: Vec<&Sped> = s.sped.values().collect();
    Ok(Json(serde_json::json!({"items": items})))
}

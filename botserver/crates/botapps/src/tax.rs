use axum::{extract::{State, Json, Path}, routing::get, Router};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NfeStatus { Pending, Approved, Rejected }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CteStatus { Draft, Sent, Authorized, Cancelled }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NfeDocument {
    pub id: Uuid,
    pub number: String,
    pub emitter_cnpj: String,
    pub value: f64,
    pub status: String,
    pub xml_content: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NfseDocument {
    pub id: Uuid,
    pub number: String,
    pub provider_cnpj: String,
    pub service_code: String,
    pub value: f64,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CteDocument {
    pub id: Uuid,
    pub number: String,
    pub modality: String,
    pub emitter_cnpj: String,
    pub recipient_cnpj: String,
    pub value: f64,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SpedFile {
    pub id: Uuid,
    pub file_type: String,
    pub period: String,
    pub status: String,
    pub line_count: u32,
    pub created_at: String,
}

#[derive(Default)]
pub struct TaxState {
    pub nfe_documents: HashMap<Uuid, NfeDocument>,
    pub nfse_documents: HashMap<Uuid, NfseDocument>,
    pub cte_documents: HashMap<Uuid, CteDocument>,
    pub sped_files: HashMap<Uuid, SpedFile>,
}

pub fn routes() -> axum::Router {
    let state = Arc::new(RwLock::new(TaxState::default()));
    Router::new()
        .route("/api/tax/nfe", get(list_nfe).post(create_nfe))
        .route("/api/tax/nfe/{id}", get(get_nfe).put(update_nfe).delete(delete_nfe))
        .route("/api/tax/nfse", get(list_nfse).post(create_nfse))
        .route("/api/tax/nfse/{id}", get(get_nfse).put(update_nfse).delete(delete_nfse))
        .route("/api/tax/cte", get(list_cte).post(create_cte))
        .route("/api/tax/cte/{id}", get(get_cte).put(update_cte).delete(delete_cte))
        .route("/api/tax/sped", get(list_sped).post(create_sped))
        .route("/api/tax/sped/{id}", get(get_sped).delete(delete_sped))
        .with_state(state)
}

async fn list_nfe(State(state): State<Arc<RwLock<TaxState>>>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    let docs: Vec<&NfeDocument> = s.nfe_documents.values().collect();
    Json(serde_json::json!({"nfe_documents": docs}))
}

async fn create_nfe(State(state): State<Arc<RwLock<TaxState>>>, Json(mut doc): Json<NfeDocument>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    let id = Uuid::new_v4();
    doc.id = id;
    doc.status = "Pending".to_string();
    doc.created_at = Utc::now().to_rfc3339();
    s.nfe_documents.insert(id, doc.clone());
    Json(serde_json::json!({"nfe_document": doc}))
}

async fn get_nfe(State(state): State<Arc<RwLock<TaxState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    match s.nfe_documents.get(&id) {
        Some(doc) => Json(serde_json::json!({"nfe_document": doc})),
        None => Json(serde_json::json!({"error": "NFe not found"})),
    }
}

async fn update_nfe(State(state): State<Arc<RwLock<TaxState>>>, Path(id): Path<Uuid>, Json(doc): Json<NfeDocument>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    if let Some(existing) = s.nfe_documents.get_mut(&id) {
        *existing = doc.clone();
        existing.id = id;
        Json(serde_json::json!({"nfe_document": existing.clone()}))
    } else {
        Json(serde_json::json!({"error": "NFe not found"}))
    }
}

async fn delete_nfe(State(state): State<Arc<RwLock<TaxState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    s.nfe_documents.remove(&id);
    Json(serde_json::json!({"deleted": true}))
}

async fn list_nfse(State(state): State<Arc<RwLock<TaxState>>>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    let docs: Vec<&NfseDocument> = s.nfse_documents.values().collect();
    Json(serde_json::json!({"nfse_documents": docs}))
}

async fn create_nfse(State(state): State<Arc<RwLock<TaxState>>>, Json(mut doc): Json<NfseDocument>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    let id = Uuid::new_v4();
    doc.id = id;
    doc.status = "Pending".to_string();
    doc.created_at = Utc::now().to_rfc3339();
    s.nfse_documents.insert(id, doc.clone());
    Json(serde_json::json!({"nfse_document": doc}))
}

async fn get_nfse(State(state): State<Arc<RwLock<TaxState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    match s.nfse_documents.get(&id) {
        Some(doc) => Json(serde_json::json!({"nfse_document": doc})),
        None => Json(serde_json::json!({"error": "NFSe not found"})),
    }
}

async fn update_nfse(State(state): State<Arc<RwLock<TaxState>>>, Path(id): Path<Uuid>, Json(doc): Json<NfseDocument>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    if let Some(existing) = s.nfse_documents.get_mut(&id) {
        *existing = doc.clone();
        existing.id = id;
        Json(serde_json::json!({"nfse_document": existing.clone()}))
    } else {
        Json(serde_json::json!({"error": "NFSe not found"}))
    }
}

async fn delete_nfse(State(state): State<Arc<RwLock<TaxState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    s.nfse_documents.remove(&id);
    Json(serde_json::json!({"deleted": true}))
}

async fn list_cte(State(state): State<Arc<RwLock<TaxState>>>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    let docs: Vec<&CteDocument> = s.cte_documents.values().collect();
    Json(serde_json::json!({"cte_documents": docs}))
}

async fn create_cte(State(state): State<Arc<RwLock<TaxState>>>, Json(mut doc): Json<CteDocument>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    let id = Uuid::new_v4();
    doc.id = id;
    doc.status = "Draft".to_string();
    doc.created_at = Utc::now().to_rfc3339();
    s.cte_documents.insert(id, doc.clone());
    Json(serde_json::json!({"cte_document": doc}))
}

async fn get_cte(State(state): State<Arc<RwLock<TaxState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    match s.cte_documents.get(&id) {
        Some(doc) => Json(serde_json::json!({"cte_document": doc})),
        None => Json(serde_json::json!({"error": "CTe not found"})),
    }
}

async fn update_cte(State(state): State<Arc<RwLock<TaxState>>>, Path(id): Path<Uuid>, Json(doc): Json<CteDocument>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    if let Some(existing) = s.cte_documents.get_mut(&id) {
        *existing = doc.clone();
        existing.id = id;
        Json(serde_json::json!({"cte_document": existing.clone()}))
    } else {
        Json(serde_json::json!({"error": "CTe not found"}))
    }
}

async fn delete_cte(State(state): State<Arc<RwLock<TaxState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    s.cte_documents.remove(&id);
    Json(serde_json::json!({"deleted": true}))
}

async fn list_sped(State(state): State<Arc<RwLock<TaxState>>>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    let docs: Vec<&SpedFile> = s.sped_files.values().collect();
    Json(serde_json::json!({"sped_files": docs}))
}

async fn create_sped(State(state): State<Arc<RwLock<TaxState>>>, Json(mut doc): Json<SpedFile>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    let id = Uuid::new_v4();
    doc.id = id;
    doc.status = "Generated".to_string();
    doc.created_at = Utc::now().to_rfc3339();
    s.sped_files.insert(id, doc.clone());
    Json(serde_json::json!({"sped_file": doc}))
}

async fn get_sped(State(state): State<Arc<RwLock<TaxState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    match s.sped_files.get(&id) {
        Some(doc) => Json(serde_json::json!({"sped_file": doc})),
        None => Json(serde_json::json!({"error": "SPED not found"})),
    }
}

async fn delete_sped(State(state): State<Arc<RwLock<TaxState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    s.sped_files.remove(&id);
    Json(serde_json::json!({"deleted": true}))
}

use axum::{extract::{State, Json, Path}, routing::get, Router};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Verification {
    pub id: Uuid,
    pub user_id: String,
    pub document_type: String,
    pub document_number: String,
    pub full_name: String,
    pub status: String,
    pub rejection_reason: Option<String>,
    pub verified_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Signature {
    pub id: Uuid,
    pub verification_id: Uuid,
    pub signer_name: String,
    pub signer_email: String,
    pub document_url: String,
    pub signed: bool,
    pub signed_at: Option<String>,
    pub ip_address: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Certificate {
    pub id: Uuid,
    pub verification_id: Uuid,
    pub cert_type: String,
    pub issuer: String,
    pub subject: String,
    pub valid_from: String,
    pub valid_until: String,
    pub serial_number: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Default)]
pub struct KycState {
    pub verifications: HashMap<Uuid, Verification>,
    pub signatures: HashMap<Uuid, Signature>,
    pub certificates: HashMap<Uuid, Certificate>,
}

pub fn routes() -> axum::Router {
    let state = Arc::new(RwLock::new(KycState::default()));
    Router::new()
        .route("/api/kyc/verifications", get(list_verifications).post(create_verification))
        .route("/api/kyc/verifications/{id}", get(get_verification).put(update_verification))
        .route("/api/kyc/signatures", get(list_signatures).post(create_signature))
        .route("/api/kyc/signatures/{id}", get(get_signature).put(update_signature))
        .route("/api/kyc/certificates", get(list_certificates).post(create_certificate))
        .route("/api/kyc/certificates/{id}", get(get_certificate).put(update_certificate).delete(delete_certificate))
        .with_state(state)
}

async fn list_verifications(State(state): State<Arc<RwLock<KycState>>>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    let items: Vec<&Verification> = s.verifications.values().collect();
    Json(serde_json::json!({"verifications": items}))
}

async fn create_verification(State(state): State<Arc<RwLock<KycState>>>, Json(mut v): Json<Verification>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    let id = Uuid::new_v4();
    v.id = id;
    v.status = "Pending".to_string();
    v.created_at = Utc::now().to_rfc3339();
    s.verifications.insert(id, v.clone());
    Json(serde_json::json!({"verification": v}))
}

async fn get_verification(State(state): State<Arc<RwLock<KycState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    match s.verifications.get(&id) {
        Some(v) => Json(serde_json::json!({"verification": v})),
        None => Json(serde_json::json!({"error": "Verification not found"})),
    }
}

async fn update_verification(State(state): State<Arc<RwLock<KycState>>>, Path(id): Path<Uuid>, Json(v): Json<Verification>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    if let Some(existing) = s.verifications.get_mut(&id) {
        *existing = v.clone();
        existing.id = id;
        Json(serde_json::json!({"verification": existing.clone()}))
    } else {
        Json(serde_json::json!({"error": "Verification not found"}))
    }
}

async fn list_signatures(State(state): State<Arc<RwLock<KycState>>>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    let items: Vec<&Signature> = s.signatures.values().collect();
    Json(serde_json::json!({"signatures": items}))
}

async fn create_signature(State(state): State<Arc<RwLock<KycState>>>, Json(mut sig): Json<Signature>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    let id = Uuid::new_v4();
    sig.id = id;
    sig.signed = false;
    sig.created_at = Utc::now().to_rfc3339();
    s.signatures.insert(id, sig.clone());
    Json(serde_json::json!({"signature": sig}))
}

async fn get_signature(State(state): State<Arc<RwLock<KycState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    match s.signatures.get(&id) {
        Some(sig) => Json(serde_json::json!({"signature": sig})),
        None => Json(serde_json::json!({"error": "Signature not found"})),
    }
}

async fn update_signature(State(state): State<Arc<RwLock<KycState>>>, Path(id): Path<Uuid>, Json(sig): Json<Signature>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    if let Some(existing) = s.signatures.get_mut(&id) {
        *existing = sig.clone();
        existing.id = id;
        Json(serde_json::json!({"signature": existing.clone()}))
    } else {
        Json(serde_json::json!({"error": "Signature not found"}))
    }
}

async fn list_certificates(State(state): State<Arc<RwLock<KycState>>>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    let items: Vec<&Certificate> = s.certificates.values().collect();
    Json(serde_json::json!({"certificates": items}))
}

async fn create_certificate(State(state): State<Arc<RwLock<KycState>>>, Json(mut cert): Json<Certificate>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    let id = Uuid::new_v4();
    cert.id = id;
    cert.status = "Active".to_string();
    cert.created_at = Utc::now().to_rfc3339();
    s.certificates.insert(id, cert.clone());
    Json(serde_json::json!({"certificate": cert}))
}

async fn get_certificate(State(state): State<Arc<RwLock<KycState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let s = state.read().unwrap();
    match s.certificates.get(&id) {
        Some(c) => Json(serde_json::json!({"certificate": c})),
        None => Json(serde_json::json!({"error": "Certificate not found"})),
    }
}

async fn update_certificate(State(state): State<Arc<RwLock<KycState>>>, Path(id): Path<Uuid>, Json(cert): Json<Certificate>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    if let Some(existing) = s.certificates.get_mut(&id) {
        *existing = cert.clone();
        existing.id = id;
        Json(serde_json::json!({"certificate": existing.clone()}))
    } else {
        Json(serde_json::json!({"error": "Certificate not found"}))
    }
}

async fn delete_certificate(State(state): State<Arc<RwLock<KycState>>>, Path(id): Path<Uuid>) -> Json<serde_json::Value> {
    let mut s = state.write().unwrap();
    s.certificates.remove(&id);
    Json(serde_json::json!({"deleted": true}))
}

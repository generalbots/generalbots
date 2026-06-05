use axum::extract::{Json, Path};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, OnceLock};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Verification {
    pub id: String,
    pub user_id: String,
    pub kind: String,
    pub status: String,
    pub documents: Vec<String>,
    pub reviewed_by: Option<String>,
    pub created_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Signature {
    pub id: String,
    pub document_id: String,
    pub signer_name: String,
    pub signer_email: String,
    pub status: String,
    pub signed_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Certificate {
    pub id: String,
    pub subject: String,
    pub issuer: String,
    pub serial: String,
    pub valid_from: String,
    pub valid_until: String,
    pub status: String,
}

#[derive(Default)]
struct AppState {
    verifications: HashMap<String, Verification>,
    signatures: HashMap<String, Signature>,
    certificates: HashMap<String, Certificate>,
}

fn state() -> &'static Arc<RwLock<AppState>> {
    static S: OnceLock<Arc<RwLock<AppState>>> = OnceLock::new();
    S.get_or_init(|| Arc::new(RwLock::new(AppState::default())))
}

pub async fn list_verifications() -> Json<serde_json::Value> {
    let s = state().read().unwrap();
    let items: Vec<&Verification> = s.verifications.values().collect();
    Json(serde_json::json!({"items": items}))
}

pub async fn update_verification(Path(id): Path<String>, Json(item): Json<Verification>) -> Json<serde_json::Value> {
    let mut s = state().write().unwrap();
    if let Some(existing) = s.verifications.get_mut(&id) {
        existing.status = item.status;
        existing.reviewed_by = item.reviewed_by;
        existing.documents = item.documents;
        Json(serde_json::json!({"item": existing}))
    } else {
        Json(serde_json::json!({"error": "Verification not found"}))
    }
}

pub async fn list_signatures() -> Json<serde_json::Value> {
    let s = state().read().unwrap();
    let items: Vec<&Signature> = s.signatures.values().collect();
    Json(serde_json::json!({"items": items}))
}

pub async fn sign_document(Path(id): Path<String>) -> Json<serde_json::Value> {
    let mut s = state().write().unwrap();
    match s.signatures.get_mut(&id) {
        Some(sig) => {
            sig.status = "signed".to_string();
            sig.signed_at = Some(chrono::Utc::now().to_rfc3339());
            Json(serde_json::json!({"item": sig}))
        }
        None => Json(serde_json::json!({"error": "Signature not found"})),
    }
}

pub async fn list_certificates() -> Json<serde_json::Value> {
    let s = state().read().unwrap();
    let items: Vec<&Certificate> = s.certificates.values().collect();
    Json(serde_json::json!({"items": items}))
}

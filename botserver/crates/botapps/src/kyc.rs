use axum::extract::{Json, Path};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, OnceLock};
use axum::http::StatusCode;

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

pub async fn list_verifications() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let s = state().read().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {}", e)))?;
    let items: Vec<&Verification> = s.verifications.values().collect();
    Ok(Json(serde_json::json!({"items": items})))
}

pub async fn update_verification(Path(id): Path<String>, Json(item): Json<Verification>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut s = state().write().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {}", e)))?;
    if let Some(existing) = s.verifications.get_mut(&id) {
        existing.status = item.status;
        existing.reviewed_by = item.reviewed_by;
        existing.documents = item.documents;
        Ok(Json(serde_json::json!({"item": existing})))
    } else {
        Err((StatusCode::NOT_FOUND, "Verification not found".to_string()))
    }
}

pub async fn list_signatures() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let s = state().read().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {}", e)))?;
    let items: Vec<&Signature> = s.signatures.values().collect();
    Ok(Json(serde_json::json!({"items": items})))
}

pub async fn sign_document(Path(id): Path<String>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut s = state().write().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {}", e)))?;
    match s.signatures.get_mut(&id) {
        Some(sig) => {
            sig.status = "signed".to_string();
            sig.signed_at = Some(chrono::Utc::now().to_rfc3339());
            Ok(Json(serde_json::json!({"item": sig})))
        }
        None => Err((StatusCode::NOT_FOUND, "Signature not found".to_string())),
    }
}

pub async fn list_certificates() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let s = state().read().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("RwLock poisoned: {}", e)))?;
    let items: Vec<&Certificate> = s.certificates.values().collect();
    Ok(Json(serde_json::json!({"items": items})))
}

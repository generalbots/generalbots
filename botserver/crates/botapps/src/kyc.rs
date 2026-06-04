use axum::{
    extract::{Path, State as AxumState},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, put},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DocumentType {
    Cpf,
    Rg,
    Passport,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum KycStatus {
    Pending,
    Verified,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SignatureStatus {
    Awaiting,
    Signed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KycVerification {
    pub id: Uuid,
    pub user_name: String,
    pub document_type: DocumentType,
    pub status: KycStatus,
    pub liveness_score: Option<f64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigitalSignature {
    pub id: Uuid,
    pub document_name: String,
    pub signer_email: String,
    pub status: SignatureStatus,
    pub signed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Certificate {
    pub id: Uuid,
    pub subject: String,
    pub issuer: String,
    pub not_before: DateTime<Utc>,
    pub not_after: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateKycRequest {
    pub user_name: String,
    pub document_type: DocumentType,
}

#[derive(Debug, Deserialize)]
pub struct UpdateKycRequest {
    pub status: Option<KycStatus>,
    pub liveness_score: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSignatureRequest {
    pub document_name: String,
    pub signer_email: String,
}

#[derive(Debug, Deserialize)]
pub struct SignDocumentRequest {
    pub signed: bool,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

#[derive(Clone)]
pub struct KycState {
    pub verifications: Arc<RwLock<Vec<KycVerification>>>,
    pub signatures: Arc<RwLock<Vec<DigitalSignature>>>,
    pub certificates: Arc<RwLock<Vec<Certificate>>>,
}

impl KycState {
    pub fn new() -> Self {
        Self {
            verifications: Arc::new(RwLock::new(Vec::new())),
            signatures: Arc::new(RwLock::new(Vec::new())),
            certificates: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

pub fn routes() -> Router {
    let state = KycState::new();
    Router::new()
        .route("/api/kyc/verifications", get(list_verifications).post(create_verification))
        .route("/api/kyc/verifications/:id", put(update_verification))
        .route("/api/kyc/signatures", get(list_signatures).post(create_signature))
        .route("/api/kyc/signatures/:id/sign", put(sign_document))
        .route("/api/kyc/certificates", get(list_certificates))
        .with_state(state)
}

async fn list_verifications(
    AxumState(state): AxumState<KycState>,
) -> Result<Json<ApiResponse<Vec<KycVerification>>>, StatusCode> {
    let verifications = state
        .verifications
        .read()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(ApiResponse {
        success: true,
        data: Some(verifications.clone()),
        error: None,
    }))
}

async fn create_verification(
    AxumState(state): AxumState<KycState>,
    Json(payload): Json<CreateKycRequest>,
) -> Result<(StatusCode, Json<ApiResponse<KycVerification>>), StatusCode> {
    let verification = KycVerification {
        id: Uuid::new_v4(),
        user_name: payload.user_name,
        document_type: payload.document_type,
        status: KycStatus::Pending,
        liveness_score: None,
        created_at: Utc::now(),
    };
    let mut verifications = state
        .verifications
        .write()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    verifications.push(verification.clone());
    Ok((StatusCode::CREATED, Json(ApiResponse {
        success: true,
        data: Some(verification),
        error: None,
    })))
}

async fn update_verification(
    AxumState(state): AxumState<KycState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateKycRequest>,
) -> Result<Json<ApiResponse<KycVerification>>, StatusCode> {
    let mut verifications = state
        .verifications
        .write()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let verification = verifications
        .iter_mut()
        .find(|v| v.id == id)
        .ok_or(StatusCode::NOT_FOUND)?;
    if let Some(status) = payload.status {
        verification.status = status;
    }
    if let Some(score) = payload.liveness_score {
        verification.liveness_score = Some(score);
    }
    Ok(Json(ApiResponse {
        success: true,
        data: Some(verification.clone()),
        error: None,
    }))
}

async fn list_signatures(
    AxumState(state): AxumState<KycState>,
) -> Result<Json<ApiResponse<Vec<DigitalSignature>>>, StatusCode> {
    let signatures = state
        .signatures
        .read()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(ApiResponse {
        success: true,
        data: Some(signatures.clone()),
        error: None,
    }))
}

async fn create_signature(
    AxumState(state): AxumState<KycState>,
    Json(payload): Json<CreateSignatureRequest>,
) -> Result<(StatusCode, Json<ApiResponse<DigitalSignature>>), StatusCode> {
    let signature = DigitalSignature {
        id: Uuid::new_v4(),
        document_name: payload.document_name,
        signer_email: payload.signer_email,
        status: SignatureStatus::Awaiting,
        signed_at: None,
    };
    let mut signatures = state
        .signatures
        .write()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    signatures.push(signature.clone());
    Ok((StatusCode::CREATED, Json(ApiResponse {
        success: true,
        data: Some(signature),
        error: None,
    })))
}

async fn sign_document(
    AxumState(state): AxumState<KycState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<SignDocumentRequest>,
) -> Result<Json<ApiResponse<DigitalSignature>>, StatusCode> {
    let mut signatures = state
        .signatures
        .write()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let signature = signatures
        .iter_mut()
        .find(|s| s.id == id)
        .ok_or(StatusCode::NOT_FOUND)?;
    if payload.signed {
        signature.status = SignatureStatus::Signed;
        signature.signed_at = Some(Utc::now());
    }
    Ok(Json(ApiResponse {
        success: true,
        data: Some(signature.clone()),
        error: None,
    }))
}

async fn list_certificates(
    AxumState(state): AxumState<KycState>,
) -> Result<Json<ApiResponse<Vec<Certificate>>>, StatusCode> {
    let certificates = state
        .certificates
        .read()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(ApiResponse {
        success: true,
        data: Some(certificates.clone()),
        error: None,
    }))
}

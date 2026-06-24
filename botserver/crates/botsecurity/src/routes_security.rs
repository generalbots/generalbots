use axum::{
    extract::Path,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::biometric::{BiometricMatcher, BiometricMatch, BiometricTemplate};
use super::digital_signature::{SignatureRequest, SignatureService, SignedDocument, ValidationStatus};
use super::kyc::{CheckStatus, CheckType, ComplianceCheck, KycProfile, KycService, KycStatus};

pub fn configure_security_api_routes<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/api/security/kyc/profiles", post(create_kyc_profile))
        .route("/api/security/kyc/profiles/{id}", get(get_kyc_profile))
        .route("/api/security/kyc/profiles/{id}/checks", post(run_kyc_check))
        .route("/api/security/biometric/verify", post(verify_biometric))
        .route("/api/security/signatures/requests", post(create_signature_request))
        .route("/api/security/signatures/requests/{id}/complete", post(complete_signature))
        .route("/api/security/signatures/{id}/validate", get(validate_signature))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateKycProfileRequest {
    pub tenant_id: Uuid,
    pub user_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RunCheckRequest {
    pub check_type: CheckType,
    pub provider: String,
    pub score: Option<f64>,
    pub status: CheckStatus,
    pub result: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyBiometricRequest {
    pub probe_embedding: Vec<f32>,
    pub probe_liveness: Option<f64>,
    pub candidate: BiometricTemplate,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSignatureRequestBody {
    pub tenant_id: Uuid,
    pub document_id: Uuid,
    pub signer_id: Uuid,
    pub document_content_b64: String,
    pub certificate_id: Option<Uuid>,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompleteSignatureBody {
    pub signature_value: String,
    pub certificate_chain: Vec<String>,
}

async fn create_kyc_profile(
    Json(req): Json<CreateKycProfileRequest>,
) -> Result<Json<KycProfile>, (StatusCode, String)> {
    let profile = KycService::start_profile(req.tenant_id, req.user_id);
    Ok(Json(profile))
}

async fn get_kyc_profile(
    Path(id): Path<Uuid>,
) -> Result<Json<KycProfile>, (StatusCode, String)> {
    Ok(Json(KycService::start_profile(Uuid::nil(), id)))
}

async fn run_kyc_check(
    Path(profile_id): Path<Uuid>,
    Json(req): Json<RunCheckRequest>,
) -> Result<Json<KycProfile>, (StatusCode, String)> {
    let mut profile = KycService::start_profile(Uuid::nil(), profile_id);
    let check = ComplianceCheck {
        id: Uuid::new_v4(),
        profile_id,
        check_type: req.check_type,
        provider: req.provider,
        status: req.status,
        score: req.score,
        result: req.result,
        checked_at: chrono::Utc::now(),
    };
    KycService::add_check(&mut profile, check);
    Ok(Json(profile))
}

async fn verify_biometric(
    Json(req): Json<VerifyBiometricRequest>,
) -> Result<Json<BiometricMatch>, (StatusCode, String)> {
    let matcher = BiometricMatcher::with_defaults();
    let result = matcher.match_template(&req.probe_embedding, req.probe_liveness, &req.candidate);
    Ok(Json(result))
}

async fn create_signature_request(
    Json(req): Json<CreateSignatureRequestBody>,
) -> Result<Json<SignatureRequest>, (StatusCode, String)> {
    use base64::Engine;
    let content = base64::engine::general_purpose::STANDARD
        .decode(&req.document_content_b64)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid base64: {e}")))?;
    let svc = SignatureService::new();
    let request = svc.create_request(
        req.tenant_id,
        req.document_id,
        req.signer_id,
        &content,
        req.certificate_id,
        req.reason,
    );
    Ok(Json(request))
}

async fn complete_signature(
    Path(id): Path<Uuid>,
    Json(req): Json<CompleteSignatureBody>,
) -> Result<Json<SignedDocument>, (StatusCode, String)> {
    let svc = SignatureService::new();
    let pending = svc.create_request(
        Uuid::nil(),
        Uuid::nil(),
        Uuid::nil(),
        b"",
        None,
        None,
    );
    let mut request = pending;
    request.id = id;
    let signed = svc.complete_signature(request, req.signature_value, req.certificate_chain);
    Ok(Json(signed))
}

async fn validate_signature(
    Path(id): Path<Uuid>,
) -> Result<Json<ValidationResponse>, (StatusCode, String)> {
    let pending = SignedDocument {
        id,
        signature_request_id: Uuid::nil(),
        document_id: Uuid::nil(),
        signer_id: Uuid::nil(),
        document_hash_sha256: String::new(),
        document_hash_sha512: String::new(),
        signed_at: chrono::Utc::now(),
        algorithm: super::digital_signature::SignatureAlgorithm::Ed25519,
        signature_value: "placeholder".into(),
        certificate_chain: vec!["cert1".into()],
        timestamp_token: None,
        ocsp_response: None,
        validation_status: ValidationStatus::Valid,
    };
    let status = SignatureService::validate(&pending);
    Ok(Json(ValidationResponse { id, status }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationResponse {
    pub id: Uuid,
    pub status: ValidationStatus,
}

impl From<KycStatus> for String {
    fn from(s: KycStatus) -> Self {
        format!("{:?}", s)
    }
}

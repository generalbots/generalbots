// Passkey HTTP handlers extracted from passkey.rs
use crate::core::shared::state::AppState;
use crate::security::passkey_types::*;
use crate::security::passkey_service::PasskeyService;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

/// Start WebAuthn registration for passkey
pub async fn start_registration(
    State(state): State<Arc<AppState>>,
    Json(request): Json<StartRegistrationRequest>,
) -> Result<Json<RegistrationOptions>, PasskeyError> {
    let user_id = request.user_id;
    let service = PasskeyService::new(Arc::clone(&state.conn));

    let options = service.generate_registration_options(&state, &request).await?;

    Ok(Json(options))
}

/// Verify passkey registration authentication
pub async fn verify_registration(
    State(state): State<Arc<AppState>>,
    Json(request): Json<VerifyAuthRequest>,
) -> Result<Json<AuthenticationResponse>, PasskeyError> {
    let user_id = request.user_id;
    let service = PasskeyService::new(Arc::clone(&state.conn));

    let verified = service.verify_registration(&request).await?;

    Ok(Json(AuthenticationResponse {
        status: "verified".to_string(),
        user_id: user_id.to_string(),
        display_name: request.display_name.unwrap_or_default(),
        new_credential_id: verified.new_credential_id,
    }))
}

/// Get all passkey credentials for user
pub async fn get_credentials(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<Vec<CredentialInfo>>, PasskeyError> {
    let service = PasskeyService::new(Arc::clone(&state.conn));
    let credentials = service.get_user_credentials(user_id).await?;

    Ok(Json(credentials))
}

/// Sign in with passkey
pub async fn sign_in(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SignInRequest>,
) -> Result<Json<AuthenticationResponse>, PasskeyError> {
    let service = PasskeyService::new(Arc::clone(&state.conn));
    let response = service.sign_in(&request).await?;

    Ok(Json(response))
}

/// Get fallback configuration
pub async fn get_fallback_config(
    State(state): State<Arc<AppState>>,
) -> Result<Json<FallbackConfig>, PasskeyError> {
    let service = PasskeyService::new(Arc::clone(&state.conn));
    let config = service.get_fallback_config().await?;

    Ok(Json(config))
}

/// Update fallback configuration
pub async fn set_fallback_config(
    State(state): State<Arc<AppState>>,
    Json(config): Json<FallbackConfig>,
) -> Result<Json<serde_json::Value>, PasskeyError> {
    let service = PasskeyService::new(Arc::clone(&state.conn));
    service.set_fallback_config(&config).await?;

    Ok(Json(serde_json::json!({"success": true})))
}

/// Clear fallback attempts
pub async fn clear_fallback(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ClearFallbackRequest>,
) -> Result<Json<serde_json::Value>, PasskeyError> {
    let service = PasskeyService::new(Arc::clone(&state.conn));
    service.clear_fallback_attempts(&request.username).await?;

    Ok(Json(serde_json::json!({"success": true})))
}

/// Get passkey challenges
pub async fn get_challenges(
    State(state): State<Arc<AppState>>,
    Json(request): Json<GetChallengesRequest>,
) -> Result<Json<Vec<ChallengeResponse>>, PasskeyError> {
    let service = PasskeyService::new(Arc::clone(&state.conn));
    let challenges = service.get_challenges(&request).await?;

    Ok(Json(challenges))
}

/// Answer passkey challenge
pub async fn answer_challenge(
    State(state): State<Arc<AppState>>,
    Path((user_id, challenge_id)): Path<(Uuid, String)>,
    Json(request): Json<AnswerChallengeRequest>,
) -> Result<Json<ChallengeResponse>, PasskeyError> {
    let service = PasskeyService::new(Arc::clone(&state.conn));
    let response = service.answer_challenge(&user_id, &challenge_id, &request).await?;

    Ok(Json(response))
}

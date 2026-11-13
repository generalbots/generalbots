//! The three SSO/login handlers and their payload types.

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, Response},
    Json,
};
use botcore::shared::state::AppState;
use botcoredirectory::auth_routes::{session_from_persisted, SESSION_CACHE};
use botsecurity_core::verified_jwt::{verify_hs256_jwt, JwtVerifyError, VerifiedJwt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::jwt::{base64_url_decode, base64_url_encode, jwt_sign_inner};
use super::pages::{redirect_to_login, sanitize_redirect, session_cookie, sso_html};
use super::session::{create_suite_session, resolve_session_roles};

#[derive(Debug, Deserialize)]
pub struct SsoRequest {
    pub cloud_token: String,
}

#[derive(Debug, Serialize)]
pub struct SsoResponse {
    pub access_token: String,
    pub email: String,
    pub user_id: String,
}

fn jwt_error_key_status(error: &JwtVerifyError) -> (&'static str, StatusCode) {
    let key = match error {
        JwtVerifyError::Malformed
        | JwtVerifyError::InvalidHeader
        | JwtVerifyError::UnsupportedAlgorithm
        | JwtVerifyError::InvalidSignature => "error-token-signature-invalid",
        JwtVerifyError::InvalidPayload => "error-token-payload-invalid",
        JwtVerifyError::MissingSubject
        | JwtVerifyError::MissingExpiry
        | JwtVerifyError::Expired
        | JwtVerifyError::NotYetValid => "error-token-invalid-format",
        JwtVerifyError::MissingSecret => "error-token-signing-failed",
    };
    let status = match error {
        JwtVerifyError::MissingSecret => StatusCode::INTERNAL_SERVER_ERROR,
        JwtVerifyError::InvalidPayload => StatusCode::BAD_REQUEST,
        _ => StatusCode::UNAUTHORIZED,
    };
    (key, status)
}

fn required_email(claims: &VerifiedJwt) -> Option<String> {
    claims
        .email
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn verified_org_id(claims: &VerifiedJwt) -> Option<String> {
    claims.org_id.map(|id| id.to_string())
}

/// POST /api/auth/cloud-sso
///
/// Exchanges a Cloud JWT for a Suite session token.
/// The JWT signature, algorithm, identity claims and expiry are fully
/// verified (#2364, #2365) — invalid, expired or incomplete tokens are
/// rejected and never mapped onto a shared identity.
pub async fn handle_cloud_sso(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(req): Json<SsoRequest>,
) -> Result<Json<SsoResponse>, (StatusCode, Json<serde_json::Value>)> {
    let locale = botcore::i18n::locale_from_headers(&headers);
    let jwt_secret = botsecurity_core::verified_jwt::resolve_saas_jwt_secret().map_err(|error| {
        log::error!("cloud-sso: {error}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(botcore::i18n::error_body(
                &locale,
                "error-token-signing-failed",
            )),
        )
    })?;

    let verified = verify_hs256_jwt(&req.cloud_token, &jwt_secret).map_err(|error| {
        log::warn!("cloud-sso: JWT rejected: {error}");
        let (key, status) = jwt_error_key_status(&error);
        (status, Json(botcore::i18n::error_body(&locale, key)))
    })?;

    let email = required_email(&verified).ok_or_else(|| {
        log::warn!("cloud-sso: JWT rejected: missing email claim");
        (
            StatusCode::UNAUTHORIZED,
            Json(botcore::i18n::error_body(&locale, "error-token-invalid-format")),
        )
    })?;
    let user_id = verified.sub.clone();

    let roles = resolve_session_roles(&state, &user_id);
    let api_token =
        create_suite_session(&email, &user_id, None, roles, verified_org_id(&verified)).await;

    Ok(Json(SsoResponse {
        access_token: api_token,
        email,
        user_id,
    }))
}

#[derive(Debug, Deserialize)]
pub struct SuiteSsoQuery {
    pub token: String,
    pub redirect: String,
}

/// GET /api/auth/suite-sso
///
/// Server-side SSO hop: validates a Cloud JWT (signature, exp, identity
/// claims) or a previously minted opaque `gb_*` Suite token, stores the
/// suite token in localStorage via HTML+script, then redirects to a clean
/// URL. Invalid, expired or incomplete credentials are rejected (#2364, #2365).
pub async fn handle_suite_sso(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SuiteSsoQuery>,
) -> Result<Response, StatusCode> {
    let jwt_secret = match botsecurity_core::verified_jwt::resolve_saas_jwt_secret() {
        Ok(secret) => secret,
        Err(error) => {
            log::error!("suite-sso: {error}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let parts: Vec<&str> = query.token.split('.').collect();

    // #1660 — `/api/auth/login` issues opaque `gb_<uuid>_<ts>` session tokens,
    // not JWTs. Validate those against the directory session store and hand
    // the same token to the suite; never answer the main frame with a 400.
    if parts.len() != 3 {
        if !query.token.starts_with("gb_") {
            return Ok(redirect_to_login("invalid_token"));
        }
        let valid = {
            let cache = SESSION_CACHE.read().await;
            match cache.get(&query.token) {
                Some(session) => chrono::Utc::now().timestamp() - session.created_at < 3600,
                // Cache miss after a restart: fall back to the persisted
                // login_sessions row the auth middleware would use.
                None => session_from_persisted(&query.token)
                    .map(|session| chrono::Utc::now().timestamp() - session.created_at < 3600)
                    .unwrap_or(false),
            }
        };
        if !valid {
            return Ok(redirect_to_login("session_expired"));
        }
        return Ok(session_cookie(
            query.token.clone(),
            Html(sso_html(&query.token, &sanitize_redirect(&query.redirect))),
        ));
    }

    let verified = match verify_hs256_jwt(&query.token, &jwt_secret) {
        Ok(claims) => claims,
        Err(error) => {
            log::warn!("suite-sso: JWT rejected: {error}");
            return Ok(redirect_to_login("invalid_token"));
        }
    };

    let Some(email) = required_email(&verified) else {
        log::warn!("suite-sso: JWT rejected: missing email claim");
        return Ok(redirect_to_login("invalid_token"));
    };
    let user_id = verified.sub.clone();
    let bucket = decode_jwt_claim_string(&query.token, "bucket");

    let roles = resolve_session_roles(&state, &user_id);
    let suite_token =
        create_suite_session(&email, &user_id, bucket, roles, verified_org_id(&verified)).await;
    let redirect = sanitize_redirect(&query.redirect);
    let html = Html(sso_html(&suite_token, &redirect));
    Ok(session_cookie(suite_token, html))
}

fn decode_jwt_claim_string(token: &str, claim: &str) -> Option<String> {
    let payload_b64 = token.split('.').nth(1)?;
    let payload_bytes = base64_url_decode(payload_b64).ok()?;
    let claims: serde_json::Value = serde_json::from_slice(&payload_bytes).ok()?;
    claims
        .get(claim)
        .and_then(|v| v.as_str())
        .map(str::to_string)
}

/// POST /api/auth/unified-login
///
/// Unified login returning both Suite (session cache) and Cloud (JWT) tokens.
/// Accepts `{ cloud_token }` and exchanges it only after full JWT
/// verification (signature, exp, identity claims) (#2364, #2365).
pub async fn handle_unified_login(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let locale = botcore::i18n::locale_from_headers(&headers);
    let jwt_secret = botsecurity_core::verified_jwt::resolve_saas_jwt_secret().map_err(|error| {
        log::error!("cloud-sso: {error}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(botcore::i18n::error_body(
                &locale,
                "error-token-signing-failed",
            )),
        )
    })?;

    let Some(cloud_token_in) = body.get("cloud_token").and_then(|v| v.as_str()) else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(botcore::i18n::error_body(&locale, "error-cloud-token-required")),
        ));
    };

    let verified = verify_hs256_jwt(cloud_token_in, &jwt_secret).map_err(|error| {
        log::warn!("unified-login: JWT rejected: {error}");
        let (key, status) = jwt_error_key_status(&error);
        (status, Json(botcore::i18n::error_body(&locale, key)))
    })?;
    let email = required_email(&verified).ok_or_else(|| {
        log::warn!("unified-login: JWT rejected: missing email claim");
        (
            StatusCode::UNAUTHORIZED,
            Json(botcore::i18n::error_body(&locale, "error-token-invalid-format")),
        )
    })?;
    let user_id = verified.sub.clone();
    let org_scope = verified_org_id(&verified);

    let roles = resolve_session_roles(&state, &user_id);
    let suite_token =
        create_suite_session(&email, &user_id, None, roles, org_scope).await;

    // Sign Cloud JWT
    let header = serde_json::json!({"alg": "HS256", "typ": "JWT"});
    let payload = serde_json::json!({
        "sub": user_id,
        "email": email,
        "exp": chrono::Utc::now().timestamp() + 3600,
    });
    let header_b64 = base64_url_encode(&serde_json::to_vec(&header).unwrap_or_default());
    let payload_b64 = base64_url_encode(&serde_json::to_vec(&payload).unwrap_or_default());
    let sig = jwt_sign_inner(&format!("{}.{}", header_b64, payload_b64), jwt_secret.as_bytes())
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": botsecurity::error_sanitizer::log_and_sanitize_json(
                        &e,
                        "main_module::routes::cloud_sso_handler",
                        None
                    )
                })),
            )
        })?;
    let cloud_token = format!("{}.{}.{}", header_b64, payload_b64, sig);

    Ok(Json(serde_json::json!({
        "access_token": suite_token,
        "cloud_token": cloud_token,
        "email": email,
        "user_id": user_id,
        "success": true,
    })))
}

//! The three SSO/login handlers and their payload types.

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, Response},
    Json,
};
use botcore::shared::state::AppState;
use botcoredirectory::auth_routes::{session_from_persisted, SESSION_CACHE};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::jwt::{
    base64_url_decode, base64_url_encode, claims_org, get_saas_jwt_secret, jwt_sign_inner,
};
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

/// POST /api/auth/cloud-sso
///
/// Troca um JWT do Cloud por um token de sessão do Suite.
/// Lê o segredo JWT de directory_config.json ou env var.
/// A assinatura é SEMPRE verificada — tokens com assinatura inválida são
/// rejeitados (fix #842: nunca aceitar payloads não assinados).
pub async fn handle_cloud_sso(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(req): Json<SsoRequest>,
) -> Result<Json<SsoResponse>, (StatusCode, Json<serde_json::Value>)> {
    let locale = botcore::i18n::locale_from_headers(&headers);
    let jwt_secret = get_saas_jwt_secret();

    // Validate JWT signature — reject on mismatch, no fallback
    let parts: Vec<&str> = req.cloud_token.split('.').collect();
    if parts.len() != 3 {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(botcore::i18n::error_body(&locale, "error-token-invalid-format")),
        ));
    }

    let (header_b64, payload_b64, sig_b64) = (parts[0], parts[1], parts[2]);
    let message = format!("{}.{}", header_b64, payload_b64);

    let expected_sig = match jwt_sign_inner(&message, jwt_secret.as_bytes()) {
        Ok(sig) => sig,
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(botcore::i18n::error_body(&locale, "error-token-signing-failed")),
            ))
        }
    };
    if !botsecurity_core::constant_time::eq_str(&sig_b64, &expected_sig) {
        log::warn!("cloud-sso: JWT signature mismatch — rejecting token");
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(botcore::i18n::error_body(
                &locale,
                "error-token-signature-invalid",
            )),
        ));
    }

    // Decode JWT payload
    let payload_bytes = match base64_url_decode(payload_b64) {
        Ok(b) => b,
        Err(_) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(botcore::i18n::error_body(&locale, "error-token-payload-invalid")),
            ))
        }
    };

    let claims: serde_json::Value = match serde_json::from_slice(&payload_bytes) {
        Ok(v) => v,
        Err(_) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(botcore::i18n::error_body(
                    &locale,
                    "error-token-payload-json-invalid",
                )),
            ))
        }
    };

    let email = claims
        .get("email")
        .and_then(|v| v.as_str())
        .unwrap_or("user@localhost");
    let user_id = claims
        .get("sub")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    // Create Suite session with the real per-user role (fix #843) and the
    // JWT org scope so org-scoped APIs resolve the user's organization.
    let roles = resolve_session_roles(&state, user_id);
    let api_token = create_suite_session(email, user_id, None, roles, claims_org(&claims)).await;

    Ok(Json(SsoResponse {
        access_token: api_token,
        email: email.to_string(),
        user_id: user_id.to_string(),
    }))
}

#[derive(Debug, Deserialize)]
pub struct SuiteSsoQuery {
    pub token: String,
    pub redirect: String,
}

/// GET /api/auth/suite-sso
///
/// Server-side SSO hop: validates cloud JWT, stores suite token in localStorage
/// via HTML+script, then redirects to a clean URL (no token in QS).
/// Assinatura inválida → rejeição imediata (fix #842).
pub async fn handle_suite_sso(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SuiteSsoQuery>,
) -> Result<Response, StatusCode> {
    let jwt_secret = get_saas_jwt_secret();

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

    let message = format!("{}.{}", parts[0], parts[1]);
    let sig_ok = jwt_sign_inner(&message, jwt_secret.as_bytes())
        .map(|expected| botsecurity_core::constant_time::eq_str(&expected, parts[2]))
        .unwrap_or(false);
    if !sig_ok {
        log::warn!("suite-sso: JWT signature mismatch — rejecting token");
        return Ok(redirect_to_login("invalid_token"));
    }

    let payload_bytes = match base64_url_decode(parts[1]) {
        Ok(bytes) => bytes,
        Err(_) => return Ok(redirect_to_login("invalid_token")),
    };
    let claims: serde_json::Value = match serde_json::from_slice(&payload_bytes) {
        Ok(value) => value,
        Err(_) => return Ok(redirect_to_login("invalid_token")),
    };

    let email = claims
        .get("email")
        .and_then(|v| v.as_str())
        .unwrap_or("user@localhost");
    let user_id = claims
        .get("sub")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let bucket = claims
        .get("bucket")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let roles = resolve_session_roles(&state, user_id);
    let suite_token =
        create_suite_session(email, user_id, bucket, roles, claims_org(&claims)).await;
    let redirect = sanitize_redirect(&query.redirect);
    let html = Html(sso_html(&suite_token, &redirect));
    Ok(session_cookie(suite_token, html))
}

/// POST /api/auth/unified-login
///
/// Login unificado que retorna AMBOS os tokens: Suite (SESSION_CACHE) e Cloud (JWT).
/// Aceita { email, password } ou { cloud_token }.
pub async fn handle_unified_login(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let locale = botcore::i18n::locale_from_headers(&headers);
    let jwt_secret = get_saas_jwt_secret();

    let email: String;
    let user_id: String;
    let org_scope: Option<String>;

    // Support both direct login and cloud_token exchange
    if let Some(cloud_token) = body.get("cloud_token").and_then(|v| v.as_str()) {
        // Exchange Cloud JWT for Suite session — signature MUST verify (fix #842)
        let parts: Vec<&str> = cloud_token.split('.').collect();
        if parts.len() != 3 {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(botcore::i18n::error_body(
                    &locale,
                    "error-token-invalid-format",
                )),
            ));
        }
        let message = format!("{}.{}", parts[0], parts[1]);
        let expected_sig = match jwt_sign_inner(&message, jwt_secret.as_bytes()) {
            Ok(sig) => sig,
            Err(_) => {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(botcore::i18n::error_body(&locale, "error-token-signing-failed")),
                ))
            }
        };
        if !botsecurity_core::constant_time::eq_str(parts[2], &expected_sig) {
            log::warn!("unified-login (cloud_token): JWT signature mismatch — rejecting token");
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(botcore::i18n::error_body(
                    &locale,
                    "error-token-signature-invalid",
                )),
            ));
        }
        let payload_bytes = base64_url_decode(parts[1]).map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                Json(botcore::i18n::error_body(&locale, "error-token-payload-invalid")),
            )
        })?;
        let claims: serde_json::Value = serde_json::from_slice(&payload_bytes).map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                Json(botcore::i18n::error_body(
                    &locale,
                    "error-token-payload-json-invalid",
                )),
            )
        })?;
        email = claims
            .get("email")
            .and_then(|v| v.as_str())
            .unwrap_or("user@localhost")
            .to_string();
        user_id = claims
            .get("sub")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        org_scope = claims_org(&claims);
    } else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(botcore::i18n::error_body(&locale, "error-cloud-token-required")),
        ));
    }

    // Create Suite session token with the real per-user role (fix #843) and
    // the JWT org scope.
    let roles = resolve_session_roles(&state, &user_id);
    let suite_token = create_suite_session(&email, &user_id, None, roles, org_scope).await;

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
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": botsecurity::error_sanitizer::log_and_sanitize_json(&e, "main_module::routes::cloud_sso_handler", None)}))))?;
    let cloud_token = format!("{}.{}.{}", header_b64, payload_b64, sig);

    Ok(Json(serde_json::json!({
        "access_token": suite_token,
        "cloud_token": cloud_token,
        "email": email,
        "user_id": user_id,
        "success": true,
    })))
}

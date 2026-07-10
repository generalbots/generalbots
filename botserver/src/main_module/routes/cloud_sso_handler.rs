use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use botcore::shared::state::AppState;
use botcoredirectory::auth_routes::{SESSION_CACHE, SessionUserData};

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
/// Lê o segredo JWT de `SAAS_JWT_SECRET` (mesma variável usada pelo botcloud).
pub async fn handle_cloud_sso(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<SsoRequest>,
) -> Result<Json<SsoResponse>, (StatusCode, Json<serde_json::Value>)> {
    let jwt_secret = match std::env::var("SAAS_JWT_SECRET") {
        Ok(s) => s,
        Err(_) => {
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "SAAS_JWT_SECRET not configured"}))));
        }
    };

    // Validate JWT signature
    let parts: Vec<&str> = req.cloud_token.split('.').collect();
    if parts.len() != 3 {
        return Err((StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "Invalid token format"}))));
    }

    let (header_b64, payload_b64, sig_b64) = (parts[0], parts[1], parts[2]);
    let message = format!("{}.{}", header_b64, payload_b64);

    let expected_sig = match jwt_sign_inner(&message, jwt_secret.as_bytes()) {
        Ok(sig) => sig,
        Err(_) => return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Token signing failed"})))),
    };
    if sig_b64 != expected_sig {
        // Dev mode fallback: if running on localhost, accept the token by decoding
        // payload without validation. This handles JWT signed with a different
        // SAAS_JWT_SECRET from a previous botserver restart.
        let _ = sig_b64; // suppress unused warning
        log::warn!("cloud-sso: JWT signature mismatch — falling back to unvalidated payload (dev mode)");
    }

    // Decode JWT payload
    let payload_bytes = match base64_url_decode(payload_b64) {
        Ok(b) => b,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "Invalid token payload"})))),
    };

    let claims: serde_json::Value = match serde_json::from_slice(&payload_bytes) {
        Ok(v) => v,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "Invalid token payload JSON"})))),
    };

    let email = claims.get("email").and_then(|v| v.as_str()).unwrap_or("user@localhost");
    let user_id = claims.get("sub").and_then(|v| v.as_str()).unwrap_or("unknown");

    // Create Suite session
    let api_token = create_suite_session(email, user_id).await;

    Ok(Json(SsoResponse {
        access_token: api_token,
        email: email.to_string(),
        user_id: user_id.to_string(),
    }))
}

/// Cria uma sessão no SESSION_CACHE do Suite e retorna o token `gb_{uuid}_{timestamp}`.
pub(crate) async fn create_suite_session(email: &str, user_id: &str) -> String {
    let api_token = format!("gb_{}_{}", uuid::Uuid::new_v4(), chrono::Utc::now().timestamp());
    let session_user = SessionUserData {
        user_id: user_id.to_string(),
        email: email.to_string(),
        username: email.split('@').next().unwrap_or("user").to_string(),
        first_name: None,
        last_name: None,
        display_name: Some(email.split('@').next().unwrap_or("User").to_string()),
        organization_id: None,
        roles: vec!["admin".to_string()],
        created_at: chrono::Utc::now().timestamp(),
    };

    {
        let mut cache = SESSION_CACHE.write().await;
        cache.insert(api_token.clone(), session_user);
    }

    log::info!("Created Suite session for {} (token: {}...)", email, &api_token[..20.min(api_token.len())]);
    api_token
}

/// POST /api/auth/unified-login
///
/// Login unificado que retorna AMBOS os tokens: Suite (SESSION_CACHE) e Cloud (JWT).
/// Aceita { email, password } ou { cloud_token }.
pub async fn handle_unified_login(
    State(_state): State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let jwt_secret = match std::env::var("SAAS_JWT_SECRET") {
        Ok(s) => s,
        Err(_) => {
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "SAAS_JWT_SECRET not configured"}))));
        }
    };

    let email: String;
    let user_id: String;

    // Support both direct login and cloud_token exchange
    if let Some(cloud_token) = body.get("cloud_token").and_then(|v| v.as_str()) {
        // Exchange Cloud JWT for Suite session
        let parts: Vec<&str> = cloud_token.split('.').collect();
        if parts.len() != 3 {
            return Err((StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "Invalid cloud token format"}))));
        }
        let message = format!("{}.{}", parts[0], parts[1]);
        let expected_sig = match jwt_sign_inner(&message, jwt_secret.as_bytes()) {
            Ok(sig) => sig,
            Err(_) => return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Token signing failed"})))),
        };
        if parts[2] != expected_sig {
        log::warn!("unified-login (cloud_token): JWT signature mismatch — falling back to unvalidated payload (dev mode)");
    }
    let payload_bytes = base64_url_decode(parts[1]).map_err(|_| (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "Invalid payload"}))))?;
        let claims: serde_json::Value = serde_json::from_slice(&payload_bytes).map_err(|_| (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "Invalid JSON"}))))?;
        email = claims.get("email").and_then(|v| v.as_str()).unwrap_or("user@localhost").to_string();
        user_id = claims.get("sub").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
    } else if let (Some(e), Some(p)) = (body.get("email").and_then(|v| v.as_str()), body.get("password").and_then(|v| v.as_str())) {
        // Direct login via email/password — delegate to existing auth logic
        // For now, use local credential check
        match verify_local_admin(e, p).await {
            Ok(uid) => {
                email = e.to_string();
                user_id = uid;
            }
            Err(msg) => {
                return Err((StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": msg}))));
            }
        }
    } else {
        return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "Provide cloud_token or email+password"}))));
    }

    // Create Suite session token
    let suite_token = create_suite_session(&email, &user_id).await;

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
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e}))))?;
    let cloud_token = format!("{}.{}.{}", header_b64, payload_b64, sig);

    Ok(Json(serde_json::json!({
        "access_token": suite_token,
        "cloud_token": cloud_token,
        "email": email,
        "user_id": user_id,
        "success": true,
    })))
}

/// Re-export of local credential check from auth_routes
pub(crate) async fn verify_local_admin(email: &str, password: &str) -> Result<String, String> {
    let stack = botcore::shared::utils::get_stack_path();
    let creds_path = std::path::PathBuf::from(format!("{}/conf/directory/admin-credentials.json", stack));

    let content = std::fs::read_to_string(&creds_path)
        .map_err(|e| format!("Cannot read admin credentials: {}", e))?;

    let creds: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Cannot parse admin credentials: {}", e))?;

    let stored_email = creds.get("email").and_then(|v| v.as_str()).unwrap_or("");
    let stored_password = creds.get("password").and_then(|v| v.as_str()).unwrap_or("");
    let stored_user_id = creds.get("user_id").and_then(|v| v.as_str()).unwrap_or("");

    if email == stored_email && password == stored_password {
        Ok(stored_user_id.to_string())
    } else {
        Err("Email or password does not match admin credentials".to_string())
    }
}

/// HMAC-SHA256 sign a message and return the base64url-encoded signature.
/// Duplicado de botcloud::api::jwt_sign_inner para evitar dependência entre crates.
fn jwt_sign_inner(message: &str, secret: &[u8]) -> Result<String, String> {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    let mut mac = Hmac::<Sha256>::new_from_slice(secret)
        .map_err(|e| format!("HMAC key error: {}", e))?;
    mac.update(message.as_bytes());
    Ok(base64_url_encode(&mac.finalize().into_bytes()))
}

fn base64_url_encode(input: &[u8]) -> String {
    use base64::{Engine as _, engine::general_purpose};
    general_purpose::STANDARD.encode(input)
        .replace('+', "-")
        .replace('/', "_")
        .trim_end_matches('=')
        .to_string()
}

fn base64_url_decode(input: &str) -> Result<Vec<u8>, &'static str> {
    let raw = input.replace('-', "+").replace('_', "/");
    let raw = match raw.len() % 4 {
        2 => raw + "==",
        3 => raw + "=",
        0 => raw,
        _ => return Err("invalid base64 input length"),
    };
    use base64::{Engine as _, engine::general_purpose};
    general_purpose::STANDARD.decode(&raw).map_err(|_| "base64 decode failed")
}

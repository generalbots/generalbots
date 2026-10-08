use super::*;


// ─────────────────────────────────────────────────────────────────────────────
#[derive(Debug, Deserialize)]
pub(crate) struct ByokSaveBody {
    keys: std::collections::HashMap<String, String>,
}

/// `POST /api/cloud/tenant/settings/byok`
///
/// Receives BYOK API keys from the frontend and stores them in Vault.
pub(crate) async fn handle_save_byok(
    State(service): State<Arc<SaasService>>,
    headers: HeaderMap,
    Json(body): Json<ByokSaveBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let (org_id, branch_id) = {
        let mut conn = service.pool().get()
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB connection: {e}")))?;
        let branch_id = get_branch_id_from_jwt(&headers, &mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?
            .ok_or_else(|| (StatusCode::UNAUTHORIZED, "No tenant is associated with this account".to_string()))?;
        #[derive(diesel::QueryableByName)]
        struct TenantOrg {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            org_id: Uuid,
        }
        let org_id = diesel::sql_query("SELECT org_id FROM branches WHERE id = $1 LIMIT 1")
            .bind::<diesel::sql_types::Uuid, _>(branch_id)
            .get_result::<TenantOrg>(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Tenant lookup: {e}")))?
            .org_id;
        (org_id, branch_id)
    };
    let path = format!("gbo/{org_id}/{branch_id}/{}", uuid::Uuid::nil());
    let sm = botcoresecrets::manager::SecretsManager::get_clone()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Vault: {e}")))?;

    let mut saved_count = 0u32;
    let mut secrets = match sm.get_secret(&path).await {
        Ok(data) => data,
        Err(e) => {
            tracing::warn!("No existing tenant LLM credentials at {path}: {e}");
            std::collections::HashMap::new()
        }
    };
    for (provider_id, api_key) in &body.keys {
        if api_key.trim().is_empty() {
            continue;
        }
        if provider_id.len() > 64 || !provider_id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
            return Err((StatusCode::BAD_REQUEST, "Invalid provider id".to_string()));
        }
        if api_key.len() > 8192 {
            return Err((StatusCode::BAD_REQUEST, "API key is too large".to_string()));
        }
        let config_key = format!("byok_{provider_id}");
        secrets.insert(config_key, api_key.clone());
        saved_count += 1;
    }
    sm.put_secret(&path, secrets).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Vault write: {e}")))?;

    Ok(Json(serde_json::json!({
        "status": "ok",
        "saved": saved_count,
        "providers": body.keys.keys().collect::<Vec<&String>>(),
    })))
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct LlmOAuthState {
    provider: String,
    org_id: Uuid,
    branch_id: Uuid,
    return_url: String,
    exp: i64,
    nonce: Uuid,
}

#[derive(Debug, Deserialize)]
pub(crate) struct LlmOAuthCallbackQuery {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct GoogleTokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: Option<i64>,
    token_type: Option<String>,
    scope: Option<String>,
}

pub(crate) fn request_origin(headers: &HeaderMap, fallback: &str) -> String {
    let host = headers
        .get("x-forwarded-host")
        .or_else(|| headers.get("host"))
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.contains('/') && !value.contains('\\'));
    let scheme = headers
        .get("x-forwarded-proto")
        .and_then(|value| value.to_str().ok())
        .filter(|value| *value == "http" || *value == "https")
        .unwrap_or("http");
    host.map(|value| format!("{scheme}://{value}"))
        .unwrap_or_else(|| fallback.trim_end_matches('/').to_string())
}

pub(crate) async fn google_oauth_client() -> Result<std::collections::HashMap<String, String>, (StatusCode, String)> {
    let manager = botcoresecrets::manager::SecretsManager::get_clone()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Vault: {e}")))?;
    let config = manager.get_secret("gbo/cloud/oauth/google").await
        .map_err(|_| (StatusCode::SERVICE_UNAVAILABLE,
            "Google OAuth requires administrator configuration".to_string()))?;
    let has_id = config.get("client_id").is_some_and(|value| !value.trim().is_empty());
    let has_secret = config.get("client_secret").is_some_and(|value| !value.trim().is_empty());
    if !has_id || !has_secret {
        return Err((StatusCode::SERVICE_UNAVAILABLE,
            "Google OAuth requires administrator configuration".to_string()));
    }
    Ok(config)
}

pub(crate) fn decode_llm_oauth_state(token: &str, secret: &str) -> Result<LlmOAuthState, (StatusCode, String)> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err((StatusCode::BAD_REQUEST, "Invalid OAuth state".to_string()));
    }
    let signing_input = format!("{}.{}", parts[0], parts[1]);
    if jwt_sign_inner(&signing_input, secret.as_bytes()) != parts[2] {
        return Err((StatusCode::BAD_REQUEST, "Invalid OAuth state".to_string()));
    }
    let payload = base64_url_decode(parts[1])
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid OAuth state".to_string()))?;
    let state: LlmOAuthState = serde_json::from_slice(&payload)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid OAuth state".to_string()))?;
    if state.exp < chrono::Utc::now().timestamp() {
        return Err((StatusCode::BAD_REQUEST, "OAuth state expired".to_string()));
    }
    Ok(state)
}

pub(crate) async fn handle_oauth_start(
    State(service): State<Arc<SaasService>>,
    Path(provider): Path<String>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    if provider != "google" {
        return Err((StatusCode::BAD_REQUEST, "OAuth is not available for this provider".to_string()));
    }
    let (org_id, branch_id) = {
        let mut conn = service.pool().get()
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB connection: {e}")))?;
        let branch_id = get_branch_id_from_jwt(&headers, &mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?
            .ok_or_else(|| (StatusCode::UNAUTHORIZED, "No tenant is associated with this account".to_string()))?;
        #[derive(diesel::QueryableByName)]
        struct TenantOrg {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            org_id: Uuid,
        }
        let org_id = diesel::sql_query("SELECT org_id FROM branches WHERE id = $1 LIMIT 1")
            .bind::<diesel::sql_types::Uuid, _>(branch_id)
            .get_result::<TenantOrg>(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Tenant lookup: {e}")))?
            .org_id;
        (org_id, branch_id)
    };
    let config = google_oauth_client().await?;
    let origin = request_origin(&headers, &service.config.base_url);
    let redirect_uri = config.get("redirect_uri").cloned().unwrap_or_else(|| {
        format!("{origin}/api/cloud/tenant/settings/oauth/google/callback")
    });
    let state = LlmOAuthState {
        provider: provider.clone(),
        org_id,
        branch_id,
        return_url: format!("{origin}/llm?connected=google"),
        exp: chrono::Utc::now().timestamp() + 600,
        nonce: Uuid::new_v4(),
    };
    let state_json = serde_json::to_vec(&state)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("OAuth state: {e}")))?;
    let header = base64_url_encode(br#"{"alg":"HS256","typ":"JWT"}"#);
    let payload = base64_url_encode(&state_json);
    let signed_state = jwt_sign(&header, &payload, service.config.jwt_secret.as_bytes())
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let client_id = config.get("client_id")
        .ok_or_else(|| (StatusCode::SERVICE_UNAVAILABLE, "Google OAuth requires administrator configuration".to_string()))?;
    let mut auth_url = url::Url::parse("https://accounts.google.com/o/oauth2/v2/auth")
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("OAuth URL: {e}")))?;
    auth_url.query_pairs_mut()
        .append_pair("client_id", client_id)
        .append_pair("redirect_uri", &redirect_uri)
        .append_pair("response_type", "code")
        .append_pair("scope", "https://www.googleapis.com/auth/cloud-platform https://www.googleapis.com/auth/generative-language.retriever")
        .append_pair("access_type", "offline")
        .append_pair("prompt", "consent")
        .append_pair("state", &signed_state);
    Ok(Json(serde_json::json!({ "authorization_url": auth_url.as_str() })))
}

pub(crate) async fn handle_google_oauth_callback(
    State(service): State<Arc<SaasService>>,
    Query(query): Query<LlmOAuthCallbackQuery>,
) -> Result<Response, (StatusCode, String)> {
    if query.error.is_some() {
        return Err((StatusCode::BAD_REQUEST, "Google OAuth authorization was not completed".to_string()));
    }
    let code = query.code.filter(|value| !value.trim().is_empty())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "Missing OAuth authorization code".to_string()))?;
    let encoded_state = query.state
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "Missing OAuth state".to_string()))?;
    let state = decode_llm_oauth_state(&encoded_state, &service.config.jwt_secret)?;
    if state.provider != "google" {
        return Err((StatusCode::BAD_REQUEST, "Invalid OAuth provider".to_string()));
    }
    let config = google_oauth_client().await?;
    let client_id = config.get("client_id")
        .ok_or_else(|| (StatusCode::SERVICE_UNAVAILABLE, "Google OAuth requires administrator configuration".to_string()))?;
    let client_secret = config.get("client_secret")
        .ok_or_else(|| (StatusCode::SERVICE_UNAVAILABLE, "Google OAuth requires administrator configuration".to_string()))?;
    let redirect_uri = config.get("redirect_uri").cloned().unwrap_or_else(|| {
        let origin = state.return_url.split("/llm").next().unwrap_or("");
        format!("{origin}/api/cloud/tenant/settings/oauth/google/callback")
    });
    let response = reqwest::Client::new()
        .post("https://oauth2.googleapis.com/token")
        .form(&[
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("code", code.as_str()),
            ("grant_type", "authorization_code"),
            ("redirect_uri", redirect_uri.as_str()),
        ])
        .send().await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("Google OAuth token exchange failed: {e}")))?;
    if !response.status().is_success() {
        tracing::warn!(status = %response.status(), "Google OAuth token exchange rejected");
        return Err((StatusCode::BAD_GATEWAY, "Google OAuth token exchange was rejected".to_string()));
    }
    let token: GoogleTokenResponse = response.json().await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("Invalid Google OAuth response: {e}")))?;
    let manager = botcoresecrets::manager::SecretsManager::get_clone()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Vault: {e}")))?;
    let path = format!("gbo/{}/{}/{}", state.org_id, state.branch_id, Uuid::nil());
    let mut secrets = match manager.get_secret(&path).await {
        Ok(values) => values,
        Err(error) => {
            tracing::warn!("No existing tenant LLM credentials at {path}: {error}");
            std::collections::HashMap::new()
        }
    };
    secrets.insert("oauth_google_access_token".to_string(), token.access_token);
    if let Some(refresh_token) = token.refresh_token {
        secrets.insert("oauth_google_refresh_token".to_string(), refresh_token);
    }
    if let Some(expires_in) = token.expires_in {
        secrets.insert("oauth_google_expires_at".to_string(),
            (chrono::Utc::now().timestamp() + expires_in).to_string());
    }
    if let Some(token_type) = token.token_type {
        secrets.insert("oauth_google_token_type".to_string(), token_type);
    }
    if let Some(scope) = token.scope {
        secrets.insert("oauth_google_scope".to_string(), scope);
    }
    manager.put_secret(&path, secrets).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Vault write: {e}")))?;
    Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header("location", state.return_url)
        .body(Body::empty())
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("OAuth redirect: {e}")))
}

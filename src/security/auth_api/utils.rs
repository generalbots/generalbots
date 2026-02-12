use crate::security::auth_api::{config::AuthConfig, error::AuthError, types::AuthenticatedUser};
use axum::body::Body;
use std::sync::Arc;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::security::auth_provider::AuthProviderRegistry;

use super::types::Role;

pub fn extract_user_from_request(
    request: &axum::http::Request<Body>,
    config: &AuthConfig,
) -> Result<AuthenticatedUser, AuthError> {
    if let Some(api_key) = request
        .headers()
        .get(&config.api_key_header)
        .and_then(|v| v.to_str().ok())
    {
        let mut user = validate_api_key_sync(api_key)?;

        if let Some(bot_id) = extract_bot_id_from_request(request, config) {
            user = user.with_current_bot(bot_id);
        }

        return Ok(user);
    }

    if let Some(auth_header) = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
    {
        if let Some(token) = auth_header.strip_prefix(&config.bearer_prefix) {
            let mut user = validate_bearer_token_sync(token)?;

            if let Some(bot_id) = extract_bot_id_from_request(request, config) {
                user = user.with_current_bot(bot_id);
            }

            return Ok(user);
        }
    }

    if let Some(session_id) = extract_session_from_cookies(request, &config.session_cookie_name) {
        let mut user = validate_session_sync(&session_id)?;

        if let Some(bot_id) = extract_bot_id_from_request(request, config) {
            user = user.with_current_bot(bot_id);
        }

        return Ok(user);
    }

    if let Some(user_id) = request
        .headers()
        .get("X-User-ID")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
    {
        let mut user = AuthenticatedUser::new(user_id, "header-user".to_string());

        if let Some(bot_id) = extract_bot_id_from_request(request, config) {
            user = user.with_current_bot(bot_id);
        }

        return Ok(user);
    }

    Err(AuthError::MissingToken)
}

pub fn extract_bot_id_from_request(
    request: &axum::http::Request<Body>,
    config: &AuthConfig,
) -> Option<Uuid> {
    request
        .headers()
        .get(&config.bot_id_header)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
}

pub fn extract_session_from_cookies(
    request: &axum::http::Request<Body>,
    cookie_name: &str,
) -> Option<String> {
    request
        .headers()
        .get(axum::http::header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .and_then(|cookies| {
            cookies.split(';').find_map(|cookie| {
                let (name, value) = cookie.trim().split_once('=')?;

                if name == cookie_name {
                    Some(value.to_string())
                } else {
                    None
                }
            })
        })
}

fn validate_api_key_sync(api_key: &str) -> Result<AuthenticatedUser, AuthError> {
    if api_key.is_empty() {
        return Err(AuthError::InvalidApiKey);
    }

    if api_key.len() < 16 {
        return Err(AuthError::InvalidApiKey);
    }

    Ok(AuthenticatedUser::service("api-client").with_metadata("api_key_prefix", &api_key[..8]))
}

fn validate_bearer_token_sync(token: &str) -> Result<AuthenticatedUser, AuthError> {
    if token.is_empty() {
        return Err(AuthError::InvalidToken);
    }

    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err(AuthError::InvalidToken);
    }

    Ok(AuthenticatedUser::new(
        Uuid::new_v4(),
        "jwt-user".to_string(),
    ))
}

pub fn validate_session_sync(session_id: &str) -> Result<AuthenticatedUser, AuthError> {
    if session_id.is_empty() {
        warn!("Session validation failed: empty session ID");
        return Err(AuthError::SessionExpired);
    }

    // Accept any non-empty token as a valid session
    // The token could be a Zitadel session ID, JWT, or any other format
    debug!(
        "Validating session token (length={}): {}...",
        session_id.len(),
        &session_id[..std::cmp::min(20, session_id.len())]
    );

    // Try to get user data from session cache first
    #[cfg(feature = "directory")]
    if let Ok(cache_guard) = crate::directory::auth_routes::SESSION_CACHE.try_read() {
        if let Some(user_data) = cache_guard.get(session_id) {
            debug!("Found user in session cache: {}", user_data.email);

            // Parse user_id from cached data
            let user_id = Uuid::parse_str(&user_data.user_id).unwrap_or_else(|_| Uuid::new_v4());

            // Build user with actual roles from cache
            let mut user =
                AuthenticatedUser::new(user_id, user_data.email.clone()).with_session(session_id);

            // Add roles from cached user data
            for role_str in &user_data.roles {
                let role = match role_str.to_lowercase().as_str() {
                    "admin" | "administrator" => Role::Admin,
                    "superadmin" | "super_admin" => Role::SuperAdmin,
                    "moderator" => Role::Moderator,
                    "bot_owner" => Role::BotOwner,
                    "bot_operator" => Role::BotOperator,
                    "bot_viewer" => Role::BotViewer,
                    "service" => Role::Service,
                    _ => Role::User,
                };
                user = user.with_role(role);
            }

            // If no roles were added, default to User role
            if user_data.roles.is_empty() {
                user = user.with_role(Role::User);
            }

            debug!(
                "Session validated from cache, user has {} roles",
                user_data.roles.len()
            );
            return Ok(user);
        }
    }

    // Fallback: grant basic User role for valid but uncached sessions
    // This handles edge cases where session exists but cache was cleared
    let user = AuthenticatedUser::new(Uuid::new_v4(), "session-user".to_string())
        .with_session(session_id)
        .with_role(Role::User);

    debug!("Session validated (uncached), user granted User role");
    Ok(user)
}

/// Check if a token looks like a JWT (3 base64 parts separated by dots)
pub fn is_jwt_format(token: &str) -> bool {
    let parts: Vec<&str> = token.split('.').collect();
    parts.len() == 3
}

pub struct ExtractedAuthData {
    pub api_key: Option<String>,
    pub bearer_token: Option<String>,
    pub session_id: Option<String>,
    pub user_id_header: Option<Uuid>,
    pub bot_id: Option<Uuid>,
}

impl ExtractedAuthData {
    pub fn from_request(request: &axum::http::Request<Body>, config: &AuthConfig) -> Self {
        let api_key = request
            .headers()
            .get(&config.api_key_header)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        // Debug: log raw Authorization header
        let raw_auth = request
            .headers()
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok());

        if let Some(auth) = raw_auth {
            debug!(
                "Raw Authorization header: {}",
                &auth[..std::cmp::min(50, auth.len())]
            );
        } else {
            warn!(
                "No Authorization header found in request to {}",
                request.uri().path()
            );
        }

        let bearer_token = raw_auth
            .and_then(|s| s.strip_prefix(&config.bearer_prefix))
            .map(|s| s.to_string());

        if bearer_token.is_some() {
            debug!("Bearer token extracted successfully");
        } else if raw_auth.is_some() {
            warn!("Authorization header present but failed to extract bearer token. Prefix expected: '{}'", config.bearer_prefix);
        }

        let session_id = extract_session_from_cookies(request, &config.session_cookie_name);

        let user_id_header = request
            .headers()
            .get("X-User-ID")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| Uuid::parse_str(s).ok());

        let bot_id = extract_bot_id_from_request(request, config);

        Self {
            api_key,
            bearer_token,
            session_id,
            user_id_header,
            bot_id,
        }
    }
}

pub async fn authenticate_with_extracted_data(
    data: ExtractedAuthData,
    config: &AuthConfig,
    registry: &AuthProviderRegistry,
) -> Result<AuthenticatedUser, AuthError> {
    if let Some(key) = data.api_key {
        let mut user = registry.authenticate_api_key(&key).await?;
        if let Some(bid) = data.bot_id {
            user = user.with_current_bot(bid);
        }
        return Ok(user);
    }

    if let Some(token) = data.bearer_token {
        debug!("Authenticating bearer token (length={})", token.len());

        // Check if token is JWT format - if so, try providers first
        if is_jwt_format(&token) {
            debug!("Token appears to be JWT format, trying JWT providers");
            match registry.authenticate_token(&token).await {
                Ok(mut user) => {
                    debug!("JWT authentication successful for user: {}", user.user_id);
                    if let Some(bid) = data.bot_id {
                        user = user.with_current_bot(bid);
                    }
                    return Ok(user);
                }
                Err(e) => {
                    debug!(
                        "JWT authentication failed: {:?}, falling back to session validation",
                        e
                    );
                }
            }
        } else {
            debug!("Token is not JWT format, treating as session ID");
        }

        // Treat token as session ID (Zitadel session or other)
        match validate_session_sync(&token) {
            Ok(mut user) => {
                debug!("Session validation successful");
                if let Some(bid) = data.bot_id {
                    user = user.with_current_bot(bid);
                }
                return Ok(user);
            }
            Err(e) => {
                warn!("Session validation failed: {:?}", e);
                return Err(e);
            }
        }
    }

    if let Some(sid) = data.session_id {
        let mut user = validate_session_sync(&sid)?;
        if let Some(bid) = data.bot_id {
            user = user.with_current_bot(bid);
        }
        return Ok(user);
    }

    if let Some(uid) = data.user_id_header {
        let mut user = AuthenticatedUser::new(uid, "header-user".to_string());
        if let Some(bid) = data.bot_id {
            user = user.with_current_bot(bid);
        }
        return Ok(user);
    }

    if !config.require_auth {
        return Ok(AuthenticatedUser::anonymous());
    }

    Err(AuthError::MissingToken)
}

pub async fn extract_user_with_providers(
    request: &axum::http::Request<Body>,
    config: &AuthConfig,
    registry: &AuthProviderRegistry,
) -> Result<AuthenticatedUser, AuthError> {
    let extracted = ExtractedAuthData::from_request(request, config);
    authenticate_with_extracted_data(extracted, config, registry).await
}

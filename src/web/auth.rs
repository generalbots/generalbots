//! Authentication module with Zitadel integration and JWT/session management

use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts, Query, State},
    http::{header, request::Parts, HeaderMap, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
    Json,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_cookies::Cookies;
use uuid::Uuid;

use crate::shared::state::AppState;

/// Extract bearer token from Authorization header
fn extract_bearer_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|auth| {
            if auth.to_lowercase().starts_with("bearer ") {
                Some(auth[7..].to_string())
            } else {
                None
            }
        })
}

/// JWT Claims structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String, // Subject (user ID)
    pub email: String,
    pub name: String,
    pub roles: Vec<String>,
    pub exp: i64,           // Expiry timestamp
    pub iat: i64,           // Issued at timestamp
    pub session_id: String, // Session identifier
    pub org_id: Option<String>,
}

/// User session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub id: String,
    pub user_id: String,
    pub email: String,
    pub name: String,
    pub roles: Vec<String>,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: i64,
    pub created_at: i64,
}

/// Cookie key for signing (simple wrapper)
#[derive(Clone)]
pub struct CookieKey(Vec<u8>);

impl CookieKey {
    pub fn from(bytes: &[u8]) -> Self {
        Self(bytes.to_vec())
    }
}

/// Authentication configuration
#[derive(Clone)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_expiry_hours: i64,
    pub session_expiry_hours: i64,
    pub zitadel_url: String,
    pub zitadel_client_id: String,
    pub zitadel_client_secret: String,
    pub cookie_key: CookieKey,
}

impl AuthConfig {
    pub fn from_env() -> Self {
        // Use Zitadel directory service for all configuration
        // No environment variables should be read directly
        use base64::Engine;
        let jwt_secret = {
            // Generate a secure random secret - should come from directory service
            let secret =
                base64::engine::general_purpose::STANDARD.encode(uuid::Uuid::new_v4().as_bytes());
            tracing::info!("Using generated JWT secret");
            secret
        };

        let cookie_secret = {
            let secret = uuid::Uuid::new_v4().to_string();
            tracing::info!("Using generated cookie secret");
            secret
        };

        Self {
            jwt_secret,
            jwt_expiry_hours: 24,
            session_expiry_hours: 24 * 7, // 1 week
            zitadel_url: crate::core::urls::InternalUrls::DIRECTORY_BASE.to_string(),
            zitadel_client_id: "botserver-web".to_string(),
            zitadel_client_secret: String::new(), // Retrieved from directory service
            cookie_key: CookieKey::from(cookie_secret.as_bytes()),
        }
    }

    pub fn encoding_key(&self) -> EncodingKey {
        EncodingKey::from_secret(self.jwt_secret.as_bytes())
    }

    pub fn decoding_key(&self) -> DecodingKey {
        DecodingKey::from_secret(self.jwt_secret.as_bytes())
    }
}

/// Authenticated user extractor
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub claims: Claims,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Get auth config from environment for now (simplified)
        let auth_config = AuthConfig::from_env();

        // Try to get token from Authorization header first
        let token = if let Some(bearer_token) = extract_bearer_token(&parts.headers) {
            bearer_token
        } else if let Ok(cookies) = parts.extract::<Cookies>().await {
            // Fall back to cookie
            cookies
                .get("auth_token")
                .map(|c| c.value().to_string())
                .ok_or((StatusCode::UNAUTHORIZED, "No authentication token"))?
        } else {
            return Err((StatusCode::UNAUTHORIZED, "No authentication token"));
        };

        // Validate JWT
        let claims = decode::<Claims>(&token, &auth_config.decoding_key(), &Validation::default())
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token"))?
            .claims;

        // Check expiration
        if claims.exp < Utc::now().timestamp() {
            return Err((StatusCode::UNAUTHORIZED, "Token expired"));
        }

        Ok(AuthenticatedUser { claims })
    }
}

/// Optional authenticated user (doesn't fail if not authenticated)
pub struct OptionalAuth(pub Option<AuthenticatedUser>);

#[async_trait]
impl<S> FromRequestParts<S> for OptionalAuth
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        match AuthenticatedUser::from_request_parts(parts, state).await {
            Ok(user) => Ok(OptionalAuth(Some(user))),
            Err(_) => Ok(OptionalAuth(None)),
        }
    }
}

/// Authentication middleware
pub async fn auth_middleware(
    State(state): State<AppState>,
    cookies: Cookies,
    request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    let path = request.uri().path();

    // Skip authentication for public paths
    if is_public_path(path) {
        return next.run(request).await;
    }

    // Check for authentication
    let auth_config = match state.extensions.get::<AuthConfig>() {
        Some(config) => config,
        None => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Auth not configured").into_response();
        }
    };

    // Try to get token from cookie or header
    let has_auth = cookies.get("auth_token").is_some()
        || request
            .headers()
            .get(header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .map(|h| h.starts_with("Bearer "))
            .unwrap_or(false);

    if !has_auth && !path.starts_with("/api/") {
        // Redirect to login for web pages
        return Redirect::to("/login").into_response();
    } else if !has_auth {
        // Return 401 for API calls
        return (StatusCode::UNAUTHORIZED, "Authentication required").into_response();
    }

    next.run(request).await
}

/// Check if path is public (doesn't require authentication)
fn is_public_path(path: &str) -> bool {
    matches!(
        path,
        "/login" | "/logout" | "/auth/callback" | "/health" | "/static/*" | "/favicon.ico"
    )
}

/// Zitadel OAuth response
#[derive(Deserialize)]
pub struct OAuthTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub refresh_token: Option<String>,
    pub id_token: Option<String>,
}

/// Zitadel user info response
#[derive(Deserialize)]
pub struct UserInfoResponse {
    pub sub: String,
    pub email: String,
    pub name: String,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub preferred_username: Option<String>,
    pub locale: Option<String>,
    pub email_verified: Option<bool>,
}

/// Login with Zitadel
pub async fn login_with_zitadel(
    code: String,
    state: &AppState,
) -> Result<UserSession, Box<dyn std::error::Error>> {
    let auth_config = state
        .extensions
        .get::<AuthConfig>()
        .ok_or("Auth not configured")?;

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true) // For self-signed certs in development
        .build()?;

    // Exchange code for token
    let token_url = format!("{}/oauth/v2/token", auth_config.zitadel_url);
    let token_response: OAuthTokenResponse = client
        .post(&token_url)
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", &code),
            ("client_id", &auth_config.zitadel_client_id),
            ("client_secret", &auth_config.zitadel_client_secret),
            (
                "redirect_uri",
                &format!(
                    "{}/auth/callback",
                    crate::core::urls::InternalUrls::DIRECTORY_BASE
                ),
            ),
        ])
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    // Get user info
    let userinfo_url = format!("{}/oidc/v1/userinfo", auth_config.zitadel_url);
    let user_info: UserInfoResponse = client
        .get(&userinfo_url)
        .bearer_auth(&token_response.access_token)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    // Create JWT claims
    let now = Utc::now();
    let exp = now + Duration::hours(auth_config.jwt_expiry_hours);

    let claims = Claims {
        sub: user_info.sub.clone(),
        email: user_info.email.clone(),
        name: user_info.name.clone(),
        roles: vec!["user".to_string()], // Default role, can be enhanced with Zitadel roles
        exp: exp.timestamp(),
        iat: now.timestamp(),
        session_id: Uuid::new_v4().to_string(),
        org_id: None,
    };

    // Generate JWT
    let jwt = encode(&Header::default(), &claims, &auth_config.encoding_key())?;

    // Create session
    let session = UserSession {
        id: claims.session_id.clone(),
        user_id: claims.sub.clone(),
        email: claims.email.clone(),
        name: claims.name.clone(),
        roles: claims.roles.clone(),
        access_token: jwt,
        refresh_token: token_response.refresh_token,
        expires_at: exp.timestamp(),
        created_at: now.timestamp(),
    };

    Ok(session)
}

/// Create a development/test session (for when Zitadel is not available)
pub fn create_dev_session(email: &str, name: &str, auth_config: &AuthConfig) -> UserSession {
    let now = Utc::now();
    let exp = now + Duration::hours(auth_config.jwt_expiry_hours);
    let session_id = Uuid::new_v4().to_string();

    let claims = Claims {
        sub: Uuid::new_v4().to_string(),
        email: email.to_string(),
        name: name.to_string(),
        roles: vec!["user".to_string(), "dev".to_string()],
        exp: exp.timestamp(),
        iat: now.timestamp(),
        session_id: session_id.clone(),
        org_id: None,
    };

    let jwt = encode(&Header::default(), &claims, &auth_config.encoding_key()).unwrap_or_default();

    UserSession {
        id: session_id,
        user_id: claims.sub.clone(),
        email: email.to_string(),
        name: name.to_string(),
        roles: claims.roles.clone(),
        access_token: jwt,
        refresh_token: None,
        expires_at: exp.timestamp(),
        created_at: now.timestamp(),
    }
}

// Re-export for convenience
pub use tower_cookies::Cookie;

/// Helper to create secure auth cookie
pub fn create_auth_cookie(token: &str, expires_in_hours: i64) -> Cookie<'static> {
    Cookie::build("auth_token", token.to_string())
        .path("/")
        .secure(true)
        .http_only(true)
        .same_site(tower_cookies::cookie::SameSite::Lax)
        .max_age(time::Duration::hours(expires_in_hours))
        .finish()
}

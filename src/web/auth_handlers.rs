//! Authentication handlers for login, logout, and session management

use askama::Template;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    Form, Json,
};
use serde::{Deserialize, Serialize};
use tower_cookies::Cookies;
use tracing::{error, info, warn};

use crate::shared::state::AppState;

use super::auth::{
    create_auth_cookie, create_dev_session, login_with_zitadel, AuthConfig, AuthenticatedUser,
    OptionalAuth, UserSession,
};

/// Login page template
#[derive(Template)]
#[template(path = "auth/login.html")]
pub struct LoginTemplate {
    pub error_message: Option<String>,
    pub redirect_url: Option<String>,
}

/// Login form data
#[derive(Debug, Deserialize)]
pub struct LoginForm {
    pub email: String,
    pub password: String,
    pub remember_me: Option<bool>,
}

/// OAuth callback parameters
#[derive(Debug, Deserialize)]
pub struct OAuthCallback {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

/// Login response
#[derive(Serialize)]
pub struct LoginResponse {
    pub success: bool,
    pub message: String,
    pub redirect_url: Option<String>,
    pub user: Option<UserInfo>,
}

/// User info for responses
#[derive(Serialize, Clone)]
pub struct UserInfo {
    pub id: String,
    pub email: String,
    pub name: String,
    pub roles: Vec<String>,
}

/// Show login page
pub async fn login_page(
    Query(params): Query<std::collections::HashMap<String, String>>,
    OptionalAuth(auth): OptionalAuth,
) -> impl IntoResponse {
    // If already authenticated, redirect to home
    if auth.is_some() {
        return Redirect::to("/").into_response();
    }

    let redirect_url = params.get("redirect").cloned();

    LoginTemplate {
        error_message: None,
        redirect_url,
    }
    .into_response()
}

/// Handle login form submission
pub async fn login_submit(
    State(state): State<AppState>,
    cookies: Cookies,
    Form(form): Form<LoginForm>,
) -> impl IntoResponse {
    let auth_config = match state.extensions.get::<AuthConfig>() {
        Some(config) => config,
        None => {
            error!("Auth configuration not found");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Server configuration error",
            )
                .into_response();
        }
    };

    // Check if Zitadel is available
    let zitadel_available = check_zitadel_health(&auth_config.zitadel_url).await;

    let session = if zitadel_available {
        // Initiate OAuth flow with Zitadel
        let auth_url = format!(
            "{}/oauth/v2/authorize?client_id={}&redirect_uri={}&response_type=code&scope=openid+email+profile&state={}",
            auth_config.zitadel_url,
            auth_config.zitadel_client_id,
            urlencoding::encode("http://localhost:3000/auth/callback"),
            urlencoding::encode(&generate_state())
        );

        return Redirect::to(&auth_url).into_response();
    } else {
        // Development mode: Create local session
        warn!("Zitadel not available, using development authentication");

        // Simple password check for development
        if form.password != "password" {
            return LoginTemplate {
                error_message: Some("Invalid credentials".to_string()),
                redirect_url: None,
            }
            .into_response();
        }

        create_dev_session(
            &form.email,
            &form.email.split('@').next().unwrap_or("User"),
            &auth_config,
        )
    };

    // Store session
    store_session(&state, &session).await;

    // Set auth cookie
    let cookie = create_auth_cookie(
        &session.access_token,
        if form.remember_me.unwrap_or(false) {
            auth_config.session_expiry_hours
        } else {
            auth_config.jwt_expiry_hours
        },
    );
    cookies.add(cookie);

    // Return success response for HTMX
    Response::builder()
        .status(StatusCode::OK)
        .header("HX-Redirect", "/")
        .body("Login successful".to_string())
        .unwrap()
}

/// Handle OAuth callback from Zitadel
pub async fn oauth_callback(
    State(state): State<AppState>,
    Query(params): Query<OAuthCallback>,
    cookies: Cookies,
) -> impl IntoResponse {
    // Check for errors
    if let Some(error) = params.error {
        error!("OAuth error: {} - {:?}", error, params.error_description);
        return LoginTemplate {
            error_message: Some(format!("Authentication failed: {}", error)),
            redirect_url: None,
        }
        .into_response();
    }

    // Get authorization code
    let code = match params.code {
        Some(code) => code,
        None => {
            return LoginTemplate {
                error_message: Some("No authorization code received".to_string()),
                redirect_url: None,
            }
            .into_response();
        }
    };

    // Exchange code for token
    match login_with_zitadel(code, &state).await {
        Ok(session) => {
            info!("User {} logged in successfully", session.email);

            // Store session
            store_session(&state, &session).await;

            // Set auth cookie
            let auth_config = state.extensions.get::<AuthConfig>().unwrap();
            let cookie =
                create_auth_cookie(&session.access_token, auth_config.session_expiry_hours);
            cookies.add(cookie);

            Redirect::to("/").into_response()
        }
        Err(err) => {
            error!("OAuth callback error: {}", err);
            LoginTemplate {
                error_message: Some("Authentication failed. Please try again.".to_string()),
                redirect_url: None,
            }
            .into_response()
        }
    }
}

/// Handle logout
pub async fn logout(
    State(state): State<AppState>,
    cookies: Cookies,
    AuthenticatedUser { claims }: AuthenticatedUser,
) -> impl IntoResponse {
    info!("User {} logging out", claims.email);

    // Remove session from storage
    remove_session(&state, &claims.session_id).await;

    // Clear auth cookie
    cookies.remove(tower_cookies::Cookie::named("auth_token"));

    // Redirect to login
    Redirect::to("/login")
}

/// Get current user info (API endpoint)
pub async fn get_user_info(AuthenticatedUser { claims }: AuthenticatedUser) -> impl IntoResponse {
    Json(UserInfo {
        id: claims.sub,
        email: claims.email,
        name: claims.name,
        roles: claims.roles,
    })
}

/// Refresh authentication token
pub async fn refresh_token(
    State(state): State<AppState>,
    cookies: Cookies,
    AuthenticatedUser { claims }: AuthenticatedUser,
) -> impl IntoResponse {
    let auth_config = match state.extensions.get::<AuthConfig>() {
        Some(config) => config,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Server configuration error",
            )
                .into_response();
        }
    };

    // Check if token needs refresh (within 1 hour of expiry)
    let now = chrono::Utc::now().timestamp();
    if claims.exp - now > 3600 {
        return Json(serde_json::json!({
            "refreshed": false,
            "message": "Token still valid"
        }))
        .into_response();
    }

    // Create new token with extended expiry
    let new_claims = super::auth::Claims {
        exp: now + (auth_config.jwt_expiry_hours * 3600),
        iat: now,
        ..claims
    };

    // Generate new JWT
    match jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &new_claims,
        &auth_config.encoding_key(),
    ) {
        Ok(token) => {
            // Update cookie
            let cookie = create_auth_cookie(&token, auth_config.jwt_expiry_hours);
            cookies.add(cookie);

            Json(serde_json::json!({
                "refreshed": true,
                "token": token,
                "expires_at": new_claims.exp
            }))
            .into_response()
        }
        Err(err) => {
            error!("Failed to refresh token: {}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to refresh token").into_response()
        }
    }
}

/// Check session validity (API endpoint)
pub async fn check_session(OptionalAuth(auth): OptionalAuth) -> impl IntoResponse {
    match auth {
        Some(user) => Json(serde_json::json!({
            "authenticated": true,
            "user": UserInfo {
                id: user.claims.sub,
                email: user.claims.email,
                name: user.claims.name,
                roles: user.claims.roles,
            }
        })),
        None => Json(serde_json::json!({
            "authenticated": false
        })),
    }
}

/// Helper: Check if Zitadel is available
async fn check_zitadel_health(zitadel_url: &str) -> bool {
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .ok();

    if let Some(client) = client {
        let health_url = format!("{}/healthz", zitadel_url);
        client.get(&health_url).send().await.is_ok()
    } else {
        false
    }
}

/// Helper: Generate random state for OAuth
fn generate_state() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..32)
        .map(|_| {
            let idx = rng.gen_range(0..62);
            let chars = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
            chars[idx] as char
        })
        .collect()
}

/// Helper: Store session in application state
async fn store_session(state: &AppState, session: &UserSession) {
    // Store in session storage (you can implement Redis or in-memory storage)
    if let Some(sessions) = state
        .extensions
        .get::<std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<String, UserSession>>>>(
        )
    {
        let mut sessions = sessions.write().await;
        sessions.insert(session.id.clone(), session.clone());
    }
}

/// Helper: Remove session from storage
async fn remove_session(state: &AppState, session_id: &str) {
    if let Some(sessions) = state
        .extensions
        .get::<std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<String, UserSession>>>>(
        )
    {
        let mut sessions = sessions.write().await;
        sessions.remove(session_id);
    }
}

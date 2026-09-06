use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use botcore::shared::utils::get_stack_path;
use std::sync::Arc;
use tokio::sync::RwLock;
use once_cell::sync::Lazy;
use std::sync::OnceLock;

use botcore::shared::state::AppState;
use crate::DbPool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionUserData {
    pub user_id: String,
    pub email: String,
    pub username: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub display_name: Option<String>,
    pub organization_id: Option<String>,
    pub roles: Vec<String>,
    pub bucket: Option<String>,
    pub created_at: i64,
}

pub static SESSION_CACHE: Lazy<RwLock<HashMap<String, SessionUserData>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

/// Session lifetime in seconds. Mirrors `expires_in` returned by login;
/// enforced on every `get_current_user` lookup (cache + persisted store) so
/// stale sessions cannot outlive their TTL after a restart.
const SESSION_TTL_SECS: i64 = 3600;

/// Returns `true` when the session has outlived its TTL and must be evicted.
/// `SessionUserData.created_at` is the epoch-seconds login timestamp; sessions
/// carry no per-session TTL, so the global constant applies uniformly.
fn session_expired(user: &SessionUserData) -> bool {
    let now = chrono::Utc::now().timestamp();
    now.saturating_sub(user.created_at) > SESSION_TTL_SECS
}

/// Optional DB pool used to persist suite sessions across restarts. Wired once
/// at bootstrap (main.rs); when present, session creation writes a row to
/// `login_sessions` and logout deletes it, and the auth lookup rehydrates on
/// in-memory cache misses.
static SESSION_POOL: OnceLock<DbPool> = OnceLock::new();

pub fn set_session_pool(pool: DbPool) {
    let _ = SESSION_POOL.set(pool);
}

pub fn persist_session(token: &str, user: &SessionUserData) {
    let Some(pool) = SESSION_POOL.get() else {
        return;
    };
    let Ok(user_json) = serde_json::to_value(user) else {
        return;
    };
    if let Ok(mut conn) = pool.get() {
        use diesel::RunQueryDsl;
        diesel::sql_query(
            "INSERT INTO login_sessions (token, user_data) VALUES ($1, $2::jsonb) \
             ON CONFLICT (token) DO UPDATE SET user_data = EXCLUDED.user_data, created_at = NOW()",
        )
        .bind::<diesel::sql_types::Text, _>(token)
        .bind::<diesel::sql_types::Text, _>(&user_json.to_string())
        .execute(&mut conn)
        .unwrap_or_else(|e| {
            log::warn!("Failed to persist login session: {e}");
            0
        });
    }
}

pub fn remove_persisted_session(token: &str) {
    let Some(pool) = SESSION_POOL.get() else {
        return;
    };
    if let Ok(mut conn) = pool.get() {
        use diesel::RunQueryDsl;
        diesel::sql_query("DELETE FROM login_sessions WHERE token = $1")
            .bind::<diesel::sql_types::Text, _>(token)
            .execute(&mut conn)
            .unwrap_or_else(|e| {
                log::warn!("Failed to remove persisted login session: {e}");
                0
            });
    }
}

/// Rehydrates a session from the `login_sessions` table for in-memory cache
/// misses (e.g. after a restart). Returns `None` when no row exists for the
/// token or the stored payload cannot be parsed.
pub fn session_from_persisted(token: &str) -> Option<SessionUserData> {
    let pool = SESSION_POOL.get()?;
    let mut conn = pool.get().ok()?;
    use diesel::RunQueryDsl;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Text)]
        user_data: String,
    }
    let row: Row = diesel::sql_query(
        "SELECT user_data FROM login_sessions WHERE token = $1 LIMIT 1",
    )
    .bind::<diesel::sql_types::Text, _>(token)
    .get_result(&mut conn)
    .ok()?;
    serde_json::from_str(&row.user_data).ok()
}

const BOOTSTRAP_SECRET_ENV: &str = "GB_BOOTSTRAP_SECRET";

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    pub remember: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub success: bool,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub expires_in: Option<i64>,
    pub requires_2fa: bool,
    pub session_token: Option<String>,
    pub redirect: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CurrentUserResponse {
    pub id: Option<String>,
    pub username: Option<String>,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub display_name: Option<String>,
    pub roles: Option<Vec<String>>,
    pub organization_id: Option<String>,
    pub bucket: Option<String>,
    pub avatar_url: Option<String>,
    pub is_anonymous: bool,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub details: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LogoutResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct TwoFactorRequest {
    pub session_token: String,
    pub code: String,
    pub trust_device: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Debug, Deserialize)]
pub struct BootstrapAdminRequest {
    pub bootstrap_secret: String,
    pub email: String,
    pub username: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub organization_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BootstrapResponse {
    pub success: bool,
    pub message: String,
    pub user_id: Option<String>,
    pub organization_id: Option<String>,
}

pub fn configure() -> Router<Arc<AppState>> {
    Router::new()
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/me", get(get_current_user))
        .route("/refresh", post(refresh_token))
        .route("/2fa/verify", post(verify_2fa))
        .route("/2fa/resend", post(resend_2fa))
        .route("/bootstrap", post(bootstrap_admin))

}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!("Login attempt for: {}", req.email);

    let auth_service = state.auth_service.as_ref().ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: "No auth service".to_string(), details: None })))?.lock().await;

    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| {
            log::error!("Failed to create HTTP client: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Internal server error".to_string(),
                    details: None,
                }),
            )
        })?;

    // Try to get admin token: first PAT file, then OAuth client credentials
    let stack = get_stack_path();
    let pat_path = std::path::PathBuf::from(format!("{}/conf/directory/admin-pat.txt", stack));
    let admin_token = std::fs::read_to_string(pat_path)
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    let admin_token = if admin_token.is_empty() {
        info!("Admin PAT token not found, using OAuth client credentials flow");
        match get_oauth_token(&http_client, &*auth_service).await {
            Ok(token) => Some(token),
            Err(e) => {
                log::warn!("Failed to get OAuth token (will try local auth): {}", e);
                None
            }
        }
    } else {
        Some(admin_token)
    };

    // If we have an admin token, try Zitadel sessions API first
    if let Some(ref admin_token) = admin_token {
        // Zitadel matches `loginName` against a user's login names, which are
        // the username and its org-domain forms. When a demo/user account was
        // created with a bare username (e.g. `sample`), sending the full email
        // (`sample@example.com`) fails. Try the full email first, then fall
        // back to the username prefix.
        let username = req.email.split('@').next().unwrap_or(&req.email).to_string();
        let login_names = [req.email.clone(), username];

        let mut session_response: Option<reqwest::Response> = None;
        let mut session_error: Option<String> = None;

        for login_name in &login_names {
            let session_url = format!("{}/v2/sessions", auth_service.api_url());
            let session_body = serde_json::json!({
                "checks": {
                    "user": {
                        "loginName": login_name
                    },
                    "password": {
                        "password": req.password
                    }
                }
            });

            match http_client
                .post(&session_url)
                .bearer_auth(admin_token)
                .json(&session_body)
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() => {
                    session_response = Some(resp);
                    break;
                }
                Ok(resp) => {
                    let status = resp.status();
                    let err = resp.text().await.unwrap_or_default();
                    log::warn!(
                        "Zitadel sessions API returned {} for loginName '{}': {}",
                        status,
                        login_name,
                        err
                    );
                    session_error = Some(format!("{status} {err}"));
                }
                Err(e) => {
                    log::warn!(
                        "Zitadel sessions API request failed for loginName '{}': {}",
                        login_name,
                        e
                    );
                    session_error = Some(e.to_string());
                }
            }
        }

        if let Some(resp) = session_response {
            let session_data: serde_json::Value = resp.json().await.map_err(|e| {
                log::error!("Failed to parse session response: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: "Invalid response from authentication server".to_string(),
                        details: None,
                    }),
                )
            })?;

            let session_id = session_data
                .get("sessionId")
                .and_then(|s| s.as_str())
                .map(String::from);

            let session_token = session_data
                .get("sessionToken")
                .and_then(|s| s.as_str())
                .map(String::from);

            let user_id_str = session_data
                .get("factors")
                .and_then(|f| f.get("user"))
                .and_then(|u| u.get("userId").or_else(|| u.get("id")))
                .and_then(|v| v.as_str())
                .map(String::from)
                .ok_or_else(|| {
                    log::error!("No user ID in session response for: {}", req.email);
                    (
                        StatusCode::UNAUTHORIZED,
                        Json(ErrorResponse {
                            error: "Invalid email or password".to_string(),
                            details: None,
                        }),
                    )
                })?;

            let api_token = format!("gb_{}_{}", uuid::Uuid::new_v4(), chrono::Utc::now().timestamp());

            let session_user = SessionUserData {
                user_id: user_id_str.clone(),
                email: req.email.clone(),
                username: req.email.split('@').next().unwrap_or("user").to_string(),
                first_name: None,
                last_name: None,
                display_name: Some(req.email.split('@').next().unwrap_or("User").to_string()),
                organization_id: None,
                roles: vec!["admin".to_string()],
                bucket: None,
                created_at: chrono::Utc::now().timestamp(),
            };

            {
                let mut cache = SESSION_CACHE.write().await;
                cache.insert(api_token.clone(), session_user.clone());
                info!("Session cached for user: {}", req.email);
                persist_session(&api_token, &session_user);
            }

            info!("Login successful for: {} (user_id: {})", req.email, user_id_str);

            return Ok(Json(LoginResponse {
                success: true,
                user_id: Some(user_id_str),
                session_id: session_id.clone(),
                access_token: Some(api_token),
                refresh_token: None,
                expires_in: Some(3600),
                requires_2fa: false,
                session_token,
                redirect: Some("/".to_string()),
                message: Some("Login successful".to_string()),
            }));
        } else {
            log::warn!(
                "Zitadel sessions API failed for {}: {} — falling back to local credential check",
                req.email,
                session_error.unwrap_or_default()
            );
        }
    } else {
        log::info!("No admin token available, falling back to local credential check");
    }

    // Zitadel sessions API failed — no fallback; Zitadel is the only auth provider
    log::error!("Zitadel authentication failed for: {}", req.email);
    Err((
        StatusCode::UNAUTHORIZED,
        Json(ErrorResponse {
            error: "Invalid email or password".to_string(),
            details: None,
        }),
    ))
}

pub async fn logout(
    State(_state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<Json<LogoutResponse>, (StatusCode, Json<ErrorResponse>)> {
    let token = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|auth| auth.strip_prefix("Bearer "))
        .map(String::from);

    if let Some(ref token_str) = token {
        let mut cache = SESSION_CACHE.write().await;
        if cache.remove(token_str).is_some() {
            info!("User logged out, session removed from cache");
        } else {
            info!("User logged out (session was not in cache)");
        }
        remove_persisted_session(token_str);
    }

    Ok(Json(LogoutResponse {
        success: true,
        message: "Logged out successfully".to_string(),
    }))
}

pub async fn get_current_user(
    State(_state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Json<CurrentUserResponse> {
    let session_token = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|auth| auth.strip_prefix("Bearer "));

    match session_token {
        None => {
            info!("get_current_user: no authorization header - returning anonymous user");
            Json(CurrentUserResponse {
                id: None,
                username: None,
                email: None,
                first_name: None,
                last_name: None,
                display_name: None,
                roles: None,
                organization_id: None,
                bucket: None,
                avatar_url: None,
                is_anonymous: true,
            })
        }
        Some("") => {
            info!("get_current_user: empty authorization token - returning anonymous user");
            Json(CurrentUserResponse {
                id: None,
                username: None,
                email: None,
                first_name: None,
                last_name: None,
                display_name: None,
                roles: None,
                organization_id: None,
                bucket: None,
                avatar_url: None,
                is_anonymous: true,
            })
        }
        Some(session_token) => {
            info!("get_current_user: looking up session token (len={}, prefix={}...)",
                  session_token.len(),
                  &session_token[..std::cmp::min(20, session_token.len())]);

            let cache = SESSION_CACHE.read().await;
            // Clone the entry so the read guard's borrow ends before any write
            // lock is taken for TTL eviction (E0505: cannot drop while borrowed).
            if let Some(user_data) = cache.get(session_token).cloned() {
                if session_expired(&user_data) {
                    drop(cache);
                    info!("get_current_user: session expired (TTL exceeded) - removing {}", user_data.email);
                    let mut cache = SESSION_CACHE.write().await;
                    cache.remove(session_token);
                    remove_persisted_session(session_token);
                    return Json(CurrentUserResponse {
                        id: None,
                        username: None,
                        email: None,
                        first_name: None,
                        last_name: None,
                        display_name: None,
                        roles: None,
                        organization_id: None,
                        bucket: None,
                        avatar_url: None,
                        is_anonymous: true,
                    });
                }
                info!("get_current_user: found cached session for user: {}", user_data.email);
                return Json(CurrentUserResponse {
                    id: Some(user_data.user_id),
                    username: Some(user_data.username),
                    email: Some(user_data.email),
                    first_name: user_data.first_name,
                    last_name: user_data.last_name,
                    display_name: user_data.display_name,
                    roles: Some(user_data.roles),
                    organization_id: user_data.organization_id,
                    bucket: user_data.bucket,
                    avatar_url: None,
                    is_anonymous: false,
                });
            }
            drop(cache);

            // Cache miss: rehydrate from the persisted `login_sessions` table so
            // sessions survive server restarts instead of silently logging users out.
            if let Some(user_data) = session_from_persisted(session_token) {
                if session_expired(&user_data) {
                    info!("get_current_user: persisted session expired - removing {}", user_data.email);
                    remove_persisted_session(session_token);
                    return Json(CurrentUserResponse {
                        id: None,
                        username: None,
                        email: None,
                        first_name: None,
                        last_name: None,
                        display_name: None,
                        roles: None,
                        organization_id: None,
                        bucket: None,
                        avatar_url: None,
                        is_anonymous: true,
                    });
                }
                info!("get_current_user: rehydrated persisted session for user: {}", user_data.email);
                {
                    let mut cache = SESSION_CACHE.write().await;
                    cache.insert(session_token.to_string(), user_data.clone());
                }
                Json(CurrentUserResponse {
                    id: Some(user_data.user_id.clone()),
                    username: Some(user_data.username.clone()),
                    email: Some(user_data.email.clone()),
                    first_name: user_data.first_name.clone(),
                    last_name: user_data.last_name.clone(),
                    display_name: user_data.display_name.clone(),
                    roles: Some(user_data.roles.clone()),
                    organization_id: user_data.organization_id.clone(),
                    bucket: user_data.bucket.clone(),
                    avatar_url: None,
                    is_anonymous: false,
                })
            } else {
                info!("get_current_user: session not found in cache or persisted store - returning anonymous user");
                Json(CurrentUserResponse {
                    id: None,
                    username: None,
                    email: None,
                    first_name: None,
                    last_name: None,
                    display_name: None,
                    roles: None,
                    organization_id: None,
                    bucket: None,
                    avatar_url: None,
                    is_anonymous: true,
                })
            }
        }
    }
}

pub async fn refresh_token(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RefreshTokenRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<ErrorResponse>)> {
    let auth_service = state.auth_service.as_ref().ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: "No auth service".to_string(), details: None })))?.lock().await;

    let token_url = format!("{}/oauth/v2/token", auth_service.api_url());

    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| {
            log::error!("Failed to create HTTP client: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Internal server error".to_string(),
                    details: None,
                }),
            )
        })?;

    let params = [
        ("grant_type", "refresh_token"),
        ("refresh_token", &req.refresh_token),
        ("scope", "openid profile email offline_access"),
    ];

    let response = http_client
        .post(&token_url)
        .form(&params)
        .send()
        .await
        .map_err(|e| {
            log::error!("Failed to refresh token: {}", e);
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Token refresh failed".to_string(),
                    details: None,
                }),
            )
        })?;

    if !response.status().is_success() {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Invalid or expired refresh token".to_string(),
                details: None,
            }),
        ));
    }

    let token_data: serde_json::Value = response.json().await.map_err(|e| {
        log::error!("Failed to parse token response: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Invalid response from authentication server".to_string(),
                details: None,
            }),
        )
    })?;

    let access_token = token_data
        .get("access_token")
        .and_then(|t| t.as_str())
        .map(String::from);

    let refresh_token = token_data
        .get("refresh_token")
        .and_then(|t| t.as_str())
        .map(String::from);

    let expires_in = token_data.get("expires_in").and_then(|t| t.as_i64());

    Ok(Json(LoginResponse {
        success: true,
        user_id: None,
        session_id: None,
        access_token,
        refresh_token,
        expires_in,
        requires_2fa: false,
        session_token: None,
        redirect: None,
        message: Some("Token refreshed successfully".to_string()),
    }))
}

pub async fn verify_2fa(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<TwoFactorRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!(
        "2FA verification attempt for session: {}",
        req.session_token
    );

    Err((
        StatusCode::NOT_IMPLEMENTED,
        Json(ErrorResponse {
            error: "2FA verification not yet implemented".to_string(),
            details: Some("This feature will be available in a future update".to_string()),
        }),
    ))
}

pub async fn resend_2fa(
    State(_state): State<Arc<AppState>>,
    Json(_req): Json<serde_json::Value>,
) -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(ErrorResponse {
            error: "2FA resend not yet implemented".to_string(),
            details: Some("This feature will be available in a future update".to_string()),
        }),
    )
}

pub async fn bootstrap_admin(
    State(state): State<Arc<AppState>>,
    Json(req): Json<BootstrapAdminRequest>,
) -> Result<Json<BootstrapResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!("Bootstrap admin request received");

    let expected_secret = std::env::var(BOOTSTRAP_SECRET_ENV).unwrap_or_default();

    if expected_secret.is_empty() {
        warn!("Bootstrap endpoint called but GB_BOOTSTRAP_SECRET not set");
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "Bootstrap not enabled".to_string(),
                details: Some("Set GB_BOOTSTRAP_SECRET environment variable to enable bootstrap".to_string()),
            }),
        ));
    }

    if req.bootstrap_secret != expected_secret {
        warn!("Bootstrap attempt with invalid secret");
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Invalid bootstrap secret".to_string(),
                details: None,
            }),
        ));
    }

    let auth_service = state.auth_service.as_ref().ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: "No auth service".to_string(), details: None })))?.lock().await;

    let existing_data = auth_service.list_users(1, 0).await.unwrap_or(serde_json::Value::Null);
    let existing_users = existing_data.as_array();
    if let Some(users_arr) = existing_users {
        if !users_arr.is_empty() {
            let has_admin = users_arr.iter().any(|u| {
                u.get("roles")
                    .and_then(|r| r.as_array())
                    .map(|roles| {
                        roles.iter().any(|r| {
                            r.as_str()
                                .map(|s| s.to_lowercase().contains("admin"))
                                .unwrap_or(false)
                        })
                    })
                    .unwrap_or(false)
            });

            if has_admin {
                return Err((
                    StatusCode::CONFLICT,
                    Json(ErrorResponse {
                        error: "Admin user already exists".to_string(),
                        details: Some("Bootstrap can only be used for initial setup".to_string()),
                    }),
                ));
            }
        }
    }

    let new_user_id = match auth_service
        .create_user(&req.email, &req.first_name, &req.last_name, Some(&req.username))
        .await
    {
        Ok(id) => {
            info!("Bootstrap admin user created: {}", id);
            id
        }
        Err(e) => {
            log::error!("Failed to create bootstrap admin: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to create admin user".to_string(),
                    details: Some(e.to_string()),
                }),
            ));
        }
    };

    if let Err(e) = auth_service.set_user_password(&new_user_id, &req.password).await {
        log::error!("Failed to set admin password: {}", e);
    }

    let org_name = req.organization_name.unwrap_or_else(|| "Default Organization".to_string());
    let new_org_id = match auth_service.create_organization(&org_name).await {
        Ok(id) => {
            info!("Bootstrap organization created: {}", id);
            Some(id)
        }
        Err(e) => {
            warn!("Failed to create organization (may already exist): {}", e);
            None
        }
    };

    if let Some(ref oid) = new_org_id {
        let admin_roles = vec![
            "admin".to_string(),
            "org_owner".to_string(),
            "user_manager".to_string(),
        ];
        if let Err(e) = auth_service.add_org_member(oid, &new_user_id, admin_roles).await {
            log::error!("Failed to add admin to organization: {}", e);
        } else {
            info!("Admin user added to organization with admin roles");
        }

        let system_groups = vec![
            ("everyone", "Everyone", vec!["basic:access", "kb:read:public"]),
            ("admins", "Administrators", vec!["org:manage", "org:members", "org:settings", "org:billing", "bot:*", "kb:*", "app:*", "analytics:*"]),
            ("managers", "Managers", vec!["org:members:view", "bot:create", "bot:edit", "bot:delete", "kb:read", "kb:write", "analytics:view"]),
            ("developers", "Developers", vec!["bot:create", "bot:edit", "kb:write", "app:create"]),
            ("human_resources", "Human Resources", vec!["people:manage", "kb:read", "org:members:view"]),
            ("finance", "Finance", vec!["billing:view", "analytics:view", "kb:read"]),
            ("marketing", "Marketing", vec!["campaigns:manage", "social:post", "analytics:view"]),
            ("support", "Support Team", vec!["bot:view", "kb:read", "tickets:manage"]),
            ("content_managers", "Content Managers", vec!["kb:write", "kb:admin", "docs:manage"]),
            ("sales", "Sales", vec!["crm:view", "crm:manage", "analytics:view"]),
            ("viewers", "Viewers (Read-Only)", vec!["bot:view", "kb:read", "app:view"]),
            ("integration_services", "Integration Services", vec!["webhooks:manage", "api:access", "sources:connect"]),
        ];

        for (group_name, display_name, perms) in &system_groups {
            let metadata_key = format!("group_{}", group_name);
            let metadata_value = serde_json::json!({
                "name": display_name,
                "description": format!("Auto-provisioned {} group", display_name),
                "permissions": perms,
                "system": true,
                "organization_id": oid
            }).to_string();

            let body = serde_json::json!({
                "key": metadata_key,
                "value": metadata_value
            });

            if let Err(e) = auth_service.http_post(
                format!("{}/metadata/organization", auth_service.api_url()),
                body,
            ).await {
                log::warn!("Failed to provision group {}: {}", group_name, e);
            } else {
                info!("Provisioned system group: {}", group_name);
            }
        }

        info!("Auto-provisioned 12 system groups for organization {}", oid);
    }

    info!(
        "Bootstrap complete: admin user {} created successfully",
        req.username
    );

    Ok(Json(BootstrapResponse {
        success: true,
        message: format!(
            "Admin user '{}' created successfully. You can now login with your credentials.",
            req.username
        ),
        user_id: Some(new_user_id),
        organization_id: new_org_id,
    }))
}

async fn get_oauth_token(
    http_client: &reqwest::Client,
    auth_service: &dyn botlib::traits::AuthServiceTrait,
) -> Result<String, String> {
    let token_url = format!("{}/oauth/v2/token", auth_service.api_url());

    let params = [
        ("grant_type", "client_credentials".to_string()),
        ("client_id", auth_service.client_id().to_string()),
        ("client_secret", auth_service.client_secret().to_string()),
        ("scope", "openid profile email urn:zitadel:iam:org:project:id:zitadel:aud".to_string()),
    ];

    let response = http_client
        .post(&token_url)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Failed to request OAuth token: {}", e))?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!("OAuth token request failed: {}", error_text));
    }

    let token_data: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse OAuth token response: {}", e))?;

    let access_token = token_data
        .get("access_token")
        .and_then(|t| t.as_str())
        .ok_or_else(|| "No access_token in OAuth response".to_string())?
        .to_string();

    info!("Successfully obtained OAuth access token via client credentials");
    Ok(access_token)
}



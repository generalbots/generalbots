use crate::{OAuthProvider, OAuthState, OAuthUserInfo};
use crate::providers::{get_enabled_providers, load_oauth_config};
use anyhow::anyhow;
use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
    Json, Router,
};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

pub struct OAuthState_ {
    pub conn: DbPool,
    pub base_url: String,
}

#[derive(Debug, Deserialize)]
pub struct OAuthStartParams {
    pub redirect: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OAuthCallbackParams {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct EnabledProvidersResponse {
    pub providers: Vec<ProviderInfo>,
}

#[derive(Debug, Serialize)]
pub struct ProviderInfo {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub login_url: String,
}

pub fn configure(state: Arc<OAuthState_>) -> Router {
    Router::new()
        .route("/auth/oauth/providers", get(list_providers))
        .route("/auth/oauth/:provider", get(start_oauth))
        .route("/auth/oauth/:provider/callback", get(oauth_callback))
        .with_state(state)
}

async fn list_providers(State(state): State<Arc<OAuthState_>>) -> impl IntoResponse {
    let bot_config = get_bot_config(&state).await;
    let base_url = &state.base_url;

    let enabled = get_enabled_providers(&bot_config, base_url);

    let providers: Vec<ProviderInfo> = enabled
        .iter()
        .map(|config| ProviderInfo {
            id: config.provider.to_string().to_lowercase(),
            name: config.provider.display_name().to_string(),
            icon: config.provider.icon().to_string(),
            login_url: format!("/auth/oauth/{}", config.provider.to_string().to_lowercase()),
        })
        .collect();

    Json(EnabledProvidersResponse { providers })
}

async fn start_oauth(
    State(state): State<Arc<OAuthState_>>,
    Path(provider_name): Path<String>,
    Query(params): Query<OAuthStartParams>,
) -> Response {
    let Some(provider) = OAuthProvider::parse(&provider_name) else {
        return (
            StatusCode::BAD_REQUEST,
            Html(format!(
                r#"<!DOCTYPE html><html><head><title>Error</title></head><body>
                <h1>Invalid OAuth Provider</h1><p>Provider '{}' is not supported.</p>
                <a href="/auth/login">Back to Login</a></body></html>"#,
                provider_name
            )),
        )
            .into_response();
    };

    let bot_config = get_bot_config(&state).await;
    let base_url = &state.base_url;

    let config = match load_oauth_config(provider, &bot_config, base_url) {
        Some(c) if c.is_valid() => c,
        _ => {
            warn!("OAuth provider {} is not configured or enabled", provider);
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Html(format!(
                    r#"<!DOCTYPE html><html><head><title>Error</title></head><body>
                    <h1>OAuth Provider Not Configured</h1><p>Login with {} is not currently enabled.</p>
                    <a href="/auth/login">Back to Login</a></body></html>"#,
                    provider.display_name()
                )),
            )
                .into_response();
        }
    };

    let oauth_state = OAuthState::new(provider, params.redirect);
    let state_encoded = oauth_state.encode();

    debug!("OAuth state created for provider {}", provider);

    let auth_url = provider.build_auth_url(&config, &state_encoded);

    info!("Starting OAuth flow for {} - redirecting to provider", provider);

    Redirect::temporary(&auth_url).into_response()
}

async fn oauth_callback(
    State(state): State<Arc<OAuthState_>>,
    Path(provider_name): Path<String>,
    Query(params): Query<OAuthCallbackParams>,
) -> Response {
    if let Some(error) = &params.error {
        let description = params.error_description.as_deref().unwrap_or("Unknown error");
        warn!("OAuth error from provider: {} - {}", error, description);
        return (
            StatusCode::UNAUTHORIZED,
            Html(format!(
                r#"<!DOCTYPE html><html><head><title>Login Failed</title></head><body>
                <h1>Login Failed</h1><p>The OAuth provider returned an error: {}</p><p>{}</p>
                <a href="/auth/login">Try Again</a></body></html>"#,
                error, description
            )),
        )
            .into_response();
    }

    let Some(code) = &params.code else {
        return (
            StatusCode::BAD_REQUEST,
            Html(r#"<!DOCTYPE html><html><head><title>Error</title></head><body>
            <h1>Missing Authorization Code</h1><a href="/auth/login">Try Again</a></body></html>"#.to_string()),
        )
            .into_response();
    };

    let Some(state_param) = &params.state else {
        return (
            StatusCode::BAD_REQUEST,
            Html(r#"<!DOCTYPE html><html><head><title>Error</title></head><body>
            <h1>Missing State Parameter</h1><a href="/auth/login">Try Again</a></body></html>"#.to_string()),
        )
            .into_response();
    };

    let Some(oauth_state) = OAuthState::decode(state_param) else {
        warn!("Failed to decode OAuth state parameter");
        return (
            StatusCode::BAD_REQUEST,
            Html(r#"<!DOCTYPE html><html><head><title>Error</title></head><body>
            <h1>Invalid State</h1><a href="/auth/login">Try Again</a></body></html>"#.to_string()),
        )
            .into_response();
    };

    if oauth_state.is_expired() {
        warn!("OAuth state expired");
        return (
            StatusCode::BAD_REQUEST,
            Html(r#"<!DOCTYPE html><html><head><title>Session Expired</title></head><body>
            <h1>Session Expired</h1><a href="/auth/login">Try Again</a></body></html>"#.to_string()),
        )
            .into_response();
    }

    let Some(provider) = OAuthProvider::parse(&provider_name) else {
        return (StatusCode::BAD_REQUEST, Html("Invalid provider".to_string())).into_response();
    };

    if provider != oauth_state.provider {
        warn!("Provider mismatch: URL says {}, state says {}", provider, oauth_state.provider);
        return (
            StatusCode::BAD_REQUEST,
            Html(r#"<!DOCTYPE html><html><head><title>Error</title></head><body>
            <h1>Provider Mismatch</h1><a href="/auth/login">Try Again</a></body></html>"#.to_string()),
        )
            .into_response();
    }

    let bot_config = get_bot_config(&state).await;
    let base_url = &state.base_url;

    let Some(config) = load_oauth_config(provider, &bot_config, base_url) else {
        return (StatusCode::SERVICE_UNAVAILABLE, Html("OAuth provider not configured".to_string())).into_response();
    };

    let http_client = reqwest::Client::new();
    let token = match provider.exchange_code(&config, code, &http_client).await {
        Ok(t) => t,
        Err(e) => {
            error!("Failed to exchange OAuth code: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR,
                Html(format!(r#"<!DOCTYPE html><html><head><title>Login Failed</title></head><body>
                <h1>Login Failed</h1><p>Failed to complete the OAuth login: {}</p>
                <a href="/auth/login">Try Again</a></body></html>"#, e)),
            ).into_response();
        }
    };

    let user_info = match provider.fetch_user_info(&token.access_token, &http_client).await {
        Ok(info) => info,
        Err(e) => {
            error!("Failed to fetch user info: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR,
                Html(format!(r#"<!DOCTYPE html><html><head><title>Login Failed</title></head><body>
                <h1>Login Failed</h1><p>Failed to retrieve user information: {}</p>
                <a href="/auth/login">Try Again</a></body></html>"#, e)),
            ).into_response();
        }
    };

    info!(
        "OAuth login successful for {} user: {} ({})",
        provider,
        user_info.name.as_deref().unwrap_or("unknown"),
        user_info.email.as_deref().unwrap_or("no email")
    );

    let user_id = match create_or_get_oauth_user(&state, &user_info).await {
        Ok(id) => id,
        Err(e) => {
            error!("Failed to create/get user: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Html("Failed to create user account".to_string())).into_response();
        }
    };

    let session_token = match create_user_session(&state, user_id).await {
        Ok(token) => token,
        Err(e) => {
            error!("Failed to create session: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Html("Failed to create session".to_string())).into_response();
        }
    };

    let redirect_url = oauth_state.redirect_after.unwrap_or_else(|| "/".to_string());

    debug!("OAuth complete, redirecting to {} with session {}", redirect_url, session_token);

    Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(header::LOCATION, redirect_url.clone())
        .header(
            header::SET_COOKIE,
            format!("session={}; Path=/; HttpOnly; SameSite=Lax; Max-Age=86400", session_token),
        )
        .body(axum::body::Body::empty())
        .unwrap_or_else(|e| {
            log::error!("Failed to build OAuth redirect response: {}", e);
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(axum::body::Body::empty())
                .unwrap_or_default()
        })
}

async fn get_bot_config(state: &OAuthState_) -> HashMap<String, String> {
    let conn = state.conn.clone();
    match tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().ok()?;
        use diesel::prelude::*;
        use crate::schema::{bots, bot_configuration};

        let bot_result: Option<Uuid> = bots::dsl::bots
            .filter(bots::dsl::is_active.eq(true))
            .select(bots::dsl::id)
            .first(&mut db_conn)
            .optional()
            .ok()?;

        let active_bot_id = bot_result?;

        let configs: Vec<(String, String)> = bot_configuration::dsl::bot_configuration
            .filter(bot_configuration::dsl::bot_id.eq(active_bot_id))
            .select((bot_configuration::dsl::config_key, bot_configuration::dsl::config_value))
            .load(&mut db_conn)
            .ok()?;

        Some(configs.into_iter().collect::<HashMap<_, _>>())
    })
    .await
    {
        Ok(Some(config)) => config,
        _ => HashMap::new(),
    }
}

async fn create_or_get_oauth_user(
    state: &OAuthState_,
    user_info: &OAuthUserInfo,
) -> anyhow::Result<Uuid> {
    let conn = state.conn.clone();
    let provider_id = user_info.provider_id.clone();
    let provider = user_info.provider.to_string().to_lowercase();
    let user_email = user_info.email.clone();
    let display_name = user_info.name.clone().unwrap_or_else(|| "OAuth User".to_string());

    tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| anyhow!("DB connection error: {}", e))?;

        use diesel::prelude::*;
        use crate::schema::users;

        let existing_user: Option<Uuid> = if let Some(ref email_addr) = user_email {
            users::dsl::users
                .filter(users::dsl::email.eq(email_addr))
                .select(users::dsl::id)
                .first(&mut db_conn)
                .optional()
                .map_err(|e| anyhow!("DB error: {}", e))?
        } else {
            let oauth_username = format!("{}_{}", provider, provider_id);
            users::dsl::users
                .filter(users::dsl::username.eq(&oauth_username))
                .select(users::dsl::id)
                .first(&mut db_conn)
                .optional()
                .map_err(|e| anyhow!("DB error: {}", e))?
        };

        if let Some(user_id) = existing_user {
            return Ok(user_id);
        }

        let new_user_id = Uuid::new_v4();

        let sanitized_name: String = display_name
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
            .take(20)
            .collect();

        let oauth_username = if sanitized_name.is_empty() {
            format!("{}_{}", provider, &provider_id[..8.min(provider_id.len())])
        } else {
            format!("{}_{}", sanitized_name, &provider_id[..6.min(provider_id.len())])
        };

        let user_email_value = user_email.unwrap_or_else(|| format!("{}@oauth.local", oauth_username));

        diesel::insert_into(users::dsl::users)
            .values((
                users::dsl::id.eq(new_user_id),
                users::dsl::username.eq(&oauth_username),
                users::dsl::email.eq(&user_email_value),
                users::dsl::password_hash.eq("OAUTH_USER_NO_PASSWORD"),
                users::dsl::is_active.eq(true),
                users::dsl::is_admin.eq(false),
                users::dsl::created_at.eq(diesel::dsl::now),
                users::dsl::updated_at.eq(diesel::dsl::now),
            ))
            .execute(&mut db_conn)
            .map_err(|e| anyhow!("Failed to create user: {}", e))?;

        debug!("Created OAuth user: {} ({}) for provider {}", oauth_username, user_email_value, provider);

        Ok(new_user_id)
    })
    .await
    .map_err(|e| anyhow!("Task error: {}", e))?
}

async fn create_user_session(state: &OAuthState_, user_id: Uuid) -> anyhow::Result<String> {
    let conn = state.conn.clone();
    let bot_id = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().ok()?;
        use diesel::prelude::*;
        use crate::schema::bots;
        bots::dsl::bots
            .filter(bots::dsl::is_active.eq(true))
            .select(bots::dsl::id)
            .first::<Uuid>(&mut db_conn)
            .optional()
            .ok()?
    })
    .await
    .ok()
    .flatten()
    .unwrap_or(Uuid::nil());

    let session_id = Uuid::new_v4();

    let conn = state.conn.clone();
    tokio::task::spawn_blocking(move || {
        use diesel::prelude::*;
        use crate::schema::user_sessions;
        diesel::insert_into(user_sessions::dsl::user_sessions)
            .values((
                user_sessions::dsl::id.eq(session_id),
                user_sessions::dsl::user_id.eq(user_id),
                user_sessions::dsl::bot_id.eq(bot_id),
                user_sessions::dsl::started_at.eq(chrono::Utc::now()),
                user_sessions::dsl::is_active.eq(true),
            ))
            .execute(&mut conn.get().map_err(|e| anyhow!("DB error: {}", e))?)
            .map_err(|e| anyhow!("Failed to create session: {}", e))?;
        Ok::<(), anyhow::Error>(())
    })
    .await
    .map_err(|e| anyhow!("Task error: {}", e))??;

    Ok(session_id.to_string())
}



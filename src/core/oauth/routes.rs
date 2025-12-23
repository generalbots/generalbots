






use crate::shared::state::AppState;
use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
    Json, Router,
};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use super::{
    providers::{get_enabled_providers, load_oauth_config},
    OAuthProvider, OAuthState, OAuthUserInfo,
};


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


pub fn configure() -> Router<Arc<AppState>> {
    Router::new()
        .route("/auth/oauth/providers", get(list_providers))
        .route("/auth/oauth/{provider}", get(start_oauth))
        .route("/auth/oauth/{provider}/callback", get(oauth_callback))
}


async fn list_providers(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let bot_config = get_bot_config(&state).await;
    let base_url = get_base_url(&state);

    let enabled = get_enabled_providers(&bot_config, &base_url);

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
    State(state): State<Arc<AppState>>,
    Path(provider_name): Path<String>,
    Query(params): Query<OAuthStartParams>,
) -> Response {

    let provider = match OAuthProvider::from_str(&provider_name) {
        Some(p) => p,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Html(format!(
                    r#"<!DOCTYPE html>
<html>
<head><title>Error</title></head>
<body>
    <h1>Invalid OAuth Provider</h1>
    <p>Provider '{}' is not supported.</p>
    <p>Supported providers: Google, Discord, Reddit, Twitter, Microsoft, Facebook</p>
    <a href="/auth/login">Back to Login</a>
</body>
</html>"#,
                    provider_name
                )),
            )
                .into_response();
        }
    };


    let bot_config = get_bot_config(&state).await;
    let base_url = get_base_url(&state);

    let config = match load_oauth_config(provider, &bot_config, &base_url) {
        Some(c) if c.is_valid() => c,
        _ => {
            warn!("OAuth provider {} is not configured or enabled", provider);
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Html(format!(
                    r#"<!DOCTYPE html>
<html>
<head><title>Error</title></head>
<body>
    <h1>OAuth Provider Not Configured</h1>
    <p>Login with {} is not currently enabled.</p>
    <p>Please contact your administrator to configure OAuth credentials.</p>
    <a href="/auth/login">Back to Login</a>
</body>
</html>"#,
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

    info!(
        "Starting OAuth flow for {} - redirecting to provider",
        provider
    );

    Redirect::temporary(&auth_url).into_response()
}


async fn oauth_callback(
    State(state): State<Arc<AppState>>,
    Path(provider_name): Path<String>,
    Query(params): Query<OAuthCallbackParams>,
) -> Response {

    if let Some(error) = &params.error {
        let description = params
            .error_description
            .as_deref()
            .unwrap_or("Unknown error");
        warn!("OAuth error from provider: {} - {}", error, description);
        return (
            StatusCode::UNAUTHORIZED,
            Html(format!(
                r#"<!DOCTYPE html>
<html>
<head><title>Login Failed</title></head>
<body>
    <h1>Login Failed</h1>
    <p>The OAuth provider returned an error: {}</p>
    <p>{}</p>
    <a href="/auth/login">Try Again</a>
</body>
</html>"#,
                error, description
            )),
        )
            .into_response();
    }


    let code = match &params.code {
        Some(c) => c,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Html(
                    r#"<!DOCTYPE html>
<html>
<head><title>Error</title></head>
<body>
    <h1>Missing Authorization Code</h1>
    <p>The OAuth callback did not include an authorization code.</p>
    <a href="/auth/login">Try Again</a>
</body>
</html>"#
                        .to_string(),
                ),
            )
                .into_response();
        }
    };

    let state_param = match &params.state {
        Some(s) => s,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Html(
                    r#"<!DOCTYPE html>
<html>
<head><title>Error</title></head>
<body>
    <h1>Missing State Parameter</h1>
    <p>The OAuth callback did not include a state parameter.</p>
    <a href="/auth/login">Try Again</a>
</body>
</html>"#
                        .to_string(),
                ),
            )
                .into_response();
        }
    };


    let oauth_state = match OAuthState::decode(state_param) {
        Some(s) => s,
        None => {
            warn!("Failed to decode OAuth state parameter");
            return (
                StatusCode::BAD_REQUEST,
                Html(
                    r#"<!DOCTYPE html>
<html>
<head><title>Error</title></head>
<body>
    <h1>Invalid State</h1>
    <p>The OAuth state parameter could not be validated.</p>
    <a href="/auth/login">Try Again</a>
</body>
</html>"#
                        .to_string(),
                ),
            )
                .into_response();
        }
    };


    if oauth_state.is_expired() {
        warn!("OAuth state expired");
        return (
            StatusCode::BAD_REQUEST,
            Html(
                r#"<!DOCTYPE html>
<html>
<head><title>Session Expired</title></head>
<body>
    <h1>Session Expired</h1>
    <p>The login session has expired. Please try again.</p>
    <a href="/auth/login">Try Again</a>
</body>
</html>"#
                    .to_string(),
            ),
        )
            .into_response();
    }


    let provider = match OAuthProvider::from_str(&provider_name) {
        Some(p) => p,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Html("Invalid provider".to_string()),
            )
                .into_response();
        }
    };


    if provider != oauth_state.provider {
        warn!(
            "Provider mismatch: URL says {}, state says {}",
            provider, oauth_state.provider
        );
        return (
            StatusCode::BAD_REQUEST,
            Html(
                r#"<!DOCTYPE html>
<html>
<head><title>Error</title></head>
<body>
    <h1>Provider Mismatch</h1>
    <p>The OAuth callback doesn't match the expected provider.</p>
    <a href="/auth/login">Try Again</a>
</body>
</html>"#
                    .to_string(),
            ),
        )
            .into_response();
    }


    let bot_config = get_bot_config(&state).await;
    let base_url = get_base_url(&state);

    let config = match load_oauth_config(provider, &bot_config, &base_url) {
        Some(c) => c,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Html("OAuth provider not configured".to_string()),
            )
                .into_response();
        }
    };


    let http_client = reqwest::Client::new();
    let token = match provider.exchange_code(&config, code, &http_client).await {
        Ok(t) => t,
        Err(e) => {
            error!("Failed to exchange OAuth code: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(format!(
                    r#"<!DOCTYPE html>
<html>
<head><title>Login Failed</title></head>
<body>
    <h1>Login Failed</h1>
    <p>Failed to complete the OAuth login: {}</p>
    <a href="/auth/login">Try Again</a>
</body>
</html>"#,
                    e
                )),
            )
                .into_response();
        }
    };


    let user_info = match provider
        .fetch_user_info(&token.access_token, &http_client)
        .await
    {
        Ok(info) => info,
        Err(e) => {
            error!("Failed to fetch user info: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(format!(
                    r#"<!DOCTYPE html>
<html>
<head><title>Login Failed</title></head>
<body>
    <h1>Login Failed</h1>
    <p>Failed to retrieve user information: {}</p>
    <a href="/auth/login">Try Again</a>
</body>
</html>"#,
                    e
                )),
            )
                .into_response();
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
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html("Failed to create user account".to_string()),
            )
                .into_response();
        }
    };


    let session_token = match create_user_session(&state, user_id).await {
        Ok(token) => token,
        Err(e) => {
            error!("Failed to create session: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html("Failed to create session".to_string()),
            )
                .into_response();
        }
    };


    let redirect_url = oauth_state
        .redirect_after
        .unwrap_or_else(|| "/".to_string());

    debug!(
        "OAuth complete, redirecting to {} with session {}",
        redirect_url, session_token
    );


    Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(header::LOCATION, redirect_url)
        .header(
            header::SET_COOKIE,
            format!(
                "session={}; Path=/; HttpOnly; SameSite=Lax; Max-Age=86400",
                session_token
            ),
        )
        .body(axum::body::Body::empty())
        .unwrap()
}


async fn get_bot_config(state: &AppState) -> HashMap<String, String> {

    let conn = state.conn.clone();

    match tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().ok()?;

        use diesel::prelude::*;

        let bot_result: Option<Uuid> = {
            use crate::shared::models::schema::bots::dsl as bots_dsl;
            bots_dsl::bots
                .filter(bots_dsl::is_active.eq(true))
                .select(bots_dsl::id)
                .first(&mut db_conn)
                .optional()
                .ok()?
        };

        let active_bot_id = bot_result?;

        let configs: Vec<(String, String)> = {
            use crate::shared::models::schema::bot_configuration::dsl as cfg_dsl;
            cfg_dsl::bot_configuration
                .filter(cfg_dsl::bot_id.eq(active_bot_id))
                .select((cfg_dsl::config_key, cfg_dsl::config_value))
                .load(&mut db_conn)
                .ok()?
        };

        Some(configs.into_iter().collect::<HashMap<_, _>>())
    })
    .await
    {
        Ok(Some(config)) => config,
        _ => HashMap::new(),
    }
}


fn get_base_url(state: &AppState) -> String {

    let _ = state;
    "http://localhost:8300".to_string()
}


async fn create_or_get_oauth_user(
    state: &AppState,
    user_info: &OAuthUserInfo,
) -> anyhow::Result<Uuid> {
    let conn = state.conn.clone();
    let provider_id = user_info.provider_id.clone();
    let provider = user_info.provider.to_string().to_lowercase();
    let user_email = user_info.email.clone();
    let display_name = user_info
        .name
        .clone()
        .unwrap_or_else(|| "OAuth User".to_string());

    tokio::task::spawn_blocking(move || {
        let mut db_conn = conn
            .get()
            .map_err(|e| anyhow::anyhow!("DB connection error: {}", e))?;

        use crate::shared::models::schema::users::dsl::*;
        use diesel::prelude::*;


        let existing_user: Option<Uuid> = if let Some(ref email_addr) = user_email {
            users
                .filter(email.eq(email_addr))
                .select(id)
                .first(&mut db_conn)
                .optional()
                .map_err(|e| anyhow::anyhow!("DB error: {}", e))?
        } else {

            let oauth_username = format!("{}_{}", provider, provider_id);
            users
                .filter(username.eq(&oauth_username))
                .select(id)
                .first(&mut db_conn)
                .optional()
                .map_err(|e| anyhow::anyhow!("DB error: {}", e))?
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
            format!(
                "{}_{}",
                sanitized_name,
                &provider_id[..6.min(provider_id.len())]
            )
        };
        let user_email_value =
            user_email.unwrap_or_else(|| format!("{}@oauth.local", oauth_username));

        diesel::insert_into(users)
            .values((
                id.eq(new_user_id),
                username.eq(&oauth_username),
                email.eq(&user_email_value),
                password_hash.eq("OAUTH_USER_NO_PASSWORD"),
                is_active.eq(true),
                is_admin.eq(false),
                created_at.eq(diesel::dsl::now),
                updated_at.eq(diesel::dsl::now),
            ))
            .execute(&mut db_conn)
            .map_err(|e| anyhow::anyhow!("Failed to create user: {}", e))?;

        debug!(
            "Created OAuth user: {} ({}) for provider {}",
            oauth_username, user_email_value, provider
        );

        Ok(new_user_id)
    })
    .await
    .map_err(|e| anyhow::anyhow!("Task error: {}", e))?
}


async fn create_user_session(state: &AppState, user_id: Uuid) -> anyhow::Result<String> {
    let mut sm = state.session_manager.lock().await;


    let bot_id = {
        let conn = state.conn.clone();
        tokio::task::spawn_blocking(move || {
            let mut db_conn = conn.get().ok()?;
            use crate::shared::models::schema::bots::dsl::*;
            use diesel::prelude::*;

            bots.filter(is_active.eq(true))
                .select(id)
                .first::<Uuid>(&mut db_conn)
                .optional()
                .ok()?
        })
        .await
        .ok()
        .flatten()
        .unwrap_or(Uuid::nil())
    };

    let session = sm
        .get_or_create_user_session(user_id, bot_id, "OAuth Login")
        .map_err(|e| anyhow::anyhow!("Session error: {}", e))?
        .ok_or_else(|| anyhow::anyhow!("Failed to create session"))?;

    Ok(session.id.to_string())
}

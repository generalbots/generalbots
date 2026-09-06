//! OAuth account linking for the Settings page (#899).
//!
//! Implements the authorization-code + PKCE flow for Google, Microsoft and
//! GitHub and binds the resulting external identity to the authenticated
//! local user. Provider configuration is read from the bot config in Vault
//! (same keys the login flow uses); linked identities are stored in
//! `oauth_account_links` and the access/refresh tokens are persisted in
//! Vault under `secret/gbo/oauth/{user_id}/{provider}` — never in the DB or
//! logs.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    Json,
};
use diesel::RunQueryDsl;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;

use botcore::shared::state::AppState;
use botcoreoauth::{OAuthConfig, OAuthProvider, OAuthState};

use crate::audit_log::record_audit_event;
use crate::settings_api::{get_conn, resolve_user_id};

/// Providers the Settings page offers for account linking.
const LINKABLE_PROVIDERS: &[&str] = &["google", "microsoft", "github"];

/// Reads the OAuth provider configuration from the bot config in Vault.
/// The redirect URI is forced to the Settings callback so the flow binds to
/// the authenticated user instead of creating a fresh login session.
fn load_link_config(
    state: &Arc<AppState>,
    provider: OAuthProvider,
    base_url: &str,
) -> Option<OAuthConfig> {
    let bot_config = read_oauth_bot_config(state);
    let mut config = botcoreoauth::load_oauth_config(provider, &bot_config, base_url)?;
    config.redirect_uri = format!(
        "{}/api/oauth/{}/callback",
        base_url,
        provider.to_string().to_lowercase()
    );
    Some(config)
}

/// Reads the bot-level OAuth config keys from Vault
/// (`secret/gbo/{org}/{branch}/{bot}`) — the same lookup the login flow
/// uses. Returns an empty map when Vault is unavailable.
fn read_oauth_bot_config(state: &Arc<AppState>) -> HashMap<String, String> {
    let manager = match botcoresecrets::SecretsManager::get_clone() {
        Ok(m) => m,
        Err(_) => return HashMap::new(),
    };
    if !manager.is_enabled() {
        return HashMap::new();
    }
    // The bot config is scoped per active bot; the login flow uses the nil
    // org/branch with the active bot id. Reuse the same path resolution.
    let active_bot = active_bot_id(state);
    let path = format!(
        "gbo/{}/{}/{}",
        uuid::Uuid::nil(),
        uuid::Uuid::nil(),
        active_bot
    );
    let manager = manager.clone();
    let path_clone = path.clone();
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build();
        let result = if let Ok(rt) = rt {
            rt.block_on(manager.get_secret(&path_clone)).ok()
        } else {
            None
        };
        let _ = tx.send(result);
    });
    rx.recv().ok().flatten().unwrap_or_default()
}

fn active_bot_id(state: &Arc<AppState>) -> uuid::Uuid {
    use diesel::prelude::*;
    let Ok(mut conn) = state.conn.get() else {
        return uuid::Uuid::nil();
    };
    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct IdRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: uuid::Uuid,
    }
    diesel::sql_query("SELECT id FROM bots WHERE is_active = true LIMIT 1")
        .get_result::<IdRow>(&mut conn)
        .map(|r| r.id)
        .unwrap_or_else(|_| uuid::Uuid::nil())
}

/// Base URL used to build the redirect URI and callback links. Prefers the
/// `PUBLIC_BASE_URL` env var (deploy-time value) and falls back to the
/// local suite origin.
fn base_url() -> String {
    std::env::var("PUBLIC_BASE_URL")
        .unwrap_or_else(|_| "http://localhost:3000".to_string())
}

/// POST /api/oauth/{provider}/connect
///
/// Starts the account-linking flow: builds an OAuth state bound to the
/// authenticated user (with a fresh PKCE verifier) and redirects to the
/// provider's authorization endpoint.
pub async fn oauth_connect(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(provider_name): Path<String>,
) -> Response {
    let provider_name_lower = provider_name.to_lowercase();
    let Some(provider) = OAuthProvider::parse(&provider_name_lower) else {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": format!("Unsupported OAuth provider: {provider_name}")
            })),
        )
            .into_response();
    };
    if !LINKABLE_PROVIDERS.contains(&provider_name_lower.as_str()) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": format!("Provider {provider_name} is not supported for account linking")
            })),
        )
            .into_response();
    }

    let user_id = match resolve_user_id(&state, &headers) {
        Ok(id) => id,
        Err(e) => return e.into_response(),
    };

    let Some(config) = load_link_config(&state, provider, &base_url()) else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "error": format!(
                    "{} is not configured for account linking — set the provider keys in the bot config.",
                    provider.display_name()
                )
            })),
        )
            .into_response();
    };

    let oauth_state = OAuthState::new(provider, None)
        .for_user(&user_id.to_string())
        .with_pkce();
    let state_encoded = oauth_state.encode();
    let auth_url = provider.build_auth_url(&config, &state_encoded, oauth_state.pkce_verifier.as_deref());

    log::info!("Starting OAuth account link for user {user_id} via {provider}");
    // JSON response: the frontend performs the top-level navigation so the
    // browser lands on the provider's authorization page with the token in
    // hand (HTMX cannot carry the bearer header on a plain redirect).
    Json(serde_json::json!({ "redirect_url": auth_url })).into_response()
}

/// GET /api/oauth/{provider}/callback
///
/// Completes the linking flow: exchanges the code (presenting the PKCE
/// verifier), verifies the bound user, stores the identity + tokens, and
/// redirects back to the Settings page.
pub async fn oauth_callback(
    State(state): State<Arc<AppState>>,
    Path(provider_name): Path<String>,
    Query(params): Query<CallbackParams>,
) -> Response {
    if let Some(error) = &params.error {
        let description = params.error_description.as_deref().unwrap_or("Unknown error");
        log::warn!("OAuth callback error from {provider_name}: {error} — {description}");
        return redirect_to_settings("error=oauth_denied");
    }

    let provider_name_lower = provider_name.to_lowercase();
    let Some(provider) = OAuthProvider::parse(&provider_name_lower) else {
        return redirect_to_settings("error=invalid_provider");
    };
    let Some(code) = &params.code else {
        return redirect_to_settings("error=missing_code");
    };
    let Some(state_param) = &params.state else {
        return redirect_to_settings("error=missing_state");
    };
    let Some(oauth_state) = OAuthState::decode(state_param) else {
        log::warn!("Failed to decode OAuth state parameter");
        return redirect_to_settings("error=invalid_state");
    };
    if oauth_state.is_expired() {
        log::warn!("OAuth state expired for provider {provider}");
        return redirect_to_settings("error=state_expired");
    }
    if oauth_state.provider != provider {
        log::warn!(
            "OAuth provider mismatch: URL {provider}, state {}",
            oauth_state.provider
        );
        return redirect_to_settings("error=provider_mismatch");
    }
    let Some(bound_user) = &oauth_state.user_id else {
        log::warn!("OAuth callback without a bound user (login flow used linking callback)");
        return redirect_to_settings("error=not_linked");
    };
    let Ok(user_id) = uuid::Uuid::parse_str(bound_user) else {
        log::warn!("OAuth callback with invalid bound user id");
        return redirect_to_settings("error=invalid_user");
    };

    let Some(config) = load_link_config(&state, provider, &base_url()) else {
        return redirect_to_settings("error=not_configured");
    };

    let http_client = reqwest::Client::new();
    let token = match provider
        .exchange_code(&config, code, &http_client, oauth_state.pkce_verifier.as_deref())
        .await
    {
        Ok(t) => t,
        Err(e) => {
            log::error!("Failed to exchange OAuth code for {provider}: {e}");
            return redirect_to_settings("error=exchange_failed");
        }
    };

    let user_info = match provider.fetch_user_info(&token.access_token, &http_client).await {
        Ok(info) => info,
        Err(e) => {
            log::error!("Failed to fetch OAuth user info for {provider}: {e}");
            return redirect_to_settings("error=userinfo_failed");
        }
    };
    if user_info.provider_id.is_empty() {
        log::error!("OAuth provider returned no identity for {provider}");
        return redirect_to_settings("error=empty_identity");
    }

    // Persist tokens in Vault (per user), never in the DB.
    if let Err(e) = store_tokens_in_vault(&user_id, provider, &token) {
        log::error!("Failed to store OAuth tokens in Vault for user {user_id}: {e}");
        return redirect_to_settings("error=token_storage_failed");
    }

    // Upsert the link row.
    match upsert_account_link(
        &state,
        user_id,
        provider,
        &user_info,
    ) {
        Ok(()) => {
            record_audit_event(
                &state,
                "oauth.link",
                user_id,
                "oauth.link",
                Some("oauth_account_links"),
                None,
                true,
                Some(&format!("Linked {} account ({})", provider, user_info.provider_id)),
            );
            log::info!(
                "Linked {provider} account {} to user {user_id}",
                user_info.provider_id
            );
            redirect_to_settings("linked=1")
        }
        Err(e) => {
            log::error!("Failed to persist OAuth link for user {user_id}: {e}");
            redirect_to_settings("error=persist_failed")
        }
    }
}

/// GET /api/oauth/accounts
///
/// Lists the OAuth accounts linked to the authenticated user (HTML fragment
/// for the Settings page).
pub async fn oauth_accounts_list(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => return (StatusCode::INTERNAL_SERVER_ERROR, Html(String::new())),
    };
    let user_id = match resolve_user_id(&state, &headers) {
        Ok(id) => id,
        Err(e) => return e,
    };

    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct LinkRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        provider: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        email: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        display_name: Option<String>,
    }
    let rows: Vec<LinkRow> = diesel::sql_query(
        "SELECT provider, email, display_name \
         FROM oauth_account_links WHERE user_id = $1 ORDER BY provider",
    )
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .load::<LinkRow>(&mut conn)
    .unwrap_or_default();

    if rows.is_empty() {
        return (
            StatusCode::OK,
            Html(r#"<div class="empty-state"><p>No accounts linked yet.</p></div>"#.to_string()),
        );
    }

    let mut html = String::from(r#"<div class="oauth-accounts-list">"#);
    for row in rows {
        let display = row
            .display_name
            .clone()
            .unwrap_or_else(|| row.provider.clone());
        let email = row.email.unwrap_or_default();
        html.push_str(&format!(
            r#"<div class="oauth-account-item">
                <span class="oauth-account-name">{display}</span>
                <span class="oauth-account-email">{email}</span>
                <span class="oauth-account-status">Linked</span>
                <button class="btn-secondary btn-sm" hx-post="/api/oauth/{}/unlink" hx-target="closest .oauth-account-item" hx-swap="outerHTML">Unlink</button>
            </div>"#,
            row.provider
        ));
    }
    html.push_str("</div>");
    (StatusCode::OK, Html(html))
}

/// POST /api/oauth/{provider}/unlink
///
/// Disconnects the provider: revokes the Vault token (best-effort),
/// deletes the link row, and audit-logs the action.
pub async fn oauth_unlink(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(provider_name): Path<String>,
) -> impl IntoResponse {
    let provider_name_lower = provider_name.to_lowercase();
    let Some(provider) = OAuthProvider::parse(&provider_name_lower) else {
        return (
            StatusCode::BAD_REQUEST,
            Html(format!(
                "Unsupported OAuth provider: {}",
                provider_name.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
            )),
        );
    };

    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => return (StatusCode::INTERNAL_SERVER_ERROR, Html("Database unavailable".to_string())),
    };
    let user_id = match resolve_user_id(&state, &headers) {
        Ok(id) => id,
        Err(e) => return e,
    };

    // Best-effort token revocation from Vault.
    let _ = delete_tokens_from_vault(&user_id, provider);

    let affected = diesel::sql_query(
        "DELETE FROM oauth_account_links WHERE user_id = $1 AND provider = $2",
    )
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .bind::<diesel::sql_types::Text, _>(&provider_name_lower)
    .execute(&mut conn)
    .unwrap_or(0);

    record_audit_event(
        &state,
        "oauth.unlink",
        user_id,
        "oauth.unlink",
        Some("oauth_account_links"),
        None,
        affected > 0,
        Some(&format!("Unlinked {} account", provider)),
    );

    if affected == 0 {
        return (
            StatusCode::NOT_FOUND,
            Html(format!("No linked {} account found", provider)),
        );
    }
    (StatusCode::OK, Html(format!("{} account unlinked", provider)))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CallbackParams {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

/// Redirects back to the Settings page (accounts section) preserving the
/// result via query params.
fn redirect_to_settings(query: &str) -> Response {
    let base = base_url();
    Redirect::temporary(&format!("{base}/settings?section=accounts&{query}"))
        .into_response()
}

/// Persists the OAuth access/refresh tokens in Vault under
/// `secret/gbo/oauth/{user_id}/{provider}`. Only non-sensitive metadata is
/// kept in the database.
fn store_tokens_in_vault(
    user_id: &uuid::Uuid,
    provider: OAuthProvider,
    token: &botcoreoauth::OAuthTokenResponse,
) -> anyhow::Result<()> {
    let manager = botcoresecrets::SecretsManager::get_clone()?;
    if !manager.is_enabled() {
        return Ok(());
    }
    let path = format!(
        "gbo/oauth/{}/{}",
        user_id,
        provider.to_string().to_lowercase()
    );
    let mut data = HashMap::new();
    data.insert("access_token".to_string(), token.access_token.clone());
    data.insert("token_type".to_string(), token.token_type.clone());
    data.insert(
        "expires_in".to_string(),
        token.expires_in.unwrap_or(0).to_string(),
    );
    if let Some(refresh) = &token.refresh_token {
        data.insert("refresh_token".to_string(), refresh.clone());
    }
    if let Some(scope) = &token.scope {
        data.insert("scope".to_string(), scope.clone());
    }
    let manager = manager.clone();
    let path_clone = path.clone();
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build();
        let result = if let Ok(rt) = rt {
            rt.block_on(manager.put_secret(&path_clone, data))
        } else {
            Err(anyhow::anyhow!("failed to create runtime"))
        };
        let _ = tx.send(result);
    });
    match rx.recv() {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(e),
        Err(e) => Err(anyhow::anyhow!("Vault write channel closed: {e}")),
    }
}

/// Removes the stored tokens for a linked account (unlink flow).
fn delete_tokens_from_vault(
    user_id: &uuid::Uuid,
    provider: OAuthProvider,
) -> anyhow::Result<()> {
    let manager = botcoresecrets::SecretsManager::get_clone()?;
    if !manager.is_enabled() {
        return Ok(());
    }
    let path = format!(
        "gbo/oauth/{}/{}",
        user_id,
        provider.to_string().to_lowercase()
    );
    let manager = manager.clone();
    let path_clone = path.clone();
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build();
        let result = if let Ok(rt) = rt {
            rt.block_on(manager.delete_secret(&path_clone))
        } else {
            Err(anyhow::anyhow!("failed to create runtime"))
        };
        let _ = tx.send(result);
    });
    match rx.recv() {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(e),
        Err(e) => Err(anyhow::anyhow!("Vault delete channel closed: {e}")),
    }
}

/// Upserts the account link row (one link per user/provider).
fn upsert_account_link(
    state: &Arc<AppState>,
    user_id: uuid::Uuid,
    provider: OAuthProvider,
    user_info: &botcoreoauth::OAuthUserInfo,
) -> anyhow::Result<()> {
    let mut conn = get_conn(state)
        .ok_or_else(|| anyhow::anyhow!("database unavailable"))?;
    let provider_lower = provider.to_string().to_lowercase();
    diesel::sql_query(
        "INSERT INTO oauth_account_links \
         (id, user_id, provider, provider_user_id, email, display_name, avatar_url, linked_at, updated_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW()) \
         ON CONFLICT (user_id, provider) \
         DO UPDATE SET provider_user_id = EXCLUDED.provider_user_id, \
                       email = EXCLUDED.email, display_name = EXCLUDED.display_name, \
                       avatar_url = EXCLUDED.avatar_url, updated_at = NOW()",
    )
    .bind::<diesel::sql_types::Uuid, _>(uuid::Uuid::new_v4())
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .bind::<diesel::sql_types::Text, _>(&provider_lower)
    .bind::<diesel::sql_types::Text, _>(&user_info.provider_id)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(user_info.email.clone())
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(user_info.name.clone())
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(user_info.avatar_url.clone())
    .execute(&mut conn)
    .map_err(|e| anyhow::anyhow!("upsert oauth link: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linkable_providers() {
        assert!(LINKABLE_PROVIDERS.contains(&"google"));
        assert!(LINKABLE_PROVIDERS.contains(&"microsoft"));
        assert!(LINKABLE_PROVIDERS.contains(&"github"));
        assert_eq!(LINKABLE_PROVIDERS.len(), 3);
    }

    #[test]
    fn test_provider_parse() {
        assert_eq!(OAuthProvider::parse("google"), Some(OAuthProvider::Google));
        assert_eq!(
            OAuthProvider::parse("microsoft"),
            Some(OAuthProvider::Microsoft)
        );
        assert_eq!(OAuthProvider::parse("github"), Some(OAuthProvider::GitHub));
        assert_eq!(OAuthProvider::parse("unknown"), None);
    }
}

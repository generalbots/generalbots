//! ChatGPT plan OAuth (#1653).
//!
//! The desktop flow uses a loopback listener: `connect` returns the
//! authorization URL, the browser returns the code to
//! `http://127.0.0.1:1455/auth/callback`, and the server exchanges it for an
//! access/refresh token. Tokens live in Vault; the database stores only the
//! display metadata and the Vault path.

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum::{Json, Router};
use base64::Engine;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use botcore::shared::utils::DbPool;

use crate::core::ai_accounts;

const AUTHORIZE_URL: &str = "https://auth.openai.com/oauth/authorize";
const TOKEN_URL: &str = "https://auth.openai.com/oauth/token";
const DEFAULT_CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";
const DEFAULT_PORT: u16 = 1455;

#[derive(Clone)]
pub(super) struct Pending {
    pub(super) verifier: String,
    pub(super) created: u64,
    pub(super) user_id: Uuid,
}

pub(super) static PENDING: LazyLock<Mutex<HashMap<String, Pending>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub(super) fn bad(status: StatusCode, message: &str) -> (StatusCode, String) {
    (status, message.to_string())
}

pub(super) fn base64url(bytes: &[u8]) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

pub(super) fn pkce_challenge(verifier: &str) -> String {
    let digest = Sha256::digest(verifier.as_bytes());
    base64url(&digest)
}

pub(super) fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

pub(super) fn client_id() -> String {
    std::env::var("OPENAI_OAUTH_CLIENT_ID")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| DEFAULT_CLIENT_ID.to_string())
}

pub(super) fn port() -> u16 {
    std::env::var("OPENAI_OAUTH_PORT")
        .ok()
        .and_then(|value| value.trim().parse::<u16>().ok())
        .unwrap_or(DEFAULT_PORT)
}

pub(super) fn redirect_uri() -> String {
    std::env::var("OPENAI_OAUTH_REDIRECT")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| format!("http://127.0.0.1:{}/auth/callback", port()))
}

/// Token endpoint; override for a local mock or a regional mirror.
pub(super) fn token_url() -> String {
    std::env::var("OPENAI_OAUTH_TOKEN_URL")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| TOKEN_URL.to_string())
}

pub(super) fn decode_jwt_payload(token: &str) -> Option<Value> {
    let segment = token.split('.').nth(1)?;
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(segment)
        .or_else(|_| base64::engine::general_purpose::STANDARD.decode(segment))
        .ok()?;
    serde_json::from_slice(&bytes).ok()
}

pub(super) fn claim_string(payload: &Value, key: &str) -> Option<String> {
    payload
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .or_else(|| {
            payload
                .get("https://api.openai.com/auth")
                .and_then(|auth| auth.get(key))
                .and_then(Value::as_str)
                .map(str::to_string)
        })
}

/// Starts the loopback listener and returns the authorization URL.
pub async fn start_connect(
    pool: &DbPool,
    user_id: Uuid,
) -> Result<Json<Value>, (StatusCode, String)> {
    let verifier = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    let challenge = pkce_challenge(&verifier);
    let state = Uuid::new_v4().to_string();
    {
        let mut pending = PENDING
            .lock()
            .map_err(|_| bad(StatusCode::INTERNAL_SERVER_ERROR, "OAuth state lock failed"))?;
        pending.retain(|_, entry| now_secs().saturating_sub(entry.created) < 900);
        pending.insert(
            state.clone(),
            Pending {
                verifier,
                created: now_secs(),
                user_id,
            },
        );
    }

    let redirect = redirect_uri();
    let mut url = url::Url::parse(AUTHORIZE_URL).map_err(|error| {
        bad(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("authorize URL: {error}"),
        )
    })?;
    url.query_pairs_mut()
        .append_pair("response_type", "code")
        .append_pair("client_id", &client_id())
        .append_pair("redirect_uri", &redirect)
        .append_pair("scope", "openid profile email offline_access")
        .append_pair("code_challenge", &challenge)
        .append_pair("code_challenge_method", "S256")
        .append_pair("state", &state)
        .append_pair("id_token_add_organizations", "true")
        .append_pair("codex_cli_simplified_flow", "true");

    spawn_listener(pool.clone(), state.clone()).await?;

    Ok(Json(json!({
        "provider": ai_accounts::CHATGPT_PLAN,
        "state": "connecting",
        "url": url.to_string(),
        "redirect_uri": redirect,
        "message": "Open the ChatGPT sign-in page; this window will update when the account is linked."
    })))
}

async fn spawn_listener(pool: DbPool, state: String) -> Result<(), (StatusCode, String)> {
    let address = format!("127.0.0.1:{}", port());
    let listener = tokio::net::TcpListener::bind(&address)
        .await
        .map_err(|error| {
            bad(
                StatusCode::CONFLICT,
                &format!(
                    "Another ChatGPT sign-in is already in progress ({address} is busy). \
                     Finish that sign-in or wait up to 15 minutes, then retry. ({error})"
                ),
            )
        })?;
    let router = Router::new()
        .route("/auth/callback", get(callback))
        .with_state((pool, state));
    // IMP-09: the OAuth callback listener is a loopback `axum::serve` of its
    // own; without the layer it answers the browser with a bare error page.
    let router = crate::security::headers::with_security_headers(
        router,
        crate::security::headers::SecurityHeadersConfig::from_env(),
    );
    tokio::spawn(async move {
        let server = axum::serve(listener, router);
        // An abandoned sign-in used to hold the fixed loopback port forever,
        // so nobody could start a new one (#1651).
        match tokio::time::timeout(std::time::Duration::from_secs(900), async { server.await })
            .await
        {
            Ok(Ok(())) => {}
            Ok(Err(error)) => log::warn!("chatgpt oauth listener stopped: {error}"),
            Err(_) => log::info!("chatgpt oauth listener expired after 15 minutes"),
        }
    });
    Ok(())
}

async fn callback(
    State((pool, expected_state)): State<(DbPool, String)>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    if params.get("state").map(String::as_str) != Some(expected_state.as_str()) {
        return Html("<h2>ChatGPT sign-in failed</h2><p>The state did not match. Start again from Settings.</p>".to_string())
            .into_response();
    }
    let Some(code) = params.get("code").filter(|code| !code.is_empty()) else {
        return Html(
            "<h2>ChatGPT sign-in failed</h2><p>No authorization code was returned.</p>".to_string(),
        )
        .into_response();
    };
    let pending = {
        match PENDING.lock() {
            Ok(mut pending) => pending.remove(&expected_state),
            Err(_) => None,
        }
    };
    let Some(pending) = pending else {
        return Html(
            "<h2>ChatGPT sign-in expired</h2><p>Start again from Settings.</p>".to_string(),
        )
        .into_response();
    };
    match exchange(&pool, code, &pending.verifier, pending.user_id).await {
        Ok(()) => Html("<h2>ChatGPT plan connected</h2><p>You can close this window and return to General Bots.</p>".to_string())
            .into_response(),
        Err(error) => {
            log::warn!("chatgpt oauth exchange failed: {error}");
            Html(format!("<h2>ChatGPT sign-in failed</h2><p>{error}</p>")).into_response()
        }
    }
}

mod oauth;

pub(crate) use oauth::access_token;
use oauth::exchange;

/// Removes the Vault token and marks the account unlinked.
pub async fn disconnect(pool: &DbPool, user_id: Uuid) -> Result<(), String> {
    if let Ok(manager) = botcoresecrets::SecretsManager::get() {
        if let Err(e) = manager
            .delete_secret(&crate::llm::subscription::chatgpt_vault_path(user_id))
            .await
        {
            log::debug!("chatgpt Vault token delete failed: {e}");
        }
    }
    ai_accounts::update_status(
        pool,
        user_id,
        ai_accounts::CHATGPT_PLAN,
        "unlinked",
        None,
        None,
        None,
    )
}

/// True while this user has an OAuth callback waiting.
pub fn pending_for(user_id: Uuid) -> bool {
    let now = now_secs();
    if let Ok(mut pending) = PENDING.lock() {
        pending.retain(|_, entry| now.saturating_sub(entry.created) < 900);
        return pending.values().any(|entry| entry.user_id == user_id);
    }
    false
}

/// Current row, without making a network call.
pub fn status(pool: &DbPool, user_id: Uuid) -> Value {
    ai_accounts::list_accounts(pool, user_id)
        .into_iter()
        .find(|row| row.get("provider").and_then(Value::as_str) == Some(ai_accounts::CHATGPT_PLAN))
        .unwrap_or_else(|| json!({ "provider": ai_accounts::CHATGPT_PLAN, "status": "unlinked" }))
}

#[cfg(test)]
mod tests;

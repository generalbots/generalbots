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
struct Pending {
    verifier: String,
    created: u64,
    user_id: Uuid,
}

static PENDING: LazyLock<Mutex<HashMap<String, Pending>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn bad(status: StatusCode, message: &str) -> (StatusCode, String) {
    (status, message.to_string())
}

fn base64url(bytes: &[u8]) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

fn pkce_challenge(verifier: &str) -> String {
    let digest = Sha256::digest(verifier.as_bytes());
    base64url(&digest)
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn client_id() -> String {
    std::env::var("OPENAI_OAUTH_CLIENT_ID")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| DEFAULT_CLIENT_ID.to_string())
}

fn port() -> u16 {
    std::env::var("OPENAI_OAUTH_PORT")
        .ok()
        .and_then(|value| value.trim().parse::<u16>().ok())
        .unwrap_or(DEFAULT_PORT)
}

fn redirect_uri() -> String {
    std::env::var("OPENAI_OAUTH_REDIRECT")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| format!("http://127.0.0.1:{}/auth/callback", port()))
}

/// Token endpoint; override for a local mock or a regional mirror.
fn token_url() -> String {
    std::env::var("OPENAI_OAUTH_TOKEN_URL")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| TOKEN_URL.to_string())
}

fn decode_jwt_payload(token: &str) -> Option<Value> {
    let segment = token.split('.').nth(1)?;
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(segment)
        .or_else(|_| base64::engine::general_purpose::STANDARD.decode(segment))
        .ok()?;
    serde_json::from_slice(&bytes).ok()
}

fn claim_string(payload: &Value, key: &str) -> Option<String> {
    payload.get(key).and_then(Value::as_str).map(str::to_string).or_else(|| {
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
            Pending { verifier, created: now_secs(), user_id },
        );
    }

    let redirect = redirect_uri();
    let mut url = url::Url::parse(AUTHORIZE_URL)
        .map_err(|error| bad(StatusCode::INTERNAL_SERVER_ERROR, &format!("authorize URL: {error}")))?;
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
    tokio::spawn(async move {
        let server = axum::serve(listener, router);
        // An abandoned sign-in used to hold the fixed loopback port forever,
        // so nobody could start a new one (#1651).
        match tokio::time::timeout(
            std::time::Duration::from_secs(900),
            async { server.await },
        )
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
        return Html("<h2>ChatGPT sign-in failed</h2><p>No authorization code was returned.</p>".to_string())
            .into_response();
    };
    let pending = {
        match PENDING.lock() {
            Ok(mut pending) => pending.remove(&expected_state),
            Err(_) => None,
        }
    };
    let Some(pending) = pending else {
        return Html("<h2>ChatGPT sign-in expired</h2><p>Start again from Settings.</p>".to_string())
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

async fn exchange(pool: &DbPool, code: &str, verifier: &str, user_id: Uuid) -> Result<(), String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|error| format!("HTTP client: {error}"))?;
    let response = client
        .post(token_url())
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", redirect_uri().as_str()),
            ("client_id", client_id().as_str()),
            ("code_verifier", verifier),
        ])
        .send()
        .await
        .map_err(|error| format!("token request: {error}"))?;
    let status = response.status();
    let body: Value = response
        .json()
        .await
        .map_err(|error| format!("token response: {error}"))?;
    if !status.is_success() {
        return Err(format!(
            "the token endpoint refused the code ({status}): {}",
            body.get("error_description")
                .or_else(|| body.get("error"))
                .and_then(Value::as_str)
                .unwrap_or("no detail")
        ));
    }
    let access = body
        .get("access_token")
        .and_then(Value::as_str)
        .ok_or_else(|| "the token endpoint returned no access token".to_string())?;
    let refresh = body.get("refresh_token").and_then(Value::as_str).unwrap_or("");
    let id_token = body.get("id_token").and_then(Value::as_str).unwrap_or("");
    let expires_in = body
        .get("expires_in")
        .and_then(Value::as_u64)
        .unwrap_or(3600);
    let claims = decode_jwt_payload(id_token).unwrap_or(Value::Null);
    let email = claims.get("email").and_then(Value::as_str).map(str::to_string);
    let account_id = claim_string(&claims, "chatgpt_account_id");
    let plan = claim_string(&claims, "chatgpt_plan_type");

    let mut data = HashMap::new();
    data.insert("access_token".to_string(), access.to_string());
    data.insert("refresh_token".to_string(), refresh.to_string());
    data.insert("id_token".to_string(), id_token.to_string());
    data.insert("expires_at".to_string(), (now_secs() + expires_in).to_string());
    data.insert(
        "base_url".to_string(),
        std::env::var("OPENAI_RESPONSES_BASE_URL")
            .unwrap_or_else(|_| botllm::openai_responses::DEFAULT_BASE_URL.to_string()),
    );
    if let Some(account) = account_id.clone() {
        data.insert("account_id".to_string(), account);
    }
    if let Some(email) = email.clone() {
        data.insert("email".to_string(), email);
    }
    if let Some(plan) = plan.clone() {
        data.insert("plan".to_string(), plan);
    }
    let manager = botcoresecrets::SecretsManager::get()
        .map_err(|error| format!("Vault is unavailable: {error}"))?;
    let vault_path = crate::llm::subscription::chatgpt_vault_path(user_id);
    manager
        .put_secret(&vault_path, data)
        .await
        .map_err(|error| format!("could not store the token in Vault: {error}"))?;
    ai_accounts::ensure_user(pool, user_id, email.as_deref())
        .map_err(|error| format!("could not record the account owner: {error}"))?;
    ai_accounts::mark_linked(
        pool,
        user_id,
        ai_accounts::CHATGPT_PLAN,
        email.as_deref(),
        plan.as_deref(),
        None,
        Some(&vault_path),
    )
    .map_err(|error| format!("could not record the linked account: {error}"))?;
    Ok(())
}

/// Reads the stored token, refreshing it when it is close to expiry.
pub async fn access_token(
    user_id: Uuid,
) -> Result<(String, Option<String>), String> {
    let manager = botcoresecrets::SecretsManager::get()
        .map_err(|error| format!("Vault is unavailable: {error}"))?;
    let path = crate::llm::subscription::chatgpt_vault_path(user_id);
    let secret = manager
        .get_secret(&path)
        .await
        .map_err(|error| format!("could not read the ChatGPT token: {error}"))?;
    let access = secret.get("access_token").cloned().unwrap_or_default();
    let expires_at = secret
        .get("expires_at")
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(0);
    let account = secret.get("account_id").cloned();
    if !access.is_empty() && expires_at > now_secs() + 60 {
        return Ok((access, account));
    }
    let refresh = secret.get("refresh_token").cloned().unwrap_or_default();
    if refresh.is_empty() {
        return Err("The ChatGPT plan token expired; connect the account again.".to_string());
    }
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|error| format!("HTTP client: {error}"))?;
    let response = client
        .post(token_url())
        .form(&[
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh.as_str()),
            ("client_id", client_id().as_str()),
        ])
        .send()
        .await
        .map_err(|error| format!("refresh request: {error}"))?;
    let status = response.status();
    let body: Value = response
        .json()
        .await
        .map_err(|error| format!("refresh response: {error}"))?;
    if !status.is_success() {
        return Err("The ChatGPT plan session expired; connect the account again.".to_string());
    }
    let access = body
        .get("access_token")
        .and_then(Value::as_str)
        .ok_or_else(|| "the refresh response returned no access token".to_string())?;
    let expires_in = body.get("expires_in").and_then(Value::as_u64).unwrap_or(3600);
    let mut updated = secret;
    updated.insert("access_token".to_string(), access.to_string());
    updated.insert("expires_at".to_string(), (now_secs() + expires_in).to_string());
    if let Some(refresh) = body.get("refresh_token").and_then(Value::as_str) {
        if !refresh.is_empty() {
            updated.insert("refresh_token".to_string(), refresh.to_string());
        }
    }
    manager
        .put_secret(&path, updated)
        .await
        .map_err(|error| format!("could not refresh the ChatGPT token: {error}"))?;
    Ok((access.to_string(), account))
}

/// Removes the Vault token and marks the account unlinked.
pub async fn disconnect(pool: &DbPool, user_id: Uuid) -> Result<(), String> {
    if let Ok(manager) = botcoresecrets::SecretsManager::get() {
        let _ = manager
            .delete_secret(&crate::llm::subscription::chatgpt_vault_path(user_id))
            .await;
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
mod tests {
    use super::{decode_jwt_payload, pkce_challenge};

    #[test]
    fn pkce_challenge_is_url_safe() {
        let challenge = pkce_challenge("verifier");
        assert!(!challenge.contains('='));
        assert!(!challenge.contains('+'));
        assert!(!challenge.contains('/'));
    }

    #[test]
    fn jwt_payload_decodes() {
        // {"sub":"1"} in base64url, unsigned test token.
        let token = "x.eyJzdWIiOiIxIn0.y";
        let payload = decode_jwt_payload(token).expect("payload");
        assert_eq!(payload["sub"], "1");
    }
}

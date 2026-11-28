//! Zitadel-backed mail credentials.
//!
//! The suite already has a Zitadel session, so a General Bots mail account can
//! reuse that access token as the XOAUTH2 bearer instead of asking for a
//! separate mailbox password. This module resolves the directory endpoint from
//! the same configuration the rest of the server uses, stores the token on the
//! account and exposes the client credentials needed for a token refresh.
//!
//! No host is hardcoded: the directory URL comes from `GB_DIRECTORY_URL` (or
//! `MAIL_OAUTH_ZITADEL_URL`), `ZITADEL_DOMAIN`, or the on-disk
//! `directory_config.json` written by the directory bootstrap.

use super::encode_secret;
use chrono::{Duration, Utc};
use diesel::prelude::*;
use uuid::Uuid;

/// Provider identifier stored on accounts backed by the suite session.
pub const ZITADEL_PROVIDER: &str = "zitadel";

/// A suite session token carries no expiry in the connect request. Treat it as
/// usable for an hour; the caller can reconnect with a fresh token afterwards.
pub(crate) const TOKEN_TTL_SECS: i64 = 3600;

/// Base URL of the directory service, e.g. `https://directory.example.com`.
pub fn api_base() -> Option<String> {
    for key in ["GB_DIRECTORY_URL", "MAIL_OAUTH_ZITADEL_URL"] {
        if let Ok(value) = std::env::var(key) {
            let url = value.trim().trim_end_matches('/');
            if !url.is_empty() {
                return Some(url.to_string());
            }
        }
    }
    if let Ok(domain) = std::env::var("ZITADEL_DOMAIN") {
        let host = domain.trim().trim_end_matches('/');
        if !host.is_empty() {
            return Some(format!("https://{host}"));
        }
    }
    let stack = std::env::var("BOTSERVER_STACK_PATH").unwrap_or_default();
    let candidates: Vec<String> = if stack.trim().is_empty() {
        vec![
            "botserver-stack/conf/system/directory_config.json".to_string(),
            "../botserver-stack/conf/system/directory_config.json".to_string(),
        ]
    } else {
        vec![format!(
            "{}/conf/system/directory_config.json",
            stack.trim_end_matches('/')
        )]
    };
    let raw = candidates
        .iter()
        .find_map(|path| std::fs::read_to_string(path).ok())?;
    let config: serde_json::Value = serde_json::from_str(&raw).ok()?;
    for key in ["api_url", "url", "base_url"] {
        if let Some(value) = config.get(key).and_then(serde_json::Value::as_str) {
            let url = value.trim().trim_end_matches('/');
            if !url.is_empty() {
                return Some(url.to_string());
            }
        }
    }
    None
}

/// Client credentials for a Zitadel token refresh. The directory client is the
/// one already configured for this installation; secrets are never logged or
/// echoed back to the caller.
pub fn client_credentials() -> Option<(String, String)> {
    let client_id = std::env::var("ZITADEL_CLIENT_ID").ok()?;
    let client_secret = std::env::var("ZITADEL_CLIENT_SECRET").ok()?;
    if client_id.is_empty() || client_secret.is_empty() {
        return None;
    }
    Some((client_id, client_secret))
}

/// Validation shared by the HTTP handler and the storage call. Runs before any
/// SQL so it is also usable by offline tests.
pub(crate) fn validate_direct_storage(provider: &str, access_token: &str) -> Result<(), String> {
    if !provider.eq_ignore_ascii_case(ZITADEL_PROVIDER) {
        return Err(format!(
            "Unsupported OAuth2 provider '{provider}' for direct token storage"
        ));
    }
    if access_token.trim().is_empty() {
        return Err("Access token is required".to_string());
    }
    Ok(())
}

/// Stores a caller-supplied suite token on an OAuth2 mail account.
///
/// The caller has already been authenticated and verified as the owner of the
/// account. Only the access token is stored: the suite-token flow has no
/// authorization code or refresh token, so the caller reconnects with a fresh
/// token when the stored one expires.
pub fn store_credentials(
    conn: &mut diesel::PgConnection,
    account_id: Uuid,
    provider: &str,
    access_token: &str,
) -> Result<(), String> {
    validate_direct_storage(provider, access_token)?;
    let encoded = encode_secret(access_token);
    let expires_at = Utc::now() + Duration::seconds(TOKEN_TTL_SECS);

    diesel::sql_query(
        "UPDATE user_email_accounts SET auth_mode = 'oauth2', oauth_provider = $1, \
         access_token_encrypted = $2, token_expires_at = $3, refresh_token_encrypted = NULL, \
         last_error = NULL, last_error_at = NULL, updated_at = now() WHERE id = $4",
    )
    .bind::<diesel::sql_types::Text, _>(ZITADEL_PROVIDER)
    .bind::<diesel::sql_types::Text, _>(encoded)
    .bind::<diesel::sql_types::Timestamptz, _>(expires_at)
    .bind::<diesel::sql_types::Uuid, _>(account_id)
    .execute(conn)
    .map_err(|e| format!("Failed to store the OAuth2 credentials: {e}"))?;
    Ok(())
}

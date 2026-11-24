use super::parse::{classify_status, percent_encode, truncate};
use crate::external::types::SyncError;
use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use serde_json::Value;

/// OAuth endpoints.
const AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";

/// Token endpoint; overridable for a local mock in tests or a proxy mirror.
fn token_url() -> String {
    std::env::var("EXTERNAL_DRIVE_GOOGLE_TOKEN_URL")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| TOKEN_URL.to_string())
}

/// A token set as returned by the Google token endpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenSet {
    /// Bearer access token.
    pub access_token: String,
    /// Refresh token (issued because we request `access_type=offline`).
    pub refresh_token: Option<String>,
    /// Absolute expiry, derived from `expires_in`.
    pub expires_at: DateTime<Utc>,
    /// Scopes the provider reports as granted (#1643). Falls back to the
    /// requested list when the provider omits it.
    pub scope: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    #[serde(default)]
    refresh_token: Option<String>,
    #[serde(default)]
    expires_in: Option<i64>,
    #[serde(default)]
    scope: Option<String>,
}

impl TokenResponse {
    fn into_token_set(self, now: DateTime<Utc>) -> TokenSet {
        TokenSet {
            access_token: self.access_token,
            refresh_token: self.refresh_token,
            expires_at: now + chrono::Duration::seconds(self.expires_in.unwrap_or(3600)),
            scope: self.scope,
        }
    }
}

/// Build the authorization URL. `access_type=offline` + `prompt=consent` is
/// what makes Google issue a refresh token for a desktop-style client.
pub fn authorize_url(client_id: &str, redirect_uri: &str, state: &str) -> String {
    authorize_url_with_pkce(client_id, redirect_uri, state, None)
}

/// Build the authorization URL with an optional PKCE challenge. Managed
/// installed-app clients have no secret, so the desktop flow always supplies
/// the S256 challenge derived from the verifier carried in the signed state.
pub fn authorize_url_with_pkce(
    client_id: &str,
    redirect_uri: &str,
    state: &str,
    code_challenge: Option<&str>,
) -> String {
    // #2047 — one consent asks for read + write, so a save target exists from
    // the first connection; existing read-only grants are not rewritten.
    let scope = crate::external::types::Provider::GoogleDrive
        .scopes()
        .join(" ");
    let mut url = format!(
        "{AUTH_URL}?client_id={client}&redirect_uri={redirect}&response_type=code\
&scope={scope}&access_type=offline&prompt=consent&include_granted_scopes=true&state={state}",
        client = percent_encode(client_id),
        redirect = percent_encode(redirect_uri),
        scope = percent_encode(&scope),
        state = percent_encode(state),
    );
    if let Some(challenge) = code_challenge.filter(|value| !value.is_empty()) {
        url.push_str("&code_challenge=");
        url.push_str(&percent_encode(challenge));
        url.push_str("&code_challenge_method=S256");
    }
    url
}

/// Exchange an authorization code for tokens.
pub async fn exchange_code(
    http: &reqwest::Client,
    client_id: &str,
    client_secret: &str,
    redirect_uri: &str,
    code: &str,
) -> Result<TokenSet, SyncError> {
    exchange_code_with_pkce(
        http,
        client_id,
        Some(client_secret),
        redirect_uri,
        code,
        None,
    )
    .await
}

/// Exchange an authorization code for tokens, omitting the secret for public
/// clients and presenting the PKCE verifier when the flow used one.
pub async fn exchange_code_with_pkce(
    http: &reqwest::Client,
    client_id: &str,
    client_secret: Option<&str>,
    redirect_uri: &str,
    code: &str,
    code_verifier: Option<&str>,
) -> Result<TokenSet, SyncError> {
    let mut form: Vec<(&str, &str)> = vec![
        ("client_id", client_id),
        ("grant_type", "authorization_code"),
        ("code", code),
        ("redirect_uri", redirect_uri),
        ("access_type", "offline"),
    ];
    if let Some(secret) = client_secret.filter(|value| !value.is_empty()) {
        form.push(("client_secret", secret));
    }
    if let Some(verifier) = code_verifier.filter(|value| !value.is_empty()) {
        form.push(("code_verifier", verifier));
    }
    post_token(http, &form).await
}

/// Refresh an expired access token.
pub async fn refresh_token(
    http: &reqwest::Client,
    client_id: &str,
    client_secret: &str,
    refresh: &str,
) -> Result<TokenSet, SyncError> {
    refresh_token_optional_secret(http, client_id, Some(client_secret), refresh).await
}

/// Refresh with an optional secret so managed public clients keep working.
pub async fn refresh_token_optional_secret(
    http: &reqwest::Client,
    client_id: &str,
    client_secret: Option<&str>,
    refresh: &str,
) -> Result<TokenSet, SyncError> {
    let mut form: Vec<(&str, &str)> = vec![
        ("client_id", client_id),
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh),
    ];
    if let Some(secret) = client_secret.filter(|value| !value.is_empty()) {
        form.push(("client_secret", secret));
    }
    post_token(http, &form).await
}

async fn post_token(http: &reqwest::Client, form: &[(&str, &str)]) -> Result<TokenSet, SyncError> {
    let res = http
        .post(token_url())
        .form(form)
        .send()
        .await
        .map_err(|e| SyncError::Transient(format!("token request failed: {e}")))?;
    let status = res.status();
    let body = match res.text().await {
        Ok(t) => t,
        Err(e) => {
            log::warn!("Google Drive token error response body unreadable: {:?}", e);
            String::new()
        }
    };
    if !status.is_success() {
        // A rejected code or refresh token is a reconnect, not a retry: 400 is
        // what Google answers an invalid_grant with, and treating it as
        // permanent would leave the UI claiming everything is fine.
        if status == 400 || status == 401 {
            return Err(SyncError::Auth(truncate(&body)));
        }
        return Err(classify_status(status.as_u16(), &body));
    }
    let parsed: TokenResponse = serde_json::from_str(&body)
        .map_err(|e| SyncError::Transient(format!("token response unreadable: {e}")))?;
    Ok(parsed.into_token_set(Utc::now()))
}

use super::delta::{percent_encode, truncate, TOKEN_PATH};
use crate::external::types::SyncError;
use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;

/// A token set as returned by the Microsoft token endpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenSet {
    /// Bearer access token.
    pub access_token: String,
    /// Refresh token, when offline access was granted.
    pub refresh_token: Option<String>,
    /// Absolute expiry of the access token.
    pub expires_at: DateTime<Utc>,
    /// Scopes the provider reports as granted (#1643). `routes/callback.rs`
    /// stores this for the consent record and reads it for both providers, so
    /// the field has to exist here too — Graph does return `scope`.
    pub scope: Option<String>,
}

/// Minimal slice of the token endpoint response.
#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    #[serde(default)]
    refresh_token: Option<String>,
    /// Lifetime in seconds.
    expires_in: i64,
    /// Granted scopes, space separated. Optional: Microsoft omits it on some
    /// tenant configurations, and the consent record then keeps the requested
    /// list (see `routes/callback.rs`).
    #[serde(default)]
    scope: Option<String>,
}

impl TokenResponse {
    fn into_token_set(self, now: DateTime<Utc>) -> TokenSet {
        // Take a small safety margin so we never race the expiry on the wire.
        let expires_at = now + Duration::seconds(self.expires_in.saturating_sub(60));
        TokenSet {
            access_token: self.access_token,
            refresh_token: self.refresh_token,
            expires_at,
            scope: self.scope,
        }
    }
}

/// Build the consent URL the user is sent to.
///
/// `state` is the signed CSRF token that round-trips through the callback.
pub fn authorize_url(tenant_id: &str, client_id: &str, redirect_uri: &str, state: &str) -> String {
    authorize_url_with_pkce(tenant_id, client_id, redirect_uri, state, None)
}

/// Build the consent URL, optionally adding a PKCE S256 challenge for public
/// clients that do not present a client secret.
pub fn authorize_url_with_pkce(
    tenant_id: &str,
    client_id: &str,
    redirect_uri: &str,
    state: &str,
    code_challenge: Option<&str>,
) -> String {
    // OneDrive uses the AppFolder (per-app isolated space) by default — it is
    // the narrowest permission that still lets us sync files without browsing
    // the user's whole drive. Files.ReadWrite.All can be granted in the Entra
    // app registration for broader access, but the request here asks for the
    // minimal set. offline_access is what yields the refresh token.
    let scopes = [
        "offline_access",
        "Files.Read.All",
        "Files.ReadWrite.AppFolder",
        "User.Read",
    ]
    .join(" ");
    let mut url = format!(
        "https://login.microsoftonline.com/{tenant}/oauth2/v2.0/authorize\
?client_id={client}&response_type=code&redirect_uri={redirect}&response_mode=query\
&scope={scopes}&state={state}",
        tenant = tenant_id,
        client = percent_encode(client_id),
        redirect = percent_encode(redirect_uri),
        scopes = percent_encode(&scopes),
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
    tenant_id: &str,
    client_id: &str,
    client_secret: &str,
    redirect_uri: &str,
    code: &str,
) -> Result<TokenSet, SyncError> {
    exchange_code_with_pkce(
        http,
        tenant_id,
        client_id,
        Some(client_secret),
        redirect_uri,
        code,
        None,
    )
    .await
}

/// Exchange an authorization code, omitting the secret for public clients and
/// presenting the PKCE verifier when the flow used one.
pub async fn exchange_code_with_pkce(
    http: &reqwest::Client,
    tenant_id: &str,
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
        ("scope", "offline_access Files.Read.All User.Read"),
    ];
    if let Some(secret) = client_secret.filter(|value| !value.is_empty()) {
        form.push(("client_secret", secret));
    }
    if let Some(verifier) = code_verifier.filter(|value| !value.is_empty()) {
        form.push(("code_verifier", verifier));
    }
    post_token(http, tenant_id, &form).await
}

/// Refresh an expired access token.
pub async fn refresh_token(
    http: &reqwest::Client,
    tenant_id: &str,
    client_id: &str,
    client_secret: &str,
    refresh: &str,
) -> Result<TokenSet, SyncError> {
    refresh_token_optional_secret(http, tenant_id, client_id, Some(client_secret), refresh).await
}

/// Refresh with an optional secret so managed public clients keep working.
pub async fn refresh_token_optional_secret(
    http: &reqwest::Client,
    tenant_id: &str,
    client_id: &str,
    client_secret: Option<&str>,
    refresh: &str,
) -> Result<TokenSet, SyncError> {
    let mut form: Vec<(&str, &str)> = vec![
        ("client_id", client_id),
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh),
        ("scope", "offline_access Files.Read.All User.Read"),
    ];
    if let Some(secret) = client_secret.filter(|value| !value.is_empty()) {
        form.push(("client_secret", secret));
    }
    post_token(http, tenant_id, &form).await
}

async fn post_token(
    http: &reqwest::Client,
    tenant_id: &str,
    form: &[(&str, &str)],
) -> Result<TokenSet, SyncError> {
    let url = format!("https://login.microsoftonline.com/{tenant_id}{TOKEN_PATH}");
    let res = http
        .post(&url)
        .form(form)
        .send()
        .await
        .map_err(|e| SyncError::Transient(format!("token request failed: {e}")))?;
    let status = res.status();
    let body = res.text().await.unwrap_or_default();
    if !status.is_success() {
        // Entra answers an expired or revoked refresh token with 400/401 and
        // `invalid_grant`; the only recovery is the user reconnecting.
        if status == 400 || status == 401 {
            return Err(SyncError::Auth(truncate(&body)));
        }
        let kind = super::delta::classify_status(status.as_u16(), &body);
        return Err(kind);
    }
    let parsed: TokenResponse = serde_json::from_str(&body)
        .map_err(|e| SyncError::Transient(format!("token response unreadable: {e}")))?;
    Ok(parsed.into_token_set(Utc::now()))
}

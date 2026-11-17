use super::parse::{classify_status, percent_encode, truncate};
use crate::external::types::SyncError;
use serde::Deserialize;
use serde_json::Value;

use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use serde_json::Value;

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
    let body = res.text().await.unwrap_or_default();
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

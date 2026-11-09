//! #2052 — OAuth2 token lifecycle for data-source connectors.
//!
//! The engine used to ignore `AuthType::OAuth2` entirely: no token was sent, no
//! code exchange existed and there was nowhere to store a token. This module
//! owns the token half: expiry, the refresh request, and applying the answer.
//! Tokens are broker material, so they live in Vault with the rest of the
//! credential envelope (#2054) and are merged in only for a request.

use super::*;

/// Refresh this many seconds before the provider expiry, so a long request
/// does not start with an already-stale token.
const EXPIRY_SKEW_SECS: i64 = 60;

/// True when the connector needs a fresh access token.
pub(super) fn token_expired(auth: &AuthConfig) -> bool {
    match auth.expires_at {
        Some(at) => Utc::now() + chrono::Duration::seconds(EXPIRY_SKEW_SECS) >= at,
        // No expiry recorded: the token is usable only when it is present.
        None => auth
            .access_token
            .as_deref()
            .map(str::trim)
            .filter(|token| !token.is_empty())
            .is_none(),
    }
}

/// The refresh request: token URL plus the form body.
pub(super) struct RefreshForm {
    pub url: String,
    pub form: Vec<(String, String)>,
}

/// Builds the RFC 6749 `refresh_token` grant from the connector's config.
pub(super) fn refresh_form(auth: &AuthConfig) -> Result<RefreshForm, String> {
    let url = auth
        .oauth2_token_url
        .as_deref()
        .map(str::trim)
        .filter(|url| !url.is_empty())
        .ok_or_else(|| "oauth2_token_url is required to refresh the grant".to_string())?;
    let refresh = auth
        .refresh_token
        .as_deref()
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .ok_or_else(|| {
            "oauth2 token expired and no refresh_token is stored; reconnect the account".to_string()
        })?;
    let client_id = auth
        .oauth2_client_id
        .as_deref()
        .map(str::trim)
        .filter(|id| !id.is_empty())
        .ok_or_else(|| "oauth2_client_id is required to refresh the grant".to_string())?;
    let mut form = vec![
        ("grant_type".to_string(), "refresh_token".to_string()),
        ("refresh_token".to_string(), refresh.to_string()),
        ("client_id".to_string(), client_id.to_string()),
    ];
    if let Some(secret) = auth
        .oauth2_client_secret
        .as_deref()
        .map(str::trim)
        .filter(|secret| !secret.is_empty())
    {
        form.push(("client_secret".to_string(), secret.to_string()));
    }
    Ok(RefreshForm {
        url: url.to_string(),
        form,
    })
}

/// Applies a token-endpoint answer to the config.
pub(super) fn apply_refresh(
    auth: &mut AuthConfig,
    body: &serde_json::Value,
) -> Result<(), String> {
    let access = body
        .get("access_token")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .ok_or_else(|| "the token endpoint returned no access_token".to_string())?;
    auth.access_token = Some(access.to_string());
    if let Some(refresh) = body
        .get("refresh_token")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|token| !token.is_empty())
    {
        auth.refresh_token = Some(refresh.to_string());
    }
    let expires_in = body
        .get("expires_in")
        .and_then(|value| value.as_i64())
        .unwrap_or(3600)
        .max(1);
    auth.expires_at = Some(Utc::now() + chrono::Duration::seconds(expires_in));
    if let Some(token_type) = body.get("token_type").and_then(|value| value.as_str()) {
        auth.token_type = Some(token_type.to_string());
    }
    Ok(())
}

/// Refreshes the access token when it is missing or near expiry. Returns `true`
/// when the config was changed, so the caller can persist the new grant.
pub(super) fn ensure_fresh(auth: &mut AuthConfig) -> Result<bool, String> {
    if auth.auth_type != AuthType::OAuth2 || !token_expired(auth) {
        return Ok(false);
    }
    let form = refresh_form(auth)?;
    let response = reqwest::blocking::Client::new()
        .post(&form.url)
        .form(&form.form)
        .timeout(std::time::Duration::from_secs(20))
        .send()
        .map_err(|e| format!("oauth2 refresh request failed: {e}"))?;
    let status = response.status();
    let body = response
        .json::<serde_json::Value>()
        .unwrap_or(serde_json::Value::Null);
    if !status.is_success() {
        return Err(format!(
            "oauth2 refresh rejected (HTTP {status}); reconnect the account"
        ));
    }
    apply_refresh(auth, &body)?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn oauth2_config() -> AuthConfig {
        AuthConfig {
            auth_type: AuthType::OAuth2,
            api_key: None,
            api_key_header: None,
            username: None,
            password: None,
            oauth2_client_id: Some("client-1".to_string()),
            oauth2_client_secret: Some("secret-1".to_string()),
            oauth2_token_url: Some("https://login.example/token".to_string()),
            oauth2_scopes: None,
            base_url: Some("https://api.example".to_string()),
            extra_headers: None,
            access_token: Some("old".to_string()),
            refresh_token: Some("refresh-1".to_string()),
            token_type: None,
            expires_at: Some(Utc::now() + chrono::Duration::hours(1)),
        }
    }

    #[test]
    fn a_token_with_room_to_live_is_not_refreshed() {
        assert!(!token_expired(&oauth2_config()));
    }

    #[test]
    fn an_expired_or_missing_token_requests_a_refresh() {
        let mut config = oauth2_config();
        config.expires_at = Some(Utc::now() - chrono::Duration::minutes(1));
        assert!(token_expired(&config));

        config.access_token = None;
        config.expires_at = None;
        assert!(token_expired(&config));
    }

    #[test]
    fn the_refresh_form_carries_the_grant() {
        let form = refresh_form(&oauth2_config()).expect("form");
        assert_eq!(form.url, "https://login.example/token");
        let pairs: std::collections::HashMap<_, _> = form.form.into_iter().collect();
        assert_eq!(pairs.get("grant_type").map(String::as_str), Some("refresh_token"));
        assert_eq!(pairs.get("refresh_token").map(String::as_str), Some("refresh-1"));
        assert_eq!(pairs.get("client_id").map(String::as_str), Some("client-1"));
        assert_eq!(pairs.get("client_secret").map(String::as_str), Some("secret-1"));
    }

    #[test]
    fn a_grant_without_a_refresh_token_is_not_silently_retried() {
        let mut config = oauth2_config();
        config.expires_at = Some(Utc::now() - chrono::Duration::minutes(1));
        config.refresh_token = None;
        let error = ensure_fresh(&mut config).expect_err("must fail");
        assert!(error.contains("reconnect the account"), "{error}");
    }

    #[test]
    fn applying_a_token_answer_updates_access_expiry_and_rotation() {
        let mut config = oauth2_config();
        apply_refresh(
            &mut config,
            &serde_json::json!({
                "access_token": "new-token",
                "refresh_token": "rotated",
                "expires_in": 120,
                "token_type": "Bearer"
            }),
        )
        .expect("apply");
        assert_eq!(config.access_token.as_deref(), Some("new-token"));
        assert_eq!(config.refresh_token.as_deref(), Some("rotated"));
        assert_eq!(config.token_type.as_deref(), Some("Bearer"));
        assert!(!token_expired(&config));
    }

    #[test]
    fn an_answer_without_an_access_token_is_an_error() {
        let mut config = oauth2_config();
        let error = apply_refresh(&mut config, &serde_json::json!({})).expect_err("must fail");
        assert!(error.contains("access_token"), "{error}");
    }
}

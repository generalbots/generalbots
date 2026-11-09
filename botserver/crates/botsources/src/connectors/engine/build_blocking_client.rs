use super::*;

pub(super) fn build_blocking_client(auth: &AuthConfig) -> Result<reqwest::blocking::Client, String> {
    let mut headers = reqwest::header::HeaderMap::new();

    match &auth.auth_type {
        AuthType::ApiKey => {
            let header_name = auth.api_key_header.as_deref().unwrap_or("X-API-Key");
            if let Some(ref key) = auth.api_key {
                headers.insert(
                    reqwest::header::HeaderName::from_bytes(header_name.as_bytes())
                        .map_err(|e| format!("Invalid header name: {e}"))?,
                    reqwest::header::HeaderValue::from_str(key)
                        .map_err(|e| format!("Invalid header value: {e}"))?,
                );
            }
        }
        AuthType::Bearer => {
            if let Some(ref key) = auth.api_key {
                headers.insert(
                    reqwest::header::AUTHORIZATION,
                    reqwest::header::HeaderValue::from_str(&format!("Bearer {key}"))
                        .map_err(|e| format!("Invalid bearer token: {e}"))?,
                );
            }
        }
        AuthType::Basic => {
            if let (Some(ref user), Some(ref pass)) = (&auth.username, &auth.password) {
                let encoded = base64::engine::general_purpose::STANDARD
                    .encode(format!("{user}:{pass}"));
                headers.insert(
                    reqwest::header::AUTHORIZATION,
                    reqwest::header::HeaderValue::from_str(&format!("Basic {encoded}"))
                        .map_err(|e| format!("Invalid basic auth: {e}"))?,
                );
            }
        }
        // #2052 — an OAuth2 connector must send the bearer token. Leaving the
        // header off made every call anonymous, and the old test_connection
        // counted the resulting 401 as a success.
        AuthType::OAuth2 => {
            headers.insert(
                reqwest::header::AUTHORIZATION,
                oauth2_bearer(auth.access_token.as_deref())?,
            );
        }
        AuthType::None | AuthType::Custom(_) => {}
    }

    if let Some(ref extra) = auth.extra_headers {
        for (name, value) in extra {
            headers.insert(
                reqwest::header::HeaderName::from_bytes(name.as_bytes())
                    .map_err(|e| format!("Invalid header name: {e}"))?,
                reqwest::header::HeaderValue::from_str(value)
                    .map_err(|e| format!("Invalid header value: {e}"))?,
            );
        }
    }

    reqwest::blocking::Client::builder()
        .default_headers(headers)
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {e}"))
}

/// #2052 — the Authorization header for an OAuth2 connector. A missing token is
/// a configuration error, not an anonymous request: the caller must complete
/// the authorization flow (or refresh) first.
fn oauth2_bearer(token: Option<&str>) -> Result<reqwest::header::HeaderValue, String> {
    let token = token
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .ok_or_else(|| {
            "oauth2 access token missing: complete the authorization flow or refresh the grant"
                .to_string()
        })?;
    reqwest::header::HeaderValue::from_str(&format!("Bearer {token}"))
        .map_err(|e| format!("Invalid bearer token: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn auth(auth_type: AuthType) -> AuthConfig {
        AuthConfig {
            auth_type,
            api_key: None,
            api_key_header: None,
            username: None,
            password: None,
            oauth2_client_id: None,
            oauth2_client_secret: None,
            oauth2_token_url: None,
            oauth2_scopes: None,
            base_url: None,
            extra_headers: None,
            access_token: None,
            refresh_token: None,
            token_type: None,
            expires_at: None,
        }
    }

    #[test]
    fn oauth2_sends_the_bearer_token() {
        let mut config = auth(AuthType::OAuth2);
        config.access_token = Some("ya29.token".to_string());
        let client = build_blocking_client(&config).expect("client");
        let headers = client
            .get("https://example.invalid/")
            .build()
            .expect("request")
            .headers()
            .clone();
        assert_eq!(
            headers.get(reqwest::header::AUTHORIZATION).map(|v| v.to_str().ok()),
            Some(Some("Bearer ya29.token"))
        );
    }

    #[test]
    fn oauth2_without_a_token_fails_instead_of_calling_anonymously() {
        let error = build_blocking_client(&auth(AuthType::OAuth2)).expect_err("must fail");
        assert!(error.contains("access token missing"), "{error}");
    }

    #[test]
    fn api_key_and_basic_headers_still_work() {
        let mut api = auth(AuthType::ApiKey);
        api.api_key = Some("k".to_string());
        let client = build_blocking_client(&api).expect("api key client");
        let request = client.get("https://example.invalid/").build().expect("req");
        assert_eq!(
            request.headers().get("X-API-Key").map(|v| v.to_str().ok()),
            Some(Some("k"))
        );

        let mut basic = auth(AuthType::Basic);
        basic.username = Some("u".to_string());
        basic.password = Some("p".to_string());
        let client = build_blocking_client(&basic).expect("basic client");
        let request = client.get("https://example.invalid/").build().expect("req");
        let header = request
            .headers()
            .get(reqwest::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default()
            .to_string();
        assert!(header.starts_with("Basic "), "{header}");
    }
}

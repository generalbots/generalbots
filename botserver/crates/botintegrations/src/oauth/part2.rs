use super::*;

/// GET /api/bots/:bot_id/integrations/oauth/:provider/start
///
/// Redirects the caller to the provider consent screen. Client credentials
/// load strictly from the branch-scoped Vault path; the signed state carries
/// the tenant scope and expires after a short window.
/// Builds the provider consent URL. Pure so the provider-specific extras
/// (Google's offline access, scopes) are unit-testable without a network or a
/// Vault entry.
pub(crate) fn build_authorize_url(
    config: &super::OAuthProviderConfig,
    client_id: &str,
    redirect_uri: &str,
    encoded_state: &str,
    signature: &str,
    provider: &str,
    code_challenge: &str,
) -> String {
    let mut url = format!(
        "{}?client_id={}&response_type=code&redirect_uri={}&state={}.{}",
        config.authorize_url,
        urlencode(client_id),
        urlencode(redirect_uri),
        urlencode(encoded_state),
        urlencode(signature),
    );
    if !config.scopes.is_empty() {
        url.push_str("&scope=");
        url.push_str(&urlencode(config.scopes));
    }
    if !code_challenge.is_empty() {
        url.push_str("&code_challenge=");
        url.push_str(&urlencode(code_challenge));
        url.push_str("&code_challenge_method=S256");
    }
    // Google only returns a refresh token when the consent explicitly asks for
    // offline access; without it the connection dies when the 1-hour access
    // token expires. Microsoft signals it through the offline_access scope.
    if matches!(
        provider,
        "gmail"
            | "drive"
            | "calendar"
            | "google_photos"
            | "google_forms"
            | "google_cloud"
            | "google_workspace_admin"
    ) {
        url.push_str("&access_type=offline&prompt=consent");
    }
    url
}

pub async fn start(
    State(state): State<Arc<IntegrationState>>,
    user: AuthenticatedUser,
    Path((bot_id, provider)): Path<(String, String)>,
    headers: axum::http::HeaderMap,
    Query(query): Query<StartQuery>,
) -> Result<Response, ResponseError> {
    // These two are connected centrally (Drive -> External) and their actions
    // read that shared connection. A per-bot credential here would be a second
    // token store for the same account, silently out of sync with the one the
    // Drive app browses.
    if crate::external_mirror::is_mirrored(&provider) {
        return Err(error_response(StatusCode::CONFLICT, IntegrationError::Validation(
            format!("{provider} is connected from Drive -> External and shared with every bot in this branch")
        )));
    }
    let config = provider_config(&provider).ok_or_else(|| {
        error_response(StatusCode::NOT_FOUND, IntegrationError::Validation("unknown oauth provider".to_string()))
    })?;
    let uuid = parse_uuid(&bot_id)?;
    let scope = resolve_scope(&state.pool, &user, uuid)
        .map_err(|error| error_response(StatusCode::FORBIDDEN, error))?;

    // The OAuth client is resolved bot → branch → platform, so an operator
    // registers the provider app once and every user just clicks Connect.
    let (client_config, vault_path) = state
        .vault
        .load_oauth_client(scope.org_id, scope.branch_id, scope.bot_id, &provider)
        .await
        .map_err(|_| {
            log::warn!("oauth start: no client configuration for provider {provider}");
            error_response(
                StatusCode::PRECONDITION_FAILED,
                IntegrationError::Validation(format!(
                    "no OAuth client registered for {provider}. Register it once at                      integrations/oauth/{provider} (client_id, client_secret) and every user                      can connect with their own account."
                )),
            )
        })?;
    log::info!("oauth start: provider {provider} client loaded from {vault_path}");
    let client_id = client_config
        .get("client_id")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            error_response(StatusCode::PRECONDITION_FAILED, IntegrationError::Validation(
                "client_id missing from the provider oauth envelope".to_string(),
            ))
        })?;

    let base = request_base(&headers).ok_or_else(|| {
        error_response(StatusCode::BAD_REQUEST, IntegrationError::Validation("request host missing".to_string()))
    })?;
    // One stable callback for every bot: the bot id travels inside the
    // signed state, so an operator registers a single redirect URI with the
    // provider (Google/Microsoft do not accept wildcards).
    let redirect_uri = format!("{base}/api/integrations/oauth/{provider}/callback");

    // PKCE: required for public (desktop/loopback) clients and harmless for
    // confidential ones. The verifier rides inside the HMAC-signed state, so a
    // self-hosted install can use a client without a secret.
    let code_verifier = format!(
        "{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    );
    let code_challenge = {
        use base64::Engine as _;
        use sha2::{Digest, Sha256};
        let digest = Sha256::digest(code_verifier.as_bytes());
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(digest)
    };

    let state_payload = json!({
        "cv": code_verifier,
        "u": scope.user_id,
        "o": scope.org_id,
        "br": scope.branch_id,
        "b": scope.bot_id,
        "p": provider,
        "base": base,
        "r": query.return_to,
        "t": chrono::Utc::now().timestamp(),
    });
    let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(state_payload.to_string());
    let signature = sign(&encoded, &signing_key()?)?;

    let url = build_authorize_url(
        &config,
        client_id,
        &redirect_uri,
        &encoded,
        &signature,
        &provider,
        &code_challenge,
    );
    Ok(Redirect::to(&url).into_response())
}

#[cfg(test)]
mod authorize_url_tests {
    use super::{build_authorize_url, provider_config};

    fn url_for(provider: &str) -> String {
        let config = provider_config(provider).expect("provider config");
        build_authorize_url(
            &config,
            "client-123",
            "https://bot.example/api/integrations/oauth/x/callback",
            "state-blob",
            "sig",
            provider,
            "challenge-abc",
        )
    }

    #[test]
    fn gmail_consent_asks_for_offline_access_and_the_read_scope() {
        let url = url_for("gmail");
        assert!(url.starts_with("https://accounts.google.com/o/oauth2/v2/auth?"));
        assert!(url.contains("client_id=client-123"));
        assert!(url.contains("redirect_uri=https%3A%2F%2Fbot.example"));
        assert!(url.contains("gmail.readonly"));
        assert!(url.contains("access_type=offline"));
        assert!(url.contains("code_challenge=challenge-abc"));
        assert!(url.contains("code_challenge_method=S256"));
        assert!(url.contains("prompt=consent"));
    }

    #[test]
    fn onedrive_uses_microsoft_and_offline_scope() {
        let url = url_for("onedrive");
        assert!(url.starts_with("https://login.microsoftonline.com/common/oauth2/v2.0/authorize"));
        assert!(url.contains("graph.microsoft.com"));
        assert!(url.contains("offline_access"));
    }

    #[test]
    fn other_providers_do_not_get_google_offline_params() {
        let url = url_for("hubspot");
        assert!(!url.contains("access_type=offline"));
    }
}

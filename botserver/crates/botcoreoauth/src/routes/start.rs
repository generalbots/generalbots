use super::*;

/// `GET /auth/oauth/:provider?redirect=/where`
///
/// Starts the flow: builds a signed, single-use state bound to this browser
/// (nonce cookie) and redirects to the provider's authorization endpoint.
/// The `redirect` parameter is sanitised here — it never reaches the
/// authorization URL, only the signed state, and it is validated again on the
/// way back.
pub(super) async fn start_oauth(
    State(state): State<Arc<OAuthState_>>,
    Path(provider_name): Path<String>,
    Query(params): Query<OAuthStartParams>,
) -> Response {
    let Some(provider) = OAuthProvider::parse(&provider_name) else {
        return error_page(
            StatusCode::BAD_REQUEST,
            "Invalid OAuth Provider",
            &format!("Provider '{}' is not supported.", provider_name),
        );
    };

    let bot_config = get_bot_config(&state).await;
    let base_url = &state.base_url;

    let config = match load_oauth_config(provider, &bot_config, base_url) {
        Some(c) if c.is_valid() => c,
        _ => {
            warn!("OAuth provider {} is not configured or enabled", provider);
            return error_page(
                StatusCode::SERVICE_UNAVAILABLE,
                "OAuth Provider Not Configured",
                &format!(
                    "Login with {} is not currently enabled.",
                    provider.display_name()
                ),
            );
        }
    };

    let policy = RedirectPolicy::from_base_url(base_url).with_env_allowlist();
    let redirect_after = policy.sanitize(params.redirect.as_deref());

    let oauth_state = OAuthState::new(provider, Some(redirect_after)).with_pkce();
    let signer = server_signer();
    let state_encoded = match signer.sign(&oauth_state) {
        Ok(encoded) => encoded,
        Err(e) => {
            log::error!("Failed to sign the OAuth state: {e}");
            return error_page(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Login Failed",
                "The login flow could not be started.",
            );
        }
    };

    nonce_store::issue(&oauth_state.nonce, now_secs());

    debug!("OAuth state created for provider {}", provider);

    let auth_url = provider.build_auth_url(
        &config,
        &state_encoded,
        oauth_state.pkce_verifier.as_deref(),
    );

    info!("Starting OAuth flow for {} - redirecting to provider", provider);

    let mut response = Redirect::temporary(&auth_url).into_response();
    if let Some(cookie) = cookies::nonce_cookie(&oauth_state.nonce, cookies::LOGIN_NONCE_PATH) {
        response.headers_mut().append(header::SET_COOKIE, cookie);
    } else {
        log::error!("Failed to build the OAuth nonce cookie — aborting the flow");
        return error_page(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Login Failed",
            "The login flow could not be started.",
        );
    }
    response
}

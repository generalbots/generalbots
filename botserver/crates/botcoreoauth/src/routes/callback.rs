use super::*;

/// `GET /auth/oauth/:provider/callback`
///
/// Completes the flow. Nothing from the query string is trusted until the
/// state MAC verifies; the flow additionally requires the nonce cookie that
/// was issued to this browser and a nonce that has never been consumed.
pub(super) async fn oauth_callback(
    State(state): State<Arc<OAuthState_>>,
    Path(provider_name): Path<String>,
    headers: HeaderMap,
    Query(params): Query<OAuthCallbackParams>,
) -> Response {
    if let Some(error) = &params.error {
        let description = params.error_description.as_deref().unwrap_or("Unknown error");
        warn!("OAuth error from provider: {} - {}", error, description);
        return error_page(
            StatusCode::UNAUTHORIZED,
            "Login Failed",
            &format!(
                "The OAuth provider returned an error: {} — {}",
                error, description
            ),
        );
    }

    let Some(code) = &params.code else {
        return error_page(
            StatusCode::BAD_REQUEST,
            "Missing Authorization Code",
            "The provider did not return an authorization code.",
        );
    };

    let Some(state_param) = &params.state else {
        return error_page(
            StatusCode::BAD_REQUEST,
            "Missing State Parameter",
            "The provider did not return the state parameter.",
        );
    };

    let oauth_state = match server_signer().verify(state_param) {
        Ok(oauth_state) => oauth_state,
        Err(error) => {
            warn!("OAuth callback rejected: {error}");
            return state_rejection_page(error);
        }
    };

    let Some(provider) = OAuthProvider::parse(&provider_name) else {
        return error_page(
            StatusCode::BAD_REQUEST,
            "Invalid OAuth Provider",
            "Provider is not supported.",
        );
    };

    if provider != oauth_state.provider {
        warn!(
            "Provider mismatch: URL says {}, state says {}",
            provider, oauth_state.provider
        );
        return error_page(
            StatusCode::BAD_REQUEST,
            "Provider Mismatch",
            "The login state belongs to a different provider.",
        );
    }

    if let Some(rejection) = reject_foreign_state(&headers, &oauth_state) {
        return rejection;
    }

    let bot_config = get_bot_config(&state).await;
    let base_url = &state.base_url;

    let Some(config) = load_oauth_config(provider, &bot_config, base_url) else {
        return error_page(
            StatusCode::SERVICE_UNAVAILABLE,
            "OAuth Provider Not Configured",
            "Login with this provider is not currently enabled.",
        );
    };

    let http_client = reqwest::Client::new();
    let token = match provider
        .exchange_code(&config, code, &http_client, oauth_state.pkce_verifier.as_deref())
        .await
    {
        Ok(t) => t,
        Err(e) => {
            log::error!("Failed to exchange OAuth code: {}", e);
            return error_page(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Login Failed",
                "Failed to complete the OAuth login.",
            );
        }
    };

    let user_info = match provider.fetch_user_info(&token.access_token, &http_client).await {
        Ok(info) => info,
        Err(e) => {
            log::error!("Failed to fetch user info: {}", e);
            return error_page(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Login Failed",
                "Failed to retrieve user information.",
            );
        }
    };

    info!(
        "OAuth login successful for {} user: {} ({})",
        provider,
        user_info.name.as_deref().unwrap_or("unknown"),
        user_info.email.as_deref().unwrap_or("no email")
    );

    let outcome = match create_or_get_oauth_user(&state, &user_info).await {
        Ok(outcome) => outcome,
        Err(e) => {
            log::error!("Failed to create/get user: {}", e);
            return error_page(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Login Failed",
                "Failed to create the user account.",
            );
        }
    };

    let user_id = match outcome {
        OAuthLoginOutcome::LoggedIn(user_id) => user_id,
        OAuthLoginOutcome::LinkRequired { provider } => {
            warn!("OAuth login requires an account link for provider {provider}");
            return error_page(
                StatusCode::FORBIDDEN,
                "Account Link Required",
                "This provider account is not linked to a local account yet. Sign in with \
                 your existing credentials and link the provider from Settings → Accounts.",
            );
        }
    };

    let session_token = match create_user_session(&state, user_id).await {
        Ok(token) => token,
        Err(e) => {
            log::error!("Failed to create session: {}", e);
            return error_page(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Login Failed",
                "Failed to create the session.",
            );
        }
    };

    // Re-validate on the way out: the signed value was sanitised at issuance,
    // and this second pass keeps a compromised/older token from becoming an
    // open redirect.
    let policy = RedirectPolicy::from_base_url(base_url).with_env_allowlist();
    let redirect_url = policy.sanitize(oauth_state.redirect_after.as_deref());

    botlog::safe_debug!("OAuth complete, redirecting to {}", redirect_url);

    let Some(session_cookie) = cookies::session_cookie(&session_token) else {
        log::error!("Failed to build the OAuth session cookie");
        return error_page(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Login Failed",
            "Failed to create the session.",
        );
    };

    let mut response = Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(header::LOCATION, redirect_url)
        .header(header::SET_COOKIE, session_cookie);
    if let Some(clear) = cookies::clear_nonce_cookie(cookies::LOGIN_NONCE_PATH) {
        response = response.header(header::SET_COOKIE, clear);
    }
    match response.body(axum::body::Body::empty()) {
        Ok(response) => response,
        Err(e) => {
            log::error!("Failed to build OAuth redirect response: {}", e);
            error_page(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Login Failed",
                "Failed to complete the login.",
            )
        }
    }
}

/// Maps a state verification failure to the response the browser receives.
///
/// A signature mismatch is a forgery attempt (`403`); a malformed or expired
/// token is a broken/abandoned flow (`400`). Neither ever reaches the identity
/// code.
fn state_rejection_page(error: StateError) -> Response {
    match error {
        StateError::BadSignature => error_page(
            StatusCode::FORBIDDEN,
            "Invalid State",
            "The login state was not issued by this server.",
        ),
        StateError::Expired => error_page(
            StatusCode::BAD_REQUEST,
            "Session Expired",
            "The login took too long. Please start again.",
        ),
        StateError::Malformed => error_page(
            StatusCode::BAD_REQUEST,
            "Invalid State",
            "The login state is malformed.",
        ),
    }
}

/// Rejects a state that this browser did not start, and one that was already
/// used. Returns the response to send when the flow must abort.
///
/// The nonce cookie is `SameSite=Lax` (not `Strict`) precisely so it survives
/// the provider's cross-site top-level redirect back to us; without it the
/// callback could be triggered by any browser, which is the login-CSRF case.
fn reject_foreign_state(headers: &HeaderMap, oauth_state: &OAuthState) -> Option<Response> {
    match binding::consume_bound_nonce(headers, &oauth_state.nonce) {
        Ok(()) => None,
        Err(e) => {
            warn!("OAuth callback rejected: {e}");
            Some(error_page(
                StatusCode::FORBIDDEN,
                "Invalid State",
                "This login was not started in this browser, or the link was already used. \
                 Please start again.",
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_forged_state_is_a_403_and_a_broken_one_is_a_400() {
        // A bad MAC is an authenticity failure: forbidden, never a login.
        assert_eq!(
            state_rejection_page(StateError::BadSignature).status(),
            StatusCode::FORBIDDEN
        );
        // Expired/malformed tokens are a stale or broken flow, not an attack
        // signature: bad request.
        assert_eq!(
            state_rejection_page(StateError::Expired).status(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            state_rejection_page(StateError::Malformed).status(),
            StatusCode::BAD_REQUEST
        );
    }
}

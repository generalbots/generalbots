use super::*;

/// Mounts the OAuth login endpoints.
///
/// * `GET /auth/oauth/providers` — which providers are configured.
/// * `GET /auth/oauth/:provider` — starts a flow (signed state + browser nonce).
/// * `GET /auth/oauth/:provider/callback` — verifies state/nonce and logs in.
pub fn configure(state: Arc<OAuthState_>) -> Router {
    Router::new()
        .route("/auth/oauth/providers", get(list_providers))
        .route("/auth/oauth/:provider", get(start_oauth))
        .route("/auth/oauth/:provider/callback", get(oauth_callback))
        .with_state(state)
}

pub(super) async fn list_providers(State(state): State<Arc<OAuthState_>>) -> impl IntoResponse {
    let bot_config = get_bot_config(&state).await;
    let base_url = &state.base_url;

    let enabled = get_enabled_providers(&bot_config, base_url);

    let providers: Vec<ProviderInfo> = enabled
        .iter()
        .map(|config| ProviderInfo {
            id: config.provider.to_string().to_lowercase(),
            name: config.provider.display_name().to_string(),
            icon: config.provider.icon().to_string(),
            login_url: format!("/auth/oauth/{}", config.provider.to_string().to_lowercase()),
        })
        .collect();

    Json(EnabledProvidersResponse { providers })
}

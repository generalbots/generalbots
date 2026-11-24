//! #2915 — acceptance tests for the deploy-gateway auth boundary that pairs
//! with the loopback-by-default bind policy. These prove the issue's "verify"
//! requirements: unauthenticated requests get 401, and key rotation is refused
//! truthfully with 501 (never a lying success that pretends to rotate).

use super::*;

const KEY: &str = "aB3dE7gH1jK4mN8pQ2rS6tV9xZ5cF0wL";

fn state_with_key(key: &str) -> GatewayState {
    GatewayState {
        deploy_keys: vec![key.to_string()],
        key_from_config: true,
        incus_socket: "/var/lib/incus/unix.socket".to_string(),
        caddy_api_url: "http://127.0.0.1:2019".to_string(),
        sites_root: std::env::temp_dir().to_string_lossy().to_string(),
    }
}

fn headers_with_key(key: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::HeaderName::from_static("x-deploy-key"),
        key.parse::<axum::http::HeaderValue>()
            .expect("valid header value"),
    );
    headers
}

#[test]
fn unauthenticated_request_is_rejected_with_401() {
    let state = state_with_key(KEY);
    // No X-Deploy-Key header at all.
    assert_eq!(
        validate_deploy_key(&HeaderMap::new(), &state),
        Err(StatusCode::UNAUTHORIZED)
    );
    // A wrong key is indistinguishable from no key.
    assert_eq!(
        validate_deploy_key(
            &headers_with_key("wrongKey00000000000000000000000000"),
            &state
        ),
        Err(StatusCode::UNAUTHORIZED)
    );
    // A process with no configured key accepts nobody (fail-closed key is
    // unguessable and never present in an incoming header).
    let no_key_state = state_with_key("");
    assert_eq!(
        validate_deploy_key(&HeaderMap::new(), &no_key_state),
        Err(StatusCode::UNAUTHORIZED)
    );
}

#[test]
fn configured_key_is_accepted() {
    let state = state_with_key(KEY);
    assert!(validate_deploy_key(&headers_with_key(KEY), &state).is_ok());
}

#[tokio::test]
async fn rotation_is_refused_with_501_and_unauthenticated_stays_401() {
    let state = Arc::new(state_with_key(KEY));
    let registry = Arc::new(ContainerRegistry::default());

    // Authenticated: rotation is truthfully unimplemented — it neither lies
    // with success nor silently rotates/invalidates the configured key.
    let authenticated = gateway_rotate_key(
        State((state.clone(), registry.clone())),
        headers_with_key(KEY),
    )
    .await;
    assert!(matches!(authenticated, Err(StatusCode::NOT_IMPLEMENTED)));

    // Unauthenticated: 401, never reaches the rotation handler body.
    let unauthenticated = gateway_rotate_key(State((state, registry)), HeaderMap::new()).await;
    assert!(matches!(unauthenticated, Err(StatusCode::UNAUTHORIZED)));
}

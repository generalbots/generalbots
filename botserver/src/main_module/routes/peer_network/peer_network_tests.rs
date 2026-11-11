//! Unit tests for the peer-network HTTP surface (#2141).
//!
//! Split out of `peer_network.rs` to keep that module under the 450-line
//! budget. The auth gates are private to the parent module, so the tests reach
//! them through `super::*`.

use super::*;
use crate::security::auth_api::types::Role;

const PEER_TOKEN: &str = "peer-node-token-with-enough-entropy";

fn disabled_network_with_token(token: Option<&str>) -> PeerNetwork {
    let mut config = peer_network::PeerConfig::disabled();
    config.token = token.map(str::to_string);
    PeerNetwork::new(config)
}

fn headers_with(value: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(header::AUTHORIZATION, value.parse().expect("header"));
    headers
}

fn peer_headers(value: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("x-peer-token", value.parse().expect("header"));
    headers
}

fn admin() -> AuthenticatedUser {
    AuthenticatedUser::new(uuid::Uuid::new_v4(), "root".to_string()).with_role(Role::Admin)
}

fn regular_user() -> AuthenticatedUser {
    AuthenticatedUser::new(uuid::Uuid::new_v4(), "alice".to_string())
}

/// #2141 — the old behaviour (`token = None` ⇒ allow) is gone: an
/// unconfigured peer network fails closed instead of accepting payloads.
#[test]
fn unconfigured_peer_network_is_not_open() {
    let network = disabled_network_with_token(None);

    assert_eq!(
        authorize_machine(&HeaderMap::new(), &network),
        Err(PeerError::Unconfigured)
    );
    assert_eq!(
        authorize_machine(&peer_headers(PEER_TOKEN), &network),
        Err(PeerError::Unconfigured)
    );
    assert_eq!(
        authorize_control(&HeaderMap::new(), &network, None),
        Err(PeerError::Unconfigured)
    );
    // The real HTTP path always carries a principal: an anonymous caller on an
    // unconfigured node gets the same 503, never an open door.
    assert_eq!(
        authorize_control(
            &HeaderMap::new(),
            &network,
            Some(&AuthenticatedUser::anonymous())
        ),
        Err(PeerError::Unconfigured)
    );
}

/// A machine call without the shared token is 401, never a silent pass.
#[test]
fn an_unauthenticated_peer_call_is_rejected() {
    let network = disabled_network_with_token(Some(PEER_TOKEN));

    assert_eq!(
        authorize_machine(&HeaderMap::new(), &network),
        Err(PeerError::Unauthorized)
    );
    assert_eq!(
        authorize_machine(&headers_with("Bearer wrong"), &network),
        Err(PeerError::Unauthorized)
    );
    assert_eq!(
        authorize_machine(&headers_with(&format!("Bearer {PEER_TOKEN}")), &network),
        Ok(())
    );
    assert_eq!(
        authorize_machine(&peer_headers(PEER_TOKEN), &network),
        Ok(())
    );
}

/// `status`/`enable`/`disable` accept a peer token or an admin session;
/// a plain authenticated user is forbidden.
#[test]
fn control_routes_require_an_admin_without_a_peer_token() {
    let network = disabled_network_with_token(None);
    assert_eq!(
        authorize_control(&HeaderMap::new(), &network, Some(&regular_user())),
        Err(PeerError::Forbidden)
    );
    assert_eq!(
        authorize_control(&HeaderMap::new(), &network, Some(&admin())),
        Ok(())
    );
    // Single-user desktop (`no_security=true`): the local owner may flip
    // the switch even though the principal is not an administrator.
    assert_eq!(
        authorize_control(
            &HeaderMap::new(),
            &network,
            Some(&AuthenticatedUser::desktop_local())
        ),
        Ok(())
    );

    let network = disabled_network_with_token(Some(PEER_TOKEN));
    assert_eq!(
        authorize_control(&peer_headers(PEER_TOKEN), &network, None),
        Ok(())
    );
    assert_eq!(
        authorize_control(&HeaderMap::new(), &network, Some(&admin())),
        Ok(())
    );
    assert_eq!(
        authorize_control(&HeaderMap::new(), &network, None),
        Err(PeerError::Unauthorized)
    );
}

/// A JWT in `Authorization` is not a peer credential: only the configured
/// token opens a machine route.
#[test]
fn a_user_token_is_not_a_peer_token() {
    let network = disabled_network_with_token(Some(PEER_TOKEN));
    assert_eq!(
        authorize_machine(
            &headers_with("Bearer eyJhbGciOiJIUzI1NiJ9.e30.sig"),
            &network
        ),
        Err(PeerError::Unauthorized)
    );
}

#[test]
fn peer_token_comparison_is_length_safe() {
    assert!(peer_token_matches("same", "same"));
    assert!(!peer_token_matches("same", "samf"));
    assert!(!peer_token_matches("short", "longer"));
}

/// The wire status codes the gates produce: an unconfigured node is 503 (not
/// open), an unauthenticated caller is 401, a non-admin is 403.
#[test]
fn auth_failures_map_to_the_documented_status_codes() {
    let locale = botcore::i18n::locale_from_headers(&HeaderMap::new());

    assert_eq!(
        error_response(&locale, PeerError::Unconfigured).status(),
        StatusCode::SERVICE_UNAVAILABLE
    );
    assert_eq!(
        error_response(&locale, PeerError::Unauthorized).status(),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        error_response(&locale, PeerError::Forbidden).status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        error_response(&locale, PeerError::Disabled).status(),
        StatusCode::FORBIDDEN
    );
}

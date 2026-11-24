//! Target-policy tests for the shared outbound fetch loop (#2131, #2618).
//!
//! Split out of `ssrf_fetch.rs` to keep that file under the 450-line ceiling.
//! The body-cap tests stay inline next to the cap helpers they exercise.

use super::ssrf_fetch::{
    check_connected_peer, fetch_internal_service_url, next_hop, FetchPolicy, Target,
};
use super::ssrf_guard::{resolve_checked, SsrfErrorKind};
use std::net::SocketAddr;
use std::time::Duration;
use url::Url;

fn policy() -> FetchPolicy {
    FetchPolicy::default().with_timeout(Duration::from_millis(900))
}

#[tokio::test]
async fn internal_fetch_refuses_a_loopback_literal() {
    // 127.0.0.1 is an allowlisted internal host, but the resolution check
    // must still refuse it as non-global (#2618).
    let err = fetch_internal_service_url("http://127.0.0.1:18080/", &policy())
        .await
        .expect_err("loopback literal must be refused");
    assert_eq!(err.kind(), SsrfErrorKind::BlockedAddress);
}

#[tokio::test]
async fn internal_fetch_refuses_allowlisted_hostname_resolving_to_loopback() {
    // `localhost` is allowlisted as a host; only resolution reveals that
    // it points at the local node.
    let err = fetch_internal_service_url("http://localhost:18080/", &policy())
        .await
        .expect_err("localhost must be refused after resolution");
    assert_eq!(err.kind(), SsrfErrorKind::BlockedAddress);
}

#[tokio::test]
async fn shared_resolution_check_refuses_private_and_metadata_addresses() {
    for target in [
        "http://10.0.0.1/",
        "http://192.168.1.10/",
        "http://169.254.169.254/latest/meta-data/",
        "http://[fe80::1]/",
    ] {
        let parsed = Url::parse(target).expect("literal fixture");
        let err = resolve_checked(&parsed).await.expect_err("non-global IP");
        // An unscoped IPv6 link-local literal can fail resolution before
        // the classifier runs on some resolvers. Both outcomes refuse the
        // request, which is the security property under test.
        assert!(
            matches!(
                err.kind(),
                SsrfErrorKind::BlockedAddress | SsrfErrorKind::ResolutionFailed
            ),
            "{target} was not refused: {:?}",
            err.kind()
        );
    }
}

#[test]
fn post_connect_peer_check_refuses_non_global_addresses() {
    for peer in [
        "127.0.0.1:443",
        "10.0.0.1:443",
        "169.254.169.254:80",
        "[::1]:443",
    ] {
        let peer: SocketAddr = peer.parse().expect("peer literal");
        let err = check_connected_peer(peer).expect_err("non-global peer");
        assert_eq!(err.kind(), SsrfErrorKind::DnsRebinding);
    }
    let public: SocketAddr = "8.8.8.8:443".parse().expect("public peer");
    assert!(check_connected_peer(public).is_ok());
}

#[test]
fn internal_redirect_to_a_caller_controlled_url_is_refused() {
    // `follow` re-runs `next_hop` on every redirect hop, so a redirect from
    // an internal host to an arbitrary URL is refused by the host
    // allowlist before any second connection is made.
    let base = Url::parse("http://vault:8200/start").expect("base fixture");
    for location in [
        "https://attacker.example/collect?token=stolen",
        "http://169.254.169.254/latest/meta-data/",
        "file:///etc/passwd",
    ] {
        let err = next_hop(&base, location, Target::InternalService)
            .expect_err("caller-controlled redirect must be refused");
        assert!(
            matches!(
                err.kind(),
                SsrfErrorKind::BlockedHost
                    | SsrfErrorKind::UnsupportedScheme
                    | SsrfErrorKind::MissingHost
            ),
            "{location} refused as {:?}",
            err.kind()
        );
    }
}

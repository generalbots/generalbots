//! SSRF fetch-policy matrix (#2131).
//!
//! The guard's whole value is that it refuses the targets an attacker picks
//! and still lets a normal public page through. These tests pin that matrix
//! offline: address classification, URL shape, redirect re-validation, DNS
//! resolution and the internal-service allowlist. The only "network" touched
//! is `localhost` resolution through `/etc/hosts`; no test leaves the host.

use super::ssrf_allowlist::{internal_service_reason, is_internal_service_host, INTERNAL_SERVICES};
use super::ssrf_fetch::{fetch_internal_service_url, fetch_user_url, FetchPolicy};
use super::ssrf_guard::{
    is_blocked_address, resolve_checked, validate_redirect_target, validate_url_ssrf_strict,
    SsrfErrorKind, BLOCKED_PORTS,
};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::time::Duration;
use url::Url;

fn v4(s: &str) -> IpAddr {
    IpAddr::V4(s.parse::<Ipv4Addr>().expect("ipv4 literal"))
}

fn v6(s: &str) -> IpAddr {
    IpAddr::V6(s.parse::<Ipv6Addr>().expect("ipv6 literal"))
}

fn url(s: &str) -> Url {
    Url::parse(s).expect("valid fixture URL")
}

fn reject(url: &str) -> SsrfErrorKind {
    validate_url_ssrf_strict(url)
        .expect_err("URL must be refused")
        .kind()
}

#[test]
fn cloud_metadata_is_refused() {
    assert_eq!(
        reject("http://169.254.169.254/latest/meta-data/iam/security-credentials/"),
        SsrfErrorKind::BlockedAddress
    );
    assert_eq!(
        reject("http://metadata.google.internal/computeMetadata/v1/"),
        SsrfErrorKind::BlockedHost
    );
    assert_eq!(
        reject("http://instance-data/latest/meta-data/"),
        SsrfErrorKind::BlockedHost
    );
}

#[test]
fn localhost_and_loopback_are_refused() {
    for target in [
        "http://localhost/",
        "http://localhost:3000/admin",
        "http://LOCALHOST./",
        "http://127.0.0.1:8080/api/product",
        "http://127.255.255.254/",
        "http://[::1]/",
        "http://[::ffff:127.0.0.1]/",
        "http://app.localhost/",
        "http://vault.internal/",
    ] {
        let kind = reject(target);
        assert!(
            matches!(
                kind,
                SsrfErrorKind::BlockedHost | SsrfErrorKind::BlockedAddress
            ),
            "{target} refused as {kind:?}"
        );
    }
}

#[test]
fn private_and_special_ranges_are_refused() {
    for target in [
        "http://10.0.0.1/",
        "http://10.255.255.254/",
        "http://172.16.0.1/",
        "http://192.168.1.10/",
        "http://100.64.0.1/",
        "http://0.0.0.0/",
        "http://[fd00::1]/",
        "http://[fe80::1]/",
        "http://224.0.0.1/",
        "http://240.0.0.1/",
    ] {
        assert_eq!(
            reject(target),
            SsrfErrorKind::BlockedAddress,
            "{target} must be refused"
        );
    }
}

#[test]
fn encoded_and_short_ipv4_forms_are_refused() {
    // The `url` crate normalises decimal/hex/short forms into dotted-decimal,
    // so these must be refused by the address check or the obfuscation check.
    for target in [
        "http://0x7f000001/",
        "http://2130706433/",
        "http://017700000001/",
        "http://127.1/",
        "http://0177.0.0.1/",
    ] {
        let kind = reject(target);
        assert!(
            matches!(
                kind,
                SsrfErrorKind::BlockedAddress
                    | SsrfErrorKind::ObfuscatedAddress
                    | SsrfErrorKind::BlockedHost
            ),
            "{target} refused as {kind:?}"
        );
    }
}

#[test]
fn non_http_schemes_and_credentials_are_refused() {
    assert_eq!(
        reject("file:///etc/passwd"),
        SsrfErrorKind::UnsupportedScheme
    );
    assert_eq!(
        reject("gopher://example.com:70/"),
        SsrfErrorKind::UnsupportedScheme
    );
    assert_eq!(
        reject("ftp://example.com/"),
        SsrfErrorKind::UnsupportedScheme
    );
    assert_eq!(
        reject("javascript:alert(1)"),
        SsrfErrorKind::UnsupportedScheme
    );
    assert_eq!(
        reject("https://user:pass@example.com/"),
        SsrfErrorKind::CredentialsInUrl
    );
    assert_eq!(
        reject("https://user@example.com/"),
        SsrfErrorKind::CredentialsInUrl
    );
}

#[test]
fn control_plane_ports_are_refused() {
    for port in BLOCKED_PORTS {
        let target = format!("http://example.com:{port}/");
        assert_eq!(
            reject(&target),
            SsrfErrorKind::BlockedPort,
            "port {port} must be refused"
        );
    }
    assert!(validate_url_ssrf_strict("https://example.com:8443/health").is_ok());
}

#[test]
fn a_public_url_is_allowed() {
    for target in [
        "https://example.com/",
        "https://example.com/docs/page?q=1#section",
        "http://example.com/feed.xml",
        "https://sub.example.co.uk/a/b",
        "https://8.8.8.8/",
        "https://[2606:4700:4700::1111]/",
    ] {
        let parsed = validate_url_ssrf_strict(target).unwrap_or_else(|err| {
            panic!("{target} must be allowed: {}", err.detail());
        });
        assert!(matches!(parsed.scheme(), "http" | "https"));
    }
}

#[test]
fn a_redirect_to_a_private_target_is_refused() {
    let base = url("https://example.com/start");
    for location in [
        "http://127.0.0.1:8080/admin",
        "http://169.254.169.254/latest/meta-data/",
        "http://10.1.2.3/",
        "http://[::1]/",
        "file:///etc/passwd",
        "gopher://example.com/",
        "//localhost/secret",
    ] {
        let err = validate_redirect_target(&base, location)
            .expect_err("redirect must be refused");
        assert!(
            matches!(
                err.kind(),
                SsrfErrorKind::BlockedAddress
                    | SsrfErrorKind::BlockedHost
                    | SsrfErrorKind::UnsupportedScheme
            ),
            "{location} refused as {:?}",
            err.kind()
        );
    }
}

#[test]
fn a_redirect_to_another_public_url_is_allowed() {
    let base = url("https://example.com/start");
    let next = validate_redirect_target(&base, "/moved?x=1").expect("relative redirect");
    assert_eq!(next.as_str(), "https://example.com/moved?x=1");
    let absolute = validate_redirect_target(&base, "https://cdn.example.net/asset").expect("absolute");
    assert_eq!(absolute.host_str(), Some("cdn.example.net"));
}

#[tokio::test]
async fn a_hostname_resolving_to_loopback_is_refused() {
    // `localhost` resolves through /etc/hosts to 127.0.0.1 / ::1. Resolving it
    // directly proves the address check, independently of the hostname list.
    let err = resolve_checked(&url("http://localhost:80/"))
        .await
        .expect_err("localhost must resolve to a blocked address");
    assert_eq!(err.kind(), SsrfErrorKind::BlockedAddress);
}

#[tokio::test]
async fn resolution_of_a_literal_ip_does_not_need_dns() {
    let addrs = resolve_checked(&url("https://8.8.8.8/"))
        .await
        .expect("public literal");
    assert_eq!(addrs, vec![v4("8.8.8.8")]);
}

#[tokio::test]
async fn fetch_refuses_a_loopback_target_before_connecting() {
    let err = fetch_user_url("http://127.0.0.1:18080/", &FetchPolicy::default())
        .await
        .expect_err("loopback must be refused");
    assert_eq!(err.kind(), SsrfErrorKind::BlockedAddress);
    // The user-facing message never echoes the host.
    assert!(!err.to_string().contains("127.0.0.1"));
}

#[tokio::test]
async fn fetch_refuses_a_metadata_target() {
    let err = fetch_user_url("http://169.254.169.254/latest/meta-data/", &FetchPolicy::default())
        .await
        .expect_err("metadata must be refused");
    assert_eq!(err.kind(), SsrfErrorKind::BlockedAddress);
}

#[tokio::test]
async fn fetch_refuses_a_file_url() {
    let err = fetch_user_url("file:///etc/passwd", &FetchPolicy::default())
        .await
        .expect_err("file scheme must be refused");
    assert_eq!(err.kind(), SsrfErrorKind::UnsupportedScheme);
}

#[tokio::test]
async fn internal_fetch_refuses_a_public_host() {
    let err = fetch_internal_service_url("https://example.com/", &FetchPolicy::default())
        .await
        .expect_err("public host is not an internal service");
    assert_eq!(err.kind(), SsrfErrorKind::BlockedHost);
}

#[test]
fn the_internal_allowlist_is_exact_and_documented() {
    assert!(is_internal_service_host("vault"));
    assert!(is_internal_service_host("VAULT."));
    assert!(is_internal_service_host("127.0.0.1"));
    assert!(!is_internal_service_host("vault.evil.com"));
    assert!(!is_internal_service_host("not-a-service"));
    assert!(internal_service_reason("drive").is_some());
    for service in INTERNAL_SERVICES {
        assert!(
            !service.reason.trim().is_empty(),
            "{} needs a documented reason",
            service.host
        );
    }
}

#[test]
fn address_classifier_covers_the_ranges_std_misses() {
    for addr in [
        v4("100.64.0.1"),
        v4("198.18.0.1"),
        v4("192.0.0.5"),
        v4("240.0.0.1"),
        v6("ff02::1"),
        v6("2001:db8::1"),
        v6("64:ff9b::7f00:1"),
    ] {
        assert!(is_blocked_address(addr), "{addr} must be blocked");
    }
    for addr in [v4("8.8.8.8"), v4("1.1.1.1"), v6("2606:4700:4700::1111")] {
        assert!(!is_blocked_address(addr), "{addr} must be allowed");
    }
}

#[test]
fn the_fetch_policy_defaults_are_bounded() {
    let policy = FetchPolicy::default();
    assert!(policy.max_bytes <= 8 * 1024 * 1024);
    assert!(policy.timeout <= Duration::from_secs(60));
    assert!(policy.max_redirects <= 10);
    assert!(!policy.user_agent.is_empty());
}

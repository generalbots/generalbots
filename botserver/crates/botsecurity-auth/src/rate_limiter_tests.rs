use super::*;
use std::net::{IpAddr, Ipv4Addr};
#[test]
fn test_http_config_presets() {
    let default = HttpRateLimitConfig::default();
    assert_eq!(default.requests_per_second, 100);
    assert_eq!(default.burst_size, 200);

    let strict = HttpRateLimitConfig::strict();
    assert_eq!(strict.requests_per_second, 10);
    assert_eq!(strict.burst_size, 10);

    let relaxed = HttpRateLimitConfig::relaxed();
    assert_eq!(relaxed.requests_per_second, 500);

    let api = HttpRateLimitConfig::api();
    assert_eq!(api.requests_per_second, 100);
    assert_eq!(api.burst_size, 150);
}

#[test]
fn test_combined_limiter_creation() {
    let limiter = CombinedRateLimiter::with_defaults();
    assert!(limiter.check_http_limit());
}

#[test]
fn test_combined_limiter_clone() {
    let limiter = CombinedRateLimiter::with_defaults();
    let cloned = limiter.clone();
    assert!(cloned.check_http_limit());
}

#[tokio::test]
async fn test_user_rate_limit() {
    let limiter = CombinedRateLimiter::with_defaults();
    assert!(limiter.check_user_limit("test-user").await.is_ok());
}
#[test]
fn test_auth_rate_limit_rejects_after_burst_for_one_client() {
    let limiter = CombinedRateLimiter::with_defaults();
    let mut rejected = 0;
    for _ in 0..12 {
        if !limiter.check_auth_limit("198.51.100.7") {
            rejected += 1;
        }
    }
    assert!(rejected > 0, "the 11th attempt must be throttled");
    assert!(
        limiter.check_auth_limit("198.51.100.8"),
        "a different client must keep its own budget"
    );
}

#[test]
fn test_client_key_ignores_forwarded_for_from_untrusted_peer() {
    let request = Request::builder()
        .header("x-forwarded-for", "203.0.113.9, 10.0.0.1")
        .header("x-real-ip", "203.0.113.10")
        .body(Body::empty())
        .expect("request builds");
    let peer = SocketAddr::from((Ipv4Addr::new(198, 51, 100, 20), 12345));
    assert_eq!(client_key(&request, Some(peer)), "198.51.100.20");
}

#[test]
fn test_client_key_falls_back_to_peer_without_trusted_proxy() {
    let request = Request::builder().body(Body::empty()).expect("request builds");
    let peer = SocketAddr::from((Ipv4Addr::new(198, 51, 100, 30), 12345));
    assert_eq!(client_key(&request, Some(peer)), "198.51.100.30");
}
#[test]
fn test_forwarded_client_ip_picks_rightmost_untrusted() {
    let request = Request::builder()
        .header("x-forwarded-for", "203.0.113.9, 198.51.100.44, 10.0.0.5")
        .body(Body::empty())
        .expect("request builds");
    let trusted = HashSet::from([
        IpAddr::V4(Ipv4Addr::new(10, 0, 0, 5)),
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
    ]);
    assert_eq!(
        forwarded_client_ip(&request, &trusted),
        Some(IpAddr::V4(Ipv4Addr::new(198, 51, 100, 44)))
    );
}

#[test]
fn test_spoofed_xff_from_untrusted_peer_still_limited_by_peer_ip() {
    let limiter = CombinedRateLimiter::with_defaults();
    let peer = SocketAddr::from((Ipv4Addr::new(198, 51, 100, 77), 4444));
    let mut rejected = 0;
    for attempt in 1..=11 {
        let request = Request::builder()
            .uri("/api/auth/login")
            .header("x-forwarded-for", format!("203.0.113.{attempt}"))
            .body(Body::empty())
            .expect("request builds");
        let key = client_key(&request, Some(peer));
        assert_eq!(key, "198.51.100.77");
        if !limiter.check_auth_limit(&key) {
            rejected += 1;
        }
    }
    assert_eq!(rejected, 1, "only the 11th attempt must be rejected");
}
#[test]
fn test_parse_trusted_proxies_ignores_invalid_entries() {
    let parsed = parse_trusted_proxies("10.0.0.1, not-an-ip, ::1,  ");
    assert!(parsed.contains(&IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))));
    assert!(parsed.contains(&"::1".parse::<IpAddr>().expect("valid ipv6")));
    assert_eq!(parsed.len(), 2);
}

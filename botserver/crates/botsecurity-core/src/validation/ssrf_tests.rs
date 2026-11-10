//! SSRF guard matrix (#2066).
//!
//! `validate_url_ssrf` is the only check between a user-supplied URL and an
//! outbound request in several crates. These tests pin both the public
//! accept/reject behaviour and the address classifier, including the
//! IPv4-mapped / unique-local / link-local forms that used to slip through.

use super::validate_url_ssrf::is_blocked_ip;
use super::*;
use std::net::{Ipv4Addr, Ipv6Addr};

fn v4(s: &str) -> IpAddr {
    IpAddr::V4(s.parse::<Ipv4Addr>().expect("ipv4"))
}

fn v6(s: &str) -> IpAddr {
    IpAddr::V6(s.parse::<Ipv6Addr>().expect("ipv6"))
}

#[test]
fn every_non_global_address_class_is_blocked() {
    let blocked = [
        v4("127.0.0.1"),
        v4("127.255.255.254"),
        v4("10.0.0.1"),
        v4("10.255.255.254"),
        v4("172.16.0.1"),
        v4("172.31.255.254"),
        v4("192.168.0.1"),
        v4("169.254.1.1"),
        v4("0.0.0.0"),
        v4("255.255.255.255"),
        v4("192.0.2.1"),
        v6("::1"),
        v6("::"),
        v6("fc00::1"),
        v6("fd12:3456:789a::1"),
        v6("fe80::1"),
        v6("feb0::1"),
        v6("::ffff:127.0.0.1"),
        v6("::ffff:10.0.0.1"),
        v6("::ffff:192.168.1.1"),
    ];
    for addr in blocked {
        assert!(is_blocked_ip(addr), "{addr} must be blocked");
    }
}

#[test]
fn globally_routable_addresses_are_allowed() {
    let allowed = [
        v4("8.8.8.8"),
        v4("1.1.1.1"),
        v4("172.32.0.1"),
        v4("192.169.0.1"),
        v6("2606:4700:4700::1111"),
        v6("2001:4860:4860::8888"),
        v6("::ffff:8.8.8.8"),
    ];
    for addr in allowed {
        assert!(!is_blocked_ip(addr), "{addr} must be allowed");
    }
}

#[test]
fn the_public_entry_point_blocks_internal_targets() {
    let blocked = [
        "https://127.0.0.1/",
        "https://127.0.0.1:8080/admin",
        "http://10.1.2.3/",
        "https://172.16.5.4/",
        "https://192.168.1.10/",
        "https://169.254.169.254/latest/meta-data",
        "https://0.0.0.0/",
        "https://localhost/",
        "https://localhost:3000/",
        "https://metadata.google.internal/computeMetadata/v1",
        "https://instance-data/",
        "https://[::1]/",
        "https://[::ffff:127.0.0.1]/",
    ];
    for url in blocked {
        assert!(
            validate_url_ssrf(url).is_err(),
            "{url} must be refused"
        );
    }
}

#[test]
fn public_targets_pass_the_public_entry_point() {
    for url in [
        "https://example.com/",
        "https://example.com/path?q=1",
        "https://8.8.8.8/",
        "https://172.32.0.1/",
        "https://sub.example.co.uk/a/b",
    ] {
        assert!(validate_url_ssrf(url).is_ok(), "{url} must be accepted");
    }
}

#[test]
fn malformed_and_non_http_urls_are_refused() {
    let oversized = format!("https://example.com/{}", "a".repeat(2048));
    for url in [
        "",
        "not-a-url",
        "ftp://example.com/",
        "file:///etc/passwd",
        "javascript:alert(1)",
        "https://",
        "https://example.com/a b",
        oversized.as_str(),
    ] {
        assert!(validate_url_ssrf(url).is_err(), "{url:?} must be refused");
    }
}

#[test]
fn a_forbidden_error_is_returned_for_internal_targets() {
    let err = validate_url_ssrf("https://10.1.2.3/").expect_err("must be refused");
    assert!(matches!(err, ValidationError::Forbidden { .. }));
}

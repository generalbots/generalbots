//! #2069 CLD-9 — whether the directory URL may carry a password over plain
//! http. Every case here is a perimeter decision, so each is explicit.
use super::*;

#[test]
fn cld9_https_is_always_allowed() {
    for url in [
        "https://directory.example.com",
        "https://directory.example.com:8443/path",
        "https://127.0.0.1:8200",
        "https://localhost",
    ] {
        assert!(directory_url_allows_password(url, false), "{url}");
    }
}

#[test]
fn cld9_http_is_allowed_on_loopback_private_and_link_local_addresses() {
    for url in [
        "http://localhost:8080",
        "http://127.0.0.1:8080",
        "http://127.1.2.3",
        "http://10.1.2.3:8080",
        "http://172.16.0.5",
        "http://192.168.1.10",
        "http://169.254.169.254",
        "http://[::1]:3000",
        "http://[fd00::1]",
        "http://[fe80::1]",
    ] {
        assert!(directory_url_allows_password(url, false), "{url} must be trusted");
    }
}

#[test]
fn cld9_http_is_refused_on_public_addresses_and_dns_names() {
    for url in [
        "http://8.8.8.8",
        "http://203.0.113.10:8080",
        "http://directory.example.com",
        "http://[2606:4700::1111]",
    ] {
        assert!(!directory_url_allows_password(url, false), "{url} must not carry a password");
    }
}

#[test]
fn cld9_a_dns_name_that_merely_starts_with_a_loopback_prefix_is_public() {
    // Regression: `starts_with("127.")` treated these as private and allowed
    // a cleartext password to a host the operator does not control.
    for url in [
        "http://127.0.0.1.attacker.example",
        "http://127.example.com",
        "http://10.0.0.1.attacker.example",
    ] {
        assert!(!directory_url_allows_password(url, false), "{url} must be public");
    }
}

#[test]
fn cld9_allow_insecure_http_overrides_every_check() {
    assert!(directory_url_allows_password("http://directory.example.com", true));
    assert!(directory_url_allows_password("https://directory.example.com", true));
}

#[test]
fn cld9_unparseable_or_scheme_less_input_is_refused() {
    for url in ["", "not a url", "directory.example.com", "ftp://directory.example.com"] {
        assert!(!directory_url_allows_password(url, false), "{url:?}");
    }
}

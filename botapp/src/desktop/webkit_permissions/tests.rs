//! Offline policy tests for the WebKit user-media permission handler (#2504).
//! No WebView, no environment and no hardware are needed: every case exercises
//! the pure decision function extracted from the permission callback.

use super::{
    decide, origin_allowed, origin_of, redacted, split_authority, BUNDLED_ORIGINS, DEV_ORIGIN,
    MediaDecision,
};

#[test]
fn test_bundled_origins_grant_audio_and_video() {
    for origin in BUNDLED_ORIGINS {
        let page = format!("{origin}/suite/desktop.html?q=1#frag");
        assert_eq!(
            decide(Some(&page), true, false, &[], false),
            MediaDecision::Grant
        );
        assert_eq!(
            decide(Some(&page), false, true, &[], false),
            MediaDecision::Grant
        );
        assert_eq!(
            decide(Some(&page), true, true, &[], false),
            MediaDecision::Grant
        );
    }
}

#[test]
fn test_product_domains_grant_only_over_https() {
    let allowed = [
        "https://pragmatismo.com.br/store",
        "https://cloud.pragmatismo.com.br/dashboard",
        "https://chat.pragmatismo.com.br/default",
        "https://login.pragmatismo.com.br/login",
        "https://generalbots.org/",
        "https://beiner.generalbots.org/drive",
        "https://BEINER.generalbots.org:443/drive",
    ];
    for page in allowed {
        assert_eq!(
            decide(Some(page), true, true, &[], false),
            MediaDecision::Grant,
            "first-party page {page} must be granted"
        );
    }

    let denied = [
        "http://chat.pragmatismo.com.br/default",
        "https://chat.pragmatismo.com.br:8443/default",
        "https://evil.pragmatismo.com.br/",
        "https://pragmatismo.com.br.evil.example/",
        "https://a.b.generalbots.org/drive",
        "https://evilgeneralbots.org/drive",
    ];
    for page in denied {
        assert_eq!(
            decide(Some(page), true, true, &[], false),
            MediaDecision::DenyForeign,
            "non-first-party page {page} must be denied"
        );
    }
}

#[test]
fn test_localhost_ports_follow_debug_policy() {
    assert_eq!(
        decide(Some(DEV_ORIGIN), true, false, &[], true),
        MediaDecision::Grant
    );
    assert_eq!(
        decide(Some(DEV_ORIGIN), true, false, &[], false),
        MediaDecision::DenyForeign
    );
    assert_eq!(
        decide(Some("https://localhost:8080/api/product"), false, true, &[], false),
        MediaDecision::Grant
    );
    for page in ["http://localhost", "http://localhost:9999", "http://127.0.0.1:8080"] {
        assert_eq!(
            decide(Some(page), true, true, &[], true),
            MediaDecision::DenyForeign,
            "unlisted local origin {page} must be denied"
        );
    }
}

#[test]
fn test_foreign_origins_are_denied() {
    let foreign = [
        "https://mail.example.com",
        "https://evil.example",
        "https://tauri.localhost.evil.example",
        "tauri://evil.example",
        "http://localhost:3001",
    ];
    for page in foreign {
        assert_eq!(
            decide(Some(page), true, true, &[], true),
            MediaDecision::DenyForeign,
            "foreign page {page} must be denied"
        );
    }
}

#[test]
fn test_harmless_permissions_never_grant() {
    assert_eq!(
        decide(Some(DEV_ORIGIN), false, false, &[], true),
        MediaDecision::DenyNoDevice
    );
    assert_eq!(
        decide(None, false, false, &[], true),
        MediaDecision::DenyNoDevice
    );
    assert_eq!(
        decide(Some("https://evil.example"), false, false, &[], false),
        MediaDecision::DenyNoDevice
    );
}

#[test]
fn test_unparseable_page_uri_is_denied() {
    assert_eq!(decide(None, true, true, &[], false), MediaDecision::DenyUnparseable);
    assert_eq!(
        decide(Some("not a uri"), true, true, &[], false),
        MediaDecision::DenyUnparseable
    );
    assert_eq!(
        decide(Some("http://user:secret@localhost:3000/"), true, false, &[], true),
        MediaDecision::DenyUnparseable
    );
    // file:// and an empty URI have no authority to compare against the
    // allow-list, so they are unparseable rather than foreign (#2504).
    assert_eq!(decide(Some("file://"), true, true, &[], true), MediaDecision::DenyUnparseable);
    assert_eq!(decide(Some(""), true, true, &[], true), MediaDecision::DenyUnparseable);
}

#[test]
fn test_env_allowlist_requires_an_exact_origin() {
    let extra = vec!["https://partner.example".to_string()];
    assert!(origin_allowed("https://partner.example", &extra, false));
    assert!(origin_allowed("HTTPS://PARTNER.EXAMPLE", &extra, false));
    assert_eq!(
        decide(Some("https://partner.example.evil.example/"), true, false, &extra, false),
        MediaDecision::DenyForeign
    );
    assert_eq!(
        decide(Some("https://partner.example/drive"), true, false, &[], false),
        MediaDecision::DenyForeign
    );
}

#[test]
fn test_origin_of_normalizes_page_uris() {
    assert_eq!(
        origin_of("tauri://localhost/suite/desktop.html"),
        Some("tauri://localhost")
    );
    assert_eq!(
        origin_of("https://tauri.localhost/suite/desktop.html?q=x#frag"),
        Some("https://tauri.localhost")
    );
    assert_eq!(origin_of("http://user:secret@localhost:3000/"), None);
    assert_eq!(origin_of("not a uri"), None);
}

#[test]
fn test_authority_port_parsing_is_fail_closed() {
    assert_eq!(split_authority("localhost:3000"), ("localhost", Some(3000u16)));
    assert_eq!(split_authority("[::1]:443"), ("::1", Some(443u16)));
    assert_eq!(split_authority("localhost:abc"), ("localhost:abc", None));
    assert_eq!(
        decide(Some("https://localhost:abc"), true, true, &[], false),
        MediaDecision::DenyForeign
    );
}

#[test]
fn test_ip_literals_are_masked_in_logs() {
    assert_eq!(redacted("http://203.0.113.5:8080"), "http://<ip>");
    assert_eq!(redacted("https://[::1]:3000"), "https://<ip>");
    assert_eq!(
        redacted("https://mail.example.com"),
        "https://mail.example.com"
    );
}

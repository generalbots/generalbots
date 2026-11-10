//! Offline unit tests for the OAuth provider model, the PKCE derivation and the
//! state token (#2066).
//!
//! Everything here is pure: no HTTP, no DB, no environment. The PKCE pair uses
//! the published RFC 7636 Appendix B vector, so the derivation is pinned to the
//! specification rather than to our own output.

use crate::providers::ProviderEndpoints;
use crate::{
    generate_pkce_verifier, pkce_s256_challenge, OAuthConfig, OAuthProvider, OAuthState,
};

fn config(provider: OAuthProvider) -> OAuthConfig {
    OAuthConfig::new(
        provider,
        "client-id".to_string(),
        "client-secret".to_string(),
        "https://example.test/callback".to_string(),
    )
}

#[test]
fn pkce_s256_matches_the_rfc7636_vector() {
    // RFC 7636 Appendix B.
    let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
    assert_eq!(
        pkce_s256_challenge(verifier),
        "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
    );
    assert_ne!(pkce_s256_challenge("a"), pkce_s256_challenge("b"));
}

#[test]
fn generated_verifier_is_base64url_and_has_no_padding() {
    let verifier = generate_pkce_verifier();
    // 48 random bytes encode to 64 base64url characters, inside RFC 7636's
    // 43..=128 window.
    assert_eq!(verifier.len(), 64);
    assert!((43..=128).contains(&verifier.len()));
    assert!(
        verifier
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'),
        "unreserved base64url alphabet only, got {verifier}"
    );
    assert!(!verifier.contains('='));
    // Two draws must not collide.
    assert_ne!(verifier, generate_pkce_verifier());
}

#[test]
fn provider_parse_accepts_aliases_and_rejects_unknown() {
    assert_eq!(OAuthProvider::parse("google"), Some(OAuthProvider::Google));
    assert_eq!(OAuthProvider::parse("GOOGLE"), Some(OAuthProvider::Google));
    assert_eq!(OAuthProvider::parse("x"), Some(OAuthProvider::Twitter));
    assert_eq!(OAuthProvider::parse("twitter"), Some(OAuthProvider::Twitter));
    assert_eq!(OAuthProvider::parse("github"), Some(OAuthProvider::GitHub));
    assert_eq!(OAuthProvider::parse(""), None);
    assert_eq!(OAuthProvider::parse("myspace"), None);
    assert_eq!(OAuthProvider::parse(" google"), None);
}

#[test]
fn provider_metadata_is_complete_and_unique() {
    let all = OAuthProvider::all();
    assert_eq!(all.len(), 7);
    let mut prefixes: Vec<&str> = Vec::new();
    for provider in &all {
        assert!(!provider.display_name().is_empty());
        assert!(provider.config_prefix().starts_with("oauth-"));
        assert_eq!(provider.to_string(), provider.display_name());
        prefixes.push(provider.config_prefix());
    }
    prefixes.sort_unstable();
    prefixes.dedup();
    assert_eq!(prefixes.len(), all.len(), "config prefixes must be unique");
}

#[test]
fn endpoints_point_at_https_and_scope_openid_providers() {
    for provider in OAuthProvider::all() {
        let endpoints: ProviderEndpoints = provider.endpoints();
        assert!(endpoints.auth_url.starts_with("https://"));
        assert!(endpoints.token_url.starts_with("https://"));
        assert!(endpoints.userinfo_url.starts_with("https://"));
        assert!(!endpoints.scopes.is_empty());
        assert!(endpoints.scopes.iter().all(|s| !s.is_empty()));
    }
    assert!(OAuthProvider::Google
        .endpoints()
        .scopes
        .contains(&"openid"));
    assert!(OAuthProvider::Discord.endpoints().use_basic_auth);
    assert!(!OAuthProvider::Google.endpoints().use_basic_auth);
}

#[test]
fn config_validity_requires_enabled_and_all_fields() {
    let mut cfg = config(OAuthProvider::GitHub);
    assert!(cfg.is_valid());

    cfg.enabled = false;
    assert!(!cfg.is_valid());
    cfg.enabled = true;

    cfg.client_id = String::new();
    assert!(!cfg.is_valid());
    cfg.client_id = "id".to_string();

    cfg.client_secret = String::new();
    assert!(!cfg.is_valid());
    cfg.client_secret = "secret".to_string();

    cfg.redirect_uri = String::new();
    assert!(!cfg.is_valid());
}

#[test]
fn auth_url_carries_the_s256_challenge_when_a_verifier_exists() {
    let cfg = config(OAuthProvider::Google);
    let url = cfg
        .provider
        .build_auth_url(&cfg, "state-token", Some("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"));

    assert!(url.starts_with("https://accounts.google.com/o/oauth2/v2/auth?"));
    assert!(url.contains("code_challenge=E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"));
    assert!(url.contains("code_challenge_method=S256"));
    assert!(url.contains("state=state-token"));
    assert!(url.contains("client_id=client-id"));
    assert!(url.contains("response_type=code"));
    // Google-specific parameters survive.
    assert!(url.contains("access_type=offline"));
    assert!(url.contains("prompt=consent"));
    // The client secret must never leak into an authorization URL.
    assert!(!url.contains("client-secret"));
}

#[test]
fn auth_url_without_verifier_omits_pkce_for_non_twitter_providers() {
    let cfg = config(OAuthProvider::GitHub);
    let url = cfg.provider.build_auth_url(&cfg, "st", None);
    assert!(!url.contains("code_challenge"));
    assert!(url.starts_with("https://github.com/login/oauth/authorize?"));
}

#[test]
fn twitter_auth_url_keeps_its_plain_challenge() {
    let cfg = config(OAuthProvider::Twitter);
    let url = cfg.provider.build_auth_url(&cfg, "st", None);
    // Twitter historically required a `plain` challenge marker.
    assert!(url.contains("code_challenge_method=plain"));
    assert!(url.contains("code_challenge=challenge"));
}

#[test]
fn redirect_uri_is_percent_encoded() {
    let mut cfg = config(OAuthProvider::Discord);
    cfg.redirect_uri = "https://example.test/cb?x=1&y=2".to_string();
    let url = cfg.provider.build_auth_url(&cfg, "st", None);
    assert!(url.contains("redirect_uri=https%3A%2F%2Fexample.test%2Fcb%3Fx%3D1%26y%3D2"));
}

#[test]
fn state_token_round_trips_through_base64url() {
    let state = OAuthState::new(OAuthProvider::Microsoft, Some("/suite/desktop".to_string()))
        .for_user("user-42")
        .with_pkce();
    let encoded = state.encode();
    assert!(!encoded.contains('='));
    assert!(!encoded.contains('+'));
    assert!(!encoded.contains('/'));

    let decoded = OAuthState::decode(&encoded).expect("state must decode");
    assert_eq!(decoded.token, state.token);
    assert_eq!(decoded.provider, OAuthProvider::Microsoft);
    assert_eq!(decoded.redirect_after.as_deref(), Some("/suite/desktop"));
    assert_eq!(decoded.user_id.as_deref(), Some("user-42"));
    assert_eq!(decoded.created_at, state.created_at);
    assert_eq!(decoded.pkce_verifier, state.pkce_verifier);
    assert!(decoded.pkce_verifier.is_some());
}

#[test]
fn state_decode_rejects_garbage() {
    assert!(OAuthState::decode("not-base64!!").is_none());
    assert!(OAuthState::decode("").is_none());
    // Valid base64 of non-JSON bytes.
    assert!(OAuthState::decode("aGVsbG8gd29ybGQ").is_none());
}

#[test]
fn state_expires_after_ten_minutes() {
    let mut state = OAuthState::new(OAuthProvider::Google, None);
    assert!(!state.is_expired());

    // A state minted long ago (epoch) is expired; the window is 600 s.
    state.created_at = 0;
    assert!(state.is_expired());

    // Exactly at the boundary the state is still usable (`>` not `>=`).
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    state.created_at = now - 600;
    assert!(!state.is_expired());
    state.created_at = now - 601;
    assert!(state.is_expired());
}

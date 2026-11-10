//! Inbound webhook authenticity, exercised offline at library level (#2113).
//!
//! `/api/whatsapp/webhook` and `/api/msteams/messages` are anonymous routes:
//! Teams and Meta post from their own infrastructure, without a user session,
//! so the credential on the request is the only proof a delivery is genuine.
//! Both gates used to accept any JSON body, which let a stranger create a bot
//! turn. These tests drive the two primitives the handlers call — the Meta
//! `X-Hub-Signature-256` HMAC and the Teams outgoing-webhook HMAC — and pin
//! that an unconfigured secret fails closed instead of accepting everything.

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine as _;
use botmsteams::auth::{authorize, AuthError, TeamsCredentials};
use botwhatsapp::utils::verify_webhook_signature;
use hmac::{Hmac, Mac};
use http::HeaderMap;
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Raw bytes exactly as Meta would deliver them.
const META_BODY: &[u8] = br#"{"object":"whatsapp_business_account","entry":[{"id":"1"}]}"#;
const META_SECRET: &str = "offline-suite-meta-app-secret";
const OTHER_META_SECRET: &str = "another-app-secret-that-must-not-verify";

/// Base64 security token Microsoft shows once when the webhook is registered.
const TEAMS_TOKEN: &str = "Z2VuZXJhbC1ib3RzLW9mZmxpbmUtdGVhbXMtdG9rZW4=";
const OTHER_TEAMS_TOKEN: &str = "YW5vdGhlci1vZmZsaW5lLXRlYW1zLXRva2Vu";
const TEAMS_BODY: &[u8] = br#"{"type":"message","text":"ping"}"#;

fn hmac_bytes(key: &[u8], body: &[u8]) -> Option<Vec<u8>> {
    match HmacSha256::new_from_slice(key) {
        Ok(mut mac) => {
            mac.update(body);
            Some(mac.finalize().into_bytes().to_vec())
        }
        // Unreachable for HMAC (any key length is valid).
        Err(_) => None,
    }
}

/// `X-Hub-Signature-256: sha256=<lowercase hex>`.
fn meta_signature(body: &[u8], secret: &str) -> String {
    match hmac_bytes(secret.as_bytes(), body) {
        Some(digest) => {
            let mut hex = String::with_capacity(digest.len() * 2);
            for byte in digest {
                hex.push_str(&format!("{byte:02x}"));
            }
            format!("sha256={hex}")
        }
        None => String::new(),
    }
}

/// The Teams key is the base64-decoded security token; a token that is not
/// base64 is used as raw bytes (documented derivation in `auth::hmac`).
fn teams_key(token: &str) -> Vec<u8> {
    match BASE64.decode(token.trim()) {
        Ok(decoded) => decoded,
        Err(_) => token.trim().as_bytes().to_vec(),
    }
}

fn teams_signature(body: &[u8], token: &str) -> String {
    match hmac_bytes(&teams_key(token), body) {
        Some(digest) => BASE64.encode(digest),
        None => String::new(),
    }
}

fn teams_headers(scheme: &str, credential: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    let value = format!("{scheme} {credential}");
    headers.insert("authorization", value.parse().expect("a header value"));
    headers
}

fn hmac_headers(signature: &str) -> HeaderMap {
    teams_headers("HMAC", signature)
}

// ─────────────────────────────────────────────────────────── WhatsApp (Meta)

#[test]
fn a_meta_signature_over_the_raw_body_is_accepted() {
    let signature = meta_signature(META_BODY, META_SECRET);
    assert!(signature.starts_with("sha256="), "Meta prefixes the algorithm");
    assert!(
        verify_webhook_signature(META_BODY, &signature, META_SECRET),
        "a delivery signed with the app secret is authentic"
    );
}

#[test]
fn a_meta_signature_over_different_bytes_is_refused() {
    let signature = meta_signature(META_BODY, META_SECRET);

    // Same JSON, different bytes: a re-serialized value must not pass.
    let reserialized = br#"{"object": "whatsapp_business_account", "entry": [{"id": "1"}]}"#;
    assert!(
        !verify_webhook_signature(reserialized, &signature, META_SECRET),
        "the digest covers the raw bytes, not the parsed document"
    );

    let extended = [META_BODY, &b" "[..]].concat();
    assert!(!verify_webhook_signature(&extended, &signature, META_SECRET));
}

#[test]
fn a_missing_or_malformed_meta_signature_is_refused() {
    let valid = meta_signature(META_BODY, META_SECRET);

    for signature in [
        "",
        "sha256=",
        "deadbeef",
        "sha1=0123456789abcdef",
        "sha256=not-hex-at-all",
    ] {
        assert!(
            !verify_webhook_signature(META_BODY, signature, META_SECRET),
            "{signature:?} must not authenticate a delivery"
        );
    }

    // Meta sends lowercase hex; an uppercase digest is a different string.
    let uppercase = format!("sha256={}", &valid["sha256=".len()..].to_uppercase());
    assert!(!verify_webhook_signature(META_BODY, &uppercase, META_SECRET));

    // A truncated digest has the wrong length.
    let truncated = &valid[..valid.len() - 2];
    assert!(!verify_webhook_signature(META_BODY, truncated, META_SECRET));
}

#[test]
fn a_meta_signature_from_another_app_secret_is_refused() {
    let foreign = meta_signature(META_BODY, OTHER_META_SECRET);
    assert!(
        !verify_webhook_signature(META_BODY, &foreign, META_SECRET),
        "only the configured app secret may sign deliveries"
    );
}

#[test]
fn an_unconfigured_meta_app_secret_fails_closed() {
    // Even a digest computed with the empty key must not be accepted: the gate
    // refuses before comparing, so a bot without `whatsapp_app_secret` cannot
    // be driven by any caller.
    let signature = meta_signature(META_BODY, "");
    assert!(
        !verify_webhook_signature(META_BODY, &signature, ""),
        "an unconfigured app secret accepts nothing"
    );
    assert!(!verify_webhook_signature(META_BODY, "", ""));
}

// ────────────────────────────────────────────────────────── Teams (HMAC)

#[tokio::test]
async fn a_teams_hmac_delivery_is_accepted() {
    let credentials = TeamsCredentials::new("", TEAMS_TOKEN);
    let headers = hmac_headers(&teams_signature(TEAMS_BODY, TEAMS_TOKEN));

    assert_eq!(authorize(&headers, TEAMS_BODY, &credentials).await, Ok(()));
}

#[tokio::test]
async fn a_teams_delivery_without_authorization_is_refused() {
    let credentials = TeamsCredentials::new("", TEAMS_TOKEN);

    assert_eq!(
        authorize(&HeaderMap::new(), TEAMS_BODY, &credentials).await,
        Err(AuthError::MissingHeader)
    );
}

#[tokio::test]
async fn a_tampered_teams_body_is_refused() {
    let credentials = TeamsCredentials::new("", TEAMS_TOKEN);
    let signature = teams_signature(TEAMS_BODY, TEAMS_TOKEN);
    let tampered = br#"{"type":"message","text":"wire-transfer"}"#;

    assert_eq!(
        authorize(&hmac_headers(&signature), tampered, &credentials).await,
        Err(AuthError::InvalidSignature),
        "the HMAC must cover the bytes that are processed"
    );
}

#[tokio::test]
async fn a_teams_signature_from_another_token_is_refused() {
    let credentials = TeamsCredentials::new("", TEAMS_TOKEN);
    let foreign = teams_signature(TEAMS_BODY, OTHER_TEAMS_TOKEN);

    assert_eq!(
        authorize(&hmac_headers(&foreign), TEAMS_BODY, &credentials).await,
        Err(AuthError::InvalidSignature)
    );

    let garbage = teams_headers("HMAC", "%%%not-base64%%%");
    assert_eq!(
        authorize(&garbage, TEAMS_BODY, &credentials).await,
        Err(AuthError::InvalidSignature)
    );
}

#[tokio::test]
async fn an_unconfigured_teams_bot_refuses_every_delivery() {
    // Neither `teams-app-id` nor `teams-security-token` is set: the gate must
    // refuse instead of routing the activity into a bot turn.
    let unconfigured = TeamsCredentials::default();
    assert!(!unconfigured.is_configured());

    assert_eq!(
        authorize(&hmac_headers("anything"), TEAMS_BODY, &unconfigured).await,
        Err(AuthError::Unconfigured)
    );
    assert_eq!(
        authorize(&HeaderMap::new(), TEAMS_BODY, &unconfigured).await,
        Err(AuthError::Unconfigured)
    );
}

#[tokio::test]
async fn a_bearer_token_without_an_app_id_is_refused_offline() {
    // The Bot Framework path needs the OpenID key set; without an app id it
    // must stop before any network call and fail closed.
    let credentials = TeamsCredentials::new("", TEAMS_TOKEN);
    let bearer = teams_headers("Bearer", "header.payload.signature");

    assert_eq!(
        authorize(&bearer, TEAMS_BODY, &credentials).await,
        Err(AuthError::Unconfigured)
    );
}

#[tokio::test]
async fn an_unsupported_authorization_scheme_is_refused() {
    let credentials = TeamsCredentials::new("", TEAMS_TOKEN);
    let basic = teams_headers("Basic", "dXNlcjpwYXNz");

    assert_eq!(
        authorize(&basic, TEAMS_BODY, &credentials).await,
        Err(AuthError::UnsupportedScheme)
    );
}

#[tokio::test]
async fn a_plain_text_security_token_is_used_as_raw_key() {
    // Operators sometimes store a random string instead of the base64 token.
    // The key derivation falls back to the raw bytes rather than rejecting
    // every delivery forever.
    let plain = "plain-offline-security-token";
    let credentials = TeamsCredentials::new("", plain);

    assert_eq!(
        authorize(
            &hmac_headers(&teams_signature(TEAMS_BODY, plain)),
            TEAMS_BODY,
            &credentials
        )
        .await,
        Ok(())
    );
}

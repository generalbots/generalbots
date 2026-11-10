//! Gate tests for inbound Teams activities (#2113).
//!
//! The RSA key pair below is a throwaway fixture generated for the test suite
//! (the same one `botsecurity-auth/src/jwt/hardening_tests.rs` uses). It signs
//! nothing outside `cargo test`, and no test touches the network: the JWT cases
//! drive `verify_with_key` with the fixture key, and the JWKS cases parse a
//! fixture document.

use axum::http::{header::AUTHORIZATION, HeaderMap, HeaderValue, StatusCode};
use base64::Engine as _;
use hmac::{Hmac, Mac};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header};
use sha2::Sha256;

use super::hmac as teams_hmac;
use super::jwt;
use super::jwks;
use super::*;

const TEST_RSA_PRIVATE_PEM: &str = "-----BEGIN PRIVATE KEY-----
MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQDAm442u3McoiOK
ogKSz08+Pe9KZuC7qdGedoJnk50/0Al/0c2QPS9KkQFy/gNHmKnBFhvoD69TWfgL
HIXeZXmietBqCWSRWJCpDuYL2grFSgSPLU32rkXB9n3BtcyXfS8rKYBTW5GxlBX9
a7JtBRHHaaNbXtYLFTD3jYpUKjiAy8jMM19wxGENA2Wb8cl06GaJFDulnDb3VphV
337WWiFMFffSa98rt1gqVnArdhrwkF7ejqLC9NHnl6agOaEcp9thfou7G3skgREM
b2zJ0Q64NcxH5xAzOJA5HIo42hY2Z7TRvq/xYNdpSCXHCEw21LBhOsBT/u0EWzVQ
Ul57GjjFAgMBAAECggEAGcDy/BiGZM69R90v0OeNZfosE9u7svwa/kBEYYsYXJ75
ljC8dsvnleXbtr+GOr2L/ThylM38GH4QL5Qpxjn1UKdxVKxa2SBpf2FxmyYViJnE
EEzzfseAplXKLOudJqhjdTcAZKT6zy7I3fDXk8I4xVw3rpD4t9SmjoNLesOGeJoL
otGHZG/mnvBK/gnjIx2e/FBMJL4KMrq/gJtJ0FkhnWPG2/QvQJsnN5B+ZxUbntUX
Zx/4/+6uv1osceDeVQ48T3Y2sroi9FmYoFk8ZcHz7XBC7bV+/RqKp+rwd9gUE8xn
BiU0kfocWO9NKw9+D+nAkvBJ1fUJdxzF3WM3818cRQKBgQDi67h34eBpTi7ROKpH
vvh9qbMtJrp6dbmQ1oY8ybzPN6bbe8/372h4nJAIq1aQbqBtZ2v0u0DeOTcbcjVh
kIfakss6E1H+hUlBWMPK9+MtR+QYQ7bHUaTJwyTjGqz0Ag0LbKZINdyHe3IJYUub
GicW9OqQtFXSCPAezF4zAyymvwKBgQDZSiurs0DHMmcHtsaU0P8CnJ4SHfH7HjrV
y2XsqwrQltynqqBd7+b+js5Sep25T0oAl35KWwehH0LlzJvcg9dHivBBEFGS3lyu
BR9TcJQspxbinPbNBs8sfbEPpqWdBc16FPSReT19F5fTAH3+3f2o8dL0DLgb4iQ3
UVpocxOlewKBgEIQbnw4fqUp5edzn9+TcbTkGw4iIFhWvde1HQ/Fi/qoMSPlP/7m
7Htw2V4sw9741iwD2CJAAfqwnaKX9YKaGXNBGXmm9lArbdBgj2AgGzurmnSWAji1
REyZS7vyoDfKARI6Z47OX43yGpkwglp9mrPMSh4m4nVrtLzCswoW+oDxAoGAbRZl
kk/t55OQUjkavT+npyIF4caS46WrmAYDZ6v/Q/VZVfu9gyub5pS/VH0mjhAWcPGd
f6rvy54sMb5lSXIPNGW2dJPCwF9OXDdKINPWMAy62o4/AXO3svhvWhIp9wkgvXej
yRz7F8AugsoudzPv00+Evu1byPPhhrwbDXEond0CgYEAlcXd6uKQ4mIDJYDarAlL
Qi8y//dSQDidLtWd2/g3xnW6aQXPN3WaBdh4n+tfku7EPrdXYo9eVqZ6ACfvXO5G
RwxZ4cmnpsEPPn+a/cF0gr8M0TylabE5bO+bM6yf9MyeseLrG0+NHzcZ9o6kPe9I
w8FXKeZhj5IWpsruVvfmFTw=
-----END PRIVATE KEY-----";

const TEST_RSA_PUBLIC_PEM: &str = "-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAwJuONrtzHKIjiqICks9P
Pj3vSmbgu6nRnnaCZ5OdP9AJf9HNkD0vSpEBcv4DR5ipwRYb6A+vU1n4CxyF3mV5
onrQaglkkViQqQ7mC9oKxUoEjy1N9q5FwfZ9wbXMl30vKymAU1uRsZQV/WuybQUR
x2mjW17WCxUw942KVCo4gMvIzDNfcMRhDQNlm/HJdOhmiRQ7pZw291aYVd9+1loh
TBX30mvfK7dYKlZwK3Ya8JBe3o6iwvTR55emoDmhHKfbYX6Luxt7JIERDG9sydEO
uDXMR+cQMziQORyKONoWNme00b6v8WDXaUglxwhMNtSwYTrAU/7tBFs1UFJeexo4
xQIDAQAB
-----END PUBLIC KEY-----";

const APP_ID: &str = "11111111-2222-3333-4444-555555555555";
/// `base64("outgoing-webhook-token")` — the form Microsoft displays once.
const SECURITY_TOKEN: &str = "b3V0Z29pbmctd2ViaG9vay10b2tlbg==";
const BODY: &[u8] = br#"{"type":"message","id":"1"}"#;

fn credentials() -> TeamsCredentials {
    TeamsCredentials::new(APP_ID, SECURITY_TOKEN)
}

fn with_authorization(value: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(value).expect("fixture header is valid"),
    );
    headers
}

fn hmac_signature(body: &[u8], token: &str) -> String {
    let key = base64::engine::general_purpose::STANDARD
        .decode(token)
        .expect("fixture token is base64");
    let mut mac = Hmac::<Sha256>::new_from_slice(&key).expect("any key length is valid");
    mac.update(body);
    base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes())
}

fn claims(audience: &str, issuer: &str, exp_offset: i64, nbf_offset: i64) -> serde_json::Value {
    let now = chrono::Utc::now().timestamp();
    serde_json::json!({
        "aud": audience,
        "iss": issuer,
        "exp": now + exp_offset,
        "nbf": now + nbf_offset,
        "serviceurl": "https://smba.trafficmanager.net/teams/",
    })
}

fn sign(claims: &serde_json::Value) -> String {
    let key = EncodingKey::from_rsa_pem(TEST_RSA_PRIVATE_PEM.as_bytes())
        .expect("fixture private key is valid");
    jsonwebtoken::encode(&Header::new(Algorithm::RS256), claims, &key)
        .expect("fixture claims serialize")
}

fn fixture_public_key() -> DecodingKey {
    DecodingKey::from_rsa_pem(TEST_RSA_PUBLIC_PEM.as_bytes()).expect("fixture public key is valid")
}

// ------------------------------------------------------------ HMAC (webhook)

#[tokio::test]
async fn a_signed_activity_is_accepted() {
    let signature = hmac_signature(BODY, SECURITY_TOKEN);
    let headers = with_authorization(&format!("HMAC {signature}"));

    assert_eq!(authorize(&headers, BODY, &credentials()).await, Ok(()));
}

#[tokio::test]
async fn the_scheme_name_is_case_insensitive() {
    let signature = hmac_signature(BODY, SECURITY_TOKEN);
    let headers = with_authorization(&format!("hmac {signature}"));

    assert_eq!(authorize(&headers, BODY, &credentials()).await, Ok(()));
}

#[tokio::test]
async fn a_tampered_body_is_rejected() {
    let signature = hmac_signature(BODY, SECURITY_TOKEN);
    let headers = with_authorization(&format!("HMAC {signature}"));

    assert_eq!(
        authorize(&headers, br#"{"type":"message","id":"2"}"#, &credentials()).await,
        Err(AuthError::InvalidSignature)
    );
}

#[tokio::test]
async fn a_wrong_signature_is_rejected() {
    let other = hmac_signature(BODY, "b3RoZXItdG9rZW4=");
    let headers = with_authorization(&format!("HMAC {other}"));

    assert_eq!(
        authorize(&headers, BODY, &credentials()).await,
        Err(AuthError::InvalidSignature)
    );
}

#[tokio::test]
async fn a_malformed_signature_is_rejected() {
    let headers = with_authorization("HMAC not-base64!");

    assert_eq!(
        authorize(&headers, BODY, &credentials()).await,
        Err(AuthError::InvalidSignature)
    );
}

#[test]
fn a_plain_token_is_used_as_the_key() {
    // Operators sometimes paste a random string instead of the base64 token.
    // The raw bytes are the key then, so the webhook still works.
    let token = "plain-token";
    let mut mac = Hmac::<Sha256>::new_from_slice(token.as_bytes()).expect("any key length");
    mac.update(BODY);
    let signature = base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes());

    assert_eq!(teams_hmac::verify(BODY, &signature, token), Ok(()));
}

// --------------------------------------------------------- header rejection

#[tokio::test]
async fn a_missing_header_is_rejected() {
    assert_eq!(
        authorize(&HeaderMap::new(), BODY, &credentials()).await,
        Err(AuthError::MissingHeader)
    );
}

#[tokio::test]
async fn an_unsupported_scheme_is_rejected() {
    let headers = with_authorization("Basic dXNlcjpwYXNz");

    assert_eq!(
        authorize(&headers, BODY, &credentials()).await,
        Err(AuthError::UnsupportedScheme)
    );
}

#[tokio::test]
async fn an_empty_scheme_value_is_rejected() {
    let headers = with_authorization("Bearer   ");

    assert_eq!(
        authorize(&headers, BODY, &credentials()).await,
        Err(AuthError::UnsupportedScheme)
    );
}

#[tokio::test]
async fn unconfigured_credentials_fail_closed() {
    let signature = hmac_signature(BODY, SECURITY_TOKEN);
    let headers = with_authorization(&format!("HMAC {signature}"));

    assert_eq!(
        authorize(&headers, BODY, &TeamsCredentials::default()).await,
        Err(AuthError::Unconfigured)
    );
}

#[tokio::test]
async fn a_scheme_without_its_credential_fails_closed() {
    let signature = hmac_signature(BODY, SECURITY_TOKEN);
    let headers = with_authorization(&format!("HMAC {signature}"));
    let only_app_id = TeamsCredentials::new(APP_ID, "");

    assert_eq!(
        authorize(&headers, BODY, &only_app_id).await,
        Err(AuthError::Unconfigured)
    );

    let bearer = with_authorization("Bearer header.payload.signature");
    let only_token = TeamsCredentials::new("", SECURITY_TOKEN);

    assert_eq!(
        authorize(&bearer, BODY, &only_token).await,
        Err(AuthError::Unconfigured)
    );
}

#[tokio::test]
async fn every_refusal_answers_unauthorized() {
    for error in [
        AuthError::Unconfigured,
        AuthError::MissingHeader,
        AuthError::UnsupportedScheme,
        AuthError::InvalidSignature,
        AuthError::InvalidToken,
    ] {
        assert_eq!(error.status(), StatusCode::UNAUTHORIZED);
        assert!(!error.reason().is_empty());
    }
}

// -------------------------------------------------------- Bot Framework JWT

#[test]
fn a_valid_bot_framework_token_is_accepted() {
    let token = sign(&claims(APP_ID, jwt::BOT_FRAMEWORK_ISSUER, 600, -10));

    assert_eq!(jwt::verify_with_key(&token, APP_ID, &fixture_public_key()), Ok(()));
}

#[test]
fn an_expired_token_is_rejected() {
    // Far enough in the past to escape the documented clock skew.
    let token = sign(&claims(APP_ID, jwt::BOT_FRAMEWORK_ISSUER, -3600, -7200));

    assert_eq!(
        jwt::verify_with_key(&token, APP_ID, &fixture_public_key()),
        Err(AuthError::InvalidToken)
    );
}

#[test]
fn a_token_that_is_not_valid_yet_is_rejected() {
    let token = sign(&claims(APP_ID, jwt::BOT_FRAMEWORK_ISSUER, 3600, 1800));

    assert_eq!(
        jwt::verify_with_key(&token, APP_ID, &fixture_public_key()),
        Err(AuthError::InvalidToken)
    );
}

#[test]
fn a_token_for_another_audience_is_rejected() {
    let token = sign(&claims("99999999-0000-0000-0000-000000000000", jwt::BOT_FRAMEWORK_ISSUER, 600, -10));

    assert_eq!(
        jwt::verify_with_key(&token, APP_ID, &fixture_public_key()),
        Err(AuthError::InvalidToken)
    );
}

#[test]
fn a_token_from_another_issuer_is_rejected() {
    let token = sign(&claims(APP_ID, "https://evil.example.com", 600, -10));

    assert_eq!(
        jwt::verify_with_key(&token, APP_ID, &fixture_public_key()),
        Err(AuthError::InvalidToken)
    );
}

#[test]
fn a_token_without_a_configured_app_id_is_rejected() {
    let token = sign(&claims(APP_ID, jwt::BOT_FRAMEWORK_ISSUER, 600, -10));

    assert_eq!(
        jwt::verify_with_key(&token, "", &fixture_public_key()),
        Err(AuthError::Unconfigured)
    );
}

#[tokio::test]
async fn an_algorithm_confusion_token_is_rejected_before_any_key_lookup() {
    // HS256 keyed with the public key is the classic RS256 confusion attack:
    // the alg check must reject it without contacting the key set.
    let key = EncodingKey::from_secret(TEST_RSA_PUBLIC_PEM.as_bytes());
    let token = jsonwebtoken::encode(
        &Header::new(Algorithm::HS256),
        &claims(APP_ID, jwt::BOT_FRAMEWORK_ISSUER, 600, -10),
        &key,
    )
    .expect("fixture claims serialize");

    assert_eq!(jwt::verify(&token, APP_ID).await, Err(AuthError::InvalidToken));
}

#[tokio::test]
async fn a_malformed_bearer_token_is_rejected() {
    let headers = with_authorization("Bearer not-a-jwt");

    assert_eq!(
        authorize(&headers, BODY, &credentials()).await,
        Err(AuthError::InvalidToken)
    );
}

// ------------------------------------------------------------------- JWKS

#[test]
fn a_published_key_set_is_indexed_by_kid() {
    let document = serde_json::json!({
        "keys": [
            {"kty": "RSA", "kid": "key-one", "n": "AQAB", "e": "AQAB"},
            {"kty": "RSA", "kid": "key-two", "n": "AQAB", "e": "AQAB"},
            {"kty": "EC", "kid": "ec-key", "crv": "P-256", "x": "AQAB", "y": "AQAB"},
            {"kty": "RSA", "n": "AQAB", "e": "AQAB"}
        ]
    })
    .to_string();

    let keys = jwks::keys_from_jwks(&document).expect("fixture key set parses");

    assert!(keys.contains_key("key-one"));
    assert!(keys.contains_key("key-two"));
    // An unknown kid (including a rolled-over one) is simply absent, which is
    // what makes the verifier refresh and then reject.
    assert!(!keys.contains_key("unknown-kid"));
    // A non-RSA key is not a verification key for RS256.
    assert!(!keys.contains_key("ec-key"));
}

#[test]
fn a_key_set_without_a_usable_key_is_an_error() {
    let empty = serde_json::json!({"keys": []}).to_string();
    let broken = serde_json::json!({"keys": [{"kty": "RSA", "kid": "k", "n": "", "e": ""}]}).to_string();

    assert!(jwks::keys_from_jwks(&empty).is_err());
    assert!(jwks::keys_from_jwks(&broken).is_err());
    assert!(jwks::keys_from_jwks("not json").is_err());
}

//! JWT hardening tests (issue #2125).
//!
//! Covers the three failure modes the audit found: an unverified decoder that
//! could be called by mistake, `alg: none` acceptance, and HS256/RS256
//! algorithm confusion. The keys below are throwaway test fixtures generated
//! for this file only; they sign nothing outside `cargo test`.

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::{Duration, Utc};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use uuid::Uuid;

use super::jwt_types::{Claims, JwtAlgorithm, JwtConfig, JwtKey, JwtManager, TokenType};

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

fn config(algorithm: JwtAlgorithm) -> JwtConfig {
    JwtConfig {
        algorithm,
        ..JwtConfig::default()
    }
}

fn rsa_manager() -> JwtManager {
    JwtManager::with_separate_keys(
        config(JwtAlgorithm::RS256),
        &JwtKey::from_rsa_private_key(TEST_RSA_PRIVATE_PEM),
        &JwtKey::from_rsa_public_key(TEST_RSA_PUBLIC_PEM),
    )
    .expect("RS256 test manager")
}

fn hmac_manager() -> JwtManager {
    JwtManager::from_secret("test-secret-key-for-testing-only-32-chars!")
        .expect("HS256 test manager")
}

fn claims_for(manager: &JwtManager) -> Claims {
    Claims::new(
        Uuid::new_v4(),
        &manager.config().issuer,
        &manager.config().audience,
        TokenType::Access,
        Utc::now() + Duration::minutes(5),
    )
    .expect("test claims")
}

fn b64(bytes: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Builds an `alg: none` token: a valid-looking payload with no signature.
fn alg_none_token(claims: &Claims) -> String {
    let header = b64(br#"{"alg":"none","typ":"JWT"}"#);
    let payload = b64(&serde_json::to_vec(claims).expect("claims json"));
    format!("{header}.{payload}.")
}

/// Builds an HS256 token whose HMAC key is arbitrary bytes — used to sign
/// with an RSA public key, the algorithm-confusion attack.
fn hs256_token_signed_with(claims: &Claims, key: &[u8]) -> String {
    let header = b64(br#"{"alg":"HS256","typ":"JWT"}"#);
    let payload = b64(&serde_json::to_vec(claims).expect("claims json"));
    let signing_input = format!("{header}.{payload}");
    let mut mac = Hmac::<Sha256>::new_from_slice(key).expect("hmac accepts any key length");
    mac.update(signing_input.as_bytes());
    format!("{signing_input}.{}", b64(&mac.finalize().into_bytes()))
}

#[test]
fn rs256_manager_accepts_its_own_tokens() {
    let manager = rsa_manager();
    let token = manager
        .generate_access_token(claims_for(&manager))
        .expect("sign RS256");
    assert!(manager.validate_token(&token).is_ok());
}

#[test]
fn alg_none_is_rejected() {
    let manager = hmac_manager();
    let token = alg_none_token(&claims_for(&manager));
    assert!(
        manager.validate_token(&token).is_err(),
        "a token with alg=none must never validate"
    );
}

#[test]
fn hs256_signed_with_rsa_public_key_is_rejected_by_rs256_manager() {
    let manager = rsa_manager();
    // The attack: HMAC key = the RSA public key bytes, header says HS256.
    let forged = hs256_token_signed_with(&claims_for(&manager), TEST_RSA_PUBLIC_PEM.as_bytes());
    assert!(
        manager.validate_token(&forged).is_err(),
        "HS256 signed with the RSA public key must not pass an RS256 verifier"
    );
}

#[test]
fn rs256_token_is_rejected_by_hs256_manager() {
    let signer = rsa_manager();
    let token = signer
        .generate_access_token(claims_for(&signer))
        .expect("sign RS256");
    let verifier = hmac_manager();
    assert!(
        verifier.validate_token(&token).is_err(),
        "an RS256 token must not pass an HS256 verifier"
    );
}

#[test]
fn one_byte_signature_difference_fails_without_panicking() {
    let manager = hmac_manager();
    let token = manager
        .generate_access_token(claims_for(&manager))
        .expect("sign HS256");

    let mut bytes = token.into_bytes();
    let last = bytes.last_mut().expect("non-empty token");
    *last = if *last == b'A' { b'B' } else { b'A' };
    let tampered = String::from_utf8(bytes).expect("still utf8");

    assert!(manager.validate_token(&tampered).is_err());
}

#[test]
fn unverified_decode_is_not_reachable_from_the_production_source() {
    // The helper is gone; this pins it so a future "just read the claims"
    // shortcut cannot reappear unnoticed.
    let source = include_str!("jwt_manager.rs");
    assert!(!source.contains("jsonwebtoken::dangerous"));
    assert!(!source.contains("insecure_decode"));
    assert!(!source.contains("decode_without_validation"));
}

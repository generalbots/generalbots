//! Shared fixtures for the inbound Teams gate tests (#2113).
//!
//! The RSA key pair is a throwaway fixture generated for the test suite (the
//! same one `botsecurity-auth/src/jwt/hardening_tests.rs` uses). It signs
//! nothing outside `cargo test`, and no test touches the network: the JWT cases
//! drive `verify_with_key` with the fixture key, and the JWKS cases parse a
//! fixture document.

use axum::http::{header::AUTHORIZATION, HeaderMap, HeaderValue};
use base64::Engine as _;
use hmac::{Hmac, Mac};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header};
use sha2::Sha256;

use super::TeamsCredentials;

pub(super) const TEST_RSA_PRIVATE_PEM: &str = "-----BEGIN PRIVATE KEY-----
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

pub(super) const TEST_RSA_PUBLIC_PEM: &str = "-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAwJuONrtzHKIjiqICks9P
Pj3vSmbgu6nRnnaCZ5OdP9AJf9HNkD0vSpEBcv4DR5ipwRYb6A+vU1n4CxyF3mV5
onrQaglkkViQqQ7mC9oKxUoEjy1N9q5FwfZ9wbXMl30vKymAU1uRsZQV/WuybQUR
x2mjW17WCxUw942KVCo4gMvIzDNfcMRhDQNlm/HJdOhmiRQ7pZw291aYVd9+1loh
TBX30mvfK7dYKlZwK3Ya8JBe3o6iwvTR55emoDmhHKfbYX6Luxt7JIERDG9sydEO
uDXMR+cQMziQORyKONoWNme00b6v8WDXaUglxwhMNtSwYTrAU/7tBFs1UFJeexo4
xQIDAQAB
-----END PUBLIC KEY-----";

pub(super) const APP_ID: &str = "11111111-2222-3333-4444-555555555555";
/// `base64("outgoing-webhook-token")` — the form Microsoft displays once.
pub(super) const SECURITY_TOKEN: &str = "b3V0Z29pbmctd2ViaG9vay10b2tlbg==";
pub(super) const BODY: &[u8] = br#"{"type":"message","id":"1"}"#;

pub(super) fn credentials() -> TeamsCredentials {
    TeamsCredentials::new(APP_ID, SECURITY_TOKEN)
}

pub(super) fn with_authorization(value: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(value).expect("fixture header is valid"),
    );
    headers
}

pub(super) fn hmac_signature(body: &[u8], token: &str) -> String {
    let key = base64::engine::general_purpose::STANDARD
        .decode(token)
        .expect("fixture token is base64");
    let mut mac = Hmac::<Sha256>::new_from_slice(&key).expect("any key length is valid");
    mac.update(body);
    base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes())
}

pub(super) fn claims(
    audience: &str,
    issuer: &str,
    exp_offset: i64,
    nbf_offset: i64,
) -> serde_json::Value {
    let now = chrono::Utc::now().timestamp();
    serde_json::json!({
        "aud": audience,
        "iss": issuer,
        "exp": now + exp_offset,
        "nbf": now + nbf_offset,
        "serviceurl": "https://smba.trafficmanager.net/teams/",
    })
}

pub(super) fn sign(claims: &serde_json::Value) -> String {
    let key = EncodingKey::from_rsa_pem(TEST_RSA_PRIVATE_PEM.as_bytes())
        .expect("fixture private key is valid");
    jsonwebtoken::encode(&Header::new(Algorithm::RS256), claims, &key)
        .expect("fixture claims serialize")
}

pub(super) fn fixture_public_key() -> DecodingKey {
    DecodingKey::from_rsa_pem(TEST_RSA_PUBLIC_PEM.as_bytes()).expect("fixture public key is valid")
}

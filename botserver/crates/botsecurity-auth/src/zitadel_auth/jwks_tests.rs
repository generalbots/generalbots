//! Unit tests for the Zitadel JWKS verifier (#2851).
//!
//! Fixtures are throwaway keypairs that sign nothing outside `cargo test`.

use super::jwks::{jwks_uri, keys_from_jwks, seed_key, verify_with_key};
use super::*;
use crate::zitadel_auth::{AuthError, ZitadelAuthConfig, ZitadelAuthProvider};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, Algorithm, DecodingKey, EncodingKey, Header};
use serde_json::{json, Value};

// Throwaway 2048-bit keypairs generated for these tests only. They sign
// nothing outside `cargo test`.
const TEST_PRIVATE_PEM: &str = "-----BEGIN PRIVATE KEY-----
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

const TEST_PUBLIC_PEM: &str = "-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAwJuONrtzHKIjiqICks9P
Pj3vSmbgu6nRnnaCZ5OdP9AJf9HNkD0vSpEBcv4DR5ipwRYb6A+vU1n4CxyF3mV5
onrQaglkkViQqQ7mC9oKxUoEjy1N9q5FwfZ9wbXMl30vKymAU1uRsZQV/WuybQUR
x2mjW17WCxUw942KVCo4gMvIzDNfcMRhDQNlm/HJdOhmiRQ7pZw291aYVd9+1loh
TBX30mvfK7dYKlZwK3Ya8JBe3o6iwvTR55emoDmhHKfbYX6Luxt7JIERDG9sydEO
uDXMR+cQMziQORyKONoWNme00b6v8WDXaUglxwhMNtSwYTrAU/7tBFs1UFJeexo4
xQIDAQAB
-----END PUBLIC KEY-----";

// Second, unrelated keypair: an attacker who signs with this one must not
// be accepted when the directory publishes TEST_PUBLIC_PEM.
const FORGED_PRIVATE_PEM: &str = "-----BEGIN PRIVATE KEY-----
MIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQDL3GFJd1vJhaiH
EWROLG1kKm5VyXWq3z3t2GWp44YXvL6PaA6Ej4xYoOkRfDBnorS2Mvmfn88dh+um
SqHhtshHfcfBdMR6hdQDlrV9WXF8CtekJBvMz5xIgxpSPhfx4QZ+nCqOg6UVbqTD
rpgsy3YN00lzrSHR0J1crlH34OQdMW1B6oI7pNtr+eFjoAaXnFhCd2HJCmOqZ8cx
cP/tU48WCpT/mJbdi5sx5jFo+2J2KJslBceK5cxdGFWs4TepPokF4OHxC7ZzpFmd
SoqgNGExwIYjdOKiTK7W0QDuJyp/mplM40MPklNFjt/784+jZvfB9OmjBmo52ms2
xRYDZ6f7AgMBAAECggEAYY6QpgovDCTgzeZWRIzBOr6Nd5NFsqRLrB8zI5ZkzMY/
wTvwOqLBbIZwkrQGLAoyhcCmMavV/1FUxYsQpgYFGSVJqzwYbHPPznnYV2iuxMgh
gwTeoug1Z4QJ3BKn/dIMfBl6yiGYpkWBJr3OSB2D3MDgdTGErjYxoDExu7hNIeRz
M5Eoxi8kB1rENgrLSkSL7HlHqQ6cgTM3GeCCFYMNnMxnoVDxXsq1UHyyhf8ourKY
Fc3QRFsnhjEbBp3eA6caMKHf4kjW6ak3n0VTGTdAnoAHZMZvL88b58a/FjOmRDhk
JuMm765ZQIDc8Oww8vAyP/5CtOAOZHnpaNfdQmtYWQKBgQDnpso0/cOrxIMTGvgC
YkVcaXM5uEaxotSPElGByXlyCAPuLj1FY1Tn/YjFq1gHuXVkrvf2FWpABKlLsxsi
IffNafINJ9yFkZm2+yFoSdbkAJjPs6CFL5Rz5lpngRt61wIIs1HWIEXSEUFmWwvn
Dp+7eHNfrwOwGtITVkaNzsW1uQKBgQDhSc6pGdm9d6YvZmvWxD53PvH6LkD1Wi/l
7WhghZcMZnKGdqYo5i0ErqL/uB9trPCB8RHJ6k+IRPw51uVSRCqaB8CcOc9t+/X7
R49nk4/AbfL2beNkiQ3iBEfm8VGizbOwtIsD5kwX0WhUoASoQfekcMfEtOD8FDby
2rV1G1olUwKBgQDg0ML/aKQkqGOZABxfm+W6yPA1vTyGmwN1Eo8dyV1eqJ6/4go+
tCcruqepRx/45ykiHJizL5ddJ+jxeZJErviiV0a9M8lVq7elmNWUM80km9ux/58y
grmtPYrRVsBHmDD2JqmuiMWaGHsDDitgA5ZLx5sY6m5G09C7SYjRiTHhqQKBgQC5
cmL+Sbw1vVb776hXSoVM7XCMiySHkqyY7RsdUPosh+xACUzvn8Fw+DxOmYatZk03
+XO2LP044+hnyCBO3rPexQDk2KWKGHy8UoSWMx6CvT7hYUVsTXEzGmkBjqWviPog
PA2usRT0CjmCZ5JN2rd6EigvIOGOPymYMTZrZo+G5wKBgCf3x+lrpzYjFsq0HlVQ
JvgNyZWQuCm08WrXivu3CUHIUsL25XvwpGxny5ktgwyDnxp38UMeuiRxJzCzQg71
LZnvpxk4LhDNBuHbbx4qZcT0d4OJhB2ChqQn3Njq21FtrDAxeGdHqxsFNgXz0p4m
6ZscfnyYMl9rH581Z1v0CImR
-----END PRIVATE KEY-----";

const KID: &str = "test-key-1";

fn config() -> ZitadelAuthConfig {
    ZitadelAuthConfig::new(
        "https://auth.example.com",
        "https://auth.example.com",
        "client",
        "secret",
    )
    .without_introspection()
}

fn b64(bytes: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
}

fn claims(exp: i64) -> Value {
    json!({
        "sub": "550e8400-e29b-41d4-a716-446655440000",
        "preferred_username": "victim@example.com",
        "email": "admin@example.com",
        "roles": ["admin"],
        "iss": "https://auth.example.com",
        "exp": exp,
    })
}

fn signed_token(exp: i64) -> String {
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(KID.to_string());
    encode(
        &header,
        &claims(exp),
        &EncodingKey::from_rsa_pem(TEST_PRIVATE_PEM.as_bytes()).expect("test signing key"),
    )
    .expect("test signed token")
}

fn unsigned_token(exp: i64) -> String {
    let header = b64(br#"{"alg":"none","typ":"JWT","kid":"test-key-1"}"#);
    let payload = b64(&serde_json::to_vec(&claims(exp)).expect("claims json"));
    format!("{header}.{payload}.")
}

fn seed_valid_key(cfg: &ZitadelAuthConfig) {
    let uri = jwks_uri(cfg).expect("test jwks uri");
    let key = DecodingKey::from_rsa_pem(TEST_PUBLIC_PEM.as_bytes()).expect("test public key");
    seed_key(&uri, KID, key);
}

#[test]
fn jwks_uri_is_derived_from_the_issuer() {
    assert_eq!(
        jwks_uri(&config()).expect("jwks uri"),
        "https://auth.example.com/oauth/v2/keys"
    );
}

#[test]
fn alg_none_payload_is_rejected_without_a_signature() {
    let cfg = config();
    let key = DecodingKey::from_rsa_pem(TEST_PUBLIC_PEM.as_bytes()).expect("test public key");
    let token = unsigned_token(Utc::now().timestamp() + 3600);
    assert!(verify_with_key(&token, &key, &cfg).is_err());
}

#[test]
fn tampered_signature_is_rejected() {
    let cfg = config();
    let key = DecodingKey::from_rsa_pem(TEST_PUBLIC_PEM.as_bytes()).expect("test public key");
    let mut token = signed_token(Utc::now().timestamp() + 3600).into_bytes();
    let last = token.last_mut().expect("non-empty token");
    *last = if *last == b'A' { b'B' } else { b'A' };
    let token = String::from_utf8(token).expect("still utf8");
    assert!(verify_with_key(&token, &key, &cfg).is_err());
}

#[test]
fn token_signed_by_another_key_is_rejected() {
    let cfg = config();
    let key = DecodingKey::from_rsa_pem(TEST_PUBLIC_PEM.as_bytes()).expect("test public key");
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(KID.to_string());
    let forged = encode(
        &header,
        &claims(Utc::now().timestamp() + 3600),
        &EncodingKey::from_rsa_pem(FORGED_PRIVATE_PEM.as_bytes()).expect("attacker key"),
    )
    .expect("attacker signed token");
    assert!(verify_with_key(&forged, &key, &cfg).is_err());
}

#[test]
fn expired_token_is_rejected() {
    let cfg = config();
    let key = DecodingKey::from_rsa_pem(TEST_PUBLIC_PEM.as_bytes()).expect("test public key");
    let token = signed_token(Utc::now().timestamp() - 3600);
    let result = verify_with_key(&token, &key, &cfg);
    assert!(matches!(result, Err(AuthError::ExpiredToken)));
}

#[test]
fn valid_signed_token_yields_the_verified_claims() {
    let cfg = config();
    let key = DecodingKey::from_rsa_pem(TEST_PUBLIC_PEM.as_bytes()).expect("test public key");
    let token = signed_token((Utc::now() + Duration::minutes(5)).timestamp());
    let claims = verify_with_key(&token, &key, &cfg).expect("valid token");
    assert_eq!(claims["sub"], "550e8400-e29b-41d4-a716-446655440000");
}

#[test]
fn keys_from_jwks_skips_non_signing_and_broken_keys() {
    let body = serde_json::json!({
        "keys": [
            { "kty": "RSA", "kid": "good", "use": "sig", "alg": "RS256", "n": "0vx7agoebGcQSuuPiLJXZptN9nndrQmbXEps2aiAFbWhM78LhWx4cbbfAAtVT86zwu1RK7aPFFxuhDR1L6tSoc_BJECPebWKRXjBZCiFV4n3oknjhMstn64tZ_2W-5JsGY4Hc5n9yBXArwl93lqt7_RN5w6Cf0h4QyQ5v-65YGjQR0_FDW2QvzqY368QQMicAtaSqzs8KJZgnYb9c7d0zgdAZHzu6qMQvRL5hajrn1n91CbOpbISD08qNLyrdkt-bFTWhAI4vMQFh6WeZu0fM4lFd2NcRwr3XPksINHaQ-G_xBniIqbw0Ls1jF44-csFCur-kEgU8awapJzKnqDKgw", "e": "AQAB" },
            { "kty": "RSA", "kid": "enc", "use": "enc", "alg": "RS256", "n": "abc", "e": "AQAB" },
            { "kty": "EC", "kid": "ec", "use": "sig", "n": "abc", "e": "AQAB" },
            { "kty": "RSA", "kid": "broken", "use": "sig", "n": "!!!", "e": "???" }
        ]
    })
    .to_string();
    let keys = keys_from_jwks(&body).expect("one usable key");
    assert_eq!(keys.len(), 1);
    assert!(keys.contains_key("good"));
}

#[tokio::test]
async fn decode_jwt_user_rejects_forged_and_unsigned_payloads() {
    let cfg = config();
    let provider = ZitadelAuthProvider::new(cfg.clone()).expect("provider");
    seed_valid_key(&cfg);

    // An attacker's payload with admin claims but no signature.
    let unsigned = unsigned_token((Utc::now() + Duration::minutes(5)).timestamp());
    assert!(provider.decode_jwt_user(&unsigned).await.is_err());

    // A correctly-shaped token whose signature was tampered with.
    let mut signed = signed_token((Utc::now() + Duration::minutes(5)).timestamp()).into_bytes();
    let last = signed.last_mut().expect("non-empty token");
    *last = if *last == b'A' { b'B' } else { b'A' };
    let tampered = String::from_utf8(signed).expect("still utf8");
    assert!(provider.decode_jwt_user(&tampered).await.is_err());

    // A validly signed token is accepted (proves the key path works).
    let valid = signed_token((Utc::now() + Duration::minutes(5)).timestamp());
    assert!(provider.decode_jwt_user(&valid).await.is_ok());
}

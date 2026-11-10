//! JWT verification boundary, exercised offline at library level (#2112).
//!
//! `botsecurity_core::verified_jwt` is the single place that turns a Bearer
//! token into an identity. Handlers used to base64url-decode the payload and
//! trust `sub`/`user_id`, so a forged `{"alg":"none"}` token — or one signed
//! with any other key — could name another tenant's branch. These tests drive
//! the public verifier with tokens minted here, so the accept/reject boundary
//! is pinned without a server, a database or the process environment.
//!
//! `verify_saas_jwt` is deliberately not exercised: it resolves the secret from
//! `directory_config.json` / the environment and the offline suite must stay
//! hermetic. It is `verify_hs256_jwt` with the process secret supplied, so
//! every rule below applies to the SaaS path unchanged.

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use botsecurity_core::verified_jwt::{bearer_token, verify_hs256_jwt, JwtVerifyError};
use hmac::{Hmac, Mac};
use http::HeaderMap;
use sha2::Sha256;
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

/// Key the "server" holds in this module.
const SECRET: &str = "offline-suite-jwt-secret-0123456789abcdef";
/// Key an attacker holds: it must never verify against [`SECRET`].
const OTHER_SECRET: &str = "attacker-controlled-key-0123456789abcdef";

fn b64(input: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(input)
}

/// Mints the signature half of a token, the way the login path does.
fn sign(signing_input: &str, secret: &str) -> String {
    match HmacSha256::new_from_slice(secret.as_bytes()) {
        Ok(mut mac) => {
            mac.update(signing_input.as_bytes());
            b64(&mac.finalize().into_bytes())
        }
        // Unreachable for HMAC (any key length is valid); an empty signature
        // makes the token fail verification instead of panicking the suite.
        Err(_) => String::new(),
    }
}

fn token_with_header(header: &str, payload: &str, secret: &str) -> String {
    let signing_input = format!("{}.{}", b64(header.as_bytes()), b64(payload.as_bytes()));
    let signature = sign(&signing_input, secret);
    format!("{signing_input}.{signature}")
}

fn token(payload: &str, secret: &str) -> String {
    token_with_header(r#"{"alg":"HS256","typ":"JWT"}"#, payload, secret)
}

fn in_an_hour() -> i64 {
    chrono::Utc::now().timestamp() + 3600
}

fn an_hour_ago() -> i64 {
    chrono::Utc::now().timestamp() - 3600
}

#[test]
fn a_valid_token_yields_the_signed_identity() {
    let org = Uuid::new_v4();
    let branch = Uuid::new_v4();
    let payload = format!(
        r#"{{"sub":"zitadel-user-1","email":"admin@example.com","org_id":"{org}","branch_id":"{branch}","exp":{}}}"#,
        in_an_hour()
    );

    let claims = verify_hs256_jwt(&token(&payload, SECRET), SECRET).expect("a signed token verifies");

    assert_eq!(claims.sub, "zitadel-user-1");
    assert_eq!(claims.email.as_deref(), Some("admin@example.com"));
    assert_eq!(claims.org_id, Some(org));
    assert_eq!(claims.branch_id, Some(branch));
    assert!(claims.exp > chrono::Utc::now().timestamp());
}

#[test]
fn a_tampered_payload_cannot_promote_itself() {
    let original = token(
        &format!(r#"{{"sub":"user-1","exp":{}}}"#, in_an_hour()),
        SECRET,
    );
    let parts: Vec<&str> = original.split('.').collect();
    assert_eq!(parts.len(), 3, "a minted token has three segments");

    // Same header, same signature, forged claims: the classic privilege
    // escalation. The signature no longer covers the payload.
    let forged_payload = b64(
        format!(
            r#"{{"sub":"admin","branch_id":"{}","exp":{}}}"#,
            Uuid::new_v4(),
            in_an_hour()
        )
        .as_bytes(),
    );
    let tampered = format!("{}.{}.{}", parts[0], forged_payload, parts[2]);

    assert_eq!(
        verify_hs256_jwt(&tampered, SECRET),
        Err(JwtVerifyError::InvalidSignature),
        "a re-encoded payload must not be accepted"
    );
}

#[test]
fn an_unsigned_alg_none_token_is_refused() {
    let payload = format!(r#"{{"sub":"admin","exp":{}}}"#, in_an_hour());

    // No signature segment at all.
    let unsigned = format!(
        "{}.{}.",
        b64(br#"{"alg":"none","typ":"JWT"}"#),
        b64(payload.as_bytes())
    );
    assert_eq!(
        verify_hs256_jwt(&unsigned, SECRET),
        Err(JwtVerifyError::UnsupportedAlgorithm)
    );

    // A signature was computed, but the header still claims `none`.
    let forged = token_with_header(r#"{"alg":"none","typ":"JWT"}"#, &payload, SECRET);
    assert_eq!(
        verify_hs256_jwt(&forged, SECRET),
        Err(JwtVerifyError::UnsupportedAlgorithm),
        "only HS256 may be accepted, whatever the signature looks like"
    );
}

#[test]
fn an_asymmetric_algorithm_is_refused() {
    let payload = format!(r#"{{"sub":"user-1","exp":{}}}"#, in_an_hour());
    let rs256 = token_with_header(r#"{"alg":"RS256","typ":"JWT"}"#, &payload, SECRET);
    assert_eq!(
        verify_hs256_jwt(&rs256, SECRET),
        Err(JwtVerifyError::UnsupportedAlgorithm)
    );
}

#[test]
fn a_token_signed_with_another_key_is_refused() {
    let payload = format!(r#"{{"sub":"admin","exp":{}}}"#, in_an_hour());
    let wrong_key = token(&payload, OTHER_SECRET);

    assert_eq!(
        verify_hs256_jwt(&wrong_key, SECRET),
        Err(JwtVerifyError::InvalidSignature),
        "a token minted with another key must not identify anyone"
    );
}

#[test]
fn an_expired_token_is_refused() {
    let payload = format!(r#"{{"sub":"user-1","exp":{}}}"#, an_hour_ago());
    assert_eq!(
        verify_hs256_jwt(&token(&payload, SECRET), SECRET),
        Err(JwtVerifyError::Expired)
    );
}

#[test]
fn a_token_without_expiry_or_subject_is_refused() {
    let no_expiry = r#"{"sub":"user-1"}"#;
    assert_eq!(
        verify_hs256_jwt(&token(no_expiry, SECRET), SECRET),
        Err(JwtVerifyError::MissingExpiry),
        "an unbounded token cannot be trusted"
    );

    let no_subject = format!(r#"{{"email":"user@example.com","exp":{}}}"#, in_an_hour());
    assert_eq!(
        verify_hs256_jwt(&token(&no_subject, SECRET), SECRET),
        Err(JwtVerifyError::MissingSubject)
    );

    let blank_subject = format!(r#"{{"sub":"   ","exp":{}}}"#, in_an_hour());
    assert_eq!(
        verify_hs256_jwt(&token(&blank_subject, SECRET), SECRET),
        Err(JwtVerifyError::MissingSubject)
    );
}

#[test]
fn a_future_not_before_claim_is_refused_but_the_leeway_is_honoured() {
    let now = chrono::Utc::now().timestamp();

    let far_future = format!(r#"{{"sub":"user-1","exp":{},"nbf":{}}}"#, now + 7200, now + 3600);
    assert_eq!(
        verify_hs256_jwt(&token(&far_future, SECRET), SECRET),
        Err(JwtVerifyError::NotYetValid)
    );

    let within_leeway = format!(r#"{{"sub":"user-1","exp":{},"nbf":{}}}"#, now + 3600, now + 5);
    assert!(
        verify_hs256_jwt(&token(&within_leeway, SECRET), SECRET).is_ok(),
        "clock skew inside the leeway must not lock a user out"
    );
}

#[test]
fn an_empty_secret_never_verifies() {
    let payload = format!(r#"{{"sub":"user-1","exp":{}}}"#, in_an_hour());
    // Signed with the empty key, so the only thing that can reject it is the
    // "no secret configured" rule itself.
    let unsigned = token(&payload, "");

    assert_eq!(
        verify_hs256_jwt(&unsigned, ""),
        Err(JwtVerifyError::MissingSecret)
    );
}

#[test]
fn the_legacy_user_id_and_organization_id_claims_are_honoured() {
    let org = Uuid::new_v4();
    let payload = format!(
        r#"{{"user_id":"42","organization_id":"{org}","exp":{}}}"#,
        in_an_hour()
    );

    let claims = verify_hs256_jwt(&token(&payload, SECRET), SECRET).expect("legacy claims verify");
    assert_eq!(claims.sub, "42");
    assert_eq!(claims.org_id, Some(org));
    assert_eq!(claims.branch_id, None, "a token without a branch scopes nothing");
}

#[test]
fn malformed_tokens_are_errors_and_never_panics() {
    for value in ["", "a", "a.b", "..", "!!!.payload.signature", "a.@@@.c"] {
        assert!(
            verify_hs256_jwt(value, SECRET).is_err(),
            "{value:?} must be refused"
        );
    }
}

#[test]
fn the_bearer_token_is_read_from_the_authorization_header() {
    let mut bearer = HeaderMap::new();
    bearer.insert(
        "authorization",
        "Bearer header.payload.signature,extra".parse().expect("a header value"),
    );
    assert_eq!(bearer_token(&bearer), Some("header.payload.signature"));

    let mut basic = HeaderMap::new();
    basic.insert("authorization", "Basic dXNlcjpwYXNz".parse().expect("a header value"));
    assert_eq!(bearer_token(&basic), None, "only a Bearer credential is a token");

    assert_eq!(bearer_token(&HeaderMap::new()), None);
}

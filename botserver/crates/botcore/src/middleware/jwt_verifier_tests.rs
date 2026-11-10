//! Request-JWT verifier tests (issue #2125).
//!
//! The middleware used to try HS256 and then RS256 with the *same* bytes, so a
//! deployment that put a public key in the HMAC slot accepted tokens signed
//! HS256 with that public key. These tests pin the explicit split: the key
//! variant decides the algorithm and there is no cross-algorithm fallback.

use super::require_role_middleware::{validate_jwt, validate_jwt_with_key, JwtVerifierKey};
use super::TokenClaims;

/// Public half of a throwaway test keypair. Public keys are not secrets; the
/// matching private key is never needed here because every assertion is about
/// rejection.
const TEST_RSA_PUBLIC_PEM: &str = "-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAwJuONrtzHKIjiqICks9P
Pj3vSmbgu6nRnnaCZ5OdP9AJf9HNkD0vSpEBcv4DR5ipwRYb6A+vU1n4CxyF3mV5
onrQaglkkViQqQ7mC9oKxUoEjy1N9q5FwfZ9wbXMl30vKymAU1uRsZQV/WuybQUR
x2mjW17WCxUw942KVCo4gMvIzDNfcMRhDQNlm/HJdOhmiRQ7pZw291aYVd9+1loh
TBX30mvfK7dYKlZwK3Ya8JBe3o6iwvTR55emoDmhHKfbYX6Luxt7JIERDG9sydEO
uDXMR+cQMziQORyKONoWNme00b6v8WDXaUglxwhMNtSwYTrAU/7tBFs1UFJeexo4
xQIDAQAB
-----END PUBLIC KEY-----";

const HMAC_SECRET: &str = "test-secret-key-for-testing-only-32-chars!";

fn claims() -> TokenClaims {
    TokenClaims {
        sub: uuid::Uuid::new_v4().to_string(),
        exp: chrono::Utc::now().timestamp() + 300,
        iat: chrono::Utc::now().timestamp(),
        iss: None,
        aud: None,
        scope: None,
        org_id: None,
        branch_id: None,
    }
}

fn b64(bytes: &[u8]) -> String {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
    URL_SAFE_NO_PAD.encode(bytes)
}

/// HS256 token whose HMAC key is `key` — the confusion attack when `key` is an
/// RSA public key.
fn hs256_signed_with(key: &[u8]) -> String {
    use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
    encode(
        &Header::new(Algorithm::HS256),
        &claims(),
        &EncodingKey::from_secret(key),
    )
    .expect("encode HS256 test token")
}

#[test]
fn opaque_secret_configures_hmac() {
    let key = JwtVerifierKey::from_secret(HMAC_SECRET).expect("valid secret");
    assert_eq!(key, JwtVerifierKey::Hs256(HMAC_SECRET.as_bytes().to_vec()));
    assert_eq!(key.algorithm(), jsonwebtoken::Algorithm::HS256);
}

#[test]
fn pem_material_configures_rsa_and_never_hmac() {
    let key = JwtVerifierKey::from_secret(TEST_RSA_PUBLIC_PEM).expect("valid PEM");
    assert!(matches!(key, JwtVerifierKey::Rs256(_)));
    assert_eq!(key.algorithm(), jsonwebtoken::Algorithm::RS256);
}

#[test]
fn empty_secret_is_rejected() {
    assert!(JwtVerifierKey::from_secret("   ").is_err());
}

#[test]
fn hs256_signed_with_rsa_public_key_is_rejected_by_rsa_config() {
    let forged = hs256_signed_with(TEST_RSA_PUBLIC_PEM.as_bytes());
    let rsa_config = JwtVerifierKey::from_secret(TEST_RSA_PUBLIC_PEM).expect("valid PEM");

    assert!(
        validate_jwt_with_key(&forged, &rsa_config).is_err(),
        "an RS256-configured verifier must reject HS256 tokens"
    );

    // Why the resolver refuses PEM material in the HMAC slot: an HS256
    // verifier handed the public key bytes *would* accept the forgery.
    let misconfigured = JwtVerifierKey::Hs256(TEST_RSA_PUBLIC_PEM.as_bytes().to_vec());
    assert!(validate_jwt_with_key(&forged, &misconfigured).is_ok());
}

#[test]
fn rs256_header_token_is_rejected_by_hmac_config() {
    // Signature is irrelevant: the algorithm check fires first, which is the
    // property under test (no fallback to RS256 for HS256 deployments).
    let header = b64(br#"{"alg":"RS256","typ":"JWT"}"#);
    let payload = b64(&serde_json::to_vec(&claims()).expect("claims json"));
    let token = format!("{header}.{payload}.{}", b64(b"not-a-real-signature"));

    assert!(validate_jwt(&token, HMAC_SECRET).is_err());
}

#[test]
fn alg_none_is_rejected_by_both_configs() {
    let header = b64(br#"{"alg":"none","typ":"JWT"}"#);
    let payload = b64(&serde_json::to_vec(&claims()).expect("claims json"));
    let token = format!("{header}.{payload}.");

    assert!(validate_jwt(&token, HMAC_SECRET).is_err());
    assert!(validate_jwt(&token, TEST_RSA_PUBLIC_PEM).is_err());
}

#[test]
fn one_byte_signature_difference_fails_without_panicking() {
    let mut token = hs256_signed_with(HMAC_SECRET.as_bytes());
    let last = token.pop().expect("non-empty token");
    token.push(if last == 'A' { 'B' } else { 'A' });

    assert!(validate_jwt(&token, HMAC_SECRET).is_err());
}

#[test]
fn valid_hs256_token_is_accepted() {
    let token = hs256_signed_with(HMAC_SECRET.as_bytes());
    let claims = validate_jwt(&token, HMAC_SECRET).expect("valid HS256 token");
    assert!(!claims.sub.is_empty());
}

//! Bot Framework bearer-token and key-set tests (#2113).

use jsonwebtoken::{Algorithm, EncodingKey, Header};

use super::fixtures::*;
use super::jwks;
use super::jwt;
use super::*;

#[test]
fn a_valid_bot_framework_token_is_accepted() {
    let token = sign(&claims(APP_ID, jwt::BOT_FRAMEWORK_ISSUER, 600, -10));

    assert_eq!(
        jwt::verify_with_key(&token, APP_ID, &fixture_public_key()),
        Ok(())
    );
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
    let token = sign(&claims(
        "99999999-0000-0000-0000-000000000000",
        jwt::BOT_FRAMEWORK_ISSUER,
        600,
        -10,
    ));

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

    assert_eq!(
        jwt::verify(&token, APP_ID).await,
        Err(AuthError::InvalidToken)
    );
}

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
    // what makes the verifier refresh once and then reject.
    assert!(!keys.contains_key("unknown-kid"));
    // A non-RSA key is not a verification key for RS256.
    assert!(!keys.contains_key("ec-key"));
}

#[test]
fn a_key_set_without_a_usable_key_is_an_error() {
    let empty = serde_json::json!({"keys": []}).to_string();
    let broken =
        serde_json::json!({"keys": [{"kty": "RSA", "kid": "k", "n": "", "e": ""}]}).to_string();

    assert!(jwks::keys_from_jwks(&empty).is_err());
    assert!(jwks::keys_from_jwks(&broken).is_err());
    assert!(jwks::keys_from_jwks("not json").is_err());
}

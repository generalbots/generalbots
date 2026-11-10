//! Header gate and outgoing-webhook HMAC tests (#2113).

use axum::http::{HeaderMap, StatusCode};
use base64::Engine as _;
use hmac::{Hmac, Mac};
use sha2::Sha256;

use super::fixtures::*;
use super::hmac as teams_hmac;
use super::*;

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
async fn a_malformed_bearer_token_is_rejected() {
    let headers = with_authorization("Bearer not-a-jwt");

    assert_eq!(
        authorize(&headers, BODY, &credentials()).await,
        Err(AuthError::InvalidToken)
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

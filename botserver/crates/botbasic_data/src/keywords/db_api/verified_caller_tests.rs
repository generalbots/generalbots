//! Tests for the verified caller resolver (#2122).
//!
//! The accept/reject boundary is exercised against real HS256 tokens signed with
//! the secret this process verifies with, so "valid assertion accepted" means
//! the same thing it means in production.

use super::*;
use axum::http::{HeaderMap, StatusCode};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use botsecurity::auth::{AuthenticatedUser, Role};
use botsecurity::botsecurity_core::verified_jwt::resolve_saas_jwt_secret;
use hmac::{Hmac, Mac};
use serde_json::{json, Value};
use sha2::Sha256;
use uuid::Uuid;

fn secret() -> String {
    resolve_saas_jwt_secret().expect("saas jwt secret")
}

/// Mints an HS256 token the way the login path and `api.exec` do.
fn mint(claims: &Value, secret: &str) -> String {
    let header = URL_SAFE_NO_PAD.encode(br#"{"alg":"HS256","typ":"JWT"}"#);
    let payload = URL_SAFE_NO_PAD.encode(claims.to_string().as_bytes());
    let signing_input = format!("{header}.{payload}");
    let mut mac = match Hmac::<Sha256>::new_from_slice(secret.as_bytes()) {
        Ok(mac) => mac,
        Err(_) => return String::new(),
    };
    mac.update(signing_input.as_bytes());
    format!(
        "{signing_input}.{}",
        URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes())
    )
}

fn headers_with(pairs: &[(&'static str, String)]) -> HeaderMap {
    let mut headers = HeaderMap::new();
    for (name, value) in pairs {
        if let Ok(parsed) = axum::http::HeaderValue::from_str(value) {
            headers.insert(*name, parsed);
        }
    }
    headers
}

fn bearer_headers(token: &str) -> HeaderMap {
    headers_with(&[("authorization", format!("Bearer {token}"))])
}

/// The claims `api_exec_assertion::mint` puts on the loopback hop.
fn assertion_claims(user_id: Uuid) -> Value {
    json!({
        "sub": user_id.to_string(),
        "email": "operator@example.com",
        "bot_id": Uuid::new_v4().to_string(),
        "gb_aud": "api-exec",
        "scope": "api-exec",
        "iat": chrono::Utc::now().timestamp(),
        "exp": chrono::Utc::now().timestamp() + 120,
    })
}

#[test]
fn header_only_request_is_rejected() {
    // The exact bypass #2122 flagged: identity and roles claimed in headers.
    let headers = headers_with(&[
        ("x-user-id", Uuid::new_v4().to_string()),
        ("x-user-roles", "admin;owner".to_string()),
    ]);

    assert_eq!(
        caller_from(None, &headers),
        Err(DbApiAuthError::Unauthenticated)
    );
}

#[test]
fn a_rejection_is_a_401() {
    let response = DbApiAuthError::Unauthenticated.into_response();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[test]
fn valid_assertion_is_accepted() {
    let user_id = Uuid::new_v4();
    let token = mint(&assertion_claims(user_id), &secret());

    assert_eq!(
        caller_from(None, &bearer_headers(&token)),
        Ok(VerifiedCaller {
            user_id,
            email: Some("operator@example.com".to_string()),
            // The assertion carries no role claim: roles come from RBAC groups.
            roles: Vec::new(),
        })
    );
}

#[test]
fn wrong_signature_is_rejected() {
    let token = mint(
        &assertion_claims(Uuid::new_v4()),
        "an-attacker-controlled-secret",
    );

    assert_eq!(
        caller_from(None, &bearer_headers(&token)),
        Err(DbApiAuthError::Unauthenticated)
    );
}

#[test]
fn a_tampered_payload_is_rejected() {
    let token = mint(&assertion_claims(Uuid::new_v4()), &secret());
    let mut parts: Vec<String> = token.split('.').map(str::to_string).collect();
    parts[1] = URL_SAFE_NO_PAD.encode(
        json!({
            "sub": Uuid::new_v4().to_string(),
            "exp": chrono::Utc::now().timestamp() + 120,
        })
        .to_string()
        .as_bytes(),
    );
    let forged = parts.join(".");

    assert_eq!(
        caller_from(None, &bearer_headers(&forged)),
        Err(DbApiAuthError::Unauthenticated)
    );
}

#[test]
fn an_expired_assertion_is_rejected() {
    let mut claims = assertion_claims(Uuid::new_v4());
    claims["exp"] = Value::from(1);
    let token = mint(&claims, &secret());

    assert_eq!(
        caller_from(None, &bearer_headers(&token)),
        Err(DbApiAuthError::Unauthenticated)
    );
}

#[test]
fn an_unsecured_token_is_rejected() {
    let payload = URL_SAFE_NO_PAD.encode(
        json!({
            "sub": Uuid::new_v4().to_string(),
            "exp": chrono::Utc::now().timestamp() + 120,
        })
        .to_string()
        .as_bytes(),
    );
    let header = URL_SAFE_NO_PAD.encode(br#"{"alg":"none","typ":"JWT"}"#);
    let forged = format!("{header}.{payload}.");

    assert_eq!(
        caller_from(None, &bearer_headers(&forged)),
        Err(DbApiAuthError::Unauthenticated)
    );
}

#[test]
fn an_anonymous_extension_never_becomes_a_caller() {
    // The middleware inserts an anonymous principal on public paths; it must
    // not be usable as an identity here.
    let headers = headers_with(&[("x-user-roles", "admin".to_string())]);
    let anonymous = AuthenticatedUser::anonymous();

    assert_eq!(
        VerifiedCaller::resolve(Some(&anonymous), &headers),
        Err(DbApiAuthError::Unauthenticated)
    );
}

#[test]
fn the_verified_extension_supplies_identity_and_roles() {
    let user_id = Uuid::new_v4();
    let user = AuthenticatedUser::new(user_id, "operator".to_string())
        .with_email("operator@example.com")
        .with_roles(vec![Role::User, Role::Admin]);
    // Headers claiming another identity must not influence the result.
    let headers = headers_with(&[
        ("x-user-id", Uuid::new_v4().to_string()),
        ("x-user-roles", "anonymous".to_string()),
    ]);

    let caller = VerifiedCaller::resolve(Some(&user), &headers);
    assert_eq!(
        caller,
        Ok(VerifiedCaller {
            user_id,
            email: Some("operator@example.com".to_string()),
            roles: vec!["user".to_string(), "admin".to_string()],
        })
    );
}

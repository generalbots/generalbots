//! Shared test helpers for the plan identity/authorization tests (#2136).
//!
//! Tokens are minted HS256 with the secret the verifier resolves in this
//! process, so every test exercises the same accept/reject boundary production
//! uses — there is no "test-only" bypass.

use crate::auth::Caller;
use axum::http::{HeaderMap, HeaderValue};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use botsecurity_core::verified_jwt::resolve_saas_jwt_secret;
use chrono::Utc;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use uuid::Uuid;

pub(crate) fn b64url(bytes: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Signs `claims` as HS256 with `secret`, the way the login path mints tokens.
pub(crate) fn sign_with(claims: serde_json::Value, secret: &str) -> String {
    let header = b64url(br#"{"alg":"HS256","typ":"JWT"}"#);
    let payload = b64url(claims.to_string().as_bytes());
    let signing_input = format!("{header}.{payload}");
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).expect("hmac key");
    mac.update(signing_input.as_bytes());
    format!("{signing_input}.{}", b64url(&mac.finalize().into_bytes()))
}

/// A token signed with the secret the process verifies against.
pub(crate) fn token(claims: serde_json::Value) -> String {
    sign_with(claims, &resolve_saas_jwt_secret().expect("saas jwt secret"))
}

/// Claims for `sub`, optionally scoped to a tenant branch.
pub(crate) fn live_claims(sub: &str, branch: Option<Uuid>) -> serde_json::Value {
    let mut claims = serde_json::json!({
        "sub": sub,
        "email": format!("{sub}@example.test"),
        "exp": Utc::now().timestamp() + 3600,
    });
    if let Some(branch) = branch {
        claims["branch_id"] = serde_json::json!(branch.to_string());
    }
    claims
}

/// A verified bearer header for `sub`.
pub(crate) fn headers_for(sub: &str, branch: Option<Uuid>) -> HeaderMap {
    let mut headers = HeaderMap::new();
    let value = HeaderValue::from_str(&format!(
        "Bearer {}",
        token(live_claims(sub, branch))
    ))
    .expect("header value");
    headers.insert("authorization", value);
    headers
}

/// A caller as the verifier would produce it (for pure authorization tests).
pub(crate) fn caller(user_id: &str, branch: Option<Uuid>) -> Caller {
    Caller {
        user_id: user_id.to_string(),
        email: None,
        org_id: None,
        branch_id: branch,
    }
}

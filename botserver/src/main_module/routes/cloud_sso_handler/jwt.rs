//! JWT primitives shared by the SSO handlers.
//!
//! Duplicated from `botcloud::api` on purpose: the routes layer must not depend
//! on the cloud crate just to verify a signature.
//!
//! #2140 — the signing secret has exactly one resolver,
//! `botsecurity_core::verified_jwt::resolve_saas_jwt_secret`. The handlers call
//! it directly so a failure is visible at the call site (and fails closed)
//! instead of being hidden behind a second wrapper with its own fallback.

/// HMAC-SHA256 sign a message and return the base64url-encoded signature.
/// Duplicado de botcloud::api::jwt_sign_inner para evitar dependência entre crates.
pub(super) fn jwt_sign_inner(message: &str, secret: &[u8]) -> Result<String, String> {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    let mut mac =
        Hmac::<Sha256>::new_from_slice(secret).map_err(|e| format!("HMAC key error: {}", e))?;
    mac.update(message.as_bytes());
    Ok(base64_url_encode(&mac.finalize().into_bytes()))
}

pub(super) fn base64_url_encode(input: &[u8]) -> String {
    use base64::{engine::general_purpose, Engine as _};
    general_purpose::STANDARD
        .encode(input)
        .replace('+', "-")
        .replace('/', "_")
        .trim_end_matches('=')
        .to_string()
}

pub(super) fn base64_url_decode(input: &str) -> Result<Vec<u8>, &'static str> {
    let raw = input.replace('-', "+").replace('_', "/");
    let raw = match raw.len() % 4 {
        2 => raw + "==",
        3 => raw + "=",
        0 => raw,
        _ => return Err("invalid base64 input length"),
    };
    use base64::{engine::general_purpose, Engine as _};
    general_purpose::STANDARD
        .decode(&raw)
        .map_err(|_| "base64 decode failed")
}

/// Extract the `org_id` claim from a decoded cloud JWT so suite sessions
/// carry the caller's organization scope (fix #1268). Returns `None` for
/// tokens minted without an org claim.
pub(super) fn claims_org(claims: &serde_json::Value) -> Option<String> {
    claims
        .get("org_id")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}

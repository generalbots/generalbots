//! Verified JWT identity for the browser-task API.
//!
//! #2112 — this module used to base64url-decode the token payload and trust
//! `uid`/`sub` without ever looking at the signature, so a forged token (or a
//! token signed with another key) could name any user. Every downstream policy
//! decision — task ownership, browsing memory, the admin policy gate — was
//! therefore made on unverified claims.
//!
//! The decoder is gone. Claims are produced only by
//! [`botsecurity_core::verified_jwt::verify_saas_jwt`], which requires an
//! `HS256` signature over `header.payload`, a non-empty `sub`/`user_id` claim
//! and an `exp` in the future. Anything else yields `None`, and every caller
//! turns `None` into `401 Unauthorized`: a token that cannot be verified
//! contributes no identity at all, so the policy path fails closed.
//!
//! Tests mint real HS256 tokens (see `test_support`) instead of hand-built
//! payloads, because a token with a placeholder signature is exactly what the
//! verifier must refuse.

use botsecurity_core::verified_jwt::{verify_saas_jwt, VerifiedJwt};

pub(crate) use botsecurity_core::verified_jwt::bearer_token;

/// Claims of a Bearer token, or `None` when the token cannot be trusted.
///
/// This is the only place in this crate where claims are read. Malformed,
/// unsigned (`alg:none`), tampered, foreign-key, expired and subject-less
/// tokens all return `None` — there is no partial-claims path.
pub(crate) fn verified_claims(token: &str) -> Option<VerifiedJwt> {
    let token = token.trim();
    if token.is_empty() {
        return None;
    }
    match verify_saas_jwt(token) {
        Ok(claims) => Some(claims),
        Err(reason) => {
            tracing::warn!("browser policy: token rejected: {reason}");
            None
        }
    }
}

#[cfg(test)]
#[path = "jwt_tests.rs"]
mod tests;

/// Test-only token minting, shared by the JWT and HTTP-surface tests.
///
/// Tokens are signed with the secret this process verifies against, so a test
/// exercises exactly the accept/reject boundary production uses — including
/// the signature check, which a hand-rolled payload would skip.
#[cfg(test)]
pub(crate) mod test_support {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    const HS256_HEADER: &str = r#"{"alg":"HS256","typ":"JWT"}"#;

    /// Secret for fixtures that must be rejected (wrong key).
    pub(crate) const OTHER_SECRET: &str = "another-key-that-nobody-should-accept-9876";

    pub(crate) fn b64(input: &[u8]) -> String {
        URL_SAFE_NO_PAD.encode(input)
    }

    pub(crate) fn sign(signing_input: &str, secret: &str) -> String {
        match Hmac::<Sha256>::new_from_slice(secret.as_bytes()) {
            Ok(mut mac) => {
                mac.update(signing_input.as_bytes());
                b64(&mac.finalize().into_bytes())
            }
            Err(_) => String::new(),
        }
    }

    /// A token with an arbitrary header segment, signed with `secret`.
    pub(crate) fn token_with_header(header: &str, payload: &str, secret: &str) -> String {
        let signing_input = format!("{}.{}", b64(header.as_bytes()), b64(payload.as_bytes()));
        let signature = sign(&signing_input, secret);
        format!("{signing_input}.{signature}")
    }

    /// A well-formed HS256 token signed with `secret`.
    pub(crate) fn hs256(payload: &str, secret: &str) -> String {
        token_with_header(HS256_HEADER, payload, secret)
    }

    /// The secret this process verifies with.
    pub(crate) fn process_secret() -> String {
        botsecurity_core::verified_jwt::resolve_saas_jwt_secret().expect("saas jwt secret")
    }

    /// A valid token for this process, carrying `sub` and a live `exp`.
    pub(crate) fn valid_token(sub: &str) -> String {
        hs256(&claims(sub, in_an_hour()), &process_secret())
    }

    /// `{"sub":..,"exp":..}` — the minimum a verified token needs.
    pub(crate) fn claims(sub: &str, exp: i64) -> String {
        format!(r#"{{"sub":"{sub}","exp":{exp}}}"#)
    }

    pub(crate) fn in_an_hour() -> i64 {
        chrono::Utc::now().timestamp() + 3600
    }

    pub(crate) fn an_hour_ago() -> i64 {
        chrono::Utc::now().timestamp() - 3600
    }
}

//! Inbound Instagram / Messenger authenticity (#2142).
//!
//! `GET/POST /api/instagram/webhook` is an anonymous route: Meta posts from its
//! own infrastructure, without a user session, so the request itself must prove
//! the delivery is genuine. Two independent proofs are required, matching the
//! two halves of the Meta webhook protocol:
//!
//! | Half | Proof | Check |
//! |---|---|---|
//! | subscription handshake (`GET`) | `hub.verify_token` | constant-time equality with the bot's `instagram-verify-token` |
//! | delivery (`POST`) | `X-Hub-Signature-256` | `sha256=hex(HMAC-SHA256(instagram-app-secret, raw body))` |
//!
//! Both used to be bypassable: the POST had no signature check at all, and the
//! verify token fell back to the hardcoded string `webhook_verify`. The gate now
//! fails closed — an unconfigured credential refuses the request and names the
//! operator action in the log, instead of accepting a stranger's payload.

use axum::http::StatusCode;
use botsecurity_core::constant_time;
use std::sync::atomic::{AtomicBool, Ordering};

/// Header Meta sends on every delivery once the app secret is configured.
pub const SIGNATURE_HEADER: &str = "x-hub-signature-256";

/// Bot config key holding the Meta app secret (Vault `instagram-app-secret`).
/// It is mandatory for deliveries: without it no POST can be proven authentic.
pub const APP_SECRET_KEY: &str = "instagram-app-secret";

/// Bot config key holding the subscription verify token. It is mandatory for
/// the handshake; there is no default, because a published default is a
/// published secret.
pub const VERIFY_TOKEN_KEY: &str = "instagram-verify-token";

/// Outcome of the inbound authenticity gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryAuth {
    /// The signature matches the raw body: the delivery came from Meta.
    Verified,
    /// `X-Hub-Signature-256` is absent or does not match the raw body.
    Mismatch,
    /// No app secret is configured, so no delivery can be proven authentic.
    Unconfigured,
}

impl DeliveryAuth {
    /// HTTP status the handler answers for this outcome.
    pub fn status(self) -> StatusCode {
        match self {
            Self::Verified => StatusCode::OK,
            Self::Mismatch => StatusCode::FORBIDDEN,
            Self::Unconfigured => StatusCode::SERVICE_UNAVAILABLE,
        }
    }
}

/// Decides whether a delivery is authentic.
///
/// The digest covers the **raw** bytes exactly as received: re-serialising a
/// `serde_json` value changes key order and whitespace and no digest would ever
/// match. The hex digest is compared in constant time, and the `sha256=` prefix
/// is mandatory — an uppercase digest is rejected rather than normalized,
/// because Meta always sends lowercase.
pub fn authorize_delivery(app_secret: &str, signature: Option<&str>, body: &[u8]) -> DeliveryAuth {
    let secret = app_secret.trim();
    if secret.is_empty() {
        return DeliveryAuth::Unconfigured;
    }

    let signature = signature.unwrap_or_default().trim();
    let provided = match signature.strip_prefix("sha256=") {
        Some(hex_digest) if !hex_digest.is_empty() => hex_digest,
        _ => return DeliveryAuth::Mismatch,
    };

    let computed = match hmac_sha256_hex(secret.as_bytes(), body) {
        Some(digest) => digest,
        None => return DeliveryAuth::Mismatch,
    };

    if constant_time::eq_str(&computed, provided) {
        DeliveryAuth::Verified
    } else {
        DeliveryAuth::Mismatch
    }
}

/// Why a subscription handshake was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandshakeError {
    /// `instagram-verify-token` is unset: nothing can match.
    Unconfigured,
    /// `hub.mode` is not `subscribe`.
    InvalidMode,
    /// `hub.verify_token` is absent or does not match.
    InvalidToken,
    /// `hub.challenge` is absent: there is nothing to echo back.
    MissingChallenge,
}

impl HandshakeError {
    pub fn status(self) -> StatusCode {
        match self {
            Self::Unconfigured => StatusCode::SERVICE_UNAVAILABLE,
            Self::MissingChallenge => StatusCode::BAD_REQUEST,
            Self::InvalidMode | Self::InvalidToken => StatusCode::FORBIDDEN,
        }
    }

    /// Log-safe description: no token and no secret.
    pub fn reason(self) -> &'static str {
        match self {
            Self::Unconfigured => "no instagram-verify-token configured",
            Self::InvalidMode => "hub.mode is not subscribe",
            Self::InvalidToken => "hub.verify_token missing or mismatched",
            Self::MissingChallenge => "hub.challenge missing",
        }
    }
}

/// Answers the Meta subscription handshake. `Ok(challenge)` is echoed back
/// verbatim (Meta compares it byte for byte).
pub fn authorize_handshake(
    expected_token: &str,
    mode: Option<&str>,
    provided_token: Option<&str>,
    challenge: Option<&str>,
) -> Result<String, HandshakeError> {
    let expected = expected_token.trim();
    if expected.is_empty() {
        return Err(HandshakeError::Unconfigured);
    }

    if mode != Some("subscribe") {
        return Err(HandshakeError::InvalidMode);
    }

    let provided = provided_token.unwrap_or_default().trim();
    if provided.is_empty() || !constant_time::eq_str(expected, provided) {
        return Err(HandshakeError::InvalidToken);
    }

    match challenge {
        Some(challenge) if !challenge.is_empty() => Ok(challenge.to_string()),
        _ => Err(HandshakeError::MissingChallenge),
    }
}

/// `hex(HMAC-SHA256(key, body))`, or `None` when the key is unusable (HMAC
/// accepts any length, so this is only reachable on an internal error).
fn hmac_sha256_hex(key: &[u8], body: &[u8]) -> Option<String> {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    let mut mac = Hmac::<Sha256>::new_from_slice(key).ok()?;
    mac.update(body);
    Some(hex::encode(mac.finalize().into_bytes()))
}

/// Set once, when the first delivery is refused for a missing app secret.
static WARNED_UNCONFIGURED: AtomicBool = AtomicBool::new(false);

/// Emits the configuration error once per process. A forged POST must not be
/// able to flood the log with one line per delivery.
pub fn warn_unconfigured_once() -> bool {
    if WARNED_UNCONFIGURED.swap(true, Ordering::Relaxed) {
        return false;
    }

    log::error!(
        "Instagram webhook delivery refused: no {APP_SECRET_KEY} configured \
         (set it in the bot configuration; Meta signs every POST with the app secret)"
    );
    true
}

#[cfg(test)]
#[path = "auth_tests.rs"]
mod auth_tests;

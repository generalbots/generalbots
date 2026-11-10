//! Signed destinations for email click tracking (#2139).
//!
//! The click endpoint used to redirect to whatever the caller put after the
//! tracking id:
//!
//! ```text
//! GET /api/marketing/track/click/{id}/https://evil.example/login  ->  302
//! ```
//!
//! Because the route is anonymous (a mail client cannot send a bearer token)
//! that made the platform domain an open redirect usable for phishing and for
//! bypassing link scanners that allowlist the domain. A destination is now
//! honoured only when this server signed it into the link:
//!
//! ```text
//! GET /api/marketing/track/click/{id}?u=<url>&e=<unix-expiry>&s=<hmac>
//! ```
//!
//! The HMAC covers the tracking record identity (tracking id, campaign and
//! recipient), the destination and the expiry. That means:
//!
//! * a destination the caller invents is rejected (no valid signature),
//! * a link issued for one recipient cannot be replayed for another,
//! * a tampered destination is rejected,
//! * a link stops working after its expiry.
//!
//! Anything else is a `400` — never a `302`. The set of destinations this
//! server is willing to redirect to is therefore exactly the set it signed,
//! which is the stored allowlist: it lives in the sent email, not in caller
//! input.

use std::sync::OnceLock;

use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::Sha256;
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

/// How long a tracked link stays valid after the email is sent.
pub const CLICK_LINK_TTL_SECS: i64 = 30 * 24 * 60 * 60;

/// Longest destination accepted; longer values are rejected before parsing.
const MAX_DESTINATION_LEN: usize = 2048;

/// Environment variable that overrides the derived signing key. Operations can
/// rotate tracked links by changing it; links signed with the old key stop
/// redirecting and answer `400` instead.
const SIGNING_KEY_ENV: &str = "MARKETING_TRACKING_SECRET";

/// Domain separation label so the click key can never collide with the SaaS
/// JWT secret it is derived from.
const KEY_DERIVATION_LABEL: &[u8] = b"general-bots/marketing-click/v1";

/// Query parameters of a tracked click link.
///
/// Short names keep the link small enough to survive line wrapping in mail
/// clients; the values are opaque to the recipient.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ClickQuery {
    /// Destination URL, signed into `s`.
    pub u: Option<String>,
    /// Expiry as a Unix timestamp in seconds, signed into `s`.
    pub e: Option<String>,
    /// Hex HMAC-SHA256 over the canonical message.
    pub s: Option<String>,
}

/// Why a click link was not honoured. Every variant maps to a `400`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClickError {
    InvalidTrackingId,
    UnknownTrackingId,
    MissingDestination,
    UnsafeDestination,
    MissingExpiry,
    Expired,
    MissingSignature,
    BadSignature,
    StorageUnavailable,
}

impl ClickError {
    /// Short, caller-safe reason. Contains no caller input, so it is safe to
    /// return verbatim in the response body.
    pub fn reason(self) -> &'static str {
        match self {
            Self::InvalidTrackingId => "invalid tracking id",
            Self::UnknownTrackingId => "unknown tracking id",
            Self::MissingDestination => "missing destination",
            Self::UnsafeDestination => "destination not allowed",
            Self::MissingExpiry => "missing or malformed expiry",
            Self::Expired => "link expired",
            Self::MissingSignature => "missing signature",
            Self::BadSignature => "signature does not match",
            Self::StorageUnavailable => "tracking store unavailable",
        }
    }
}

/// The identity a signature is bound to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrackingRecord {
    pub id: Uuid,
    pub campaign_id: Option<Uuid>,
    pub recipient_id: Option<Uuid>,
}

/// What the click endpoint must answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClickOutcome {
    /// Signed, unexpired and safe: the only path that produces a redirect.
    Redirect(String),
    /// Rejected target: answer `400` with [`ClickError::reason`].
    BadRequest(&'static str),
    /// No such tracking record: answer `404`, never a redirect.
    NotFound,
}

/// Process-wide signing key.
///
/// Resolved once: `MARKETING_TRACKING_SECRET` when set (operations), otherwise
/// an HMAC derivation of the persisted SaaS secret. Deriving instead of
/// duplicating keeps one rotation point and one source of truth for the
/// deployment secret, while the label prevents key reuse across features.
pub fn signing_key() -> &'static str {
    static KEY: OnceLock<String> = OnceLock::new();
    KEY.get_or_init(resolve_signing_key).as_str()
}

fn resolve_signing_key() -> String {
    if let Ok(secret) = std::env::var(SIGNING_KEY_ENV) {
        if !secret.trim().is_empty() {
            return secret;
        }
    }
    let base = botsecurity_core::verified_jwt::resolve_saas_jwt_secret();
    match hmac_sha256(base.as_bytes(), KEY_DERIVATION_LABEL) {
        Some(digest) => hex::encode(digest),
        None => String::new(),
    }
}

fn hmac_sha256(key: &[u8], message: &[u8]) -> Option<[u8; 32]> {
    let mut mac = HmacSha256::new_from_slice(key).ok()?;
    mac.update(message);
    let bytes = mac.finalize().into_bytes();
    <[u8; 32]>::try_from(bytes.as_slice()).ok()
}

/// Canonical message covered by the signature.
///
/// Every field that decides where the recipient lands is included, so the
/// signature cannot be transplanted to another record, destination or expiry.
fn canonical_message(record: &TrackingRecord, destination: &str, expires_at: i64) -> String {
    let optional = |value: Option<Uuid>| {
        value
            .map(|id| id.to_string())
            .unwrap_or_else(|| "-".to_string())
    };
    format!(
        "v1|{}|{}|{}|{}|{}",
        record.id,
        optional(record.campaign_id),
        optional(record.recipient_id),
        destination,
        expires_at
    )
}

/// Hex HMAC-SHA256 signature over the canonical message.
pub fn sign(key: &str, record: &TrackingRecord, destination: &str, expires_at: i64) -> String {
    let message = canonical_message(record, destination, expires_at);
    hmac_sha256(key.as_bytes(), message.as_bytes())
        .map(hex::encode)
        .unwrap_or_default()
}

/// Validates a destination and returns it unchanged.
///
/// Two shapes are allowed:
///
/// * a relative path (`/pricing`) — stays on the platform domain;
/// * an absolute `http(s)` URL with a host — a campaign's own landing page.
///
/// Everything else is rejected: schemes such as `javascript:`, `data:` or
/// `file:`, protocol-relative `//host` and `\\host` (browsers treat both as
/// another host), embedded control characters (header injection) and values
/// over [`MAX_DESTINATION_LEN`]. The check runs before the signature check so
/// a signed link can never smuggle one of these shapes through.
pub fn validate_destination(raw: &str) -> Result<String, ClickError> {
    let destination = raw.trim();
    if destination.is_empty() {
        return Err(ClickError::MissingDestination);
    }
    if destination.len() > MAX_DESTINATION_LEN {
        return Err(ClickError::UnsafeDestination);
    }
    if destination
        .chars()
        .any(|c| c.is_control() || c.is_whitespace() || c == '\\')
    {
        return Err(ClickError::UnsafeDestination);
    }
    if destination.starts_with('/') {
        // `//host` and `/\host` are host-relative in every browser.
        if destination.starts_with("//") {
            return Err(ClickError::UnsafeDestination);
        }
        return Ok(destination.to_string());
    }
    let lower = destination.to_ascii_lowercase();
    if !(lower.starts_with("https://") || lower.starts_with("http://")) {
        return Err(ClickError::UnsafeDestination);
    }
    match url::Url::parse(destination) {
        Ok(parsed)
            if matches!(parsed.scheme(), "http" | "https") && parsed.host_str().is_some() =>
        {
            Ok(destination.to_string())
        }
        _ => Err(ClickError::UnsafeDestination),
    }
}

/// Verifies a click link and returns the destination to redirect to.
pub fn verify(
    key: &str,
    record: &TrackingRecord,
    destination: &str,
    expires_at: i64,
    now: i64,
    signature: &str,
) -> Result<String, ClickError> {
    let destination = validate_destination(destination)?;
    if expires_at <= now {
        return Err(ClickError::Expired);
    }
    let expected = sign(key, record, &destination, expires_at);
    if expected.is_empty() || !botsecurity_core::constant_time::eq_str(&expected, signature) {
        return Err(ClickError::BadSignature);
    }
    Ok(destination)
}

/// Builds the tracked link embedded in an outgoing email.
///
/// Returns `None` when the destination is not one this server would accept, so
/// the caller leaves the original href untouched instead of emitting a link
/// that can only answer `400`.
pub fn build_click_url(
    base_url: &str,
    record: &TrackingRecord,
    destination: &str,
    expires_at: i64,
    key: &str,
) -> Option<String> {
    let destination = validate_destination(destination).ok()?;
    let signature = sign(key, record, &destination, expires_at);
    if signature.is_empty() {
        return None;
    }
    let encoded: String =
        url::form_urlencoded::byte_serialize(destination.as_bytes()).collect();
    Some(format!(
        "{}/api/marketing/track/click/{}?u={}&e={}&s={}",
        base_url.trim_end_matches('/'),
        record.id,
        encoded,
        expires_at,
        signature
    ))
}

/// Decides the response for a click request.
///
/// `record` is the row loaded from `email_tracking` (never caller input); a
/// missing row answers [`ClickOutcome::NotFound`] and only a verified link
/// answers [`ClickOutcome::Redirect`].
pub fn evaluate(
    query: &ClickQuery,
    record: Option<&TrackingRecord>,
    now: i64,
    key: &str,
) -> ClickOutcome {
    let Some(record) = record else {
        return ClickOutcome::NotFound;
    };
    let Some(destination) = query.u.as_deref().map(str::trim).filter(|u| !u.is_empty()) else {
        return ClickOutcome::BadRequest(ClickError::MissingDestination.reason());
    };
    let Some(signature) = query.s.as_deref().map(str::trim).filter(|s| !s.is_empty()) else {
        return ClickOutcome::BadRequest(ClickError::MissingSignature.reason());
    };
    let Some(expires_at) = query.e.as_deref().and_then(|e| e.trim().parse::<i64>().ok()) else {
        return ClickOutcome::BadRequest(ClickError::MissingExpiry.reason());
    };
    match verify(key, record, destination, expires_at, now, signature) {
        Ok(destination) => ClickOutcome::Redirect(destination),
        Err(err) => ClickOutcome::BadRequest(err.reason()),
    }
}

#[cfg(test)]
#[path = "click_sign_tests.rs"]
mod tests;

//! Compliance-grade gates for campaign delivery (#2536).
//!
//! These modules close the offline-tractable part of the delivery gap:
//! suppression, a typed consent predicate, a signed one-click unsubscribe and
//! bounce classification with bounded retry. Every send path (worker, publish
//! and the email helper) calls [`evaluate_send_gate`] before contacting a
//! provider; a skip is always recorded as an event row, never silent.

pub mod bounce;
pub mod consent;
pub mod suppression;
pub mod unsubscribe;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

pub use bounce::{
    classify_bounce, retry_delay_millis, AttemptState, BounceClass, BounceWebhookPayload,
    MAX_SEND_ATTEMPTS,
};
pub use consent::{channel_consent, ConsentState};
pub use suppression::{
   MarketingChannel, SuppressionError, SuppressionKey, SuppressionOutcome, SuppressionStore,
};
pub use unsubscribe::{
    build_unsubscribe_url, issue_unsubscribe_token, record_unsubscribe,
    sign_claims_with_key, unsubscribe, unsubscribe_headers, verify_unsubscribe_token,
    UnsubscribeClaims, UnsubscribeError, UnsubscribeResult,
};

use serde_json::Value;
use uuid::Uuid;

/// Terminal or opted-out recipient statuses that must never be sent to.
const BLOCKED_RECIPIENT_STATUSES: [&str; 7] = [
    "unsubscribed",
    "suppressed",
    "opted_out",
    "bounced",
    "complaint",
    "complained",
    "dead_letter",
];

/// Whether a persisted recipient status already bars delivery.
pub fn recipient_status_blocks_send(status: Option<&str>) -> bool {
    status
        .map(|raw| BLOCKED_RECIPIENT_STATUSES.contains(&raw.trim().to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}

/// Why a recipient was not sent to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SendGateDecision {
    Allow,
    Suppressed,
    NoConsent,
}

impl SendGateDecision {
    /// Event `status` value recorded for a skipped recipient.
    pub fn status(self) -> &'static str {
        match self {
            Self::Allow => "allowed",
            Self::Suppressed => "suppressed",
            Self::NoConsent => "consent_skipped",
        }
    }

    pub fn is_allowed(self) -> bool {
        matches!(self, Self::Allow)
    }
}

/// Single gate used by every send path.
///
/// Order matters: a suppression hit is reported as such even when consent is
/// also absent, so an opted-out recipient is never mislabelled as merely
/// missing consent.
pub fn evaluate_send_gate(
    branch_id: Uuid,
    channel: MarketingChannel,
    address: &str,
    recipient_status: Option<&str>,
    subscribed: Option<bool>,
    metadata: Option<&Value>,
) -> SendGateDecision {
    if recipient_status_blocks_send(recipient_status) {
        return SendGateDecision::Suppressed;
    }
    let key = SuppressionKey::new(branch_id, channel, address);
    if SuppressionStore::global().is_suppressed(&key) {
        return SendGateDecision::Suppressed;
    }
    match consent::channel_consent(subscribed, metadata, channel) {
        ConsentState::OptedIn => SendGateDecision::Allow,
        _ => SendGateDecision::NoConsent,
    }
}

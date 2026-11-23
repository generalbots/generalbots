//! Keyed signatures for meeting minutes (issue #2783).
//!
//! Minutes used to be "signed" with an unkeyed `SHA512(canonical_content)`.
//! That is only a checksum: anyone able to submit the request could mint a
//! valid signature for arbitrary content, and the stored record carried a nil
//! signer, so nothing identified who signed. A signature is now an
//! HMAC-SHA256 over the canonical minutes, the authenticated signer and the
//! minute id, keyed with a server-held secret provisioned from Vault via the
//! environment. Without that key the service fails closed: no signature is
//! produced and every verification rejects.

use std::sync::OnceLock;

use hmac::{Hmac, Mac};
use sha2::Sha256;

use botsecurity_core::constant_time;
use chrono::Utc;
use log::info;
use uuid::Uuid;

use crate::minutes::types::{MeetingMinute, MinuteSignature, MinuteStatus};

type HmacSha256 = Hmac<Sha256>;

/// Environment variable holding the server-held signing key. In production it
/// is injected from Vault (never committed). Rotating it invalidates every
/// signature made with the previous value.
const SIGNING_KEY_ENV: &str = "MEET_MINUTES_SIGNING_KEY";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignatureError {
    /// No server-held key is configured: signing is disabled and fails closed.
    KeyUnavailable,
    /// The signer is not an authenticated user (nil id).
    UnauthenticatedSigner,
}

impl std::fmt::Display for SignatureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::KeyUnavailable => write!(f, "meeting-minute signing key unavailable"),
            Self::UnauthenticatedSigner => write!(f, "an authenticated signer is required"),
        }
    }
}

impl std::error::Error for SignatureError {}

fn resolve_signing_key() -> Option<Vec<u8>> {
    match std::env::var(SIGNING_KEY_ENV) {
        Ok(value) if !value.trim().is_empty() => Some(value.into_bytes()),
        Ok(_) => {
            log::error!("{SIGNING_KEY_ENV} is empty; meeting-minute signing is disabled");
            None
        }
        Err(_) => {
            log::error!("{SIGNING_KEY_ENV} is not set; meeting-minute signing is disabled");
            None
        }
    }
}

/// Process-wide signing key, resolved once from the environment (Vault).
///
/// Returns `None` when the key is missing or empty so callers fail closed.
pub fn signing_key() -> Option<&'static [u8]> {
    static KEY: OnceLock<Option<Vec<u8>>> = OnceLock::new();
    KEY.get_or_init(resolve_signing_key).as_deref()
}

#[derive(Debug, Clone)]
pub struct DigitalSigner;

impl Default for DigitalSigner {
    fn default() -> Self {
        Self::new()
    }
}

impl DigitalSigner {
    pub fn new() -> Self {
        Self
    }

    /// Signs as `user_id` with the server key. An unauthenticated (nil) signer
    /// is refused: every signature must identify a real user.
    pub fn sign(&self, minute: &MeetingMinute, user_id: Uuid) -> anyhow::Result<MinuteSignature> {
        Ok(SignatureService::sign_minute(minute, user_id, None, None)?)
    }
}

pub struct SignatureService;

impl SignatureService {
    /// Creates a keyed signature bound to the minutes content, the
    /// authenticated signer and the minute id.
    pub fn sign_minute(
        minute: &MeetingMinute,
        user_id: Uuid,
        signature_id: Option<Uuid>,
        ip_address: Option<String>,
    ) -> Result<MinuteSignature, SignatureError> {
        if user_id.is_nil() {
            return Err(SignatureError::UnauthenticatedSigner);
        }
        let key = signing_key().ok_or(SignatureError::KeyUnavailable)?;
        let signed_hash =
            Self::compute_signature(key, minute, user_id).ok_or(SignatureError::KeyUnavailable)?;

        // Never log signature material itself.
        info!(
            "Minute {} signed by user {} with HMAC-SHA256",
            minute.id, user_id
        );

        Ok(MinuteSignature {
            id: Uuid::new_v4(),
            minute_id: minute.id,
            user_id,
            signature_id,
            signed_hash,
            signed_at: Utc::now(),
            ip_address,
        })
    }

    /// Verifies a signature with a constant-time comparison. Rejects a missing
    /// key, an empty digest, a minute-id mismatch, and (because the signer is
    /// part of the signed message) a signature relocated to another user.
    pub fn verify_signature(signature: &MinuteSignature, minute: &MeetingMinute) -> bool {
        match signing_key() {
            Some(key) => Self::verify_with_key(key, signature, minute),
            None => false,
        }
    }

    fn verify_with_key(key: &[u8], signature: &MinuteSignature, minute: &MeetingMinute) -> bool {
        if key.is_empty() || signature.signed_hash.is_empty() || signature.minute_id != minute.id {
            return false;
        }
        let Some(expected) = Self::compute_signature(key, minute, signature.user_id) else {
            return false;
        };
        constant_time::eq_str(&expected, &signature.signed_hash)
    }

    fn compute_signature(key: &[u8], minute: &MeetingMinute, user_id: Uuid) -> Option<String> {
        let mut mac = HmacSha256::new_from_slice(key).ok()?;
        mac.update(Self::canonical_message(minute, user_id).as_bytes());
        Some(hex::encode(mac.finalize().into_bytes()))
    }

    pub fn is_signed_by_user(signatures: &[MinuteSignature], user_id: Uuid) -> bool {
        signatures.iter().any(|s| s.user_id == user_id)
    }

    pub fn min_signatures_required() -> usize {
        1
    }

    pub fn can_finalize(signatures: &[MinuteSignature], total_attendees: usize) -> bool {
        if total_attendees == 0 {
            return !signatures.is_empty();
        }
        let min_required = (total_attendees as f64 * 0.5).ceil() as usize;
        signatures.len() >= min_required.max(Self::min_signatures_required())
    }

    /// Exact bytes covered by the MAC: the canonical minutes followed by the
    /// signer and minute id as suffixes. Binding all three prevents moving a
    /// signature to different content, a different signer or a different
    /// minute.
    fn canonical_message(minute: &MeetingMinute, user_id: Uuid) -> String {
        format!(
            "{}user_id:{}\nminute_id:{}\n",
            Self::canonical_content(minute),
            user_id,
            minute.id
        )
    }

    fn canonical_content(minute: &MeetingMinute) -> String {
        let mut content = String::new();
        content.push_str(&format!("id:{}\n", minute.id));
        content.push_str(&format!("title:{}\n", minute.title));
        content.push_str(&format!("summary:{}\n", minute.summary));
        for kp in &minute.key_points {
            content.push_str(&format!("key_point:{kp}\n"));
        }
        for ai in &minute.action_items {
            content.push_str(&format!(
                "action:{}|{}|{:?}\n",
                ai.task,
                ai.assignee.as_deref().unwrap_or(""),
                ai.due_date.as_deref().unwrap_or("")
            ));
        }
        for d in &minute.decisions {
            content.push_str(&format!("decision:{d}\n"));
        }
        for a in &minute.attendees {
            content.push_str(&format!(
                "attendee:{}|{}\n",
                a.name,
                a.role.as_deref().unwrap_or("")
            ));
        }
        content.push_str(&format!("version:{}\n", minute.version));
        content
    }

    /// Only cryptographically valid signatures move a minute towards `Signed`.
    /// A forged or tampered stored record can no longer grant that state.
    pub fn update_minute_status(minute: &mut MeetingMinute, signatures: &[MinuteSignature]) {
        let valid: Vec<MinuteSignature> = signatures
            .iter()
            .filter(|signature| Self::verify_signature(signature, minute))
            .cloned()
            .collect();
        if valid.is_empty() {
            minute.status = MinuteStatus::Draft;
        } else if Self::can_finalize(&valid, minute.attendees.len()) {
            minute.status = MinuteStatus::Signed;
        } else {
            minute.status = MinuteStatus::Final;
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;

    const TEST_KEY: &[u8] = b"vault-provisioned-test-key";

    fn minute() -> MeetingMinute {
        MeetingMinute {
            id: Uuid::new_v4(),
            bot_id: Uuid::nil(),
            recording_id: None,
            meeting_id: None,
            title: "Board meeting".to_string(),
            summary: "Discussed Q3 figures".to_string(),
            key_points: vec!["Revenue up".to_string()],
            action_items: vec![],
            decisions: vec![],
            attendees: vec![],
            duration_minutes: Some(45),
            status: MinuteStatus::Final,
            version: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn sign_with(key: &[u8], minute: &MeetingMinute, user_id: Uuid) -> MinuteSignature {
        MinuteSignature {
            id: Uuid::new_v4(),
            minute_id: minute.id,
            user_id,
            signature_id: None,
            signed_hash: SignatureService::compute_signature(key, minute, user_id)
                .expect("test key is valid"),
            signed_at: Utc::now(),
            ip_address: None,
        }
    }

    #[test]
    fn sign_then_verify_with_the_same_key_succeeds() {
        let minute = minute();
        let signature = sign_with(TEST_KEY, &minute, Uuid::new_v4());
        assert!(SignatureService::verify_with_key(
            TEST_KEY, &signature, &minute
        ));
    }

    #[test]
    fn tampered_content_is_rejected() {
        let minute = minute();
        let signature = sign_with(TEST_KEY, &minute, Uuid::new_v4());

        let mut altered = minute.clone();
        altered.summary = "Revenue was actually down".to_string();
        assert!(!SignatureService::verify_with_key(
            TEST_KEY, &signature, &altered
        ));
    }

    #[test]
    fn a_different_signer_is_rejected() {
        let minute = minute();
        let signature = sign_with(TEST_KEY, &minute, Uuid::new_v4());

        let mut forged = signature.clone();
        forged.user_id = Uuid::new_v4();
        assert!(!SignatureService::verify_with_key(
            TEST_KEY, &forged, &minute
        ));
    }

    #[test]
    fn a_forged_digest_and_a_foreign_key_are_rejected() {
        let minute = minute();
        let signature = sign_with(TEST_KEY, &minute, Uuid::new_v4());

        let mut forged = signature.clone();
        forged.signed_hash = "00".repeat(32);
        assert!(!SignatureService::verify_with_key(
            TEST_KEY, &forged, &minute
        ));
        assert!(!SignatureService::verify_with_key(
            b"another-key",
            &signature,
            &minute
        ));
    }

    #[test]
    fn a_signature_for_another_minute_is_rejected() {
        let minute = minute();
        let signature = sign_with(TEST_KEY, &minute, Uuid::new_v4());

        let mut other = minute.clone();
        other.id = Uuid::new_v4();
        assert!(!SignatureService::verify_with_key(
            TEST_KEY, &signature, &other
        ));
    }

    #[test]
    fn nil_signer_cannot_sign() {
        let minute = minute();
        // A nil signer is rejected before any key lookup.
        assert!(matches!(
            SignatureService::sign_minute(&minute, Uuid::nil(), None, None),
            Err(SignatureError::UnauthenticatedSigner)
        ));
    }
}

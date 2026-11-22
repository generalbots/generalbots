//! HMAC-SHA256 webhook signatures with timestamp tolerance and secret rotation.
//!
//! The signed message is `"<unix_timestamp>.<raw_body>"`. This binds the body
//! and the time to the signature, so a captured delivery cannot be replayed
//! outside the tolerance window. Signing is implemented with `sha2`, which the
//! crate already depends on, so no new dependency is needed.

use sha2::{Digest, Sha256};

use super::subscription::WebhookSubscription;

/// Header carrying `t=<unix>,v1=<hex hmac>`.
pub const SIGNATURE_HEADER: &str = "X-GB-Signature";
/// Header carrying the delivery idempotency key.
pub const DELIVERY_HEADER: &str = "X-GB-Delivery";
/// Header carrying the event id.
pub const EVENT_HEADER: &str = "X-GB-Event";
/// Default accepted clock skew for incoming signatures.
pub const DEFAULT_TOLERANCE_SECS: i64 = 300;

const HMAC_BLOCK: usize = 64;
const HMAC_OUTPUT: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signature {
    pub timestamp: i64,
    pub hex: String,
}

impl Signature {
    #[must_use]
    pub fn header_value(&self) -> String {
        format!("t={},v1={}", self.timestamp, self.hex)
    }

    /// Parses a `t=...,v1=...` header. Extra keys are ignored.
    pub fn parse(header: &str) -> Option<Self> {
        let mut timestamp = None;
        let mut hex = None;
        for part in header.split(',') {
            let (key, value) = part.split_once('=')?;
            match key.trim() {
                "t" => timestamp = Some(value.trim().parse::<i64>().ok()?),
                "v1" => hex = Some(value.trim().to_string()),
                _ => {}
            }
        }
        Some(Self {
            timestamp: timestamp?,
            hex: hex?,
        })
    }
}

/// Canonical byte string signed by both the sender and the verifier.
#[must_use]
pub fn canonical_message(timestamp: i64, body: &[u8]) -> Vec<u8> {
    let mut message = timestamp.to_string().into_bytes();
    message.push(b'.');
    message.extend_from_slice(body);
    message
}

/// Standard HMAC-SHA256 (RFC 2104) over a 64-byte block.
#[must_use]
pub fn hmac_sha256(key: &[u8], message: &[u8]) -> [u8; HMAC_OUTPUT] {
    let mut key_block = [0u8; HMAC_BLOCK];
    if key.len() > HMAC_BLOCK {
        let digest = Sha256::digest(key);
        key_block[..HMAC_OUTPUT].copy_from_slice(&digest[..HMAC_OUTPUT]);
    } else {
        key_block[..key.len()].copy_from_slice(key);
    }

    let mut ipad = [0x36u8; HMAC_BLOCK];
    let mut opad = [0x5cu8; HMAC_BLOCK];
    for index in 0..HMAC_BLOCK {
        ipad[index] ^= key_block[index];
        opad[index] ^= key_block[index];
    }

    let mut inner = Sha256::new();
    inner.update(ipad);
    inner.update(message);
    let inner_digest = inner.finalize();

    let mut outer = Sha256::new();
    outer.update(opad);
    outer.update(inner_digest);
    let result = outer.finalize();

    let mut output = [0u8; HMAC_OUTPUT];
    output.copy_from_slice(&result[..HMAC_OUTPUT]);
    output
}

#[must_use]
pub fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn hex_decode(text: &str) -> Option<Vec<u8>> {
    if text.len() % 2 != 0 {
        return None;
    }
    let mut output = Vec::with_capacity(text.len() / 2);
    let bytes = text.as_bytes();
    for pair in bytes.chunks(2) {
        let high = nibble(pair[0])?;
        let low = nibble(pair[1])?;
        output.push((high << 4) | low);
    }
    Some(output)
}

fn nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut difference = 0u8;
    for (a, b) in left.iter().zip(right) {
        difference |= a ^ b;
    }
    difference == 0
}

#[must_use]
pub fn sign(secret: &[u8], timestamp: i64, body: &[u8]) -> Signature {
    let message = canonical_message(timestamp, body);
    Signature {
        timestamp,
        hex: hex_encode(&hmac_sha256(secret, &message)),
    }
}

/// Signs a body with the subscription's active secret. Returns `None` when
/// the subscription has no active secret (a misconfiguration, not a retry).
#[must_use]
pub fn sign_body(subscription: &WebhookSubscription, body: &[u8]) -> Option<Signature> {
    subscription.active_secret().map(|secret| {
        sign(
            secret.value.as_bytes(),
            chrono::Utc::now().timestamp(),
            body,
        )
    })
}

/// Verifies a signature against any secret still registered on the
/// subscription, within `tolerance_secs` of `now`.
#[must_use]
pub fn verify(
    subscription: &WebhookSubscription,
    header: &str,
    body: &[u8],
    now: i64,
    tolerance_secs: i64,
) -> bool {
    let Some(signature) = Signature::parse(header) else {
        return false;
    };
    if (now - signature.timestamp).abs() > tolerance_secs {
        return false;
    }
    let Some(provided) = hex_decode(&signature.hex) else {
        return false;
    };
    if provided.len() != HMAC_OUTPUT {
        return false;
    }
    let message = canonical_message(signature.timestamp, body);
    subscription.secrets.iter().any(|secret| {
        let expected = hmac_sha256(secret.value.as_bytes(), &message);
        constant_time_eq(&provided, &expected)
    })
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::{hex_encode, hmac_sha256, sign, sign_body, verify, Signature, DEFAULT_TOLERANCE_SECS};
    use crate::office::subscription::WebhookSubscription;

    #[test]
    fn matches_rfc4231_test_vector() {
        let digest = hmac_sha256(b"key", b"The quick brown fox jumps over the lazy dog");
        assert_eq!(
            hex_encode(&digest),
            "f7bc83f430538424b13298e6aa6fb143ef4d59a14946175997479dbc2d1a3cd8"
        );
    }

    #[test]
    fn signature_header_round_trips() {
        let signature = sign(b"secret", 1_764_000_000, b"{\"event_id\":\"e1\"}");
        let parsed = Signature::parse(&signature.header_value());
        assert_eq!(parsed, Some(signature));
    }

    #[test]
    fn signs_and_verifies_a_subscription_delivery() {
        let subscription = WebhookSubscription::new("s1", "https://x.test", "k1", "secret-value");
        let body = br#"{"event_id":"evt-1","type":"doc.comment.added"}"#;
        let now = Utc::now().timestamp();
        let signature = sign(b"secret-value", now, body);
        assert!(verify(
            &subscription,
            &signature.header_value(),
            body,
            now,
            DEFAULT_TOLERANCE_SECS
        ));
        assert!(!verify(
            &subscription,
            &signature.header_value(),
            b"tampered body",
            now,
            DEFAULT_TOLERANCE_SECS
        ));
    }

    #[test]
    fn rotation_grace_and_stale_timestamps() {
        let mut subscription = WebhookSubscription::new("s1", "https://x.test", "k1", "old-value");
        let now = Utc::now().timestamp();
        let old_signature = sign(b"old-value", now, b"body");
        subscription.rotate_secret("k2", "new-value", Utc::now());
        let new_signature = sign_body(&subscription, b"body");
        assert!(new_signature.is_some());
        assert!(verify(
            &subscription,
            &old_signature.header_value(),
            b"body",
            now,
            DEFAULT_TOLERANCE_SECS
        ));
        assert!(!verify(
            &subscription,
            &old_signature.header_value(),
            b"body",
            now + 10_000,
            DEFAULT_TOLERANCE_SECS
        ));
        subscription.retire_secret("k1");
        assert!(!verify(
            &subscription,
            &old_signature.header_value(),
            b"body",
            now,
            DEFAULT_TOLERANCE_SECS
        ));
    }

    #[test]
    fn malformed_headers_are_rejected() {
        let subscription = WebhookSubscription::new("s1", "https://x.test", "k1", "secret-value");
        assert!(!verify(&subscription, "garbage", b"body", 0, 300));
        assert!(!verify(
            &subscription,
            "t=not-a-number,v1=abcd",
            b"body",
            0,
            300
        ));
    }
}

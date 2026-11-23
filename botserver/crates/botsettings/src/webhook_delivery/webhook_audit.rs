//! V2 signature audit material for outbound webhook deliveries (#2829).
//!
//! A delivery stores the signature, timestamp and nonce it actually sent so the
//! record can be re-verified. The legacy v1 scheme is never produced: it has no
//! freshness data and would leave captured deliveries replayable.

use super::{create_signature_headers, SIGNATURE_V2};

/// The full v2 material for one delivery, both on the wire and in the audit row.
pub struct DeliverySignature {
    pub stored: String,
    pub signature: String,
    pub timestamp: String,
    pub nonce: String,
}

/// Generates the v2 triple once so the audit row matches the sent headers.
pub fn sign_delivery(payload: &str, secret: &str) -> DeliverySignature {
    let (signature, timestamp, nonce) = create_signature_headers(payload, secret);
    let stored = format_audit_signature(&signature, &timestamp, &nonce);
    DeliverySignature {
        stored,
        signature,
        timestamp,
        nonce,
    }
}

/// Serializes the full v2 triple into the `webhook_deliveries.signature` audit
/// column (`VARCHAR(128)`). The payload lives in its own column, so these three
/// headers are all a receiver needs to re-verify the sent bytes (#2645, #2829).
pub fn format_audit_signature(signature: &str, timestamp: &str, nonce: &str) -> String {
    format!("{signature}|{timestamp}|{nonce}")
}

/// Inverse of [`format_audit_signature`]; rejects legacy `v1=` values.
pub fn parse_audit_signature(stored: &str) -> Option<(&str, &str, &str)> {
    let (signature, rest) = stored.split_once('|')?;
    let (timestamp, nonce) = rest.split_once('|')?;
    let is_v2 = matches!(
        signature.strip_prefix(&format!("{SIGNATURE_V2}=")),
        Some(hex) if !hex.is_empty()
    );
    if is_v2 && !timestamp.is_empty() && !nonce.is_empty() {
        Some((signature, timestamp, nonce))
    } else {
        None
    }
}

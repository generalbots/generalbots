//! Decision signatures and version hashes for the approvals workflow.
//!
//! This mirrors the canonical scheme used by `botevents::office::signing`
//! (`"<unix_timestamp>.<raw_body>"` signed with HMAC-SHA256) using the `sha2`
//! dependency already present in this crate. Reusing the botevents public API
//! directly would require a Cargo.toml dependency change, which is outside
//! this change's file ownership. The wire format is identical, so a verifier
//! built on botevents semantics can check these headers unchanged.

use sha2::{Digest, Sha256};

const HMAC_BLOCK: usize = 64;
const HMAC_OUTPUT: usize = 32;

/// SHA-256 of the document content. Approvals bind to this value; later
/// content edits produce a different hash and visibly invalidate the trail.
#[must_use]
pub fn version_hash(content: &[u8]) -> String {
    hex_encode(&Sha256::digest(content))
}

/// Hex-encodes a digest without panicking or relying on allocation helpers.
#[must_use]
pub fn hex_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

fn hex_decode(text: &str) -> Option<Vec<u8>> {
    if text.len() % 2 != 0 {
        return None;
    }
    let mut out = Vec::with_capacity(text.len() / 2);
    for pair in text.as_bytes().chunks(2) {
        let high = nibble(pair[0])?;
        let low = nibble(pair[1])?;
        out.push((high << 4) | low);
    }
    Some(out)
}

fn nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
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

/// Canonical bytes signed by both the signer and the verifier.
#[must_use]
pub fn canonical_message(timestamp: i64, body: &[u8]) -> Vec<u8> {
    let mut message = timestamp.to_string().into_bytes();
    message.push(b'.');
    message.extend_from_slice(body);
    message
}

/// Produces a `t=<unix>,v1=<hex>` signature header, identical in shape to the
/// botevents webhook signature header.
#[must_use]
pub fn sign_decision(secret: &[u8], timestamp: i64, body: &[u8]) -> String {
    let message = canonical_message(timestamp, body);
    format!(
        "t={timestamp},v1={}",
        hex_encode(&hmac_sha256(secret, &message))
    )
}

fn parse_header(header: &str) -> Option<(i64, Vec<u8>)> {
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
    Some((timestamp?, hex_decode(&hex?)?))
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

/// Verifies a decision header against the signing key. Returns `false` on a
/// malformed header or a body/time mismatch.
#[must_use]
pub fn verify_decision(secret: &[u8], header: &str, body: &[u8]) -> bool {
    let Some((timestamp, provided)) = parse_header(header) else {
        return false;
    };
    if provided.len() != HMAC_OUTPUT {
        return false;
    }
    let expected = hmac_sha256(secret, &canonical_message(timestamp, body));
    constant_time_eq(&provided, &expected)
}

/// Hashes the ordered decision signature headers to form an audit chain.
#[must_use]
pub fn chain_hash(headers: &[String]) -> String {
    let mut joined = Vec::new();
    for header in headers {
        joined.extend_from_slice(header.as_bytes());
        joined.push(b'\n');
    }
    hex_encode(&Sha256::digest(&joined))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_hash_changes_with_content() {
        let a = version_hash(b"contract v7");
        let b = version_hash(b"contract v8");
        assert_ne!(a, b);
        assert_eq!(a.len(), 64);
    }

    #[test]
    fn hmac_matches_botevents_known_vector() {
        // RFC 4231 test case 2, shared with botevents::office::signing tests.
        let digest = hmac_sha256(b"Jefe", b"what do ya want for nothing?");
        assert_eq!(
            hex_encode(&digest),
            "750c783e6ab0b503eaa86e310a5db73818ff56e3a781c81b60f21ba6796ac6fa"
        );
    }

    #[test]
    fn decision_header_round_trips_and_rejects_tampering() {
        let body = b"{\"actor\":\"cfo\",\"decision\":\"approve\"}";
        let header = sign_decision(b"secret", 1_764_000_000, body);
        assert!(verify_decision(b"secret", &header, body));
        assert!(!verify_decision(b"other", &header, body));
        assert!(!verify_decision(b"secret", &header, b"tampered"));
        assert!(!verify_decision(b"secret", "not a header", body));
    }

    #[test]
    fn chain_hash_binds_order() {
        let a = chain_hash(&["one".to_string(), "two".to_string()]);
        let b = chain_hash(&["two".to_string(), "one".to_string()]);
        assert_ne!(a, b);
    }
}

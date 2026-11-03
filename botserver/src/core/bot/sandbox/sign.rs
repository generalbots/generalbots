//! Short-lived signed URLs for sandbox resources.
//!
//! Format: `{exp}.{thread}.{hash}.{sig}` where `sig` is HMAC-SHA256 over
//! `{exp}.{thread}.{hash}` keyed with the platform master key. The session
//! transport token is deliberately not reused: it is a broad capability, while
//! this token authorizes one resource in one conversation until `exp`.

use base64::Engine;
use hmac::{Hmac, Mac};
use sha2::Sha256;

use super::store::resource_hash;

type HmacSha256 = Hmac<Sha256>;

fn signing_key() -> Vec<u8> {
    botsecurity_crypto::encryption::load_master_encryption_key()
}

fn b64(input: &[u8]) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(input)
}

fn unb64(input: &str) -> Option<Vec<u8>> {
    base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(input.as_bytes())
        .ok()
}

fn mac(exp: i64, thread: &str, hash: &str) -> Option<Vec<u8>> {
    let mut mac = HmacSha256::new_from_slice(&signing_key()).ok()?;
    mac.update(format!("{exp}.{thread}.{hash}").as_bytes());
    Some(mac.finalize().into_bytes().to_vec())
}

/// Signs `hash` for one conversation until `exp` (unix seconds).
pub fn sign_resource(thread_id: &str, hash: &str, ttl_secs: i64) -> String {
    let exp = chrono::Utc::now().timestamp() + ttl_secs.clamp(30, 3600);
    let thread = b64(thread_id.as_bytes());
    let signature = mac(exp, &thread, hash)
        .map(|bytes| b64(&bytes))
        .unwrap_or_default();
    format!("{exp}.{thread}.{hash}.{signature}")
}

/// Verifies signature, expiry and that the requested hash is the signed one.
pub fn verify_resource(hash: &str, token: &str) -> Result<String, &'static str> {
    let mut parts = token.split('.');
    let exp = parts.next().ok_or("missing expiry")?;
    let thread = parts.next().ok_or("missing thread")?;
    let signed_hash = parts.next().ok_or("missing hash")?;
    let signature = parts.next().ok_or("missing signature")?;
    if parts.next().is_some() {
        return Err("token has extra fields");
    }
    let exp: i64 = exp.parse().map_err(|_| "invalid expiry")?;
    if exp < chrono::Utc::now().timestamp() {
        return Err("token expired");
    }
    if signed_hash != hash {
        return Err("token does not cover this resource");
    }
    let provided = unb64(signature).ok_or("invalid signature encoding")?;
    let mut mac = HmacSha256::new_from_slice(&signing_key()).map_err(|_| "signing key unavailable")?;
    mac.update(format!("{exp}.{thread}.{hash}").as_bytes());
    mac.verify_slice(&provided)
        .map_err(|_| "invalid signature")?;
    let thread_bytes = unb64(thread).ok_or("invalid thread encoding")?;
    String::from_utf8(thread_bytes).map_err(|_| "invalid thread encoding")
}

/// Hex sha256, the same identifier the store uses.
#[must_use]
pub fn hash_of(html: &str) -> String {
    resource_hash(html.as_bytes())
}

/// The signed path for a published resource.
#[must_use]
pub fn signed_path(thread_id: &str, hash: &str, ttl_secs: i64) -> String {
    let token = sign_resource(thread_id, hash, ttl_secs);
    format!("/sandbox/ui/{hash}?t={token}")
}

#[cfg(test)]
mod tests {
    use super::{sign_resource, verify_resource};

    #[test]
    fn round_trips_and_rejects_wrong_hash() {
        let token = sign_resource("thread-1", "abc123", 60);
        assert_eq!(verify_resource("abc123", &token).expect("valid"), "thread-1");
        assert!(verify_resource("other", &token).is_err());
    }

    #[test]
    fn rejects_tampering() {
        let token = sign_resource("thread-1", "abc123", 60);
        let mut parts: Vec<&str> = token.split('.').collect();
        parts[3] = "AAAA";
        let tampered = parts.join(".");
        assert!(verify_resource("abc123", &tampered).is_err());
    }
}

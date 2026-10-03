//! Token and CSRF handling for the external-drive flows.
//!
//! OAuth access/refresh tokens are stored encrypted with the platform master
//! key (`botsecurity-crypto`), never in plain text and never logged. The OAuth
//! `state` parameter is a signed binding of (branch, provider) so a callback
//! cannot be replayed against another tenant's connection.

use crate::external::types::Provider;
use base64::Engine;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::sync::atomic::{AtomicU64, Ordering};
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

/// Encrypt a token for storage. The returned string is what goes in the
/// `*_enc` column.
pub fn encrypt_token(plaintext: &str) -> Result<String, String> {
    if plaintext.is_empty() {
        return Ok(String::new());
    }
    let key = botsecurity_crypto::encryption::load_master_encryption_key();
    botsecurity_crypto::encryption::encrypt_field(plaintext, &key)
        .map_err(|e| format!("token encryption failed: {e}"))
}

/// Decrypt a stored token. A value that cannot be decrypted yields an empty
/// string rather than an error: the caller treats that as "reconnect", which
/// is the correct outcome for a token encrypted under a rotated key.
pub fn decrypt_or_empty(encrypted: &str) -> String {
    if encrypted.is_empty() {
        return String::new();
    }
    let key = botsecurity_crypto::encryption::load_master_encryption_key();
    botsecurity_crypto::encryption::decrypt_field(encrypted, &key).unwrap_or_default()
}

/// Monotonic counter mixed into the state so two tokens minted in the same
/// second are never identical.
static NONCE: AtomicU64 = AtomicU64::new(0);

/// Mint the OAuth `state` for a connect request.
///
/// Layout: `base64url(payload).base64url(hmac)` where payload is
/// `{branch}:{provider}:{issued_at}:{nonce}`. The signature is what makes the
/// callback safe: without it, any user could complete a flow and have the
/// tokens stored against somebody else's branch.
pub fn state_token(branch_id: Uuid, provider: Provider) -> String {
    let nonce = NONCE.fetch_add(1, Ordering::Relaxed);
    let issued = chrono::Utc::now().timestamp();
    let payload = format!("{branch_id}:{}:{issued}:{nonce}", provider.as_str());
    let encoded = b64(payload.as_bytes());
    // Signing cannot fail (the key is a fixed-size slice), so an empty
    // signature would only ever mean a bug; leaving the payload unsigned
    // would be the dangerous outcome, hence the empty second half.
    let sig = sign(encoded.as_bytes()).map(|s| b64(&s)).unwrap_or_default();
    format!("{encoded}.{sig}")
}

/// Verify a callback's `state` against the branch and provider it must belong
/// to. Expiry is 15 minutes — long enough for a consent screen, short enough
/// that a leaked URL is not replayable later.
pub fn verify_state(token: &str, branch_id: Uuid, provider: Provider, now: chrono::DateTime<chrono::Utc>) -> bool {
    const MAX_AGE_SECS: i64 = 900;
    let Some((encoded, sig)) = token.split_once('.') else {
        return false;
    };
    // An unsigned token must never verify, whatever the payload claims.
    let Ok(expected) = sign(encoded.as_bytes()).map(|s| b64(&s)) else {
        return false;
    };
    if !constant_time_eq(expected.as_bytes(), sig.as_bytes()) {
        return false;
    }
    let Ok(decoded) = b64_decode(encoded) else {
        return false;
    };
    let Ok(text) = String::from_utf8(decoded) else {
        return false;
    };
    let parts: Vec<&str> = text.split(':').collect();
    if parts.len() != 4 {
        return false;
    }
    if parts[0] != branch_id.to_string() || parts[1] != provider.as_str() {
        return false;
    }
    parts[2]
        .parse::<i64>()
        .map(|issued| (now.timestamp() - issued).abs() <= MAX_AGE_SECS)
        .unwrap_or(false)
}

/// Read the branch and provider a state token claims, **without** trusting it.
///
/// The OAuth callback arrives anonymous and has to learn which connection the
/// state belongs to before it can verify anything, so this only decodes. The
/// caller must still pass the result to `verify_state`, which is what
/// authenticates both values.
pub fn peek_state(token: &str) -> Option<(Uuid, Provider)> {
    let (encoded, _) = token.split_once('.')?;
    let text = String::from_utf8(b64_decode(encoded).ok()?).ok()?;
    let parts: Vec<&str> = text.split(':').collect();
    if parts.len() != 4 {
        return None;
    }
    Some((Uuid::parse_str(parts[0]).ok()?, Provider::parse(parts[1])?))
}

/// HMAC-SHA256 over the state payload. An unusable key (no master key loaded)
/// is an error rather than a silent fallback: an unverified state must fail
/// closed, never be accepted under a well-known key.
fn sign(payload: &[u8]) -> Result<Vec<u8>, String> {
    let key = botsecurity_crypto::encryption::load_master_encryption_key();
    let mut mac = HmacSha256::new_from_slice(&key).map_err(|e| format!("state key rejected: {e}"))?;
    mac.update(payload);
    Ok(mac.finalize().into_bytes().to_vec())
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b.iter()).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

fn b64(bytes: &[u8]) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

fn b64_decode(value: &str) -> Result<Vec<u8>, String> {
    base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(value)
        .map_err(|e| format!("state decode: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn now() -> chrono::DateTime<chrono::Utc> {
        chrono::Utc::now()
    }

    #[test]
    fn token_round_trips_through_encryption() {
        let secret = "ya29.super-secret-access-token";
        let enc = encrypt_token(secret).expect("encrypt");
        assert_ne!(enc, secret);
        assert!(!enc.contains("super-secret"));
        assert_eq!(decrypt_or_empty(&enc), secret);
    }

    #[test]
    fn empty_and_corrupt_tokens_degrade_to_empty() {
        assert_eq!(decrypt_or_empty(""), "");
        // Not decryptable: the connection is treated as needing a reconnect
        // rather than crashing the request.
        assert_eq!(decrypt_or_empty("garbage-not-a-ciphertext"), "");
    }

    #[test]
    fn state_token_verifies_for_its_own_branch_and_provider() {
        let branch = Uuid::new_v4();
        let token = state_token(branch, Provider::OneDrive);
        assert!(verify_state(&token, branch, Provider::OneDrive, now()));
    }

    #[test]
    fn state_token_is_rejected_for_another_branch_or_provider() {
        let branch = Uuid::new_v4();
        let token = state_token(branch, Provider::OneDrive);
        assert!(!verify_state(&token, Uuid::new_v4(), Provider::OneDrive, now()));
        assert!(!verify_state(&token, branch, Provider::GoogleDrive, now()));
    }

    #[test]
    fn state_token_expires_and_resists_tampering() {
        let branch = Uuid::new_v4();
        let token = state_token(branch, Provider::GoogleDrive);
        let stale = now() - chrono::Duration::minutes(30);
        assert!(!verify_state(&token, branch, Provider::GoogleDrive, stale));
        let future = now() + chrono::Duration::minutes(30);
        assert!(!verify_state(&token, branch, Provider::GoogleDrive, future));
        // Any edit invalidates the signature.
        let mut tampered = token.clone();
        tampered.push('x');
        assert!(!verify_state(&tampered, branch, Provider::GoogleDrive, now()));
        assert!(!verify_state("no-dot-here", branch, Provider::GoogleDrive, now()));
    }

    #[test]
    fn peek_state_reports_the_claim_without_trusting_it() {
        let branch = Uuid::new_v4();
        let token = state_token(branch, Provider::GoogleDrive);
        assert_eq!(peek_state(&token), Some((branch, Provider::GoogleDrive)));
        // A forged payload is readable but its verification still fails, which
        // is why the callback must never use `peek_state` alone.
        assert!(peek_state("bm90LWEtdG9rZW4").is_none());
        assert!(peek_state("garbage").is_none());
    }

    #[test]
    fn state_tokens_are_unique_within_the_same_instant() {
        let branch = Uuid::new_v4();
        let a = state_token(branch, Provider::OneDrive);
        let b = state_token(branch, Provider::OneDrive);
        assert_ne!(a, b);
    }
}
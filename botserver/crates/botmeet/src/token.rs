//! Signed, expiring meeting tokens (#2736).
//!
//! A meeting token is a compact, authenticated statement of the form
//! `base64url(json claims).base64url(HMAC-SHA256(claims, server secret))`.
//! It is minted only from verified caller identity (see `handle_mute_all`)
//! and carries its own expiry, so a client can never forge a room/user pair
//! or reuse a token after it has expired.

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use chrono::Utc;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::sync::OnceLock;

type HmacSha256 = Hmac<Sha256>;

/// Current claim format. Bump when the claim layout changes incompatibly.
const CLAIM_VERSION: u8 = 1;
/// Meeting tokens are short lived; a join is a one-shot, near-term action.
pub const MEETING_TOKEN_TTL_SECS: i64 = 900;

/// Environment override for the signing secret (injected from Vault).
const TOKEN_SECRET_ENV: &str = "MEET_TOKEN_SECRET";
/// Domain separator so the shared encryption key is never used bare here.
const TOKEN_KEY_DOMAIN: &[u8] = b"generalbots:botmeet:token:v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeetingClaims {
    pub v: u8,
    pub room: String,
    pub user: String,
    pub exp: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssuedToken {
    pub token: String,
    pub claims: MeetingClaims,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenError {
    Malformed,
    BadSignature,
    Expired,
    UnsupportedVersion,
    IdentityMismatch,
    Encoding(String),
}

impl std::fmt::Display for TokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::Malformed => "malformed meeting token",
            Self::BadSignature => "invalid meeting token signature",
            Self::Expired => "meeting token expired",
            Self::UnsupportedVersion => "unsupported meeting token version",
            Self::IdentityMismatch => "meeting token does not match caller",
            Self::Encoding(detail) => detail,
        };
        f.write_str(message)
    }
}

impl std::error::Error for TokenError {}

/// Server-held signing key. `MEET_TOKEN_SECRET` (Vault) wins; otherwise the
/// Vault-backed master encryption key is domain-separated and reused. Both are
/// process-stable, so tokens survive a restart within their TTL.
pub fn signing_secret() -> &'static [u8] {
    static KEY: OnceLock<Vec<u8>> = OnceLock::new();
    KEY.get_or_init(|| {
        if let Ok(value) = std::env::var(TOKEN_SECRET_ENV) {
            if !value.trim().is_empty() {
                return value.into_bytes();
            }
        }
        let master = botsecurity::botsecurity_crypto::encryption::load_master_encryption_key();
        let mut material = Vec::with_capacity(TOKEN_KEY_DOMAIN.len() + master.len());
        material.extend_from_slice(TOKEN_KEY_DOMAIN);
        material.extend_from_slice(&master);
        material
    })
}

fn mac_for(secret: &[u8]) -> Result<HmacSha256, TokenError> {
    HmacSha256::new_from_slice(secret)
        .map_err(|e| TokenError::Encoding(format!("signing key rejected: {e}")))
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter()
        .zip(b.iter())
        .fold(0u8, |acc, (x, y)| acc | (x ^ y))
        == 0
}

fn sign_claims(claims: &MeetingClaims, secret: &[u8]) -> Result<String, TokenError> {
    let payload = serde_json::to_vec(claims)
        .map_err(|e| TokenError::Encoding(format!("claim encode failed: {e}")))?;
    let encoded = URL_SAFE_NO_PAD.encode(payload);
    let mut mac = mac_for(secret)?;
    mac.update(encoded.as_bytes());
    let signature = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());
    Ok(format!("{encoded}.{signature}"))
}

/// Issue a token for `room`/`user` with an explicit clock (used by tests).
pub fn issue_at(
    room: &str,
    user: &str,
    ttl_secs: i64,
    now: i64,
    secret: &[u8],
) -> Result<String, TokenError> {
    let claims = MeetingClaims {
        v: CLAIM_VERSION,
        room: room.to_string(),
        user: user.to_string(),
        exp: now.saturating_add(ttl_secs),
    };
    sign_claims(&claims, secret)
}

/// Issue a token with the default TTL using the current clock and server key,
/// returning both the wire token and its verified-shape claims for the caller.
pub fn issue_meeting_token(room: &str, user: &str) -> Result<IssuedToken, TokenError> {
    let claims = MeetingClaims {
        v: CLAIM_VERSION,
        room: room.to_string(),
        user: user.to_string(),
        exp: Utc::now()
            .timestamp()
            .saturating_add(MEETING_TOKEN_TTL_SECS),
    };
    let token = sign_claims(&claims, signing_secret())?;
    Ok(IssuedToken { token, claims })
}

/// Verify signature, version and expiry, returning the verified claims.
pub fn verify_at(token: &str, secret: &[u8], now: i64) -> Result<MeetingClaims, TokenError> {
    let (encoded, signature) = token.split_once('.').ok_or(TokenError::Malformed)?;
    if encoded.is_empty() || signature.is_empty() {
        return Err(TokenError::Malformed);
    }
    let supplied = URL_SAFE_NO_PAD
        .decode(signature.as_bytes())
        .map_err(|e| TokenError::Encoding(format!("signature decode failed: {e}")))?;
    let mut mac = mac_for(secret)?;
    mac.update(encoded.as_bytes());
    if !constant_time_eq(&supplied, &mac.finalize().into_bytes()) {
        return Err(TokenError::BadSignature);
    }
    let payload = URL_SAFE_NO_PAD
        .decode(encoded.as_bytes())
        .map_err(|e| TokenError::Encoding(format!("payload decode failed: {e}")))?;
    let claims: MeetingClaims = serde_json::from_slice(&payload)
        .map_err(|e| TokenError::Encoding(format!("claim decode failed: {e}")))?;
    if claims.v != CLAIM_VERSION {
        return Err(TokenError::UnsupportedVersion);
    }
    if now >= claims.exp {
        return Err(TokenError::Expired);
    }
    Ok(claims)
}

/// Verify a token and bind it to the room and user presenting it. Use this at
/// every join/use site: a valid token for room A must not open room B.
pub fn verify_for(
    token: &str,
    secret: &[u8],
    expected_room: &str,
    expected_user: &str,
    now: i64,
) -> Result<MeetingClaims, TokenError> {
    let claims = verify_at(token, secret, now)?;
    if claims.room != expected_room || claims.user != expected_user {
        return Err(TokenError::IdentityMismatch);
    }
    Ok(claims)
}

/// Convenience wrapper using the current clock and server key.
pub fn verify_meeting_token(
    token: &str,
    expected_room: &str,
    expected_user: &str,
) -> Result<MeetingClaims, TokenError> {
    verify_for(
        token,
        signing_secret(),
        expected_room,
        expected_user,
        Utc::now().timestamp(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &[u8] = b"test-only-signing-secret";

    #[test]
    fn signs_and_verifies_roundtrip() {
        let token = issue_at("room-a", "user-1", 900, 1000, SECRET).expect("issue");
        let claims = verify_at(&token, SECRET, 1001).expect("verify");
        assert_eq!(claims.room, "room-a");
        assert_eq!(claims.user, "user-1");
        assert_eq!(claims.exp, 1900);
    }

    #[test]
    fn rejects_tampered_signature() {
        let token = issue_at("room-a", "user-1", 900, 1000, SECRET).expect("issue");
        let (payload, signature) = token.split_once('.').expect("split");
        let mut sig = URL_SAFE_NO_PAD.decode(signature).expect("decode sig");
        sig[0] ^= 0x01;
        let forged = format!("{payload}.{}", URL_SAFE_NO_PAD.encode(sig));
        assert_eq!(
            verify_at(&forged, SECRET, 1001),
            Err(TokenError::BadSignature)
        );
    }

    #[test]
    fn rejects_tampered_payload() {
        let token = issue_at("room-a", "user-1", 900, 1000, SECRET).expect("issue");
        let (payload, signature) = token.split_once('.').expect("split");
        let mut json = URL_SAFE_NO_PAD.decode(payload).expect("decode payload");
        let source = String::from_utf8(json.clone()).expect("utf8 payload");
        let forged_json = source.replacen("room-a", "room-b", 1);
        json.clear();
        json.extend_from_slice(forged_json.as_bytes());
        let forged = format!("{}.{signature}", URL_SAFE_NO_PAD.encode(json));
        assert_eq!(
            verify_at(&forged, SECRET, 1001),
            Err(TokenError::BadSignature)
        );
    }

    #[test]
    fn rejects_wrong_secret_and_malformed() {
        let token = issue_at("room-a", "user-1", 900, 1000, SECRET).expect("issue");
        assert_eq!(
            verify_at(&token, b"other-secret", 1001),
            Err(TokenError::BadSignature)
        );
        assert_eq!(
            verify_at("not-a-token", SECRET, 1001),
            Err(TokenError::Malformed)
        );
    }

    #[test]
    fn rejects_expired_token() {
        let token = issue_at("room-a", "user-1", 900, 1000, SECRET).expect("issue");
        assert_eq!(verify_at(&token, SECRET, 1900), Err(TokenError::Expired));
        assert!(verify_at(&token, SECRET, 1899).is_ok());
    }

    #[test]
    fn rejects_cross_room_and_impersonated_user() {
        let token = issue_at("room-a", "user-1", 900, 1000, SECRET).expect("issue");
        assert!(verify_for(&token, SECRET, "room-a", "user-1", 1001).is_ok());
        assert_eq!(
            verify_for(&token, SECRET, "room-b", "user-1", 1001),
            Err(TokenError::IdentityMismatch)
        );
        assert_eq!(
            verify_for(&token, SECRET, "room-a", "user-2", 1001),
            Err(TokenError::IdentityMismatch)
        );
    }
}

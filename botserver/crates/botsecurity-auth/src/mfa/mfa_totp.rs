//! Canonical TOTP secret handling and code generation.
//!
//! # Wire format (single source of truth)
//!
//! The TOTP shared secret is stored **and** emitted in the QR code as
//! **unpadded, uppercase RFC 4648 base32** — the exact string an authenticator
//! app reads from `otpauth://totp/...?secret=...`. Both sides therefore derive
//! the same HMAC key:
//!
//! ```text
//! stored secret  = base32(raw key bytes)      // 20 bytes -> 32 chars
//! QR secret      = same string
//! HMAC key       = base32_decode(secret)      // == raw key bytes
//! ```
//!
//! # Legacy rows
//!
//! Enrollments written before #2114 stored `hex::encode(raw key)` (40 chars,
//! all `[0-9a-f]`). [`TotpSecret::parse_stored`] recognizes that exact shape and
//! converts it to base32, so an old row keeps working and is rewritten in the
//! canonical encoding on its next verification. No secret is ever logged.

use anyhow::{anyhow, Result};
use hmac::{Hmac, Mac};
use rand::Rng;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sha1::Sha1;
use sha2::digest::KeyInit;
use sha2::{Sha256, Sha512};

use crate::mfa::mfa_types::{TotpAlgorithm, TOTP_DIGITS, TOTP_SECRET_LENGTH};

const BASE32_ALPHABET: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

/// A validated TOTP shared secret (raw key bytes).
///
/// The type deliberately does not expose a plain `String`: every conversion
/// goes through [`TotpSecret::to_base32`] / [`TotpSecret::from_base32`], so the
/// QR path and the verification path cannot drift apart again.
#[derive(Clone, PartialEq, Eq)]
pub struct TotpSecret(Vec<u8>);

impl TotpSecret {
    /// Build a secret from raw key bytes. Empty keys are rejected because HMAC
    /// accepts them silently while authenticators refuse the URI.
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self> {
        if bytes.is_empty() {
            return Err(anyhow!("TOTP secret must not be empty"));
        }
        Ok(Self(bytes))
    }

    /// Generate a fresh secret with [`TOTP_SECRET_LENGTH`] random bytes.
    pub fn generate() -> Self {
        let mut rng = rand::rng();
        let bytes: Vec<u8> = (0..TOTP_SECRET_LENGTH).map(|_| rng.random()).collect();
        Self(bytes)
    }

    /// Raw key bytes used as the HMAC key.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Canonical wire format: unpadded, uppercase base32.
    pub fn to_base32(&self) -> String {
        base32_encode_bytes(&self.0)
    }

    /// Parse the canonical wire format (padding and whitespace tolerated).
    pub fn from_base32(encoded: &str) -> Result<Self> {
        Self::from_bytes(base32_decode(encoded)?)
    }

    /// True when `encoded` has the shape produced by the pre-#2114 code path:
    /// `hex::encode` of a [`TOTP_SECRET_LENGTH`] byte key.
    pub fn is_legacy_hex(encoded: &str) -> bool {
        let trimmed = encoded.trim();
        trimmed.len() == TOTP_SECRET_LENGTH * 2
            && trimmed.bytes().all(|b| b.is_ascii_hexdigit())
    }

    /// Parse a stored secret, accepting the canonical base32 form first and
    /// converting a legacy hex row when it matches [`Self::is_legacy_hex`].
    pub fn parse_stored(encoded: &str) -> Result<Self> {
        if Self::is_legacy_hex(encoded) {
            let bytes = hex::decode(encoded.trim())
                .map_err(|_| anyhow!("Stored TOTP secret is not valid hex"))?;
            return Self::from_bytes(bytes);
        }
        Self::from_base32(encoded)
    }
}

impl std::fmt::Debug for TotpSecret {
    /// Redacted: a secret must never reach a log line or an error message.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("TotpSecret(<redacted>)")
    }
}

impl Serialize for TotpSecret {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_base32())
    }
}

impl<'de> Deserialize<'de> for TotpSecret {
    fn deserialize<D: Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        let encoded = String::deserialize(deserializer)?;
        Self::parse_stored(&encoded).map_err(serde::de::Error::custom)
    }
}

/// RFC 4648 base32 without padding (the form authenticators expect).
pub fn base32_encode_bytes(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(5) * 8);
    let mut buffer: u32 = 0;
    let mut bits: u32 = 0;

    for &byte in bytes {
        buffer = (buffer << 8) | u32::from(byte);
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            out.push(BASE32_ALPHABET[((buffer >> bits) & 0x1f) as usize] as char);
        }
    }

    if bits > 0 {
        out.push(BASE32_ALPHABET[((buffer << (5 - bits)) & 0x1f) as usize] as char);
    }

    out
}

/// RFC 4648 base32 decode. Case-insensitive; ignores `=`, `-` and whitespace.
pub fn base32_decode(encoded: &str) -> Result<Vec<u8>> {
    let mut out = Vec::with_capacity(encoded.len() * 5 / 8);
    let mut buffer: u32 = 0;
    let mut bits: u32 = 0;
    let mut symbols = 0usize;

    for ch in encoded.chars() {
        if ch.is_ascii_whitespace() || ch == '=' || ch == '-' {
            continue;
        }
        let upper = ch.to_ascii_uppercase();
        let value = BASE32_ALPHABET
            .iter()
            .position(|&c| char::from(c) == upper)
            .ok_or_else(|| anyhow!("TOTP secret is not valid base32"))?;

        buffer = (buffer << 5) | value as u32;
        bits += 5;
        symbols += 1;
        if bits >= 8 {
            bits -= 8;
            out.push(((buffer >> bits) & 0xff) as u8);
        }
    }

    if symbols == 0 {
        // RFC 4648 vector: the empty string decodes to zero bytes. An empty
        // *secret* is still rejected by `TotpSecret::from_bytes`.
        return Ok(Vec::new());
    }
    Ok(out)
}

/// RFC 4226 HOTP: HMAC over the big-endian counter, dynamic truncation, then
/// modulo `10^digits`.
pub fn generate_hotp(
    secret: &TotpSecret,
    counter: u64,
    digits: u32,
    algorithm: TotpAlgorithm,
) -> Result<String> {
    if !(1..=10).contains(&digits) {
        return Err(anyhow!("TOTP digits must be between 1 and 10"));
    }

    let digest = match algorithm {
        TotpAlgorithm::Sha1 => hmac_sha1(secret.as_bytes(), counter)?,
        TotpAlgorithm::Sha256 => hmac_sha256(secret.as_bytes(), counter)?,
        TotpAlgorithm::Sha512 => hmac_sha512(secret.as_bytes(), counter)?,
    };

    let offset = usize::from(digest[digest.len() - 1] & 0x0f);
    let slice = digest
        .get(offset..offset + 4)
        .ok_or_else(|| anyhow!("TOTP HMAC digest is too short"))?;

    let binary = u32::from_be_bytes([slice[0] & 0x7f, slice[1], slice[2], slice[3]]) as u64;
    let modulus = 10u64
        .checked_pow(digits)
        .ok_or_else(|| anyhow!("TOTP digit count is out of range"))?;

    Ok(format!(
        "{:0width$}",
        binary % modulus,
        width = digits as usize
    ))
}

/// RFC 6238 code for an explicit Unix timestamp (deterministic, testable).
pub fn generate_totp_code_at(
    secret: &TotpSecret,
    unix_seconds: u64,
    period: u64,
    digits: u32,
    algorithm: TotpAlgorithm,
) -> Result<String> {
    if period == 0 {
        return Err(anyhow!("TOTP period must be greater than zero"));
    }
    generate_hotp(secret, unix_seconds / period, digits, algorithm)
}

/// RFC 6238 code for the current 30 s period (SHA-1, 6 digits).
pub fn generate_totp_code(secret: &TotpSecret, counter: u64) -> Result<String> {
    generate_hotp(secret, counter, TOTP_DIGITS, TotpAlgorithm::Sha1)
}

/// Verify `code` at an explicit timestamp with a ±`window` period tolerance.
pub fn verify_totp_at(
    secret: &TotpSecret,
    code: &str,
    unix_seconds: u64,
    period: u64,
    digits: u32,
    algorithm: TotpAlgorithm,
    window: i64,
) -> bool {
    if period == 0 {
        return false;
    }

    let candidate = code.trim();
    if candidate.is_empty() {
        return false;
    }

    let counter = (unix_seconds / period) as i64;
    let mut valid = false;
    for offset in -window..=window {
        let Some(check_counter) = counter.checked_add(offset) else {
            continue;
        };
        if check_counter < 0 {
            continue;
        }
        if let Ok(expected) =
            generate_hotp(secret, check_counter as u64, digits, algorithm)
        {
            // Bitwise accumulation keeps the comparison timing-independent.
            valid |= super::mfa_helpers::constant_time_compare(&expected, candidate);
        }
    }
    valid
}

/// Verify a code against the current time, ±1 period (RFC 6238 §5.2).
pub fn verify_totp_full(
    secret: &TotpSecret,
    code: &str,
    period: u64,
    digits: u32,
    algorithm: TotpAlgorithm,
) -> bool {
    verify_totp_at(secret, code, unix_now(), period, digits, algorithm, 1)
}

/// Verify with the default SHA-1/6-digit profile (compatibility helper).
pub fn verify_totp(secret: &TotpSecret, code: &str, period: u64) -> bool {
    verify_totp_full(secret, code, period, TOTP_DIGITS, TotpAlgorithm::Sha1)
}

/// Current Unix time; a clock before the epoch is treated as zero.
pub fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Fresh canonical secret, in the same wire format the QR code carries.
pub fn generate_totp_secret() -> TotpSecret {
    TotpSecret::generate()
}

fn hmac_sha1(key: &[u8], counter: u64) -> Result<Vec<u8>> {
    let mut mac = <Hmac<Sha1> as KeyInit>::new_from_slice(key)
        .map_err(|_| anyhow!("Invalid TOTP HMAC key"))?;
    mac.update(&counter.to_be_bytes());
    Ok(mac.finalize().into_bytes().to_vec())
}

fn hmac_sha256(key: &[u8], counter: u64) -> Result<Vec<u8>> {
    let mut mac = <Hmac<Sha256> as KeyInit>::new_from_slice(key)
        .map_err(|_| anyhow!("Invalid TOTP HMAC key"))?;
    mac.update(&counter.to_be_bytes());
    Ok(mac.finalize().into_bytes().to_vec())
}

fn hmac_sha512(key: &[u8], counter: u64) -> Result<Vec<u8>> {
    let mut mac = <Hmac<Sha512> as KeyInit>::new_from_slice(key)
        .map_err(|_| anyhow!("Invalid TOTP HMAC key"))?;
    mac.update(&counter.to_be_bytes());
    Ok(mac.finalize().into_bytes().to_vec())
}

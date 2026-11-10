//! Non-TOTP MFA helpers: challenges, recovery codes, e-mail/SMS OTP and
//! constant-time comparison.
//!
//! TOTP lives in [`super::mfa_totp`] — one canonical secret type shared by the
//! QR code and the verifier (#2114).

use rand::Rng;
use sha2::{Digest, Sha256};

use crate::mfa::mfa_types::MfaMethod;
use crate::mfa::mfa_types::RECOVERY_CODE_LENGTH;

pub fn generate_challenge() -> Vec<u8> {
    let mut rng = rand::rng();
    let challenge: Vec<u8> = (0..32).map(|_| rng.random()).collect();
    challenge
}

pub fn generate_recovery_code() -> String {
    const CHARS: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let mut rng = rand::rng();

    let code: String = (0..RECOVERY_CODE_LENGTH)
        .map(|_| CHARS[rng.random_range(0..CHARS.len())] as char)
        .collect();

    format!("{}-{}", &code[..4], &code[4..])
}

pub fn generate_otp_code() -> String {
    let mut rng = rand::rng();
    format!("{:06}", rng.random_range(0..1_000_000u32))
}

pub fn hash_recovery_code(code: &str) -> String {
    let normalized = code.replace('-', "").to_uppercase();
    let hash = Sha256::digest(normalized.as_bytes());
    hex::encode(hash)
}

pub fn verify_recovery_code(code: &str, hash: &str) -> bool {
    let code_hash = hash_recovery_code(code);
    constant_time_compare(&code_hash, hash)
}

pub fn hash_otp_code(code: &str) -> String {
    let hash = Sha256::digest(code.as_bytes());
    hex::encode(hash)
}

pub fn verify_otp_code(code: &str, hash: &str) -> bool {
    let code_hash = hash_otp_code(code);
    constant_time_compare(&code_hash, hash)
}

pub fn constant_time_compare(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (x, y) in a.bytes().zip(b.bytes()) {
        result |= x ^ y;
    }
    result == 0
}

pub fn mask_destination(destination: &str, method: MfaMethod) -> String {
    match method {
        MfaMethod::EmailOtp => {
            if let Some(at_pos) = destination.find('@') {
                let local = &destination[..at_pos];
                let domain = &destination[at_pos..];
                if local.len() <= 2 {
                    format!("{}***{}", &local[..1], domain)
                } else {
                    format!("{}***{}{}", &local[..2], &local[local.len() - 1..], domain)
                }
            } else {
                "***@***".into()
            }
        }
        MfaMethod::SmsOtp => {
            let digits: String = destination
                .chars()
                .filter(|c| c.is_ascii_digit())
                .collect();
            if digits.len() <= 4 {
                "***".into()
            } else {
                format!("***{}", &digits[digits.len() - 4..])
            }
        }
        _ => "***".into(),
    }
}

//! MFA (Multi-Factor Authentication) module
//!
//! Provides TOTP (Time-based One-Time Password) and WebAuthn support for
//! adding an extra layer of security to user accounts.
//!
//! This module includes:
//! - TOTP enrollment, verification, and management
//! - WebAuthn (FIDO2) authentication support
//! - Recovery code generation and verification
//! - User MFA state management

pub mod mfa_types;
pub mod mfa_manager;
pub mod mfa_helpers;

pub use mfa_types::MfaMethod;
pub use mfa_types::MfaStatus;
pub use mfa_types::MfaConfig;
pub use mfa_types::TotpAlgorithm;
pub use mfa_types::TotpEnrollment;
pub use mfa_types::WebAuthnCredential;
pub use mfa_types::WebAuthnChallenge;
pub use mfa_types::RecoveryCode;
pub use mfa_types::UserMfaState;
pub use mfa_types::OtpChallenge;

pub use mfa_manager::MfaManager;

pub use mfa_helpers::generate_totp_secret;
pub use mfa_helpers::generate_challenge;
pub use mfa_helpers::generate_recovery_code;
pub use mfa_helpers::generate_otp_code;
pub use mfa_helpers::hash_recovery_code;
pub use mfa_helpers::verify_recovery_code;
pub use mfa_helpers::hash_otp_code;
pub use mfa_helpers::verify_otp_code;
pub use mfa_helpers::constant_time_compare;
pub use mfa_helpers::base32_encode;
pub use mfa_helpers::verify_totp;
pub use mfa_helpers::generate_totp_code;
pub use mfa_helpers::mask_destination;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_totp_enrollment_creation() {
        let config = MfaConfig::default();
        let user_id = uuid::Uuid::new_v4();
        let enrollment = TotpEnrollment::new(user_id, "test@example.com", &config);

        assert_eq!(enrollment.user_id, user_id);
        assert!(!enrollment.secret.is_empty());
        assert!(!enrollment.verified);
    }

    #[test]
    fn test_totp_uri_generation() {
        let config = MfaConfig::default();
        let user_id = uuid::Uuid::new_v4();
        let enrollment = TotpEnrollment::new(user_id, "test@example.com", &config);

        let uri = enrollment.to_uri();
        assert!(uri.starts_with("otpauth://totp/"));
        assert!(uri.contains("secret="));
        assert!(uri.contains("issuer="));
    }

    #[test]
    fn test_recovery_code_generation() {
        let (codes, recovery_codes) = RecoveryCode::generate_set(10);

        assert_eq!(codes.len(), 10);
        assert_eq!(recovery_codes.len(), 10);

        for code in &codes {
            assert!(code.contains('-'));
            assert_eq!(code.len(), 9);
        }
    }

    #[test]
    fn test_recovery_code_verification() {
        let (codes, mut recovery_codes) = RecoveryCode::generate_set(1);

        assert!(recovery_codes[0].verify(&codes[0]));
        assert!(!recovery_codes[0].verify("WRONG-CODE"));

        recovery_codes[0].mark_used();
        assert!(!recovery_codes[0].verify(&codes[0]));
    }

    #[test]
    fn test_user_mfa_state() {
        let user_id = uuid::Uuid::new_v4();
        let state = UserMfaState::new(user_id);

        assert_eq!(state.user_id, user_id);
        assert_eq!(state.status, MfaStatus::NotEnrolled);
        assert!(!state.is_enrolled());
        assert!(!state.is_locked());
    }

    #[test]
    fn test_otp_challenge_creation() {
        let user_id = uuid::Uuid::new_v4();
        let (code, challenge) = OtpChallenge::new(user_id, MfaMethod::EmailOtp, "test@example.com", 300);

        assert_eq!(code.len(), 6);
        assert_eq!(challenge.user_id, user_id);
        assert!(!challenge.is_expired());
    }

    #[test]
    fn test_webauthn_challenge() {
        let user_id = uuid::Uuid::new_v4();
        let challenge = WebAuthnChallenge::new_registration(user_id, 300);

        assert_eq!(challenge.user_id, user_id);
        assert!(challenge.is_registration);
        assert!(!challenge.is_expired());
        assert!(!challenge.challenge_base64().is_empty());
    }

    #[test]
    fn test_mfa_method_names() {
        assert_eq!(MfaMethod::Totp.as_str(), "totp");
        assert_eq!(MfaMethod::WebAuthn.as_str(), "webauthn");
        assert_eq!(MfaMethod::Totp.display_name(), "Authenticator App");
        assert_eq!(MfaMethod::WebAuthn.display_name(), "Security Key");
    }

    #[test]
    fn test_mask_email() {
        let masked = mask_destination("test@example.com", MfaMethod::EmailOtp);
        assert!(masked.contains("***"));
        assert!(masked.contains("@example.com"));
    }

    #[test]
    fn test_mask_phone() {
        let masked = mask_destination("+1234567890", MfaMethod::SmsOtp);
        assert!(masked.contains("***"));
        assert!(masked.ends_with("7890"));
    }

    #[tokio::test]
    async fn test_mfa_manager_creation() {
        let config = MfaConfig::default();
        let manager = MfaManager::new(config);

        let user_id = uuid::Uuid::new_v4();
        let state = manager.get_user_state(user_id).await;

        assert_eq!(state.user_id, user_id);
        assert!(!state.is_enrolled());
    }

    #[tokio::test]
    async fn test_totp_enrollment_flow() {
        let config = MfaConfig::default();
        let manager = MfaManager::new(config);
        let user_id = uuid::Uuid::new_v4();

        let enrollment = manager
            .start_totp_enrollment(user_id, "test@example.com")
            .await
            .expect("Enrollment failed");

        assert_eq!(enrollment.user_id, user_id);
        assert!(!enrollment.verified);

        let state = manager.get_user_state(user_id).await;
        assert_eq!(state.status, MfaStatus::Pending);
    }

    #[test]
    fn test_constant_time_compare() {
        assert!(constant_time_compare("abc123", "abc123"));
        assert!(!constant_time_compare("abc123", "abc124"));
        assert!(!constant_time_compare("abc", "abcd"));
    }

    #[test]
    fn test_base32_encode() {
        let encoded = base32_encode("test");
        assert!(!encoded.is_empty());
        assert!(encoded.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()));
    }

    /// #2066 — the QR used to encode base32 of the *hex text* while the
    /// verifier hex-decoded it, so enrollment could never succeed. RFC 4648's
    /// base32 of the RFC 6238 key pins the bytes an authenticator will use.
    #[test]
    fn the_qr_secret_is_base32_of_the_raw_key() {
        // "12345678901234567890" (the RFC 6238 key) as ASCII bytes.
        let raw = b"12345678901234567890";
        assert_eq!(
            super::mfa_helpers::base32_encode_bytes(raw),
            "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ"
        );

        // A hex-encoded secret of the same bytes must yield the same base32.
        let hex_secret = hex::encode(raw);
        assert_eq!(
            super::mfa_helpers::base32_encode_totp_secret(&hex_secret),
            "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ"
        );

        // And it must differ from base32 of the hex text itself (the old bug).
        assert_ne!(
            super::mfa_helpers::base32_encode_totp_secret(&hex_secret),
            super::mfa_helpers::base32_encode(&hex_secret)
        );
    }

    #[test]
    fn a_non_hex_secret_falls_back_without_panicking() {
        let encoded = super::mfa_helpers::base32_encode_totp_secret("not-hex!");
        assert!(!encoded.is_empty());
    }

    #[test]
    fn totp_codes_are_six_digits_and_change_with_the_counter() {
        let secret = hex::encode(b"12345678901234567890");
        let first = super::mfa_helpers::generate_totp_code(&secret, 1).expect("code");
        let again = super::mfa_helpers::generate_totp_code(&secret, 1).expect("code");
        let later = super::mfa_helpers::generate_totp_code(&secret, 2).expect("code");

        assert_eq!(first, again, "the same counter must give the same code");
        assert_eq!(first.len(), 6);
        assert!(first.chars().all(|c| c.is_ascii_digit()));
        assert_ne!(first, later, "a new counter must give a new code");
    }

    #[test]
    fn an_invalid_hex_secret_is_an_error_not_a_panic() {
        assert!(super::mfa_helpers::generate_totp_code("zz", 1).is_err());
    }

    #[test]
    fn verify_accepts_the_code_for_the_current_period() {
        let secret = hex::encode(b"12345678901234567890");
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_secs();
        let period = 30u64;
        let code = super::mfa_helpers::generate_totp_code(&secret, now / period).expect("code");

        assert!(super::mfa_helpers::verify_totp(&secret, &code, period));
        assert!(!super::mfa_helpers::verify_totp(&secret, "000000", period));
    }
}

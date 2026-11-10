//! TOTP encoding and verification, exercised offline at library level (#2114).
//!
//! Enrollment wrote the shared secret as `hex(raw key)` while the QR code
//! carried base32 of the same bytes, so the authenticator and the server
//! derived different HMAC keys and every code was rejected. `mfa_totp` now owns
//! one wire format (unpadded uppercase RFC 4648 base32) for both sides and
//! migrates legacy hex rows. These tests pin the encoding against the RFC
//! vectors, the acceptance criterion (a code from the QR secret verifies), the
//! ±1 period window and the legacy migration.

use botsecurity_auth::mfa::{
    base32_decode, base32_encode_bytes, generate_totp_code_at, verify_totp_at, MfaConfig,
    TotpAlgorithm, TotpEnrollment, TotpSecret,
};

/// RFC 4648 §10 base32 vectors, unpadded.
const BASE32_VECTORS: &[(&[u8], &str)] = &[
    (b"", ""),
    (b"f", "MY"),
    (b"fo", "MZXQ"),
    (b"foo", "MZXW6"),
    (b"foob", "MZXW6YQ"),
    (b"fooba", "MZXW6YTB"),
    (b"foobar", "MZXW6YTBOI"),
];

/// RFC 6238 Appendix B: `(time, SHA-1, SHA-256, SHA-512)` with 8 digits.
const RFC6238_VECTORS: &[(u64, &str, &str, &str)] = &[
    (59, "94287082", "46119246", "90693936"),
    (1111111109, "07081804", "68084774", "25091201"),
    (1111111111, "14050471", "67062674", "99943326"),
    (1234567890, "89005924", "91819424", "93441116"),
    (2000000000, "69279037", "90698825", "38618901"),
    (20000000000, "65353130", "77737706", "47863826"),
];

const SHA1_KEY: &[u8] = b"12345678901234567890";
const SHA256_KEY: &[u8] = b"12345678901234567890123456789012";
const SHA512_KEY: &[u8] = b"1234567890123456789012345678901234567890123456789012345678901234";

/// The base32 form of [`SHA1_KEY`] — the string the RFC 6238 vectors assume.
const SHA1_BASE32: &str = "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ";

/// A fixed instant inside a 30 s period, so no test reads the clock.
const AT: u64 = 1_700_000_000;

fn secret(key: &[u8]) -> TotpSecret {
    TotpSecret::from_bytes(key.to_vec()).expect("a non-empty key is a valid secret")
}

fn code_at(secret: &TotpSecret, unix_seconds: u64) -> String {
    generate_totp_code_at(secret, unix_seconds, 30, 6, TotpAlgorithm::Sha1)
        .expect("a 30 s / 6 digit / SHA-1 profile always produces a code")
}

fn accepts(secret: &TotpSecret, code: &str, unix_seconds: u64, window: i64) -> bool {
    verify_totp_at(secret, code, unix_seconds, 30, 6, TotpAlgorithm::Sha1, window)
}

#[test]
fn rfc4648_base32_vectors_round_trip() {
    for (plain, encoded) in BASE32_VECTORS {
        assert_eq!(
            base32_encode_bytes(plain),
            *encoded,
            "base32 of {plain:?} must be {encoded}"
        );
        assert_eq!(
            base32_decode(encoded).expect("a published vector decodes"),
            plain.to_vec(),
            "{encoded} must decode back to {plain:?}"
        );
    }
}

#[test]
fn rfc6238_vectors_match_for_every_algorithm() {
    let sha1 = secret(SHA1_KEY);
    let sha256 = secret(SHA256_KEY);
    let sha512 = secret(SHA512_KEY);

    for (time, expected_sha1, expected_sha256, expected_sha512) in RFC6238_VECTORS {
        assert_eq!(
            generate_totp_code_at(&sha1, *time, 30, 8, TotpAlgorithm::Sha1).expect("SHA-1 code"),
            *expected_sha1,
            "SHA-1 at T={time}"
        );
        assert_eq!(
            generate_totp_code_at(&sha256, *time, 30, 8, TotpAlgorithm::Sha256).expect("SHA-256 code"),
            *expected_sha256,
            "SHA-256 at T={time}"
        );
        assert_eq!(
            generate_totp_code_at(&sha512, *time, 30, 8, TotpAlgorithm::Sha512).expect("SHA-512 code"),
            *expected_sha512,
            "SHA-512 at T={time}"
        );
    }
}

#[test]
fn the_six_digit_profile_is_the_tail_of_the_eight_digit_code() {
    let key = secret(SHA1_KEY);
    let eight = generate_totp_code_at(&key, 59, 30, 8, TotpAlgorithm::Sha1).expect("code");
    let six = generate_totp_code_at(&key, 59, 30, 6, TotpAlgorithm::Sha1).expect("code");

    assert_eq!(eight, "94287082");
    assert_eq!(six, "287082");
    assert_eq!(six.len(), 6, "the code keeps its leading zeros");
}

#[test]
fn a_code_from_the_qr_secret_verifies_on_the_server() {
    let config = MfaConfig::default();
    let enrollment = TotpEnrollment::new(uuid::Uuid::new_v4(), "user@example.com", &config);
    let uri = enrollment.to_uri();

    assert!(uri.contains("algorithm=SHA1"));
    assert!(uri.contains("digits=6"));
    assert!(uri.contains("period=30"));

    // What an authenticator app reads out of the QR code.
    let scanned = uri
        .split_once("secret=")
        .and_then(|(_, rest)| rest.split('&').next())
        .expect("the URI carries a secret parameter");
    let scanned = TotpSecret::from_base32(scanned).expect("the QR secret is valid base32");

    let code = code_at(&scanned, AT);
    assert!(
        accepts(&enrollment.secret, &code, AT, 0),
        "the server must accept a code derived from the QR secret"
    );
    assert_eq!(scanned, enrollment.secret, "both sides share one key");

    // The stored row and the QR text are the same 32-character base32 string.
    let stored = enrollment.secret_base32();
    assert_eq!(stored.len(), 32, "a 20-byte key is 32 base32 characters");
    assert!(!stored.contains('='), "the wire format is unpadded");
    assert_eq!(stored, scanned.to_base32());
}

#[test]
fn a_code_from_another_secret_is_refused() {
    let server = secret(SHA1_KEY);
    let other = secret(b"an-entirely-different-totp-key");

    let foreign_code = code_at(&other, AT);
    assert!(
        !accepts(&server, &foreign_code, AT, 1),
        "a code from another enrollment must not open this account"
    );
}

#[test]
fn a_tampered_or_empty_code_is_refused() {
    let key = secret(SHA1_KEY);
    let valid = code_at(&key, AT);

    let mut digits: Vec<char> = valid.chars().collect();
    digits[0] = if digits[0] == '0' { '1' } else { '0' };
    let tampered: String = digits.into_iter().collect();

    assert!(!accepts(&key, &tampered, AT, 1));
    assert!(!accepts(&key, "", AT, 1));
    assert!(!accepts(&key, "abcdef", AT, 1));
}

#[test]
fn the_window_accepts_one_period_either_side_and_nothing_further() {
    let key = secret(SHA1_KEY);

    let previous = code_at(&key, AT - 30);
    let current = code_at(&key, AT);
    let next = code_at(&key, AT + 30);
    let stale = code_at(&key, AT - 120);
    let early = code_at(&key, AT + 120);

    assert!(accepts(&key, &current, AT, 1));
    assert!(accepts(&key, &previous, AT, 1), "one period of clock skew back");
    assert!(accepts(&key, &next, AT, 1), "one period of clock skew forward");
    assert!(!accepts(&key, &stale, AT, 1), "two periods back is expired");
    assert!(!accepts(&key, &early, AT, 1), "two periods ahead is not valid yet");
}

#[test]
fn a_zero_window_accepts_only_the_current_period() {
    let key = secret(SHA1_KEY);

    assert!(accepts(&key, &code_at(&key, AT), AT, 0));
    assert!(!accepts(&key, &code_at(&key, AT - 30), AT, 0));
    assert!(!accepts(&key, &code_at(&key, AT + 30), AT, 0));
}

#[test]
fn a_zero_period_is_refused() {
    let key = secret(SHA1_KEY);

    assert!(
        generate_totp_code_at(&key, AT, 0, 6, TotpAlgorithm::Sha1).is_err(),
        "a zero period has no counter"
    );
    assert!(!verify_totp_at(&key, "123456", AT, 0, 6, TotpAlgorithm::Sha1, 1));
}

#[test]
fn a_legacy_hex_row_migrates_to_the_same_base32_key() {
    let mut legacy = String::with_capacity(SHA1_KEY.len() * 2);
    for byte in SHA1_KEY {
        legacy.push_str(&format!("{byte:02x}"));
    }

    assert!(TotpSecret::is_legacy_hex(&legacy));
    let migrated = TotpSecret::parse_stored(&legacy).expect("a legacy row still parses");
    let canonical = TotpSecret::from_base32(SHA1_BASE32).expect("the canonical base32 key");

    assert_eq!(migrated, canonical, "both encodings name the same key bytes");
    assert_eq!(migrated.to_base32(), SHA1_BASE32, "the row is rewritten as base32");
    assert!(!TotpSecret::is_legacy_hex(&migrated.to_base32()));

    // The migrated row verifies codes exactly like a freshly enrolled one.
    assert!(accepts(&migrated, &code_at(&canonical, AT), AT, 0));
}

#[test]
fn a_base32_row_round_trips_through_storage() {
    let key = secret(SHA1_KEY);
    let stored = TotpSecret::parse_stored(&key.to_base32()).expect("base32 is the canonical form");

    assert_eq!(stored, key);
    assert!(TotpSecret::from_base32("gezdgnbvgy3tqojqgezdgnbvgy3tqojq").is_ok(), "case-insensitive");
}

#[test]
fn the_secret_serializes_as_base32_and_never_debug_prints_bytes() {
    let key = secret(SHA1_KEY);

    let json = serde_json::to_string(&key).expect("a secret serializes");
    assert_eq!(json, format!("\"{SHA1_BASE32}\""));

    let back: TotpSecret = serde_json::from_str(&json).expect("a stored secret deserializes");
    assert_eq!(back, key);

    let debug = format!("{key:?}");
    assert!(debug.contains("redacted"), "a secret must not reach a log line");
    assert!(!debug.contains("GEZD"));
}

#[test]
fn an_invalid_secret_is_an_error_not_a_panic() {
    assert!(TotpSecret::from_base32("!!!!").is_err());
    assert!(TotpSecret::from_bytes(Vec::new()).is_err());
    assert!(TotpSecret::parse_stored("!!!").is_err());
}

//! RFC 4648 / RFC 6238 vectors plus the end-to-end QR → verify path (#2114).

use crate::mfa::mfa_totp::{
    base32_decode, base32_encode_bytes, generate_totp_code_at, verify_totp_at, TotpSecret,
};
use crate::mfa::mfa_types::{MfaConfig, TotpAlgorithm, TotpEnrollment};

/// RFC 4648 §10 test vectors for base32 (no padding).
const BASE32_VECTORS: &[(&[u8], &str)] = &[
    (b"", ""),
    (b"f", "MY"),
    (b"fo", "MZXQ"),
    (b"foo", "MZXW6"),
    (b"foob", "MZXW6YQ"),
    (b"fooba", "MZXW6YTB"),
    (b"foobar", "MZXW6YTBOI"),
];

#[test]
fn rfc4648_base32_vectors_encode_and_decode() {
    for (plain, encoded) in BASE32_VECTORS {
        assert_eq!(
            base32_encode_bytes(plain),
            *encoded,
            "base32 of {plain:?} must be {encoded}"
        );
        assert_eq!(
            base32_decode(encoded).expect("decode"),
            plain.to_vec(),
            "base32 decode of {encoded} must round-trip"
        );
    }
}

/// RFC 6238 Appendix B: (time, SHA-1, SHA-256, SHA-512) with 8 digits.
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
const SHA512_KEY: &[u8] =
    b"1234567890123456789012345678901234567890123456789012345678901234";

#[test]
fn rfc6238_sha1_vectors() {
    let secret = TotpSecret::from_bytes(SHA1_KEY.to_vec()).expect("secret");
    for (time, expected, _, _) in RFC6238_VECTORS {
        assert_eq!(
            generate_totp_code_at(&secret, *time, 30, 8, TotpAlgorithm::Sha1).expect("code"),
            *expected,
            "SHA-1 at T={time}"
        );
    }
}

#[test]
fn rfc6238_sha256_vectors() {
    let secret = TotpSecret::from_bytes(SHA256_KEY.to_vec()).expect("secret");
    for (time, _, expected, _) in RFC6238_VECTORS {
        assert_eq!(
            generate_totp_code_at(&secret, *time, 30, 8, TotpAlgorithm::Sha256).expect("code"),
            *expected,
            "SHA-256 at T={time}"
        );
    }
}

#[test]
fn rfc6238_sha512_vectors() {
    let secret = TotpSecret::from_bytes(SHA512_KEY.to_vec()).expect("secret");
    for (time, _, _, expected) in RFC6238_VECTORS {
        assert_eq!(
            generate_totp_code_at(&secret, *time, 30, 8, TotpAlgorithm::Sha512).expect("code"),
            *expected,
            "SHA-512 at T={time}"
        );
    }
}

#[test]
fn rfc6238_sha1_vector_has_an_eight_digit_width() {
    let secret = TotpSecret::from_bytes(SHA1_KEY.to_vec()).expect("secret");
    let code = generate_totp_code_at(&secret, 59, 30, 8, TotpAlgorithm::Sha1).expect("code");
    assert_eq!(code.len(), 8);

    let six = generate_totp_code_at(&secret, 59, 30, 6, TotpAlgorithm::Sha1).expect("code");
    assert_eq!(six, "287082", "6 digits are the last 6 of the 8-digit code");
}

fn secret_from_uri(uri: &str) -> TotpSecret {
    let query = uri.split_once('?').expect("query").1;
    let secret = query
        .split('&')
        .find_map(|pair| pair.strip_prefix("secret="))
        .expect("secret parameter");
    TotpSecret::from_base32(secret).expect("base32 secret")
}

/// The acceptance criterion: a code derived from the QR `secret` verifies on
/// the server, using the algorithm/digits/period the URI advertises.
#[test]
fn a_code_from_the_qr_secret_verifies_on_the_server() {
    let config = MfaConfig::default();
    let enrollment = TotpEnrollment::new(uuid::Uuid::new_v4(), "user@example.com", &config);
    let uri = enrollment.to_uri();

    assert!(uri.contains("algorithm=SHA1"));
    assert!(uri.contains("digits=6"));
    assert!(uri.contains("period=30"));

    let scanned = secret_from_uri(&uri);
    let at = 1_700_000_000u64;
    let code = generate_totp_code_at(
        &scanned,
        at,
        enrollment.period,
        enrollment.digits,
        enrollment.algorithm,
    )
    .expect("code");

    assert!(verify_totp_at(
        &enrollment.secret,
        &code,
        at,
        enrollment.period,
        enrollment.digits,
        enrollment.algorithm,
        0,
    ));

    // The URI secret and the stored secret are the same key, by construction.
    assert_eq!(scanned, enrollment.secret);
    assert_eq!(enrollment.secret_base32(), scanned.to_base32());
    assert!(!enrollment.secret_base32().contains('='));
}

#[test]
fn a_tampered_code_is_rejected() {
    let secret = TotpSecret::from_bytes(SHA1_KEY.to_vec()).expect("secret");
    let at = 1_700_000_000u64;
    let code = generate_totp_code_at(&secret, at, 30, 6, TotpAlgorithm::Sha1).expect("code");
    let mut tampered: Vec<char> = code.chars().collect();
    tampered[0] = if tampered[0] == '0' { '1' } else { '0' };
    let tampered: String = tampered.into_iter().collect();

    assert!(!verify_totp_at(
        &secret,
        &tampered,
        at,
        30,
        6,
        TotpAlgorithm::Sha1,
        1,
    ));
    assert!(!verify_totp_at(&secret, "", at, 30, 6, TotpAlgorithm::Sha1, 1));
    assert!(!verify_totp_at(&secret, "abc", at, 30, 6, TotpAlgorithm::Sha1, 1));
}

#[test]
fn the_window_accepts_one_period_either_side_and_rejects_further_away() {
    let secret = TotpSecret::from_bytes(SHA1_KEY.to_vec()).expect("secret");
    let at = 1_700_000_000u64;

    let previous = generate_totp_code_at(&secret, at - 30, 30, 6, TotpAlgorithm::Sha1)
        .expect("previous code");
    let next = generate_totp_code_at(&secret, at + 30, 30, 6, TotpAlgorithm::Sha1)
        .expect("next code");
    let stale = generate_totp_code_at(&secret, at - 90, 30, 6, TotpAlgorithm::Sha1)
        .expect("stale code");

    assert!(verify_totp_at(&secret, &previous, at, 30, 6, TotpAlgorithm::Sha1, 1));
    assert!(verify_totp_at(&secret, &next, at, 30, 6, TotpAlgorithm::Sha1, 1));
    assert!(!verify_totp_at(&secret, &stale, at, 30, 6, TotpAlgorithm::Sha1, 1));
}

#[test]
fn a_legacy_hex_row_is_read_as_the_same_key_and_rewritten_as_base32() {
    let legacy = hex::encode(SHA1_KEY);
    let migrated = TotpSecret::parse_stored(&legacy).expect("legacy secret");
    let canonical = TotpSecret::from_base32("GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ").expect("secret");

    assert_eq!(migrated, canonical);
    assert_eq!(migrated.to_base32(), "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ");
    assert!(TotpSecret::is_legacy_hex(&legacy));
    assert!(!TotpSecret::is_legacy_hex(&migrated.to_base32()));
}

#[test]
fn the_secret_serializes_as_base32_and_never_debug_prints_bytes() {
    let secret = TotpSecret::from_bytes(SHA1_KEY.to_vec()).expect("secret");
    let json = serde_json::to_string(&secret).expect("serialize");
    assert_eq!(json, "\"GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ\"");

    let back: TotpSecret = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back, secret);

    let debug = format!("{secret:?}");
    assert!(debug.contains("redacted"));
    assert!(!debug.contains("GEZD"));
}

#[test]
fn an_invalid_secret_is_an_error_not_a_panic() {
    assert!(TotpSecret::from_base32("!!!!").is_err());
    assert!(TotpSecret::from_bytes(Vec::new()).is_err());
    assert!(TotpSecret::parse_stored("!!!").is_err());
}

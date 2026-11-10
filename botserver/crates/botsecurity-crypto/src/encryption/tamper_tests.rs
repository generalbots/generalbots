//! Ciphertext integrity matrix for AES-256-GCM field encryption (#2066).
//!
//! GCM authenticates the nonce and the ciphertext: every single-bit change
//! must fail closed. These tests walk each byte instead of asserting one
//! happy-path round trip, which cannot detect a consistently-broken cipher.

use super::*;

fn key(seed: u8) -> Vec<u8> {
    vec![seed; KEY_SIZE]
}

fn decode_b64(value: &str) -> Vec<u8> {
    BASE64.decode(value).expect("base64")
}

#[test]
fn a_clean_round_trip_succeeds_for_ascii_and_multibyte_text() {
    let key = key(1);
    let long = "a".repeat(4096);
    for plaintext in ["", "top secret", "contrato ção 日本語 😀", long.as_str()] {
        let encrypted = encrypt_field(plaintext, &key).expect("encrypt");
        assert_eq!(decrypt_field(&encrypted, &key).expect("decrypt"), plaintext);
    }
}

#[test]
fn every_flipped_ciphertext_bit_is_rejected() {
    let key = key(2);
    let (nonce, ciphertext) = encrypt_aes_gcm(b"attack at dawn", &key).expect("encrypt");
    let original = decode_b64(&ciphertext);
    assert!(!original.is_empty());

    for index in 0..original.len() {
        for bit in 0..8 {
            let mut tampered = original.clone();
            tampered[index] ^= 1 << bit;
            assert!(
                decrypt_aes_gcm(&nonce, &BASE64.encode(&tampered), &key).is_err(),
                "byte {index} bit {bit} was accepted"
            );
        }
    }
}

#[test]
fn every_flipped_nonce_bit_is_rejected() {
    let key = key(3);
    let (nonce, ciphertext) = encrypt_aes_gcm(b"nonce matters", &key).expect("encrypt");
    let original = decode_b64(&nonce);

    for index in 0..original.len() {
        for bit in 0..8 {
            let mut tampered = original.clone();
            tampered[index] ^= 1 << bit;
            assert!(
                decrypt_aes_gcm(&BASE64.encode(&tampered), &ciphertext, &key).is_err(),
                "nonce byte {index} bit {bit} was accepted"
            );
        }
    }
}

#[test]
fn truncated_ciphertexts_are_rejected() {
    let key = key(4);
    let (nonce, ciphertext) = encrypt_aes_gcm(b"truncate me", &key).expect("encrypt");
    let original = decode_b64(&ciphertext);

    assert!(decrypt_aes_gcm(&nonce, &BASE64.encode(Vec::<u8>::new()), &key).is_err());
    for len in 0..original.len() {
        assert!(
            decrypt_aes_gcm(&nonce, &BASE64.encode(&original[..len]), &key).is_err(),
            "ciphertext truncated to {len} bytes was accepted"
        );
    }
}

#[test]
fn truncated_and_extended_nonces_are_rejected() {
    let key = key(5);
    let (_, ciphertext) = encrypt_aes_gcm(b"data", &key).expect("encrypt");

    for len in [0usize, 1, 11, 13, 24] {
        let nonce = BASE64.encode(vec![0u8; len]);
        let err = decrypt_aes_gcm(&nonce, &ciphertext, &key).expect_err("must fail");
        assert!(err.to_string().contains("Invalid nonce size"));
    }
}

#[test]
fn a_different_key_can_never_decrypt() {
    let (nonce, ciphertext) = encrypt_aes_gcm(b"confidential", &key(6)).expect("encrypt");
    for seed in [0u8, 5, 7, 255] {
        assert!(decrypt_aes_gcm(&nonce, &ciphertext, &key(seed)).is_err());
    }
}

#[test]
fn a_tampered_field_payload_is_rejected_at_every_component() {
    let key = key(8);
    let encrypted = encrypt_field("patient record", &key).expect("encrypt");
    let parts: Vec<&str> = encrypted.split(':').collect();
    assert_eq!(parts.len(), 3);

    // Version bump.
    assert!(decrypt_field(&format!("2:{}:{}", parts[1], parts[2]), &key).is_err());
    // Nonce tamper.
    let mut nonce = decode_b64(parts[1]);
    nonce[0] ^= 0x80;
    assert!(decrypt_field(
        &format!("1:{}:{}", BASE64.encode(nonce), parts[2]),
        &key
    )
    .is_err());
    // Ciphertext tamper.
    let mut body = decode_b64(parts[2]);
    body[0] ^= 0x01;
    assert!(decrypt_field(
        &format!("1:{}:{}", parts[1], BASE64.encode(body)),
        &key
    )
    .is_err());
    // Removed component.
    assert!(decrypt_field(&format!("1:{}", parts[1]), &key).is_err());
}

#[test]
fn a_field_encrypted_under_one_key_is_unreadable_under_another() {
    let encrypted = encrypt_field("salary=1000", &key(9)).expect("encrypt");
    assert!(decrypt_field(&encrypted, &key(10)).is_err());
    assert_eq!(
        decrypt_field(&encrypted, &key(9)).expect("decrypt"),
        "salary=1000"
    );
}

#[test]
fn non_base64_components_are_rejected_without_panicking() {
    let key = key(11);
    for payload in [
        "1:!!!:AAAA",
        "1:AAAA:!!!",
        "1::",
        "::",
        "1:AAAA:AAAA:AAAA",
        "x:AAAA:AAAA",
        "-1:AAAA:AAAA",
    ] {
        assert!(decrypt_field(payload, &key).is_err(), "{payload} accepted");
    }
}

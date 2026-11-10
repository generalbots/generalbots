//! #2069 CLD-1/CLD-2 — the bearer-token segment decoder parses every cloud
//! request, so it is differential-tested against an independent reference
//! implementation instead of being trusted by inspection.
use super::*;

const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

/// Independent 6-bits-at-a-time reference decoder (unpadded URL-safe base64).
fn reference_decode(input: &str) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let mut buf: u32 = 0;
    let mut bits = 0u32;
    for ch in input.bytes() {
        if ch == b'=' {
            break;
        }
        let val = ALPHABET.iter().position(|c| *c == ch)? as u32;
        buf = (buf << 6) | val;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push(((buf >> bits) & 0xFF) as u8);
        }
    }
    // A lone trailing 6-bit group cannot encode a byte.
    if bits >= 6 {
        return None;
    }
    Some(out)
}

/// Deterministic pseudo-random byte vectors: no `rand` seed juggling in tests.
fn sample_bytes(len: usize) -> Vec<u8> {
    (0..len).map(|i| (i.wrapping_mul(37).wrapping_add(11) & 0xFF) as u8).collect()
}

#[test]
fn cld1_round_trips_every_length_from_zero_to_eight_bytes() {
    for len in 0..=8 {
        let bytes = sample_bytes(len);
        let encoded = base64_url_encode(&bytes);
        assert_eq!(
            base64_url_decode(&encoded).expect("decode"),
            bytes,
            "len {len} (encoded {encoded:?})"
        );
    }
}

#[test]
fn cld1_matches_the_independent_reference_decoder() {
    for len in 0..=64 {
        let bytes = sample_bytes(len);
        let encoded = base64_url_encode(&bytes);
        assert_eq!(
            base64_url_decode(&encoded).expect("decode"),
            reference_decode(&encoded).expect("reference"),
            "differential mismatch at len {len}"
        );
    }
}

#[test]
fn cld1_accepts_padding_and_both_url_safe_letters() {
    // "-" and "_" are the two characters that differ from standard base64.
    let bytes: Vec<u8> = vec![0xfb, 0xff, 0xbf, 0x00];
    let encoded = base64_url_encode(&bytes);
    assert!(encoded.contains('-') || encoded.contains('_'), "sample must exercise -/_");
    assert_eq!(base64_url_decode(&encoded).expect("decode"), bytes);

    // The same segment with the padding a non-JWT encoder would emit.
    let padded = format!("{encoded}==");
    assert_eq!(base64_url_decode(&padded).expect("decode"), bytes);
}

#[test]
fn cld2_rejects_an_impossible_length() {
    // len % 4 == 1 can never come from an encoder (one spare 6-bit group).
    for input in ["A", "AAAAA", "AAAAAAAAA"] {
        assert_eq!(base64_url_decode(input), Err("invalid base64url input"), "{input:?}");
    }
}

#[test]
fn cld2_rejects_characters_outside_the_alphabet_instead_of_skipping_them() {
    // #2069 — the old decoder dropped these and returned the *other* bytes.
    for input in ["!!!", "eyJ!x", "AAAA AAAA", "AAAA\nAAAA", "eyJ+YQ", "eyJ/YQ", "AA=AA"] {
        assert_eq!(
            base64_url_decode(input),
            Err("invalid base64url input"),
            "{input:?} must not decode"
        );
    }
}

#[test]
fn cld2_rejects_non_ascii_input_without_panicking() {
    for input in ["é", "eyJé", "日本語", "\u{0}"] {
        assert!(base64_url_decode(input).is_err(), "{input:?}");
    }
}

#[test]
fn an_empty_segment_decodes_to_no_bytes() {
    assert_eq!(base64_url_decode(""), Ok(Vec::new()));
    assert_eq!(base64_url_decode("=="), Ok(Vec::new()));
}

#[test]
fn signed_tokens_carry_a_verifiable_segment() {
    let payload = r#"{"sub":"u-1"}"#;
    let signed = jwt_sign("header", payload, b"secret").expect("sign");
    let parts: Vec<&str> = signed.split('.').collect();
    assert_eq!(parts.len(), 3);
    assert_eq!(parts[0], "header");
    assert_eq!(parts[1], payload);
    assert_eq!(base64_url_decode(parts[2]).expect("signature").len(), 32);
}

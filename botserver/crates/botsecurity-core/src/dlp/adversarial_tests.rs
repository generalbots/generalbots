//! Adversarial DLP tests (#2066): published Luhn vectors, redaction
//! completeness, and the context extractor's byte/char boundary handling.

use super::extract_context::{extract_context, validate_credit_card_checksum};
use super::*;

#[test]
fn luhn_accepts_the_published_valid_vectors() {
    // Publicly documented test numbers (Stripe/PayPal docs, ISO/IEC 7812).
    let valid = [
        "4111111111111111",  // Visa
        "4012888888881881",  // Visa
        "5500005555555559",  // Mastercard
        "5105105105105100",  // Mastercard
        "378282246310005",   // Amex
        "371449635398431",   // Amex
        "6011111111111117",  // Discover
        "3530111333300000",  // JCB
        "30569309025904",    // Diners
    ];
    for number in valid {
        assert!(
            validate_credit_card_checksum(number),
            "{number} must pass the Luhn check"
        );
    }
}

#[test]
fn luhn_rejects_the_published_invalid_vectors() {
    let invalid = [
        "4111111111111112",
        "5500005555555558",
        "378282246310006",
        "6011111111111118",
        "3530111333300001",
        "1234567890123456",
    ];
    for number in invalid {
        assert!(
            !validate_credit_card_checksum(number),
            "{number} must fail the Luhn check"
        );
    }
}

#[test]
fn luhn_ignores_separators_but_enforces_the_length_window() {
    assert!(validate_credit_card_checksum("4111-1111-1111-1111"));
    assert!(validate_credit_card_checksum("4111 1111 1111 1111"));
    assert!(validate_credit_card_checksum("4111a1111b1111c1111"));

    // 12 digits is below the ISO minimum, 20 above the maximum.
    assert!(!validate_credit_card_checksum("411111111111"));
    assert!(!validate_credit_card_checksum("41111111111111111111"));
    assert!(!validate_credit_card_checksum(""));
    assert!(!validate_credit_card_checksum("not a number"));
}

#[tokio::test]
async fn a_valid_card_number_is_detected_and_fully_redacted() {
    let manager = DlpManager::with_defaults().expect("manager");
    let content = "card 4111 1111 1111 1111 exp 12/30";
    let result = manager.scan(content).await;
    assert!(result.has_sensitive_data());
    assert!(result
        .data_types_found()
        .contains(&SensitiveDataType::CreditCard));

    let redacted = manager.redact_content(content, &result.matches);
    assert!(!redacted.contains("4111"));
    assert!(!redacted.contains("1111 1111"));
    assert!(redacted.contains("[CARD REDACTED]"));
    assert!(redacted.contains("exp 12/30"), "non-sensitive text survives");
}

#[tokio::test]
async fn every_occurrence_of_a_secret_is_redacted() {
    let manager = DlpManager::with_defaults().expect("manager");
    let content = "a@example.com b@example.com c@example.com";
    let result = manager.scan(content).await;
    let emails = result
        .matches
        .iter()
        .filter(|m| m.data_type == SensitiveDataType::Email)
        .count();
    assert_eq!(emails, 3, "each address must be matched");

    let redacted = manager.redact_content(content, &result.matches);
    for secret in ["a@example.com", "b@example.com", "c@example.com"] {
        assert!(!redacted.contains(secret), "{secret} leaked");
    }
    assert_eq!(redacted.matches("[EMAIL REDACTED]").count(), 3);
}

#[tokio::test]
async fn partial_redaction_keeps_the_last_four_digits_only() {
    let manager = DlpManager::with_defaults().expect("manager");
    let content = "pay with 4111111111111111 now";
    let result = manager.scan(content).await;
    let redacted = manager.partial_redact_content(content, &result.matches);
    assert!(!redacted.contains("4111111111111111"));
    assert!(redacted.contains("41"), "{redacted}");
    assert!(redacted.contains("**"), "{redacted}");
}

#[tokio::test]
async fn credentials_and_tokens_are_detected_then_removed() {
    let manager = DlpManager::with_defaults().expect("manager");
    let cases = [
        (
            "aws AKIAIOSFODNN7EXAMPLE key",
            "AKIAIOSFODNN7EXAMPLE",
            SensitiveDataType::AwsAccessKey,
        ),
        (
            "token eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxIn0.c2lnbmF0dXJl",
            "eyJhbGciOiJIUzI1NiJ9",
            SensitiveDataType::JwtToken,
        ),
        (
            "-----BEGIN PRIVATE KEY-----\nMIIEvQIBADANBgkq\n-----END PRIVATE KEY-----",
            "-----BEGIN PRIVATE KEY-----",
            SensitiveDataType::PrivateKey,
        ),
    ];
    for (content, secret, data_type) in cases {
        let result = manager.scan(content).await;
        assert!(
            result.data_types_found().contains(&data_type),
            "{data_type:?} must be detected in {content:?}"
        );
        let redacted = manager.redact_content(content, &result.matches);
        assert!(!redacted.contains(secret), "{secret} leaked from {content:?}");
    }
}

#[test]
fn context_extraction_never_splits_a_multibyte_character() {
    // 'α' is two bytes; every padding offset below lands inside one of them.
    let content = "αααααααααα";
    for padding in 0..=content.len() {
        for start in 0..=content.len() {
            let end = start.min(content.len());
            let excerpt = extract_context(content, start, end, padding);
            assert!(excerpt.starts_with("...") || excerpt.starts_with('α'));
        }
    }
}

#[test]
fn context_extraction_keeps_the_whole_match_and_marks_truncation() {
    let content = "0123456789SECRET0123456789";
    let excerpt = extract_context(content, 10, 16, 4);
    assert!(excerpt.starts_with("..."));
    assert!(excerpt.ends_with("..."));
    assert!(excerpt.contains("SECRET"));
    assert!(excerpt.len() < content.len());
}

#[test]
fn context_extraction_clamps_out_of_range_offsets() {
    let content = "short";
    assert_eq!(extract_context(content, 0, content.len(), 0), "short");
    assert_eq!(extract_context(content, 99, 200, 0), "...");
    assert_eq!(
        extract_context(content, 99, 200, 5),
        "...",
        "an out-of-range start clamps to the end of the content"
    );
}

#[test]
fn a_multibyte_match_at_the_padding_edge_is_not_panicking() {
    // Regression for the #2066 panic: the match ends inside a 4-byte emoji.
    let content = "text 😀 more text";
    let emoji_start = content.find('😀').expect("emoji");
    let excerpt = extract_context(content, emoji_start, emoji_start + 4, 1);
    assert!(excerpt.contains('😀'));
    assert!(excerpt.starts_with("..."));
}

fn dlp_match(data_type: SensitiveDataType, start: usize, end: usize) -> DlpMatch {
    DlpMatch {
        data_type,
        matched_text: String::new(),
        start_position: start,
        end_position: end,
        confidence: 1.0,
        context: None,
    }
}

#[test]
fn overlapping_matches_are_redacted_once_without_panicking() {
    let manager = DlpManager::with_defaults().expect("manager");
    let content = "aaaa user@example.com";
    // The email match and a fabricated SSN match share the same span, and the
    // card match extends to the end of the string: both shapes occur when the
    // built-in patterns overlap.
    let matches = vec![
        dlp_match(SensitiveDataType::Email, 5, content.len()),
        dlp_match(SensitiveDataType::Ssn, 5, content.len()),
        dlp_match(SensitiveDataType::CreditCard, 0, content.len()),
    ];

    let redacted = manager.redact_content(content, &matches);
    assert!(!redacted.contains("user@example.com"));
    assert_eq!(redacted, "[CARD REDACTED]");

    let partial = manager.partial_redact_content(content, &matches);
    assert!(!partial.contains("user@example.com"));
}

#[test]
fn out_of_range_and_split_matches_never_panic() {
    let manager = DlpManager::with_defaults().expect("manager");
    let content = "αβγδε";

    let out_of_range = vec![dlp_match(SensitiveDataType::Email, 0, 9_999)];
    let redacted = manager.redact_content(content, &out_of_range);
    assert_eq!(redacted, "[EMAIL REDACTED]");

    // Offsets 1..3 split the two-byte 'α'; the match is skipped rather than
    // slicing inside a character.
    let split = vec![dlp_match(SensitiveDataType::Email, 1, 3)];
    assert_eq!(manager.redact_content(content, &split), content);
}

#[test]
fn masking_a_multibyte_value_does_not_panic() {
    let m = DlpMatch {
        data_type: SensitiveDataType::Custom,
        matched_text: "😀😀😀😀😀😀".to_string(),
        start_position: 0,
        end_position: 24,
        confidence: 1.0,
        context: None,
    };
    let masked = m.masked_value();
    assert_eq!(masked.chars().count(), 6);
    assert!(masked.starts_with("😀😀"));
    assert!(masked.ends_with("😀😀"));
}

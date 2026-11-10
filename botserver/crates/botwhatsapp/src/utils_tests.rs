//! Pure WhatsApp helpers: phone normalization, list detection, chunking,
//! inbound text extraction, timestamps and the webhook signature (WA-1..WA-5,
//! WA-7 in #2070).

use super::*;
use crate::models::{
    ButtonContent, ButtonReplyContent, ImageContent, InteractiveContent, ListReplyContent,
    TextContent, WhatsAppMessage,
};

fn message(message_type: &str) -> WhatsAppMessage {
    WhatsAppMessage {
        from: Some("5511987654321".to_string()),
        id: Some("wamid.1".to_string()),
        timestamp: Some("1700000000".to_string()),
        message_type: Some(message_type.to_string()),
        text: None,
        image: None,
        video: None,
        audio: None,
        document: None,
        sticker: None,
        interactive: None,
        button: None,
    }
}

// ---------------------------------------------------------------- WA-1 phone

#[test]
fn brazilian_numbers_gain_the_country_code() {
    assert_eq!(format_phone_number("11987654321"), "5511987654321");
    assert_eq!(format_phone_number("011987654321"), "5511987654321");
}

#[test]
fn numbers_already_international_are_unchanged() {
    assert_eq!(format_phone_number("5511987654321"), "5511987654321");
    assert_eq!(format_phone_number("+5511987654321"), "5511987654321");
}

#[test]
fn punctuation_and_spaces_are_stripped() {
    assert_eq!(format_phone_number("+55 (11) 98765-4321"), "5511987654321");
    assert_eq!(format_phone_number(" 11 9 8765 4321 "), "5511987654321");
}

#[test]
fn an_input_without_digits_stays_empty() {
    // Regression: "" and "abc" used to become "55", which then addressed a
    // Graph API send to a nonexistent recipient.
    assert_eq!(format_phone_number(""), "");
    assert_eq!(format_phone_number("abc"), "");
    assert_eq!(format_phone_number("---"), "");
}

#[test]
fn a_leading_zero_is_dropped_even_for_a_short_number() {
    // Pinned: "0" is not long enough for the leading-zero branch, so it keeps
    // the country-code branch and becomes "550".
    assert_eq!(format_phone_number("0"), "550");
}

#[test]
fn unicode_digits_are_not_treated_as_digits() {
    // `is_ascii_digit` is what the code filters on: full-width digits are
    // dropped, so the result falls back to empty rather than a wrong number.
    assert_eq!(format_phone_number("１１９８７６５４３２１"), "");
}

// ----------------------------------------------------------- WA-2 list shapes

#[test]
fn two_or_more_bullet_lines_are_a_list() {
    assert!(is_list_message("- Item 1\n- Item 2\n- Item 3"));
    assert!(is_list_message("* Item 1\n* Item 2"));
    assert!(is_list_message("• Item 1\n• Item 2"));
    assert!(is_list_message("1. First\n2. Second\n3. Third"));
    // Mixed markers still count as a list.
    assert!(is_list_message("- One\n2. Two"));
}

#[test]
fn a_single_bullet_or_bullets_in_prose_are_not_a_list() {
    assert!(!is_list_message("Just a single line"));
    assert!(!is_list_message("- only one bullet"));
    assert!(!is_list_message("Hello\nWorld\nTest"));
    assert!(!is_list_message("Segue a lista:\n- um item"));
}

#[test]
fn a_decimal_number_is_not_a_list_marker() {
    assert!(!is_list_message("1.5 is a number\n2.5 is another"));
}

#[test]
fn a_numbered_marker_needs_a_space_or_the_end_of_the_line() {
    assert!(is_list_message("1.\n2."));
    assert!(is_list_message("1. First\n2. Second"));
    // No space after the dot: prose or a version, not a list.
    assert!(!is_list_message("1.item\n2.item"));
    assert!(!is_list_message("v1.2 is a version\nv1.3 is another"));
}

// ------------------------------------------------------- WA-3 message chunking

#[test]
fn a_message_at_the_limit_is_not_split() {
    let text = "a".repeat(4096);
    let parts = split_long_message(&text);

    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0], text);
}

#[test]
fn a_message_one_byte_over_the_limit_is_split() {
    let parts = split_long_message(&"a".repeat(4097));

    assert_eq!(parts.len(), 2);
    assert_eq!(parts[0].len(), 4096);
    assert_eq!(parts[1].len(), 1);
}

#[test]
fn short_multi_line_messages_keep_their_newlines() {
    let text = "Line 1\nLine 2\nLine 3";
    let parts = split_long_message(text);

    assert_eq!(parts, vec![text.to_string()]);
}

#[test]
fn one_huge_line_is_split_without_losing_bytes() {
    let text = "x".repeat(5000);
    let parts = split_long_message(&text);

    assert_eq!(parts.len(), 2);
    assert!(parts.iter().all(|part| part.len() <= 4096));
    assert_eq!(parts.concat(), text);
}

#[test]
fn a_multibyte_message_splits_on_char_boundaries() {
    // 3000 'é' = 6000 bytes: the byte length crosses the limit while the
    // character count does not, so the chunker must not slice a char in half.
    let text = "é".repeat(3000);
    let parts = split_long_message(&text);

    assert!(parts.len() > 1);
    for part in &parts {
        assert!(part.len() <= 4096, "part too long: {}", part.len());
        assert_eq!(part.chars().count() * 2, part.len());
    }
    assert_eq!(parts.concat(), text);
}

#[test]
fn an_empty_message_is_one_empty_part() {
    assert_eq!(split_long_message(""), vec![String::new()]);
}

#[test]
fn lines_are_packed_up_to_the_limit() {
    let line = "y".repeat(1000);
    let text = vec![line.as_str(); 6].join("\n"); // 6005 bytes
    let parts = split_long_message(&text);

    assert!(parts.len() >= 2);
    assert!(parts.iter().all(|part| part.len() <= 4096));
    // Every original line survives in order.
    assert_eq!(parts.join("\n"), text);
}

// ------------------------------------------------ WA-4 inbound text extraction

#[test]
fn a_text_message_yields_its_body() {
    let mut update = message("text");
    update.text = Some(TextContent {
        body: Some("bom dia".to_string()),
    });

    assert_eq!(extract_message_text(&update).as_deref(), Some("bom dia"));
}

#[test]
fn a_button_reply_yields_its_title() {
    let mut update = message("interactive");
    update.interactive = Some(InteractiveContent {
        interactive_type: Some("button_reply".to_string()),
        button_reply: Some(ButtonReplyContent {
            id: Some("btn-1".to_string()),
            title: Some("Confirmar".to_string()),
        }),
        list_reply: None,
    });

    assert_eq!(extract_message_text(&update).as_deref(), Some("Confirmar"));
}

#[test]
fn a_list_reply_yields_its_title() {
    let mut update = message("interactive");
    update.interactive = Some(InteractiveContent {
        interactive_type: Some("list_reply".to_string()),
        button_reply: None,
        list_reply: Some(ListReplyContent {
            id: Some("row-1".to_string()),
            title: Some("Segunda opção".to_string()),
            description: None,
        }),
    });

    assert_eq!(
        extract_message_text(&update).as_deref(),
        Some("Segunda opção")
    );
}

#[test]
fn a_button_reply_without_a_title_yields_an_empty_string() {
    let mut update = message("interactive");
    update.interactive = Some(InteractiveContent {
        interactive_type: Some("button_reply".to_string()),
        button_reply: Some(ButtonReplyContent {
            id: Some("btn-1".to_string()),
            title: None,
        }),
        list_reply: None,
    });

    // Some("") — not None: the turn exists, it just has no text, and the
    // caller decides to drop it.
    assert_eq!(extract_message_text(&update).as_deref(), Some(""));
}

#[test]
fn a_quick_reply_button_yields_its_text() {
    let mut update = message("button");
    update.button = Some(ButtonContent {
        payload: Some("yes".to_string()),
        text: Some("Sim".to_string()),
    });

    assert_eq!(extract_message_text(&update).as_deref(), Some("Sim"));
}

#[test]
fn media_and_unknown_types_yield_none() {
    let mut image = message("image");
    image.image = Some(ImageContent {
        id: Some("media-1".to_string()),
        mime_type: Some("image/jpeg".to_string()),
        caption: None,
    });
    // Media is handled by `media::select_media`, never by text extraction.
    assert!(extract_message_text(&image).is_none());

    assert!(extract_message_text(&message("sticker")).is_none());
    assert!(extract_message_text(&message("location")).is_none());
    assert!(extract_message_text(&message("reaction")).is_none());
}

#[test]
fn a_message_without_a_type_yields_none() {
    let mut update = message("text");
    update.message_type = None;
    update.text = Some(TextContent {
        body: Some("ignored".to_string()),
    });

    assert!(extract_message_text(&update).is_none());
}

// --------------------------------------------- WA-5 webhook signature (HMAC)

/// Vectors computed once from the documented algorithm
/// `hex(hmac_sha256("test-app-secret", raw_body))`.
const BODY: &str = r#"{"object":"whatsapp_business_account"}"#;
const BODY_DIGEST: &str = "b6978b21c4467654c466607663db9b43fae44b71083568df403e0a077089208e";
const EMPTY_BODY_DIGEST: &str = "a7dfadfa864dba471c925e814eca5ee032a7ded01574b16ac33dd477b5d6d0f7";
const SECRET: &str = "test-app-secret";

fn signed(body: &str) -> String {
    format!("sha256={}", if body.is_empty() { EMPTY_BODY_DIGEST } else { BODY_DIGEST })
}

#[test]
fn a_published_vector_verifies() {
    assert!(verify_webhook_signature(BODY, &signed(BODY), SECRET));
    // An empty body has its own digest: the signature still binds the request.
    assert!(verify_webhook_signature("", &signed(""), SECRET));
}

#[test]
fn a_tampered_body_fails() {
    let tampered = r#"{"object":"whatsapp_business_account","entry":[]}"#;
    assert!(!verify_webhook_signature(tampered, &signed(BODY), SECRET));
}

#[test]
fn a_different_secret_fails() {
    assert!(!verify_webhook_signature(BODY, &signed(BODY), "other-secret"));
    assert!(!verify_webhook_signature(BODY, &signed(BODY), ""));
}

#[test]
fn a_missing_or_malformed_signature_fails() {
    assert!(!verify_webhook_signature(BODY, "", SECRET));
    assert!(!verify_webhook_signature(BODY, BODY_DIGEST, SECRET));
    assert!(!verify_webhook_signature(BODY, "sha1=b6978b21", SECRET));
    assert!(!verify_webhook_signature(BODY, "sha256=", SECRET));
    assert!(!verify_webhook_signature(BODY, "sha256=zzzz", SECRET));
}

#[test]
fn an_uppercase_digest_is_rejected() {
    let uppercase = format!("sha256={}", BODY_DIGEST.to_uppercase());
    assert!(!verify_webhook_signature(BODY, &uppercase, SECRET));
}

#[test]
fn a_flipped_digest_byte_fails() {
    let mut flipped = BODY_DIGEST.to_string();
    let last = flipped.pop().unwrap_or('0');
    flipped.push(if last == '0' { '1' } else { '0' });
    assert!(!verify_webhook_signature(BODY, &format!("sha256={flipped}"), SECRET));
}

#[test]
fn surrounding_whitespace_in_the_header_is_tolerated() {
    assert!(verify_webhook_signature(BODY, &format!(" {} ", signed(BODY)), SECRET));
}

#[test]
fn constant_time_eq_does_not_short_circuit() {
    assert!(constant_time_eq(b"", b""));
    assert!(constant_time_eq(b"sha256=abc", b"sha256=abc"));
    assert!(!constant_time_eq(b"sha256=abc", b"sha256=abd"));
    assert!(!constant_time_eq(b"sha256=abc", b"sha256=ab"));
    assert!(!constant_time_eq(b"sha256=abc", b""));
}

// ------------------------------------------------------- WA-7 timestamps

#[test]
fn an_epoch_second_timestamp_round_trips() {
    let parsed = format_whatsapp_timestamp("1700000000").expect("valid timestamp");
    assert_eq!(parsed.timestamp(), 1_700_000_000);
    assert_eq!(parsed.timestamp_millis(), 1_700_000_000_000);
}

#[test]
fn zero_is_the_epoch_and_negative_values_go_before_it() {
    assert_eq!(format_whatsapp_timestamp("0").expect("epoch").timestamp(), 0);
    assert_eq!(format_whatsapp_timestamp("-1").expect("-1").timestamp(), -1);
}

#[test]
fn a_non_numeric_timestamp_is_an_error() {
    assert!(format_whatsapp_timestamp("abc").is_err());
    assert!(format_whatsapp_timestamp("").is_err());
    assert!(format_whatsapp_timestamp("17.5").is_err());
}

#[test]
fn an_out_of_range_timestamp_is_an_error() {
    assert!(format_whatsapp_timestamp("99999999999999999999").is_err());
}

#[test]
fn a_millisecond_timestamp_is_read_as_seconds() {
    // Pinned: the field is parsed as epoch seconds, so a millisecond value
    // lands far in the future instead of being divided by 1000.
    let parsed = format_whatsapp_timestamp("1700000000000").expect("in range");
    assert_eq!(parsed.timestamp(), 1_700_000_000_000);
}

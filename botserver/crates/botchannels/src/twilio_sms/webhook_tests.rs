//! Twilio inbound webhook parsing and request-signature verification
//! (CHAN-10, CHAN-12 in #2070).
//!
//! The signature vector is fixed-literal: URL + sorted `key+value` pairs signed
//! with HMAC-SHA1 and base64-encoded, which is what Twilio documents.

use super::*;

const AUTH_TOKEN: &str = "test-auth-token";
const URL: &str = "https://example.com/webhook";
/// `base64(hmac_sha1("test-auth-token", URL + "BodyhelloCallSidCA1234567890From+15551234567To+15559876543"))`
const SIGNATURE: &str = "Q76ka7WPHBCXCqM9ZLIHzTz7HjY=";

fn channel() -> TwilioSmsChannel {
    TwilioSmsChannel::new(TwilioConfig {
        account_sid: "AC-test".to_string(),
        auth_token: AUTH_TOKEN.to_string(),
        from_number: "+15550000001".to_string(),
        messaging_service_sid: None,
        status_callback_url: None,
    })
}

fn params(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    pairs
        .iter()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect()
}

fn signature_params() -> HashMap<String, String> {
    params(&[
        ("CallSid", "CA1234567890"),
        ("From", "+15551234567"),
        ("To", "+15559876543"),
        ("Body", "hello"),
    ])
}

fn full_webhook() -> HashMap<String, String> {
    params(&[
        ("MessageSid", "SM-1"),
        ("SmsSid", "SM-1"),
        ("AccountSid", "AC-test"),
        ("From", "+15551234567"),
        ("To", "+15550000001"),
        ("Body", "bom dia"),
        ("NumMedia", "2"),
        ("NumSegments", "1"),
        ("SmsStatus", "received"),
        ("ApiVersion", "2010-04-01"),
        ("FromCity", "SAO PAULO"),
        ("FromState", "SP"),
        ("FromZip", "01000-000"),
        ("FromCountry", "BR"),
        ("ToCity", "RIO DE JANEIRO"),
        ("ToState", "RJ"),
        ("ToZip", "20000-000"),
        ("ToCountry", "BR"),
        ("MediaUrl0", "https://api.twilio.com/media/0"),
        ("MediaUrl1", "https://api.twilio.com/media/1"),
    ])
}

// ------------------------------------------------------------ CHAN-10 parsing

#[test]
fn a_recorded_webhook_maps_every_scalar() {
    let webhook = channel()
        .parse_incoming_webhook(&full_webhook())
        .expect("webhook");

    assert_eq!(webhook.message_sid, "SM-1");
    assert_eq!(webhook.sms_sid.as_deref(), Some("SM-1"));
    assert_eq!(webhook.account_sid, "AC-test");
    assert_eq!(webhook.from, "+15551234567");
    assert_eq!(webhook.to, "+15550000001");
    assert_eq!(webhook.body, "bom dia");
    assert_eq!(webhook.num_media, Some(2));
    assert_eq!(webhook.num_segments, Some(1));
    assert_eq!(webhook.sms_status.as_deref(), Some("received"));
    assert_eq!(webhook.api_version.as_deref(), Some("2010-04-01"));
    assert_eq!(webhook.from_city.as_deref(), Some("SAO PAULO"));
    assert_eq!(webhook.from_state.as_deref(), Some("SP"));
    assert_eq!(webhook.from_zip.as_deref(), Some("01000-000"));
    assert_eq!(webhook.from_country.as_deref(), Some("BR"));
    assert_eq!(webhook.to_city.as_deref(), Some("RIO DE JANEIRO"));
    assert_eq!(webhook.to_state.as_deref(), Some("RJ"));
    assert_eq!(webhook.to_zip.as_deref(), Some("20000-000"));
    assert_eq!(webhook.to_country.as_deref(), Some("BR"));
    assert_eq!(
        webhook.media_urls,
        vec![
            "https://api.twilio.com/media/0".to_string(),
            "https://api.twilio.com/media/1".to_string()
        ]
    );
}

#[test]
fn a_webhook_without_media_reports_no_urls() {
    let webhook = channel()
        .parse_incoming_webhook(&params(&[
            ("MessageSid", "SM-1"),
            ("AccountSid", "AC-test"),
            ("From", "+15551234567"),
            ("To", "+15550000001"),
            ("Body", "hi"),
            ("NumMedia", "0"),
        ]))
        .expect("webhook");

    assert_eq!(webhook.num_media, Some(0));
    assert!(webhook.media_urls.is_empty());
}

#[test]
fn a_num_media_larger_than_the_urls_yields_only_the_urls_present() {
    let webhook = channel()
        .parse_incoming_webhook(&params(&[
            ("MessageSid", "SM-1"),
            ("AccountSid", "AC-test"),
            ("From", "+15551234567"),
            ("To", "+15550000001"),
            ("NumMedia", "3"),
            ("MediaUrl0", "https://api.twilio.com/media/0"),
        ]))
        .expect("webhook");

    assert_eq!(webhook.num_media, Some(3));
    assert_eq!(webhook.media_urls.len(), 1);
}

#[test]
fn a_non_numeric_num_media_is_none_not_an_error() {
    let webhook = channel()
        .parse_incoming_webhook(&params(&[
            ("MessageSid", "SM-1"),
            ("AccountSid", "AC-test"),
            ("From", "+15551234567"),
            ("To", "+15550000001"),
            ("NumMedia", "many"),
        ]))
        .expect("webhook");

    assert!(webhook.num_media.is_none());
    assert!(webhook.media_urls.is_empty());
}

#[test]
fn an_absent_body_is_an_empty_string() {
    let webhook = channel()
        .parse_incoming_webhook(&params(&[
            ("MessageSid", "SM-1"),
            ("AccountSid", "AC-test"),
            ("From", "+15551234567"),
            ("To", "+15550000001"),
        ]))
        .expect("webhook");

    // A media-only MMS has no Body: an empty string, never a parse failure.
    assert_eq!(webhook.body, "");
}

#[test]
fn each_missing_required_field_is_named_in_the_error() {
    for (missing, expected) in [
        ("MessageSid", "Missing MessageSid"),
        ("AccountSid", "Missing AccountSid"),
        ("From", "Missing From"),
        ("To", "Missing To"),
    ] {
        let mut map = full_webhook();
        map.remove(missing);

        match channel().parse_incoming_webhook(&map) {
            Err(TwilioError::ParseError(message)) => assert_eq!(message, expected),
            other => panic!("{missing} missing must be a ParseError, got {other:?}"),
        }
    }
}

// -------------------------------------------------- CHAN-12 signature

#[test]
fn the_documented_signing_input_is_url_then_sorted_pairs() {
    let input = TwilioSmsChannel::webhook_signing_input(URL, &signature_params());

    assert_eq!(
        input,
        "https://example.com/webhookBodyhelloCallSidCA1234567890From+15551234567To+15559876543"
    );
}

#[test]
fn a_published_vector_verifies() {
    assert!(channel().validate_webhook_signature(SIGNATURE, URL, &signature_params()));
}

#[test]
fn a_tampered_body_fails() {
    let mut tampered = signature_params();
    tampered.insert("Body".to_string(), "HELLO".to_string());

    assert!(!channel().validate_webhook_signature(SIGNATURE, URL, &tampered));
}

#[test]
fn a_changed_value_or_extra_parameter_fails() {
    let mut changed = signature_params();
    changed.insert("From".to_string(), "+15559999999".to_string());
    assert!(!channel().validate_webhook_signature(SIGNATURE, URL, &changed));

    let mut extra = signature_params();
    extra.insert("Injected".to_string(), "1".to_string());
    assert!(!channel().validate_webhook_signature(SIGNATURE, URL, &extra));
}

#[test]
fn a_changed_url_fails() {
    assert!(!channel().validate_webhook_signature(
        SIGNATURE,
        "https://example.com/other",
        &signature_params()
    ));
}

#[test]
fn an_empty_signature_fails() {
    assert!(!channel().validate_webhook_signature("", URL, &signature_params()));
}

#[test]
fn a_different_auth_token_fails() {
    let other = TwilioSmsChannel::new(TwilioConfig {
        account_sid: "AC-test".to_string(),
        auth_token: "other-token".to_string(),
        from_number: "+15550000001".to_string(),
        messaging_service_sid: None,
        status_callback_url: None,
    });

    assert!(!other.validate_webhook_signature(SIGNATURE, URL, &signature_params()));
}

#[test]
fn surrounding_whitespace_in_the_header_is_tolerated() {
    let padded = format!(" {SIGNATURE} ");

    assert!(channel().validate_webhook_signature(&padded, URL, &signature_params()));
}

#[test]
fn a_flipped_base64_character_fails() {
    let mut flipped: Vec<char> = SIGNATURE.chars().collect();
    let last = flipped.len() - 2;
    flipped[last] = if flipped[last] == 'A' { 'B' } else { 'A' };
    let flipped: String = flipped.into_iter().collect();

    assert!(!channel().validate_webhook_signature(&flipped, URL, &signature_params()));
}

#[test]
fn constant_time_eq_does_not_short_circuit() {
    assert!(constant_time_eq(b"", b""));
    assert!(constant_time_eq(b"Q76ka7WPHBCXCqM9ZLIHzTz7HjY=", SIGNATURE.as_bytes()));
    assert!(!constant_time_eq(b"Q76ka7WPHBCXCqM9ZLIHzTz7HjY=", b"Q76ka7WPHBCXCqM9ZLIHzTz7HjZ="));
    assert!(!constant_time_eq(b"Q76ka7WPHBCXCqM9ZLIHzTz7HjY=", b"Q76ka7WPHBCXCqM9ZLIHzTz7HjY"));
    assert!(!constant_time_eq(b"abc", b""));
}

//! Twilio delivery-status state machine and TwiML serialization
//! (CHAN-11 in #2070).
//!
//! Status callbacks arrive out of order and are redelivered, so the parser must
//! map every documented status and the caller must compare the enum — never the
//! raw string.

use super::*;

fn channel() -> TwilioSmsChannel {
    TwilioSmsChannel::new(TwilioConfig {
        account_sid: "AC-test".to_string(),
        auth_token: "test-auth-token".to_string(),
        from_number: "+15550000001".to_string(),
        messaging_service_sid: None,
        status_callback_url: None,
    })
}

#[test]
fn every_documented_status_parses_case_insensitively() {
    for (wire, expected) in [
        ("queued", MessageStatus::Queued),
        ("sending", MessageStatus::Sending),
        ("sent", MessageStatus::Sent),
        ("delivered", MessageStatus::Delivered),
        ("undelivered", MessageStatus::Undelivered),
        ("failed", MessageStatus::Failed),
        ("receiving", MessageStatus::Receiving),
        ("received", MessageStatus::Received),
        ("accepted", MessageStatus::Accepted),
        ("scheduled", MessageStatus::Scheduled),
        ("read", MessageStatus::Read),
        ("partially_delivered", MessageStatus::PartiallyDelivered),
        ("canceled", MessageStatus::Canceled),
        ("DELIVERED", MessageStatus::Delivered),
        // Twilio sends `Partially_Delivered` in callbacks.
        ("Partially_Delivered", MessageStatus::PartiallyDelivered),
    ] {
        assert_eq!(parse_message_status(wire), Ok(expected), "status {wire:?}");
    }
}

#[test]
fn an_unknown_status_is_a_parse_error_naming_the_input() {
    match parse_message_status("bogus") {
        Err(TwilioError::ParseError(message)) => assert_eq!(message, "Unknown status: bogus"),
        other => panic!("expected ParseError, got {other:?}"),
    }
}

#[test]
fn the_four_directions_map_and_unknown_defaults_to_outbound_api() {
    assert_eq!(parse_direction("inbound"), MessageDirection::Inbound);
    assert_eq!(parse_direction("outbound-api"), MessageDirection::OutboundApi);
    assert_eq!(parse_direction("outbound-call"), MessageDirection::OutboundCall);
    assert_eq!(
        parse_direction("outbound-reply"),
        MessageDirection::OutboundReply
    );
    // Pinned: an unrecognized direction is treated as an API send rather than
    // an error, so the message is never dropped.
    assert_eq!(parse_direction("carrier-pigeon"), MessageDirection::OutboundApi);
    assert_eq!(parse_direction(""), MessageDirection::OutboundApi);
}

#[test]
fn the_documented_delivery_progression_round_trips() {
    let progression = [
        MessageStatus::Accepted,
        MessageStatus::Scheduled,
        MessageStatus::Queued,
        MessageStatus::Sending,
        MessageStatus::Sent,
        MessageStatus::Delivered,
    ];

    for status in progression {
        let wire = serde_json::to_string(&status).expect("serialize");
        let parsed = parse_message_status(wire.trim_matches('"')).expect("parse");
        assert_eq!(parsed, status);
    }

    // Terminal failures are distinct from a successful delivery: the UI keys
    // the "retry" action off exactly these two.
    for terminal in [MessageStatus::Failed, MessageStatus::Undelivered] {
        let wire = serde_json::to_string(&terminal).expect("serialize");
        let parsed = parse_message_status(wire.trim_matches('"')).expect("parse");
        assert_eq!(parsed, terminal);
        assert_ne!(parsed, MessageStatus::Delivered);
    }
}

#[test]
fn a_status_callback_parses_the_delivery_outcome() {
    let sms = channel();
    let mut params = HashMap::new();
    params.insert("MessageSid".to_string(), "SM-1".to_string());
    params.insert("AccountSid".to_string(), "AC-test".to_string());
    params.insert("MessageStatus".to_string(), "delivered".to_string());
    params.insert("From".to_string(), "+15550000001".to_string());
    params.insert("To".to_string(), "+15551234567".to_string());

    let callback = sms.parse_status_callback(&params).expect("callback");

    assert_eq!(callback.message_sid, "SM-1");
    assert_eq!(callback.message_status, MessageStatus::Delivered);
    assert_eq!(callback.from, "+15550000001");
    assert_eq!(callback.to, "+15551234567");
    assert!(callback.error_code.is_none());
}

#[test]
fn a_failed_callback_carries_the_error_code_and_message() {
    let sms = channel();
    let mut params = HashMap::new();
    params.insert("MessageSid".to_string(), "SM-2".to_string());
    params.insert("AccountSid".to_string(), "AC-test".to_string());
    params.insert("MessageStatus".to_string(), "failed".to_string());
    params.insert("ErrorCode".to_string(), "30003".to_string());
    params.insert("ErrorMessage".to_string(), "Unreachable".to_string());

    let callback = sms.parse_status_callback(&params).expect("callback");

    assert_eq!(callback.message_status, MessageStatus::Failed);
    assert_eq!(callback.error_code, Some(30003));
    assert_eq!(callback.error_message.as_deref(), Some("Unreachable"));
    // From/To are optional on a status callback and default to empty.
    assert_eq!(callback.from, "");
}

#[test]
fn a_callback_without_a_status_is_rejected() {
    let sms = channel();
    let mut params = HashMap::new();
    params.insert("MessageSid".to_string(), "SM-3".to_string());
    params.insert("AccountSid".to_string(), "AC-test".to_string());

    assert!(matches!(
        sms.parse_status_callback(&params),
        Err(TwilioError::ParseError(message)) if message == "Missing MessageStatus"
    ));
}

#[test]
fn escape_xml_escapes_each_of_the_five_characters_once() {
    assert_eq!(
        escape_xml(r#"<a href="x">&'</a>"#),
        "&lt;a href=&quot;x&quot;&gt;&amp;&apos;&lt;/a&gt;"
    );
    // `&` is escaped exactly once, which is what TwiML expects.
    assert_eq!(escape_xml("&amp;"), "&amp;amp;");
    assert_eq!(escape_xml("plain"), "plain");
}

#[test]
fn an_empty_twiml_response_is_an_empty_response_element() {
    let twiml = channel().generate_twiml_response(None, None);

    assert!(twiml.contains("<Response>"), "twiml: {twiml}");
    assert!(twiml.contains("</Response>"), "twiml: {twiml}");
    assert!(!twiml.contains("<Message>"), "twiml: {twiml}");
}

#[test]
fn a_twiml_message_is_xml_escaped() {
    let twiml = channel().generate_twiml_response(Some("5 < 6 & \"ok\""), None);

    assert!(
        twiml.contains("<Message>5 &lt; 6 &amp; &quot;ok&quot;</Message>"),
        "twiml: {twiml}"
    );
    assert!(!twiml.contains("<Media>"), "twiml: {twiml}");
}

#[test]
fn a_twiml_message_with_media_carries_both() {
    let twiml = channel().generate_twiml_response(Some("foto"), Some("https://cdn/a.jpg?x=1&y=2"));

    assert!(twiml.contains("<Body>foto</Body>"), "twiml: {twiml}");
    assert!(
        twiml.contains("<Media>https://cdn/a.jpg?x=1&amp;y=2</Media>"),
        "twiml: {twiml}"
    );
}

#[test]
fn media_without_a_message_is_ignored() {
    let twiml = channel().generate_twiml_response(None, Some("https://cdn.a/a.jpg"));

    // Pinned: TwiML cannot send media without a message body, so the media is
    // dropped rather than emitting an invalid `<Media>`-only response.
    assert!(!twiml.contains("<Media>"), "twiml: {twiml}");
}

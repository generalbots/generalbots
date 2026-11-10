//! Twilio SMS delivery: form-body builder and delivery-status state machine
//! (CHAN-11, CHAN-13 in #2070).

use super::*;

fn config() -> TwilioConfig {
    TwilioConfig {
        account_sid: "AC-test".to_string(),
        auth_token: "test-auth-token".to_string(),
        from_number: "+15550000001".to_string(),
        messaging_service_sid: None,
        status_callback_url: None,
    }
}

fn channel(config: TwilioConfig) -> TwilioSmsChannel {
    TwilioSmsChannel::new(config)
}

fn request(to: &str, body: &str) -> SendSmsRequest {
    SendSmsRequest {
        to: to.to_string(),
        body: body.to_string(),
        media_url: None,
        status_callback: None,
        max_price: None,
        validity_period: None,
        schedule_type: None,
        send_at: None,
    }
}

#[test]
fn a_plain_sms_carries_to_body_and_the_configured_sender() {
    let sms = channel(config());
    let params = sms.build_send_sms_params(&request("+15551234567", "hello"));

    assert_eq!(params.get("To"), Some(&"+15551234567".to_string()));
    assert_eq!(params.get("Body"), Some(&"hello".to_string()));
    assert_eq!(params.get("From"), Some(&"+15550000001".to_string()));
    // From and MessagingServiceSid are mutually exclusive.
    assert!(!params.contains_key("MessagingServiceSid"));
    assert!(!params.contains_key("StatusCallback"));
    assert!(!params.contains_key("MediaUrl0"));
}

#[test]
fn a_messaging_service_replaces_the_from_number() {
    let mut cfg = config();
    cfg.messaging_service_sid = Some("MG-test".to_string());
    let sms = channel(cfg);

    let params = sms.build_send_sms_params(&request("+15551234567", "hello"));

    assert_eq!(params.get("MessagingServiceSid"), Some(&"MG-test".to_string()));
    assert!(!params.contains_key("From"));
}

#[test]
fn media_urls_are_numbered_from_zero() {
    let sms = channel(config());
    let mut sms_request = request("+15551234567", "two pictures");
    sms_request.media_url = Some(vec![
        "https://cdn/a.jpg".to_string(),
        "https://cdn/b.jpg".to_string(),
    ]);

    let params = sms.build_send_sms_params(&sms_request);

    assert_eq!(params.get("MediaUrl0"), Some(&"https://cdn/a.jpg".to_string()));
    assert_eq!(params.get("MediaUrl1"), Some(&"https://cdn/b.jpg".to_string()));
    assert!(!params.contains_key("MediaUrl2"));
}

#[test]
fn the_request_callback_wins_over_the_configured_one() {
    let mut cfg = config();
    cfg.status_callback_url = Some("https://configured/cb".to_string());

    let sms = channel(cfg);
    let configured_only = sms.build_send_sms_params(&request("+15551234567", "hi"));
    assert_eq!(
        configured_only.get("StatusCallback"),
        Some(&"https://configured/cb".to_string())
    );

    let mut sms_request = request("+15551234567", "hi");
    sms_request.status_callback = Some("https://per-request/cb".to_string());
    let per_request = sms.build_send_sms_params(&sms_request);
    assert_eq!(
        per_request.get("StatusCallback"),
        Some(&"https://per-request/cb".to_string())
    );
}

#[test]
fn the_optional_delivery_controls_are_forwarded() {
    let sms = channel(config());
    let mut sms_request = request("+15551234567", "later");
    sms_request.max_price = Some("0.05".to_string());
    sms_request.validity_period = Some(3600);
    sms_request.schedule_type = Some(ScheduleType::Fixed);
    sms_request.send_at = chrono::DateTime::from_timestamp(1_700_000_000, 0);

    let params = sms.build_send_sms_params(&sms_request);

    assert_eq!(params.get("MaxPrice"), Some(&"0.05".to_string()));
    assert_eq!(params.get("ValidityPeriod"), Some(&"3600".to_string()));
    assert_eq!(params.get("ScheduleType"), Some(&"fixed".to_string()));
    assert_eq!(
        params.get("SendAt"),
        Some(&"2023-11-14T22:13:20+00:00".to_string())
    );
}

#[test]
fn a_schedule_type_without_a_time_omits_send_at() {
    let sms = channel(config());
    let mut sms_request = request("+15551234567", "later");
    sms_request.schedule_type = Some(ScheduleType::Fixed);

    let params = sms.build_send_sms_params(&sms_request);

    assert_eq!(params.get("ScheduleType"), Some(&"fixed".to_string()));
    assert!(!params.contains_key("SendAt"));
}

#[test]
fn an_empty_body_is_still_sent() {
    let sms = channel(config());
    let params = sms.build_send_sms_params(&request("+15551234567", ""));

    assert_eq!(params.get("Body"), Some(&String::new()));
}

//! Contract tests for the SMS provider/priority normalisation (#2064).
//!
//! Both enums are parsed from configuration strings. An unknown priority must
//! not escalate, and an unknown provider must stay addressable as `Custom`.

use super::{SmsPriority, SmsProvider};

#[test]
fn sms_priorities_are_parsed_and_rendered() {
    assert_eq!(SmsPriority::from("low"), SmsPriority::Low);
    assert_eq!(SmsPriority::from("HIGH"), SmsPriority::High);
    assert_eq!(SmsPriority::from("urgent"), SmsPriority::Urgent);
    assert_eq!(SmsPriority::from("critical"), SmsPriority::Urgent);
    assert_eq!(SmsPriority::from("normal"), SmsPriority::Normal);
    // Unknown values must not escalate: the fallback is `Normal`.
    assert_eq!(SmsPriority::from("emergency"), SmsPriority::Normal);
    assert_eq!(SmsPriority::from(""), SmsPriority::Normal);

    let rendered: Vec<String> = [
        SmsPriority::Low,
        SmsPriority::Normal,
        SmsPriority::High,
        SmsPriority::Urgent,
    ]
    .iter()
    .map(|priority| priority.to_string())
    .collect();
    assert_eq!(rendered, ["low", "normal", "high", "urgent"]);
}

#[test]
fn sms_providers_keep_unknown_names_as_custom() {
    assert_eq!(SmsProvider::from("twilio"), SmsProvider::Twilio);
    assert_eq!(SmsProvider::from("AWS"), SmsProvider::AwsSns);
    assert_eq!(SmsProvider::from("sns"), SmsProvider::AwsSns);
    assert_eq!(SmsProvider::from("aws_sns"), SmsProvider::AwsSns);
    assert_eq!(SmsProvider::from("nexmo"), SmsProvider::Vonage);
    assert_eq!(SmsProvider::from("vonage"), SmsProvider::Vonage);
    assert_eq!(SmsProvider::from("messagebird"), SmsProvider::MessageBird);

    // Anything else is preserved so a custom gateway stays addressable.
    assert_eq!(
        SmsProvider::from("Zenith SMS"),
        SmsProvider::Custom("zenith sms".to_string())
    );
}

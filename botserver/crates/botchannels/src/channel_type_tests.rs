//! `ChannelType` parsing and the `ChannelError` strings the UI matches
//! (CHAN-1, CHAN-2 in #2070).

use super::*;
use std::str::FromStr;

const ALL_CHANNELS: [ChannelType; 16] = [
    ChannelType::Bluesky,
    ChannelType::Discord,
    ChannelType::Facebook,
    ChannelType::Instagram,
    ChannelType::LinkedIn,
    ChannelType::Pinterest,
    ChannelType::Reddit,
    ChannelType::Snapchat,
    ChannelType::Telegram,
    ChannelType::Threads,
    ChannelType::TikTok,
    ChannelType::TwilioSms,
    ChannelType::Twitter,
    ChannelType::WeChat,
    ChannelType::WhatsApp,
    ChannelType::YouTube,
];

#[test]
fn every_channel_round_trips_through_its_wire_name() {
    for channel in ALL_CHANNELS {
        let wire = channel.to_string();
        assert_eq!(
            ChannelType::from_str(&wire),
            Ok(channel.clone()),
            "{channel:?} did not round-trip through {wire:?}"
        );
    }
}

#[test]
fn the_wire_names_are_the_documented_ones() {
    assert_eq!(ChannelType::TwilioSms.to_string(), "twilio_sms");
    assert_eq!(ChannelType::TikTok.to_string(), "tiktok");
    assert_eq!(ChannelType::WeChat.to_string(), "wechat");
    assert_eq!(ChannelType::YouTube.to_string(), "youtube");
}

#[test]
fn the_short_aliases_are_accepted() {
    assert_eq!(ChannelType::from_str("fb"), Ok(ChannelType::Facebook));
    assert_eq!(ChannelType::from_str("ig"), Ok(ChannelType::Instagram));
    assert_eq!(ChannelType::from_str("tg"), Ok(ChannelType::Telegram));
    assert_eq!(ChannelType::from_str("twilio"), Ok(ChannelType::TwilioSms));
    assert_eq!(ChannelType::from_str("sms"), Ok(ChannelType::TwilioSms));
    assert_eq!(ChannelType::from_str("x"), Ok(ChannelType::Twitter));
    assert_eq!(ChannelType::from_str("wa"), Ok(ChannelType::WhatsApp));
    assert_eq!(ChannelType::from_str("yt"), Ok(ChannelType::YouTube));
}

#[test]
fn parsing_is_case_insensitive() {
    assert_eq!(ChannelType::from_str("TeLeGrAm"), Ok(ChannelType::Telegram));
    assert_eq!(ChannelType::from_str("INSTAGRAM"), Ok(ChannelType::Instagram));
}

#[test]
fn an_unknown_channel_is_reported_with_the_original_input() {
    for input in ["", "myspace", "twitter "] {
        match ChannelType::from_str(input) {
            Err(ChannelError::UnknownChannel(name)) => assert_eq!(name, input),
            other => panic!("{input:?} must be UnknownChannel, got {other:?}"),
        }
    }
}

#[test]
fn the_serde_representation_matches_the_wire_names() {
    let json = serde_json::to_string(&ChannelType::TwilioSms).expect("serialize");
    assert_eq!(json, "\"twilio_sms\"");

    let parsed: ChannelType = serde_json::from_str("\"instagram\"").expect("deserialize");
    assert_eq!(parsed, ChannelType::Instagram);
}

#[test]
fn error_messages_are_the_documented_strings() {
    assert_eq!(
        ChannelError::RateLimited {
            retry_after: Some(30)
        }
        .to_string(),
        "Rate limited, retry after 30 seconds"
    );
    assert_eq!(
        ChannelError::RateLimited { retry_after: None }.to_string(),
        "Rate limited"
    );
    assert_eq!(
        ChannelError::ContentTooLong {
            max_length: 280,
            actual_length: 300
        }
        .to_string(),
        "Content too long: 300 characters (max: 280)"
    );
    assert_eq!(
        ChannelError::ApiError {
            code: Some("429".to_string()),
            message: "slow down".to_string()
        }
        .to_string(),
        "API error [429]: slow down"
    );
    assert_eq!(
        ChannelError::ApiError {
            code: None,
            message: "boom".to_string()
        }
        .to_string(),
        "API error: boom"
    );
    assert_eq!(ChannelError::NotConfigured.to_string(), "Channel not configured");
    assert_eq!(
        ChannelError::AccountNotFound("marketing".to_string()).to_string(),
        "Account not found: marketing"
    );
    assert_eq!(
        ChannelError::AuthenticationFailed("bad token".to_string()).to_string(),
        "Authentication failed: bad token"
    );
    assert_eq!(
        ChannelError::UnsupportedMediaType("image/svg+xml".to_string()).to_string(),
        "Unsupported media type: image/svg+xml"
    );
    assert_eq!(
        ChannelError::NetworkError("timeout".to_string()).to_string(),
        "Network error: timeout"
    );
    assert_eq!(
        ChannelError::UnknownChannel("myspace".to_string()).to_string(),
        "Unknown channel: myspace"
    );
}

#[test]
fn the_error_implements_std_error() {
    fn assert_error<E: std::error::Error>(_: &E) {}
    assert_error(&ChannelError::NotConfigured);
}

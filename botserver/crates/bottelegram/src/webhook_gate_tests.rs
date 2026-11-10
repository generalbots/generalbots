//! Telegram inbound authenticity (#2142).
//!
//! The gate is the only thing between an anonymous POST and a bot session, so
//! the tests pin the four outcomes: a matching secret is accepted, a missing or
//! wrong one is refused, and an **unconfigured** secret refuses every delivery
//! instead of accepting them (the pre-#2142 behaviour).

use super::*;
use uuid::Uuid;

#[test]
fn a_matching_secret_is_accepted() {
    assert_eq!(
        authorize_delivery("s3cret", Some("s3cret")),
        DeliveryAuth::Verified
    );
    // Both sides are trimmed: an operator may paste the value with spaces and
    // the Bot API forwards the header verbatim.
    assert_eq!(
        authorize_delivery(" s3cret ", Some("s3cret")),
        DeliveryAuth::Verified
    );
    assert_eq!(
        authorize_delivery("s3cret", Some("s3cret ")),
        DeliveryAuth::Verified
    );
}

#[test]
fn a_missing_header_is_refused() {
    assert_eq!(authorize_delivery("s3cret", None), DeliveryAuth::Mismatch);
    assert_eq!(
        authorize_delivery("s3cret", Some("")),
        DeliveryAuth::Mismatch
    );
    assert_eq!(
        authorize_delivery("s3cret", Some("   ")),
        DeliveryAuth::Mismatch
    );
}

#[test]
fn a_wrong_or_truncated_secret_is_refused() {
    for provided in ["s3cre", "s3cret extra", "S3CRET", "s3cret\n"] {
        assert_eq!(
            authorize_delivery("s3cret", Some(provided)),
            DeliveryAuth::Mismatch,
            "{provided:?} must not authenticate a delivery"
        );
    }
}

#[test]
fn an_unconfigured_secret_fails_closed() {
    // Every shape of "no secret": absent, empty, whitespace-only. Any of them
    // used to return `true`, so a fresh bot accepted forged deliveries.
    for expected in ["", "   ", "\n"] {
        assert_eq!(
            authorize_delivery(expected, Some("anything")),
            DeliveryAuth::Unconfigured
        );
        assert_eq!(
            authorize_delivery(expected, None),
            DeliveryAuth::Unconfigured
        );
        assert_eq!(
            authorize_delivery(expected, Some("")),
            DeliveryAuth::Unconfigured
        );
    }
}

#[test]
fn the_header_name_matches_the_bot_api() {
    assert_eq!(SECRET_HEADER, "x-telegram-bot-api-secret-token");
    assert_eq!(SECRET_CONFIG_KEY, "telegram-webhook-secret");
}

#[test]
fn the_configuration_error_is_logged_once_per_bot() {
    let bot = Uuid::new_v4();
    assert!(
        warn_unconfigured_once(bot),
        "the first refusal names the missing secret"
    );
    assert!(
        !warn_unconfigured_once(bot),
        "a forged POST must not be able to flood the log one line per delivery"
    );

    // Another bot is a different configuration error and is reported.
    assert!(warn_unconfigured_once(Uuid::new_v4()));
}

#[test]
fn the_gate_is_constant_time_for_equal_length_inputs() {
    // Content, not identity: the comparison walks every byte, so a difference
    // in the last position is as rejected as one in the first.
    assert_eq!(
        authorize_delivery("abcdef", Some("abcdeg")),
        DeliveryAuth::Mismatch
    );
    assert_eq!(
        authorize_delivery("abcdef", Some("zbcdef")),
        DeliveryAuth::Mismatch
    );
}

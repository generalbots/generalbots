//! Tests for signed click destinations (#2139).
//!
//! The signing key is passed explicitly so the suite is deterministic and does
//! not depend on process environment or on `directory_config.json`.

use super::{
    build_click_url, evaluate, sign, validate_destination, ClickError, ClickOutcome, ClickQuery,
    TrackingRecord,
};
use url::form_urlencoded;
use uuid::Uuid;

const KEY: &str = "test-signing-key";
const NOW: i64 = 1_700_000_000;
const LATER: i64 = NOW + 3_600;

fn record() -> TrackingRecord {
    TrackingRecord {
        id: Uuid::parse_str("6f1f6f9a-0d1a-4c02-9a53-2f7f4b0c1d11").expect("tracking id"),
        campaign_id: Some(
            Uuid::parse_str("9a1c2b3d-4e5f-4a6b-8c7d-9e0f1a2b3c4d").expect("campaign id"),
        ),
        recipient_id: Some(
            Uuid::parse_str("0b1c2d3e-4f5a-6b7c-8d9e-0f1a2b3c4d5e").expect("recipient id"),
        ),
    }
}

fn other_recipient() -> TrackingRecord {
    TrackingRecord {
        recipient_id: Some(
            Uuid::parse_str("aaaaaaaa-bbbb-4ccc-8ddd-eeeeeeeeeeee").expect("recipient id"),
        ),
        ..record()
    }
}

fn parse_query(url: &str) -> ClickQuery {
    let raw = url.split_once('?').map(|(_, q)| q).unwrap_or_default();
    let mut query = ClickQuery::default();
    for (key, value) in form_urlencoded::parse(raw.as_bytes()) {
        match key.as_ref() {
            "u" => query.u = Some(value.into_owned()),
            "e" => query.e = Some(value.into_owned()),
            "s" => query.s = Some(value.into_owned()),
            _ => {}
        }
    }
    query
}

fn issued(record: &TrackingRecord, destination: &str, expires_at: i64) -> ClickQuery {
    let url = build_click_url("https://bot.example", record, destination, expires_at, KEY)
        .expect("a safe destination is always signable");
    parse_query(&url)
}

fn signed_query(destination: &str, expires_at: i64) -> ClickQuery {
    ClickQuery {
        u: Some(destination.to_string()),
        e: Some(expires_at.to_string()),
        s: Some(sign(KEY, &record(), destination, expires_at)),
    }
}

#[test]
fn a_valid_signed_link_redirects_to_its_destination() {
    let query = issued(&record(), "https://shop.example/promo?a=1", LATER);
    assert_eq!(
        evaluate(&query, Some(&record()), NOW, KEY),
        ClickOutcome::Redirect("https://shop.example/promo?a=1".to_string())
    );
}

#[test]
fn a_valid_signed_relative_link_redirects() {
    let query = issued(&record(), "/pricing", LATER);
    assert_eq!(
        evaluate(&query, Some(&record()), NOW, KEY),
        ClickOutcome::Redirect("/pricing".to_string())
    );
}

#[test]
fn a_tampered_destination_is_rejected() {
    let mut query = issued(&record(), "https://shop.example/promo", LATER);
    query.u = Some("https://evil.example/login".to_string());
    assert_eq!(
        evaluate(&query, Some(&record()), NOW, KEY),
        ClickOutcome::BadRequest(ClickError::BadSignature.reason())
    );
}

#[test]
fn an_unsigned_destination_is_rejected() {
    let query = ClickQuery {
        u: Some("https://evil.example/login".to_string()),
        e: Some(LATER.to_string()),
        s: None,
    };
    assert_eq!(
        evaluate(&query, Some(&record()), NOW, KEY),
        ClickOutcome::BadRequest(ClickError::MissingSignature.reason())
    );

    let empty_signature = ClickQuery {
        s: Some(String::new()),
        ..query
    };
    assert_eq!(
        evaluate(&empty_signature, Some(&record()), NOW, KEY),
        ClickOutcome::BadRequest(ClickError::MissingSignature.reason())
    );
}

#[test]
fn an_expired_link_is_rejected() {
    let query = issued(&record(), "https://shop.example/promo", NOW - 1);
    assert_eq!(
        evaluate(&query, Some(&record()), NOW, KEY),
        ClickOutcome::BadRequest(ClickError::Expired.reason())
    );
}

#[test]
fn a_protocol_relative_destination_is_rejected_even_when_signed() {
    let query = signed_query("//evil.example/login", LATER);
    assert_eq!(
        evaluate(&query, Some(&record()), NOW, KEY),
        ClickOutcome::BadRequest(ClickError::UnsafeDestination.reason())
    );
}

#[test]
fn a_javascript_destination_is_rejected_even_when_signed() {
    for destination in [
        "javascript:alert(document.cookie)",
        "data:text/html,<script>alert(1)</script>",
        "file:///etc/passwd",
        "vbscript:msgbox(1)",
    ] {
        let query = signed_query(destination, LATER);
        assert_eq!(
            evaluate(&query, Some(&record()), NOW, KEY),
            ClickOutcome::BadRequest(ClickError::UnsafeDestination.reason()),
            "{destination:?}"
        );
    }
}

#[test]
fn header_injection_shapes_are_rejected() {
    for destination in [
        "/ok\r\nSet-Cookie: a=b",
        "/ok\nX-Injected: 1",
        "/ok\tvalue",
        "\\\\evil.example/login",
        "/\\evil.example",
    ] {
        assert_eq!(
            validate_destination(destination),
            Err(ClickError::UnsafeDestination),
            "{destination:?}"
        );
    }
}

#[test]
fn an_unknown_tracking_id_never_redirects() {
    let query = issued(&record(), "https://shop.example/promo", LATER);
    assert_eq!(evaluate(&query, None, NOW, KEY), ClickOutcome::NotFound);
}

#[test]
fn a_signature_for_another_recipient_is_rejected() {
    let query = issued(&record(), "https://shop.example/promo", LATER);
    assert_eq!(
        evaluate(&query, Some(&other_recipient()), NOW, KEY),
        ClickOutcome::BadRequest(ClickError::BadSignature.reason())
    );
}

#[test]
fn a_signature_from_another_key_is_rejected() {
    let query = issued(&record(), "https://shop.example/promo", LATER);
    assert_eq!(
        evaluate(&query, Some(&record()), NOW, "a-different-key"),
        ClickOutcome::BadRequest(ClickError::BadSignature.reason())
    );
}

#[test]
fn a_malformed_expiry_is_rejected() {
    let mut query = issued(&record(), "/pricing", LATER);
    query.e = Some("soon".to_string());
    assert_eq!(
        evaluate(&query, Some(&record()), NOW, KEY),
        ClickOutcome::BadRequest(ClickError::MissingExpiry.reason())
    );
}

#[test]
fn a_missing_destination_is_rejected() {
    let query = ClickQuery {
        u: None,
        e: Some(LATER.to_string()),
        s: Some(sign(KEY, &record(), "", LATER)),
    };
    assert_eq!(
        evaluate(&query, Some(&record()), NOW, KEY),
        ClickOutcome::BadRequest(ClickError::MissingDestination.reason())
    );
}

#[test]
fn destinations_with_query_characters_round_trip_through_the_link() {
    let destination = "https://shop.example/s?q=a&b=c#frag";
    let query = issued(&record(), destination, LATER);
    assert_eq!(query.u.as_deref(), Some(destination));
    assert_eq!(
        evaluate(&query, Some(&record()), NOW, KEY),
        ClickOutcome::Redirect(destination.to_string())
    );
}

#[test]
fn an_unsafe_destination_is_never_wrapped_into_a_link() {
    for destination in ["javascript:alert(1)", "//evil.example", "mailto:x@example.com"] {
        assert!(
            build_click_url("https://bot.example", &record(), destination, LATER, KEY).is_none(),
            "{destination:?}"
        );
    }
}

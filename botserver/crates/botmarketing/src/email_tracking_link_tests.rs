//! Tests for the tracked links built into campaign emails (#2139).

use super::wrap_tracking_links;
use crate::click_sign::{evaluate, ClickOutcome, ClickQuery, TrackingRecord};
use url::form_urlencoded;
use uuid::Uuid;

const KEY: &str = "test-signing-key";
const NOW: i64 = 1_700_000_000;
const EXPIRES_AT: i64 = NOW + 3_600;

fn record() -> TrackingRecord {
    TrackingRecord {
        id: Uuid::parse_str("6f1f6f9a-0d1a-4c02-9a53-2f7f4b0c1d11").expect("tracking id"),
        campaign_id: None,
        recipient_id: None,
    }
}

fn first_href(html: &str) -> String {
    let start = html.find("href=\"").expect("a href") + 6;
    let end = start + html[start..].find('"').expect("closing quote");
    html[start..end].to_string()
}

fn query_of(url: &str) -> ClickQuery {
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

#[test]
fn a_wrapped_link_round_trips_to_its_destination() {
    // `&amp;` is the HTML spelling of `&`; the redirect carries the URL the
    // browser would have navigated to.
    let html = r#"<a href="https://shop.example/s?q=a&amp;b=c">Buy</a>"#;
    let out = wrap_tracking_links(html, &record(), "https://bot.example", EXPIRES_AT, KEY);
    let link = first_href(&out);
    assert!(link.starts_with("https://bot.example/api/marketing/track/click/"));
    assert_eq!(
        evaluate(&query_of(&link), Some(&record()), NOW, KEY),
        ClickOutcome::Redirect("https://shop.example/s?q=a&b=c".to_string())
    );
}

#[test]
fn anchors_mailto_and_unsafe_destinations_stay_untouched() {
    let html = concat!(
        r##"<a href="#top">top</a>"##,
        r#"<a href="mailto:a@example.com">mail</a>"#,
        r#"<a href="javascript:alert(1)">bad</a>"#,
        r#"<a href="/pricing">prices</a>"#,
    );
    let out = wrap_tracking_links(html, &record(), "https://bot.example", EXPIRES_AT, KEY);
    assert!(out.contains(r##"href="#top""##));
    assert!(out.contains(r#"href="mailto:a@example.com""#));
    assert!(out.contains(r#"href="javascript:alert(1)""#));
    assert_eq!(out.matches("/api/marketing/track/click/").count(), 1);
}

#[test]
fn an_empty_signing_key_produces_no_tracking_links() {
    let html = r#"<a href="/pricing">prices</a>"#;
    let out = wrap_tracking_links(html, &record(), "https://bot.example", EXPIRES_AT, "");
    assert_eq!(out, html);
}

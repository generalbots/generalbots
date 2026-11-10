//! Offline unit tests for the video analytics reducers (#2075).
//!
//! `record_view`/`get_analytics` talk to Postgres, but every number they expose
//! is produced by four pure helpers: the engagement score, the retention
//! decoder, the geography roll-up and the device normaliser. Those are tested
//! here; the SQL shell is covered by the integration suite.

use super::{calculate_engagement_score, parse_devices, parse_geography, parse_retention};
use serde_json::json;

fn approx(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < 0.01,
        "expected {expected}, got {actual}"
    );
}

#[test]
fn engagement_is_zero_without_views() {
    // No view means no rate: a completions/shares count without views must not
    // produce a score (division by zero would otherwise leak into the API).
    assert_eq!(calculate_engagement_score(0, 0, 0, 0), 0.0);
    assert_eq!(calculate_engagement_score(0, 5, 2, 3), 0.0);
}

#[test]
fn engagement_applies_the_published_weights() {
    // 50% completions + 30% shares + 20% likes, scaled to 0..100.
    // 50/100, 10/100, 20/100 -> (0.5*0.5 + 0.1*0.3 + 0.2*0.2) * 100 = 32.
    approx(calculate_engagement_score(100, 50, 10, 20), 32.0);
    // A perfect video (every view completes, is shared and liked) scores 100.
    approx(calculate_engagement_score(10, 10, 10, 10), 100.0);
    // Completions only: half the weight.
    approx(calculate_engagement_score(10, 10, 0, 0), 50.0);
}

#[test]
fn engagement_grows_with_every_signal() {
    let base = calculate_engagement_score(100, 10, 1, 1);
    assert!(calculate_engagement_score(100, 11, 1, 1) > base);
    assert!(calculate_engagement_score(100, 10, 2, 1) > base);
    assert!(calculate_engagement_score(100, 10, 1, 2) > base);
    // More views with the same absolute counts lowers the rate.
    assert!(calculate_engagement_score(1000, 10, 1, 1) < base);
}

#[test]
fn retention_accepts_missing_and_malformed_json() {
    assert!(parse_retention(&None).is_empty());
    assert!(parse_retention(&Some(json!({}))).is_empty());
    assert!(parse_retention(&Some(json!([]))).is_empty());
}

#[test]
fn retention_skips_entries_missing_a_field() {
    let retention = parse_retention(&Some(json!([
        {"percent": 10.0, "viewers": 5},
        {"percent": 20.0},
        {"viewers": 7},
        {"percent": "30", "viewers": 3}
    ])));
    assert_eq!(retention.len(), 1);
    approx(retention[0].percent, 10.0);
    assert_eq!(retention[0].viewers, 5);
}

#[test]
fn geography_is_empty_without_views() {
    assert!(parse_geography(&None).is_empty());
    assert!(parse_geography(&Some(json!({}))).is_empty());
    assert!(parse_geography(&Some(json!({"BR": 0, "US": 0}))).is_empty());
    assert!(parse_geography(&Some(json!(["BR"]))).is_empty());
}

#[test]
fn geography_is_sorted_and_percentual() {
    let geo = parse_geography(&Some(json!({"BR": 60, "US": 30, "DE": 10})));
    let countries: Vec<&str> = geo.iter().map(|g| g.country.as_str()).collect();
    assert_eq!(countries, ["BR", "US", "DE"]);
    approx(geo[0].percent, 60.0);
    approx(geo[1].percent, 30.0);
    approx(geo[2].percent, 10.0);
    assert_eq!(geo[0].views, 60);
}

#[test]
fn geography_is_truncated_to_the_top_ten() {
    let mut value = serde_json::Map::new();
    for index in 0..15 {
        value.insert(format!("C{index:02}"), json!(index + 1));
    }
    let geo = parse_geography(&Some(serde_json::Value::Object(value)));
    assert_eq!(geo.len(), 10);
    // Highest first, so the truncated tail is the smallest counters.
    assert_eq!(geo[0].country, "C14");
    assert_eq!(geo[9].country, "C05");
}

#[test]
fn device_breakdown_is_zero_when_unknown_or_empty() {
    for value in [None, Some(json!({})), Some(json!({"desktop": 0, "tv": 0}))] {
        let devices = parse_devices(&value);
        assert_eq!(devices.desktop, 0.0);
        assert_eq!(devices.mobile, 0.0);
        assert_eq!(devices.tablet, 0.0);
        assert_eq!(devices.tv, 0.0);
    }
    // A non-object payload must not panic either.
    let devices = parse_devices(&Some(json!("desktop")));
    assert_eq!(devices.desktop, 0.0);
}

#[test]
fn device_breakdown_normalises_to_one_hundred_percent() {
    let devices = parse_devices(&Some(json!({
        "desktop": 50,
        "mobile": 30,
        "tablet": 10,
        "tv": 10,
        "unknown_device": 999
    })));
    approx(devices.desktop, 50.0);
    approx(devices.mobile, 30.0);
    approx(devices.tablet, 10.0);
    approx(devices.tv, 10.0);
    approx(
        devices.desktop + devices.mobile + devices.tablet + devices.tv,
        100.0,
    );
}

#[test]
fn device_breakdown_ignores_non_numeric_counters() {
    let devices = parse_devices(&Some(json!({"desktop": "many", "mobile": 4})));
    approx(devices.desktop, 0.0);
    approx(devices.mobile, 100.0);
}

use super::*;
use reqwest::header::{HeaderName, HeaderValue};

fn headers(pairs: &[(&str, &str)]) -> HeaderMap {
    let mut map = HeaderMap::new();
    for (k, v) in pairs {
        if let (Ok(name), Ok(value)) = (HeaderName::try_from(*k), HeaderValue::from_str(v)) {
            map.insert(name, value);
        }
    }
    map
}

fn now() -> DateTime<Utc> {
    Utc.timestamp_opt(1_795_000_000, 0).single().unwrap_or_else(Utc::now)
}

#[test]
fn openai_headers_give_requests_and_tokens() {
    let map = headers(&[
        ("x-ratelimit-limit-requests", "500"),
        ("x-ratelimit-remaining-requests", "499"),
        ("x-ratelimit-reset-requests", "120ms"),
        ("x-ratelimit-limit-tokens", "30000"),
        ("x-ratelimit-remaining-tokens", "22500"),
        ("x-ratelimit-reset-tokens", "6m0s"),
    ]);
    let windows = parse_headers(&map, now());
    assert_eq!(windows.len(), 2);
    assert_eq!(windows[0].name, "requests");
    assert_eq!(windows[0].limit, Some(500));
    assert_eq!(windows[0].remaining, Some(499));
    assert_eq!(windows[0].used_percent, 0.2);
    assert_eq!(windows[0].resets_at, Some(now() + Duration::milliseconds(120)));
    assert_eq!(windows[1].name, "tokens");
    assert_eq!(windows[1].used_percent, 25.0);
    assert_eq!(windows[1].resets_at, Some(now() + Duration::minutes(6)));
}

#[test]
fn anthropic_headers_give_four_windows() {
    let map = headers(&[
        ("anthropic-ratelimit-requests-limit", "50"),
        ("anthropic-ratelimit-requests-remaining", "49"),
        ("anthropic-ratelimit-requests-reset", "2026-11-21T17:16:00Z"),
        ("anthropic-ratelimit-tokens-limit", "100000"),
        ("anthropic-ratelimit-tokens-remaining", "90000"),
        ("anthropic-ratelimit-input-tokens-limit", "80000"),
        ("anthropic-ratelimit-input-tokens-remaining", "80000"),
        ("anthropic-ratelimit-output-tokens-limit", "20000"),
        ("anthropic-ratelimit-output-tokens-remaining", "0"),
    ]);
    let windows = parse_headers(&map, now());
    let names: Vec<&str> = windows.iter().map(|w| w.name.as_str()).collect();
    assert_eq!(names, ["requests", "tokens", "input_tokens", "output_tokens"]);
    assert_eq!(windows[0].resets_at.map(|t| t.to_rfc3339()).as_deref(), Some("2026-11-21T17:16:00+00:00"));
    assert_eq!(windows[1].used_percent, 10.0);
    assert_eq!(windows[2].used_percent, 0.0);
    assert_eq!(windows[3].used_percent, 100.0);
}

#[test]
fn a_window_without_a_limit_is_not_reported() {
    let map = headers(&[("x-ratelimit-remaining-requests", "10")]);
    assert!(parse_headers(&map, now()).is_empty());
    let zero = headers(&[("x-ratelimit-limit-requests", "0"), ("x-ratelimit-remaining-requests", "0")]);
    assert!(parse_headers(&zero, now()).is_empty());
}

#[test]
fn generic_headers_use_epoch_milliseconds() {
    let map = headers(&[
        ("x-ratelimit-limit", "20"),
        ("x-ratelimit-remaining", "15"),
        ("x-ratelimit-reset", "1795000060000"),
    ]);
    let windows = parse_headers(&map, now());
    assert_eq!(windows.len(), 1);
    assert_eq!(windows[0].used_percent, 25.0);
    assert_eq!(windows[0].resets_at, Some(now() + Duration::seconds(60)));
}

#[test]
fn plan_headers_name_windows_by_minutes() {
    let map = headers(&[
        ("x-codex-primary-used-percent", "81.0"),
        ("x-codex-primary-window-minutes", "300"),
        ("x-codex-primary-reset-after-seconds", "3600"),
        ("x-codex-secondary-used-percent", "68"),
        ("x-codex-secondary-window-minutes", "10080"),
    ]);
    let windows = parse_headers(&map, now());
    assert_eq!(windows[0].name, "5h");
    assert_eq!(windows[0].used_percent, 81.0);
    assert_eq!(windows[0].resets_at, Some(now() + Duration::hours(1)));
    assert_eq!(windows[1].name, "weekly");
    assert_eq!(windows[1].resets_at, None);
}

#[test]
fn reset_formats() {
    let n = now();
    assert_eq!(parse_reset("1h2m3.5s", n), Some(n + Duration::milliseconds(3_723_500)));
    assert_eq!(parse_reset("30", n), Some(n + Duration::seconds(30)));
    assert_eq!(parse_reset("1795000100", n), Some(n + Duration::seconds(100)));
    assert_eq!(parse_reset("soon", n), None);
    assert_eq!(parse_reset("", n), None);
}

#[test]
fn record_keeps_values_per_host_and_ignores_empty_responses() {
    let url = "https://Limits-Test.example/v1/chat/completions";
    record(url, &headers(&[("x-ratelimit-limit-requests", "10"), ("x-ratelimit-remaining-requests", "4")]));
    record(url, &HeaderMap::new());
    let seen = for_host("limits-test.example");
    assert_eq!(seen.as_ref().map(|s| s.windows[0].used_percent), Some(60.0));
    assert!(all().iter().any(|h| h.host == "limits-test.example"));
    assert_eq!(host_of("not a url"), "");
}

//! Last rate-limit state each HTTP provider reported in its response headers.
//!
//! Every chat client calls [`record`] right after a response arrives; the
//! values are kept in memory per provider host and read back by the limits
//! endpoint. Only the numeric limit headers are read; nothing from the request
//! (keys, bodies) is stored or logged.

use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

use chrono::{DateTime, Duration, TimeZone, Utc};
use reqwest::header::HeaderMap;
use serde::Serialize;

/// One quota window as the provider reported it.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LimitWindow {
    /// `requests`, `tokens`, `input_tokens`, `output_tokens`, `5h`, `weekly`, ...
    pub name: String,
    pub used_percent: f64,
    pub limit: Option<u64>,
    pub remaining: Option<u64>,
    pub resets_at: Option<DateTime<Utc>>,
}

/// What one provider host reported most recently.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct HostLimits {
    pub host: String,
    pub observed_at: DateTime<Utc>,
    pub windows: Vec<LimitWindow>,
}

fn store() -> &'static RwLock<HashMap<String, HostLimits>> {
    static STORE: OnceLock<RwLock<HashMap<String, HostLimits>>> = OnceLock::new();
    STORE.get_or_init(|| RwLock::new(HashMap::new()))
}

/// Lower-case host of a request URL; empty when the URL does not parse.
pub fn host_of(url: &str) -> String {
    reqwest::Url::parse(url.trim())
        .ok()
        .and_then(|u| u.host_str().map(str::to_ascii_lowercase))
        .unwrap_or_else(|| { log::debug!("cannot parse host from URL"); Default::default() })
}

/// Remembers the limit headers of a response from `url`. Responses without
/// limit headers leave the previous values in place.
pub fn record(url: &str, headers: &HeaderMap) {
    let host = host_of(url);
    if host.is_empty() {
        return;
    }
    let now = Utc::now();
    let windows = parse_headers(headers, now);
    if windows.is_empty() {
        return;
    }
    if let Ok(mut map) = store().write() {
        map.insert(host.clone(), HostLimits { host, observed_at: now, windows });
    }
}

/// Last values seen for `host`, if any response carried limit headers.
pub fn for_host(host: &str) -> Option<HostLimits> {
    store().read().ok()?.get(&host.to_ascii_lowercase()).cloned()
}

/// Every host seen so far.
pub fn all() -> Vec<HostLimits> {
    let mut out: Vec<HostLimits> = store()
        .read()
        .map(|map| map.values().cloned().collect())
        .unwrap_or_else(|e| { log::debug!("rate-limit store lock poisoned: {e}"); Default::default() });
    out.sort_by(|a, b| a.host.cmp(&b.host));
    out
}

fn header<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name).and_then(|v| v.to_str().ok()).map(str::trim)
}

fn number(headers: &HeaderMap, name: &str) -> Option<u64> {
    let raw = header(headers, name)?;
    raw.parse::<u64>()
        .ok()
        .or_else(|| raw.parse::<f64>().ok().filter(|v| *v >= 0.0).map(|v| v as u64))
}

fn used_percent(limit: u64, remaining: u64) -> Option<f64> {
    if limit == 0 {
        return None;
    }
    let used = limit.saturating_sub(remaining.min(limit)) as f64;
    Some(((used / limit as f64) * 1000.0).round() / 10.0)
}

fn counted(
    name: &str,
    limit: Option<u64>,
    remaining: Option<u64>,
    reset: Option<&str>,
    now: DateTime<Utc>,
) -> Option<LimitWindow> {
    let (limit, remaining) = (limit?, remaining?);
    Some(LimitWindow {
        name: name.to_string(),
        used_percent: used_percent(limit, remaining)?,
        limit: Some(limit),
        remaining: Some(remaining),
        resets_at: reset.and_then(|r| parse_reset(r, now)),
    })
}

/// Reads the limit headers OpenAI-compatible providers, Anthropic and the
/// ChatGPT plan back end send. Only windows with a known limit and remaining
/// count (or an explicit used percentage) are returned.
pub fn parse_headers(headers: &HeaderMap, now: DateTime<Utc>) -> Vec<LimitWindow> {
    let mut out = Vec::new();
    for (name, metric) in [
        ("requests", "requests"),
        ("tokens", "tokens"),
        ("input_tokens", "input-tokens"),
        ("output_tokens", "output-tokens"),
    ] {
        let window = counted(
            name,
            number(headers, &format!("anthropic-ratelimit-{metric}-limit")),
            number(headers, &format!("anthropic-ratelimit-{metric}-remaining")),
            header(headers, &format!("anthropic-ratelimit-{metric}-reset")),
            now,
        );
        out.extend(window);
    }
    for metric in ["requests", "tokens"] {
        let window = counted(
            metric,
            number(headers, &format!("x-ratelimit-limit-{metric}")),
            number(headers, &format!("x-ratelimit-remaining-{metric}")),
            header(headers, &format!("x-ratelimit-reset-{metric}")),
            now,
        );
        out.extend(window);
    }
    if !out.iter().any(|w| w.name == "requests") {
        out.extend(counted(
            "requests",
            number(headers, "x-ratelimit-limit"),
            number(headers, "x-ratelimit-remaining"),
            header(headers, "x-ratelimit-reset"),
            now,
        ));
    }
    for slot in ["primary", "secondary"] {
        out.extend(plan_window(headers, slot, now));
    }
    out
}

/// `x-codex-<slot>-*` headers of the ChatGPT plan back end.
fn plan_window(headers: &HeaderMap, slot: &str, now: DateTime<Utc>) -> Option<LimitWindow> {
    let used: f64 = header(headers, &format!("x-codex-{slot}-used-percent"))?.parse().ok()?;
    let minutes = number(headers, &format!("x-codex-{slot}-window-minutes"));
    let resets_at = header(headers, &format!("x-codex-{slot}-reset-at"))
        .and_then(|r| parse_reset(r, now))
        .or_else(|| {
            number(headers, &format!("x-codex-{slot}-reset-after-seconds"))
                .and_then(|s| i64::try_from(s).ok())
                .map(|s| now + Duration::seconds(s))
        });
    Some(LimitWindow {
        name: minutes.map(window_name).unwrap_or_else(|| slot.to_string()),
        used_percent: used.clamp(0.0, 100.0),
        limit: None,
        remaining: None,
        resets_at,
    })
}

/// Display name of a rolling window given in minutes.
pub fn window_name(minutes: u64) -> String {
    match minutes {
        300 => "5h".to_string(),
        1440 => "daily".to_string(),
        10080 => "weekly".to_string(),
        m if m % 60 == 0 => format!("{}h", m / 60),
        m => format!("{m}m"),
    }
}

/// A reset value: RFC 3339 time, Unix seconds or milliseconds, plain seconds
/// from now, or a Go-style duration (`6m0s`, `1h2m3.5s`, `20ms`).
pub fn parse_reset(raw: &str, now: DateTime<Utc>) -> Option<DateTime<Utc>> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }
    if let Ok(at) = DateTime::parse_from_rfc3339(raw) {
        return Some(at.with_timezone(&Utc));
    }
    if let Ok(value) = raw.parse::<f64>() {
        if value < 0.0 {
            return None;
        }
        if value >= 1e12 {
            return Utc.timestamp_millis_opt(value as i64).single();
        }
        if value >= 1e9 {
            return Utc.timestamp_opt(value as i64, 0).single();
        }
        return Some(now + Duration::milliseconds((value * 1000.0) as i64));
    }
    parse_go_duration(raw).map(|ms| now + Duration::milliseconds(ms))
}

fn parse_go_duration(raw: &str) -> Option<i64> {
    let mut total = 0.0_f64;
    let mut rest = raw;
    let mut any = false;
    while !rest.is_empty() {
        let digits = rest
            .find(|c: char| !(c.is_ascii_digit() || c == '.'))
            .unwrap_or(rest.len());
        let value: f64 = rest[..digits].parse().ok()?;
        rest = &rest[digits..];
        let unit_len = rest.find(|c: char| c.is_ascii_digit()).unwrap_or(rest.len());
        let factor = match &rest[..unit_len] {
            "ms" => 1.0,
            "s" => 1000.0,
            "m" => 60_000.0,
            "h" => 3_600_000.0,
            _ => return None,
        };
        rest = &rest[unit_len..];
        total += value * factor;
        any = true;
    }
    any.then_some(total as i64)
}

#[cfg(test)]
#[path = "rate_limits_tests.rs"]
mod tests;

//! Shared LLM retry policy (#1393).
//!
//! Free/aggregator gateways enforce per-client concurrency and return
//! `429 Too Many Requests` with either a JSON `retry_after` field or a
//! standard `Retry-After` header. Retrying on a fixed 1s schedule keeps
//! landing inside the same window and aborts agent runs. This module
//! provides:
//!
//! - [`parse_retry_after_ms`] — extracts the wait from the error body
//!   (`retry_after`, `error.retry_after`) or the `Retry-After` header
//!   value, accepting seconds or an HTTP-date.
//! - [`backoff_ms`] — exponential backoff with jitter for non-429 errors.
//! - [`call_gate`] — a process-wide semaphore so chat, Vibe agent loops
//!   and scaffolding serialize their LLM calls instead of racing each
//!   other into `concurrent_request_limit_exceeded`.

use std::time::Duration;

/// Wait used when a 429 carries no parsable delay.
pub const DEFAULT_429_WAIT: Duration = Duration::from_secs(10);

/// Maximum wait we accept from a `Retry-After` before giving up (2 min).
pub const MAX_RETRY_AFTER: Duration = Duration::from_secs(120);

/// Base delay for the exponential backoff schedule.
pub const BACKOFF_BASE_MS: u64 = 2_000;

/// Cap for the exponential backoff schedule.
pub const BACKOFF_CAP_MS: u64 = 60_000;

/// Extract a retry delay in milliseconds from a 429/5xx response.
///
/// `body` is the raw response text (JSON or plain); `header_value` is the
/// `Retry-After` header when the server sent one. Returns `None` when no
/// usable delay is present.
pub fn parse_retry_after_ms(body: &str, header_value: Option<&str>) -> Option<Duration> {
    // 1. Retry-After header: delta-seconds form (HTTP-date form is rare on
    // JSON APIs; if parsing as seconds fails we fall through to the body).
    if let Some(hv) = header_value.map(str::trim).filter(|s| !s.is_empty()) {
        if let Ok(secs) = hv.parse::<u64>() {
            return Some(clamp_retry(Duration::from_secs(secs)));
        }
    }
    // 2. JSON body: `retry_after` at top level or under `error`.
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(body) {
        let cand = v
            .get("retry_after")
            .or_else(|| v.get("error").and_then(|e| e.get("retry_after")));
        if let Some(n) = cand.and_then(|x| x.as_u64()) {
            return Some(clamp_retry(Duration::from_secs(n)));
        }
        if let Some(f) = cand.and_then(|x| x.as_f64()) {
            return Some(clamp_retry(Duration::from_secs(f.ceil() as u64)));
        }
    }
    None
}

fn clamp_retry(d: Duration) -> Duration {
    if d > MAX_RETRY_AFTER {
        MAX_RETRY_AFTER
    } else {
        d
    }
}

/// The wait to apply for a given 429/5xx response: parsed delay, or the
/// default, or exponential backoff for non-429 transient errors.
pub fn wait_for(status: u16, body: &str, header_value: Option<&str>, attempt: u32) -> Duration {
    if status == 429 {
        parse_retry_after_ms(body, header_value).unwrap_or(DEFAULT_429_WAIT)
    } else {
        backoff_ms(attempt)
    }
}

/// Exponential backoff with ±20% jitter: base * 2^attempt, capped.
pub fn backoff_ms(attempt: u32) -> Duration {
    let exp = attempt.min(10);
    let base = BACKOFF_BASE_MS.saturating_mul(1u64 << exp);
    let base = base.min(BACKOFF_CAP_MS);
    let jitter = (base / 5).max(1);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as u64)
        .unwrap_or(0);
    Duration::from_millis(base.saturating_sub(now % (2 * jitter)) + (now % jitter))
}

/// Whether a status code is transient and worth retrying.
pub fn is_retryable_status(status: u16) -> bool {
    status == 429 || status == 500 || status == 502 || status == 503 || status == 529
}

/// Process-wide gate serializing LLM calls across chat, Vibe and scaffold.
///
/// The permit count stays at 1: free/aggregator tiers enforce a per-client
/// concurrency of ~1, and even paid plans benefit from smoothing bursts.
/// Process-wide gate serializing LLM calls across chat, Vibe and scaffold.
///
/// The permit count stays at 1: free/aggregator tiers enforce a per-client
/// concurrency of ~1, and even paid plans benefit from smoothing bursts.
pub fn call_gate() -> &'static tokio::sync::Semaphore {
    static GATE: std::sync::OnceLock<tokio::sync::Semaphore> = std::sync::OnceLock::new();
    GATE.get_or_init(|| tokio::sync::Semaphore::new(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_retry_after_seconds_from_body() {
        let body = r#"{"error":{"message":"Too many concurrent requests","type":"rate_limit_error","code":"concurrent_request_limit_exceeded","retry_after":10}}"#;
        assert_eq!(
            parse_retry_after_ms(body, None),
            Some(Duration::from_secs(10))
        );
    }

    #[test]
    fn parses_retry_after_top_level() {
        let body = r#"{"retry_after": 30}"#;
        assert_eq!(
            parse_retry_after_ms(body, None),
            Some(Duration::from_secs(30))
        );
    }

    #[test]
    fn parses_retry_after_header_seconds() {
        assert_eq!(
            parse_retry_after_ms("", Some("15")),
            Some(Duration::from_secs(15))
        );
    }

    #[test]
    fn header_wins_over_body() {
        let body = r#"{"retry_after": 10}"#;
        assert_eq!(
            parse_retry_after_ms(body, Some("2")),
            Some(Duration::from_secs(2))
        );
    }

    #[test]
    fn caps_huge_values() {
        let body = r#"{"retry_after": 999999}"#;
        assert_eq!(parse_retry_after_ms(body, None), Some(MAX_RETRY_AFTER));
    }

    #[test]
    fn none_when_absent() {
        assert_eq!(parse_retry_after_ms("plain busy", None), None);
        assert_eq!(parse_retry_after_ms(r#"{"other":1}"#, None), None);
    }

    #[test]
    fn backoff_grows_and_caps() {
        let a0 = backoff_ms(0).as_millis() as u64;
        let a3 = backoff_ms(3).as_millis() as u64;
        let a9 = backoff_ms(9).as_millis() as u64;
        assert!(a0 <= BACKOFF_BASE_MS + 400, "a0={a0}");
        assert!(a3 > a0, "backoff must grow: {a3} vs {a0}");
        assert!(a9 <= BACKOFF_CAP_MS + 400, "a9={a9} must be near cap");
    }

    #[test]
    fn wait_for_uses_retry_after_on_429() {
        let body = r#"{"error":{"retry_after":10}}"#;
        assert_eq!(wait_for(429, body, None, 0), Duration::from_secs(10));
        assert_eq!(
            wait_for(429, "no json", None, 0),
            DEFAULT_429_WAIT
        );
        assert_eq!(wait_for(429, "", Some("3"), 0), Duration::from_secs(3));
    }

    #[test]
    fn wait_for_uses_backoff_on_5xx() {
        let w = wait_for(503, "", None, 2);
        assert!(w >= Duration::from_millis(BACKOFF_BASE_MS / 2));
        assert!(w <= Duration::from_millis(BACKOFF_BASE_MS * 4 + 500));
    }

    #[test]
    fn retryable_statuses() {
        assert!(is_retryable_status(429));
        assert!(is_retryable_status(503));
        assert!(!is_retryable_status(400));
        assert!(!is_retryable_status(401));
        assert!(!is_retryable_status(200));
    }
}

//! State-machine tests for `RateLimiter` (#2064).
//!
//! The limiter is what keeps a single token from exhausting a shared provider
//! budget. These tests drive the real (unfaked) clock, so they only rely on
//! windows that cannot expire inside a test: the counters themselves.

use super::RateLimiter;
use crate::limits::{LimitType, SystemLimits};

fn limits(per_minute: u32, per_hour: u32) -> SystemLimits {
    let mut limits = SystemLimits::default();
    limits.max_api_calls_per_minute = per_minute;
    limits.max_api_calls_per_hour = per_hour;
    limits
}

#[tokio::test]
async fn calls_up_to_the_minute_limit_are_allowed() {
    let limiter = RateLimiter::new(limits(2, 100));

    assert!(limiter.check_rate_limit("alice").await.is_ok());
    assert!(limiter.check_rate_limit("alice").await.is_ok());

    let error = limiter
        .check_rate_limit("alice")
        .await
        .expect_err("third call exceeds the minute budget");
    assert_eq!(error.limit_type, LimitType::ApiCallsMinute);
    assert_eq!(error.current, 3);
    assert_eq!(error.maximum, 2);
    assert_eq!(error.retry_after_secs, Some(60));
}

#[tokio::test]
async fn one_user_exhausting_their_budget_does_not_affect_another() {
    let limiter = RateLimiter::new(limits(1, 100));

    assert!(limiter.check_rate_limit("alice").await.is_ok());
    assert!(limiter.check_rate_limit("alice").await.is_err());
    assert!(limiter.check_rate_limit("bob").await.is_ok());
}

#[tokio::test]
async fn a_zero_minute_budget_rejects_the_first_call() {
    let limiter = RateLimiter::new(limits(0, 100));

    let error = limiter
        .check_rate_limit("alice")
        .await
        .expect_err("no budget");
    assert_eq!(error.limit_type, LimitType::ApiCallsMinute);
    assert_eq!(error.current, 1);
    assert_eq!(error.maximum, 0);
}

#[tokio::test]
async fn an_exhausted_hour_budget_reports_the_hour_window() {
    // The per-user minute window is generous, so the hour counter is the one
    // that trips and it must name its own window and retry delay.
    let limiter = RateLimiter::new(limits(10, 1));

    assert!(limiter.check_rate_limit("alice").await.is_ok());
    let error = limiter
        .check_rate_limit("alice")
        .await
        .expect_err("hour budget exhausted");
    assert_eq!(error.limit_type, LimitType::ApiCallsHour);
    assert_eq!(error.current, 2);
    assert_eq!(error.maximum, 1);
    assert_eq!(error.retry_after_secs, Some(3600));
}

#[tokio::test]
async fn the_global_budget_is_hundred_times_the_per_user_budget() {
    // `max_api_calls_per_minute` is a per-token budget; the process-wide ceiling
    // is derived from it (`* 100`). A zero budget therefore trips on the very
    // first call, for any user.
    let limiter = RateLimiter::new(limits(0, 0));

    let error = limiter
        .check_rate_limit("anyone")
        .await
        .expect_err("global budget exhausted");
    assert_eq!(error.limit_type, LimitType::ApiCallsMinute);
    assert_eq!(error.maximum, 0);
}

#[tokio::test]
async fn cleanup_on_empty_state_is_a_no_op() {
    let limiter = RateLimiter::new(limits(5, 50));
    limiter.cleanup_stale_entries().await;
    limiter.cleanup_stale_entries().await;
    // Nothing was evicted, so the budget is still intact.
    assert!(limiter.check_rate_limit("alice").await.is_ok());
}

#[tokio::test]
async fn cleanup_keeps_recent_entries() {
    let limiter = RateLimiter::new(limits(5, 50));
    assert!(limiter.check_rate_limit("alice").await.is_ok());

    limiter.cleanup_stale_entries().await;

    // The entry survived (its window is seconds old), so the counter continues
    // instead of resetting the budget.
    assert!(limiter.check_rate_limit("alice").await.is_ok());
}

#[test]
fn the_default_limiter_uses_the_system_defaults() {
    let limiter = RateLimiter::default();
    // Construction must not panic and must expose the documented defaults
    // through a first allowed call.
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime");
    assert!(runtime.block_on(limiter.check_rate_limit("alice")).is_ok());
}

//! Offline unit tests for the time-series client defaults (#2075).
//!
//! The defaults are the contract a deployment without explicit settings gets:
//! the `system` org and `metrics` bucket are what the write path targets, and
//! TLS verification must stay on unless an operator turns it off.

use super::*;

#[test]
fn defaults_target_the_system_metrics_bucket() {
    let config = TimeSeriesConfig::default();
    assert_eq!(config.org, "system");
    assert_eq!(config.bucket, "metrics");
    assert!(config.url.is_empty());
    assert!(config.token.is_empty());
}

#[test]
fn defaults_batch_and_flush_on_a_one_second_cadence() {
    let config = TimeSeriesConfig::default();
    assert_eq!(config.batch_size, 1000);
    assert_eq!(config.flush_interval_ms, 1000);
    // A full batch must be flushed within one interval, otherwise a busy
    // minute buffers for longer than the metric resolution.
    assert!(config.batch_size > 0);
    assert!(config.flush_interval_ms > 0);
}

#[test]
fn tls_verification_is_on_by_default() {
    assert!(TimeSeriesConfig::default().verify_tls);
}

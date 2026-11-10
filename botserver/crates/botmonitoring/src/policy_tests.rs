//! Unit tests for the pure monitoring policy: status thresholds, request
//! arithmetic, uptime/status formatting and service-URL parsing.
//!
//! Every expectation here is a published contract — the HTML panels and the
//! health payload read these values verbatim, so a silent change (a `>=` where
//! a `>` belongs, a renamed CSS class) is a user-visible regression.

use crate::policy::{
    calculate_overall_status, request_stats, service_host_port, HealthThresholds, RequestStats,
};
use crate::real_time::{ComponentHealth, HealthStatus};
use crate::{format_uptime_duration, health_status_class, host_port_from_url};
use chrono::Utc;

fn component(name: &str, status: HealthStatus) -> ComponentHealth {
    ComponentHealth {
        name: name.to_string(),
        status,
        latency_ms: Some(1.0),
        message: None,
        last_check: Utc::now(),
    }
}

fn status_of(cpu: f64, mem: f64, err: f64, components: &[ComponentHealth]) -> HealthStatus {
    calculate_overall_status(cpu, mem, err, components, &HealthThresholds::default())
}

#[test]
fn thresholds_default_is_the_documented_80_95_5_10() {
    let t = HealthThresholds::default();
    assert_eq!(t.cpu_degraded, 80.0);
    assert_eq!(t.cpu_unhealthy, 95.0);
    assert_eq!(t.memory_degraded, 80.0);
    assert_eq!(t.memory_unhealthy, 95.0);
    assert_eq!(t.error_rate_degraded, 5.0);
    assert_eq!(t.error_rate_unhealthy, 10.0);
}

#[test]
fn quiet_host_with_healthy_components_is_healthy() {
    assert_eq!(
        status_of(0.0, 0.0, 0.0, &[component("db", HealthStatus::Healthy)]),
        HealthStatus::Healthy
    );
}

#[test]
fn exactly_on_the_degraded_threshold_stays_healthy() {
    // Strict `>`: a box sitting at exactly 80/80/5% must not flap to degraded.
    assert_eq!(status_of(80.0, 80.0, 5.0, &[]), HealthStatus::Healthy);
}

#[test]
fn exactly_on_the_unhealthy_threshold_is_degraded_not_unhealthy() {
    assert_eq!(status_of(95.0, 95.0, 10.0, &[]), HealthStatus::Degraded);
}

#[test]
fn cpu_above_degraded_is_degraded() {
    assert_eq!(status_of(80.1, 0.0, 0.0, &[]), HealthStatus::Degraded);
}

#[test]
fn memory_above_degraded_is_degraded() {
    assert_eq!(status_of(0.0, 80.1, 0.0, &[]), HealthStatus::Degraded);
}

#[test]
fn error_rate_above_degraded_is_degraded() {
    assert_eq!(status_of(0.0, 0.0, 5.1, &[]), HealthStatus::Degraded);
}

#[test]
fn cpu_above_unhealthy_is_unhealthy() {
    assert_eq!(status_of(95.1, 0.0, 0.0, &[]), HealthStatus::Unhealthy);
}

#[test]
fn memory_above_unhealthy_is_unhealthy() {
    assert_eq!(status_of(0.0, 95.1, 0.0, &[]), HealthStatus::Unhealthy);
}

#[test]
fn error_rate_above_unhealthy_is_unhealthy() {
    assert_eq!(status_of(0.0, 0.0, 10.1, &[]), HealthStatus::Unhealthy);
}

#[test]
fn one_unhealthy_component_outranks_every_good_signal() {
    let components = [
        component("db", HealthStatus::Healthy),
        component("cache", HealthStatus::Unhealthy),
    ];
    assert_eq!(status_of(0.0, 0.0, 0.0, &components), HealthStatus::Unhealthy);
}

#[test]
fn one_degraded_component_degrades_the_host() {
    let components = [
        component("db", HealthStatus::Healthy),
        component("cache", HealthStatus::Degraded),
    ];
    assert_eq!(status_of(0.0, 0.0, 0.0, &components), HealthStatus::Degraded);
}

#[test]
fn unknown_components_do_not_change_the_status() {
    assert_eq!(
        status_of(0.0, 0.0, 0.0, &[component("llm", HealthStatus::Unknown)]),
        HealthStatus::Healthy
    );
}

#[test]
fn nan_signals_never_panic_and_never_escalate() {
    // A failed /proc read yields NaN; `NaN > x` is false for every threshold.
    assert_eq!(
        status_of(f64::NAN, f64::NAN, f64::NAN, &[]),
        HealthStatus::Healthy
    );
}

#[test]
fn empty_component_list_is_evaluated_on_signals_only() {
    assert_eq!(status_of(0.0, 0.0, 0.0, &[]), HealthStatus::Healthy);
    assert_eq!(status_of(99.0, 0.0, 0.0, &[]), HealthStatus::Unhealthy);
}

#[test]
fn health_status_class_matches_the_panel_css_classes() {
    // A typo here silently leaves panels unstyled.
    let cases = [
        (HealthStatus::Healthy, "healthy"),
        (HealthStatus::Degraded, "degraded"),
        (HealthStatus::Unhealthy, "unhealthy"),
        (HealthStatus::Unknown, "unknown"),
    ];
    for (status, expected) in cases {
        assert_eq!(health_status_class(&status), expected);
    }
}

#[test]
fn format_uptime_duration_drops_empty_leading_units() {
    let cases = [
        (0u64, "0m"),
        (59, "0m"),
        (60, "1m"),
        (3599, "59m"),
        (3600, "1h 0m"),
        (3661, "1h 1m"),
        (86400, "1d 0h 0m"),
        (90000, "1d 1h 0m"),
        (u64::MAX, "213503982334601d 7h 0m"),
    ];
    for (seconds, expected) in cases {
        assert_eq!(format_uptime_duration(seconds), expected, "seconds={seconds}");
    }
}

#[test]
fn request_stats_matches_the_documented_three_request_example() {
    let stats = request_stats(3, 1, 60, 10);
    assert_eq!(
        stats,
        RequestStats {
            error_rate_percent: (1.0 / 3.0) * 100.0,
            average_latency_ms: 20.0,
            requests_per_second: 0.3,
        }
    );
    assert!((stats.error_rate_percent - 33.333_333).abs() < 1e-5);
}

#[test]
fn request_stats_never_divides_by_zero() {
    let stats = request_stats(0, 0, 0, 0);
    assert_eq!(stats.error_rate_percent, 0.0);
    assert_eq!(stats.average_latency_ms, 0.0);
    assert_eq!(stats.requests_per_second, 0.0);
    assert!(!stats.error_rate_percent.is_nan());
    assert!(!stats.requests_per_second.is_nan());
}

#[test]
fn request_stats_with_zero_uptime_still_reports_latency() {
    // The collector can see traffic before the first second has elapsed.
    let stats = request_stats(4, 0, 40, 0);
    assert_eq!(stats.error_rate_percent, 0.0);
    assert_eq!(stats.average_latency_ms, 10.0);
    assert_eq!(stats.requests_per_second, 0.0);
}

#[test]
fn request_stats_rounds_no_counters() {
    let stats = request_stats(8, 2, 100, 20);
    assert_eq!(stats.error_rate_percent, 25.0);
    assert_eq!(stats.average_latency_ms, 12.5);
    assert_eq!(stats.requests_per_second, 0.4);
}

#[test]
fn service_host_port_accepts_the_configured_url_shapes() {
    let cases = [
        ("http://localhost:8080", 5432u16, Some(("localhost", 8080u16))),
        ("https://db.internal/health", 5432, Some(("db.internal", 5432))),
        ("postgres://user:pa%40ss@tables:5433/bots", 5432, Some(("tables", 5433))),
        ("cache.internal", 6379, Some(("cache.internal", 6379))),
        ("  http://spaced:9000  ", 9000, Some(("spaced", 9000))),
        ("http://host:99999", 8080, Some(("host", 8080))),
        ("http://host:", 8080, Some(("host", 8080))),
        ("http://host:notaport", 8080, Some(("host:notaport", 8080))),
        ("http://", 8080, None),
        ("", 8080, None),
    ];
    for (url, default, expected) in cases {
        let got = service_host_port(url, default);
        let expected = expected.map(|(h, p)| (h.to_string(), p));
        assert_eq!(got, expected, "url={url:?}");
    }
}

#[test]
fn host_port_from_url_is_the_public_entry_point() {
    assert_eq!(
        host_port_from_url("http://minio:9100/api", 9000),
        Some(("minio".to_string(), 9100))
    );
    assert_eq!(
        host_port_from_url("minio", 9000),
        Some(("minio".to_string(), 9000))
    );
    assert_eq!(host_port_from_url("", 9000), None);
}

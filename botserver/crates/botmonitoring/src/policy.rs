use crate::real_time::{ComponentHealth, HealthStatus};

/// Thresholds at which the overall system status degrades.
///
/// Every comparison in [`calculate_overall_status`] is strict (`>`), so a value
/// sitting exactly on a threshold keeps the better status — a panel that turns
/// red at exactly 80% would flap on every normal load peak.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct HealthThresholds {
    pub cpu_degraded: f64,
    pub cpu_unhealthy: f64,
    pub memory_degraded: f64,
    pub memory_unhealthy: f64,
    pub error_rate_degraded: f64,
    pub error_rate_unhealthy: f64,
}

impl Default for HealthThresholds {
    fn default() -> Self {
        Self {
            cpu_degraded: 80.0,
            cpu_unhealthy: 95.0,
            memory_degraded: 80.0,
            memory_unhealthy: 95.0,
            error_rate_degraded: 5.0,
            error_rate_unhealthy: 10.0,
        }
    }
}

/// Collapses per-component health and the three host signals into one status.
///
/// A single unhealthy component always wins, and the worst signal decides
/// otherwise. `NaN` signals are not "over" any threshold (all comparisons with
/// `NaN` are false), so a failed `/proc` read degrades nothing — it reports
/// `0.0` upstream anyway.
pub(crate) fn calculate_overall_status(
    cpu_usage: f64,
    memory_usage: f64,
    error_rate: f64,
    components: &[ComponentHealth],
    thresholds: &HealthThresholds,
) -> HealthStatus {
    let unhealthy_components = components
        .iter()
        .filter(|c| c.status == HealthStatus::Unhealthy)
        .count();

    let degraded_components = components
        .iter()
        .filter(|c| c.status == HealthStatus::Degraded)
        .count();

    if unhealthy_components > 0
        || cpu_usage > thresholds.cpu_unhealthy
        || memory_usage > thresholds.memory_unhealthy
        || error_rate > thresholds.error_rate_unhealthy
    {
        HealthStatus::Unhealthy
    } else if degraded_components > 0
        || cpu_usage > thresholds.cpu_degraded
        || memory_usage > thresholds.memory_degraded
        || error_rate > thresholds.error_rate_degraded
    {
        HealthStatus::Degraded
    } else {
        HealthStatus::Healthy
    }
}

/// Request counters as the percentages the health payload reports.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct RequestStats {
    pub error_rate_percent: f64,
    pub average_latency_ms: f64,
    pub requests_per_second: f64,
}

/// Derives the request statistics from the raw counters.
///
/// All three divisions are guarded: a collector that has seen no traffic (and
/// therefore has no uptime yet) reports zeros instead of `NaN`/`inf`, which
/// would otherwise serialise as `null` and break the dashboard charts.
pub(crate) fn request_stats(
    request_count: u64,
    error_count: u64,
    total_latency_ms: u64,
    uptime_seconds: u64,
) -> RequestStats {
    let error_rate_percent = if request_count > 0 {
        (error_count as f64 / request_count as f64) * 100.0
    } else {
        0.0
    };

    let average_latency_ms = if request_count > 0 {
        total_latency_ms as f64 / request_count as f64
    } else {
        0.0
    };

    let requests_per_second = if uptime_seconds > 0 {
        request_count as f64 / uptime_seconds as f64
    } else {
        0.0
    };

    RequestStats {
        error_rate_percent,
        average_latency_ms,
        requests_per_second,
    }
}

/// Host and port addressed by a service URL, used by the liveness probes.
///
/// Accepts a bare host, a host with a port and a full URL with credentials,
/// path or query. A non-numeric suffix after the last `:` stays part of the
/// hostname (IPv6 addresses are not supported — the probes are IPv4-only).
pub(crate) fn service_host_port(url: &str, default_port: u16) -> Option<(String, u16)> {
    let trimmed = url.trim();
    let without_scheme = trimmed
        .split_once("://")
        .map(|(_, rest)| rest)
        .unwrap_or(trimmed);
    // Split the authority (host[:port]) from the path/query/fragment *before*
    // stripping credentials: `postgres://user:pass@tables:5432/bots` used to
    // yield the userinfo as the host, so the probe always reported the
    // database down.
    let authority = without_scheme
        .split(['/', '?', '#'])
        .next()
        .unwrap_or(without_scheme);
    let host_part = match authority.rsplit_once('@') {
        Some((_, host)) => host,
        None => authority,
    };
    if host_part.is_empty() {
        return None;
    }
    let (host, port) = match host_part.rsplit_once(':') {
        Some((h, p)) if !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()) => {
            (h.to_string(), p.parse::<u16>().unwrap_or(default_port))
        }
        // A trailing colon is an empty port, not part of the hostname.
        Some((h, "")) => (h.to_string(), default_port),
        // A non-numeric suffix stays in the hostname.
        _ => (host_part.to_string(), default_port),
    };
    if host.is_empty() {
        return None;
    }
    Some((host, port))
}

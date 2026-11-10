//! SLO definitions, error budgets and burn-rate math for the operational view
//! (#2116).
//!
//! Targets are read from `ops/slo.yaml`, which is the single source of truth
//! for the dashboard, the alerts and the runbook links. Issue #2115 owns the
//! ops/ directory; while that lands, this crate ships the same shape as a
//! built-in default and the dashboard reports which source it used, so a
//! missing file degrades to a labelled fallback instead of an empty panel.
//!
//! The file is a flat key/value YAML map (a subset, parsed without a YAML
//! dependency):
//!
//! ```yaml
//! version: 1
//! window_days: 30
//! owner: platform-oncall
//! objective.api_p95.metric: latency_histogram
//! objective.api_p95.threshold_ms: 500
//! objective.api_p95.target: 0.95
//! objective.api_p95.runbook: botbook/src/10-configuration-deployment/operations/runbook-disk-and-queue.md
//! objective.api_p95.panel: ops-endpoints
//! alert.fast_burn: 14.4
//! alert.medium_burn: 6
//! alert.slow_burn: 1
//! ```

use std::collections::BTreeMap;
use std::path::Path;

/// Burn-rate windows shown by the SLO panel, matching the multi-window alert
/// policy: 1 h catches hard outages, 6 h catches sustained degradation and
/// 3 d catches a slow leak that never pages on the short windows.
pub const BURN_WINDOWS: [(&str, i32); 3] = [("1h", 60), ("6h", 360), ("3d", 4_320)];

/// Metric kinds an objective may reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricKind {
    /// Share of API requests answered under `threshold_ms` (histogram based).
    LatencyHistogram,
    /// Share of API requests that did not answer 5xx.
    ErrorRate,
    /// Share of LLM calls whose first token arrived under `threshold_ms`.
    LlmFirstToken,
    /// Share of LLM calls that finished under `threshold_ms`.
    LlmAnswer,
    /// Share of WebSocket handshakes that were not rejected.
    WsHandshake,
}

impl MetricKind {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "latency_histogram" => Some(Self::LatencyHistogram),
            "error_rate" => Some(Self::ErrorRate),
            "llm_first_token" => Some(Self::LlmFirstToken),
            "llm_answer" => Some(Self::LlmAnswer),
            "ws_handshake" => Some(Self::WsHandshake),
            _ => None,
        }
    }

    /// Human label for the panel.
    pub fn label(self) -> &'static str {
        match self {
            Self::LatencyHistogram => "API latency",
            Self::ErrorRate => "API availability",
            Self::LlmFirstToken => "First token",
            Self::LlmAnswer => "Answer latency",
            Self::WsHandshake => "WS handshake",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Objective {
    pub id: String,
    pub metric: MetricKind,
    pub target: f64,
    pub threshold_ms: Option<f64>,
    pub runbook: String,
    pub panel: String,
}

impl Objective {
    /// Error budget as a ratio: `1 - target` (99 % → 1 %).
    pub fn budget_ratio(&self) -> f64 {
        (1.0 - self.target).max(f64::EPSILON)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AlertPolicy {
    pub fast_burn: f64,
    pub medium_burn: f64,
    pub slow_burn: f64,
}

impl Default for AlertPolicy {
    fn default() -> Self {
        Self {
            fast_burn: 14.4,
            medium_burn: 6.0,
            slow_burn: 1.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SloConfig {
    pub version: u32,
    pub window_days: u32,
    pub owner: String,
    /// Where the numbers came from, shown in the panel header.
    pub source: String,
    pub alert: AlertPolicy,
    pub objectives: Vec<Objective>,
}

impl SloConfig {
    /// Defaults mirror `ops/slo.yaml`; used when the file is not deployed
    /// (for example a bare binary without the repository checkout).
    pub fn defaults() -> Self {
        Self {
            version: 1,
            window_days: 30,
            owner: "platform-oncall".to_string(),
            source: "built-in defaults (ops/slo.yaml not found)".to_string(),
            alert: AlertPolicy::default(),
            objectives: vec![
                Objective {
                    id: "api_p95".to_string(),
                    metric: MetricKind::LatencyHistogram,
                    target: 0.95,
                    threshold_ms: Some(500.0),
                    runbook:
                        "botbook/src/10-configuration-deployment/operations/runbook-disk-and-queue.md"
                            .to_string(),
                    panel: "ops-endpoints".to_string(),
                },
                Objective {
                    id: "error_rate".to_string(),
                    metric: MetricKind::ErrorRate,
                    target: 0.99,
                    threshold_ms: None,
                    runbook: "botbook/src/10-configuration-deployment/operations/runbook-deploy-rollback.md"
                        .to_string(),
                    panel: "ops-endpoints".to_string(),
                },
                Objective {
                    id: "first_token_p95".to_string(),
                    metric: MetricKind::LlmFirstToken,
                    target: 0.95,
                    threshold_ms: Some(2_000.0),
                    runbook:
                        "botbook/src/10-configuration-deployment/operations/runbook-llm-outage.md"
                            .to_string(),
                    panel: "ops-cost".to_string(),
                },
                Objective {
                    id: "answer_p95".to_string(),
                    metric: MetricKind::LlmAnswer,
                    target: 0.95,
                    threshold_ms: Some(15_000.0),
                    runbook:
                        "botbook/src/10-configuration-deployment/operations/runbook-llm-outage.md"
                            .to_string(),
                    panel: "ops-cost".to_string(),
                },
                Objective {
                    id: "ws_handshake".to_string(),
                    metric: MetricKind::WsHandshake,
                    target: 0.99,
                    threshold_ms: None,
                    runbook:
                        "botbook/src/10-configuration-deployment/operations/runbook-disable-tenant.md"
                            .to_string(),
                    panel: "ops-traffic".to_string(),
                },
            ],
        }
    }

    /// Resolves the SLO file and parses it, falling back to [`Self::defaults`].
    pub fn load() -> Self {
        for candidate in candidate_paths() {
            if !Path::new(&candidate).is_file() {
                continue;
            }
            match std::fs::read_to_string(&candidate) {
                Ok(text) => match Self::parse(&text) {
                    Some(mut config) => {
                        config.source = candidate;
                        return config;
                    }
                    None => log::error!("ops/slo.yaml at {candidate} has no objective entries"),
                },
                Err(error) => log::error!("ops/slo.yaml at {candidate} could not be read: {error}"),
            }
        }
        Self::defaults()
    }

    /// Parses the flat key/value subset documented at the top of this module.
    /// Returns `None` when no objective survived parsing.
    pub fn parse(text: &str) -> Option<Self> {
        let mut flat: BTreeMap<String, String> = BTreeMap::new();
        for raw in text.lines() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((key, value)) = line.split_once(':') else {
                continue;
            };
            flat.insert(key.trim().to_string(), value.trim().to_string());
        }

        let mut grouped: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
        for (key, value) in &flat {
            if let Some(rest) = key.strip_prefix("objective.") {
                if let Some((id, field)) = rest.rsplit_once('.') {
                    grouped
                        .entry(id.to_string())
                        .or_default()
                        .insert(field.to_string(), value.clone());
                }
            }
        }

        let mut objectives = Vec::new();
        for (id, fields) in grouped {
            let metric = match fields.get("metric").and_then(|m| MetricKind::parse(m)) {
                Some(metric) => metric,
                None => {
                    log::error!("SLO objective {id} has no recognised metric, skipped");
                    continue;
                }
            };
            let target = match fields.get("target").and_then(|t| t.parse::<f64>().ok()) {
                Some(target) if target > 0.0 && target < 1.0 => target,
                _ => {
                    log::error!("SLO objective {id} has no target in (0,1), skipped");
                    continue;
                }
            };
            objectives.push(Objective {
                id,
                metric,
                target,
                threshold_ms: fields
                    .get("threshold_ms")
                    .and_then(|v| v.parse::<f64>().ok()),
                runbook: fields.get("runbook").cloned().unwrap_or_default(),
                panel: fields.get("panel").cloned().unwrap_or_default(),
            });
        }

        if objectives.is_empty() {
            return None;
        }

        let defaults = Self::defaults();
        let number = |key: &str, fallback: f64| {
            flat.get(key)
                .and_then(|v| v.parse::<f64>().ok())
                .unwrap_or(fallback)
        };

        Some(Self {
            version: flat
                .get("version")
                .and_then(|v| v.parse::<u32>().ok())
                .unwrap_or(defaults.version),
            window_days: flat
                .get("window_days")
                .and_then(|v| v.parse::<u32>().ok())
                .unwrap_or(defaults.window_days),
            owner: flat
                .get("owner")
                .cloned()
                .unwrap_or(defaults.owner.clone()),
            source: "ops/slo.yaml".to_string(),
            alert: AlertPolicy {
                fast_burn: number("alert.fast_burn", defaults.alert.fast_burn),
                medium_burn: number("alert.medium_burn", defaults.alert.medium_burn),
                slow_burn: number("alert.slow_burn", defaults.alert.slow_burn),
            },
            objectives,
        })
    }
}

fn candidate_paths() -> Vec<String> {
    let mut paths = Vec::new();
    if let Ok(explicit) = std::env::var("GB_OPS_SLO_PATH") {
        paths.push(explicit);
    }
    paths.push("ops/slo.yaml".to_string());
    paths.push("../ops/slo.yaml".to_string());
    paths.push("/opt/gbo/ops/slo.yaml".to_string());
    paths
}

/// Burn rate = observed bad ratio / error budget. `None` means "no data", which
/// the panel renders as such instead of a fake zero.
pub fn burn_rate(bad_ratio: Option<f64>, objective: &Objective) -> Option<f64> {
    bad_ratio.map(|bad| bad / objective.budget_ratio())
}

/// Severity of a burn rate under the multi-window alert policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BurnSeverity {
    Ok,
    Slow,
    Medium,
    Fast,
    NoData,
}

impl BurnSeverity {
    pub fn css_class(self) -> &'static str {
        match self {
            Self::Ok => "ops-ok",
            Self::Slow => "ops-warn",
            Self::Medium => "ops-warn",
            Self::Fast => "ops-page",
            Self::NoData => "ops-nodata",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Ok => "within budget",
            Self::Slow => "slow burn",
            Self::Medium => "medium burn",
            Self::Fast => "fast burn",
            Self::NoData => "no data",
        }
    }
}

/// Applies the three burn-rate thresholds to one window.
pub fn severity(burn: Option<f64>, alert: AlertPolicy) -> BurnSeverity {
    match burn {
        None => BurnSeverity::NoData,
        Some(value) if value >= alert.fast_burn => BurnSeverity::Fast,
        Some(value) if value >= alert.medium_burn => BurnSeverity::Medium,
        Some(value) if value >= alert.slow_burn => BurnSeverity::Slow,
        Some(_) => BurnSeverity::Ok,
    }
}

/// Runbook base URL so the panel can deep-link without hardcoding a host.
pub fn runbook_base_url() -> String {
    std::env::var("GB_RUNBOOK_BASE_URL").unwrap_or_else(|_| {
        "https://github.com/generalbots/generalbots/blob/main".to_string()
    })
}

#[cfg(test)]
#[path = "ops_slo_tests.rs"]
mod tests;

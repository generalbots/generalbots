//! `telemetry::query` — split per #1443 (AGENTS.md 450-line rule).

use super::*;

pub struct VibeTelemetry {
    pub(crate) events: RwLock<std::collections::VecDeque<VibeTelemetryEvent>>,
    pub(crate) run_metrics: RwLock<HashMap<Uuid, RunMetrics>>,
    pub(crate) pool: Option<crate::types::DbPool>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct RunMetrics {
    pub(crate) total_tool_calls: u32,
    pub(crate) successful_tool_calls: u32,
    pub(crate) failed_tool_calls: u32,
    pub(crate) total_latency_ms: u64,
    pub(crate) total_tokens: u64,
    pub(crate) total_cost: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VibeRunMetricsSummary {
    pub run_id: Uuid,
    pub use_case: VibeUseCase,
    pub total_tool_calls: u32,
    pub successful_tool_calls: u32,
    pub failed_tool_calls: u32,
    pub avg_latency_ms: f64,
    pub total_tokens: u64,
    pub total_cost: f64,
    pub success_rate: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VibeGlobalMetrics {
    pub total_runs: u64,
    pub completed_runs: u64,
    pub failed_runs: u64,
    pub total_tool_calls: u64,
    pub avg_latency_ms: f64,
    pub total_cost: f64,
    pub by_use_case: HashMap<VibeUseCase, UseCaseMetrics>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct UseCaseMetrics {
    pub total_runs: u64,
    pub completed_runs: u64,
    pub failed_runs: u64,
    pub total_tool_calls: u64,
    pub avg_latency_ms: f64,
    pub total_cost: f64,
}

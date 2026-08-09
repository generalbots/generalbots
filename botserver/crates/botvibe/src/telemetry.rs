//! Telemetry module for Vibe run instrumentation.
//!
//! Records events, aggregates metrics per run and globally, and provides
//! query endpoints for dashboards. Uses in-memory storage with a cap of
//! 50 000 events.

use crate::types::{VibeRun, VibeTelemetryEvent, VibeTelemetryEventType, VibeUseCase};
use serde::Deserialize;
use std::collections::HashMap;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Structured record for logging a tool call execution.
///
/// Replaces the previous 8-parameter signature to keep the call site
/// readable and to support optional fields cleanly.
#[derive(Debug, Clone, Deserialize)]
pub struct ToolCallRecord {
    /// ID of the Vibe run this tool call belongs to.
    pub run_id: Uuid,
    /// Use case context (SoftwareDevelopment, CustomerSupport, etc.).
    pub use_case: VibeUseCase,
    /// Name of the tool that was invoked.
    pub tool_name: String,
    /// Execution latency in milliseconds.
    pub latency_ms: u64,
    /// Token count if available (LLM tools only).
    pub tokens: Option<u32>,
    /// Estimated cost in USD for the tool call.
    pub cost: f64,
    /// Whether the tool call completed without error.
    pub success: bool,
    /// Error message if the tool call failed.
    pub error: Option<String>,
}

const MAX_EVENTS: usize = 50000;

pub struct VibeTelemetry {
    events: RwLock<Vec<VibeTelemetryEvent>>,
    run_metrics: RwLock<HashMap<Uuid, RunMetrics>>,
}

#[derive(Debug, Clone, Default)]
struct RunMetrics {
    total_tool_calls: u32,
    successful_tool_calls: u32,
    failed_tool_calls: u32,
    total_latency_ms: u64,
    total_tokens: u64,
    total_cost: f64,
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

impl VibeTelemetry {
    pub fn new() -> Self {
        Self {
            events: RwLock::new(Vec::new()),
            run_metrics: RwLock::new(HashMap::new()),
        }
    }

    pub async fn record(&self, event: VibeTelemetryEvent) {
        let run_id = event.run_id;
        let success = event.success;
        let latency = event.latency_ms;
        let tokens = event.tokens_used.unwrap_or(0) as u64;
        let cost = event.estimated_cost;
        let use_case = event.use_case;
        let is_tool = matches!(
            event.event_type,
            VibeTelemetryEventType::ToolCallCompleted | VibeTelemetryEventType::ToolCallFailed
        );
        let is_run_end = matches!(
            event.event_type,
            VibeTelemetryEventType::RunCompleted | VibeTelemetryEventType::RunFailed
        );

        {
            let mut metrics = self.run_metrics.write().await;
            let m = metrics.entry(run_id).or_default();
            if is_tool {
                m.total_tool_calls += 1;
                if success {
                    m.successful_tool_calls += 1;
                } else {
                    m.failed_tool_calls += 1;
                }
            }
            m.total_latency_ms += latency;
            m.total_tokens += tokens;
            m.total_cost += cost;
        }

        {
            let mut events = self.events.write().await;
            events.push(event);
            if events.len() > MAX_EVENTS {
                events.drain(0..5000);
            }
        }

        if is_run_end {
            let _ = use_case;
            let mut metrics = self.run_metrics.write().await;
            metrics.remove(&run_id);
        }
    }

    pub async fn record_run_start(&self, run: &VibeRun) {
        let event = VibeTelemetryEvent {
            event_id: Uuid::new_v4(),
            run_id: run.run_id,
            event_type: VibeTelemetryEventType::RunStarted,
            tool_name: None,
            use_case: run.use_case,
            latency_ms: 0,
            tokens_used: None,
            estimated_cost: 0.0,
            success: true,
            error: None,
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
        };
        self.record(event).await;
    }

    pub async fn record_run_completion(&self, run: &VibeRun, latency_ms: u64, tokens: Option<u32>, cost: f64) {
        let event = VibeTelemetryEvent {
            event_id: Uuid::new_v4(),
            run_id: run.run_id,
            event_type: if run.state == crate::types::VibeRunState::Completed {
                VibeTelemetryEventType::RunCompleted
            } else {
                VibeTelemetryEventType::RunFailed
            },
            tool_name: None,
            use_case: run.use_case,
            latency_ms,
            tokens_used: tokens,
            estimated_cost: cost,
            success: run.state == crate::types::VibeRunState::Completed,
            error: run.error.clone(),
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
        };
        self.record(event).await;
    }

    pub async fn record_tool_call(&self, record: ToolCallRecord) {
        let event = VibeTelemetryEvent {
            event_id: Uuid::new_v4(),
            run_id: record.run_id,
            event_type: if record.success {
                VibeTelemetryEventType::ToolCallCompleted
            } else {
                VibeTelemetryEventType::ToolCallFailed
            },
            tool_name: Some(record.tool_name),
            use_case: record.use_case,
            latency_ms: record.latency_ms,
            tokens_used: record.tokens,
            estimated_cost: record.cost,
            success: record.success,
            error: record.error,
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
        };
        self.record(event).await;
    }

    pub async fn get_run_metrics(&self, run_id: Uuid) -> Option<VibeRunMetricsSummary> {
        let events = self.events.read().await;
        let run_events: Vec<&VibeTelemetryEvent> = events.iter().filter(|e| e.run_id == run_id).collect();
        if run_events.is_empty() {
            return None;
        }
        if run_events.iter().any(|e| {
            matches!(
                e.event_type,
                VibeTelemetryEventType::RunCompleted | VibeTelemetryEventType::RunFailed
            )
        }) {
            return None;
        }

        let use_case = run_events[0].use_case;
        let mut total_tool_calls = 0u32;
        let mut successful = 0u32;
        let mut failed = 0u32;
        let mut total_latency = 0u64;
        let mut total_tokens = 0u64;
        let mut total_cost = 0.0;

        for e in &run_events {
            match e.event_type {
                VibeTelemetryEventType::ToolCallCompleted => {
                    total_tool_calls += 1;
                    successful += 1;
                }
                VibeTelemetryEventType::ToolCallFailed => {
                    total_tool_calls += 1;
                    failed += 1;
                }
                _ => {}
            }
            total_latency += e.latency_ms;
            total_tokens += e.tokens_used.unwrap_or(0) as u64;
            total_cost += e.estimated_cost;
        }

        let count = run_events.len().max(1);
        Some(VibeRunMetricsSummary {
            run_id,
            use_case,
            total_tool_calls,
            successful_tool_calls: successful,
            failed_tool_calls: failed,
            avg_latency_ms: total_latency as f64 / count as f64,
            total_tokens,
            total_cost,
            success_rate: if total_tool_calls > 0 {
                successful as f64 / total_tool_calls as f64
            } else {
                1.0
            },
        })
    }

    pub async fn get_global_metrics(&self) -> VibeGlobalMetrics {
        let events = self.events.read().await;
        let mut by_use_case: HashMap<VibeUseCase, UseCaseMetrics> = HashMap::new();
        let mut total_runs = 0u64;
        let mut completed_runs = 0u64;
        let mut failed_runs = 0u64;
        let mut total_tool_calls = 0u64;
        let mut total_latency = 0u64;
        let mut total_cost = 0.0;
        let mut event_count = 0usize;

        for e in events.iter() {
            event_count += 1;
            total_latency += e.latency_ms;
            total_cost += e.estimated_cost;

            let m = by_use_case.entry(e.use_case).or_default();

            match e.event_type {
                VibeTelemetryEventType::RunStarted => {
                    total_runs += 1;
                    m.total_runs += 1;
                }
                VibeTelemetryEventType::RunCompleted => {
                    completed_runs += 1;
                    m.completed_runs += 1;
                }
                VibeTelemetryEventType::RunFailed => {
                    failed_runs += 1;
                    m.failed_runs += 1;
                }
                VibeTelemetryEventType::ToolCallCompleted | VibeTelemetryEventType::ToolCallFailed => {
                    total_tool_calls += 1;
                    m.total_tool_calls += 1;
                }
                _ => {}
            }
            m.total_cost += e.estimated_cost;
        }

        let avg_latency = if event_count > 0 {
            total_latency as f64 / event_count as f64
        } else {
            0.0
        };

        for m in by_use_case.values_mut() {
            m.avg_latency_ms = avg_latency;
        }

        VibeGlobalMetrics {
            total_runs,
            completed_runs,
            failed_runs,
            total_tool_calls,
            avg_latency_ms: avg_latency,
            total_cost,
            by_use_case,
        }
    }

    pub async fn get_events_for_run(&self, run_id: Uuid, limit: usize) -> Vec<VibeTelemetryEvent> {
        let events = self.events.read().await;
        events.iter().rev().filter(|e| e.run_id == run_id).take(limit).cloned().collect()
    }
}

impl Default for VibeTelemetry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(run_id: Uuid, event_type: VibeTelemetryEventType, success: bool, latency: u64, cost: f64) -> VibeTelemetryEvent {
        VibeTelemetryEvent {
            event_id: Uuid::new_v4(),
            run_id,
            event_type,
            tool_name: None,
            use_case: VibeUseCase::SoftwareDevelopment,
            latency_ms: latency,
            tokens_used: Some(10),
            estimated_cost: cost,
            success,
            error: None,
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn record_and_metrics_aggregate_correctly() {
        let telemetry = VibeTelemetry::new();
        let run_id = Uuid::new_v4();

        telemetry.record(event(run_id, VibeTelemetryEventType::RunStarted, true, 0, 0.0)).await;
        telemetry.record(event(run_id, VibeTelemetryEventType::ToolCallCompleted, true, 10, 0.5)).await;
        telemetry.record(event(run_id, VibeTelemetryEventType::ToolCallCompleted, true, 20, 0.5)).await;
        telemetry.record(event(run_id, VibeTelemetryEventType::ToolCallFailed, false, 5, 0.1)).await;

        let m = telemetry.get_run_metrics(run_id).await.unwrap();
        assert_eq!(m.total_tool_calls, 3);
        assert_eq!(m.successful_tool_calls, 2);
        assert_eq!(m.failed_tool_calls, 1);
        assert_eq!(m.total_tokens, 40);
        assert_eq!(m.total_cost, 1.1);
        assert!((m.success_rate - 2.0 / 3.0).abs() < 1e-9);
    }

    #[tokio::test]
    async fn run_end_removes_in_flight_metrics() {
        let telemetry = VibeTelemetry::new();
        let run_id = Uuid::new_v4();
        telemetry.record(event(run_id, VibeTelemetryEventType::RunStarted, true, 0, 0.0)).await;
        telemetry.record(event(run_id, VibeTelemetryEventType::RunCompleted, true, 100, 2.0)).await;
        assert!(telemetry.get_run_metrics(run_id).await.is_none());
    }

    #[tokio::test]
    async fn global_metrics_tally_runs_and_tools() {
        let telemetry = VibeTelemetry::new();
        let run_id = Uuid::new_v4();
        telemetry.record(event(run_id, VibeTelemetryEventType::RunStarted, true, 0, 0.0)).await;
        telemetry.record(event(run_id, VibeTelemetryEventType::RunCompleted, true, 30, 1.0)).await;
        telemetry.record(event(Uuid::new_v4(), VibeTelemetryEventType::RunStarted, true, 0, 0.0)).await;
        telemetry.record(event(Uuid::new_v4(), VibeTelemetryEventType::RunFailed, false, 5, 0.2)).await;
        telemetry.record(event(Uuid::new_v4(), VibeTelemetryEventType::ToolCallCompleted, true, 4, 0.1)).await;

        let g = telemetry.get_global_metrics().await;
        assert_eq!(g.total_runs, 2);
        assert_eq!(g.completed_runs, 1);
        assert_eq!(g.failed_runs, 1);
        assert_eq!(g.total_tool_calls, 1);
        assert!((g.total_cost - 1.3).abs() < 1e-9);
        assert_eq!(g.by_use_case.len(), 1);
    }

    #[tokio::test]
    async fn empty_telemetry_has_no_run_metrics() {
        let telemetry = VibeTelemetry::new();
        assert!(telemetry.get_run_metrics(Uuid::new_v4()).await.is_none());
        let g = telemetry.get_global_metrics().await;
        assert_eq!(g.total_runs, 0);
        assert!(g.by_use_case.is_empty());
    }

    #[tokio::test]
    async fn events_for_run_are_newest_first_and_limited() {
        let telemetry = VibeTelemetry::new();
        let run_id = Uuid::new_v4();
        for _ in 0..5 {
            telemetry.record(event(run_id, VibeTelemetryEventType::ToolCallCompleted, true, 1, 0.0)).await;
        }
        let events = telemetry.get_events_for_run(run_id, 2).await;
        assert_eq!(events.len(), 2);
        let all = telemetry.get_events_for_run(run_id, 100).await;
        assert_eq!(all.len(), 5);
    }

    #[tokio::test]
    async fn record_helpers_create_expected_event_types() {
        let telemetry = VibeTelemetry::new();
        let mut run = crate::types::VibeRun::new(Uuid::nil(), Uuid::nil(), Uuid::nil(), "i".into(), crate::types::VibeRunConfig::default());
        let run_id = run.run_id;

        telemetry.record_run_start(&run).await;
        run.transition(crate::types::VibeRunState::Completed);
        telemetry.record_run_completion(&run, 42, Some(100), 0.7).await;
        telemetry.record_tool_call(ToolCallRecord {
            run_id,
            use_case: VibeUseCase::SoftwareDevelopment,
            tool_name: "web/search".into(),
            latency_ms: 9,
            tokens: Some(20),
            cost: 0.01,
            success: true,
            error: None,
        }).await;

        let events = telemetry.get_events_for_run(run_id, 100).await;
        assert_eq!(events.len(), 3);
        assert!(events.iter().any(|e| e.event_type == VibeTelemetryEventType::RunStarted));
        assert!(events.iter().any(|e| e.event_type == VibeTelemetryEventType::RunCompleted));
        assert!(events.iter().any(|e| e.event_type == VibeTelemetryEventType::ToolCallCompleted && e.tool_name.as_deref() == Some("web/search")));
    }
}

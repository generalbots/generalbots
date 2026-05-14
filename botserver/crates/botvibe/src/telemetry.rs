use crate::types::{VibeRun, VibeTelemetryEvent, VibeTelemetryEventType, VibeUseCase};
use std::collections::HashMap;
use tokio::sync::RwLock;
use uuid::Uuid;

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

    pub async fn record_tool_call(
        &self,
        run_id: Uuid,
        use_case: VibeUseCase,
        tool_name: &str,
        latency_ms: u64,
        tokens: Option<u32>,
        cost: f64,
        success: bool,
        error: Option<String>,
    ) {
        let event = VibeTelemetryEvent {
            event_id: Uuid::new_v4(),
            run_id,
            event_type: if success {
                VibeTelemetryEventType::ToolCallCompleted
            } else {
                VibeTelemetryEventType::ToolCallFailed
            },
            tool_name: Some(tool_name.to_string()),
            use_case,
            latency_ms,
            tokens_used: tokens,
            estimated_cost: cost,
            success,
            error,
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

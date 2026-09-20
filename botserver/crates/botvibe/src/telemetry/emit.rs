//! `telemetry::emit` — split per #1443 (AGENTS.md 450-line rule).

use super::*;

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
    /// Extra structured facts about the call (e.g. `input_tokens` /
    /// `output_tokens` from the provider usage payload). Persisted into the
    /// telemetry `metadata` column; empty for non-LLM calls.
    pub metadata: std::collections::HashMap<String, String>,
}

impl VibeTelemetry {
    pub fn new() -> Self {
        Self {
            events: RwLock::new(std::collections::VecDeque::new()),
            run_metrics: RwLock::new(HashMap::new()),
            pool: None,
        }
    }

    /// #921 — constructs telemetry with write-through persistence so events
    /// and dashboards survive a restart. Hydrates the most recent events from
    /// `vibe_telemetry` on construction; the in-memory vec remains the live
    /// authority.
    pub fn with_persistence(pool: crate::types::DbPool) -> Self {
        let events = crate::catalog_persistence::load_telemetry_events(&pool, MAX_EVENTS as i64)
            .unwrap_or_else(|e| {
                log::warn!("telemetry hydrate failed: {e}");
                Vec::new()
            });
        Self {
            events: RwLock::new(events.into()),
            run_metrics: RwLock::new(HashMap::new()),
            pool: Some(pool),
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
            // #1446 — VecDeque keeps the compaction O(k): drain from the
            // front instead of shifting the remaining 45k elements.
            events.push_back(event.clone());
            if events.len() > MAX_EVENTS {
                events.drain(0..5000);
            }
        }

        // #921 — write-through persistence. Telemetry is non-critical and the
        // `vibe_telemetry.run_id` FK may reject an event whose run has not
        // been persisted yet, so a failed insert is logged (not propagated).
        if let Some(pool) = &self.pool {
            if let Err(e) = crate::catalog_persistence::save_telemetry_event(pool, &event) {
                log::debug!("telemetry persist skipped: {e}");
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
            metadata: record.metadata.clone(),
        };
        self.record(event).await;
    }

    pub async fn get_run_metrics(&self, run_id: Uuid) -> Option<VibeRunMetricsSummary> {
        let events = self.events.read().await;
        let run_events: Vec<&VibeTelemetryEvent> = events.iter().filter(|e| e.run_id == run_id).collect();
        if run_events.is_empty() {
            return None;
        }
        // #1268 — keep per-run metrics available after the run ends: the
        // frontend queries /api/vibe/metrics/{run_id} exactly when the run
        // reaches a terminal state, so dropping the summary here made the
        // run dock budget and Metrics window show nothing for finished runs.

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

        // #1446 — real per-use-case latency: the global average used to be
        // copied into every bucket, which made the metric meaningless.
        let mut uc_latency: HashMap<VibeUseCase, (u64, usize)> = HashMap::new();
        for e in events.iter() {
            let entry = uc_latency.entry(e.use_case).or_default();
            entry.0 += e.latency_ms;
            entry.1 += 1;
        }
        for (uc, m) in by_use_case.iter_mut() {
            m.avg_latency_ms = uc_latency
                .get(uc)
                .map(|(sum, n)| {
                    if *n > 0 {
                        *sum as f64 / *n as f64
                    } else {
                        0.0
                    }
                })
                .unwrap_or(0.0);
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

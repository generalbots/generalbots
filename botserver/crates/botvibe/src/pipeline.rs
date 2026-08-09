//! Run pipeline orchestration for the Vibe platform (Issue #805).
//!
//! A run is executed as a sequence of named stages (classify intent,
//! compile plan, execute plan). The pipeline engine drives the stages
//! through the tool registry, records per-stage telemetry and returns a
//! structured report. Stage outcomes mirror the registered tools, so a
//! not-yet-wired tool surfaces as an honest stage failure instead of a
//! silent success.

use crate::telemetry::{ToolCallRecord, VibeTelemetry};
use crate::tool_executor::VibeToolExecutor;
use crate::types::{VibeState, VibeToolCall, VibeUseCase};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// The kind of work a pipeline stage performs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PipelineStageKind {
    ClassifyIntent,
    CompilePlan,
    ExecutePlan,
}

impl PipelineStageKind {
    /// Tool registered in the registry backing this stage.
    pub fn tool_name(&self) -> &'static str {
        match self {
            Self::ClassifyIntent => "classify_intent",
            Self::CompilePlan => "compile_plan",
            Self::ExecutePlan => "execute_plan",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::ClassifyIntent => "Intent classification",
            Self::CompilePlan => "Plan compilation",
            Self::ExecutePlan => "Plan execution",
        }
    }
}

/// A single runnable stage of a pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStage {
    pub id: String,
    pub name: String,
    pub kind: PipelineStageKind,
    pub timeout_secs: u64,
}

/// Ordered pipeline definition for a use case.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunPipeline {
    pub pipeline_id: String,
    pub use_case: VibeUseCase,
    pub stages: Vec<PipelineStage>,
}

impl RunPipeline {
    /// Default three-stage pipeline, identical for every use case.
    pub fn for_use_case(use_case: VibeUseCase) -> Self {
        Self {
            pipeline_id: format!("default/{}", use_case_str(use_case)),
            use_case,
            stages: vec![
                stage("intent", PipelineStageKind::ClassifyIntent, 30),
                stage("plan", PipelineStageKind::CompilePlan, 30),
                stage("execute", PipelineStageKind::ExecutePlan, 300),
            ],
        }
    }

    pub fn stage(&self, id: &str) -> Option<&PipelineStage> {
        self.stages.iter().find(|s| s.id == id)
    }
}

fn stage(id: &str, kind: PipelineStageKind, timeout_secs: u64) -> PipelineStage {
    PipelineStage {
        id: id.to_string(),
        name: kind.display_name().to_string(),
        kind,
        timeout_secs,
    }
}

/// Final outcome of a single pipeline stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StageStatus {
    Completed,
    Failed,
}

/// Per-stage result within a pipeline run report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStageReport {
    pub stage_id: String,
    pub stage_name: String,
    pub tool_name: String,
    pub status: StageStatus,
    pub took_ms: u64,
    pub error: Option<String>,
}

/// Full report of one pipeline execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineRunReport {
    pub pipeline_id: String,
    pub use_case: String,
    pub run_id: Uuid,
    pub stages: Vec<PipelineStageReport>,
}

/// Drives pipeline stages through the tool executor with telemetry.
pub struct PipelineEngine {
    telemetry: Arc<VibeTelemetry>,
}

impl PipelineEngine {
    pub fn new(telemetry: Arc<VibeTelemetry>) -> Self {
        Self { telemetry }
    }

    pub async fn run(
        &self,
        pipeline: &RunPipeline,
        run_id: Uuid,
        use_case: VibeUseCase,
        executor: &VibeToolExecutor,
        state: &dyn VibeState,
    ) -> PipelineRunReport {
        let mut reports = Vec::new();
        for stage in &pipeline.stages {
            let start = std::time::Instant::now();
            let tool_name = stage.kind.tool_name();
            let mut tool_call = VibeToolCall::new(
                run_id,
                tool_name.to_string(),
                serde_json::json!({}),
                false,
            );
            // The engine is the authorized orchestrator: approval policy is
            // decided upstream (e.g. the agent loop), not per stage here.
            tool_call.approved = true;
            let outcome = executor.execute(&mut tool_call, use_case, state).await;
            let took_ms = start.elapsed().as_millis() as u64;
            let (status, error) = stage_outcome(&outcome, tool_call.result.as_ref().map(|r| r.success), tool_call.result.as_ref().and_then(|r| r.error.clone()));
            reports.push(PipelineStageReport {
                stage_id: stage.id.clone(),
                stage_name: stage.name.clone(),
                tool_name: tool_name.to_string(),
                status,
                took_ms,
                error: error.clone(),
            });
            self.telemetry
                .record_tool_call(ToolCallRecord {
                    run_id,
                    use_case,
                    tool_name: tool_name.to_string(),
                    latency_ms: took_ms,
                    tokens: None,
                    cost: 0.0,
                    success: status == StageStatus::Completed,
                    error,
                })
                .await;
        }
        PipelineRunReport {
            pipeline_id: pipeline.pipeline_id.clone(),
            use_case: use_case_str(use_case).to_string(),
            run_id,
            stages: reports,
        }
    }
}

/// Maps the executor outcome plus the handler result into a stage status.
fn stage_outcome(
    outcome: &Result<(), String>,
    result_success: Option<bool>,
    result_error: Option<String>,
) -> (StageStatus, Option<String>) {
    match outcome {
        Err(e) => (StageStatus::Failed, Some(e.clone())),
        Ok(()) if result_success == Some(true) => (StageStatus::Completed, None),
        Ok(()) => (StageStatus::Failed, result_error),
    }
}

fn use_case_str(use_case: VibeUseCase) -> &'static str {
    match use_case {
        VibeUseCase::SoftwareDevelopment => "software_development",
        VibeUseCase::CustomerSupport => "customer_support",
        VibeUseCase::FinancialAnalysis => "financial_analysis",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::VibeRun;
    use std::collections::HashMap;
    use tokio::sync::RwLock;

    #[test]
    fn default_pipeline_has_three_ordered_stages() {
        for use_case in [
            VibeUseCase::SoftwareDevelopment,
            VibeUseCase::CustomerSupport,
            VibeUseCase::FinancialAnalysis,
        ] {
            let pipeline = RunPipeline::for_use_case(use_case);
            assert_eq!(pipeline.stages.len(), 3);
            assert_eq!(pipeline.stages[0].id, "intent");
            assert_eq!(pipeline.stages[1].kind, PipelineStageKind::CompilePlan);
            assert_eq!(pipeline.stages[2].kind, PipelineStageKind::ExecutePlan);
            assert_eq!(pipeline.stage("plan").expect("stage found").name, "Plan compilation");
            assert!(pipeline.stage("missing").is_none());
            assert!(pipeline.pipeline_id.contains(use_case_str(use_case)));
        }
    }

    #[test]
    fn stage_outcome_maps_execution_results() {
        assert_eq!(stage_outcome(&Err("rejected".into()), None, None).0, StageStatus::Failed);
        let (status, error) = stage_outcome(&Ok(()), Some(true), None);
        assert_eq!(status, StageStatus::Completed);
        assert!(error.is_none());
        let (status, error) = stage_outcome(&Ok(()), Some(false), Some("not wired up yet".into()));
        assert_eq!(status, StageStatus::Failed);
        assert_eq!(error.as_deref(), Some("not wired up yet"));
        let (status, _) = stage_outcome(&Ok(()), None, None);
        assert_eq!(status, StageStatus::Failed);
    }

    struct MockState {
        runs: Arc<RwLock<HashMap<Uuid, VibeRun>>>,
    }

    impl MockState {
        fn new() -> Self {
            Self { runs: Arc::new(RwLock::new(HashMap::new())) }
        }
    }

    impl VibeState for MockState {
        fn db_pool(&self) -> &crate::types::DbPool {
            unreachable!("db_pool not exercised in pipeline tests")
        }
        fn broadcast_progress(&self, _event: crate::types::VibeProgressEvent) {}
        fn progress_sender(&self) -> Option<&tokio::sync::broadcast::Sender<crate::types::VibeProgressEvent>> {
            None
        }
        fn active_runs(&self) -> &Arc<RwLock<HashMap<Uuid, crate::types::VibeRun>>> {
            &self.runs
        }
        fn run_signal_sender(&self) -> Option<&tokio::sync::broadcast::Sender<crate::types::VibeRunSignal>> {
            None
        }
    }

    #[tokio::test]
    async fn engine_runs_all_stages_and_reports_failures_honestly() {
        let telemetry = Arc::new(VibeTelemetry::new());
        let executor = Arc::new(VibeToolExecutor::new(Arc::new(crate::tool_executor::ToolRegistry::new())));
        let engine = PipelineEngine::new(telemetry.clone());
        let pipeline = RunPipeline::for_use_case(VibeUseCase::SoftwareDevelopment);
        let run_id = Uuid::new_v4();
        let report = engine
            .run(&pipeline, run_id, VibeUseCase::SoftwareDevelopment, &executor, &MockState::new())
            .await;
        assert_eq!(report.stages.len(), 3);
        assert_eq!(report.run_id, run_id);
        for stage in &report.stages {
            assert!(!stage.tool_name.is_empty());
            assert_eq!(stage.status, StageStatus::Failed, "autotask tools are stubs -> honest failure");
            assert!(stage.error.as_deref().unwrap_or_default().contains("not wired up"));
        }
        let metrics = telemetry.get_run_metrics(run_id).await.expect("metrics recorded");
        assert_eq!(metrics.total_tool_calls, 3);
    }
}
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
    BuildTest,
    /// #1271 — snapshot the currently deployed commit into a
    /// `release/prev-<ts>` branch before the new state is committed, so the
    /// toolbar branch combo offers a rollback point to re-deploy.
    SnapshotPrevious,
    CommitPush,
    PublishApp,
    BindDomain,
    VerifyDomain,
    IssueTls,
}

impl PipelineStageKind {
    /// Tool registered in the registry backing this stage.
    pub fn tool_name(&self) -> &'static str {
        match self {
            Self::ClassifyIntent => "classify_intent",
            Self::CompilePlan => "compile_plan",
            Self::ExecutePlan => "execute_plan",
            Self::BuildTest => "test/run",
            Self::SnapshotPrevious => "git/snapshot-previous",
            Self::CommitPush => "git/commit",
            Self::PublishApp => "publish/project",
            Self::BindDomain => "domain/bind",
            Self::VerifyDomain => "domain/verify",
            Self::IssueTls => "domain/tls",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::ClassifyIntent => "Intent classification",
            Self::CompilePlan => "Plan compilation",
            Self::ExecutePlan => "Plan execution",
            Self::BuildTest => "Build and test",
            Self::SnapshotPrevious => "Snapshot previous release",
            Self::CommitPush => "Commit and push",
            Self::PublishApp => "Publish application",
            Self::BindDomain => "Bind domain and TLS",
            Self::VerifyDomain => "Verify domain ownership",
            Self::IssueTls => "Issue TLS certificate",
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
    pub requires_approval: bool,
    /// vibe33 #812 — when false (default) a failed stage aborts the pipeline
    /// and the remaining stages are marked Skipped (fail-fast). Set true for
    /// non-critical stages that must not block the rest.
    pub continue_on_failure: bool,
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

    /// Orchestrated build-test-publish pipeline (Issue #805). The last three
    /// stages mutate external state and require per-step human approval.
    pub fn deploy_pipeline(use_case: VibeUseCase) -> Self {
        #[cfg(target_os = "windows")]
        let stages = vec![
            stage_approval("build_test", PipelineStageKind::BuildTest, 300, false),
            stage_approval("publish", PipelineStageKind::PublishApp, 300, true),
        ];
        #[cfg(not(target_os = "windows"))]
        let stages = vec![
            stage("intent", PipelineStageKind::ClassifyIntent, 30),
            stage("plan", PipelineStageKind::CompilePlan, 30),
            stage_approval("build_test", PipelineStageKind::BuildTest, 300, false),
            // Snapshot the current deployment BEFORE the new state is
            // committed so `release/prev-<ts>` points at the version that is
            // being replaced — the rollback point in the branch combo.
            stage("snapshot_prev", PipelineStageKind::SnapshotPrevious, 30),
            stage_approval("commit_push", PipelineStageKind::CommitPush, 60, true),
            stage_approval("publish", PipelineStageKind::PublishApp, 300, true),
            stage_approval("domain", PipelineStageKind::BindDomain, 60, true),
            // #1268 — a bound domain must not stay verified=false/tls=pending
            // forever: verify ownership right after binding, then (re)apply
            // the route so ACME issues on first request. Platform-managed
            // wildcard hosts verify against the platform zone; custom domains
            // keep requiring the manual TXT token. Both stages tolerate
            // failure (verification may legitimately need DNS propagation).
            stage_continue("domain_verify", PipelineStageKind::VerifyDomain, 30),
            stage_continue("domain_tls", PipelineStageKind::IssueTls, 60),
        ];
        Self {
            pipeline_id: format!("deploy/{}", use_case_str(use_case)),
            use_case,
            stages,
        }
    }

    pub fn stage(&self, id: &str) -> Option<&PipelineStage> {
        self.stages.iter().find(|s| s.id == id)
    }
}

fn stage(id: &str, kind: PipelineStageKind, timeout_secs: u64) -> PipelineStage {
    stage_approval(id, kind, timeout_secs, false)
}

/// #1268 — a stage whose failure must not abort the deploy (DNS propagation
/// delays, upstream ACME hiccups): the pipeline continues when it fails.
fn stage_continue(id: &str, kind: PipelineStageKind, timeout_secs: u64) -> PipelineStage {
    PipelineStage {
        id: id.to_string(),
        name: kind.display_name().to_string(),
        kind,
        timeout_secs,
        requires_approval: false,
        continue_on_failure: true,
    }
}

fn stage_approval(
    id: &str,
    kind: PipelineStageKind,
    timeout_secs: u64,
    requires_approval: bool,
) -> PipelineStage {
    PipelineStage {
        id: id.to_string(),
        name: kind.display_name().to_string(),
        kind,
        timeout_secs,
        requires_approval,
        continue_on_failure: false,
    }
}

/// Final outcome of a single pipeline stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StageStatus {
    Completed,
    Failed,
    /// vibe33 #812 — stage not executed because a previous fail-fast stage
    /// aborted the pipeline (e.g. publish skipped when tests failed).
    Skipped,
}

/// How an approval wait resolved (#929). The caller must map each variant to
/// a distinct, honest user-visible verdict instead of collapsing every
/// failure into a hardcoded "Approval denied".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalOutcome {
    Approved,
    Cancelled,
    TimedOut,
    ChannelClosed,
    NotAvailable,
}

impl ApprovalOutcome {
    /// User/telemetry-facing reason for the non-approved outcomes.
    pub fn error_message(self) -> Option<&'static str> {
        match self {
            Self::Approved => None,
            Self::Cancelled => Some("Approval denied"),
            Self::TimedOut => Some("Approval wait timed out"),
            Self::ChannelClosed => Some("Approval channel closed"),
            Self::NotAvailable => Some("Approval channel unavailable"),
        }
    }
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

/// Run-scoped inputs for [`PipelineEngine::run`], bundled to keep the method
/// signature within clippy's argument-count threshold.
pub struct PipelineRunContext<'a> {
    pub run_id: Uuid,
    pub use_case: VibeUseCase,
    pub intent: &'a str,
    pub project_id: Option<&'a str>,
    pub project_name: Option<&'a str>,
    /// #1269 — when the run carries auto_approve (admin-issued), skip the
    /// per-stage human approval gates so headless deploys complete instead
    /// of timing out waiting for a signal nobody sends.
    pub auto_approve: bool,
}

impl PipelineEngine {
    pub fn new(telemetry: Arc<VibeTelemetry>) -> Self {
        Self { telemetry }
    }

    async fn wait_for_approval(&self, state: &dyn VibeState, run_id: Uuid) -> ApprovalOutcome {
        let approval_timeout = std::time::Duration::from_secs(300);
        let start = tokio::time::Instant::now();
        let Some(tx) = state.run_signal_sender() else {
            return ApprovalOutcome::NotAvailable;
        };
        let mut rx = tx.subscribe();
        loop {
            let remaining = if start.elapsed() > approval_timeout {
                return ApprovalOutcome::TimedOut;
            } else {
                approval_timeout.saturating_sub(start.elapsed())
            };
            match tokio::time::timeout(remaining, rx.recv()).await {
                Ok(Ok(crate::types::VibeRunSignal::Approved(id))) if id == run_id => {
                    return ApprovalOutcome::Approved;
                }
                Ok(Ok(crate::types::VibeRunSignal::Cancelled(id))) if id == run_id => {
                    return ApprovalOutcome::Cancelled;
                }
                Ok(Ok(_)) => continue,
                Ok(Err(tokio::sync::broadcast::error::RecvError::Lagged(_))) => continue,
                Ok(Err(tokio::sync::broadcast::error::RecvError::Closed)) => {
                    return ApprovalOutcome::ChannelClosed;
                }
                Err(_) => return ApprovalOutcome::TimedOut,
            }
        }
    }

    pub async fn run(
        &self,
        pipeline: &RunPipeline,
        executor: &VibeToolExecutor,
        state: &dyn VibeState,
        ctx: &PipelineRunContext<'_>,
    ) -> PipelineRunReport {
        let PipelineRunContext {
            run_id,
            use_case,
            intent,
            project_id,
            project_name,
            auto_approve,
        } = *ctx;
        let mut reports = Vec::new();
        for stage in &pipeline.stages {
            let start = std::time::Instant::now();
            let tool_name = stage.kind.tool_name();
            // vibe33 #811/#812 — stage tool args are injected from the run
            // intent for the intent-dependent stages; the project-scoped
            // stages (test/run, git/commit, publish/project, domain/bind)
            // receive the run's project context so validation passes and the
            // tools operate on the right project instead of failing with
            // "Parâmetro obrigatório ausente: 'project'" (issue #8xx).
            let arguments = if matches!(
                stage.kind,
                PipelineStageKind::ClassifyIntent
                    | PipelineStageKind::CompilePlan
                    | PipelineStageKind::ExecutePlan
            ) {
                serde_json::json!({ "intent": intent })
            } else {
                match stage.kind {
                    PipelineStageKind::PublishApp => serde_json::json!({
                        "project_id": project_id.unwrap_or(""),
                        "env": "production",
                    }),
                    PipelineStageKind::BindDomain => serde_json::json!({
                        "project_id": project_id.unwrap_or(""),
                        "env": "production",
                        "domain": format!("{}.gb.solutions", project_name.unwrap_or("app")),
                    }),
                    // #1268 — verify/verify-then-issue for the just-bound
                    // host. domain/verify reads the binding itself (token
                    // already recorded at bind time); domain/tls re-applies
                    // the route so ACME issues on first request.
                    PipelineStageKind::VerifyDomain | PipelineStageKind::IssueTls => {
                        serde_json::json!({
                            "env": "production",
                            "domain": format!("{}.gb.solutions", project_name.unwrap_or("app")),
                        })
                    }
                    PipelineStageKind::BuildTest
                    | PipelineStageKind::SnapshotPrevious
                    | PipelineStageKind::CommitPush => {
                        serde_json::json!({
                            "project": project_name.unwrap_or(""),
                            "project_id": project_id.unwrap_or(""),
                            // git/commit declares 'message' as required; the
                            // stage derives one from the project name so the
                            // deploy pipeline no longer dies with
                            // "Parâmetro obrigatório ausente: 'message'".
                            "message": format!("Deploy {} via deploy pipeline", project_name.unwrap_or("app")),
                        })
                    }
                    _ => serde_json::json!({}),
                }
            };
            let mut tool_call = VibeToolCall::new(
                run_id,
                tool_name.to_string(),
                arguments,
                stage.requires_approval,
            );
            // #1269 — an auto-approved (admin-issued) run skips the human
            // approval gate but still executes the stage tool; only the wait
            // is bypassed, never the work.
            if stage.requires_approval && !auto_approve {
                let outcome = self.wait_for_approval(state, run_id).await;
                if outcome != ApprovalOutcome::Approved {
                    let error = outcome
                        .error_message()
                        .unwrap_or("Approval denied")
                        .to_string();
                    reports.push(PipelineStageReport {
                        stage_id: stage.id.clone(),
                        stage_name: stage.name.clone(),
                        tool_name: tool_name.to_string(),
                        status: StageStatus::Failed,
                        took_ms: 0,
                        error: Some(error.clone()),
                    });
                    self.telemetry
                        .record_tool_call(ToolCallRecord {
                            run_id,
                            use_case,
                            tool_name: tool_name.to_string(),
                            latency_ms: 0,
                            tokens: None,
                            cost: 0.0,
                            success: false,
                            error: Some(error),
                            metadata: std::collections::HashMap::new(),
                        })
                        .await;
                    for rest in &pipeline.stages[reports.len()..] {
                        reports.push(PipelineStageReport {
                            stage_id: rest.id.clone(),
                            stage_name: rest.name.clone(),
                            tool_name: rest.kind.tool_name().to_string(),
                            status: StageStatus::Skipped,
                            took_ms: 0,
                            error: None,
                        });
                    }
                    break;
                }
            }
            // The engine is the authorized orchestrator: approval policy is
            // decided upstream (e.g. the agent loop), not per stage here.
            tool_call.approved = true;
            let outcome = executor.execute(&mut tool_call, use_case, state).await;
            let took_ms = start.elapsed().as_millis() as u64;
            let (status, error) = stage_outcome(
                &outcome,
                tool_call.result.as_ref().map(|r| r.success),
                tool_call.result.as_ref().and_then(|r| r.error.clone()),
            );
            if status == StageStatus::Failed && !stage.continue_on_failure {
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
                        success: false,
                        error,
                        metadata: std::collections::HashMap::new(),
                    })
                    .await;
                for rest in &pipeline.stages[reports.len()..] {
                    reports.push(PipelineStageReport {
                        stage_id: rest.id.clone(),
                        stage_name: rest.name.clone(),
                        tool_name: rest.kind.tool_name().to_string(),
                        status: StageStatus::Skipped,
                        took_ms: 0,
                        error: None,
                    });
                }
                break;
            }
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
                    metadata: std::collections::HashMap::new(),
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
    fn deploy_pipeline_has_mutation_gates() {
        let pipeline = RunPipeline::deploy_pipeline(VibeUseCase::SoftwareDevelopment);
        assert!(pipeline.pipeline_id.starts_with("deploy/"));
        #[cfg(target_os = "windows")]
        {
            assert_eq!(pipeline.stages.len(), 2);
            assert_eq!(pipeline.stages[0].kind, PipelineStageKind::BuildTest);
            assert_eq!(pipeline.stages[1].kind, PipelineStageKind::PublishApp);
            assert!(!pipeline.stages[0].requires_approval);
            assert!(pipeline.stages[1].requires_approval);
            assert!(pipeline.stage("commit_push").is_none());
            assert!(pipeline.stage("domain").is_none());
        }
        #[cfg(not(target_os = "windows"))]
        {
            // #1268 — 9 stages: the original 7 plus domain_verify + domain_tls
            // appended so a bound domain never stays verified=false/tls=pending.
            assert_eq!(pipeline.stages.len(), 9);
            assert_eq!(pipeline.stages[2].kind, PipelineStageKind::BuildTest);
            // #1271 — the deploy pipeline snapshots the current deployment
            // before committing the new state, so rollback is one combo click.
            assert_eq!(pipeline.stages[3].kind, PipelineStageKind::SnapshotPrevious);
            assert_eq!(pipeline.stages[4].kind, PipelineStageKind::CommitPush);
            assert_eq!(pipeline.stages[5].kind, PipelineStageKind::PublishApp);
            assert!(pipeline.stages[4].requires_approval);
            assert!(pipeline.stages[6].requires_approval);
            assert!(pipeline.stage("domain").unwrap().requires_approval);
            assert_eq!(
                pipeline.stage("domain").unwrap().name,
                "Bind domain and TLS"
            );
            assert!(pipeline.stage("snapshot_prev").is_some());
            assert!(pipeline.stage("domain_verify").is_some());
            assert!(pipeline.stage("domain_tls").is_some());
        }
    }

    #[test]
    fn approval_outcome_error_messages_are_distinct() {
        assert_eq!(ApprovalOutcome::Approved.error_message(), None);
        assert_eq!(
            ApprovalOutcome::Cancelled.error_message(),
            Some("Approval denied")
        );
        assert_eq!(
            ApprovalOutcome::TimedOut.error_message(),
            Some("Approval wait timed out")
        );
        assert_eq!(
            ApprovalOutcome::ChannelClosed.error_message(),
            Some("Approval channel closed")
        );
        assert_eq!(
            ApprovalOutcome::NotAvailable.error_message(),
            Some("Approval channel unavailable")
        );
    }

    #[test]
    fn real_stage_kinds_map_to_registered_tools() {
        assert_eq!(PipelineStageKind::BuildTest.tool_name(), "test/run");
        assert_eq!(PipelineStageKind::CommitPush.tool_name(), "git/commit");
        assert_eq!(PipelineStageKind::PublishApp.tool_name(), "publish/project");
        assert_eq!(PipelineStageKind::BindDomain.tool_name(), "domain/bind");
    }

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
            assert_eq!(
                pipeline.stage("plan").expect("stage found").name,
                "Plan compilation"
            );
            assert!(pipeline.stage("missing").is_none());
            assert!(pipeline.pipeline_id.contains(use_case_str(use_case)));
        }
    }

    #[test]
    fn stage_outcome_maps_execution_results() {
        assert_eq!(
            stage_outcome(&Err("rejected".into()), None, None).0,
            StageStatus::Failed
        );
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
            Self {
                runs: Arc::new(RwLock::new(HashMap::new())),
            }
        }
    }

    impl VibeState for MockState {
        fn db_pool(&self) -> &crate::types::DbPool {
            unreachable!("db_pool not exercised in pipeline tests")
        }
        fn broadcast_progress(&self, _event: crate::types::VibeProgressEvent) {}
        fn progress_sender(
            &self,
        ) -> Option<&tokio::sync::broadcast::Sender<crate::types::VibeProgressEvent>> {
            None
        }
        fn active_runs(&self) -> &Arc<RwLock<HashMap<Uuid, crate::types::VibeRun>>> {
            &self.runs
        }
        fn run_signal_sender(
            &self,
        ) -> Option<&tokio::sync::broadcast::Sender<crate::types::VibeRunSignal>> {
            None
        }
        fn llm_config(&self, _bot_id: &uuid::Uuid) -> Option<crate::types::LlmConfig> {
            None
        }
    }

    #[tokio::test]
    async fn engine_fails_fast_and_skips_remaining_stages() {
        let telemetry = Arc::new(VibeTelemetry::new());
        let executor = Arc::new(VibeToolExecutor::new(Arc::new(
            crate::tool_executor::ToolRegistry::new(),
        )));
        let engine = PipelineEngine::new(telemetry.clone());
        let pipeline = RunPipeline::for_use_case(VibeUseCase::SoftwareDevelopment);
        let run_id = Uuid::new_v4();
        let report = engine
            .run(
                &pipeline,
                &executor,
                &MockState::new(),
                &PipelineRunContext {
                    run_id,
                    use_case: VibeUseCase::SoftwareDevelopment,
                    intent: "x",
                    project_id: None,
                    project_name: None,
                    auto_approve: true,
                },
            )
            .await;
        assert_eq!(report.stages.len(), 3);
        assert_eq!(report.run_id, run_id);
        // Intent-dependent stages now receive the run intent as arguments;
        // an unrecognized intent fails the classify stage, and the remaining
        // stages must be Skipped (fail-fast), never executed.
        let first_failed = report
            .stages
            .iter()
            .position(|s| s.status == StageStatus::Failed);
        if let Some(i) = first_failed {
            assert!(
                report.stages[i + 1..]
                    .iter()
                    .all(|s| s.status == StageStatus::Skipped),
                "stages after a fail-fast stage must be Skipped: {:?}",
                report.stages
            );
        }
        let metrics = telemetry
            .get_run_metrics(run_id)
            .await
            .expect("metrics recorded");
        assert_eq!(report.stages.len(), 3);
        assert!(metrics.total_tool_calls >= 1 && metrics.total_tool_calls <= 3);
    }
}

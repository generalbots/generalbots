//! `pipeline::runner` — split per #1443 (AGENTS.md 450-line rule).

use super::*;

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

pub(crate) fn stage(id: &str, kind: PipelineStageKind, timeout_secs: u64) -> PipelineStage {
    stage_approval(id, kind, timeout_secs, false)
}

/// #1268 — a stage whose failure must not abort the deploy (DNS propagation
/// delays, upstream ACME hiccups): the pipeline continues when it fails.
pub(crate) fn stage_continue(id: &str, kind: PipelineStageKind, timeout_secs: u64) -> PipelineStage {
    PipelineStage {
        id: id.to_string(),
        name: kind.display_name().to_string(),
        kind,
        timeout_secs,
        requires_approval: false,
        continue_on_failure: true,
    }
}

pub(crate) fn stage_approval(
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
    pub(crate) telemetry: Arc<VibeTelemetry>,
}

/// Run-scoped inputs for [`PipelineEngine::run`], bundled to keep the method
/// signature within clippy's argument-count threshold.
pub struct PipelineRunContext<'a> {
    pub run_id: Uuid,
    pub use_case: VibeUseCase,
    pub intent: &'a str,
    pub project_id: Option<&'a str>,
    pub project_name: Option<&'a str>,
    /// #1280 — the run's acting user, forwarded to the RBAC-guarded
    /// deployment API via publish args (the pipeline path never passes
    /// through the agent-loop arg injection).
    pub user_id: Uuid,
}

impl PipelineEngine {
    pub fn new(telemetry: Arc<VibeTelemetry>) -> Self {
        Self { telemetry }
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
            user_id,
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
                    PipelineStageKind::PublishApp => {
                        // #1280 — deployment API enforces deploy RBAC even on
                        // internal (X-Internal-Token) calls; without the acting
                        // user the request is rejected as anonymous.
                        serde_json::json!({
                            "project_id": project_id.unwrap_or(""),
                            "env": "production",
                            "on_behalf_of_user": user_id.to_string(),
                        })
                        // The deploy pipeline is the ONLY sanctioned writer of
                        // a site project's public slug: the stamp is injected
                        // by the executor for internal calls (see
                        // `VibeToolCall::internal`), keeping it out of the
                        // schema so a client payload can never supply it.
                    }
                    PipelineStageKind::BindDomain => serde_json::json!({
                        "project_id": project_id.unwrap_or(""),
                        "env": "production",
                        "domain": format!("{}.{domain}", project_name.unwrap_or("app"), domain = crate::publish::published_domain()),
                    }),
                    // #1268 — verify/verify-then-issue for the just-bound
                    // host. domain/verify reads the binding itself (token
                    // already recorded at bind time); domain/tls re-applies
                    // the route so ACME issues on first request.
                    PipelineStageKind::VerifyDomain | PipelineStageKind::IssueTls => {
                        serde_json::json!({
                            "env": "production",
                            "domain": format!("{}.{domain}", project_name.unwrap_or("app"), domain = crate::publish::published_domain()),
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
                    // #1504 — bot PROD promotion takes the project id only;
                    // the tool resolves the branch/bot layout server-side.
                    PipelineStageKind::PromoteBotProd => serde_json::json!({
                        "project_id": project_id.unwrap_or(""),
                    }),
                    _ => serde_json::json!({}),
                }
            };
            let mut tool_call = VibeToolCall::new(
                run_id,
                tool_name.to_string(),
                arguments,
                false,
            );
            // Approval gate removed — all stages run automatically; decisions
            // and failures surface in the Vibe chat instead of blocking.
            // The engine is the authorized orchestrator: approval policy is
            // decided upstream (e.g. the agent loop), not per stage here.
            tool_call.approved = true;
            // Internal pipeline stages may carry sanctioned hidden arguments
            // (the publish production stamp); agent-loop calls never do.
            tool_call.internal = true;
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
pub(crate) fn stage_outcome(
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

pub(crate) fn use_case_str(use_case: VibeUseCase) -> &'static str {
    match use_case {
        VibeUseCase::SoftwareDevelopment => "software_development",
        VibeUseCase::CustomerSupport => "customer_support",
        VibeUseCase::FinancialAnalysis => "financial_analysis",
    }
}

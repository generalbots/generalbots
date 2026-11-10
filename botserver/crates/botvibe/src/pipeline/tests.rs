//! `pipeline::tests` — split per #1443.

use super::*;
    use super::runner::stage_outcome;
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
            // #1268 — the original 7 plus domain_verify + domain_tls appended
            // so a bound domain never stays verified=false/tls=pending;
            // #1504 — promote_bot runs before publish to move the TEST
            // release into the PROD bot layout (tolerates failure).
            assert_eq!(pipeline.stages.len(), 10);
            assert_eq!(pipeline.stages[2].kind, PipelineStageKind::BuildTest);
            // #1271 — the deploy pipeline snapshots the current deployment
            // before committing the new state, so rollback is one combo click.
            assert_eq!(pipeline.stages[3].kind, PipelineStageKind::SnapshotPrevious);
            assert_eq!(pipeline.stages[4].kind, PipelineStageKind::CommitPush);
            assert_eq!(pipeline.stages[5].kind, PipelineStageKind::PromoteBotProd);
            assert_eq!(pipeline.stages[6].kind, PipelineStageKind::PublishApp);
            assert!(pipeline.stages[4].requires_approval);
            assert!(pipeline.stages[6].requires_approval);
            assert!(pipeline.stage("promote_bot").is_some());
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
        events: Arc<std::sync::Mutex<Vec<crate::types::VibeProgressEvent>>>,
    }

    impl MockState {
        fn new() -> Self {
            Self {
                runs: Arc::new(RwLock::new(HashMap::new())),
                events: Arc::new(std::sync::Mutex::new(Vec::new())),
            }
        }

        fn events(&self) -> Vec<crate::types::VibeProgressEvent> {
            self.events
                .lock()
                .map(|events| events.clone())
                .unwrap_or_default()
        }
    }

    impl VibeState for MockState {
        fn db_pool(&self) -> &crate::types::DbPool {
            unreachable!("db_pool not exercised in pipeline tests")
        }
        fn broadcast_progress(&self, event: crate::types::VibeProgressEvent) {
            if let Ok(mut events) = self.events.lock() {
                events.push(event);
            }
        }
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
                    user_id: Uuid::nil(),
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

/// #2045/#2046 — a stage cut off by its workspace budget must report
/// `timed_out` with the in-flight step named, keep the rows that already
/// finished, and publish the budget to the run card sink.
#[tokio::test]
async fn a_stage_cut_off_by_the_budget_keeps_the_finished_rows() {
    use crate::tool_executor::{ToolCategory, ToolDescriptor, ToolHandler, ToolSchema};
    use crate::types::VibeToolResult;
    use crate::workspace_run::BudgetLimits;

    fn tool_result() -> VibeToolResult {
        VibeToolResult {
            success: true,
            data: serde_json::Value::Null,
            error: None,
            latency_ms: 0,
        }
    }

    let registry = Arc::new(crate::tool_executor::ToolRegistry::new());
    let fast: ToolHandler = Arc::new(|_args, _state| Box::pin(async move { tool_result() }));
    registry
        .register(
            ToolDescriptor {
                schema: ToolSchema::new("bot/deploy-prod", "fast stage").with_parameters(
                    serde_json::json!({
                        "type": "object",
                        "properties": { "project_id": { "type": "string" } },
                    }),
                ),
                category: ToolCategory::Deployment,
            },
            fast,
        )
        .await;
    let slow: ToolHandler = Arc::new(|_args, _state| {
        Box::pin(async move {
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            tool_result()
        })
    });
    registry
        .register(
            ToolDescriptor {
                schema: ToolSchema::new("test/run", "slow build").with_parameters(
                    serde_json::json!({
                        "type": "object",
                        "properties": {
                            "project": { "type": "string" },
                            "project_id": { "type": "string" },
                            "message": { "type": "string" },
                        },
                    }),
                ),
                category: ToolCategory::Deployment,
            },
            slow,
        )
        .await;
    let executor = VibeToolExecutor::new(registry);

    let pipeline = RunPipeline {
        pipeline_id: "test/budget".to_string(),
        use_case: VibeUseCase::SoftwareDevelopment,
        stages: vec![
            stage("fast", PipelineStageKind::PromoteBotProd, 1_000),
            stage("slow", PipelineStageKind::BuildTest, 0),
        ],
    };
    let state = MockState::new();
    let engine = PipelineEngine::new(Arc::new(VibeTelemetry::new()));
    let limits = BudgetLimits::from_values(Some(1), Some(2_000), Some(0), None);
    let report = engine
        .run_with_limits(
            &pipeline,
            &executor,
            &state,
            &PipelineRunContext {
                run_id: Uuid::new_v4(),
                use_case: VibeUseCase::SoftwareDevelopment,
                intent: "build it",
                project_id: Some("project-1"),
                project_name: Some("demo"),
                user_id: Uuid::nil(),
            },
            limits,
        )
        .await;

    assert_eq!(report.stages.len(), 2, "the finished stage must be kept");
    assert_eq!(report.stages[0].status, StageStatus::Completed);
    assert_eq!(report.stages[1].status, StageStatus::TimedOut);
    let error = report.stages[1].error.clone().unwrap_or_default();
    assert!(
        error.contains("Build and test"),
        "the in-flight step must be named: {error}"
    );
    assert!(error.contains("budget"), "the budget must be quoted: {error}");

    let events = state.events();
    assert!(
        events.iter().any(|event| event.message.contains("budget")),
        "the run card must receive the budget: {:?}",
        events.iter().map(|e| &e.message).collect::<Vec<_>>()
    );
}

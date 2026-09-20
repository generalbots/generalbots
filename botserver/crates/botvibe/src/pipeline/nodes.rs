//! `pipeline::nodes` — split per #1443 (AGENTS.md 450-line rule).

use super::*;

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
    /// #1504 — bot-kind projects: promote the TEST materialization into the
    /// `{bot}` PROD layout and queue recompiles (git monitor hook). Sits
    /// between CommitPush and PublishApp; PublishApp stays a no-op for bots.
    PromoteBotProd,
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
            Self::PromoteBotProd => "bot/deploy-prod",
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
            Self::PromoteBotProd => "Promote TEST bot to PROD",
            Self::PublishApp => "Publish application",
            Self::BindDomain => "Bind domain and TLS",
            Self::VerifyDomain => "Verify domain ownership",
            Self::IssueTls => "Issue TLS certificate",
        }
    }
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
            // #1504 — bot projects promote the TEST release into the PROD
            // bot layout; websites/apps run the regular publish instead. Both
            // stages tolerate failure so a wrong-kind tool never blocks the
            // pipeline (the other stage is the effective one).
            stage_continue("promote_bot", PipelineStageKind::PromoteBotProd, 120),
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

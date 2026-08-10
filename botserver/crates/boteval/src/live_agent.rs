use crate::runner::{HarnessOutcome, HarnessTarget};
use async_trait::async_trait;
use botvibe::agent_loop::AgentLoop;
use botvibe::permissions::{PermissionEngine, PermissionEngineRef, PermissionMode};
use botvibe::skills::SkillStore;
use botvibe::telemetry::VibeTelemetry;
use botvibe::tool_executor::{ToolRegistry, VibeToolExecutor};
use botvibe::types::{
    DbPool, LlmConfig, VibeProgressEvent, VibeRun, VibeRunConfig, VibeRunState, VibeRunSignal,
    VibeState,
};
use botvibe::VibePromptManager;
use diesel::r2d2::{ConnectionManager, Pool};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::OnceLock;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

const EVAL_BUDGET_CENTS: u64 = 1000;
const EVAL_MAX_TOOL_CALLS: u32 = 8;
const EVAL_TIMEOUT_SECS: u64 = 120;

pub struct LiveHarnessTarget {
    pub workspace_root: std::path::PathBuf,
}

impl LiveHarnessTarget {
    pub fn new(workspace_root: impl Into<std::path::PathBuf>) -> Self {
        Self {
            workspace_root: workspace_root.into(),
        }
    }
}

#[async_trait]
impl HarnessTarget for LiveHarnessTarget {
    async fn run(&self, prompt: &str) -> HarnessOutcome {
        let mut config = VibeRunConfig::default();
        config.auto_approve = true;
        config.max_tool_calls = EVAL_MAX_TOOL_CALLS;
        config.timeout_seconds = EVAL_TIMEOUT_SECS;
        config.budget_cents = EVAL_BUDGET_CENTS;
        let run_id = Uuid::new_v4();
        let mut run = VibeRun::new(
            Uuid::nil(),
            run_id,
            Uuid::nil(),
            prompt.to_string(),
            config,
        );
        let registry = Arc::new(ToolRegistry::new());
        let executor = VibeToolExecutor::new(registry);
        let telemetry = Arc::new(VibeTelemetry::new());
        let prompt_manager = Arc::new(VibePromptManager::new());
        let state = Arc::new(EvalVibeState::new());
        let permissions: PermissionEngineRef = Arc::new(PermissionEngine::new());
        permissions.set_mode(PermissionMode::Bypass).await;
        let skills = Arc::new(SkillStore::new());
        let agent = AgentLoop::new(
            prompt_manager,
            executor,
            telemetry.clone(),
            state.clone(),
        )
        .with_security(permissions, skills);
        agent.execute_run(&mut run).await;
        let passed = run.state == VibeRunState::Completed;
        let cost = telemetry
            .get_run_metrics(run_id)
            .await
            .map(|m| m.total_cost)
            .unwrap_or(0.0);
        let summary = serde_json::json!({
            "state": run.state.to_string(),
            "tool_calls": run.tool_calls.len(),
            "intent": run.intent,
        })
        .to_string();
        HarnessOutcome {
            passed,
            tool_calls: run.tool_calls.len() as u32,
            cost,
            summary,
        }
    }
}

static EVAL_POOL: OnceLock<Box<DbPool>> = OnceLock::new();

fn shared_pool() -> &'static DbPool {
    EVAL_POOL
        .get_or_init(|| {
            let manager = ConnectionManager::new("postgres://eval-unused-host/eval");
            Box::new(Pool::new(manager))
        })
        .as_ref()
}

struct EvalVibeState {
    runs: Arc<RwLock<HashMap<Uuid, VibeRun>>>,
    pool: &'static DbPool,
    progress: broadcast::Sender<VibeProgressEvent>,
    signals: broadcast::Sender<VibeRunSignal>,
}

impl EvalVibeState {
    fn new() -> Self {
        let (progress, _) = broadcast::channel(64);
        let (signals, _) = broadcast::channel(16);
        Self {
            runs: Arc::new(RwLock::new(HashMap::new())),
            pool: shared_pool(),
            progress,
            signals,
        }
    }
}

impl VibeState for EvalVibeState {
    fn db_pool(&self) -> &DbPool {
        self.pool
    }

    fn broadcast_progress(&self, event: VibeProgressEvent) {
        let _ = self.progress.send(event);
    }

    fn progress_sender(&self) -> Option<&broadcast::Sender<VibeProgressEvent>> {
        Some(&self.progress)
    }

    fn active_runs(&self) -> &Arc<RwLock<HashMap<Uuid, VibeRun>>> {
        &self.runs
    }

    fn run_signal_sender(&self) -> Option<&broadcast::Sender<VibeRunSignal>> {
        Some(&self.signals)
    }

    fn llm_config(&self, _bot_id: &Uuid) -> Option<LlmConfig> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eval_state_exposes_isolated_channels() {
        let state = EvalVibeState::new();
        assert!(state.progress_sender().is_some());
        assert!(state.run_signal_sender().is_some());
        assert!(state.llm_config(&Uuid::nil()).is_none());
    }

    #[test]
    fn eval_budget_and_caps_are_sane() {
        assert!(EVAL_BUDGET_CENTS >= 100);
        assert!(EVAL_MAX_TOOL_CALLS >= 1);
        assert!(EVAL_TIMEOUT_SECS >= 30);
    }
}
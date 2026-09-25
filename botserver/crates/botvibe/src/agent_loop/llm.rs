//! `agent_loop::llm` — split per #1443 (AGENTS.md 450-line rule).

use super::*;

/// #1386 — hard cap on one run. The default (600s) fits fast providers, but a
/// large local model or a congested gateway needs several minutes per turn and
/// every scaffold run then dies as "Agent loop timed out". Operators can raise
/// the cap with `VIBE_RUN_TIMEOUT_SECS`; an unset/invalid value keeps 600s.
pub(crate) fn run_timeout_cap() -> u64 {
    std::env::var("VIBE_RUN_TIMEOUT_SECS")
        .ok()
        .and_then(|v| v.trim().parse::<u64>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(DEFAULT_TIMEOUT_SECS)
}

// vibe33 #813 — transient provider failures must not kill the run.
pub(crate) const MAX_LLM_RETRIES: u32 = 6;

pub(crate) const LLM_RETRY_BACKOFF_SECS: &[u64] = &[1, 2, 4, 8, 12, 16];

// A hung gateway (no response, no error — common behind flaky providers
// such as the CodeBuddy proxy) otherwise pins the run for the whole 120s
// per attempt through every retry. A shorter per-request timeout lets the
// retry/backoff path recover instead of stalling the run for ~13 minutes.
pub(crate) const LLM_REQUEST_TIMEOUT_SECS: u64 = 45;

// Some providers (e.g. tabitoken.com behind Cloudflare) reject the default
// reqwest/curl User-Agent with a 403 WAF block. Send a browser UA so the
// agent loop's LLM calls are not mistaken for scraping.
pub(crate) const LLM_USER_AGENT: &str =
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/149.0.0.0 Safari/537.36";

pub struct AgentLoop {
    pub(crate) prompt_manager: Arc<VibePromptManager>,
    pub(crate) tool_executor: Arc<VibeToolExecutor>,
    pub(crate) telemetry: Arc<VibeTelemetry>,
    pub(crate) state: Arc<dyn VibeState>,
    pub(crate) permissions: PermissionEngineRef,
    pub(crate) skills: Arc<SkillStore>,
}

impl AgentLoop {
    pub fn new(
        prompt_manager: Arc<VibePromptManager>,
        tool_executor: Arc<VibeToolExecutor>,
        telemetry: Arc<VibeTelemetry>,
        state: Arc<dyn VibeState>,
    ) -> Self {
        Self {
            prompt_manager,
            tool_executor,
            telemetry,
            state,
            permissions: Arc::new(PermissionEngine::new()),
            skills: Arc::new(SkillStore::new()),
        }
    }

    pub async fn execute_run(&self, run: &mut VibeRun) {
        run.transition(VibeRunState::Running);
        self.sync_active_run(run).await;
        self.broadcast_event(run, "running", "Autonomous agent loop started", 0);
        let config = run.config.clone();
        let max_steps = config.max_tool_calls.min(DEFAULT_MAX_STEPS);
        let timeout_duration = Duration::from_secs(config.timeout_seconds.min(run_timeout_cap()));
        // #1270 — shared with the LLM retry loop so it can stop retrying
        // when the remaining run time can no longer fit another attempt.
        let deadline = tokio::time::Instant::now() + timeout_duration;

        let result = timeout(timeout_duration, self.run_loop(run, max_steps, deadline)).await;

        match result {
            Ok(()) => {
                if run.state == VibeRunState::Running {
                    self.finish_truthfully(run, "Agent loop completed successfully")
                        .await;
                }
            }
            Err(_) => {
                run.transition(VibeRunState::Failed);
                run.error = Some("Agent loop timed out".to_string());
                self.sync_active_run(run).await;
                self.telemetry
                    .record_run_completion(run, 0, None, 0.0)
                    .await;
                self.broadcast_event(run, "failed", "Agent loop timed out", 100);
                warn!(
                    "Vibe run {} timed out after {}s",
                    run.run_id,
                    timeout_duration.as_secs()
                );
            }
        }
    }

    /// #819 — a run that executed zero tool calls performed no work and must
    /// not be reported as "Completed" (the model text-replied instead of acting).
    /// This terminal helper fails the run with an honest verdict when no tool
    /// ever ran, and otherwise completes it normally.
    /// #819/#923b — whether the user's intent demands a persisted change
    /// (shared by the prose nudge in the run loop and the honest verdict in
    /// finish_truthfully).
    pub(crate) fn intent_requires_mutation(intent: &str) -> bool {
        let intent = intent.to_ascii_lowercase();
        [
            "change",
            "edit",
            "modify",
            "rename",
            "update",
            "make",
            "set ",
            "title",
            "color",
            "colour",
            "theme",
            "background",
            "publish",
            "deploy",
            // #1276 — creation intents are mutations too: a "create a page"
            // run that never writes a file must not be reported as completed.
            "create",
            "build",
            "generate",
            "scaffold",
            "write",
            "add ",
        ]
        .iter()
        .any(|word| intent.contains(word))
    }
}

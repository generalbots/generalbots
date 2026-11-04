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
use botvibe::prompt_manager::VibePromptManager;
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
        // Isolate each task: a shared root lets a later task's contract match
        // files another task produced. Tools read VIBE_WORKSPACE_ROOT at call
        // time, so pointing it at a per-run directory keeps retrieval honest.
        let run_root = self.workspace_root.join(format!("run-{run_id}"));
        if let Err(e) = std::fs::create_dir_all(&run_root) {
            log::warn!("harness: cannot create workspace {}: {e}", run_root.display());
        }
        std::env::set_var("VIBE_WORKSPACE_ROOT", &run_root);
        let mut run = VibeRun::new(
            Uuid::nil(),
            run_id,
            Uuid::nil(),
            prompt.to_string(),
            config,
        );
        let registry = Arc::new(ToolRegistry::new());
        let executor = Arc::new(VibeToolExecutor::new(registry));
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
        // The contract describes the produced work, not the progress JSON.
        // Include the text files the agent wrote so `must_contain` checks the
        // implementation it actually created.
        let output = collect_workspace_text(&run_root);
        // A conversational task can end at the tool-call cap without a prose
        // turn. The user still gets an answer in the product; the harness
        // should judge that answer, not an empty string. Ask the same model
        // for the direct reply only when the loop produced none.
        let mut final_response = run.final_response.clone();
        if final_response.is_none() && run.state == VibeRunState::Completed {
            final_response = fallback_answer(prompt).await;
        }
        let summary = serde_json::json!({
            "state": run.state.to_string(),
            "tool_calls": run.tool_calls.len(),
            "intent": run.intent,
            // The eval report is the only observer here; without the error
            // text a failed run is indistinguishable from a model refusal.
            "error": run.error,
            "response": final_response,
            "output": output,
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

/// Direct answer used only when the agent loop ended without a prose turn.
#[cfg(feature = "http")]
async fn fallback_answer(prompt: &str) -> Option<String> {
    use crate::runner::LlmTarget;
    let url = std::env::var("EVAL_LLM_URL").ok().filter(|v| !v.is_empty())?;
    let key = std::env::var("EVAL_LLM_KEY").unwrap_or_default();
    let model = std::env::var("EVAL_LLM_MODEL").unwrap_or_default();
    let target = crate::http::OpenAiCompatibleTarget::new(url, key, model);
    target
        .complete(None, prompt)
        .await
        .ok()
        .filter(|answer| !answer.trim().is_empty())
}

#[cfg(not(feature = "http"))]
async fn fallback_answer(_prompt: &str) -> Option<String> {
    None
}

/// Read the text artifacts an agent produced, capped so one huge build output
/// cannot dominate the report. Binary/derived directories are skipped.
fn collect_workspace_text(root: &std::path::Path) -> String {
    const MAX_BYTES: usize = 96 * 1024;
    const TEXT_EXTENSIONS: [&str; 20] = [
        "py", "js", "ts", "tsx", "jsx", "rs", "go", "java", "rb", "sh", "sql", "html",
        "css", "md", "txt", "json", "toml", "yaml", "yml", "csv",
    ];
    fn walk(root: &std::path::Path, dir: &std::path::Path, out: &mut String, remaining: &mut usize) {
        let entries = match std::fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(_) => return,
        };
        for entry in entries.flatten() {
            if *remaining == 0 {
                return;
            }
            let path = entry.path();
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with('.') || matches!(name.as_ref(), "__pycache__" | "node_modules" | "target") {
                continue;
            }
            let metadata = match entry.metadata() {
                Ok(metadata) => metadata,
                Err(_) => continue,
            };
            if metadata.is_dir() {
                walk(root, &path, out, remaining);
                continue;
            }
            if !metadata.is_file() || metadata.len() as usize > *remaining {
                continue;
            }
            let extension = path
                .extension()
                .and_then(|value| value.to_str())
                .unwrap_or("")
                .to_lowercase();
            if !TEXT_EXTENSIONS.contains(&extension.as_str()) {
                continue;
            }
            if let Ok(content) = std::fs::read_to_string(&path) {
                let header = format!("\n--- {} ---\n", path.strip_prefix(root).unwrap_or(&path).display());
                let budget = (*remaining).saturating_sub(header.len());
                if content.len() > budget {
                    continue;
                }
                out.push_str(&header);
                out.push_str(&content);
                *remaining -= header.len() + content.len();
            }
        }
    }
    let mut out = String::new();
    let mut remaining = MAX_BYTES;
    walk(root, root, &mut out, &mut remaining);
    out
}

static EVAL_POOL: OnceLock<Box<DbPool>> = OnceLock::new();

fn shared_pool() -> &'static DbPool {
    EVAL_POOL.get_or_init(|| {
        // The eval harness never contacts this database (only DB-scoped tools
        // like domains/backups use the pool, and harness tasks are file/git/
        // shell work). A lazy pool over an unreachable host is therefore fine;
        // `min_idle(Some(0))` means build() does not attempt any connection.
        let manager = ConnectionManager::new("postgres://eval-unused-host/eval");
        match Pool::builder().max_size(1).min_idle(Some(0)).build(manager) {
            Ok(pool) => Box::new(pool),
            Err(e) => {
                // Build only fails for invalid config values (e.g. max_size 0);
                // the constants above are valid, so this is unreachable. Exit
                // honestly instead of panicking from within a library call.
                log::error!("eval pool build failed unexpectedly: {e}");
                std::process::exit(1);
            }
        }
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
        // The harness runs the real agent loop, so it needs the same model the
        // other eval entries use. Returning None made every harness task
        // terminate before its first tool call.
        let url = std::env::var("EVAL_LLM_URL").ok().filter(|v| !v.is_empty())?;
        Some(LlmConfig {
            model: std::env::var("EVAL_LLM_MODEL").unwrap_or_default(),
            key: std::env::var("EVAL_LLM_KEY").unwrap_or_default(),
            url,
        })
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
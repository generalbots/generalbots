//! A subscription turn that ran into the plan's limit.
//!
//! The CLI's sentence becomes the agent-limit error the chat pipeline ends
//! the turn with, with the instant the plan comes back when it can be pinned
//! down (see `botagent::plan_limit::reset_at`). Codex keeps its plan windows
//! in its session logs; Claude Code and Antigravity only say it in the
//! sentence itself.

use axum::http::StatusCode;
use chrono::{DateTime, SecondsFormat, Utc};

use botagent::plan_limit::{reset_at, reset_hint, MARKER};
use botagent::AgentService;

use crate::core::ai_accounts::{provider_label, CODEX_CLI};
use crate::core::bot::pipeline::llm::agent_limit::AgentLimit;

/// The turn's result for the LLM trait: a spent plan becomes the encoded
/// agent limit, every other error passes through unchanged.
pub(super) async fn settle(
    result: Result<String, (StatusCode, String)>,
    service: &AgentService,
    vm_name: &str,
    agent: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    match result {
        Ok(text) => Ok(text),
        Err((_, error)) => Err(super::boxed(turn_error(service, vm_name, agent, error).await)),
    }
}

async fn turn_error(service: &AgentService, vm_name: &str, agent: &str, error: String) -> String {
    let Some(phrase) = error.strip_prefix(MARKER).map(str::trim) else {
        return error;
    };
    let now = Utc::now();
    let windows = if agent == CODEX_CLI {
        codex_windows(service, vm_name).await
    } else {
        Vec::new()
    };
    let reset = reset_at(reset_hint(phrase, now), &windows, now);
    botagent::plan_usage::record_spent(vm_name, agent, reset);
    AgentLimit {
        agent: agent.to_string(),
        label: provider_label(agent).to_string(),
        reset: reset.map(|t| t.to_rfc3339_opts(SecondsFormat::Secs, true)),
        detail: phrase.to_string(),
    }
    .encode()
}

/// The plan windows the Codex CLI recorded, read after the failed turn so
/// the one that ran out is the newest state.
async fn codex_windows(service: &AgentService, vm_name: &str) -> Vec<(f64, Option<DateTime<Utc>>)> {
    match botagent::codex::limits(service, vm_name).await {
        Ok(Some(limits)) => limits.windows.iter().map(|w| (w.used_percent, w.resets_at)).collect(),
        Ok(None) => Vec::new(),
        Err((code, message)) => {
            log::warn!("agent plan limit: codex session logs unreadable ({}): {message}", code.as_u16());
            Vec::new()
        }
    }
}

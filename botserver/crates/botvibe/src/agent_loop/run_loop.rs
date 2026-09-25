//! `agent_loop::run_loop` — split per #1443 (AGENTS.md 450-line rule).

use super::*;

pub(crate) const DEFAULT_MAX_STEPS: u32 = 50;

impl AgentLoop {
    pub(crate) async fn sync_active_run(&self, run: &VibeRun) {
        let mut runs = self.state.active_runs().write().await;
        runs.insert(run.run_id, run.clone());
    }

    pub(crate) async fn finish_truthfully(&self, run: &mut VibeRun, completed_msg: &str) {
        let requires_mutation = Self::intent_requires_mutation(&run.intent);
        let successful_mutation = run.tool_calls.iter().any(|call| {
            matches!(
                call.tool_name.as_str(),
                "file/write"
                    | "file/replace"
                    | "file/delete"
                    | "file/set-title"
                    | "shell/run"
                    | "git/commit"
                    | "publish/project"
            ) && call.result.as_ref().is_some_and(|result| result.success)
        });
        let failure = if run.tool_calls.is_empty() {
            Some("Agent produced no tool calls — no work was executed")
        } else if requires_mutation && !successful_mutation {
            Some("Agent did not complete a successful mutation — requested edit was not applied")
        } else {
            None
        };
        if let Some(error) = failure {
            run.transition(VibeRunState::Failed);
            run.error = Some(error.to_string());
            self.sync_active_run(run).await;
            self.telemetry
                .record_run_completion(run, 0, None, 0.0)
                .await;
            self.broadcast_event(
                run,
                "failed",
                "Requested work was not successfully applied",
                100,
            );
        } else {
            run.transition(VibeRunState::Completed);
            self.sync_active_run(run).await;
            self.telemetry
                .record_run_completion(run, 0, None, 0.0)
                .await;
            self.broadcast_event(run, "completed", completed_msg, 100);
        }
    }

    pub(crate) fn local_deterministic_tool_response(api_url: &str, run: &VibeRun) -> Option<String> {
        match Self::local_forced_tool(api_url, run)? {
            "file/set-title" => {
                let title = Self::requested_title(&run.intent)?;
                Some(
                    serde_json::json!({
                        "tool_calls": [{
                            "tool_name": "file/set-title",
                            "arguments": {"title": title}
                        }]
                    })
                    .to_string(),
                )
            }
            "publish/project" => Some(
                serde_json::json!({
                    "tool_calls": [{
                        "tool_name": "publish/project",
                        "arguments": {"env": "production"}
                    }]
                })
                .to_string(),
            ),
            _ => None,
        }
    }

    pub(crate) async fn budget_exceeded(&self, run: &VibeRun) -> bool {
        let budget_cents = run.config.budget_cents;
        if budget_cents == 0 {
            return false;
        }
        let spent = self
            .telemetry
            .get_run_metrics(run.run_id)
            .await
            .map(|m| m.total_cost)
            .unwrap_or(0.0);
        let spent_cents = (spent * 100.0).round() as u64;
        if spent_cents >= budget_cents {
            warn!(
                "Vibe run {} exceeded budget: {spent_cents} cents >= {budget_cents} cents",
                run.run_id
            );
            true
        } else {
            false
        }
    }
}

/// Caps a string at `limit` characters, cutting on a char boundary and
/// marking the truncation so long tool results stay visible but bounded.
pub(crate) fn truncate(text: &str, limit: usize) -> String {
    if text.len() <= limit {
        return text.to_string();
    }
    let mut end = limit;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}… (truncated)", &text[..end])
}

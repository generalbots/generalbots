//! Parallel sub-agent bookkeeping (#1694).
//!
//! A parent run can split its task into focused sub-agents. Each sub-agent is
//! a row in `subagent_runs` with its own status and result; this module owns
//! the progress arithmetic and the request validation so both are covered by
//! executed tests instead of only living in the HTTP handler.

/// Hard bound on sub-agents per request (cost and fan-out control).
pub const MAX_SUBAGENTS: usize = 8;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Progress {
    pub total: u64,
    pub pending: u64,
    pub running: u64,
    pub succeeded: u64,
    pub failed: u64,
}

impl Progress {
    /// Percent of sub-agents that reached a terminal state.
    #[must_use]
    pub fn percent_done(&self) -> u8 {
        if self.total == 0 {
            return 100;
        }
        let done = self.succeeded.saturating_add(self.failed);
        let percent = done.saturating_mul(100) / self.total;
        u8::try_from(percent.min(100)).unwrap_or(100)
    }

    /// Overall state of the fan-out: failed > running > pending > succeeded.
    #[must_use]
    pub fn overall(&self) -> &'static str {
        if self.total == 0 {
            return "succeeded";
        }
        if self.running > 0 {
            return "running";
        }
        if self.pending > 0 {
            return "pending";
        }
        if self.failed > 0 {
            return "failed";
        }
        "succeeded"
    }
}

/// Summarizes status strings from `subagent_runs`.
#[must_use]
pub fn summarize(statuses: &[&str]) -> Progress {
    let mut progress = Progress {
        total: statuses.len() as u64,
        ..Progress::default()
    };
    for status in statuses {
        match *status {
            "running" => progress.running += 1,
            "succeeded" | "completed" => progress.succeeded += 1,
            "failed" | "error" => progress.failed += 1,
            _ => progress.pending += 1,
        }
    }
    progress
}

/// Validates a fan-out request: 1..=MAX tasks, each with a title and prompt.
pub fn validate_tasks(tasks: &[(String, String)]) -> Result<(), String> {
    if tasks.is_empty() {
        return Err("at least one sub-agent task is required".to_string());
    }
    if tasks.len() > MAX_SUBAGENTS {
        return Err(format!(
            "at most {MAX_SUBAGENTS} sub-agents per request"
        ));
    }
    for (index, (title, prompt)) in tasks.iter().enumerate() {
        if title.trim().is_empty() {
            return Err(format!("tasks[{index}].title is empty"));
        }
        if prompt.trim().is_empty() {
            return Err(format!("tasks[{index}].prompt is empty"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn task(title: &str, prompt: &str) -> (String, String) {
        (title.to_string(), prompt.to_string())
    }

    #[test]
    fn empty_requests_are_rejected() {
        assert!(validate_tasks(&[]).is_err());
    }

    #[test]
    fn requests_are_bounded_and_require_content() {
        let too_many: Vec<(String, String)> = (0..=MAX_SUBAGENTS)
            .map(|index| task(&format!("t{index}"), "p"))
            .collect();
        assert!(validate_tasks(&too_many).is_err());
        assert!(validate_tasks(&[task("", "p")]).is_err());
        assert!(validate_tasks(&[task("t", "   ")]).is_err());
        assert!(validate_tasks(&[task("t", "p")]).is_ok());
    }

    #[test]
    fn progress_counts_each_state() {
        let progress = summarize(&["pending", "running", "succeeded", "failed", "others"]);
        assert_eq!(progress.total, 5);
        assert_eq!(progress.pending, 2);
        assert_eq!(progress.running, 1);
        assert_eq!(progress.succeeded, 1);
        assert_eq!(progress.failed, 1);
    }

    #[test]
    fn percent_and_overall_follow_terminal_states() {
        let mut progress = summarize(&["running", "pending"]);
        assert_eq!(progress.percent_done(), 0);
        assert_eq!(progress.overall(), "running");
        progress = summarize(&["succeeded", "failed"]);
        assert_eq!(progress.percent_done(), 100);
        assert_eq!(progress.overall(), "failed");
        progress = summarize(&["succeeded", "succeeded"]);
        assert_eq!(progress.percent_done(), 100);
        assert_eq!(progress.overall(), "succeeded");
        assert_eq!(Progress::default().overall(), "succeeded");
    }
}

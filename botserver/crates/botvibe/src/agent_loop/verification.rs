//! `agent_loop::verification` — split per #1443 (AGENTS.md 450-line rule).

pub(crate) const MAX_VERIFY_FAILURES: u32 = 2;

#[cfg(target_os = "windows")]
pub(crate) fn path_requires_repair(path: &str) -> bool {
    matches!(
        path.trim(),
        "" | "." | "./" | ".\\" | "/" | "\\" | "..." | "…"
    )
}

/// Outcome of one tool-call step in the agent loop. `Executed` = the tool
/// ran (regardless of outcome); `Skipped` = the tool was rejected
/// pre-execution (validation) and the loop keeps going.
pub(crate) enum ToolStep {
    Executed,
    Skipped,
}

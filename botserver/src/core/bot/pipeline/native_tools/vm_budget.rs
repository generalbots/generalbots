//! #2045 — workspace budget, timeout cap and the failure messages.
//!
//! Split from `vm_exec.rs` so that file stays under the size limit. The
//! defaults are overridable so a node with slow storage or a real build can
//! run longer without editing code.

/// Default time allowed beyond the requested program timeout to wake or
/// provision the workspace and return its result (#2045 — overridable).
const DEFAULT_WORKSPACE_OVERHEAD_SECS: u64 = 120;

/// Default cap for the caller's `timeout_ms`. A cold image pull plus a real
/// build does not fit in two minutes, so the cap is ten; the environment can
/// raise or lower it.
const DEFAULT_TIMEOUT_CAP_MS: u64 = 600_000;

/// Parses a positive integer out of an optional environment value. Blank,
/// non-numeric and zero values mean "not set" so a misconfigured override
/// falls back to the default instead of disabling the budget.
fn parse_positive_u64(value: Option<&str>) -> Option<u64> {
    value
        .map(str::trim)
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| *value > 0)
}

fn env_u64(name: &str) -> Option<u64> {
    parse_positive_u64(std::env::var(name).ok().as_deref())
}

/// Overhead added to a program timeout, in seconds. `GB_WORKSPACE_OVERHEAD_SECS`
/// covers cold starts; the default stays 120 s.
fn workspace_overhead_secs() -> u64 {
    env_u64("GB_WORKSPACE_OVERHEAD_SECS").unwrap_or(DEFAULT_WORKSPACE_OVERHEAD_SECS)
}

/// Hard cap for the caller's `timeout_ms`; `GB_WORKSPACE_TIMEOUT_MAX_MS` can
/// raise it for a machine that legitimately builds for longer.
pub(super) fn timeout_cap_ms() -> u64 {
    env_u64("GB_WORKSPACE_TIMEOUT_MAX_MS").unwrap_or(DEFAULT_TIMEOUT_CAP_MS)
}

/// Overall budget for one `vm_exec` call, in milliseconds.
///
/// `GB_WORKSPACE_BUDGET_MS` sets an absolute floor (useful for an
/// always-on node with slow storage); otherwise the program timeout plus the
/// configurable overhead.
pub(super) fn workspace_budget_ms(timeout_ms: u64) -> u64 {
    budget_from(
        timeout_ms,
        workspace_overhead_secs(),
        env_u64("GB_WORKSPACE_BUDGET_MS"),
    )
}

/// Pure budget arithmetic, so the floors and caps are testable without touching
/// process-wide environment variables.
fn budget_from(timeout_ms: u64, overhead_secs: u64, absolute: Option<u64>) -> u64 {
    let computed = timeout_ms.saturating_add(overhead_secs.saturating_mul(1_000));
    match absolute {
        Some(absolute) => computed.max(absolute),
        None => computed,
    }
}

/// A run timeout means the program took too long; it says nothing about the
/// sandbox host being down, so it must not advise a restart or a wait.
fn budget_exhausted_message(timeout_ms: u64, last_step: &str) -> String {
    let budget_secs = workspace_budget_ms(timeout_ms) / 1_000;
    let overhead = workspace_overhead_secs();
    format!(
        "The workspace run exceeded its {budget_secs}-second budget while in '{last_step}' ({timeout_ms} ms program timeout plus {overhead} s of workspace overhead). This is a run timeout, not a host outage. Retry, reduce the task scope or split it into smaller steps, and check the budget: raise `timeout_ms` (max {} ms, GB_WORKSPACE_TIMEOUT_MAX_MS), raise the overhead with GB_WORKSPACE_OVERHEAD_SECS, or pin a floor with GB_WORKSPACE_BUDGET_MS. Files the program already wrote are kept in the workspace.",
        timeout_cap_ms()
    )
}

fn is_budget_exhausted(reason: &str) -> bool {
    reason.to_ascii_lowercase().contains("run budget elapsed")
}

/// The truthful replacement for the dead-end "workspace is not available"
/// phrase: show the concrete reason the VM subsystem reports (status is
/// logged only, never shown), then one short pointer forward. A run-budget
/// timeout is a distinct failure and gets its own guidance.
pub(super) fn workspace_failure_message(reason: &str, timeout_ms: u64) -> String {
    if is_budget_exhausted(reason) {
        // #2045 — the reason carries the stage that was running, so the user
        // learns where the time went instead of only the total.
        let last_step = reason
            .split("during '")
            .nth(1)
            .and_then(|rest| rest.split('\'').next())
            .unwrap_or("execute");
        return budget_exhausted_message(timeout_ms, last_step);
    }
    let trimmed = reason.trim();
    if trimmed.is_empty() {
        return "The workspace could not run this program. Start a new message and try again."
            .to_string();
    }
    let clipped: String = trimmed.chars().take(320).collect();
    let clipped = if trimmed.chars().count() > 320 {
        format!("{clipped}…")
    } else {
        clipped
    };
    // The reason is already actionable (a cold start says it is still pulling
    // the image; a host problem names the missing Incus setup). Do not append
    // the old "restart the app" advice: it made a transient first-run race look
    // like a permanently dead sandbox host (issue #1798).
    format!("The workspace could not run this program: {clipped}. Start a new message to retry.")
}

#[cfg(test)]
mod workspace_failure_message_tests;

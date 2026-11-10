//! #2045 — the workspace run budget, resolved instead of hardcoded.
//!
//! A workspace run pays for a cold image pull, container start, package
//! install and (often) a real build before the program produces its first
//! byte. A fixed 180 s window (60 s program + 120 s overhead) cannot cover
//! that, so every value is resolved from the environment with a documented
//! default:
//!
//! | Variable | Meaning | Default |
//! |---|---|---|
//! | `GB_WORKSPACE_PROGRAM_TIMEOUT_MS` | program time when the caller asks for none | 300 000 |
//! | `GB_WORKSPACE_TIMEOUT_MAX_MS` | ceiling for a caller-supplied timeout | 600 000 |
//! | `GB_WORKSPACE_OVERHEAD_SECS` | provisioning/wake overhead added to the program time | 120 |
//! | `GB_WORKSPACE_BUDGET_MS` | absolute floor for the whole run | none |
//!
//! [`HARD_CAP_MS`] is not configurable: a looping program must always be cut
//! off, so 15 minutes is the ceiling no override can raise.

use std::time::Instant;

/// Program time granted when the caller does not ask for one. 5 minutes lets
/// a cold start plus a dependency install plus a small build finish.
pub const DEFAULT_PROGRAM_TIMEOUT_MS: u64 = 300_000;

/// Ceiling for a caller-supplied timeout (`timeout_ms`).
pub const DEFAULT_TIMEOUT_CAP_MS: u64 = 600_000;

/// Provisioning/wake overhead added to the program time.
pub const DEFAULT_WORKSPACE_OVERHEAD_SECS: u64 = 120;

/// Absolute ceiling for one workspace run, in milliseconds (15 minutes).
pub const HARD_CAP_MS: u64 = 900_000;

/// Parses a positive integer out of an optional environment value. Blank,
/// non-numeric and zero values mean "not set", so a misconfigured override
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

/// The resolved budget of one workspace run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkspaceBudget {
    /// Time the program itself may run.
    pub program_ms: u64,
    /// Provisioning/wake overhead added on top of the program time.
    pub overhead_ms: u64,
    /// Whole-run budget (`program_ms + overhead_ms`, at least the floor).
    pub total_ms: u64,
}

impl WorkspaceBudget {
    /// Builds a budget from its two parts, applying the hard cap.
    pub fn new(program_ms: u64, overhead_ms: u64) -> Self {
        let program_ms = program_ms.max(1).min(HARD_CAP_MS);
        let overhead_ms = overhead_ms.min(HARD_CAP_MS.saturating_sub(program_ms));
        Self {
            program_ms,
            overhead_ms,
            total_ms: program_ms.saturating_add(overhead_ms).min(HARD_CAP_MS),
        }
    }

    /// Milliseconds consumed since `started`.
    pub fn elapsed_ms(&self, started: Instant) -> u64 {
        started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64
    }

    /// Milliseconds left before the budget is exhausted (never negative).
    pub fn remaining_ms(&self, started: Instant) -> u64 {
        self.total_ms.saturating_sub(self.elapsed_ms(started))
    }

    /// True once the whole-run budget is spent.
    pub fn is_exhausted(&self, started: Instant) -> bool {
        self.elapsed_ms(started) >= self.total_ms
    }

    /// One-line description shown in the run card next to the stage.
    pub fn describe(&self) -> String {
        format!(
            "budget {}s ({}s program + {}s overhead)",
            self.total_ms / 1_000,
            self.program_ms / 1_000,
            self.overhead_ms / 1_000
        )
    }

    /// Truthful failure text for a run cut off by the budget: names the step
    /// that was running and the artifacts that survived, and points at the
    /// overrides instead of telling the user to shrink the task.
    pub fn timeout_message(&self, in_flight: &str, kept_artifacts: &[String]) -> String {
        let kept = if kept_artifacts.is_empty() {
            "No files were written before the cut-off.".to_string()
        } else {
            let listed: Vec<&str> = kept_artifacts.iter().take(8).map(String::as_str).collect();
            format!("Kept in the workspace: {}.", listed.join(", "))
        };
        format!(
            "The workspace run exceeded its {total}-second budget while running '{in_flight}' \
             ({program} ms program timeout plus {overhead} s of workspace overhead). \
             This is a run timeout, not a host outage: the stage that was in flight and every \
             step that already finished are shown above. {kept} Raise the budget instead of \
             splitting the work: `timeout_ms` (max {cap} ms, GB_WORKSPACE_TIMEOUT_MAX_MS), \
             GB_WORKSPACE_OVERHEAD_SECS for cold starts, or GB_WORKSPACE_BUDGET_MS for a floor.",
            total = self.total_ms / 1_000,
            program = self.program_ms,
            overhead = self.overhead_ms / 1_000,
            cap = DEFAULT_TIMEOUT_CAP_MS,
        )
    }
}

/// Environment-provided limits, kept apart from [`WorkspaceBudget`] so the
/// arithmetic is testable without touching process-wide variables.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BudgetLimits {
    pub program_ms: u64,
    pub timeout_cap_ms: u64,
    pub overhead_ms: u64,
    pub floor_ms: Option<u64>,
}

impl Default for BudgetLimits {
    fn default() -> Self {
        Self {
            program_ms: DEFAULT_PROGRAM_TIMEOUT_MS,
            timeout_cap_ms: DEFAULT_TIMEOUT_CAP_MS,
            overhead_ms: DEFAULT_WORKSPACE_OVERHEAD_SECS.saturating_mul(1_000),
            floor_ms: None,
        }
    }
}

impl BudgetLimits {
    /// Reads the limits from the documented environment variables.
    pub fn from_env() -> Self {
        Self::from_values(
            env_u64("GB_WORKSPACE_PROGRAM_TIMEOUT_MS"),
            env_u64("GB_WORKSPACE_TIMEOUT_MAX_MS"),
            env_u64("GB_WORKSPACE_OVERHEAD_SECS"),
            env_u64("GB_WORKSPACE_BUDGET_MS"),
        )
    }

    /// Raw-value form: `None` keeps the default, a positive value overrides.
    pub fn from_values(
        program_ms: Option<u64>,
        timeout_cap_ms: Option<u64>,
        overhead_secs: Option<u64>,
        floor_ms: Option<u64>,
    ) -> Self {
        let defaults = Self::default();
        Self {
            program_ms: program_ms.unwrap_or(defaults.program_ms),
            timeout_cap_ms: timeout_cap_ms.unwrap_or(defaults.timeout_cap_ms),
            overhead_ms: overhead_secs
                .map(|secs| secs.saturating_mul(1_000))
                .unwrap_or(defaults.overhead_ms),
            floor_ms,
        }
    }

    /// Resolves the budget for one run. A caller-supplied timeout is clamped
    /// to [`Self::timeout_cap_ms`]; the whole run never exceeds
    /// [`HARD_CAP_MS`].
    pub fn resolve(&self, requested_ms: Option<u64>) -> WorkspaceBudget {
        let requested = requested_ms.unwrap_or(self.program_ms);
        let program_ms = requested.min(self.timeout_cap_ms).max(1);
        let mut budget = WorkspaceBudget::new(program_ms, self.overhead_ms);
        if let Some(floor) = self.floor_ms {
            if floor > budget.total_ms {
                let total = floor.min(HARD_CAP_MS);
                budget.overhead_ms = total.saturating_sub(budget.program_ms);
                budget.total_ms = total;
            }
        }
        budget
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn defaults_cover_a_cold_build_instead_of_180_seconds() {
        let budget = BudgetLimits::default().resolve(None);
        assert_eq!(budget.program_ms, DEFAULT_PROGRAM_TIMEOUT_MS);
        assert_eq!(budget.overhead_ms, 120_000);
        assert_eq!(budget.total_ms, 420_000);
        assert!(
            budget.total_ms > 180_000,
            "the old 180 s window must no longer be the default"
        );
        assert!(budget.describe().contains("420s"));
        assert!(budget.describe().contains("300s program"));
    }

    #[test]
    fn a_caller_timeout_is_clamped_to_the_cap() {
        let limits = BudgetLimits::default();
        assert_eq!(limits.resolve(Some(240_000)).program_ms, 240_000);
        // 15 minutes requested, 10 minutes granted.
        assert_eq!(
            limits.resolve(Some(900_000)).program_ms,
            DEFAULT_TIMEOUT_CAP_MS
        );
        // 0 is treated as "the smallest possible run", never as unlimited.
        assert_eq!(limits.resolve(Some(0)).program_ms, 1);
    }

    #[test]
    fn env_overrides_raise_the_budget_but_never_past_the_hard_cap() {
        let limits = BudgetLimits::from_values(Some(480_000), Some(900_000), Some(300), None);
        let budget = limits.resolve(None);
        assert_eq!(budget.program_ms, 480_000);
        assert_eq!(budget.overhead_ms, 300_000);
        assert_eq!(budget.total_ms, 780_000);

        let greedy = BudgetLimits::from_values(Some(3_600_000), Some(3_600_000), Some(3_600), None);
        let capped = greedy.resolve(Some(3_600_000));
        assert_eq!(capped.total_ms, HARD_CAP_MS);
        assert_eq!(capped.program_ms + capped.overhead_ms, HARD_CAP_MS);
    }

    #[test]
    fn a_floor_applies_and_is_capped() {
        let limits = BudgetLimits::from_values(None, None, None, Some(500_000));
        let budget = limits.resolve(Some(60_000));
        assert_eq!(budget.program_ms, 60_000);
        assert_eq!(budget.total_ms, 500_000);

        let absurd = BudgetLimits::from_values(None, None, None, Some(10_000_000));
        assert_eq!(absurd.resolve(None).total_ms, HARD_CAP_MS);
    }

    #[test]
    fn blank_and_invalid_overrides_fall_back_to_the_default() {
        assert_eq!(parse_positive_u64(Some("")), None);
        assert_eq!(parse_positive_u64(Some("   ")), None);
        assert_eq!(parse_positive_u64(Some("0")), None);
        assert_eq!(parse_positive_u64(Some("soon")), None);
        assert_eq!(parse_positive_u64(Some("45000")), Some(45_000));
    }

    #[test]
    fn elapsed_and_remaining_are_measured_from_the_run_start() {
        let budget = WorkspaceBudget::new(10_000, 5_000);
        let started = Instant::now();
        assert_eq!(budget.elapsed_ms(started), 0);
        assert_eq!(budget.remaining_ms(started), 15_000);
        assert!(!budget.is_exhausted(started));

        let in_the_past = started
            .checked_sub(Duration::from_secs(20))
            .unwrap_or(started);
        assert!(budget.is_exhausted(in_the_past));
        assert_eq!(budget.remaining_ms(in_the_past), 0);
        assert!(budget.elapsed_ms(in_the_past) >= 15_000);
    }

    #[test]
    fn the_timeout_message_names_the_step_and_the_kept_artifacts() {
        let budget = BudgetLimits::default().resolve(None);
        let message = budget.timeout_message(
            "Build and test",
            &["build/app.js".to_string(), "logs/build.log".to_string()],
        );
        assert!(message.contains("Build and test"));
        assert!(message.contains("420-second budget"));
        assert!(message.contains("build/app.js"));
        assert!(message.contains("logs/build.log"));
        assert!(message.contains("GB_WORKSPACE_BUDGET_MS"));
        assert!(!message.contains("reduce the task scope"));

        let empty = budget.timeout_message("Execute plan", &[]);
        assert!(empty.contains("Execute plan"));
        assert!(empty.contains("No files were written"));
    }
}

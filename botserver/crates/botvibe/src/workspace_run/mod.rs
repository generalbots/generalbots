//! #2045 / #2046 — workspace runs that survive a real build.
//!
//! Two pieces the chat pipeline needs for a workspace run:
//!
//! - [`budget`] resolves the run budget from the environment (program time,
//!   provisioning overhead, ceiling and floor) and turns a cut-off into a
//!   truthful message naming the in-flight step and the kept artifacts.
//! - [`trace`] turns the program's `@@GB_TRACE@@` markers into a task/time
//!   table, kept after a failure or a timeout so the run card can render it.
//!
//! [`TraceRecorder::run`] joins them: it drives a step under the budget and,
//! when the budget elapses, closes the open rows as `timed_out` instead of
//! discarding the progress.

mod budget;
#[cfg(test)]
mod tests;
mod trace;
#[cfg(test)]
mod trace_tests;

pub use budget::{
    BudgetLimits, WorkspaceBudget, DEFAULT_PROGRAM_TIMEOUT_MS, DEFAULT_TIMEOUT_CAP_MS,
    DEFAULT_WORKSPACE_OVERHEAD_SECS, HARD_CAP_MS,
};
pub use trace::{
    parse_trace_line, step_id_for, TraceMarker, TraceRecorder, TraceRow, TraceStatus, TraceTable,
    TRACE_PREFIX,
};

use std::future::Future;
use std::time::{Duration, Instant};

/// How a step driven under a [`WorkspaceBudget`] ended.
#[derive(Debug, Clone)]
pub enum BudgetOutcome<T> {
    /// The step finished on its own.
    Completed { value: T, elapsed_ms: u64 },
    /// The step returned an error before the budget elapsed.
    Failed { error: String, elapsed_ms: u64 },
    /// The budget elapsed; partial progress stays in the recorder.
    TimedOut {
        budget: WorkspaceBudget,
        elapsed_ms: u64,
    },
}

impl<T> BudgetOutcome<T> {
    pub fn timed_out(&self) -> bool {
        matches!(self, Self::TimedOut { .. })
    }

    pub fn elapsed_ms(&self) -> u64 {
        match self {
            Self::Completed { elapsed_ms, .. }
            | Self::Failed { elapsed_ms, .. }
            | Self::TimedOut { elapsed_ms, .. } => *elapsed_ms,
        }
    }
}

impl TraceRecorder {
    /// Drives `future` under this recorder's budget. A step that outlives the
    /// budget is cut off, its open rows are closed as [`TraceStatus::TimedOut`]
    /// and the rows already finished stay in the table — that is the partial
    /// progress a timeout must not throw away.
    pub async fn run<F, T>(&mut self, future: F) -> BudgetOutcome<T>
    where
        F: Future<Output = Result<T, String>>,
    {
        let started = Instant::now();
        let budget = self.budget();
        match tokio::time::timeout(Duration::from_millis(budget.total_ms), future).await {
            Ok(Ok(value)) => BudgetOutcome::Completed {
                value,
                elapsed_ms: elapsed(started),
            },
            Ok(Err(error)) => BudgetOutcome::Failed {
                error,
                elapsed_ms: elapsed(started),
            },
            Err(_) => {
                self.mark_timed_out();
                self.close_open(TraceStatus::TimedOut, "interrupted by the run budget");
                BudgetOutcome::TimedOut {
                    budget,
                    elapsed_ms: elapsed(started),
                }
            }
        }
    }
}

fn elapsed(started: Instant) -> u64 {
    started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64
}

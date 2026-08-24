//! Step and cost budget enforcement for browser tasks.

use crate::policy::PolicyConfig;
use std::fmt;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Hard caps derived from the policy configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BudgetCaps {
    pub max_steps: u32,
    pub max_cost_milli: u64,
}

impl BudgetCaps {
    pub fn from_policy(cfg: &PolicyConfig) -> Self {
        Self {
            max_steps: cfg.max_steps,
            max_cost_milli: cfg.max_cost_units,
        }
    }
}

/// Raised when a charge would exceed the configured cap.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BudgetExceeded {
    Steps { used: u32, max: u32 },
    Cost { used_milli: u64, max_milli: u64 },
}

impl fmt::Display for BudgetExceeded {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Steps { used, max } => write!(f, "step budget exceeded: {used}/{max}"),
            Self::Cost { used_milli, max_milli } => {
                write!(f, "cost budget exceeded: {used_milli}/{max_milli} milli-units")
            }
        }
    }
}

impl std::error::Error for BudgetExceeded {}

/// Point-in-time view of consumption against the caps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct BudgetSnapshot {
    pub steps_used: u32,
    pub cost_milli_used: u64,
    pub steps_remaining: u32,
    pub cost_milli_remaining: u64,
}

/// Thread-safe accumulator enforcing step-count and milli-unit cost caps.
///
/// The tracker is process-local; `advance_task` rehydrates it from the
/// persisted progress document on every call so correctness never depends on
/// process lifetime across restarts.
#[derive(Debug)]
pub struct BudgetTracker {
    steps: AtomicU32,
    cost_milli: AtomicU64,
    caps: BudgetCaps,
}

impl BudgetTracker {
    pub fn new(caps: BudgetCaps) -> Self {
        Self::with_usage(caps, 0, 0)
    }

    /// Builds a tracker pre-loaded with usage already persisted in the task
    /// progress document.
    pub fn with_usage(caps: BudgetCaps, steps_used: u32, cost_milli_used: u64) -> Self {
        Self {
            steps: AtomicU32::new(steps_used),
            cost_milli: AtomicU64::new(cost_milli_used),
            caps,
        }
    }

    /// Charges one step. Returns the new cumulative step count.
    pub fn charge_step(&self) -> Result<u32, BudgetExceeded> {
        let used = self.steps.fetch_add(1, Ordering::SeqCst) + 1;
        if used > self.caps.max_steps {
            return Err(BudgetExceeded::Steps {
                used,
                max: self.caps.max_steps,
            });
        }
        Ok(used)
    }

    /// Charges a cost delta expressed in milli-units (1 unit = 1000 milli).
    /// Returns the new cumulative cost.
    pub fn charge_cost(&self, delta_milli: u64) -> Result<u64, BudgetExceeded> {
        let used = self.cost_milli.fetch_add(delta_milli, Ordering::SeqCst) + delta_milli;
        if used > self.caps.max_cost_milli {
            return Err(BudgetExceeded::Cost {
                used_milli: used,
                max_milli: self.caps.max_cost_milli,
            });
        }
        Ok(used)
    }

    pub fn snapshot(&self) -> BudgetSnapshot {
        let steps_used = self.steps.load(Ordering::SeqCst);
        let cost_milli_used = self.cost_milli.load(Ordering::SeqCst);
        BudgetSnapshot {
            steps_used,
            cost_milli_used,
            steps_remaining: self.caps.max_steps.saturating_sub(steps_used),
            cost_milli_remaining: self.caps.max_cost_milli.saturating_sub(cost_milli_used),
        }
    }

    pub fn caps(&self) -> BudgetCaps {
        self.caps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn caps() -> BudgetCaps {
        BudgetCaps {
            max_steps: 3,
            max_cost_milli: 500,
        }
    }

    #[test]
    fn charges_within_caps_and_reports_snapshot() {
        let t = BudgetTracker::new(caps());
        assert_eq!(t.charge_step(), Ok(1));
        assert_eq!(t.charge_step(), Ok(2));
        assert_eq!(t.charge_cost(250), Ok(250));
        assert_eq!(t.charge_cost(250), Ok(500));
        let s = t.snapshot();
        assert_eq!(s.steps_used, 2);
        assert_eq!(s.cost_milli_used, 500);
        assert_eq!(s.steps_remaining, 1);
        assert_eq!(s.cost_milli_remaining, 0);
    }

    #[test]
    fn rejects_step_over_cap() {
        let t = BudgetTracker::new(caps());
        for _ in 0..3 {
            assert!(t.charge_step().is_ok());
        }
        assert_eq!(
            t.charge_step(),
            Err(BudgetExceeded::Steps { used: 4, max: 3 })
        );
    }

    #[test]
    fn rejects_cost_over_cap() {
        let t = BudgetTracker::new(caps());
        assert_eq!(t.charge_cost(400), Ok(400));
        assert_eq!(
            t.charge_cost(200),
            Err(BudgetExceeded::Cost { used_milli: 600, max_milli: 500 })
        );
    }

    #[test]
    fn rehydrated_usage_counts_toward_cap() {
        let t = BudgetTracker::with_usage(caps(), 3, 0);
        assert!(t.charge_step().is_err());
    }

    #[test]
    fn caps_derive_from_policy_defaults() {
        let c = BudgetCaps::from_policy(&PolicyConfig::default());
        assert_eq!(c.max_steps, 60);
        assert_eq!(c.max_cost_milli, 10_000);
    }
}

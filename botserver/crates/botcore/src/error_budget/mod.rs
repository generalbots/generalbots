//! Pure, dependency-free primitives for the issue #2543 error-budget ratchet.
//!
//! The companion scanner (`scripts/check-error-budget.py`) emits per-crate
//! counts of error-handling debt proxies. CI feeds those counts to
//! [`Budget::evaluate`] together with the agreed baseline; the budget only
//! allows counts that stay equal to or fall below that baseline. There is no
//! I/O, no global state and no external crate here, so the decision is fully
//! deterministic and unit-testable.

mod budget;

pub use budget::{Budget, Decision, ErrorCounts, RULE_NAMES};

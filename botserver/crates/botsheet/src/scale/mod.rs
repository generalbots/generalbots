//! #2435 — pure, offline primitives for scaling sheets to enterprise workbooks.
//!
//! This module holds the allocation-free policy layer shared by the windowed
//! read handler and (later) the streaming importer, calculation worker and
//! guardrails. It deliberately depends only on `std`: no I/O, no database and
//! no runtime state, so every rule can be unit-tested without a server.
//!
//! * [`MemoryBudget`] assigns hard per-tier limits (cells, formulas, bytes,
//!   tile geometry). A tier is chosen with [`WorkbookTier::classify`].
//! * [`plan_window`] partitions a requested A1 region into bounded fetch tiles
//!   with deterministic ids, clamped to the sheet and ranked by viewport.
//! * [`estimate_recalc`] prices an incremental recalculation and refuses work
//!   that would exceed the budget, returning an explicit [`BudgetError`].

mod budget;
mod planner;
mod recalc;

pub use budget::{BudgetError, MemoryBudget, WorkbookTier};
pub use planner::{CellRect, FetchTile, WindowPlanRequest, plan_window, total_cells};
pub use recalc::{RecalcEstimate, RecalcRequest, estimate_recalc};

#[cfg(test)]
mod tests;

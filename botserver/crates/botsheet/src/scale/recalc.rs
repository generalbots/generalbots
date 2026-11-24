//! #2435 — recalculation cost estimation with explicit budget refusal.
//!
//! Incremental recalculation only visits the edited cells plus the formula
//! nodes the dependency graph says are affected. This module prices that work
//! before it starts: if the estimate exceeds the tier's recalculation byte
//! budget (or formula ceiling), the caller gets a [`BudgetError`] it can show
//! to the user instead of an out-of-memory kill of the whole server process.

use crate::scale::budget::{BudgetError, MemoryBudget};

/// The work a recalculation request would perform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecalcRequest {
    /// Cells directly edited by the user.
    pub dirty_cells: u64,
    /// Formula cells reachable from the dirty set through the dependency graph.
    pub dependent_formulas: u64,
    /// Total formulas in the workbook; bounds the dependents and sets the scale.
    pub total_formulas: u64,
    /// Estimated bytes needed per evaluated cell (value + intermediates).
    pub bytes_per_cell: u64,
}

/// A priced, bounded recalculation plan.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RecalcEstimate {
    /// Number of cells that must be evaluated.
    pub cells_to_evaluate: u64,
    /// Peak bytes the evaluation is expected to hold.
    pub estimated_bytes: u64,
    /// Share of the workbook recalculated, in basis points (0..=10_000).
    pub workbook_fraction_bps: u32,
    /// True when fewer than all workbook formulas need recomputing.
    pub incremental: bool,
}

/// Price an incremental recalculation, refusing any work over the budget.
///
/// All arithmetic saturates, so even `u64::MAX` inputs yield a deterministic
/// refusal rather than a wrapping panic.
pub fn estimate_recalc(
    request: &RecalcRequest,
    budget: &MemoryBudget,
) -> Result<RecalcEstimate, BudgetError> {
    let dependents = request.dependent_formulas.min(request.total_formulas);
    let cells_to_evaluate = request.dirty_cells.saturating_add(dependents);

    if cells_to_evaluate > budget.max_formulas {
        return Err(BudgetError::TooManyFormulas {
            requested: cells_to_evaluate,
            max: budget.max_formulas,
        });
    }

    let estimated_bytes = cells_to_evaluate.saturating_mul(request.bytes_per_cell);
    if estimated_bytes > budget.max_recalc_bytes {
        return Err(BudgetError::RecalcOverBudget {
            estimated_bytes,
            max_bytes: budget.max_recalc_bytes,
        });
    }

    let workbook_fraction_bps = if request.total_formulas == 0 {
        0
    } else {
        let bps = cells_to_evaluate.saturating_mul(10_000) / request.total_formulas;
        match u32::try_from(bps.min(10_000)) {
            Ok(value) => value,
            Err(_) => 10_000,
        }
    };

    Ok(RecalcEstimate {
        cells_to_evaluate,
        estimated_bytes,
        workbook_fraction_bps,
        incremental: cells_to_evaluate < request.total_formulas,
    })
}

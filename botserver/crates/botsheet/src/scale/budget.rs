//! #2435 — memory budgets for enterprise workbooks.
//!
//! Every scale primitive (window tiles, recalculation, cache sizing) is sized
//! from a [`MemoryBudget`] chosen by [`WorkbookTier`]. Keeping the limits in one
//! pure model lets a handler refuse an over-budget request with an explicit
//! [`BudgetError`] long before it can allocate enough memory to OOM the process.

use std::fmt;

/// Coarse workbook size classes. Larger tiers allow more resident state and
/// bigger fetch tiles, but every tier remains strictly bounded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum WorkbookTier {
    Small,
    Medium,
    Large,
}

impl WorkbookTier {
    /// Smallest tier whose cell/formula ceilings fit the workbook. A workbook
    /// beyond even the large ceilings is still classified `Large`; the import
    /// guardrail and the cost estimator then reject the excess explicitly.
    pub fn classify(cells: u64, formulas: u64) -> Self {
        for tier in [Self::Small, Self::Medium, Self::Large] {
            let budget = MemoryBudget::for_tier(tier);
            if cells <= budget.max_cells && formulas <= budget.max_formulas {
                return tier;
            }
        }
        Self::Large
    }
}

/// All hard limits applied to a workbook of a given tier.
///
/// Instances are only built by [`MemoryBudget::for_tier`] (or copied from it),
/// so production code cannot accidentally construct a zero-sized tile that
/// would panic on division. Every size is an integer number of bytes/cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryBudget {
    pub tier: WorkbookTier,
    /// Maximum populated cells tracked in memory.
    pub max_cells: u64,
    /// Maximum formula cells a workbook may contain.
    pub max_formulas: u64,
    /// Maximum resident bytes for the whole workbook session.
    pub max_resident_bytes: u64,
    /// Slice of the resident budget reserved for one recalculation.
    pub max_recalc_bytes: u64,
    /// Maximum bytes held in the window tile cache.
    pub max_window_cache_bytes: u64,
    /// Maximum number of fetch tiles returned by one window plan.
    pub max_window_tiles: u32,
    /// Rows per fetch tile (must be > 0).
    pub tile_rows: u32,
    /// Columns per fetch tile (must be > 0).
    pub tile_cols: u32,
}

const KIB: u64 = 1_024;
const MIB: u64 = 1_024 * KIB;

impl MemoryBudget {
    /// The canonical, immutable budget for a tier.
    pub const fn for_tier(tier: WorkbookTier) -> Self {
        match tier {
            WorkbookTier::Small => Self {
                tier,
                max_cells: 10_000,
                max_formulas: 1_000,
                max_resident_bytes: 64 * MIB,
                max_recalc_bytes: 8 * MIB,
                max_window_cache_bytes: 8 * MIB,
                max_window_tiles: 32,
                tile_rows: 200,
                tile_cols: 64,
            },
            WorkbookTier::Medium => Self {
                tier,
                max_cells: 500_000,
                max_formulas: 50_000,
                max_resident_bytes: 256 * MIB,
                max_recalc_bytes: 32 * MIB,
                max_window_cache_bytes: 32 * MIB,
                max_window_tiles: 64,
                tile_rows: 500,
                tile_cols: 128,
            },
            WorkbookTier::Large => Self {
                tier,
                max_cells: 5_000_000,
                max_formulas: 200_000,
                max_resident_bytes: 1_024 * MIB,
                max_recalc_bytes: 128 * MIB,
                max_window_cache_bytes: 128 * MIB,
                max_window_tiles: 128,
                tile_rows: 1_000,
                tile_cols: 256,
            },
        }
    }

    /// Guardrail for import/load: refuse a workbook no tier can hold instead
    /// of letting it exhaust server memory.
    pub fn within_cells_and_formulas(&self, cells: u64, formulas: u64) -> Result<(), BudgetError> {
        if cells > self.max_cells {
            return Err(BudgetError::TooManyCells {
                requested: cells,
                max: self.max_cells,
            });
        }
        if formulas > self.max_formulas {
            return Err(BudgetError::TooManyFormulas {
                requested: formulas,
                max: self.max_formulas,
            });
        }
        Ok(())
    }
}

/// Every refusal in the scale layer is one of these. Each variant carries the
/// numbers a handler can turn into a clear user-facing message ("workbook is
/// too large"), not an opaque OOM.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BudgetError {
    InvalidBudget,
    RecalcOverBudget {
        estimated_bytes: u64,
        max_bytes: u64,
    },
    TooManyCells {
        requested: u64,
        max: u64,
    },
    TooManyFormulas {
        requested: u64,
        max: u64,
    },
    TooManyTiles {
        requested: u32,
        max: u32,
    },
}

impl fmt::Display for BudgetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidBudget => {
                write!(f, "workbook memory budget has zero-sized fetch tiles")
            }
            Self::RecalcOverBudget {
                estimated_bytes,
                max_bytes,
            } => write!(
                f,
                "recalculation would need {estimated_bytes} bytes, above the {max_bytes}-byte budget"
            ),
            Self::TooManyCells { requested, max } => {
                write!(
                    f,
                    "workbook has {requested} cells, above the {max}-cell limit"
                )
            }
            Self::TooManyFormulas { requested, max } => {
                write!(
                    f,
                    "workbook has {requested} formulas, above the {max}-formula limit"
                )
            }
            Self::TooManyTiles { requested, max } => write!(
                f,
                "range needs {requested} fetch tiles, above the {max}-tile window limit"
            ),
        }
    }
}

impl std::error::Error for BudgetError {}

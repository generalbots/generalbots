//! #2435 — deterministic, bounded fetch-tile planning for windowed reads.
//!
//! A client asks for a region around its viewport; the planner splits that
//! region into fixed-size tiles, clamps it to the real sheet, ranks tiles that
//! intersect the viewport first and gives every tile a stable id. Because tile
//! geometry comes from the [`MemoryBudget`], a 1M-row sheet never produces a
//! plan larger than `max_window_tiles` tiles of at most `tile_rows × tile_cols`
//! cells each.

use crate::scale::budget::{BudgetError, MemoryBudget};
use std::cmp::Ordering;

/// A 0-based inclusive rectangle of rows and columns, matching the coordinates
/// used by the windowed-read handler (`WindowRange`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellRect {
    pub first_row: u32,
    pub first_col: u32,
    pub last_row: u32,
    pub last_col: u32,
}

impl CellRect {
    /// Number of cells in the rectangle, saturating instead of overflowing.
    pub fn cell_count(&self) -> u64 {
        if self.is_empty() {
            return 0;
        }
        let rows = u64::from(self.last_row - self.first_row + 1);
        let cols = u64::from(self.last_col - self.first_col + 1);
        rows.saturating_mul(cols)
    }

    /// True when the rectangle spans no rows or no columns.
    pub fn is_empty(&self) -> bool {
        self.first_row > self.last_row || self.first_col > self.last_col
    }

    /// Inclusive overlap of two rectangles, or `None` when they are disjoint.
    pub fn intersection(&self, other: &CellRect) -> Option<Self> {
        let first_row = self.first_row.max(other.first_row);
        let last_row = self.last_row.min(other.last_row);
        let first_col = self.first_col.max(other.first_col);
        let last_col = self.last_col.min(other.last_col);
        if first_row > last_row || first_col > last_col {
            None
        } else {
            Some(Self {
                first_row,
                last_row,
                first_col,
                last_col,
            })
        }
    }

    /// Whether the two rectangles share at least one cell.
    pub fn overlaps(&self, other: &CellRect) -> bool {
        self.intersection(other).is_some()
    }
}

/// What the planner needs to know: the region the client wants, the visible
/// area (for priority) and the sheet's real bounds (for clamping).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowPlanRequest {
    pub requested: CellRect,
    pub viewport: CellRect,
    pub sheet_bounds: CellRect,
}

/// One bounded fetch unit. `id` is derived from absolute tile-grid coordinates,
/// so the same region always yields the same id regardless of request order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchTile {
    pub id: String,
    pub rect: CellRect,
    pub intersects_viewport: bool,
}

/// Build a bounded, gap-free, overlap-free tile plan.
///
/// Returns an empty plan when the requested region lies entirely outside the
/// sheet, and a [`BudgetError`] when the tile geometry is invalid or the region
/// would need more than `max_window_tiles` tiles.
pub fn plan_window(
    request: &WindowPlanRequest,
    budget: &MemoryBudget,
) -> Result<Vec<FetchTile>, BudgetError> {
    if budget.tile_rows == 0 || budget.tile_cols == 0 {
        return Err(BudgetError::InvalidBudget);
    }

    let clamped = match request.requested.intersection(&request.sheet_bounds) {
        Some(rect) => rect,
        None => return Ok(Vec::new()),
    };
    let viewport = request.viewport.intersection(&request.sheet_bounds);

    let mut tiles = Vec::new();
    for row in (clamped.first_row..=clamped.last_row).step_by(budget.tile_rows as usize) {
        let last_row = row
            .saturating_add(budget.tile_rows - 1)
            .min(clamped.last_row);
        for col in (clamped.first_col..=clamped.last_col).step_by(budget.tile_cols as usize) {
            let last_col = col
                .saturating_add(budget.tile_cols - 1)
                .min(clamped.last_col);
            let rect = CellRect {
                first_row: row,
                last_row,
                first_col: col,
                last_col,
            };
            let intersects_viewport = viewport.as_ref().map_or(false, |v| rect.overlaps(v));
            let id = format!("r{}c{}", row / budget.tile_rows, col / budget.tile_cols);
            tiles.push(FetchTile {
                id,
                rect,
                intersects_viewport,
            });
        }
    }

    // Stable total order: viewport tiles first, then top-to-bottom,
    // left-to-right. Row/col are unique per tile, so the plan is deterministic.
    tiles.sort_by(
        |a, b| match (a.intersects_viewport, b.intersects_viewport) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            _ => a
                .rect
                .first_row
                .cmp(&b.rect.first_row)
                .then_with(|| a.rect.first_col.cmp(&b.rect.first_col)),
        },
    );

    let requested_tiles = match u32::try_from(tiles.len()) {
        Ok(count) => count,
        Err(_) => u32::MAX,
    };
    if requested_tiles > budget.max_window_tiles {
        return Err(BudgetError::TooManyTiles {
            requested: requested_tiles,
            max: budget.max_window_tiles,
        });
    }
    Ok(tiles)
}

/// Total cells across a plan — useful for sizing the window cache and for
/// asserting a plan fits the budget's byte cap.
pub fn total_cells(tiles: &[FetchTile]) -> u64 {
    tiles
        .iter()
        .fold(0u64, |sum, tile| sum.saturating_add(tile.rect.cell_count()))
}

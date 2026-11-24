//! #2435 — unit tests for the pure scale primitives (no server required).

use super::*;
use std::collections::HashMap;

fn rect(first_row: u32, first_col: u32, last_row: u32, last_col: u32) -> CellRect {
    CellRect {
        first_row,
        first_col,
        last_row,
        last_col,
    }
}

#[test]
fn tiles_cover_requested_region_without_gaps_or_overlaps() {
    let budget = MemoryBudget {
        tile_rows: 2,
        tile_cols: 2,
        ..MemoryBudget::for_tier(WorkbookTier::Small)
    };
    let request = WindowPlanRequest {
        requested: rect(0, 0, 5, 5),
        viewport: rect(0, 0, 1, 1),
        sheet_bounds: rect(0, 0, 1_000, 1_000),
    };
    let tiles = plan_window(&request, &budget).unwrap_or_else(|error| {
        log::warn!("scale: window plan refused: {error}");
        Vec::new()
    });

    let mut visits: HashMap<(u32, u32), u32> = HashMap::new();
    for tile in &tiles {
        for row in tile.rect.first_row..=tile.rect.last_row {
            for col in tile.rect.first_col..=tile.rect.last_col {
                *visits.entry((row, col)).or_insert(0) += 1;
            }
        }
    }
    assert_eq!(tiles.len(), 9);
    assert_eq!(visits.len(), 36);
    assert!(visits.values().all(|count| *count == 1));
    assert_eq!(total_cells(&tiles), 36);
}

#[test]
fn boundary_clamping_handles_partial_and_outside_requests() {
    let budget = MemoryBudget::for_tier(WorkbookTier::Large);
    let sheet = rect(0, 0, 9, 19);

    let oversized = WindowPlanRequest {
        requested: rect(0, 0, 9_999, 9_999),
        viewport: sheet,
        sheet_bounds: sheet,
    };
    let tiles = plan_window(&oversized, &budget).unwrap_or_else(|error| {
        log::warn!("scale: window plan refused: {error}");
        Vec::new()
    });
    assert_eq!(tiles.len(), 1);
    assert_eq!(tiles[0].rect, sheet);

    let partial = WindowPlanRequest {
        requested: rect(8, 18, 50, 50),
        viewport: sheet,
        sheet_bounds: sheet,
    };
    let tiles = plan_window(&partial, &budget).unwrap_or_else(|error| {
        log::warn!("scale: window plan refused: {error}");
        Vec::new()
    });
    assert_eq!(tiles.len(), 1);
    assert_eq!(tiles[0].rect, rect(8, 18, 9, 19));

    let outside = WindowPlanRequest {
        requested: rect(100, 100, 200, 200),
        viewport: sheet,
        sheet_bounds: sheet,
    };
    assert!(
        plan_window(&outside, &budget)
            .unwrap_or_else(|error| {
                log::warn!("scale: window plan refused: {error}");
                Vec::new()
            })
            .is_empty()
    );
}

#[test]
fn budget_refuses_over_budget_recalculation() {
    let small = MemoryBudget::for_tier(WorkbookTier::Small);
    let too_many = RecalcRequest {
        dirty_cells: 1,
        dependent_formulas: 2_000,
        total_formulas: 200_000,
        bytes_per_cell: 16,
    };
    assert_eq!(
        estimate_recalc(&too_many, &small),
        Err(BudgetError::TooManyFormulas {
            requested: 2_001,
            max: 1_000
        })
    );

    let large = MemoryBudget::for_tier(WorkbookTier::Large);
    let too_big = RecalcRequest {
        dirty_cells: 0,
        dependent_formulas: 200_000,
        total_formulas: 200_000,
        bytes_per_cell: 1_024,
    };
    assert_eq!(
        estimate_recalc(&too_big, &large),
        Err(BudgetError::RecalcOverBudget {
            estimated_bytes: 204_800_000,
            max_bytes: large.max_recalc_bytes,
        })
    );
    assert!(matches!(
        large.within_cells_and_formulas(10, 250_000),
        Err(BudgetError::TooManyFormulas { .. })
    ));
}

#[test]
fn large_workbook_plan_stays_under_cap_and_is_deterministic() {
    let budget = MemoryBudget::for_tier(WorkbookTier::Large);
    let request = WindowPlanRequest {
        requested: rect(499_800, 0, 500_239, 9),
        viewport: rect(500_000, 0, 500_039, 9),
        sheet_bounds: rect(0, 0, 999_999, 9),
    };
    let tiles = plan_window(&request, &budget).unwrap_or_else(|error| {
        log::warn!("scale: window plan refused: {error}");
        Vec::new()
    });

    assert_eq!(tiles.len(), 1);
    assert!(tiles.len() as u32 <= budget.max_window_tiles);
    assert_eq!(tiles[0].rect, request.requested);
    assert!(tiles[0].intersects_viewport);
    assert_eq!(tiles[0].id, "r499c0");
    assert_eq!(total_cells(&tiles), 4_400);

    let cap = u64::from(budget.max_window_tiles)
        .saturating_mul(u64::from(budget.tile_rows))
        .saturating_mul(u64::from(budget.tile_cols));
    assert!(total_cells(&tiles) <= cap);
    assert_eq!(tiles, plan_window(&request, &budget).unwrap_or_else(|error| {
        log::warn!("scale: window plan refused: {error}");
        Vec::new()
    }));
}

#[test]
fn viewport_tiles_are_ranked_first() {
    let budget = MemoryBudget {
        tile_rows: 2,
        tile_cols: 2,
        ..MemoryBudget::for_tier(WorkbookTier::Small)
    };
    let sheet = rect(0, 0, 5, 5);
    let request = WindowPlanRequest {
        requested: sheet,
        viewport: rect(4, 4, 5, 5),
        sheet_bounds: sheet,
    };
    let tiles = plan_window(&request, &budget).unwrap_or_else(|error| {
        log::warn!("scale: window plan refused: {error}");
        Vec::new()
    });

    assert_eq!(tiles.len(), 9);
    assert!(tiles[0].intersects_viewport);
    assert_eq!(tiles[0].rect, rect(4, 4, 5, 5));
    assert_eq!(
        tiles.iter().filter(|tile| tile.intersects_viewport).count(),
        1
    );
}

#[test]
fn incremental_estimate_prices_only_the_affected_cells() {
    let budget = MemoryBudget::for_tier(WorkbookTier::Large);
    let request = RecalcRequest {
        dirty_cells: 1,
        dependent_formulas: 99,
        total_formulas: 200_000,
        bytes_per_cell: 64,
    };
    let estimate = estimate_recalc(&request, &budget).unwrap_or_else(|error| {
        log::warn!("scale: recalc estimate refused: {error}");
        RecalcEstimate::default()
    });

    assert_eq!(estimate.cells_to_evaluate, 100);
    assert_eq!(estimate.estimated_bytes, 6_400);
    assert!(estimate.incremental);
    assert_eq!(estimate.workbook_fraction_bps, 5);
    assert!(estimate.estimated_bytes <= budget.max_recalc_bytes);
}

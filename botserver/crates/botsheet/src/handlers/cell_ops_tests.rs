//! #2435 — cell-edit guardrails: protection rules and recalculation budget.

use super::*;

fn protection(protected: bool, locked: &[&str]) -> botsheet_core::types::SheetProtection {
    botsheet_core::types::SheetProtection {
        protected,
        locked_cells: locked.iter().map(|cell| cell.to_string()).collect(),
        ..Default::default()
    }
}

#[test]
fn unprotected_sheet_allows_every_cell() {
    assert!(!cell_edit_blocked(Some(&protection(false, &[])), 0, 0));
    assert!(!cell_edit_blocked(None, 5, 5));
}

#[test]
fn protected_sheet_locks_every_cell_by_default() {
    assert!(cell_edit_blocked(Some(&protection(true, &[])), 0, 0));
    assert!(cell_edit_blocked(Some(&protection(true, &[])), 9, 3));
}

#[test]
fn protected_sheet_can_lock_an_explicit_list() {
    let p = protection(true, &["1,1", "2,0"]);
    assert!(cell_edit_blocked(Some(&p), 1, 1));
    assert!(cell_edit_blocked(Some(&p), 2, 0));
    assert!(!cell_edit_blocked(Some(&p), 0, 0));
}

#[test]
fn recalc_budget_refuses_over_budget_cascade() {
    let result = guard_recalc_budget(5_000_000, 200_000, 1, 200_000);
    assert!(result.is_err(), "a full-workbook fan-out must be refused");
    if let Err((status, body)) = result {
        assert_eq!(status, StatusCode::BAD_REQUEST);
        let message = body.0["error"].as_str().unwrap_or("");
        assert!(message.contains("Recalculation refused"));
        assert!(message.contains("too many dependent formulas"));
    }
}

#[test]
fn preflight_allows_within_budget_incremental_cascade() {
    let mut data = HashMap::new();
    data.insert(
        "0,0".to_string(),
        CellData {
            formula: Some("=5".to_string()),
            ..Default::default()
        },
    );
    data.insert(
        "0,1".to_string(),
        CellData {
            formula: Some("=A1+1".to_string()),
            ..Default::default()
        },
    );
    data.insert(
        "0,2".to_string(),
        CellData {
            formula: Some("=10".to_string()),
            ..Default::default()
        },
    );
    let worksheet = Worksheet {
        data,
        ..Default::default()
    };

    let result = preflight_recalc(&worksheet, &[(0, 0)]);
    assert!(result.is_ok(), "small cascade must pass the budget");
    if let Ok(estimate) = result {
        assert_eq!(estimate.cells_to_evaluate, 2);
        assert_eq!(estimate.estimated_bytes, 2 * RECALC_BYTES_PER_CELL);
        assert!(estimate.incremental);
        let budget = MemoryBudget::for_tier(WorkbookTier::Small);
        assert!(estimate.estimated_bytes <= budget.max_recalc_bytes);
    }
}

#[test]
fn recalc_budget_saturates_on_extreme_input() {
    let result = guard_recalc_budget(u64::MAX, u64::MAX, u64::MAX, u64::MAX);
    assert!(
        result.is_err(),
        "saturating inputs must refuse, never panic"
    );
    if let Err((status, body)) = result {
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(
            body.0["error"]
                .as_str()
                .unwrap_or("")
                .contains("Recalculation refused")
        );
    }
    assert!(
        guard_recalc_budget(0, 0, 0, 0).is_ok(),
        "an empty sheet has nothing to refuse"
    );
}

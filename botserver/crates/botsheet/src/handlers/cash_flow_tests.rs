//! Unit tests for the deterministic cash-flow orchestration (#2260).

use super::*;
use botsheet_core::engine::CellValue;

fn cell(sheet: &Spreadsheet, row: u32, col: u32) -> &CellData {
    sheet.worksheets[0]
        .data
        .get(&format!("{row},{col}"))
        .expect("cash-flow cell exists")
}

/// A forecast only writes its requested month columns; cells beyond the last
/// month are deliberately absent, so absence must be asserted without the
/// panicking `cell()` helper.
fn has_cell(sheet: &Spreadsheet, row: u32, col: u32) -> bool {
    sheet.worksheets[0]
        .data
        .contains_key(&format!("{row},{col}"))
}

fn default_request() -> CashFlowRequest {
    CashFlowRequest {
        title: None,
        months: None,
        opening_balance: None,
        inflows: None,
        outflows: None,
        negative_color: None,
        positive_color: None,
    }
}

#[test]
fn default_forecast_has_twelve_month_columns_and_a_balance_row() {
    let sheet =
        build_cash_flow_sheet("owner-1", &default_request()).expect("default request builds");

    assert_eq!(sheet.name, "Cash-Flow Forecast");
    for (offset, month) in MONTH_LABELS.iter().enumerate() {
        assert_eq!(
            cell(&sheet, 1, FIRST_DATA_COL + offset as u32)
                .value
                .as_deref(),
            Some(*month)
        );
    }
    assert!(
        !has_cell(&sheet, 1, 13),
        "no column after the twelfth month"
    );
    assert_eq!(
        cell(&sheet, CLOSING_ROW, 0).value.as_deref(),
        Some("Closing Balance")
    );
    assert_eq!(cell(&sheet, NET_ROW, 1).formula.as_deref(), Some("=B4-B5"));
    assert_eq!(
        cell(&sheet, CLOSING_ROW, 1).formula.as_deref(),
        Some("=B3+B6")
    );
    // First month opens with a literal balance; later months chain the prior
    // month's closing balance (=B7 on the second month, =L7 on the twelfth).
    assert_eq!(cell(&sheet, 2, 1).value.as_deref(), Some("5000"));
    assert_eq!(cell(&sheet, 2, 2).formula.as_deref(), Some("=B7"));
    assert_eq!(cell(&sheet, 2, 12).formula.as_deref(), Some("=L7"));
}

#[test]
fn forecast_carries_opposing_red_and_green_rules() {
    let sheet =
        build_cash_flow_sheet("owner-1", &default_request()).expect("default request builds");

    let rules = sheet.worksheets[0]
        .conditional_formats
        .as_ref()
        .expect("conditional rules exist");
    assert_eq!(rules.len(), 2);
    assert_eq!(rules[0].condition, "<0");
    assert_eq!(
        rules[0].style.background.as_deref(),
        Some(DEFAULT_NEGATIVE_COLOR)
    );
    assert_eq!(rules[1].condition, ">0");
    assert_eq!(
        rules[1].style.background.as_deref(),
        Some(DEFAULT_POSITIVE_COLOR)
    );
    for rule in rules {
        assert_eq!(rule.start_row, NET_ROW);
        assert_eq!(rule.end_row, CLOSING_ROW);
        assert_eq!(rule.start_col, FIRST_DATA_COL);
        assert_eq!(rule.end_col, 12);
    }
    // Number formats are a second, independent negative-red signal.
    assert_eq!(
        cell(&sheet, NET_ROW, 1).format.as_deref(),
        Some(NUMBER_FORMAT)
    );
    assert!(sheet.worksheets[0].data.values().any(|value| {
        value
            .typed
            .as_ref()
            .map(|typed| matches!(typed, CellValue::Number(n) if *n < 0.0))
            .unwrap_or(false)
    }));
}

#[test]
fn editing_an_inflow_recalculates_the_balance_row() {
    let mut sheet =
        build_cash_flow_sheet("owner-1", &default_request()).expect("default request builds");
    let before_net = cell(&sheet, NET_ROW, 1).value.clone();
    let before_closing = cell(&sheet, CLOSING_ROW, 1).value.clone();

    let key = format!("{},{}", INFLOW_ROW, 1);
    if let Some(inflow) = sheet.worksheets[0].data.get_mut(&key) {
        inflow.value = Some("100".to_string());
        inflow.typed = Some(CellValue::Number(100.0));
    }
    let keys: Vec<(u32, u32)> = sheet.worksheets[0]
        .data
        .keys()
        .filter_map(|key| key_from_string(key))
        .collect();
    let mut graph = DepGraph::build(&sheet.worksheets[0]);
    graph.on_edit(&sheet.worksheets[0], &[(INFLOW_ROW, 1)]);
    graph.recalc_cascade_typed_many(&mut sheet.worksheets[0], &keys, MAX_RECALC_CELLS);

    let after_net = cell(&sheet, NET_ROW, 1).value.clone();
    let after_closing = cell(&sheet, CLOSING_ROW, 1).value.clone();
    assert_ne!(
        before_net, after_net,
        "net cash flow follows the edited inflow"
    );
    assert_ne!(
        before_closing, after_closing,
        "closing balance follows the edited inflow"
    );
    assert_eq!(after_net, Some("-6100".to_string()));
}

#[test]
fn custom_months_colors_and_flows_are_honored() {
    let sheet = build_cash_flow_sheet(
        "owner-1",
        &CashFlowRequest {
            title: Some("  My forecast  ".to_string()),
            months: Some(vec!["Q1".to_string(), "Q2".to_string()]),
            opening_balance: Some(-200.0),
            inflows: Some(vec![100.0, 400.0]),
            outflows: Some(vec![300.0, 100.0]),
            negative_color: Some("#FF0000".to_string()),
            positive_color: Some("#00FF00".to_string()),
        },
    )
    .expect("custom request builds");

    assert_eq!(sheet.name, "My forecast");
    assert_eq!(cell(&sheet, 1, 2).value.as_deref(), Some("Q2"));
    assert!(!has_cell(&sheet, 1, 3), "only two month columns requested");
    let rules = sheet.worksheets[0]
        .conditional_formats
        .as_ref()
        .expect("rules exist");
    assert_eq!(rules[0].style.background.as_deref(), Some("#FF0000"));
    assert_eq!(rules[1].style.background.as_deref(), Some("#00FF00"));
    assert_eq!(rules[0].end_col, 2);
}

#[test]
fn too_many_months_are_rejected() {
    let months = (1..=13).map(|index| format!("M{index}")).collect();
    assert!(build_cash_flow_sheet(
        "owner-1",
        &CashFlowRequest {
            title: None,
            months: Some(months),
            opening_balance: None,
            inflows: None,
            outflows: None,
            negative_color: None,
            positive_color: None,
        }
    )
    .is_err());
}

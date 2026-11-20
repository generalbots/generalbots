//! Deterministic cash-flow forecast orchestration (#2260).
//!
//! One `POST /api/sheet/cash-flow` produces the artifact UAT SHEET-06 asks for:
//! twelve month columns, formula-backed balance rows and a pair of opposing
//! conditional rules (negative red / positive green). Building it on the
//! server keeps presentation attached to the sheet instead of depending on a
//! chat model composing a dozen separate per-cell formatting calls.

use crate::auth::{SheetError, SheetUser, resolve_user_id};
use crate::handlers::cell_ops::MAX_RECALC_CELLS;
use crate::handlers::create::cell_from_text;
use crate::handlers::data_ops::apply_conditional_request;
use crate::state::{SheetState, save_sheet_to_drive};
use crate::types::{CellData, CellStyle, ConditionalFormatRequest, Spreadsheet, Worksheet};
use axum::{
    Json,
    extract::{Extension, State},
    http::StatusCode,
};
use botsheet_core::dependency_graph::{DepGraph, key_from_string};
use chrono::Utc;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

/// Calendar labels used when the caller does not supply its own months.
const MONTH_LABELS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// Sample flows deliberately cross zero so both conditional rules have cases
/// to paint on first load.
const DEFAULT_INFLOWS: [f64; 12] = [
    8000.0, 9200.0, 7500.0, 6100.0, 9800.0, 10400.0, 7200.0, 5800.0, 8800.0, 9600.0, 11200.0,
    12500.0,
];
const DEFAULT_OUTFLOWS: [f64; 12] = [
    6200.0, 7100.0, 9100.0, 9400.0, 8400.0, 11500.0, 11200.0, 10800.0, 7900.0, 8900.0, 9600.0,
    9800.0,
];

const OPENING_ROW: u32 = 2;
const INFLOW_ROW: u32 = 3;
const OUTFLOW_ROW: u32 = 4;
const NET_ROW: u32 = 5;
const CLOSING_ROW: u32 = 6;
const FIRST_DATA_COL: u32 = 1;

/// Negative numbers get a red section; positives stay in the default colour
/// and are additionally painted by the `>0` conditional rule below.
const NUMBER_FORMAT: &str = "#,##0;[Red]-#,##0";
const DEFAULT_NEGATIVE_COLOR: &str = "#DC2626";
const DEFAULT_POSITIVE_COLOR: &str = "#16A34A";
const DEFAULT_OPENING_BALANCE: f64 = 5000.0;

/// `POST /api/sheet/cash-flow` body. Every field is optional: the bare call
/// still yields a complete, formatted twelve-month forecast.
#[derive(Debug, Clone, Deserialize)]
pub struct CashFlowRequest {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub months: Option<Vec<String>>,
    #[serde(default)]
    pub opening_balance: Option<f64>,
    #[serde(default)]
    pub inflows: Option<Vec<f64>>,
    #[serde(default)]
    pub outflows: Option<Vec<f64>>,
    #[serde(default)]
    pub negative_color: Option<String>,
    #[serde(default)]
    pub positive_color: Option<String>,
}

/// Zero-based column index to A1 letters (0 -> A, 12 -> M).
fn column_letter(index: u32) -> String {
    let mut remaining = index;
    let mut reversed = Vec::new();
    loop {
        reversed.push((b'A' + (remaining % 26) as u8) as char);
        if remaining < 26 {
            break;
        }
        remaining = remaining / 26 - 1;
    }
    reversed.into_iter().rev().collect()
}

/// A finite JSON number as cell text; non-finite input degrades to zero so a
/// caller cannot poison the forecast with NaN/infinity.
fn number_text(value: f64) -> String {
    if value.is_finite() {
        value.to_string()
    } else {
        "0".to_string()
    }
}

fn title_style() -> CellStyle {
    CellStyle {
        font_size: Some(14),
        font_weight: Some("bold".to_string()),
        ..CellStyle::default()
    }
}

fn header_style() -> CellStyle {
    CellStyle {
        font_weight: Some("bold".to_string()),
        color: Some("#FFFFFF".to_string()),
        background: Some("#0B3D91".to_string()),
        ..CellStyle::default()
    }
}

fn label_style() -> CellStyle {
    CellStyle {
        font_weight: Some("bold".to_string()),
        ..CellStyle::default()
    }
}

/// Evaluates one cell and attaches the requested presentation before it is
/// stored, so a later block write cannot strip it.
fn styled_cell(
    text: &str,
    worksheets: &[Worksheet],
    style: Option<&CellStyle>,
    format: Option<&str>,
) -> CellData {
    let mut cell = cell_from_text(text, worksheets, 0);
    cell.style = style.cloned();
    cell.format = format.map(str::to_string);
    cell
}

/// Adds the two opposing rules over the Net + Closing balance rows.
fn add_balance_rules(sheet: &mut Spreadsheet, req: &CashFlowRequest, last_col: u32) {
    let colors = [
        (
            "less_than",
            DEFAULT_NEGATIVE_COLOR,
            req.negative_color.as_deref(),
        ),
        (
            "greater_than",
            DEFAULT_POSITIVE_COLOR,
            req.positive_color.as_deref(),
        ),
    ];
    for (rule_type, fallback, override_color) in colors {
        let color = override_color
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(fallback);
        let rule = ConditionalFormatRequest {
            sheet_id: sheet.id.clone(),
            worksheet_index: 0,
            start_row: NET_ROW,
            start_col: FIRST_DATA_COL,
            end_row: CLOSING_ROW,
            end_col: last_col,
            range: None,
            rule_type: rule_type.to_string(),
            condition: String::new(),
            style: CellStyle::default(),
            value: Some("0".to_string()),
            bg_color: Some(color.to_string()),
        };
        apply_conditional_request(&mut sheet.worksheets[0], &rule);
    }
}

/// Builds a spreadsheet from the request without touching Drive, so the exact
/// layout, formulas and rules are testable on their own.
pub fn build_cash_flow_sheet(owner_id: &str, req: &CashFlowRequest) -> Result<Spreadsheet, String> {
    let months: Vec<String> = match req.months.as_ref() {
        Some(labels) if !labels.is_empty() => labels.clone(),
        _ => MONTH_LABELS.iter().map(|label| label.to_string()).collect(),
    };
    if months.len() > MONTH_LABELS.len() {
        return Err(format!(
            "a cash-flow forecast supports at most {} months",
            MONTH_LABELS.len()
        ));
    }
    if months.iter().any(|label| label.trim().is_empty()) {
        return Err("month labels must not be empty".to_string());
    }

    let mut sheet = crate::storage::create_new_spreadsheet(owner_id);
    sheet.name = req
        .title
        .as_deref()
        .map(str::trim)
        .filter(|title| !title.is_empty())
        .unwrap_or("Cash-Flow Forecast")
        .to_string();

    let title = title_style();
    let header = header_style();
    let label = label_style();
    let title_cell = styled_cell(&sheet.name, &sheet.worksheets, Some(&title), None);
    sheet.worksheets[0].data.insert("0,0".to_string(), title_cell);

    let month_cell = styled_cell("Month", &sheet.worksheets, Some(&header), None);
    sheet.worksheets[0].data.insert("1,0".to_string(), month_cell);
    for (offset, month) in months.iter().enumerate() {
        let col = FIRST_DATA_COL + offset as u32;
        let cell = styled_cell(month, &sheet.worksheets, Some(&header), None);
        sheet.worksheets[0].data.insert(format!("1,{col}"), cell);
    }

    for (row, text) in [
        (OPENING_ROW, "Opening Balance"),
        (INFLOW_ROW, "Inflows"),
        (OUTFLOW_ROW, "Outflows"),
        (NET_ROW, "Net Cash Flow"),
        (CLOSING_ROW, "Closing Balance"),
    ] {
        let cell = styled_cell(text, &sheet.worksheets, Some(&label), None);
        sheet.worksheets[0].data.insert(format!("{row},0"), cell);
    }

    let inflows = req.inflows.as_deref().unwrap_or(&DEFAULT_INFLOWS);
    let outflows = req.outflows.as_deref().unwrap_or(&DEFAULT_OUTFLOWS);
    for offset in 0..months.len() {
        let col = FIRST_DATA_COL + offset as u32;
        let letter = column_letter(col);
        let opening = if offset == 0 {
            number_text(req.opening_balance.unwrap_or(DEFAULT_OPENING_BALANCE))
        } else {
            format!("={}{}", column_letter(col - 1), CLOSING_ROW + 1)
        };
        let inflow = inflows.get(offset).copied().filter(|v| v.is_finite()).unwrap_or(0.0);
        let outflow = outflows.get(offset).copied().filter(|v| v.is_finite()).unwrap_or(0.0);
        let rows = [
            (OPENING_ROW, opening),
            (INFLOW_ROW, number_text(inflow)),
            (OUTFLOW_ROW, number_text(outflow)),
            (
                NET_ROW,
                format!("={letter}{}-{letter}{}", INFLOW_ROW + 1, OUTFLOW_ROW + 1),
            ),
            (
                CLOSING_ROW,
                format!("={letter}{}+{letter}{}", OPENING_ROW + 1, NET_ROW + 1),
            ),
        ];
        for (row, text) in rows {
            let cell = styled_cell(&text, &sheet.worksheets, None, Some(NUMBER_FORMAT));
            sheet.worksheets[0].data.insert(format!("{row},{col}"), cell);
        }
    }

    // Formula cells can reference rows written later in the loop, so evaluate
    // every formula once in dependency order before the sheet is persisted.
    let keys: Vec<(u32, u32)> = sheet.worksheets[0]
        .data
        .keys()
        .filter_map(|key| key_from_string(key))
        .collect();
    let graph = DepGraph::build(&sheet.worksheets[0]);
    graph.recalc_cascade_typed_many(&mut sheet.worksheets[0], &keys, MAX_RECALC_CELLS);

    add_balance_rules(&mut sheet, req, FIRST_DATA_COL + months.len() as u32 - 1);
    sheet.updated_at = Utc::now();
    Ok(sheet)
}

/// `POST /api/sheet/cash-flow` — create and persist a formatted forecast.
pub async fn handle_create_cash_flow(
    State(state): State<Arc<SheetState>>,
    user: Option<Extension<SheetUser>>,
    Json(req): Json<CashFlowRequest>,
) -> Result<Json<Spreadsheet>, SheetError> {
    let user_id = resolve_user_id(user.as_deref())?;
    let sheet = build_cash_flow_sheet(&user_id, &req).map_err(|message| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": message })),
        )
    })?;
    let sheet_id = sheet.id.clone();
    save_sheet_to_drive(&state, &user_id, &sheet).await.map_err(|error| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": botsecurity_core::error_sanitizer::log_and_sanitize_json(
                    &error,
                    "botsheet::handlers::cash_flow",
                    None,
                )
            })),
        )
    })?;
    log::info!("Created cash-flow forecast sheet {sheet_id}");
    Ok(Json(sheet))
}

#[cfg(test)]
#[path = "cash_flow_tests.rs"]
mod tests;

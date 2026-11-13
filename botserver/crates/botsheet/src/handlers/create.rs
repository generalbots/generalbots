//! Creation verbs driven from chat (#2143).
//!
//! `sheet.create` needs a spreadsheet that already holds the requested rows, and
//! `sheet.cells.set` needs a block write. Both routes live here so the curated
//! commands create a real artifact (with an id) instead of only opening the app.

use crate::auth::{resolve_user_id, SheetError, SheetUser};
use crate::handlers::cell_ops::{MAX_COLS, MAX_RECALC_CELLS, MAX_ROWS};
use crate::state::{save_sheet_to_drive, SheetState};
use crate::types::{CellData, SaveResponse, Spreadsheet, Worksheet};
use botsheet_core::dependency_graph::DepGraph;
use axum::extract::{Extension, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::Utc;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

/// `POST /api/sheet/new` body: a title and rows as a JSON array of arrays.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateSheetRequest {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub rows: Option<Value>,
}

/// `POST /api/sheet/cells` body: write `values` starting at the A1 `range`.
///
/// `styles` and `formats` are keyed by A1 reference so one call can produce a
/// formatted block: without them a generated model needed a second round trip
/// per formatted cell, which is why cash-flow/timetable runs landed unstyled
/// (#2260, #2263).
#[derive(Debug, Clone, Deserialize)]
pub struct SetCellsRequest {
    #[serde(alias = "id", alias = "file_id")]
    pub sheet_id: String,
    #[serde(default)]
    pub worksheet_index: usize,
    pub range: String,
    pub values: Value,
    #[serde(default)]
    pub styles: Option<std::collections::HashMap<String, botsheet_core::types::CellStyle>>,
    #[serde(default)]
    pub formats: Option<std::collections::HashMap<String, String>>,
}

/// Sanitized 500 for a failed persistence step: the detail goes to the log.
fn internal(error: &str) -> SheetError {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({
            "error": botsecurity_core::error_sanitizer::log_and_sanitize_json(
                error,
                "botsheet::handlers::create",
                None,
            )
        })),
    )
}

fn bad_request(message: String) -> SheetError {
    (StatusCode::BAD_REQUEST, Json(json!({ "error": message })))
}

fn not_found(message: String) -> SheetError {
    (StatusCode::NOT_FOUND, Json(json!({ "error": message })))
}

/// Rows arrive as a JSON array of arrays; a flat array is treated as one row.
fn row_values(rows: &Value) -> Result<Vec<Vec<String>>, String> {
    let outer = rows
        .as_array()
        .ok_or_else(|| "rows must be a JSON array".to_string())?;
    if outer.iter().all(|row| !row.is_array()) {
        return Ok(vec![outer.iter().map(cell_text).collect()]);
    }
    let mut out = Vec::with_capacity(outer.len());
    for row in outer {
        let cells = row
            .as_array()
            .ok_or_else(|| "rows must contain arrays".to_string())?;
        out.push(cells.iter().map(cell_text).collect());
    }
    Ok(out)
}

/// A JSON scalar as the text the grid stores. Null becomes an empty cell.
fn cell_text(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}

/// Builds the stored cell for one text value, evaluating `=FORMULA` in place.
fn cell_from_text(text: &str, worksheets: &[Worksheet], index: usize) -> CellData {
    if text.starts_with('=') {
        let result = botsheet_core::engine::evaluate_typed_in(text, worksheets, index);
        return CellData {
            value: Some(result.display()),
            typed: Some(result),
            formula: Some(text.to_string()),
            style: None,
            format: None,
            note: None,
            locked: None,
            has_comment: None,
            array_formula_id: None,
        };
    }
    let typed = botsheet_core::engine::CellValue::parse(text);
    CellData {
        value: Some(text.to_string()),
        typed: Some(typed),
        formula: None,
        style: None,
        format: None,
        note: None,
        locked: None,
        has_comment: None,
        array_formula_id: None,
    }
}

/// Carries the presentation of an already formatted cell onto a new value.
///
/// Presentation lives on the cell, not on the value, so writing `Cinza` into a
/// yellow `A1` must keep the fill, number format and note the user set before
/// (#2150 UAT-06: five colours vanished as soon as the labels were written).
fn keep_presentation(cell: &mut CellData, existing: Option<&CellData>) {
    if let Some(existing) = existing {
        cell.style = existing.style.clone();
        cell.format = existing.format.clone();
        cell.note = existing.note.clone();
        cell.locked = existing.locked;
        cell.has_comment = existing.has_comment;
    }
}

/// Applies A1-keyed styles and number formats to a worksheet that was just
/// written. Unknown references are skipped: a stale key must never create a
/// cell that holds no value.
fn apply_block_presentation(
    worksheet: &mut Worksheet,
    styles: &Option<std::collections::HashMap<String, botsheet_core::types::CellStyle>>,
    formats: &Option<std::collections::HashMap<String, String>>,
) {
    if let Some(styles) = styles.as_ref() {
        for (reference, style) in styles {
            let Some((row, col)) = botsheet_core::formulas::parse_cell_ref(reference) else {
                continue;
            };
            let key = format!("{row},{col}");
            if let Some(cell) = worksheet.data.get_mut(&key) {
                cell.style = Some(style.clone());
            }
        }
    }
    if let Some(formats) = formats.as_ref() {
        for (reference, code) in formats {
            let Some((row, col)) = botsheet_core::formulas::parse_cell_ref(reference) else {
                continue;
            };
            let key = format!("{row},{col}");
            if let Some(cell) = worksheet.data.get_mut(&key) {
                cell.format = if code.trim().is_empty() {
                    None
                } else {
                    Some(code.clone())
                };
            }
        }
    }
}


/// Builds a spreadsheet from a title and rows without touching Drive, so the
/// shape of the created artifact is testable on its own.
pub fn build_spreadsheet(
    owner_id: &str,
    title: Option<&str>,
    rows: Option<&Value>,
) -> Result<Spreadsheet, String> {
    let mut sheet = crate::storage::create_new_spreadsheet(owner_id);
    if let Some(title) = title.map(str::trim).filter(|title| !title.is_empty()) {
        sheet.name = title.to_string();
    }
    let Some(rows) = rows else {
        return Ok(sheet);
    };
    let grid = row_values(rows)?;
    if grid.len() > MAX_ROWS as usize {
        return Err("rows exceed the spreadsheet grid".to_string());
    }
    for (row_index, row) in grid.iter().enumerate() {
        if row.len() > MAX_COLS as usize {
            return Err("a row exceeds the spreadsheet grid".to_string());
        }
        for (col_index, text) in row.iter().enumerate() {
            if text.is_empty() {
                continue;
            }
            let cell = cell_from_text(text, &sheet.worksheets, 0);
            sheet.worksheets[0]
                .data
                .insert(format!("{row_index},{col_index}"), cell);
        }
    }
    sheet.updated_at = Utc::now();
    Ok(sheet)
}

/// `POST /api/sheet/new` — create a spreadsheet from `{title, rows}`.
pub async fn handle_create_sheet(
    State(state): State<Arc<SheetState>>,
    user: Option<Extension<SheetUser>>,
    Json(req): Json<CreateSheetRequest>,
) -> Result<Json<Spreadsheet>, SheetError> {
    let user_id = resolve_user_id(user.as_deref())?;
    let sheet = build_spreadsheet(&user_id, req.title.as_deref(), req.rows.as_ref())
        .map_err(bad_request)?;
    save_sheet_to_drive(&state, &user_id, &sheet)
        .await
        .map_err(|e| internal(&e))?;
    Ok(Json(sheet))
}

/// `POST /api/sheet/cells` — write a block of values from an A1 range.
pub async fn handle_set_cells(
    State(state): State<Arc<SheetState>>,
    user: Option<Extension<SheetUser>>,
    Json(req): Json<SetCellsRequest>,
) -> Result<Json<SaveResponse>, SheetError> {
    let user_id = resolve_user_id(user.as_deref())?;
    let ((start_row, start_col), _) =
        botsheet_core::formulas::refs::parse_range_or_single(&req.range)
            .ok_or_else(|| bad_request(format!("invalid A1 range '{}'", req.range)))?;
    let grid = row_values(&req.values).map_err(bad_request)?;

    let session = state
        .sessions
        .get_or_load(&state, &user_id, &req.sheet_id)
        .await
        .map_err(not_found)?;

    let written = {
        let mut sheet = session.sheet.write().await;
        if let Err(e) = botsheet_core::state::ensure_write_allowed(&user_id, &sheet) {
            return Err((StatusCode::FORBIDDEN, Json(json!({ "error": e }))));
        }
        if req.worksheet_index >= sheet.worksheets.len() {
            return Err(bad_request("Invalid worksheet index".to_string()));
        }

        let mut written = 0usize;
        let mut written_keys: Vec<(u32, u32)> = Vec::with_capacity(grid.len());
        for (row_offset, row) in grid.iter().enumerate() {
            for (col_offset, text) in row.iter().enumerate() {
                let row_index = start_row.saturating_add(row_offset as u32);
                let col_index = start_col.saturating_add(col_offset as u32);
                if row_index >= MAX_ROWS || col_index >= MAX_COLS {
                    return Err(bad_request("Cell out of grid bounds".to_string()));
                }
                let mut cell = cell_from_text(text, &sheet.worksheets, req.worksheet_index);
                let key = format!("{row_index},{col_index}");
                let existing = sheet.worksheets[req.worksheet_index].data.get(&key).cloned();
                keep_presentation(&mut cell, existing.as_ref());
                sheet.worksheets[req.worksheet_index]
                    .data
                    .insert(key, cell);
                written_keys.push((row_index, col_index));
                written += 1;
            }
        }

        // Presentation for the whole block in the same call (#2260/#2263):
        // A1 keyed styles and number formats applied after the values land, so
        // `keep_presentation` cannot wipe them.
        apply_block_presentation(
            &mut sheet.worksheets[req.worksheet_index],
            &req.styles,
            &req.formats,
        );

        {
            // A block write invalidates every formula that reads the block:
            // without this pass `SUM(C2:C4)` keeps the total it had before the
            // edit (#2150 SHEET-04 step 3).
            let mut graphs = match session.dep_graphs.lock() {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            };
            if graphs.len() != sheet.worksheets.len() {
                *graphs = sheet.worksheets.iter().map(DepGraph::build).collect();
            }
            let index = req.worksheet_index;
            graphs[index].on_edit(&sheet.worksheets[index], &written_keys);
            graphs[index].recalc_cascade_typed_many(
                &mut sheet.worksheets[index],
                &written_keys,
                MAX_RECALC_CELLS,
            );
        }

        sheet.updated_at = Utc::now();
        written
    };

    state.sessions.record(
        &session,
        &state,
        &user_id,
        "cells_set",
        json!({
            "sheet_id": req.sheet_id.clone(),
            "worksheet_index": req.worksheet_index,
            "range": req.range,
            "cells": written,
        }),
    );
    session
        .persist_now(&state)
        .await
        .map_err(|e| internal(&e))?;

    Ok(Json(SaveResponse {
        id: req.sheet_id,
        success: true,
        message: Some(format!("{written} cells updated")),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn created_spreadsheet_carries_an_id_and_the_rows() {
        let rows = json!([["Region", "Total"], ["North", 1200], ["South", 980],]);
        let sheet = build_spreadsheet("owner-1", Some("  Budget  "), Some(&rows))
            .expect("rows are a valid grid");
        assert!(!sheet.id.is_empty(), "the created artifact needs an id");
        assert_eq!(sheet.name, "Budget");
        let data = &sheet.worksheets[0].data;
        assert_eq!(
            data.get("0,0").and_then(|cell| cell.value.as_deref()),
            Some("Region")
        );
        assert_eq!(
            data.get("1,1").and_then(|cell| cell.value.as_deref()),
            Some("1200")
        );
    }

    #[test]
    fn a_flat_array_is_one_row() {
        let rows = json!(["Name", "Amount"]);
        let sheet = build_spreadsheet("owner-1", None, Some(&rows)).expect("valid row");
        assert_eq!(
            sheet.worksheets[0]
                .data
                .get("0,1")
                .and_then(|cell| cell.value.as_deref()),
            Some("Amount")
        );
    }

    #[test]
    fn rows_that_are_not_an_array_are_rejected() {
        let rows = json!({ "not": "a grid" });
        assert!(build_spreadsheet("owner-1", None, Some(&rows)).is_err());
    }

    #[test]
    fn a_cell_block_writes_from_the_range_start() {
        let grid = row_values(&json!([["a", "b"], ["c", "d"]])).expect("valid grid");
        assert_eq!(grid, vec![vec!["a", "b"], vec!["c", "d"]]);
        let ((row, col), _) =
            botsheet_core::formulas::refs::parse_range_or_single("B2:C3").expect("valid range");
        assert_eq!((row, col), (1, 1));
        let ((row, col), _) =
            botsheet_core::formulas::refs::parse_range_or_single("A1").expect("single cell");
        assert_eq!((row, col), (0, 0));
    }

    #[test]
    fn writing_a_value_keeps_the_existing_presentation() {
        let mut cell = cell_from_text("Cinza", &[], 0);
        let existing = CellData {
            value: Some("old".to_string()),
            typed: None,
            formula: None,
            style: Some(botsheet_core::types::CellStyle {
                background: Some("#fde68a".to_string()),
                font_weight: Some("bold".to_string()),
                ..Default::default()
            }),
            format: Some("0.00".to_string()),
            note: Some("keep me".to_string()),
            locked: Some(true),
            has_comment: Some(true),
            array_formula_id: None,
        };

        keep_presentation(&mut cell, Some(&existing));

        assert_eq!(cell.value.as_deref(), Some("Cinza"));
        let style = cell.style.as_ref().expect("style survives the write");
        assert_eq!(style.background.as_deref(), Some("#fde68a"));
        assert_eq!(style.font_weight.as_deref(), Some("bold"));
        assert_eq!(cell.format.as_deref(), Some("0.00"));
        assert_eq!(cell.note.as_deref(), Some("keep me"));
        assert_eq!(cell.locked, Some(true));
    }
}

use crate::auth::{resolve_user_id, SheetUser};
use crate::handlers::cell_ops::MAX_RECALC_CELLS;
use crate::handlers::paste_html::{parse_html_table, style_for};
use crate::state::SheetState;
use crate::types::{
    CellData, PasteRequest, ResizeRequest, SaveResponse, TableRequest,
};
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    Json,
};
use botsheet_core::dependency_graph::DepGraph;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;

fn sheet_error(e: String) -> (StatusCode, Json<serde_json::Value>) {
    (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": e })))
}

fn bad_request(msg: &str) -> (StatusCode, Json<serde_json::Value>) {
    (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": msg })))
}

/// Sets a row height or column width on a worksheet (#786). Both may be sent
/// in one request; bounds are Excel's grid limits.
pub async fn handle_resize(
    State(state): State<Arc<SheetState>>,
    user: Option<Extension<SheetUser>>,
    Json(req): Json<ResizeRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = resolve_user_id(user.as_deref());

    let session = state
        .sessions
        .get_or_load(&state, &user_id, &req.sheet_id)
        .await
        .map_err(sheet_error)?;

    let mut sheet = session.sheet.write().await;
    if let Err(e) = botsheet_core::state::ensure_write_allowed(&user_id, &sheet) {
        return Err((StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": e }))));
    }

    if req.worksheet_index >= sheet.worksheets.len() {
        return Err(bad_request("Invalid worksheet index"));
    }

    let worksheet = &mut sheet.worksheets[req.worksheet_index];

    if let (Some(row), Some(height)) = (req.row, req.height) {
        if row >= crate::handlers::cell_ops::MAX_ROWS {
            return Err(bad_request("Row out of grid bounds"));
        }
        let heights = worksheet.row_heights.get_or_insert_with(HashMap::new);
        if height == 0 {
            heights.remove(&row);
        } else {
            heights.insert(row, height);
        }
    }

    if let (Some(col), Some(width)) = (req.col, req.width) {
        if col >= crate::handlers::cell_ops::MAX_COLS {
            return Err(bad_request("Column out of grid bounds"));
        }
        let widths = worksheet.column_widths.get_or_insert_with(HashMap::new);
        if width == 0 {
            widths.remove(&col);
        } else {
            widths.insert(col, width);
        }
    }

    sheet.updated_at = Utc::now();
    state.sessions.record(
        &session,
        &state,
        &user_id,
        "resize",
        serde_json::json!({
            "sheet_id": req.sheet_id,
            "worksheet_index": req.worksheet_index,
            "row": req.row,
            "col": req.col,
            "height": req.height,
            "width": req.width,
        }),
    );

    Ok(Json(SaveResponse {
        id: req.sheet_id,
        success: true,
        message: Some("Size updated".to_string()),
    }))
}

/// Pastes clipboard content into a sheet (#787). The primary flavor is the
/// HTML table Excel/Sheets put on the clipboard; `mode` controls Paste
/// Special semantics: "values" writes values only, "formats" applies styles,
/// "all" (default) writes values plus style hints.
pub async fn handle_paste(
    State(state): State<Arc<SheetState>>,
    user: Option<Extension<SheetUser>>,
    Json(req): Json<PasteRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = resolve_user_id(user.as_deref());

    if req.html.trim().is_empty() {
        return Err(bad_request("Clipboard content is empty"));
    }
    if req.start_row >= crate::handlers::cell_ops::MAX_ROWS
        || req.start_col >= crate::handlers::cell_ops::MAX_COLS
    {
        return Err(bad_request("Target out of grid bounds"));
    }

    let grid = parse_html_table(&req.html);
    if grid.is_empty() {
        return Err(bad_request("No table cells found in clipboard content"));
    }

    let session = state
        .sessions
        .get_or_load(&state, &user_id, &req.sheet_id)
        .await
        .map_err(sheet_error)?;

    let mut sheet = session.sheet.write().await;
    if let Err(e) = botsheet_core::state::ensure_write_allowed(&user_id, &sheet) {
        return Err((StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": e }))));
    }

    if req.worksheet_index >= sheet.worksheets.len() {
        return Err(bad_request("Invalid worksheet index"));
    }

    let mut written = 0usize;
    let mut pasted_keys: Vec<(u32, u32)> = Vec::new();
    let mode = req.mode.to_ascii_lowercase();
    // Pre-compute payloads for every pasted cell while the worksheet set is
    // immutably readable, so cross-sheet references resolve against all
    // worksheets instead of only the pasted one (#783).
    #[derive(Clone)]
    struct PastePayload {
        display: Option<String>,
        formula: Option<String>,
        typed: Option<botsheet_core::engine::CellValue>,
        style: Option<crate::types::CellStyle>,
    }

    let mut payloads: Vec<PastePayload> = Vec::new();
    for (row_offset, cells) in grid.iter().enumerate() {
        let target_row = req.start_row + row_offset as u32;
        if target_row >= crate::handlers::cell_ops::MAX_ROWS {
            break;
        }
        for (col_offset, cell) in cells.iter().enumerate() {
            let target_col = req.start_col + col_offset as u32;
            if target_col >= crate::handlers::cell_ops::MAX_COLS {
                break;
            }
            let style = style_for(cell.style.as_deref(), &mode);
            let payload = if mode != "formats" && cell.value.starts_with('=') {
                let result = botsheet_core::engine::evaluate_typed_in(
                    &cell.value,
                    &sheet.worksheets,
                    req.worksheet_index,
                );
                PastePayload {
                    display: Some(result.display()),
                    formula: Some(cell.value.clone()),
                    typed: Some(result),
                    style,
                }
            } else if mode != "formats" {
                PastePayload {
                    display: Some(cell.value.clone()),
                    formula: None,
                    typed: Some(botsheet_core::engine::CellValue::parse(&cell.value)),
                    style,
                }
            } else {
                PastePayload {
                    display: None,
                    formula: None,
                    typed: None,
                    style,
                }
            };
            payloads.push(payload);
        }
    }
    {
        let worksheet = &mut sheet.worksheets[req.worksheet_index];
        let mut payload_index = 0usize;
        for (row_offset, cells) in grid.iter().enumerate() {
            let target_row = req.start_row + row_offset as u32;
            if target_row >= crate::handlers::cell_ops::MAX_ROWS {
                break;
            }
            for col_offset in 0..cells.len() {
                let target_col = req.start_col + col_offset as u32;
                if target_col >= crate::handlers::cell_ops::MAX_COLS {
                    break;
                }
                let payload = payloads[payload_index].clone();
                let key = format!("{target_row},{target_col}");
                let entry = worksheet.data.entry(key).or_insert_with(|| CellData {
                    value: None,
                    typed: None,
                    formula: None,
                    style: None,
                    format: None,
                    note: None,
                    locked: None,
                    has_comment: None,
                    array_formula_id: None,
                });
                entry.value = payload.display;
                entry.formula = payload.formula;
                entry.typed = payload.typed;
                entry.style = payload.style;
                written += 1;
                pasted_keys.push((target_row, target_col));
                payload_index += 1;
            }
        }
    }

    {
        // Refresh the cached dependency graph for every written cell and
        // recalculate dependents in one topological pass (#784).
        let mut graphs = match session.dep_graphs.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        if graphs.len() != sheet.worksheets.len() {
            *graphs = sheet.worksheets.iter().map(DepGraph::build).collect();
        }
        let index = req.worksheet_index;
        graphs[index].on_edit(&sheet.worksheets[index], &pasted_keys);
        graphs[index].recalc_cascade_typed_many(
            &mut sheet.worksheets[index],
            &pasted_keys,
            MAX_RECALC_CELLS,
        );
    }

    sheet.updated_at = Utc::now();
    state.sessions.record(
        &session,
        &state,
        &user_id,
        "paste",
        serde_json::json!({
            "sheet_id": req.sheet_id,
            "worksheet_index": req.worksheet_index,
            "start_row": req.start_row,
            "start_col": req.start_col,
            "cells": written,
            "mode": mode,
        }),
    );

    Ok(Json(SaveResponse {
        id: req.sheet_id,
        success: true,
        message: Some(format!("Pasted {written} cells")),
    }))
}

/// Creates a table over the given range (#790). Tables live in
/// `Worksheet.tables` and are persisted with the sheet; the id doubles as the
/// table's human-readable name for now.
pub async fn handle_table_create(
    State(state): State<Arc<SheetState>>,
    user: Option<Extension<SheetUser>>,
    Json(req): Json<TableRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = resolve_user_id(user.as_deref());
    let name = req
        .name
        .clone()
        .filter(|n| !n.trim().is_empty())
        .ok_or_else(|| bad_request("Table name is required"))?;

    if req.start_row > req.end_row || req.start_col > req.end_col {
        return Err(bad_request("Invalid table range"));
    }

    let session = state
        .sessions
        .get_or_load(&state, &user_id, &req.sheet_id)
        .await
        .map_err(sheet_error)?;

    let mut sheet = session.sheet.write().await;
    if let Err(e) = botsheet_core::state::ensure_write_allowed(&user_id, &sheet) {
        return Err((StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": e }))));
    }

    if req.worksheet_index >= sheet.worksheets.len() {
        return Err(bad_request("Invalid worksheet index"));
    }

    let worksheet = &mut sheet.worksheets[req.worksheet_index];
    let tables = worksheet.tables.get_or_insert_with(Vec::new);
    if tables.iter().any(|t| t.id == name) {
        return Err(bad_request("Table with this name already exists"));
    }
    tables.push(crate::types::TableConfig {
        id: name.clone(),
        name,
        start_row: req.start_row,
        start_col: req.start_col,
        end_row: req.end_row,
        end_col: req.end_col,
        has_header_row: true,
    });

    sheet.updated_at = Utc::now();
    state.sessions.record(
        &session,
        &state,
        &user_id,
        "table_create",
        serde_json::json!({
            "sheet_id": req.sheet_id,
            "worksheet_index": req.worksheet_index,
            "name": req.name,
            "range": format!("{}:{}", req.start_row, req.end_row),
        }),
    );

    Ok(Json(SaveResponse {
        id: req.sheet_id,
        success: true,
        message: Some("Table created".to_string()),
    }))
}

/// Lists all tables on a worksheet (#790).
pub async fn handle_table_list(
    State(state): State<Arc<SheetState>>,
    user: Option<Extension<SheetUser>>,
    Json(req): Json<TableRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = resolve_user_id(user.as_deref());

    let session = state
        .sessions
        .get_or_load(&state, &user_id, &req.sheet_id)
        .await
        .map_err(sheet_error)?;

    let sheet = session.sheet.read().await;
    if req.worksheet_index >= sheet.worksheets.len() {
        return Err(bad_request("Invalid worksheet index"));
    }
    let worksheet = &sheet.worksheets[req.worksheet_index];
    let tables = worksheet.tables.as_deref().unwrap_or(&[]);
    Ok(Json(serde_json::json!({
        "tables": tables.iter().map(|t| {
            serde_json::json!({
                "id": t.id,
                "name": t.name,
                "start_row": t.start_row,
                "start_col": t.start_col,
                "end_row": t.end_row,
                "end_col": t.end_col,
                "has_header_row": t.has_header_row,
            })
        }).collect::<Vec<_>>(),
    })))
}

pub async fn handle_table_delete(
    State(state): State<Arc<SheetState>>,
    user: Option<Extension<SheetUser>>,
    Json(req): Json<TableRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = resolve_user_id(user.as_deref());

    let session = state
        .sessions
        .get_or_load(&state, &user_id, &req.sheet_id)
        .await
        .map_err(sheet_error)?;

    let mut sheet = session.sheet.write().await;
    if let Err(e) = botsheet_core::state::ensure_write_allowed(&user_id, &sheet) {
        return Err((StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": e }))));
    }

    if req.worksheet_index >= sheet.worksheets.len() {
        return Err(bad_request("Invalid worksheet index"));
    }

    let worksheet = &mut sheet.worksheets[req.worksheet_index];
    let Some(tables) = &mut worksheet.tables else {
        return Err(bad_request("Table not found"));
    };
    let before = tables.len();
    tables.retain(|t| t.id != req.name.as_deref().unwrap_or(""));
    if tables.len() == before {
        return Err(bad_request("Table not found"));
    }

    sheet.updated_at = Utc::now();
    state.sessions.record(
        &session,
        &state,
        &user_id,
        "table_delete",
        serde_json::json!({ "sheet_id": req.sheet_id, "table_id": req.name }),
    );

    Ok(Json(SaveResponse {
        id: req.sheet_id,
        success: true,
        message: Some("Table deleted".to_string()),
    }))
}

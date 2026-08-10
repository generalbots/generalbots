use crate::auth::{resolve_user_id, SheetUser};
use crate::handlers::cell_ops::MAX_RECALC_CELLS;
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

/// A single parsed clipboard cell (HTML table flavor).
#[derive(Clone)]
struct HtmlCell {
    value: String,
    style: Option<String>,
    colspan: usize,
    rowspan: usize,
}

/// Parses an HTML table fragment from the clipboard (Excel/Sheets flavor) into
/// a value grid. Handles `<table><tr><td>` structure, inline styles for the
/// `formats` Paste Special mode, and cells spanning via colspan/rowspan.
fn parse_html_table(html: &str) -> Vec<Vec<HtmlCell>> {
    let mut grid: Vec<Vec<HtmlCell>> = Vec::new();
    let mut pos = 0usize;
    let lower = html.to_ascii_lowercase();

    // `occupied` tracks columns still claimed by a rowspan from a previous row.
    let mut occupied: Vec<(usize, usize)> = Vec::new();

    while let Some(tr_start) = find_tag(&lower, "tr", pos) {
        let row_open_end = find_tag_close(&lower, tr_start);
        // End of this row's body: the row's own closing tag.
        let row_close = find_tag(&lower, "/tr", row_open_end)
            .or_else(|| {
                lower[row_open_end..]
                    .find("</")
                    .map(|i| row_open_end + i)
            })
            .unwrap_or(lower.len());

        let mut out_row: Vec<HtmlCell> = Vec::new();
        let mut col = 0usize;
        occupied.retain_mut(|(c, remaining)| {
            if *remaining > 1 {
                *remaining -= 1;
                while out_row.len() <= *c {
                    out_row.push(HtmlCell {
                        value: String::new(),
                        style: None,
                        colspan: 1,
                        rowspan: 1,
                    });
                }
                true
            } else {
                false
            }
        });

        let mut cell_pos = tr_start;
        loop {
            let next_td = find_tag(&lower, "td", cell_pos);
            let next_th = find_tag(&lower, "th", cell_pos);
            let cell_start = match (next_td, next_th) {
                (Some(td), Some(th)) => td.min(th),
                (Some(td), None) => td,
                (None, Some(th)) => th,
                (None, None) => break,
            };
            if cell_start >= row_close {
                break;
            }
            let cell_open_end = find_tag_close(&lower, cell_start);
            if cell_open_end <= cell_start {
                break;
            }
            let cell_close = [find_tag(&lower, "/td", cell_open_end),
                              find_tag(&lower, "/th", cell_open_end)]
                .into_iter()
                .flatten()
                .filter(|&c| c < row_close)
                .min()
                .unwrap_or(row_close);
            let content = extract_text(html, cell_open_end, cell_close);
            let attrs = extract_tag_attrs(&lower, cell_start);
            let cell = HtmlCell {
                value: content,
                style: attrs.get("style").cloned(),
                colspan: attrs.get("colspan").and_then(|v| v.parse().ok()).unwrap_or(1),
                rowspan: attrs.get("rowspan").and_then(|v| v.parse().ok()).unwrap_or(1),
            };

            while occupied.iter().any(|(c, _)| *c == col) {
                col += 1;
            }
            while out_row.len() < col + cell.colspan {
                out_row.push(HtmlCell {
                    value: String::new(),
                    style: None,
                    colspan: 1,
                    rowspan: 1,
                });
            }
            for c in col..col + cell.colspan {
                if c == col {
                    out_row[c] = cell.clone();
                }
            }
            if cell.rowspan > 1 {
                for c in col..col + cell.colspan {
                    occupied.push((c, cell.rowspan));
                }
            }
            col += cell.colspan;
            cell_pos = cell_open_end;
        }

        if !out_row.is_empty() {
            grid.push(out_row);
        }
        pos = find_tag_close(&lower, row_close);
    }

    grid
}

fn find_tag(haystack: &str, tag: &str, from: usize) -> Option<usize> {
    let needle = format!("<{tag}");
    let mut idx = from;
    while idx < haystack.len() {
        let Some(rel) = haystack[idx..].find(&needle) else {
            return None;
        };
        let abs = idx + rel;
        let boundary_ok = haystack[abs + needle.len()..]
            .chars()
            .next()
            .map(|c| matches!(c, '>' | ' ' | '/' | '\n' | '\t' | '\r'))
            .unwrap_or(true);
        if boundary_ok {
            return Some(abs);
        }
        idx = abs + needle.len();
    }
    None
}

fn find_tag_close(haystack: &str, tag_start: usize) -> usize {
    let rel = haystack[tag_start..].find('>').unwrap_or(0);
    tag_start + rel + 1
}

fn extract_tag_attrs(lower: &str, tag_start: usize) -> HashMap<String, String> {
    let mut out = HashMap::new();
    let rest = &lower[tag_start..];
    let end = rest.find('>').unwrap_or(rest.len());
    let tag = &rest[..end];
    let mut iter = tag.split_whitespace().skip(1);
    while let Some(part) = iter.next() {
        if let Some(eq) = part.find('=') {
            let key = &part[..eq];
            let value = part[eq + 1..]
                .trim_matches('"')
                .trim_matches('\'')
                .to_string();
            out.insert(key.to_string(), value);
        }
    }
    out
}

fn extract_text(html: &str, start: usize, end: usize) -> String {
    let mut out = String::new();
    let mut i = start;
    let chars: Vec<char> = html.chars().collect();
    while i < end && i < chars.len() {
        if chars[i] == '<' {
            let mut j = i + 1;
            while j < end && j < chars.len() && chars[j] != '>' {
                j += 1;
            }
            if j < end && j < chars.len() {
                i = j + 1;
            } else {
                break;
            }
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn style_for(style: Option<&str>, mode: &str) -> Option<crate::types::CellStyle> {
    if mode == "values" {
        return None;
    }
    style.map(|s| crate::types::CellStyle {
        font_family: None,
        font_size: None,
        font_weight: None,
        font_style: None,
        text_decoration: None,
        color: None,
        background: None,
        text_align: None,
        vertical_align: None,
        border: Some(s.to_string()),
    })
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
    let mut payloads: Vec<(
        Option<String>,
        Option<String>,
        Option<botsheet_core::engine::CellValue>,
        Option<String>,
    )> = Vec::new();
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
                (
                    Some(result.display()),
                    Some(cell.value.clone()),
                    Some(result),
                    style,
                )
            } else if mode != "formats" {
                (
                    Some(cell.value.clone()),
                    None,
                    Some(botsheet_core::engine::CellValue::parse(&cell.value)),
                    style,
                )
            } else {
                (None, None, None, style)
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
            for (col_offset, cell) in cells.iter().enumerate() {
                let target_col = req.start_col + col_offset as u32;
                if target_col >= crate::handlers::cell_ops::MAX_COLS {
                    break;
                }
                let (value, formula, typed, style) = payloads[payload_index].clone();
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
                entry.value = value;
                entry.formula = formula;
                entry.typed = typed;
                entry.style = style;
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
    let tables = worksheet.tables.as_ref().map(Vec::as_slice).unwrap_or(&[]);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_basic_html_table() {
        let html = "<table><tr><td>A1</td><td>B1</td></tr><tr><td>A2</td><td>42</td></tr></table>";
        let grid = parse_html_table(html);
        assert_eq!(grid.len(), 2);
        assert_eq!(grid[0].len(), 2);
        assert_eq!(grid[0][0].value, "A1");
        assert_eq!(grid[0][1].value, "B1");
        assert_eq!(grid[1][1].value, "42");
    }

    #[test]
    fn parses_th_headers_and_styles() {
        let html = "<table><tr><th style=\"font-weight:bold\">Name</th><th>Qty</th></tr><tr><td>x</td><td>1</td></tr></table>";
        let grid = parse_html_table(html);
        assert_eq!(grid[0][0].value, "Name");
        assert!(grid[0][0].style.is_some());
        assert_eq!(grid[1][0].value, "x");
    }

    #[test]
    fn handles_colspan() {
        let html = "<table><tr><td colspan=\"2\">wide</td></tr><tr><td>a</td><td>b</td></tr></table>";
        let grid = parse_html_table(html);
        assert_eq!(grid[0].len(), 2);
        assert_eq!(grid[0][0].value, "wide");
    }
}

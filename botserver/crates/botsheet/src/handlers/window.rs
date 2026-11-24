//! #2435 — Server-side windowed reads for large workbooks.
//!
//! `GET /api/sheet/:id/window?range=A1:Z200` returns only the populated cells
//! of a bounded A1 rectangle, together with the workbook version and an ETag.
//! The range parser is pure and caps both the requested span and the absolute
//! A1 coordinates, so a hostile query cannot allocate an unbounded grid.

use crate::auth::{resolve_user_id, SheetError, SheetUser};
use crate::scale::{
    CellRect, FetchTile, MemoryBudget, WindowPlanRequest, WorkbookTier, plan_window,
};
use crate::state::SheetState;
use crate::ui_fragments::Lang;
use axum::{
    extract::{Extension, Path, Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use botsheet_core::engine::formats::{self, NumberLocale};
use botsheet_core::types::{CellData, Worksheet};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::sync::Arc;

/// Absolute Excel column ceiling (XFD is column 16_384).
const MAX_EXCEL_COL: u32 = 16_384;
/// Absolute Excel row ceiling (1_048_576 rows).
const MAX_EXCEL_ROW: u32 = 1_048_576;
/// Maximum rows in one window. Bounds the server-side scan loop.
pub(crate) const MAX_WINDOW_ROWS: u32 = 1_000;
/// Maximum columns in one window. Bounds the server-side scan loop.
pub(crate) const MAX_WINDOW_COLS: u32 = 256;

/// A validated, 0-based inclusive A1 rectangle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowRange {
    pub start_row: u32,
    pub start_col: u32,
    pub end_row: u32,
    pub end_col: u32,
}

/// Parses one `A1`-style reference into 0-based `(row, col)`.
fn parse_a1_ref(reference: &str) -> Option<(u32, u32)> {
    let digit_index = reference
        .char_indices()
        .find(|(_, ch)| ch.is_ascii_digit())
        .map(|(index, _)| index)?;
    let (col_part, row_part) = reference.split_at(digit_index);
    if col_part.is_empty() || col_part.len() > 3 {
        return None;
    }
    let mut col: u32 = 0;
    for byte in col_part.bytes() {
        let letter = byte.to_ascii_uppercase();
        if !letter.is_ascii_uppercase() {
            return None;
        }
        col = col
            .checked_mul(26)?
            .checked_add((letter - b'A') as u32 + 1)?;
    }
    if col == 0 || col > MAX_EXCEL_COL {
        return None;
    }
    if row_part.is_empty() || row_part.len() > 7 {
        return None;
    }
    let one_based_row: u32 = row_part.parse().ok()?;
    if one_based_row == 0 || one_based_row > MAX_EXCEL_ROW {
        return None;
    }
    Some((one_based_row - 1, col - 1))
}

/// Pure range parser: validates `A1:Z200` syntax and caps the span so a single
/// request can never scan more than [`MAX_WINDOW_ROWS`] × [`MAX_WINDOW_COLS`]
/// cells. Returns `None` for every malformed, reversed or oversized range.
pub fn parse_window_range(input: &str) -> Option<WindowRange> {
    let mut parts = input.trim().split(':');
    let start = parts.next()?;
    let end = parts.next()?;
    if parts.next().is_some() {
        return None;
    }
    let (start_row, start_col) = parse_a1_ref(start)?;
    let (end_row, end_col) = parse_a1_ref(end)?;
    if start_row > end_row || start_col > end_col {
        return None;
    }
    let rows = end_row - start_row + 1;
    let cols = end_col - start_col + 1;
    if rows > MAX_WINDOW_ROWS || cols > MAX_WINDOW_COLS {
        return None;
    }
    Some(WindowRange {
        start_row,
        start_col,
        end_row,
        end_col,
    })
}

/// Predicate form of [`parse_window_range`], for guards and tests.
pub fn is_valid_window_range(input: &str) -> bool {
    parse_window_range(input).is_some()
}

/// Collect populated cells of planner-produced tiles in `"row,col"` key form,
/// applying number formats for display (`typed` stays raw).
fn collect_tile_cells(
    worksheet: &Worksheet,
    tiles: &[CellRect],
    locale: NumberLocale,
) -> HashMap<String, CellData> {
    let mut cells: HashMap<String, CellData> = HashMap::new();
    for rect in tiles {
        for row in rect.first_row..=rect.last_row {
            for col in rect.first_col..=rect.last_col {
                let key = format!("{row},{col}");
                if let Some(cell) = worksheet.data.get(&key) {
                    let mut cell = cell.clone();
                    if let (Some(typed), Some(code)) = (cell.typed.as_ref(), cell.format.as_deref())
                    {
                        cell.value = Some(formats::display_cell_locale(typed, Some(code), locale));
                    }
                    cells.insert(key, cell);
                }
            }
        }
    }
    cells
}

/// Single-rectangle form used by the pure range tests.
#[cfg(test)]
fn collect_window_cells(
    worksheet: &Worksheet,
    range: WindowRange,
    locale: NumberLocale,
) -> HashMap<String, CellData> {
    let rect = CellRect {
        first_row: range.start_row,
        first_col: range.start_col,
        last_row: range.end_row,
        last_col: range.end_col,
    };
    collect_tile_cells(worksheet, &[rect], locale)
}

/// Partition a validated range into budgeted fetch tiles. A worksheet has no
/// dimension record, so its bounds are Excel's limits and clamping drops no
/// requested cell; the planner still enforces tile geometry and the tile cap.
fn bounded_window_tiles(range: WindowRange) -> Result<Vec<FetchTile>, SheetError> {
    let rect = CellRect {
        first_row: range.start_row,
        first_col: range.start_col,
        last_row: range.end_row,
        last_col: range.end_col,
    };
    // Large tier's 1000x256 tile matches the handler caps: one tile per valid
    // range, so collected cells stay identical to the previous direct scan.
    let budget = MemoryBudget::for_tier(WorkbookTier::Large);
    let request = WindowPlanRequest {
        requested: rect,
        viewport: rect,
        sheet_bounds: CellRect {
            first_row: 0,
            first_col: 0,
            last_row: MAX_EXCEL_ROW - 1,
            last_col: MAX_EXCEL_COL - 1,
        },
    };
    plan_window(&request, &budget).map_err(|error| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Window range exceeds the sheet window budget",
                "detail": error.to_string(),
            })),
        )
    })
}

#[derive(Debug, Deserialize)]
pub struct WindowQuery {
    pub range: Option<String>,
    #[serde(default)]
    pub sheet: Option<usize>,
}

/// Builds an ETag that changes on every edit (session version) and survives a
/// session reload (`updated_at` from the persisted workbook).
fn window_etag(updated_at_millis: i64, version: u64) -> Result<(String, HeaderValue), SheetError> {
    let etag = format!("\"{updated_at_millis}-{version}\"");
    match HeaderValue::from_str(&etag) {
        Ok(value) => Ok((etag, value)),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "Failed to build ETag" })),
        )),
    }
}

pub async fn handle_sheet_window(
    State(state): State<Arc<SheetState>>,
    user: Option<Extension<SheetUser>>,
    headers: HeaderMap,
    Path(sheet_id): Path<String>,
    Query(query): Query<WindowQuery>,
) -> Result<Response, SheetError> {
    let user_id = resolve_user_id(user.as_deref())?;

    let raw_range = query.range.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "range query parameter is required" })),
        )
    })?;
    let range = parse_window_range(&raw_range).ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Invalid window range",
                "detail": format!(
                    "expected A1:Z200 syntax, at most {MAX_WINDOW_ROWS} rows and \
                     {MAX_WINDOW_COLS} columns"
                ),
            })),
        )
    })?;

    // Tenant scoping: the session is keyed by user id and `get_or_load`
    // refuses anyone who cannot read the workbook (#2127).
    let session = state
        .sessions
        .get_or_load(&state, &user_id, &sheet_id)
        .await
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": e })),
            )
        })?;

    let locale = Lang::from_headers(&headers).number_locale();
    let sheet = session.sheet.read().await;
    let sheet_index = query.sheet.unwrap_or(0);
    let worksheet = sheet.worksheets.get(sheet_index).ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Invalid worksheet index" })),
        )
    })?;

    let version = session.version.load(Ordering::SeqCst);
    let (etag, etag_value) = window_etag(sheet.updated_at.timestamp_millis(), version)?;

    if let Some(requested) = headers.get(header::IF_NONE_MATCH) {
        if let Ok(value) = requested.to_str() {
            if value.trim() == etag {
                let mut response = StatusCode::NOT_MODIFIED.into_response();
                response.headers_mut().insert(header::ETAG, etag_value);
                return Ok(response);
            }
        }
    }

    let tiles = bounded_window_tiles(range)?;
    let rects: Vec<CellRect> = tiles.iter().map(|tile| tile.rect).collect();
    let cells = collect_tile_cells(worksheet, &rects, locale);
    let body = serde_json::json!({
        "id": sheet_id,
        "worksheet": sheet_index,
        "worksheet_name": worksheet.name,
        "range": raw_range,
        "version": version,
        "cells": cells,
    });
    let mut response = Json(body).into_response();
    response.headers_mut().insert(header::ETAG, etag_value);
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use botsheet_core::engine::value::CellValue;

    #[test]
    fn parses_a_canonical_window() {
        assert_eq!(
            parse_window_range("A1:Z200"),
            Some(WindowRange {
                start_row: 0,
                start_col: 0,
                end_row: 199,
                end_col: 25,
            })
        );
    }

    #[test]
    fn accepts_lowercase_and_surrounding_whitespace() {
        let expected = parse_window_range("A1:Z200");
        assert_eq!(parse_window_range("a1:z200"), expected);
        assert_eq!(parse_window_range("  A1:Z200  "), expected);
    }

    #[test]
    fn parses_multi_letter_columns() {
        assert_eq!(
            parse_window_range("AA1:AB2"),
            Some(WindowRange {
                start_row: 0,
                start_col: 26,
                end_row: 1,
                end_col: 27,
            })
        );
    }

    #[test]
    fn accepts_the_largest_allowed_window() {
        assert!(is_valid_window_range("A1:IV1000"));
        assert_eq!(
            parse_window_range("A1:A1"),
            Some(WindowRange {
                start_row: 0,
                start_col: 0,
                end_row: 0,
                end_col: 0,
            })
        );
    }

    #[test]
    fn hostile_ranges_are_rejected() {
        let hostile = [
            "",
            "   ",
            "A1",
            "A1:",
            ":Z200",
            ":",
            "A1:B2:C3",
            "1A:B2",
            "A0:B2",
            "A1:Z0",
            "A1:XFE1",
            "ZZZ1:ZZZ2",
            "A1048577:B1048578",
            "A1048576:A1048577",
            "A9999999999:B1",
            "B2:A1",
            "A2:B1",
            "A1:A1001",
            "A1:IW1",
            "XFE1:XFE2",
            "A-1:B2",
            "A1.5:B2",
            "A1:B2\nC3:D4",
            "A1:B2;DROP TABLE",
        ];
        for input in hostile {
            assert!(!is_valid_window_range(input), "should reject: {input:?}");
        }
    }

    #[test]
    fn collects_only_populated_cells_in_the_window() {
        let mut worksheet = Worksheet::default();
        worksheet.data.insert(
            "0,0".to_string(),
            CellData {
                value: Some("in".to_string()),
                ..Default::default()
            },
        );
        worksheet.data.insert(
            "3,0".to_string(),
            CellData {
                value: Some("out".to_string()),
                ..Default::default()
            },
        );
        let range = parse_window_range("A1:B2").expect("valid range");
        let cells = collect_window_cells(&worksheet, range, NumberLocale::EN);
        assert_eq!(cells.len(), 1);
        assert_eq!(
            cells.get("0,0").and_then(|cell| cell.value.as_deref()),
            Some("in")
        );
        assert!(!cells.contains_key("3,0"));
    }

    #[test]
    fn applies_locale_to_typed_display_values() {
        let mut worksheet = Worksheet::default();
        worksheet.data.insert(
            "0,0".to_string(),
            CellData {
                typed: Some(CellValue::Number(1234.5)),
                format: Some("#,##0.00".to_string()),
                ..Default::default()
            },
        );
        let range = parse_window_range("A1:A1").expect("valid range");
        let cells = collect_window_cells(&worksheet, range, NumberLocale::PT);
        assert_eq!(
            cells.get("0,0").and_then(|cell| cell.value.as_deref()),
            Some("1.234,50")
        );
    }

    #[test]
    fn planned_window_collects_the_same_cells_as_a_direct_scan() {
        let mut worksheet = Worksheet::default();
        for (key, value) in [("0,0", "in"), ("3,0", "out"), ("199,25", "edge")] {
            worksheet.data.insert(
                key.to_string(),
                CellData {
                    value: Some(value.to_string()),
                    ..Default::default()
                },
            );
        }
        let range = parse_window_range("A1:Z200").expect("valid range");
        let tiles = bounded_window_tiles(range).expect("in-budget range");
        assert_eq!(tiles.len(), 1);
        let expected = CellRect {
            first_row: 0,
            first_col: 0,
            last_row: 199,
            last_col: 25,
        };
        assert_eq!(tiles[0].rect, expected);
        let rects: Vec<CellRect> = tiles.iter().map(|tile| tile.rect).collect();
        // CellData has no PartialEq; compare the rendered display values the
        // handler actually returns to the client.
        let display = |cells: HashMap<String, CellData>| {
            let mut pairs: Vec<(String, Option<String>)> = cells
                .into_iter()
                .map(|(key, cell)| (key, cell.value))
                .collect();
            pairs.sort();
            pairs
        };
        assert_eq!(
            display(collect_tile_cells(&worksheet, &rects, NumberLocale::EN)),
            display(collect_window_cells(&worksheet, range, NumberLocale::EN)),
        );
    }
}

//! Cell-structure and range-read handlers (split from [`super::cell_ops`]).

use crate::auth::{resolve_user_id, SheetUser};
use crate::state::SheetState;
use crate::types::{
    CellData, FreezePanesRequest, MergeCellsRequest, MergedCell, RangeRequest, RangeResponse,
    SaveResponse, WorksheetMetaResponse,
};
use crate::ui_fragments::Lang;
use axum::{
    extract::{Extension, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;

pub async fn handle_merge_cells(
    State(state): State<Arc<SheetState>>,
    user: Option<Extension<SheetUser>>,
    Json(req): Json<MergeCellsRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = resolve_user_id(user.as_deref());

    let session = state
        .sessions
        .get_or_load(&state, &user_id, &req.sheet_id)
        .await
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": e })),
            )
        })?;

    let mut sheet = session.sheet.write().await;
    if let Err(e) = botsheet_core::state::ensure_write_allowed(&user_id, &sheet) {
        return Err((StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": e }))));
    }

    if req.worksheet_index >= sheet.worksheets.len() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Invalid worksheet index" })),
        ));
    }

    let worksheet = &mut sheet.worksheets[req.worksheet_index];
    let merged = MergedCell {
        start_row: req.start_row,
        start_col: req.start_col,
        end_row: req.end_row,
        end_col: req.end_col,
    };

    let merged_cells = worksheet.merged_cells.get_or_insert_with(Vec::new);
    merged_cells.push(merged);

    sheet.updated_at = Utc::now();

    state.sessions.record(
        &session,
        &state,
        &user_id,
        "merge",
        serde_json::json!({
            "sheet_id": req.sheet_id,
            "worksheet_index": req.worksheet_index,
            "start_row": req.start_row,
            "start_col": req.start_col,
            "end_row": req.end_row,
            "end_col": req.end_col,
        }),
    );

    Ok(Json(SaveResponse {
        id: req.sheet_id,
        success: true,
        message: Some("Cells merged".to_string()),
    }))
}

pub async fn handle_unmerge_cells(
    State(state): State<Arc<SheetState>>,
    user: Option<Extension<SheetUser>>,
    Json(req): Json<MergeCellsRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = resolve_user_id(user.as_deref());

    let session = state
        .sessions
        .get_or_load(&state, &user_id, &req.sheet_id)
        .await
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": e })),
            )
        })?;

    let mut sheet = session.sheet.write().await;
    if let Err(e) = botsheet_core::state::ensure_write_allowed(&user_id, &sheet) {
        return Err((StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": e }))));
    }

    if req.worksheet_index >= sheet.worksheets.len() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Invalid worksheet index" })),
        ));
    }

    let worksheet = &mut sheet.worksheets[req.worksheet_index];
    if let Some(ref mut merged_cells) = worksheet.merged_cells {
        merged_cells.retain(|m| {
            !(m.start_row == req.start_row
                && m.start_col == req.start_col
                && m.end_row == req.end_row
                && m.end_col == req.end_col)
        });
    }

    sheet.updated_at = Utc::now();

    state.sessions.record(
        &session,
        &state,
        &user_id,
        "unmerge",
        serde_json::json!({
            "sheet_id": req.sheet_id,
            "worksheet_index": req.worksheet_index,
            "start_row": req.start_row,
            "start_col": req.start_col,
            "end_row": req.end_row,
            "end_col": req.end_col,
        }),
    );

    Ok(Json(SaveResponse {
        id: req.sheet_id,
        success: true,
        message: Some("Cells unmerged".to_string()),
    }))
}

pub async fn handle_freeze_panes(
    State(state): State<Arc<SheetState>>,
    user: Option<Extension<SheetUser>>,
    Json(req): Json<FreezePanesRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = resolve_user_id(user.as_deref());

    let session = state
        .sessions
        .get_or_load(&state, &user_id, &req.sheet_id)
        .await
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": e })),
            )
        })?;

    let mut sheet = session.sheet.write().await;
    if let Err(e) = botsheet_core::state::ensure_write_allowed(&user_id, &sheet) {
        return Err((StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": e }))));
    }

    if req.worksheet_index >= sheet.worksheets.len() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Invalid worksheet index" })),
        ));
    }

    let worksheet = &mut sheet.worksheets[req.worksheet_index];
    worksheet.frozen_rows = Some(req.frozen_rows);
    worksheet.frozen_cols = Some(req.frozen_cols);

    sheet.updated_at = Utc::now();

    state.sessions.record(
        &session,
        &state,
        &user_id,
        "freeze",
        serde_json::json!({
            "sheet_id": req.sheet_id,
            "worksheet_index": req.worksheet_index,
            "frozen_rows": req.frozen_rows,
            "frozen_cols": req.frozen_cols,
        }),
    );

    Ok(Json(SaveResponse {
        id: req.sheet_id,
        success: true,
        message: Some("Panes frozen".to_string()),
    }))
}

pub async fn handle_get_range(
    State(state): State<Arc<SheetState>>,
    user: Option<Extension<SheetUser>>,
    headers: HeaderMap,
    Json(req): Json<RangeRequest>,
) -> Result<Json<RangeResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = resolve_user_id(user.as_deref());

    let session = state
        .sessions
        .get_or_load(&state, &user_id, &req.sheet_id)
        .await
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": e })),
            )
        })?;

    let sheet = session.sheet.read().await;

    if req.worksheet_index >= sheet.worksheets.len() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Invalid worksheet index" })),
        ));
    }

    let worksheet = &sheet.worksheets[req.worksheet_index];
    let locale = Lang::from_headers(&headers).number_locale();

    let mut cells: HashMap<String, CellData> = HashMap::new();
    for row in req.start_row..=req.end_row {
        for col in req.start_col..=req.end_col {
            let key = format!("{},{}", row, col);
            if let Some(cell) = worksheet.data.get(&key) {
                let mut c = cell.clone();
                // Apply the stored number format to the display value while the
                // raw typed value stays available for arithmetic (#785).
                if let (Some(typed), Some(code)) = (c.typed.as_ref(), c.format.as_deref()) {
                    c.value = Some(botsheet_core::engine::formats::display_cell_locale(
                        typed, Some(code), locale,
                    ));
                }
                cells.insert(key, c);
            }
        }
    }

    let total_rows: u32 = worksheet
        .data
        .keys()
        .filter_map(|k| k.split(',').next().and_then(|r| r.parse().ok()))
        .max()
        .unwrap_or(0);
    let total_cols: u32 = worksheet
        .data
        .keys()
        .filter_map(|k| k.split(',').nth(1).and_then(|c| c.parse().ok()))
        .max()
        .unwrap_or(0);

    Ok(Json(RangeResponse {
        cells,
        total_rows,
        total_cols,
        range_start: format!("{},{}", req.start_row, req.start_col),
        range_end: format!("{},{}", req.end_row, req.end_col),
    }))
}

pub async fn handle_worksheet_meta(
    State(state): State<Arc<SheetState>>,
    user: Option<Extension<SheetUser>>,
    Json(req): Json<RangeRequest>,
) -> Result<Json<WorksheetMetaResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = resolve_user_id(user.as_deref());

    let session = state
        .sessions
        .get_or_load(&state, &user_id, &req.sheet_id)
        .await
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": e })),
            )
        })?;

    let sheet = session.sheet.read().await;

    if req.worksheet_index >= sheet.worksheets.len() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Invalid worksheet index" })),
        ));
    }

    let worksheet = &sheet.worksheets[req.worksheet_index];

    let total_rows: u32 = worksheet
        .data
        .keys()
        .filter_map(|k| k.split(',').next().and_then(|r| r.parse().ok()))
        .max()
        .unwrap_or(0);
    let total_cols: u32 = worksheet
        .data
        .keys()
        .filter_map(|k| k.split(',').nth(1).and_then(|c| c.parse().ok()))
        .max()
        .unwrap_or(0);

    Ok(Json(WorksheetMetaResponse {
        total_rows,
        total_cols,
        name: worksheet.name.clone(),
        frozen_rows: worksheet.frozen_rows.unwrap_or(0),
        frozen_cols: worksheet.frozen_cols.unwrap_or(0),
    }))
}

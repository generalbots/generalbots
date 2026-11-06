//! Sort and filter mutations.

use crate::auth::{resolve_user_id, SheetUser};
use crate::state::SheetState;
use crate::types::{
    CellData, ClearFilterRequest, FilterConfig, FilterRequest, SaveResponse, SortRequest,
};
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    Json,
};
use botsheet_core::engine::value::CellValue;
use chrono::Utc;
use std::sync::Arc;

pub async fn handle_sort_range(
    State(state): State<Arc<SheetState>>,
    user: Option<Extension<SheetUser>>,
    Json(req): Json<SortRequest>,
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

    let mut rows: Vec<Vec<Option<CellData>>> = Vec::new();
    for row in req.start_row..=req.end_row {
        let mut row_data = Vec::new();
        for col in req.start_col..=req.end_col {
            let key = format!("{},{}", row, col);
            row_data.push(worksheet.data.get(&key).cloned());
        }
        rows.push(row_data);
    }

    let sort_col_idx = (req.sort_col - req.start_col) as usize;
    rows.sort_by(|a, b| {
        let cell_a = a.get(sort_col_idx).and_then(|c| c.as_ref());
        let cell_b = b.get(sort_col_idx).and_then(|c| c.as_ref());

        // A typed Number sorts numerically; any other typed value (text, bool,
        // error, empty) sorts by its display string so a text `0123` is not
        // re-parsed into the number 123. Untyped cells (CSV/ODS) fall back to
        // a numeric parse of the display value.
        let num_a = match cell_a.and_then(|c| c.typed.as_ref()) {
            Some(CellValue::Number(n)) => Some(*n),
            Some(_) => None,
            None => cell_a
                .and_then(|c| c.value.as_deref())
                .and_then(|v| v.parse::<f64>().ok()),
        };
        let num_b = match cell_b.and_then(|c| c.typed.as_ref()) {
            Some(CellValue::Number(n)) => Some(*n),
            Some(_) => None,
            None => cell_b
                .and_then(|c| c.value.as_deref())
                .and_then(|v| v.parse::<f64>().ok()),
        };

        let cmp = match (num_a, num_b) {
            (Some(na), Some(nb)) => na.partial_cmp(&nb).unwrap_or(std::cmp::Ordering::Equal),
            _ => {
                let str_a = cell_a.and_then(|c| c.value.as_deref()).unwrap_or_default();
                let str_b = cell_b.and_then(|c| c.value.as_deref()).unwrap_or_default();
                str_a.cmp(str_b)
            }
        };

        if req.ascending { cmp } else { cmp.reverse() }
    });

    for (row_offset, row_data) in rows.iter().enumerate() {
        for (col_offset, cell) in row_data.iter().enumerate() {
            let key = format!(
                "{},{}",
                req.start_row + row_offset as u32,
                req.start_col + col_offset as u32
            );
            if let Some(c) = cell {
                worksheet.data.insert(key, c.clone());
            } else {
                worksheet.data.remove(&key);
            }
        }
    }

    // Re-sorting moves values between rows; recompute which rows the active
    // filters hide so the grid stays consistent (#790).
    worksheet.hidden_rows = Some(botsheet_core::filter::compute_hidden_rows(worksheet));

    sheet.updated_at = Utc::now();
    state.sessions.record(
        &session,
        &state,
        &user_id,
        "sheet_mutation",
        serde_json::json!({ "sheet_id": req.sheet_id }),
    );

    Ok(Json(SaveResponse {
        id: req.sheet_id,
        success: true,
        message: Some("Range sorted".to_string()),
    }))
}

pub async fn handle_filter_data(
    State(state): State<Arc<SheetState>>,
    user: Option<Extension<SheetUser>>,
    Json(req): Json<FilterRequest>,
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
    let filters = worksheet.filters.get_or_insert_with(std::collections::HashMap::new);

    filters.insert(
        req.col,
        FilterConfig {
            filter_type: req.filter_type,
            values: req.values,
            condition: req.condition,
            value1: req.value1,
            value2: req.value2,
        },
    );

    // Materialize the hidden rows the filter implies so grid views can render
    // them without re-evaluating filter semantics (#790).
    worksheet.hidden_rows = Some(botsheet_core::filter::compute_hidden_rows(worksheet));

    sheet.updated_at = Utc::now();
    state.sessions.record(
        &session,
        &state,
        &user_id,
        "sheet_mutation",
        serde_json::json!({ "sheet_id": req.sheet_id }),
    );

    Ok(Json(SaveResponse {
        id: req.sheet_id,
        success: true,
        message: Some("Filter applied".to_string()),
    }))
}

pub async fn handle_clear_filter(
    State(state): State<Arc<SheetState>>,
    user: Option<Extension<SheetUser>>,
    Json(req): Json<ClearFilterRequest>,
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
    if let Some(ref mut filters) = worksheet.filters {
        if let Some(col) = req.col {
            filters.remove(&col);
        } else {
            filters.clear();
        }
    }

    // Clearing a filter un-hides every row (#790).
    worksheet.hidden_rows = Some(botsheet_core::filter::compute_hidden_rows(worksheet));

    sheet.updated_at = Utc::now();
    state.sessions.record(
        &session,
        &state,
        &user_id,
        "sheet_mutation",
        serde_json::json!({ "sheet_id": req.sheet_id }),
    );

    Ok(Json(SaveResponse {
        id: req.sheet_id,
        success: true,
        message: Some("Filter cleared".to_string()),
    }))
}

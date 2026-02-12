use crate::core::shared::state::AppState;
use crate::sheet::collaboration::broadcast_sheet_change;
use crate::sheet::formulas::evaluate_formula;
use crate::sheet::storage::{get_current_user_id, load_sheet_by_id, save_sheet_to_drive};
use crate::sheet::types::{
    CellData, CellUpdateRequest, FormatRequest, FormulaRequest, FormulaResult, FreezePanesRequest,
    MergeCellsRequest, MergedCell, SaveResponse, Worksheet,
};
use axum::{extract::State, http::StatusCode, Json};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;

pub async fn handle_update_cell(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CellUpdateRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();

    let mut sheet = match load_sheet_by_id(&state, &user_id, &req.sheet_id).await {
        Ok(s) => s,
        Err(e) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": e })),
            ))
        }
    };

    if req.worksheet_index >= sheet.worksheets.len() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Invalid worksheet index" })),
        ));
    }

    let worksheet = &mut sheet.worksheets[req.worksheet_index];
    let key = format!("{},{}", req.row, req.col);

    let (value, formula) = if req.value.starts_with('=') {
        let result = evaluate_formula(&req.value, worksheet);
        (Some(result.value), Some(req.value.clone()))
    } else {
        (Some(req.value.clone()), None)
    };

    let cell = worksheet.data.entry(key).or_insert_with(|| CellData {
        value: None,
        formula: None,
        style: None,
        format: None,
        note: None,
        locked: None,
        has_comment: None,
        array_formula_id: None,
    });

    cell.value = value;
    cell.formula = formula;

    sheet.updated_at = Utc::now();

    if let Err(e) = save_sheet_to_drive(&state, &user_id, &sheet).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ));
    }

    broadcast_sheet_change(
        &req.sheet_id,
        &user_id,
        "User",
        req.row,
        req.col,
        &req.value,
        req.worksheet_index,
    )
    .await;

    Ok(Json(SaveResponse {
        id: req.sheet_id,
        success: true,
        message: Some("Cell updated".to_string()),
    }))
}

pub async fn handle_format_cells(
    State(state): State<Arc<AppState>>,
    Json(req): Json<FormatRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();

    let mut sheet = match load_sheet_by_id(&state, &user_id, &req.sheet_id).await {
        Ok(s) => s,
        Err(e) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": e })),
            ))
        }
    };

    if req.worksheet_index >= sheet.worksheets.len() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Invalid worksheet index" })),
        ));
    }

    let worksheet = &mut sheet.worksheets[req.worksheet_index];

    for row in req.start_row..=req.end_row {
        for col in req.start_col..=req.end_col {
            let key = format!("{},{}", row, col);
            let cell = worksheet.data.entry(key).or_insert_with(|| CellData {
                value: None,
                formula: None,
                style: None,
                format: None,
                note: None,
                locked: None,
                has_comment: None,
                array_formula_id: None,
            });
            cell.style = Some(req.style.clone());
        }
    }

    sheet.updated_at = Utc::now();

    if let Err(e) = save_sheet_to_drive(&state, &user_id, &sheet).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ));
    }

    Ok(Json(SaveResponse {
        id: req.sheet_id,
        success: true,
        message: Some("Format applied".to_string()),
    }))
}

pub async fn handle_evaluate_formula(
    State(state): State<Arc<AppState>>,
    Json(req): Json<FormulaRequest>,
) -> Result<Json<FormulaResult>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();

    let sheet = match load_sheet_by_id(&state, &user_id, &req.sheet_id).await {
        Ok(s) => s,
        Err(_) => {
            return Ok(Json(evaluate_formula(
                &req.formula,
                &Worksheet {
                    name: "temp".to_string(),
                    data: HashMap::new(),
                    column_widths: None,
                    row_heights: None,
                    frozen_rows: None,
                    frozen_cols: None,
                    merged_cells: None,
                    filters: None,
                    hidden_rows: None,
                    validations: None,
                    conditional_formats: None,
                    charts: None,
                    comments: None,
                    protection: None,
                    array_formulas: None,
                },
            )))
        }
    };

    if req.worksheet_index >= sheet.worksheets.len() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Invalid worksheet index" })),
        ));
    }

    let result = evaluate_formula(&req.formula, &sheet.worksheets[req.worksheet_index]);
    Ok(Json(result))
}

pub async fn handle_merge_cells(
    State(state): State<Arc<AppState>>,
    Json(req): Json<MergeCellsRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut sheet = match load_sheet_by_id(&state, &user_id, &req.sheet_id).await {
        Ok(s) => s,
        Err(e) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": e })),
            ))
        }
    };

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
    if let Err(e) = save_sheet_to_drive(&state, &user_id, &sheet).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ));
    }

    Ok(Json(SaveResponse {
        id: req.sheet_id,
        success: true,
        message: Some("Cells merged".to_string()),
    }))
}

pub async fn handle_unmerge_cells(
    State(state): State<Arc<AppState>>,
    Json(req): Json<MergeCellsRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut sheet = match load_sheet_by_id(&state, &user_id, &req.sheet_id).await {
        Ok(s) => s,
        Err(e) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": e })),
            ))
        }
    };

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
    if let Err(e) = save_sheet_to_drive(&state, &user_id, &sheet).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ));
    }

    Ok(Json(SaveResponse {
        id: req.sheet_id,
        success: true,
        message: Some("Cells unmerged".to_string()),
    }))
}

pub async fn handle_freeze_panes(
    State(state): State<Arc<AppState>>,
    Json(req): Json<FreezePanesRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut sheet = match load_sheet_by_id(&state, &user_id, &req.sheet_id).await {
        Ok(s) => s,
        Err(e) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": e })),
            ))
        }
    };

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
    if let Err(e) = save_sheet_to_drive(&state, &user_id, &sheet).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ));
    }

    Ok(Json(SaveResponse {
        id: req.sheet_id,
        success: true,
        message: Some("Panes frozen".to_string()),
    }))
}

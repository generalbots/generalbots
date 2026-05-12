use crate::state::{get_current_user_id, load_sheet_by_id, save_sheet_to_drive, SheetState};
use crate::types::{
    ArrayFormula, ArrayFormulaRequest, CellData, CreateNamedRangeRequest,
    DeleteArrayFormulaRequest, DeleteNamedRangeRequest, ListNamedRangesResponse, NamedRange,
    SaveResponse, UpdateNamedRangeRequest,
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

pub async fn handle_array_formula(
    State(state): State<Arc<SheetState>>,
    Json(req): Json<ArrayFormulaRequest>,
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

    let array_formula_id = Uuid::new_v4().to_string();
    let array_formula = ArrayFormula {
        id: array_formula_id.clone(),
        formula: req.formula.clone(),
        start_row: req.start_row,
        start_col: req.start_col,
        end_row: req.end_row,
        end_col: req.end_col,
        is_dynamic: req.formula.starts_with('=') && req.formula.contains('#'),
    };

    let worksheet = &mut sheet.worksheets[req.worksheet_index];
    let array_formulas = worksheet.array_formulas.get_or_insert_with(Vec::new);
    array_formulas.push(array_formula);

    for row in req.start_row..=req.end_row {
        for col in req.start_col..=req.end_col {
            let key = format!("{row},{col}");
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
            cell.array_formula_id = Some(array_formula_id.clone());
            if row == req.start_row && col == req.start_col {
                cell.formula = Some(format!("{{{}}}", req.formula));
            }
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
        message: Some("Array formula created".to_string()),
    }))
}

pub async fn handle_delete_array_formula(
    State(state): State<Arc<SheetState>>,
    Json(req): Json<DeleteArrayFormulaRequest>,
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

    if let Some(array_formulas) = &mut worksheet.array_formulas {
        array_formulas.retain(|af| af.id != req.array_formula_id);
    }

    for cell in worksheet.data.values_mut() {
        if cell.array_formula_id.as_ref() == Some(&req.array_formula_id) {
            cell.array_formula_id = None;
            cell.formula = None;
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
        message: Some("Array formula deleted".to_string()),
    }))
}

pub async fn handle_create_named_range(
    State(state): State<Arc<SheetState>>,
    Json(req): Json<CreateNamedRangeRequest>,
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

    let named_range = NamedRange {
        id: Uuid::new_v4().to_string(),
        name: req.name,
        scope: req.scope,
        worksheet_index: req.worksheet_index,
        start_row: req.start_row,
        start_col: req.start_col,
        end_row: req.end_row,
        end_col: req.end_col,
        comment: req.comment,
    };

    let named_ranges = sheet.named_ranges.get_or_insert_with(Vec::new);
    named_ranges.push(named_range);

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
        message: Some("Named range created".to_string()),
    }))
}

pub async fn handle_update_named_range(
    State(state): State<Arc<SheetState>>,
    Json(req): Json<UpdateNamedRangeRequest>,
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

    if let Some(named_ranges) = &mut sheet.named_ranges {
        for range in named_ranges.iter_mut() {
            if range.id == req.range_id {
                if let Some(ref name) = req.name {
                    range.name = name.clone();
                }
                if let Some(start_row) = req.start_row {
                    range.start_row = start_row;
                }
                if let Some(start_col) = req.start_col {
                    range.start_col = start_col;
                }
                if let Some(end_row) = req.end_row {
                    range.end_row = end_row;
                }
                if let Some(end_col) = req.end_col {
                    range.end_col = end_col;
                }
                if let Some(ref comment) = req.comment {
                    range.comment = Some(comment.clone());
                }
            }
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
        message: Some("Named range updated".to_string()),
    }))
}

pub async fn handle_delete_named_range(
    State(state): State<Arc<SheetState>>,
    Json(req): Json<DeleteNamedRangeRequest>,
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

    if let Some(named_ranges) = &mut sheet.named_ranges {
        named_ranges.retain(|r| r.id != req.range_id);
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
        message: Some("Named range deleted".to_string()),
    }))
}

pub async fn handle_list_named_ranges(
    State(state): State<Arc<SheetState>>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ListNamedRangesResponse>, (StatusCode, Json<serde_json::Value>)> {
    let sheet_id = params.get("sheet_id").cloned().unwrap_or_default();
    let user_id = get_current_user_id();
    let sheet = match load_sheet_by_id(&state, &user_id, &sheet_id).await {
        Ok(s) => s,
        Err(e) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": e })),
            ))
        }
    };

    let ranges = sheet.named_ranges.unwrap_or_default();
    Ok(Json(ListNamedRangesResponse { ranges }))
}

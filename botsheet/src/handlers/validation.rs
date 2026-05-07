use crate::state::{get_current_user_id, load_sheet_by_id, save_sheet_to_drive, SheetState};
use crate::types::{
    AddNoteRequest, CellData, DataValidationRequest, SaveResponse, ValidateCellRequest,
    ValidationResult, ValidationRule,
};
use axum::{extract::State, http::StatusCode, Json};
use chrono::Utc;
use std::sync::Arc;

pub async fn handle_data_validation(
    State(state): State<Arc<SheetState>>,
    Json(req): Json<DataValidationRequest>,
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
    let validations = worksheet
        .validations
        .get_or_insert_with(std::collections::HashMap::new);

    for row in req.start_row..=req.end_row {
        for col in req.start_col..=req.end_col {
            let key = format!("{},{}", row, col);
            validations.insert(
                key,
                ValidationRule {
                    validation_type: req.validation_type.clone(),
                    operator: req.operator.clone(),
                    value1: req.value1.clone(),
                    value2: req.value2.clone(),
                    allowed_values: req.allowed_values.clone(),
                    error_title: None,
                    error_message: req.error_message.clone(),
                    input_title: None,
                    input_message: None,
                },
            );
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
        message: Some("Data validation applied".to_string()),
    }))
}

pub async fn handle_validate_cell(
    State(state): State<Arc<SheetState>>,
    Json(req): Json<ValidateCellRequest>,
) -> Result<Json<ValidationResult>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let sheet = match load_sheet_by_id(&state, &user_id, &req.sheet_id).await {
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

    let worksheet = &sheet.worksheets[req.worksheet_index];
    let key = format!("{},{}", req.row, req.col);

    if let Some(ref validations) = worksheet.validations {
        if let Some(rule) = validations.get(&key) {
            let result = validate_value(&req.value, rule);
            return Ok(Json(result));
        }
    }

    Ok(Json(ValidationResult {
        valid: true,
        error_message: None,
    }))
}

fn validate_value(value: &str, rule: &ValidationRule) -> ValidationResult {
    let valid = match rule.validation_type.as_str() {
        "number" => value.parse::<f64>().is_ok(),
        "integer" => value.parse::<i64>().is_ok(),
        "list" => rule
            .allowed_values
            .as_ref()
            .map(|v| v.contains(&value.to_string()))
            .unwrap_or(true),
        "date" => chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d").is_ok(),
        "text_length" => {
            let len = value.len();
            let min = rule
                .value1
                .as_ref()
                .and_then(|v| v.parse::<usize>().ok())
                .unwrap_or(0);
            let max = rule
                .value2
                .as_ref()
                .and_then(|v| v.parse::<usize>().ok())
                .unwrap_or(usize::MAX);
            len >= min && len <= max
        }
        _ => true,
    };

    ValidationResult {
        valid,
        error_message: if valid {
            None
        } else {
            rule.error_message
                .clone()
                .or_else(|| Some("Invalid value".to_string()))
        },
    }
}

pub async fn handle_add_note(
    State(state): State<Arc<SheetState>>,
    Json(req): Json<AddNoteRequest>,
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
    cell.note = Some(req.note);

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
        message: Some("Note added".to_string()),
    }))
}

use crate::auth::{resolve_user_id, SheetUser};
use crate::state::SheetState;
use crate::types::{
    AddNoteRequest, CellData, DataValidationRequest, SaveResponse, ValidateCellRequest,
    ValidationResult, ValidationRule,
};
use axum::{extract::{Extension, State}, http::StatusCode, Json};
use chrono::Utc;
use std::sync::Arc;

pub async fn handle_data_validation(
    State(state): State<Arc<SheetState>>,
    user: Option<Extension<SheetUser>>,
    Json(req): Json<DataValidationRequest>,
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
        message: Some("Data validation applied".to_string()),
    }))
}

pub async fn handle_validate_cell(
    State(state): State<Arc<SheetState>>,
    user: Option<Extension<SheetUser>>,
    Json(req): Json<ValidateCellRequest>,
) -> Result<Json<ValidationResult>, (StatusCode, Json<serde_json::Value>)> {
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
    let sheet = session.sheet.read().await.clone();

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
    user: Option<Extension<SheetUser>>,
    Json(req): Json<AddNoteRequest>,
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
    let key = format!("{},{}", req.row, req.col);

    let cell = worksheet.data.entry(key).or_insert_with(|| CellData {
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
    cell.note = Some(req.note);

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
        message: Some("Note added".to_string()),
    }))
}

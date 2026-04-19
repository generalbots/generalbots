use crate::core::shared::state::AppState;
use crate::sheet::storage::{get_current_user_id, load_sheet_by_id, save_sheet_to_drive};
use crate::sheet::types::{
    AddCommentRequest, AddNoteRequest, CellComment, CellData, CommentReply, CommentWithLocation,
    DataValidationRequest, DeleteCommentRequest, ListCommentsRequest, ListCommentsResponse,
    ReplyCommentRequest, ResolveCommentRequest, SaveResponse, ValidateCellRequest,
    ValidationResult, ValidationRule,
};
use axum::{extract::State, http::StatusCode, Json};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

pub async fn handle_data_validation(
    State(state): State<Arc<AppState>>,
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
    State(state): State<Arc<AppState>>,
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
    State(state): State<Arc<AppState>>,
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

pub async fn handle_add_comment(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddCommentRequest>,
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

    let comment = CellComment {
        id: Uuid::new_v4().to_string(),
        author_id: user_id.clone(),
        author_name: "User".to_string(),
        content: req.content,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        replies: vec![],
        resolved: false,
    };

    let comments = worksheet
        .comments
        .get_or_insert_with(std::collections::HashMap::new);
    comments.insert(key.clone(), comment);

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
    cell.has_comment = Some(true);

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
        message: Some("Comment added".to_string()),
    }))
}

pub async fn handle_reply_comment(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ReplyCommentRequest>,
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

    if let Some(comments) = &mut worksheet.comments {
        if let Some(comment) = comments.get_mut(&key) {
            if comment.id == req.comment_id {
                let reply = CommentReply {
                    id: Uuid::new_v4().to_string(),
                    author_id: user_id.clone(),
                    author_name: "User".to_string(),
                    content: req.content,
                    created_at: Utc::now(),
                };
                comment.replies.push(reply);
                comment.updated_at = Utc::now();
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
        message: Some("Reply added".to_string()),
    }))
}

pub async fn handle_resolve_comment(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ResolveCommentRequest>,
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

    if let Some(comments) = &mut worksheet.comments {
        if let Some(comment) = comments.get_mut(&key) {
            if comment.id == req.comment_id {
                comment.resolved = req.resolved;
                comment.updated_at = Utc::now();
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
        message: Some("Comment resolved".to_string()),
    }))
}

pub async fn handle_delete_comment(
    State(state): State<Arc<AppState>>,
    Json(req): Json<DeleteCommentRequest>,
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

    if let Some(comments) = &mut worksheet.comments {
        comments.remove(&key);
    }

    if let Some(cell) = worksheet.data.get_mut(&key) {
        cell.has_comment = Some(false);
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
        message: Some("Comment deleted".to_string()),
    }))
}

pub async fn handle_list_comments(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ListCommentsRequest>,
) -> Result<Json<ListCommentsResponse>, (StatusCode, Json<serde_json::Value>)> {
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
    let mut comments_list = vec![];

    if let Some(comments) = &worksheet.comments {
        for (key, comment) in comments {
            let parts: Vec<&str> = key.split(',').collect();
            if parts.len() == 2 {
                if let (Ok(row), Ok(col)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                    comments_list.push(CommentWithLocation {
                        row,
                        col,
                        comment: comment.clone(),
                    });
                }
            }
        }
    }

    Ok(Json(ListCommentsResponse {
        comments: comments_list,
    }))
}

use crate::core::shared::state::AppState;
use crate::sheet::storage::{get_current_user_id, load_sheet_by_id, save_sheet_to_drive};
use crate::sheet::types::{
    AddExternalLinkRequest, ArrayFormula, ArrayFormulaRequest, CellData,
    CreateNamedRangeRequest, DeleteArrayFormulaRequest, DeleteNamedRangeRequest, ExternalLink,
    ListExternalLinksResponse, ListNamedRangesResponse, LockCellsRequest, NamedRange,
    ProtectSheetRequest, RefreshExternalLinkRequest, RemoveExternalLinkRequest, SaveResponse,
    UnprotectSheetRequest, UpdateNamedRangeRequest,
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::Arc;
use uuid::Uuid;

pub async fn handle_protect_sheet(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ProtectSheetRequest>,
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

    let mut protection = req.protection;
    if let Some(password) = req.password {
        let mut hasher = DefaultHasher::new();
        password.hash(&mut hasher);
        protection.password_hash = Some(format!("{:x}", hasher.finish()));
    }

    sheet.worksheets[req.worksheet_index].protection = Some(protection);
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
        message: Some("Sheet protected".to_string()),
    }))
}

pub async fn handle_unprotect_sheet(
    State(state): State<Arc<AppState>>,
    Json(req): Json<UnprotectSheetRequest>,
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
    if let Some(protection) = &worksheet.protection {
        if let Some(hash) = &protection.password_hash {
            if let Some(password) = &req.password {
                let mut hasher = DefaultHasher::new();
                password.hash(&mut hasher);
                let provided_hash = format!("{:x}", hasher.finish());
                if &provided_hash != hash {
                    return Err((
                        StatusCode::UNAUTHORIZED,
                        Json(serde_json::json!({ "error": "Invalid password" })),
                    ));
                }
            } else {
                return Err((
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({ "error": "Password required" })),
                ));
            }
        }
    }

    worksheet.protection = None;
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
        message: Some("Sheet unprotected".to_string()),
    }))
}

pub async fn handle_lock_cells(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LockCellsRequest>,
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
            cell.locked = Some(req.locked);
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
        message: Some(
            if req.locked {
                "Cells locked"
            } else {
                "Cells unlocked"
            }
            .to_string(),
        ),
    }))
}

pub async fn handle_add_external_link(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddExternalLinkRequest>,
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

    let link = ExternalLink {
        id: Uuid::new_v4().to_string(),
        source_path: req.source_path,
        link_type: req.link_type,
        target_sheet: req.target_sheet,
        target_range: req.target_range,
        status: "active".to_string(),
        last_updated: Utc::now(),
    };

    let links = sheet.external_links.get_or_insert_with(Vec::new);
    links.push(link);

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
        message: Some("External link added".to_string()),
    }))
}

pub async fn handle_refresh_external_link(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RefreshExternalLinkRequest>,
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

    if let Some(links) = &mut sheet.external_links {
        for link in links.iter_mut() {
            if link.id == req.link_id {
                link.last_updated = Utc::now();
                link.status = "refreshed".to_string();
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
        message: Some("External link refreshed".to_string()),
    }))
}

pub async fn handle_remove_external_link(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RemoveExternalLinkRequest>,
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

    if let Some(links) = &mut sheet.external_links {
        links.retain(|link| link.id != req.link_id);
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
        message: Some("External link removed".to_string()),
    }))
}

pub async fn handle_list_external_links(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ListExternalLinksResponse>, (StatusCode, Json<serde_json::Value>)> {
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

    let links = sheet.external_links.unwrap_or_default();
    Ok(Json(ListExternalLinksResponse { links }))
}

pub async fn handle_array_formula(
    State(state): State<Arc<AppState>>,
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
    State(state): State<Arc<AppState>>,
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
    State(state): State<Arc<AppState>>,
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
    State(state): State<Arc<AppState>>,
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
    State(state): State<Arc<AppState>>,
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
    State(state): State<Arc<AppState>>,
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

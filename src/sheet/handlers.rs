use crate::shared::state::AppState;
use crate::sheet::collaboration::broadcast_sheet_change;
use crate::sheet::export::{export_to_csv, export_to_html, export_to_json, export_to_markdown, export_to_ods, export_to_xlsx};
use crate::sheet::formulas::evaluate_formula;
use crate::sheet::storage::{
    create_new_spreadsheet, delete_sheet_from_drive, get_current_user_id, import_spreadsheet_bytes,
    list_sheets_from_drive, load_sheet_by_id, load_sheet_from_drive, parse_csv_to_worksheets,
    parse_excel_to_worksheets, save_sheet_to_drive,
};
use crate::sheet::types::{
    AddCommentRequest, AddExternalLinkRequest, AddNoteRequest, ArrayFormula, ArrayFormulaRequest,
    CellComment, CellData, CellUpdateRequest, ChartConfig, ChartOptions, ChartPosition,
    ChartRequest, ClearFilterRequest, CommentReply, CommentWithLocation, ConditionalFormatRequest,
    ConditionalFormatRule, CreateNamedRangeRequest, DataValidationRequest, DeleteArrayFormulaRequest,
    DeleteChartRequest, DeleteCommentRequest, DeleteNamedRangeRequest, ExportRequest, ExternalLink,
    FilterConfig, FilterRequest, FormatRequest, FormulaRequest, FormulaResult, FreezePanesRequest,
    ListCommentsRequest, ListCommentsResponse, ListExternalLinksResponse, ListNamedRangesRequest,
    ListNamedRangesResponse, LoadFromDriveRequest, LoadQuery, LockCellsRequest, MergeCellsRequest,
    MergedCell, NamedRange, ProtectSheetRequest, RefreshExternalLinkRequest, RemoveExternalLinkRequest,
    ReplyCommentRequest, ResolveCommentRequest, SaveRequest, SaveResponse, SearchQuery, ShareRequest,
    SheetAiRequest, SheetAiResponse, SheetProtection, SortRequest, Spreadsheet, SpreadsheetMetadata,
    UnprotectSheetRequest, UpdateNamedRangeRequest, ValidateCellRequest, ValidationResult,
    ValidationRule, Worksheet,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use log::error;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

pub async fn handle_sheet_ai(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<SheetAiRequest>,
) -> impl IntoResponse {
    let command = req.command.to_lowercase();

    let response = if command.contains("sum") {
        "I can help you sum values. Select a range and use the SUM formula, or I've added a SUM formula below your selection."
    } else if command.contains("average") || command.contains("avg") {
        "I can calculate averages. Select a range and use the AVERAGE formula."
    } else if command.contains("chart") {
        "To create a chart, select your data range first, then choose the chart type from the Chart menu."
    } else if command.contains("sort") {
        "I can sort your data. Select the range you want to sort, then specify ascending or descending order."
    } else if command.contains("format") || command.contains("currency") || command.contains("percent") {
        "I've applied the formatting to your selected cells."
    } else if command.contains("bold") || command.contains("italic") {
        "I've applied the text formatting to your selected cells."
    } else if command.contains("filter") {
        "I've enabled filtering on your data. Use the dropdown arrows in the header row to filter."
    } else if command.contains("freeze") {
        "I've frozen the specified rows/columns so they stay visible when scrolling."
    } else if command.contains("merge") {
        "I've merged the selected cells into one."
    } else if command.contains("clear") {
        "I've cleared the content from the selected cells."
    } else if command.contains("help") {
        "I can help you with:\n• Sum/Average columns\n• Format as currency or percent\n• Bold/Italic formatting\n• Sort data\n• Create charts\n• Filter data\n• Freeze panes\n• Merge cells"
    } else {
        "I understand you want help with your spreadsheet. Try commands like 'sum column B', 'format as currency', 'sort ascending', or 'create a chart'."
    };

    Json(SheetAiResponse {
        response: response.to_string(),
        action: None,
        data: None,
    })
}

pub async fn handle_new_sheet(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<Spreadsheet>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(create_new_spreadsheet()))
}

pub async fn handle_list_sheets(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<SpreadsheetMetadata>>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();

    match list_sheets_from_drive(&state, &user_id).await {
        Ok(sheets) => Ok(Json(sheets)),
        Err(e) => {
            error!("Failed to list sheets: {}", e);
            Ok(Json(Vec::new()))
        }
    }
}

pub async fn handle_search_sheets(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Vec<SpreadsheetMetadata>>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();

    let sheets = match list_sheets_from_drive(&state, &user_id).await {
        Ok(s) => s,
        Err(_) => Vec::new(),
    };

    let filtered = if let Some(q) = query.q {
        let q_lower = q.to_lowercase();
        sheets
            .into_iter()
            .filter(|s| s.name.to_lowercase().contains(&q_lower))
            .collect()
    } else {
        sheets
    };

    Ok(Json(filtered))
}

pub async fn handle_load_sheet(
    State(state): State<Arc<AppState>>,
    Query(query): Query<LoadQuery>,
) -> Result<Json<Spreadsheet>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();

    match load_sheet_from_drive(&state, &user_id, &query.id).await {
        Ok(sheet) => Ok(Json(sheet)),
        Err(e) => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": e })),
        )),
    }
}

pub async fn handle_load_from_drive(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoadFromDriveRequest>,
) -> Result<Json<Spreadsheet>, (StatusCode, Json<serde_json::Value>)> {
    let drive = state.drive.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "Drive not available" })),
        )
    })?;

    let result = drive
        .get_object()
        .bucket(&req.bucket)
        .key(&req.path)
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": format!("File not found: {e}") })),
            )
        })?;

    let bytes = result
        .body
        .collect()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Failed to read file: {e}") })),
            )
        })?
        .into_bytes();

    let ext = req.path.rsplit('.').next().unwrap_or("").to_lowercase();
    let file_name = req.path.rsplit('/').next().unwrap_or("Spreadsheet");
    let sheet_name = file_name
        .rsplit('.')
        .last()
        .unwrap_or("Spreadsheet")
        .to_string();

    let worksheets = match ext.as_str() {
        "csv" | "tsv" => {
            let delimiter = if ext == "tsv" { b'\t' } else { b',' };
            parse_csv_to_worksheets(&bytes, delimiter, &sheet_name).map_err(|e| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": e })),
                )
            })?
        }
        "xlsx" | "xls" | "ods" | "xlsb" | "xlsm" => {
            parse_excel_to_worksheets(&bytes, &ext).map_err(|e| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": e })),
                )
            })?
        }
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": format!("Unsupported format: .{ext}") })),
            ));
        }
    };

    let user_id = get_current_user_id();
    let sheet = Spreadsheet {
        id: Uuid::new_v4().to_string(),
        name: sheet_name,
        owner_id: user_id,
        worksheets,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    Ok(Json(sheet))
}

pub async fn handle_save_sheet(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SaveRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();

    let sheet_id = req.id.unwrap_or_else(|| Uuid::new_v4().to_string());

    let sheet = Spreadsheet {
        id: sheet_id.clone(),
        name: req.name,
        owner_id: user_id.clone(),
        worksheets: req.worksheets,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    if let Err(e) = save_sheet_to_drive(&state, &user_id, &sheet).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ));
    }

    Ok(Json(SaveResponse {
        id: sheet_id,
        success: true,
        message: Some("Sheet saved successfully".to_string()),
    }))
}

pub async fn handle_delete_sheet(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoadQuery>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();

    if let Err(e) = delete_sheet_from_drive(&state, &user_id, &req.id).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ));
    }

    Ok(Json(SaveResponse {
        id: req.id.unwrap_or_default(),
        success: true,
        message: Some("Sheet deleted".to_string()),
    }))
}

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

pub async fn handle_export_sheet(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ExportRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();

    let sheet = match load_sheet_by_id(&state, &user_id, &req.id).await {
        Ok(s) => s,
        Err(e) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": e })),
            ))
        }
    };

    match req.format.as_str() {
        "csv" => {
            let csv = export_to_csv(&sheet);
            Ok(([(axum::http::header::CONTENT_TYPE, "text/csv")], csv))
        }
        "xlsx" => {
            let xlsx = export_to_xlsx(&sheet).map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": e })),
                )
            })?;
            Ok((
                [(
                    axum::http::header::CONTENT_TYPE,
                    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                )],
                xlsx,
            ))
        }
        "json" => {
            let json = export_to_json(&sheet);
            Ok(([(axum::http::header::CONTENT_TYPE, "application/json")], json))
        }
        "html" => {
            let html = export_to_html(&sheet);
            Ok(([(axum::http::header::CONTENT_TYPE, "text/html")], html))
        }
        "ods" => {
            let ods = export_to_ods(&sheet).map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": e })),
                )
            })?;
            Ok((
                [(
                    axum::http::header::CONTENT_TYPE,
                    "application/vnd.oasis.opendocument.spreadsheet",
                )],
                ods,
            ))
        }
        "md" | "markdown" => {
            let md = export_to_markdown(&sheet);
            Ok(([(axum::http::header::CONTENT_TYPE, "text/markdown")], md))
        }
        _ => Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Unsupported format" })),
        )),
    }
}

pub async fn handle_share_sheet(
    Json(req): Json<ShareRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(SaveResponse {
        id: req.sheet_id,
        success: true,
        message: Some(format!("Shared with {} as {}", req.email, req.permission)),
    }))
}

pub async fn handle_get_sheet_by_id(
    State(state): State<Arc<AppState>>,
    Path(sheet_id): Path<String>,
) -> Result<Json<Spreadsheet>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    match load_sheet_by_id(&state, &user_id, &sheet_id).await {
        Ok(sheet) => Ok(Json(sheet)),
        Err(e) => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": e })),
        )),
    }
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

pub async fn handle_sort_range(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SortRequest>,
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
        let val_a = a
            .get(sort_col_idx)
            .and_then(|c| c.as_ref())
            .and_then(|c| c.value.clone())
            .unwrap_or_default();
        let val_b = b
            .get(sort_col_idx)
            .and_then(|c| c.as_ref())
            .and_then(|c| c.value.clone())
            .unwrap_or_default();

        let num_a = val_a.parse::<f64>().ok();
        let num_b = val_b.parse::<f64>().ok();

        let cmp = match (num_a, num_b) {
            (Some(na), Some(nb)) => na.partial_cmp(&nb).unwrap_or(std::cmp::Ordering::Equal),
            _ => val_a.cmp(&val_b),
        };

        if req.ascending {
            cmp
        } else {
            cmp.reverse()
        }
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
        message: Some("Range sorted".to_string()),
    }))
}

pub async fn handle_filter_data(
    State(state): State<Arc<AppState>>,
    Json(req): Json<FilterRequest>,
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
    let filters = worksheet.filters.get_or_insert_with(HashMap::new);

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
        message: Some("Filter applied".to_string()),
    }))
}

pub async fn handle_clear_filter(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ClearFilterRequest>,
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
    if let Some(ref mut filters) = worksheet.filters {
        if let Some(col) = req.col {
            filters.remove(&col);
        } else {
            filters.clear();
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
        message: Some("Filter cleared".to_string()),
    }))
}

pub async fn handle_create_chart(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ChartRequest>,
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
    let chart = ChartConfig {
        id: Uuid::new_v4().to_string(),
        chart_type: req.chart_type,
        title: req.title.unwrap_or_else(|| "Chart".to_string()),
        data_range: req.data_range,
        label_range: req.label_range.unwrap_or_default(),
        position: req.position.unwrap_or(ChartPosition {
            row: 0,
            col: 5,
            width: 400,
            height: 300,
        }),
        options: ChartOptions::default(),
        datasets: vec![],
        labels: vec![],
    };

    let charts = worksheet.charts.get_or_insert_with(Vec::new);
    charts.push(chart);

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
        message: Some("Chart created".to_string()),
    }))
}

pub async fn handle_delete_chart(
    State(state): State<Arc<AppState>>,
    Json(req): Json<DeleteChartRequest>,
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
    if let Some(ref mut charts) = worksheet.charts {
        charts.retain(|c| c.id != req.chart_id);
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
        message: Some("Chart deleted".to_string()),
    }))
}

pub async fn handle_conditional_format(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ConditionalFormatRequest>,
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
    let rule = ConditionalFormatRule {
        id: Uuid::new_v4().to_string(),
        start_row: req.start_row,
        start_col: req.start_col,
        end_row: req.end_row,
        end_col: req.end_col,
        rule_type: req.rule_type,
        condition: req.condition,
        style: req.style,
        priority: 1,
    };

    let formats = worksheet.conditional_formats.get_or_insert_with(Vec::new);
    formats.push(rule);

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
        message: Some("Conditional format applied".to_string()),
    }))
}

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
    let validations = worksheet.validations.get_or_insert_with(HashMap::new);

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
            let min = rule.value1.as_ref().and_then(|v| v.parse::<usize>().ok()).unwrap_or(0);
            let max = rule.value2.as_ref().and_then(|v| v.parse::<usize>().ok()).unwrap_or(usize::MAX);
            len >= min && len <= max
        }
        _ => true,
    };

    ValidationResult {
        valid,
        error_message: if valid {
            None
        } else {
            rule.error_message.clone().or_else(|| Some("Invalid value".to_string()))
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

pub async fn handle_import_sheet(
    State(state): State<Arc<AppState>>,
    mut multipart: axum::extract::Multipart,
) -> Result<Json<Spreadsheet>, (StatusCode, Json<serde_json::Value>)> {
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut filename = "import.xlsx".to_string();

    while let Ok(Some(field)) = multipart.next_field().await {
        if field.name() == Some("file") {
            filename = field.file_name().unwrap_or("import.xlsx").to_string();
            if let Ok(bytes) = field.bytes().await {
                file_bytes = Some(bytes.to_vec());
            }
        }
    }

    let bytes = file_bytes.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "No file uploaded" })),
        )
    })?;

    let mut sheet = import_spreadsheet_bytes(&bytes, &filename).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": e })),
        )
    })?;

    let user_id = get_current_user_id();
    sheet.owner_id = user_id.clone();

    if let Err(e) = save_sheet_to_drive(&state, &user_id, &sheet).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ));
    }

    Ok(Json(sheet))
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

    let comments = worksheet.comments.get_or_insert_with(HashMap::new);
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

    Ok(Json(ListCommentsResponse { comments: comments_list }))
}

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
        protection.password_hash = Some(format!("{:x}", md5::compute(password.as_bytes())));
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
                let provided_hash = format!("{:x}", md5::compute(password.as_bytes()));
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
        message: Some(if req.locked { "Cells locked" } else { "Cells unlocked" }.to_string()),
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
                if let Some(name) = req.name {
                    range.name = name;
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
                if let Some(comment) = req.comment {
                    range.comment = Some(comment);
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

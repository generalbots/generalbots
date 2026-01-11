use crate::shared::state::AppState;
use crate::sheet::storage::{get_current_user_id, load_sheet_by_id, save_sheet_to_drive};
use crate::sheet::types::{
    CellData, ChartConfig, ChartOptions, ChartPosition, ChartRequest, ClearFilterRequest,
    ConditionalFormatRequest, ConditionalFormatRule, DeleteChartRequest, FilterConfig,
    FilterRequest, SaveResponse, SortRequest,
};
use axum::{extract::State, http::StatusCode, Json};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

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

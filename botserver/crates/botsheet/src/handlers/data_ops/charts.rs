//! Chart create/delete mutations.

use crate::auth::{resolve_user_id, SheetUser};
use crate::state::SheetState;
use crate::types::{
    ChartConfig, ChartDataset, ChartOptions, ChartPosition, ChartRequest, DeleteChartRequest,
    SaveResponse,
};
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

pub async fn handle_create_chart(
    State(state): State<Arc<SheetState>>,
    user: Option<Extension<SheetUser>>,
    Json(req): Json<ChartRequest>,
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

    // Resolve the series before mutably borrowing the worksheet; otherwise a
    // toolbar chart is stored with empty datasets/labels and renders blank.
    let values = crate::storage::chart_read::resolve_chart_values(
        &req.data_range,
        req.worksheet_index,
        &sheet.worksheets,
    );
    let labels = req
        .label_range
        .as_deref()
        .map(|range| {
            crate::storage::chart_read::resolve_chart_labels(
                range,
                req.worksheet_index,
                &sheet.worksheets,
            )
        })
        .unwrap_or_default();
    let worksheet = &mut sheet.worksheets[req.worksheet_index];
    let chart = ChartConfig {
        id: Uuid::new_v4().to_string(),
        chart_type: req.chart_type,
        title: req.title.unwrap_or_else(|| "Chart".to_string()),
        data_range: req.data_range,
        label_range: req.label_range.unwrap_or_default(),
        position: req.position.unwrap_or_else(|| {
            let (row, col) = req
                .anchor
                .as_deref()
                .and_then(botsheet_core::formulas::refs::parse_cell_ref)
                .unwrap_or((0, 5));
            ChartPosition {
                row,
                col,
                width: 400,
                height: 300,
            }
        }),
        options: ChartOptions::default(),
        datasets: if values.is_empty() {
            vec![]
        } else {
            vec![ChartDataset {
                label: "Series 1".to_string(),
                data: values,
                color: "#3b82f6".to_string(),
                background_color: None,
            }]
        },
        labels,
    };

    let charts = worksheet.charts.get_or_insert_with(Vec::new);
    charts.push(chart);

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
        message: Some("Chart created".to_string()),
    }))
}

pub async fn handle_delete_chart(
    State(state): State<Arc<SheetState>>,
    user: Option<Extension<SheetUser>>,
    Json(req): Json<DeleteChartRequest>,
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
    if let Some(ref mut charts) = worksheet.charts {
        charts.retain(|c| c.id != req.chart_id);
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
        message: Some("Chart deleted".to_string()),
    }))
}

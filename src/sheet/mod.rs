use crate::shared::state::AppState;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, Query, State,
    },
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use calamine::{open_workbook_auto, Reader, Data};
use chrono::{DateTime, Datelike, Local, NaiveDate, Utc};
use rust_xlsxwriter::{Workbook, Format, Color, FormatAlign, FormatBorder};
use futures_util::{SinkExt, StreamExt};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;
use uuid::Uuid;

type CollaborationChannels =
    Arc<tokio::sync::RwLock<HashMap<String, broadcast::Sender<CollabMessage>>>>;

static COLLAB_CHANNELS: std::sync::OnceLock<CollaborationChannels> = std::sync::OnceLock::new();

fn get_collab_channels() -> &'static CollaborationChannels {
    COLLAB_CHANNELS.get_or_init(|| Arc::new(tokio::sync::RwLock::new(HashMap::new())))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollabMessage {
    pub msg_type: String,
    pub sheet_id: String,
    pub user_id: String,
    pub user_name: String,
    pub user_color: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub row: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub col: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worksheet_index: Option<usize>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collaborator {
    pub id: String,
    pub name: String,
    pub color: String,
    pub cursor_row: Option<u32>,
    pub cursor_col: Option<u32>,
    pub connected_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spreadsheet {
    pub id: String,
    pub name: String,
    pub owner_id: String,
    pub worksheets: Vec<Worksheet>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Worksheet {
    pub name: String,
    pub data: HashMap<String, CellData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column_widths: Option<HashMap<u32, u32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub row_heights: Option<HashMap<u32, u32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frozen_rows: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frozen_cols: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merged_cells: Option<Vec<MergedCell>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<HashMap<u32, FilterConfig>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hidden_rows: Option<Vec<u32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validations: Option<HashMap<String, ValidationRule>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conditional_formats: Option<Vec<ConditionalFormatRule>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub charts: Option<Vec<ChartConfig>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formula: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<CellStyle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CellStyle {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_family: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_size: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_weight: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_style: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_decoration: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_align: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vertical_align: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergedCell {
    pub start_row: u32,
    pub start_col: u32,
    pub end_row: u32,
    pub end_col: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterConfig {
    pub filter_type: String,
    pub values: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value1: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value2: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub validation_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value1: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value2: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_values: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalFormatRule {
    pub id: String,
    pub start_row: u32,
    pub start_col: u32,
    pub end_row: u32,
    pub end_col: u32,
    pub rule_type: String,
    pub condition: String,
    pub style: CellStyle,
    pub priority: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartConfig {
    pub id: String,
    pub chart_type: String,
    pub title: String,
    pub data_range: String,
    pub label_range: Option<String>,
    pub position: ChartPosition,
    pub options: ChartOptions,
    pub datasets: Vec<ChartDataset>,
    pub labels: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartPosition {
    pub row: u32,
    pub col: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartOptions {
    pub show_legend: bool,
    pub show_grid: bool,
    pub stacked: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legend_position: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x_axis_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y_axis_title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartDataset {
    pub label: String,
    pub data: Vec<f64>,
    pub color: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpreadsheetMetadata {
    pub id: String,
    pub name: String,
    pub owner_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub worksheet_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveRequest {
    pub id: Option<String>,
    pub name: String,
    pub worksheets: Vec<Worksheet>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadQuery {
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct LoadFromDriveRequest {
    pub bucket: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellUpdateRequest {
    pub sheet_id: String,
    pub worksheet_index: usize,
    pub row: u32,
    pub col: u32,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatRequest {
    pub sheet_id: String,
    pub worksheet_index: usize,
    pub start_row: u32,
    pub start_col: u32,
    pub end_row: u32,
    pub end_col: u32,
    pub style: CellStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportRequest {
    pub id: String,
    pub format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareRequest {
    pub sheet_id: String,
    pub email: String,
    pub permission: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveResponse {
    pub id: String,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormulaResult {
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormulaRequest {
    pub sheet_id: String,
    pub worksheet_index: usize,
    pub formula: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeCellsRequest {
    pub sheet_id: String,
    pub worksheet_index: usize,
    pub start_row: u32,
    pub start_col: u32,
    pub end_row: u32,
    pub end_col: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreezePanesRequest {
    pub sheet_id: String,
    pub worksheet_index: usize,
    pub frozen_rows: u32,
    pub frozen_cols: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortRequest {
    pub sheet_id: String,
    pub worksheet_index: usize,
    pub start_row: u32,
    pub start_col: u32,
    pub end_row: u32,
    pub end_col: u32,
    pub sort_col: u32,
    pub ascending: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterRequest {
    pub sheet_id: String,
    pub worksheet_index: usize,
    pub col: u32,
    pub filter_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub values: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value1: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value2: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartRequest {
    pub sheet_id: String,
    pub worksheet_index: usize,
    pub chart_type: String,
    pub data_range: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label_range: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<ChartPosition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalFormatRequest {
    pub sheet_id: String,
    pub worksheet_index: usize,
    pub start_row: u32,
    pub start_col: u32,
    pub end_row: u32,
    pub end_col: u32,
    pub rule_type: String,
    pub condition: String,
    pub style: CellStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataValidationRequest {
    pub sheet_id: String,
    pub worksheet_index: usize,
    pub start_row: u32,
    pub start_col: u32,
    pub end_row: u32,
    pub end_col: u32,
    pub validation_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value1: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value2: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_values: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateCellRequest {
    pub sheet_id: String,
    pub worksheet_index: usize,
    pub row: u32,
    pub col: u32,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClearFilterRequest {
    pub sheet_id: String,
    pub worksheet_index: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub col: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteChartRequest {
    pub sheet_id: String,
    pub worksheet_index: usize,
    pub chart_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddNoteRequest {
    pub sheet_id: String,
    pub worksheet_index: usize,
    pub row: u32,
    pub col: u32,
    pub note: String,
}

pub fn configure_sheet_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/sheet/list", get(handle_list_sheets))
        .route("/api/sheet/search", get(handle_search_sheets))
        .route("/api/sheet/load", get(handle_load_sheet))
        .route("/api/sheet/load-from-drive", post(handle_load_from_drive))
        .route("/api/sheet/save", post(handle_save_sheet))
        .route("/api/sheet/delete", post(handle_delete_sheet))
        .route("/api/sheet/cell", post(handle_update_cell))
        .route("/api/sheet/format", post(handle_format_cells))
        .route("/api/sheet/formula", post(handle_evaluate_formula))
        .route("/api/sheet/export", post(handle_export_sheet))
        .route("/api/sheet/share", post(handle_share_sheet))
        .route("/api/sheet/new", get(handle_new_sheet))
        .route("/api/sheet/merge", post(handle_merge_cells))
        .route("/api/sheet/unmerge", post(handle_unmerge_cells))
        .route("/api/sheet/freeze", post(handle_freeze_panes))
        .route("/api/sheet/sort", post(handle_sort_range))
        .route("/api/sheet/filter", post(handle_filter_data))
        .route("/api/sheet/filter/clear", post(handle_clear_filter))
        .route("/api/sheet/chart", post(handle_create_chart))
        .route("/api/sheet/chart/delete", post(handle_delete_chart))
        .route("/api/sheet/conditional-format", post(handle_conditional_format))
        .route("/api/sheet/data-validation", post(handle_data_validation))
        .route("/api/sheet/validate-cell", post(handle_validate_cell))
        .route("/api/sheet/note", post(handle_add_note))
        .route("/api/sheet/import", post(handle_import_sheet))
        .route("/api/sheet/:id", get(handle_get_sheet_by_id))
        .route("/api/sheet/:id/collaborators", get(handle_get_collaborators))
        .route("/ws/sheet/:sheet_id", get(handle_sheet_websocket))
}

fn get_user_sheets_path(user_id: &str) -> String {
    format!("users/{}/sheets", user_id)
}

async fn save_sheet_to_drive(
    state: &Arc<AppState>,
    user_id: &str,
    sheet: &Spreadsheet,
) -> Result<(), String> {
    let drive = state
        .drive
        .as_ref()
        .ok_or_else(|| "Drive not available".to_string())?;

    let path = format!("{}/{}.json", get_user_sheets_path(user_id), sheet.id);
    let content =
        serde_json::to_string_pretty(sheet).map_err(|e| format!("Serialization error: {e}"))?;

    drive
        .put_object()
        .bucket("gbo")
        .key(&path)
        .body(content.into_bytes().into())
        .content_type("application/json")
        .send()
        .await
        .map_err(|e| format!("Failed to save sheet: {e}"))?;

    Ok(())
}

async fn load_sheet_from_drive(
    state: &Arc<AppState>,
    user_id: &str,
    sheet_id: &str,
) -> Result<Spreadsheet, String> {
    let drive = state
        .drive
        .as_ref()
        .ok_or_else(|| "Drive not available".to_string())?;

    let path = format!("{}/{}.json", get_user_sheets_path(user_id), sheet_id);

    let result = drive
        .get_object()
        .bucket("gbo")
        .key(&path)
        .send()
        .await
        .map_err(|e| format!("Failed to load sheet: {e}"))?;

    let bytes = result
        .body
        .collect()
        .await
        .map_err(|e| format!("Failed to read sheet: {e}"))?
        .into_bytes();

    let sheet: Spreadsheet =
        serde_json::from_slice(&bytes).map_err(|e| format!("Failed to parse sheet: {e}"))?;

    Ok(sheet)
}

async fn list_sheets_from_drive(
    state: &Arc<AppState>,
    user_id: &str,
) -> Result<Vec<SpreadsheetMetadata>, String> {
    let drive = state
        .drive
        .as_ref()
        .ok_or_else(|| "Drive not available".to_string())?;

    let prefix = format!("{}/", get_user_sheets_path(user_id));

    let result = drive
        .list_objects_v2()
        .bucket("gbo")
        .prefix(&prefix)
        .send()
        .await
        .map_err(|e| format!("Failed to list sheets: {e}"))?;

    let mut sheets = Vec::new();

    if let Some(contents) = result.contents {
        for obj in contents {
            if let Some(key) = obj.key {
                if key.ends_with(".json") {
                    if let Ok(sheet) =
                        load_sheet_from_drive(state, user_id, &extract_id_from_path(&key)).await
                    {
                        sheets.push(SpreadsheetMetadata {
                            id: sheet.id,
                            name: sheet.name,
                            owner_id: sheet.owner_id,
                            created_at: sheet.created_at,
                            updated_at: sheet.updated_at,
                            worksheet_count: sheet.worksheets.len(),
                        });
                    }
                }
            }
        }
    }

    sheets.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

    Ok(sheets)
}

fn extract_id_from_path(path: &str) -> String {
    path.split('/')
        .last()
        .unwrap_or("")
        .trim_end_matches(".json")
        .to_string()
}

async fn delete_sheet_from_drive(
    state: &Arc<AppState>,
    user_id: &str,
    sheet_id: &str,
) -> Result<(), String> {
    let drive = state
        .drive
        .as_ref()
        .ok_or_else(|| "Drive not available".to_string())?;

    let path = format!("{}/{}.json", get_user_sheets_path(user_id), sheet_id);

    drive
        .delete_object()
        .bucket("gbo")
        .key(&path)
        .send()
        .await
        .map_err(|e| format!("Failed to delete sheet: {e}"))?;

    Ok(())
}

fn get_current_user_id() -> String {
    "default-user".to_string()
}

pub async fn handle_new_sheet(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<Spreadsheet>, (StatusCode, Json<serde_json::Value>)> {
    let sheet = Spreadsheet {
        id: Uuid::new_v4().to_string(),
        name: "Untitled Spreadsheet".to_string(),
        owner_id: get_current_user_id(),
        worksheets: vec![Worksheet {
            name: "Sheet1".to_string(),
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
        }],
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    Ok(Json(sheet))
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
        (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({ "error": "Drive not available" })))
    })?;

    let result = drive
        .get_object()
        .bucket(&req.bucket)
        .key(&req.path)
        .send()
        .await
        .map_err(|e| {
            (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": format!("File not found: {e}") })))
        })?;

    let bytes = result.body.collect().await
        .map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": format!("Failed to read file: {e}") })))
        })?
        .into_bytes();

    let ext = req.path.rsplit('.').next().unwrap_or("").to_lowercase();
    let file_name = req.path.rsplit('/').next().unwrap_or("Spreadsheet");
    let sheet_name = file_name.rsplit('.').last().unwrap_or("Spreadsheet").to_string();

    let worksheets = match ext.as_str() {
        "csv" | "tsv" => {
            let delimiter = if ext == "tsv" { b'\t' } else { b',' };
            parse_csv_to_worksheets(&bytes, delimiter, &sheet_name)?
        }
        "xlsx" | "xls" | "ods" | "xlsb" | "xlsm" => {
            parse_excel_to_worksheets(&bytes, &ext)?
        }
        _ => {
            return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": format!("Unsupported format: .{ext}") }))));
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

fn parse_csv_to_worksheets(
    bytes: &[u8],
    delimiter: u8,
    sheet_name: &str,
) -> Result<Vec<Worksheet>, (StatusCode, Json<serde_json::Value>)> {
    let content = String::from_utf8_lossy(bytes);
    let mut data: HashMap<String, CellData> = HashMap::new();

    for (row_idx, line) in content.lines().enumerate() {
        let cols: Vec<&str> = if delimiter == b'\t' {
            line.split('\t').collect()
        } else {
            line.split(',').collect()
        };

        for (col_idx, value) in cols.iter().enumerate() {
            let clean_value = value.trim().trim_matches('"').to_string();
            if !clean_value.is_empty() {
                let key = format!("{row_idx},{col_idx}");
                data.insert(key, CellData {
                    value: Some(clean_value),
                    formula: None,
                    style: None,
                    format: None,
                    note: None,
                });
            }
        }
    }

    Ok(vec![Worksheet {
        name: sheet_name.to_string(),
        data,
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
    }])
}

fn parse_excel_to_worksheets(
    bytes: &[u8],
    _ext: &str,
) -> Result<Vec<Worksheet>, (StatusCode, Json<serde_json::Value>)> {
    use std::io::Cursor;

    let cursor = Cursor::new(bytes);
    let mut workbook = open_workbook_auto(cursor).map_err(|e| {
        (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": format!("Failed to parse spreadsheet: {e}") })))
    })?;

    let sheet_names: Vec<String> = workbook.sheet_names().to_vec();
    let mut worksheets = Vec::new();

    for sheet_name in sheet_names {
        let range = workbook.worksheet_range(&sheet_name).map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": format!("Failed to read sheet {sheet_name}: {e}") })))
        })?;

        let mut data: HashMap<String, CellData> = HashMap::new();

        for (row_idx, row) in range.rows().enumerate() {
            for (col_idx, cell) in row.iter().enumerate() {
                let value = match cell {
                    Data::Empty => continue,
                    Data::String(s) => s.clone(),
                    Data::Int(i) => i.to_string(),
                    Data::Float(f) => f.to_string(),
                    Data::Bool(b) => b.to_string(),
                    Data::DateTime(dt) => dt.to_string(),
                    Data::Error(e) => format!("#ERR:{e:?}"),
                    Data::DateTimeIso(s) => s.clone(),
                    Data::DurationIso(s) => s.clone(),
                };

                let key = format!("{row_idx},{col_idx}");
                data.insert(key, CellData {
                    value: Some(value),
                    formula: None,
                    style: None,
                    format: None,
                    note: None,
                });
            }
        }

        worksheets.push(Worksheet {
            name: sheet_name,
            data,
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
        });
    }

    if worksheets.is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Spreadsheet has no sheets" }))));
    }

    Ok(worksheets)
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
        id: req.id,
        success: true,
        message: Some("Sheet deleted".to_string()),
    }))
}

pub async fn handle_update_cell(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CellUpdateRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();

    let mut sheet = match load_sheet_from_drive(&state, &user_id, &req.sheet_id).await {
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
        "cellChange",
        &user_id,
        Some(req.row),
        Some(req.col),
        Some(&req.value),
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

    let mut sheet = match load_sheet_from_drive(&state, &user_id, &req.sheet_id).await {
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

    let sheet = match load_sheet_from_drive(&state, &user_id, &req.sheet_id).await {
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

fn evaluate_formula(formula: &str, worksheet: &Worksheet) -> FormulaResult {
    if !formula.starts_with('=') {
        return FormulaResult {
            value: formula.to_string(),
            error: None,
        };
    }

    let expr = formula[1..].to_uppercase();

    let evaluators: Vec<fn(&str, &Worksheet) -> Option<String>> = vec![
        evaluate_sum,
        evaluate_average,
        evaluate_count,
        evaluate_counta,
        evaluate_countblank,
        evaluate_countif,
        evaluate_sumif,
        evaluate_averageif,
        evaluate_max,
        evaluate_min,
        evaluate_if,
        evaluate_iferror,
        evaluate_vlookup,
        evaluate_hlookup,
        evaluate_index_match,
        evaluate_concatenate,
        evaluate_left,
        evaluate_right,
        evaluate_mid,
        evaluate_len,
        evaluate_trim,
        evaluate_upper,
        evaluate_lower,
        evaluate_proper,
        evaluate_substitute,
        evaluate_round,
        evaluate_roundup,
        evaluate_rounddown,
        evaluate_abs,
        evaluate_sqrt,
        evaluate_power,
        evaluate_mod_formula,
        evaluate_and,
        evaluate_or,
        evaluate_not,
        evaluate_today,
        evaluate_now,
        evaluate_date,
        evaluate_year,
        evaluate_month,
        evaluate_day,
        evaluate_datedif,
        evaluate_arithmetic,
    ];

    for evaluator in evaluators {
        if let Some(result) = evaluator(&expr, worksheet) {
            return FormulaResult {
                value: result,
                error: None,
            };
        }
    }

    FormulaResult {
        value: "#ERROR!".to_string(),
        error: Some("Invalid formula".to_string()),
    }
}

fn evaluate_sum(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("SUM(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[4..expr.len() - 1];
    let values = get_range_values(inner, worksheet);
    let sum: f64 = values.iter().sum();
    Some(format_number(sum))
}

fn evaluate_average(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("AVERAGE(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[8..expr.len() - 1];
    let values = get_range_values(inner, worksheet);
    if values.is_empty() {
        return Some("#DIV/0!".to_string());
    }
    let avg = values.iter().sum::<f64>() / values.len() as f64;
    Some(format_number(avg))
}

fn evaluate_count(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("COUNT(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[6..expr.len() - 1];
    let values = get_range_values(inner, worksheet);
    Some(values.len().to_string())
}

fn evaluate_counta(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("COUNTA(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[7..expr.len() - 1];
    let count = get_range_string_values(inner, worksheet)
        .iter()
        .filter(|v| !v.is_empty())
        .count();
    Some(count.to_string())
}

fn evaluate_countblank(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("COUNTBLANK(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[11..expr.len() - 1];
    let (start, end) = parse_range(inner)?;
    let mut count = 0;
    for row in start.0..=end.0 {
        for col in start.1..=end.1 {
            let key = format!("{},{}", row, col);
            let is_blank = worksheet
                .data
                .get(&key)
                .and_then(|c| c.value.as_ref())
                .map(|v| v.is_empty())
                .unwrap_or(true);
            if is_blank {
                count += 1;
            }
        }
    }
    Some(count.to_string())
}

fn evaluate_countif(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("COUNTIF(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[8..expr.len() - 1];
    let parts: Vec<&str> = split_args(inner);
    if parts.len() != 2 {
        return None;
    }
    let range = parts[0].trim();
    let criteria = parts[1].trim().trim_matches('"');
    let values = get_range_string_values(range, worksheet);
    let count = count_matching(&values, criteria);
    Some(count.to_string())
}

fn evaluate_sumif(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("SUMIF(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[6..expr.len() - 1];
    let parts: Vec<&str> = split_args(inner);
    if parts.len() < 2 {
        return None;
    }
    let criteria_range = parts[0].trim();
    let criteria = parts[1].trim().trim_matches('"');
    let sum_range = if parts.len() > 2 {
        parts[2].trim()
    } else {
        criteria_range
    };

    let criteria_values = get_range_string_values(criteria_range, worksheet);
    let sum_values = get_range_values(sum_range, worksheet);

    let mut sum = 0.0;
    for (i, cv) in criteria_values.iter().enumerate() {
        if matches_criteria(cv, criteria) {
            if let Some(sv) = sum_values.get(i) {
                sum += sv;
            }
        }
    }
    Some(format_number(sum))
}

fn evaluate_averageif(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("AVERAGEIF(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[10..expr.len() - 1];
    let parts: Vec<&str> = split_args(inner);
    if parts.len() < 2 {
        return None;
    }
    let criteria_range = parts[0].trim();
    let criteria = parts[1].trim().trim_matches('"');
    let avg_range = if parts.len() > 2 {
        parts[2].trim()
    } else {
        criteria_range
    };

    let criteria_values = get_range_string_values(criteria_range, worksheet);
    let avg_values = get_range_values(avg_range, worksheet);

    let mut sum = 0.0;
    let mut count = 0;
    for (i, cv) in criteria_values.iter().enumerate() {
        if matches_criteria(cv, criteria) {
            if let Some(av) = avg_values.get(i) {
                sum += av;
                count += 1;
            }
        }
    }
    if count == 0 {
        return Some("#DIV/0!".to_string());
    }
    Some(format_number(sum / count as f64))
}

fn evaluate_max(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("MAX(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[4..expr.len() - 1];
    let values = get_range_values(inner, worksheet);
    if values.is_empty() {
        return Some("0".to_string());
    }
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    Some(format_number(max))
}

fn evaluate_min(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("MIN(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[4..expr.len() - 1];
    let values = get_range_values(inner, worksheet);
    if values.is_empty() {
        return Some("0".to_string());
    }
    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    Some(format_number(min))
}

fn evaluate_if(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("IF(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[3..expr.len() - 1];
    let parts: Vec<&str> = split_args(inner);
    if parts.len() < 2 {
        return None;
    }
    let condition = parts[0].trim();
    let true_val = parts[1].trim().trim_matches('"');
    let false_val = if parts.len() > 2 {
        parts[2].trim().trim_matches('"')
    } else {
        "FALSE"
    };

    let result = evaluate_condition(condition, worksheet);
    Some(if result { true_val } else { false_val }.to_string())
}

fn evaluate_iferror(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("IFERROR(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[8..expr.len() - 1];
    let parts: Vec<&str> = split_args(inner);
    if parts.len() != 2 {
        return None;
    }
    let value_expr = parts[0].trim();
    let error_val = parts[1].trim().trim_matches('"');

    let result = evaluate_formula(&format!("={}", value_expr), worksheet);
    if result.value.starts_with('#') {
        Some(error_val.to_string())
    } else {
        Some(result.value)
    }
}

fn evaluate_vlookup(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("VLOOKUP(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[8..expr.len() - 1];
    let parts: Vec<&str> = split_args(inner);
    if parts.len() < 3 {
        return None;
    }

    let search_value = resolve_cell_value(parts[0].trim(), worksheet);
    let range = parts[1].trim();
    let col_index: usize = parts[2].trim().parse().ok()?;
    let exact_match = parts.get(3).map(|v| v.trim() == "FALSE").unwrap_or(true);

    let (start, end) = parse_range(range)?;

    for row in start.0..=end.0 {
        let first_col_key = format!("{},{}", row, start.1);
        let cell_value = worksheet
            .data
            .get(&first_col_key)
            .and_then(|c| c.value.clone())
            .unwrap_or_default();

        let matches = if exact_match {
            cell_value.to_uppercase() == search_value.to_uppercase()
        } else {
            cell_value
                .to_uppercase()
                .starts_with(&search_value.to_uppercase())
        };

        if matches {
            let result_col = start.1 + col_index as u32 - 1;
            if result_col <= end.1 {
                let result_key = format!("{},{}", row, result_col);
                return worksheet
                    .data
                    .get(&result_key)
                    .and_then(|c| c.value.clone())
                    .or(Some(String::new()));
            }
        }
    }
    Some("#N/A".to_string())
}

fn evaluate_hlookup(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("HLOOKUP(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[8..expr.len() - 1];
    let parts: Vec<&str> = split_args(inner);
    if parts.len() < 3 {
        return None;
    }

    let search_value = resolve_cell_value(parts[0].trim(), worksheet);
    let range = parts[1].trim();
    let row_index: usize = parts[2].trim().parse().ok()?;
    let exact_match = parts.get(3).map(|v| v.trim() == "FALSE").unwrap_or(true);

    let (start, end) = parse_range(range)?;

    for col in start.1..=end.1 {
        let first_row_key = format!("{},{}", start.0, col);
        let cell_value = worksheet
            .data
            .get(&first_row_key)
            .and_then(|c| c.value.clone())
            .unwrap_or_default();

        let matches = if exact_match {
            cell_value.to_uppercase() == search_value.to_uppercase()
        } else {
            cell_value
                .to_uppercase()
                .starts_with(&search_value.to_uppercase())
        };

        if matches {
            let result_row = start.0 + row_index as u32 - 1;
            if result_row <= end.0 {
                let result_key = format!("{},{}", result_row, col);
                return worksheet
                    .data
                    .get(&result_key)
                    .and_then(|c| c.value.clone())
                    .or(Some(String::new()));
            }
        }
    }
    Some("#N/A".to_string())
}

fn evaluate_index_match(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("INDEX(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[6..expr.len() - 1];
    let parts: Vec<&str> = split_args(inner);
    if parts.len() < 2 {
        return None;
    }

    let range = parts[0].trim();
    let row_num: u32 = parts[1].trim().parse().ok()?;
    let col_num: u32 = parts.get(2).and_then(|v| v.trim().parse().ok()).unwrap_or(1);

    let (start, _end) = parse_range(range)?;
    let target_row = start.0 + row_num - 1;
    let target_col = start.1 + col_num - 1;
    let key = format!("{},{}", target_row, target_col);

    worksheet
        .data
        .get(&key)
        .and_then(|c| c.value.clone())
        .or(Some(String::new()))
}

fn evaluate_concatenate(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("CONCATENATE(") && !expr.starts_with("CONCAT(") {
        return None;
    }
    let start_idx = if expr.starts_with("CONCATENATE(") {
        12
    } else {
        7
    };
    if !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[start_idx..expr.len() - 1];
    let parts: Vec<&str> = split_args(inner);
    let result: String = parts
        .iter()
        .map(|p| {
            let trimmed = p.trim().trim_matches('"');
            resolve_cell_value(trimmed, worksheet)
        })
        .collect();
    Some(result)
}

fn evaluate_left(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("LEFT(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[5..expr.len() - 1];
    let parts: Vec<&str> = split_args(inner);
    if parts.is_empty() {
        return None;
    }
    let text = resolve_cell_value(parts[0].trim().trim_matches('"'), worksheet);
    let num: usize = parts.get(1).and_then(|v| v.trim().parse().ok()).unwrap_or(1);
    Some(text.chars().take(num).collect())
}

fn evaluate_right(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("RIGHT(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[6..expr.len() - 1];
    let parts: Vec<&str> = split_args(inner);
    if parts.is_empty() {
        return None;
    }
    let text = resolve_cell_value(parts[0].trim().trim_matches('"'), worksheet);
    let num: usize = parts.get(1).and_then(|v| v.trim().parse().ok()).unwrap_or(1);
    let len = text.chars().count();
    Some(text.chars().skip(len.saturating_sub(num)).collect())
}

fn evaluate_mid(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("MID(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[4..expr.len() - 1];
    let parts: Vec<&str> = split_args(inner);
    if parts.len() < 3 {
        return None;
    }
    let text = resolve_cell_value(parts[0].trim().trim_matches('"'), worksheet);
    let start: usize = parts[1].trim().parse().ok()?;
    let num: usize = parts[2].trim().parse().ok()?;
    Some(
        text.chars()
            .skip(start.saturating_sub(1))
            .take(num)
            .collect(),
    )
}

fn evaluate_len(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("LEN(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[4..expr.len() - 1];
    let text = resolve_cell_value(inner.trim().trim_matches('"'), worksheet);
    Some(text.chars().count().to_string())
}

fn evaluate_trim(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("TRIM(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[5..expr.len() - 1];
    let text = resolve_cell_value(inner.trim().trim_matches('"'), worksheet);
    Some(text.split_whitespace().collect::<Vec<_>>().join(" "))
}

fn evaluate_upper(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("UPPER(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[6..expr.len() - 1];
    let text = resolve_cell_value(inner.trim().trim_matches('"'), worksheet);
    Some(text.to_uppercase())
}

fn evaluate_lower(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("LOWER(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[6..expr.len() - 1];
    let text = resolve_cell_value(inner.trim().trim_matches('"'), worksheet);
    Some(text.to_lowercase())
}

fn evaluate_proper(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("PROPER(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[7..expr.len() - 1];
    let text = resolve_cell_value(inner.trim().trim_matches('"'), worksheet);
    Some(
        text.split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    Some(first) => {
                        first.to_uppercase().to_string() + chars.as_str().to_lowercase().as_str()
                    }
                    None => String::new(),
                }
            })
            .collect::<Vec<_>>()
            .join(" "),
    )
}

fn evaluate_substitute(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("SUBSTITUTE(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[11..expr.len() - 1];
    let parts: Vec<&str> = split_args(inner);
    if parts.len() < 3 {
        return None;
    }
    let text = resolve_cell_value(parts[0].trim().trim_matches('"'), worksheet);
    let old_text = parts[1].trim().trim_matches('"');
    let new_text = parts[2].trim().trim_matches('"');
    Some(text.replace(old_text, new_text))
}

fn evaluate_round(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("ROUND(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[6..expr.len() - 1];
    let parts: Vec<&str> = split_args(inner);
    if parts.is_empty() {
        return None;
    }
    let num: f64 = resolve_cell_value(parts[0].trim(), worksheet).parse().ok()?;
    let decimals: i32 = parts.get(1).and_then(|v| v.trim().parse().ok()).unwrap_or(0);
    let factor = 10_f64.powi(decimals);
    Some(format_number((num * factor).round() / factor))
}

fn evaluate_roundup(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("ROUNDUP(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[8..expr.len() - 1];
    let parts: Vec<&str> = split_args(inner);
    if parts.is_empty() {
        return None;
    }
    let num: f64 = resolve_cell_value(parts[0].trim(), worksheet).parse().ok()?;
    let decimals: i32 = parts.get(1).and_then(|v| v.trim().parse().ok()).unwrap_or(0);
    let factor = 10_f64.powi(decimals);
    Some(format_number((num * factor).ceil() / factor))
}

fn evaluate_rounddown(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("ROUNDDOWN(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[10..expr.len() - 1];
    let parts: Vec<&str> = split_args(inner);
    if parts.is_empty() {
        return None;
    }
    let num: f64 = resolve_cell_value(parts[0].trim(), worksheet).parse().ok()?;
    let decimals: i32 = parts.get(1).and_then(|v| v.trim().parse().ok()).unwrap_or(0);
    let factor = 10_f64.powi(decimals);
    Some(format_number((num * factor).floor() / factor))
}

fn evaluate_abs(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("ABS(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[4..expr.len() - 1];
    let num: f64 = resolve_cell_value(inner.trim(), worksheet).parse().ok()?;
    Some(format_number(num.abs()))
}

fn evaluate_sqrt(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("SQRT(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[5..expr.len() - 1];
    let num: f64 = resolve_cell_value(inner.trim(), worksheet).parse().ok()?;
    if num < 0.0 {
        return Some("#NUM!".to_string());
    }
    Some(format_number(num.sqrt()))
}

fn evaluate_power(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("POWER(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[6..expr.len() - 1];
    let parts: Vec<&str> = split_args(inner);
    if parts.len() != 2 {
        return None;
    }
    let base: f64 = resolve_cell_value(parts[0].trim(), worksheet).parse().ok()?;
    let exp: f64 = resolve_cell_value(parts[1].trim(), worksheet).parse().ok()?;
    Some(format_number(base.powf(exp)))
}

fn evaluate_mod_formula(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("MOD(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[4..expr.len() - 1];
    let parts: Vec<&str> = split_args(inner);
    if parts.len() != 2 {
        return None;
    }
    let num: f64 = resolve_cell_value(parts[0].trim(), worksheet).parse().ok()?;
    let divisor: f64 = resolve_cell_value(parts[1].trim(), worksheet).parse().ok()?;
    if divisor == 0.0 {
        return Some("#DIV/0!".to_string());
    }
    Some(format_number(num % divisor))
}

fn evaluate_and(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("AND(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[4..expr.len() - 1];
    let parts: Vec<&str> = split_args(inner);
    let result = parts
        .iter()
        .all(|p| evaluate_condition(p.trim(), worksheet));
    Some(if result { "TRUE" } else { "FALSE" }.to_string())
}

fn evaluate_or(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("OR(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[3..expr.len() - 1];
    let parts: Vec<&str> = split_args(inner);
    let result = parts
        .iter()
        .any(|p| evaluate_condition(p.trim(), worksheet));
    Some(if result { "TRUE" } else { "FALSE" }.to_string())
}

fn evaluate_not(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("NOT(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[4..expr.len() - 1];
    let result = !evaluate_condition(inner.trim(), worksheet);
    Some(if result { "TRUE" } else { "FALSE" }.to_string())
}

fn evaluate_today(_expr: &str, _worksheet: &Worksheet) -> Option<String> {
    if _expr != "TODAY()" {
        return None;
    }
    let today = Local::now().format("%Y-%m-%d").to_string();
    Some(today)
}

fn evaluate_now(_expr: &str, _worksheet: &Worksheet) -> Option<String> {
    if _expr != "NOW()" {
        return None;
    }
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    Some(now)
}

fn evaluate_date(expr: &str, _worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("DATE(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[5..expr.len() - 1];
    let parts: Vec<&str> = split_args(inner);
    if parts.len() != 3 {
        return None;
    }
    let year: i32 = parts[0].trim().parse().ok()?;
    let month: u32 = parts[1].trim().parse().ok()?;
    let day: u32 = parts[2].trim().parse().ok()?;
    let date = NaiveDate::from_ymd_opt(year, month, day)?;
    Some(date.format("%Y-%m-%d").to_string())
}

fn evaluate_year(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("YEAR(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[5..expr.len() - 1];
    let date_str = resolve_cell_value(inner.trim().trim_matches('"'), worksheet);
    let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").ok()?;
    Some(date.year().to_string())
}

fn evaluate_month(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("MONTH(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[6..expr.len() - 1];
    let date_str = resolve_cell_value(inner.trim().trim_matches('"'), worksheet);
    let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").ok()?;
    Some(date.month().to_string())
}

fn evaluate_day(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("DAY(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[4..expr.len() - 1];
    let date_str = resolve_cell_value(inner.trim().trim_matches('"'), worksheet);
    let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").ok()?;
    Some(date.day().to_string())
}

fn evaluate_datedif(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("DATEDIF(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[8..expr.len() - 1];
    let parts: Vec<&str> = split_args(inner);
    if parts.len() != 3 {
        return None;
    }
    let start_str = resolve_cell_value(parts[0].trim().trim_matches('"'), worksheet);
    let end_str = resolve_cell_value(parts[1].trim().trim_matches('"'), worksheet);
    let unit = parts[2].trim().trim_matches('"').to_uppercase();

    let start = NaiveDate::parse_from_str(&start_str, "%Y-%m-%d").ok()?;
    let end = NaiveDate::parse_from_str(&end_str, "%Y-%m-%d").ok()?;

    let diff = match unit.as_str() {
        "D" => (end - start).num_days(),
        "M" => {
            let months = (end.year() - start.year()) * 12 + (end.month() as i32 - start.month() as i32);
            months as i64
        }
        "Y" => (end.year() - start.year()) as i64,
        _ => return Some("#VALUE!".to_string()),
    };
    Some(diff.to_string())
}

fn evaluate_arithmetic(expr: &str, worksheet: &Worksheet) -> Option<String> {
    let resolved = resolve_cell_references(expr, worksheet);
    eval_simple_arithmetic(&resolved).map(format_number)
}

fn resolve_cell_references(expr: &str, worksheet: &Worksheet) -> String {
    let mut result = expr.to_string();
    let re = regex::Regex::new(r"([A-Z]+)(\d+)").ok();

    if let Some(regex) = re {
        for cap in regex.captures_iter(expr) {
            if let (Some(col_match), Some(row_match)) = (cap.get(1), cap.get(2)) {
                let col = col_name_to_index(col_match.as_str());
                let row: u32 = row_match.as_str().parse().unwrap_or(1) - 1;
                let key = format!("{},{}", row, col);

                let value = worksheet
                    .data
                    .get(&key)
                    .and_then(|c| c.value.clone())
                    .unwrap_or_else(|| "0".to_string());

                let cell_ref = format!("{}{}", col_match.as_str(), row_match.as_str());
                result = result.replace(&cell_ref, &value);
            }
        }
    }
    result
}

fn eval_simple_arithmetic(expr: &str) -> Option<f64> {
    let expr = expr.replace(' ', "");
    if let Ok(num) = expr.parse::<f64>() {
        return Some(num);
    }
    if let Some(pos) = expr.rfind('+') {
        if pos > 0 {
            let left = eval_simple_arithmetic(&expr[..pos])?;
            let right = eval_simple_arithmetic(&expr[pos + 1..])?;
            return Some(left + right);
        }
    }
    if let Some(pos) = expr.rfind('-') {
        if pos > 0 {
            let left = eval_simple_arithmetic(&expr[..pos])?;
            let right = eval_simple_arithmetic(&expr[pos + 1..])?;
            return Some(left - right);
        }
    }
    if let Some(pos) = expr.rfind('*') {
        let left = eval_simple_arithmetic(&expr[..pos])?;
        let right = eval_simple_arithmetic(&expr[pos + 1..])?;
        return Some(left * right);
    }
    if let Some(pos) = expr.rfind('/') {
        let left = eval_simple_arithmetic(&expr[..pos])?;
        let right = eval_simple_arithmetic(&expr[pos + 1..])?;
        if right != 0.0 {
            return Some(left / right);
        }
    }
    None
}

fn get_range_values(range: &str, worksheet: &Worksheet) -> Vec<f64> {
    let parts: Vec<&str> = range.split(':').collect();
    if parts.len() != 2 {
        if let Some(val) = resolve_cell_value(range.trim(), worksheet).parse::<f64>().ok() {
            return vec![val];
        }
        return Vec::new();
    }
    let (start, end) = match parse_range(range) {
        Some(r) => r,
        None => return Vec::new(),
    };
    let mut values = Vec::new();
    for row in start.0..=end.0 {
        for col in start.1..=end.1 {
            let key = format!("{},{}", row, col);
            if let Some(cell) = worksheet.data.get(&key) {
                if let Some(ref value) = cell.value {
                    if let Ok(num) = value.parse::<f64>() {
                        values.push(num);
                    }
                }
            }
        }
    }
    values
}

fn get_range_string_values(range: &str, worksheet: &Worksheet) -> Vec<String> {
    let (start, end) = match parse_range(range) {
        Some(r) => r,
        None => return Vec::new(),
    };
    let mut values = Vec::new();
    for row in start.0..=end.0 {
        for col in start.1..=end.1 {
            let key = format!("{},{}", row, col);
            let value = worksheet
                .data
                .get(&key)
                .and_then(|c| c.value.clone())
                .unwrap_or_default();
            values.push(value);
        }
    }
    values
}

fn parse_range(range: &str) -> Option<((u32, u32), (u32, u32))> {
    let parts: Vec<&str> = range.split(':').collect();
    if parts.len() != 2 {
        return None;
    }
    let start = parse_cell_ref(parts[0].trim())?;
    let end = parse_cell_ref(parts[1].trim())?;
    Some((start, end))
}

fn parse_cell_ref(cell_ref: &str) -> Option<(u32, u32)> {
    let cell_ref = cell_ref.trim().to_uppercase();
    let mut col_str = String::new();
    let mut row_str = String::new();
    for ch in cell_ref.chars() {
        if ch.is_ascii_alphabetic() {
            col_str.push(ch);
        } else if ch.is_ascii_digit() {
            row_str.push(ch);
        }
    }
    if col_str.is_empty() || row_str.is_empty() {
        return None;
    }
    let col = col_name_to_index(&col_str);
    let row: u32 = row_str.parse::<u32>().ok()? - 1;
    Some((row, col))
}

fn col_name_to_index(name: &str) -> u32 {
    let mut col: u32 = 0;
    for ch in name.chars() {
        col = col * 26 + (ch as u32 - 'A' as u32 + 1);
    }
    col - 1
}

fn format_number(num: f64) -> String {
    if num.fract() == 0.0 {
        format!("{}", num as i64)
    } else {
        format!("{:.6}", num).trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

fn resolve_cell_value(value: &str, worksheet: &Worksheet) -> String {
    if let Some((row, col)) = parse_cell_ref(value) {
        let key = format!("{},{}", row, col);
        worksheet
            .data
            .get(&key)
            .and_then(|c| c.value.clone())
            .unwrap_or_default()
    } else {
        value.to_string()
    }
}

fn split_args(s: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0;
    let mut start = 0;
    for (i, ch) in s.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => {
                parts.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    if start < s.len() {
        parts.push(&s[start..]);
    }
    parts
}

fn evaluate_condition(condition: &str, worksheet: &Worksheet) -> bool {
    let condition = condition.trim();
    if condition.eq_ignore_ascii_case("TRUE") {
        return true;
    }
    if condition.eq_ignore_ascii_case("FALSE") {
        return false;
    }

    let ops = [">=", "<=", "<>", "!=", "=", ">", "<"];
    for op in ops {
        if let Some(pos) = condition.find(op) {
            let left = resolve_cell_value(&condition[..pos].trim(), worksheet);
            let right = resolve_cell_value(&condition[pos + op.len()..].trim().trim_matches('"'), worksheet);

            let left_num = left.parse::<f64>().ok();
            let right_num = right.parse::<f64>().ok();

            return match (op, left_num, right_num) {
                (">=", Some(l), Some(r)) => l >= r,
                ("<=", Some(l), Some(r)) => l <= r,
                ("<>", _, _) | ("!=", _, _) => left != right,
                ("=", _, _) => left == right,
                (">", Some(l), Some(r)) => l > r,
                ("<", Some(l), Some(r)) => l < r,
                _ => false,
            };
        }
    }

    let val = resolve_cell_value(condition, worksheet);
    !val.is_empty() && val != "0" && !val.eq_ignore_ascii_case("FALSE")
}

fn matches_criteria(value: &str, criteria: &str) -> bool {
    if criteria.starts_with(">=") {
        if let (Ok(v), Ok(c)) = (value.parse::<f64>(), criteria[2..].parse::<f64>()) {
            return v >= c;
        }
    } else if criteria.starts_with("<=") {
        if let (Ok(v), Ok(c)) = (value.parse::<f64>(), criteria[2..].parse::<f64>()) {
            return v <= c;
        }
    } else if criteria.starts_with("<>") || criteria.starts_with("!=") {
        return value != &criteria[2..];
    } else if criteria.starts_with('>') {
        if let (Ok(v), Ok(c)) = (value.parse::<f64>(), criteria[1..].parse::<f64>()) {
            return v > c;
        }
    } else if criteria.starts_with('<') {
        if let (Ok(v), Ok(c)) = (value.parse::<f64>(), criteria[1..].parse::<f64>()) {
            return v < c;
        }
    } else if criteria.starts_with('=') {
        return value == &criteria[1..];
    } else if criteria.contains('*') || criteria.contains('?') {
        let pattern = criteria.replace('*', ".*").replace('?', ".");
        if let Ok(re) = regex::Regex::new(&format!("^{}$", pattern)) {
            return re.is_match(value);
        }
    }
    value.eq_ignore_ascii_case(criteria)
}

fn count_matching(values: &[String], criteria: &str) -> usize {
    values.iter().filter(|v| matches_criteria(v, criteria)).count()
}

pub async fn handle_export_sheet(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ExportRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let sheet = match load_sheet_from_drive(&state, &user_id, &req.id).await {
        Ok(s) => s,
        Err(e) => return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": e })))),
    };

    match req.format.as_str() {
        "csv" => {
            let csv = export_to_csv(&sheet);
            Ok(([(axum::http::header::CONTENT_TYPE, "text/csv")], csv))
        }
        "xlsx" => {
            let xlsx = export_to_xlsx(&sheet).map_err(|e| {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e })))
            })?;
            Ok(([(axum::http::header::CONTENT_TYPE, "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")], xlsx))
        }
        "json" => {
            let json = serde_json::to_string_pretty(&sheet).unwrap_or_default();
            Ok(([(axum::http::header::CONTENT_TYPE, "application/json")], json))
        }
        _ => Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Unsupported format" })))),
    }
}

fn export_to_xlsx(sheet: &Spreadsheet) -> Result<String, String> {
    let mut workbook = Workbook::new();

    for ws in &sheet.worksheets {
        let worksheet = workbook.add_worksheet();
        worksheet.set_name(&ws.name).map_err(|e| e.to_string())?;

        let mut max_row: u32 = 0;
        let mut max_col: u16 = 0;

        for key in ws.data.keys() {
            let parts: Vec<&str> = key.split(',').collect();
            if parts.len() == 2 {
                if let (Ok(row), Ok(col)) = (parts[0].parse::<u32>(), parts[1].parse::<u16>()) {
                    max_row = max_row.max(row);
                    max_col = max_col.max(col);
                }
            }
        }

        for (key, cell) in &ws.data {
            let parts: Vec<&str> = key.split(',').collect();
            if parts.len() != 2 {
                continue;
            }
            let (row, col) = match (parts[0].parse::<u32>(), parts[1].parse::<u16>()) {
                (Ok(r), Ok(c)) => (r, c),
                _ => continue,
            };

            let value = cell.value.as_deref().unwrap_or("");

            let mut format = Format::new();

            if let Some(ref style) = cell.style {
                if let Some(ref bg) = style.background {
                    if let Some(color) = parse_color(bg) {
                        format = format.set_background_color(color);
                    }
                }
                if let Some(ref fg) = style.color {
                    if let Some(color) = parse_color(fg) {
                        format = format.set_font_color(color);
                    }
                }
                if let Some(ref weight) = style.font_weight {
                    if weight == "bold" {
                        format = format.set_bold();
                    }
                }
                if let Some(ref style_val) = style.font_style {
                    if style_val == "italic" {
                        format = format.set_italic();
                    }
                }
                if let Some(ref align) = style.text_align {
                    format = match align.as_str() {
                        "center" => format.set_align(FormatAlign::Center),
                        "right" => format.set_align(FormatAlign::Right),
                        _ => format.set_align(FormatAlign::Left),
                    };
                }
                if let Some(ref size) = style.font_size {
                    format = format.set_font_size(*size as f64);
                }
            }

            if let Some(ref formula) = cell.formula {
                worksheet.write_formula_with_format(row, col, formula, &format)
                    .map_err(|e| e.to_string())?;
            } else if let Ok(num) = value.parse::<f64>() {
                worksheet.write_number_with_format(row, col, num, &format)
                    .map_err(|e| e.to_string())?;
            } else {
                worksheet.write_string_with_format(row, col, value, &format)
                    .map_err(|e| e.to_string())?;
            }
        }

        if let Some(ref widths) = ws.column_widths {
            for (col_str, width) in widths {
                if let Ok(col) = col_str.parse::<u16>() {
                    worksheet.set_column_width(col, *width).map_err(|e| e.to_string())?;
                }
            }
        }

        if let Some(ref heights) = ws.row_heights {
            for (row_str, height) in heights {
                if let Ok(row) = row_str.parse::<u32>() {
                    worksheet.set_row_height(row, *height).map_err(|e| e.to_string())?;
                }
            }
        }

        if let Some(frozen_rows) = ws.frozen_rows {
            if let Some(frozen_cols) = ws.frozen_cols {
                worksheet.set_freeze_panes(frozen_rows, frozen_cols as u16)
                    .map_err(|e| e.to_string())?;
            }
        }
    }

    let buffer = workbook.save_to_buffer().map_err(|e| e.to_string())?;
    Ok(base64::engine::general_purpose::STANDARD.encode(&buffer))
}

fn parse_color(color_str: &str) -> Option<Color> {
    let hex = color_str.trim_start_matches('#');
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some(Color::RGB(((r as u32) << 16) | ((g as u32) << 8) | (b as u32)))
    } else {
        None
    }
}

fn export_to_csv(sheet: &Spreadsheet) -> String {
    let mut csv = String::new();
    if let Some(worksheet) = sheet.worksheets.first() {
        let mut max_row: u32 = 0;
        let mut max_col: u32 = 0;
        for key in worksheet.data.keys() {
            let parts: Vec<&str> = key.split(',').collect();
            if parts.len() == 2 {
                if let (Ok(row), Ok(col)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                    max_row = max_row.max(row);
                    max_col = max_col.max(col);
                }
            }
        }
        for row in 0..=max_row {
            let mut row_values = Vec::new();
            for col in 0..=max_col {
                let key = format!("{},{}", row, col);
                let value = worksheet.data.get(&key).and_then(|c| c.value.clone()).unwrap_or_default();
                let escaped = if value.contains(',') || value.contains('"') || value.contains('\n') {
                    format!("\"{}\"", value.replace('"', "\"\""))
                } else {
                    value
                };
                row_values.push(escaped);
            }
            csv.push_str(&row_values.join(","));
            csv.push('\n');
        }
    }
    csv
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
    match load_sheet_from_drive(&state, &user_id, &sheet_id).await {
        Ok(sheet) => Ok(Json(sheet)),
        Err(e) => Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": e })))),
    }
}

pub async fn handle_merge_cells(
    State(state): State<Arc<AppState>>,
    Json(req): Json<MergeCellsRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut sheet = match load_sheet_from_drive(&state, &user_id, &req.sheet_id).await {
        Ok(s) => s,
        Err(e) => return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": e })))),
    };

    if req.worksheet_index >= sheet.worksheets.len() {
        return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Invalid worksheet index" }))));
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
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e }))));
    }

    broadcast_sheet_change(&req.sheet_id, "merge", &user_id, None, None, None).await;
    Ok(Json(SaveResponse { id: req.sheet_id, success: true, message: Some("Cells merged".to_string()) }))
}

pub async fn handle_unmerge_cells(
    State(state): State<Arc<AppState>>,
    Json(req): Json<MergeCellsRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut sheet = match load_sheet_from_drive(&state, &user_id, &req.sheet_id).await {
        Ok(s) => s,
        Err(e) => return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": e })))),
    };

    if req.worksheet_index >= sheet.worksheets.len() {
        return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Invalid worksheet index" }))));
    }

    let worksheet = &mut sheet.worksheets[req.worksheet_index];
    if let Some(ref mut merged_cells) = worksheet.merged_cells {
        merged_cells.retain(|m| {
            !(m.start_row == req.start_row && m.start_col == req.start_col &&
              m.end_row == req.end_row && m.end_col == req.end_col)
        });
    }

    sheet.updated_at = Utc::now();
    if let Err(e) = save_sheet_to_drive(&state, &user_id, &sheet).await {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e }))));
    }

    Ok(Json(SaveResponse { id: req.sheet_id, success: true, message: Some("Cells unmerged".to_string()) }))
}

pub async fn handle_freeze_panes(
    State(state): State<Arc<AppState>>,
    Json(req): Json<FreezePanesRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut sheet = match load_sheet_from_drive(&state, &user_id, &req.sheet_id).await {
        Ok(s) => s,
        Err(e) => return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": e })))),
    };

    if req.worksheet_index >= sheet.worksheets.len() {
        return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Invalid worksheet index" }))));
    }

    let worksheet = &mut sheet.worksheets[req.worksheet_index];
    worksheet.frozen_rows = if req.frozen_rows > 0 { Some(req.frozen_rows) } else { None };
    worksheet.frozen_cols = if req.frozen_cols > 0 { Some(req.frozen_cols) } else { None };

    sheet.updated_at = Utc::now();
    if let Err(e) = save_sheet_to_drive(&state, &user_id, &sheet).await {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e }))));
    }

    Ok(Json(SaveResponse { id: req.sheet_id, success: true, message: Some("Freeze panes updated".to_string()) }))
}

pub async fn handle_sort_range(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SortRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut sheet = match load_sheet_from_drive(&state, &user_id, &req.sheet_id).await {
        Ok(s) => s,
        Err(e) => return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": e })))),
    };

    if req.worksheet_index >= sheet.worksheets.len() {
        return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Invalid worksheet index" }))));
    }

    let worksheet = &mut sheet.worksheets[req.worksheet_index];
    let mut rows_data: Vec<(u32, HashMap<u32, CellData>)> = Vec::new();

    for row in req.start_row..=req.end_row {
        let mut row_data = HashMap::new();
        for col in req.start_col..=req.end_col {
            let key = format!("{},{}", row, col);
            if let Some(cell) = worksheet.data.get(&key) {
                row_data.insert(col, cell.clone());
            }
        }
        rows_data.push((row, row_data));
    }

    rows_data.sort_by(|a, b| {
        let val_a = a.1.get(&req.sort_col).and_then(|c| c.value.clone()).unwrap_or_default();
        let val_b = b.1.get(&req.sort_col).and_then(|c| c.value.clone()).unwrap_or_default();
        let num_a = val_a.parse::<f64>().ok();
        let num_b = val_b.parse::<f64>().ok();
        let cmp = match (num_a, num_b) {
            (Some(a), Some(b)) => a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal),
            _ => val_a.cmp(&val_b),
        };
        if req.ascending { cmp } else { cmp.reverse() }
    });

    for (idx, (_, row_data)) in rows_data.into_iter().enumerate() {
        let target_row = req.start_row + idx as u32;
        for col in req.start_col..=req.end_col {
            let key = format!("{},{}", target_row, col);
            if let Some(cell) = row_data.get(&col) {
                worksheet.data.insert(key, cell.clone());
            } else {
                worksheet.data.remove(&key);
            }
        }
    }

    sheet.updated_at = Utc::now();
    if let Err(e) = save_sheet_to_drive(&state, &user_id, &sheet).await {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e }))));
    }

    broadcast_sheet_change(&req.sheet_id, "sort", &user_id, None, None, None).await;
    Ok(Json(SaveResponse { id: req.sheet_id, success: true, message: Some("Range sorted".to_string()) }))
}

pub async fn handle_filter_data(
    State(state): State<Arc<AppState>>,
    Json(req): Json<FilterRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut sheet = match load_sheet_from_drive(&state, &user_id, &req.sheet_id).await {
        Ok(s) => s,
        Err(e) => return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": e })))),
    };

    if req.worksheet_index >= sheet.worksheets.len() {
        return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Invalid worksheet index" }))));
    }

    let worksheet = &mut sheet.worksheets[req.worksheet_index];
    let filters = worksheet.filters.get_or_insert_with(HashMap::new);

    filters.insert(req.col, FilterConfig {
        filter_type: req.filter_type.clone(),
        values: req.values.clone().unwrap_or_default(),
        condition: req.condition.clone(),
        value1: req.value1.clone(),
        value2: req.value2.clone(),
    });

    let mut hidden_rows = Vec::new();
    let mut max_row = 0u32;
    for key in worksheet.data.keys() {
        if let Some(row) = key.split(',').next().and_then(|r| r.parse::<u32>().ok()) {
            max_row = max_row.max(row);
        }
    }

    for row in 0..=max_row {
        let key = format!("{},{}", row, req.col);
        let cell_value = worksheet.data.get(&key).and_then(|c| c.value.clone()).unwrap_or_default();

        let should_hide = match req.filter_type.as_str() {
            "values" => {
                let values = req.values.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);
                !values.is_empty() && !values.iter().any(|v| v == &cell_value)
            }
            "greaterThan" => {
                if let (Ok(cv), Some(Ok(v1))) = (cell_value.parse::<f64>(), req.value1.as_ref().map(|v| v.parse::<f64>())) {
                    cv <= v1
                } else { false }
            }
            "lessThan" => {
                if let (Ok(cv), Some(Ok(v1))) = (cell_value.parse::<f64>(), req.value1.as_ref().map(|v| v.parse::<f64>())) {
                    cv >= v1
                } else { false }
            }
            "between" => {
                if let (Ok(cv), Some(Ok(v1)), Some(Ok(v2))) = (
                    cell_value.parse::<f64>(),
                    req.value1.as_ref().map(|v| v.parse::<f64>()),
                    req.value2.as_ref().map(|v| v.parse::<f64>())
                ) {
                    cv < v1 || cv > v2
                } else { false }
            }
            "contains" => {
                if let Some(ref v1) = req.value1 {
                    !cell_value.to_lowercase().contains(&v1.to_lowercase())
                } else { false }
            }
            "notContains" => {
                if let Some(ref v1) = req.value1 {
                    cell_value.to_lowercase().contains(&v1.to_lowercase())
                } else { false }
            }
            "isEmpty" => !cell_value.is_empty(),
            "isNotEmpty" => cell_value.is_empty(),
            _ => false,
        };

        if should_hide {
            hidden_rows.push(row);
        }
    }

    worksheet.hidden_rows = if hidden_rows.is_empty() { None } else { Some(hidden_rows.clone()) };
    sheet.updated_at = Utc::now();

    if let Err(e) = save_sheet_to_drive(&state, &user_id, &sheet).await {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e }))));
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "sheet_id": req.sheet_id,
        "hidden_rows": hidden_rows
    })))
}

pub async fn handle_clear_filter(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ClearFilterRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut sheet = match load_sheet_from_drive(&state, &user_id, &req.sheet_id).await {
        Ok(s) => s,
        Err(e) => return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": e })))),
    };

    if req.worksheet_index >= sheet.worksheets.len() {
        return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Invalid worksheet index" }))));
    }

    let worksheet = &mut sheet.worksheets[req.worksheet_index];

    if let Some(col) = req.col {
        if let Some(ref mut filters) = worksheet.filters {
            filters.remove(&col);
        }
    } else {
        worksheet.filters = None;
    }
    worksheet.hidden_rows = None;

    sheet.updated_at = Utc::now();
    if let Err(e) = save_sheet_to_drive(&state, &user_id, &sheet).await {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e }))));
    }

    Ok(Json(SaveResponse { id: req.sheet_id, success: true, message: Some("Filter cleared".to_string()) }))
}

pub async fn handle_create_chart(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ChartRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut sheet = match load_sheet_from_drive(&state, &user_id, &req.sheet_id).await {
        Ok(s) => s,
        Err(e) => return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": e })))),
    };

    if req.worksheet_index >= sheet.worksheets.len() {
        return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Invalid worksheet index" }))));
    }

    let worksheet = &sheet.worksheets[req.worksheet_index];
    let chart_id = Uuid::new_v4().to_string();

    let (start, end) = match parse_range(&req.data_range) {
        Some(r) => r,
        None => return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Invalid data range" })))),
    };

    let mut labels = Vec::new();
    let mut datasets = Vec::new();
    let colors = ["#3b82f6", "#ef4444", "#22c55e", "#f59e0b", "#8b5cf6", "#ec4899", "#14b8a6"];

    if let Some(ref label_range) = req.label_range {
        if let Some((ls, le)) = parse_range(label_range) {
            for row in ls.0..=le.0 {
                for col in ls.1..=le.1 {
                    let key = format!("{},{}", row, col);
                    let val = worksheet.data.get(&key).and_then(|c| c.value.clone()).unwrap_or_default();
                    labels.push(val);
                }
            }
        }
    } else {
        for row in start.0..=end.0 {
            let key = format!("{},{}", row, start.1);
            let val = worksheet.data.get(&key).and_then(|c| c.value.clone()).unwrap_or_else(|| format!("Row {}", row + 1));
            labels.push(val);
        }
    }

    for (col_idx, col) in (start.1..=end.1).enumerate() {
        let mut data = Vec::new();
        for row in start.0..=end.0 {
            let key = format!("{},{}", row, col);
            let val = worksheet.data.get(&key).and_then(|c| c.value.clone()).unwrap_or_default();
            data.push(val.parse::<f64>().unwrap_or(0.0));
        }
        datasets.push(ChartDataset {
            label: format!("Series {}", col_idx + 1),
            data,
            color: colors[col_idx % colors.len()].to_string(),
            background_color: Some(colors[col_idx % colors.len()].to_string()),
        });
    }

    let chart = ChartConfig {
        id: chart_id.clone(),
        chart_type: req.chart_type.clone(),
        title: req.title.clone().unwrap_or_else(|| "Chart".to_string()),
        data_range: req.data_range.clone(),
        label_range: req.label_range.clone(),
        position: req.position.clone().unwrap_or(ChartPosition { row: 0, col: end.1 + 2, width: 400, height: 300 }),
        options: ChartOptions {
            show_legend: true,
            show_grid: true,
            stacked: false,
            legend_position: Some("bottom".to_string()),
            x_axis_title: None,
            y_axis_title: None,
        },
        datasets: datasets.clone(),
        labels: labels.clone(),
    };

    let worksheet_mut = &mut sheet.worksheets[req.worksheet_index];
    let charts = worksheet_mut.charts.get_or_insert_with(Vec::new);
    charts.push(chart);

    sheet.updated_at = Utc::now();
    if let Err(e) = save_sheet_to_drive(&state, &user_id, &sheet).await {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e }))));
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "chart_id": chart_id,
        "chart_type": req.chart_type,
        "labels": labels,
        "datasets": datasets
    })))
}

pub async fn handle_delete_chart(
    State(state): State<Arc<AppState>>,
    Json(req): Json<DeleteChartRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut sheet = match load_sheet_from_drive(&state, &user_id, &req.sheet_id).await {
        Ok(s) => s,
        Err(e) => return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": e })))),
    };

    if req.worksheet_index >= sheet.worksheets.len() {
        return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Invalid worksheet index" }))));
    }

    let worksheet = &mut sheet.worksheets[req.worksheet_index];
    if let Some(ref mut charts) = worksheet.charts {
        charts.retain(|c| c.id != req.chart_id);
    }

    sheet.updated_at = Utc::now();
    if let Err(e) = save_sheet_to_drive(&state, &user_id, &sheet).await {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e }))));
    }

    Ok(Json(SaveResponse { id: req.sheet_id, success: true, message: Some("Chart deleted".to_string()) }))
}

pub async fn handle_conditional_format(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ConditionalFormatRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut sheet = match load_sheet_from_drive(&state, &user_id, &req.sheet_id).await {
        Ok(s) => s,
        Err(e) => return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": e })))),
    };

    if req.worksheet_index >= sheet.worksheets.len() {
        return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Invalid worksheet index" }))));
    }

    let worksheet = &mut sheet.worksheets[req.worksheet_index];
    let rule = ConditionalFormatRule {
        id: Uuid::new_v4().to_string(),
        start_row: req.start_row,
        start_col: req.start_col,
        end_row: req.end_row,
        end_col: req.end_col,
        rule_type: req.rule_type.clone(),
        condition: req.condition.clone(),
        style: req.style.clone(),
        priority: 0,
    };

    let rules = worksheet.conditional_formats.get_or_insert_with(Vec::new);
    rules.push(rule);

    for row in req.start_row..=req.end_row {
        for col in req.start_col..=req.end_col {
            let key = format!("{},{}", row, col);
            let cell_value = worksheet.data.get(&key).and_then(|c| c.value.clone()).unwrap_or_default();

            let should_apply = match req.rule_type.as_str() {
                "greaterThan" => {
                    if let (Ok(val), Ok(cond)) = (cell_value.parse::<f64>(), req.condition.parse::<f64>()) {
                        val > cond
                    } else { false }
                }
                "lessThan" => {
                    if let (Ok(val), Ok(cond)) = (cell_value.parse::<f64>(), req.condition.parse::<f64>()) {
                        val < cond
                    } else { false }
                }
                "equals" => cell_value == req.condition,
                "notEquals" => cell_value != req.condition,
                "contains" => cell_value.to_lowercase().contains(&req.condition.to_lowercase()),
                "notContains" => !cell_value.to_lowercase().contains(&req.condition.to_lowercase()),
                "isEmpty" => cell_value.is_empty(),
                "isNotEmpty" => !cell_value.is_empty(),
                "between" => {
                    let parts: Vec<&str> = req.condition.split(',').collect();
                    if parts.len() == 2 {
                        if let (Ok(val), Ok(min), Ok(max)) = (cell_value.parse::<f64>(), parts[0].trim().parse::<f64>(), parts[1].trim().parse::<f64>()) {
                            val >= min && val <= max
                        } else { false }
                    } else { false }
                }
                _ => false,
            };

            if should_apply {
                let cell = worksheet.data.entry(key).or_insert_with(|| CellData {
                    value: None, formula: None, style: None, format: None, note: None,
                });
                cell.style = Some(req.style.clone());
            }
        }
    }

    sheet.updated_at = Utc::now();
    if let Err(e) = save_sheet_to_drive(&state, &user_id, &sheet).await {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e }))));
    }

    Ok(Json(SaveResponse { id: req.sheet_id, success: true, message: Some("Conditional formatting applied".to_string()) }))
}

pub async fn handle_data_validation(
    State(state): State<Arc<AppState>>,
    Json(req): Json<DataValidationRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut sheet = match load_sheet_from_drive(&state, &user_id, &req.sheet_id).await {
        Ok(s) => s,
        Err(e) => return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": e })))),
    };

    if req.worksheet_index >= sheet.worksheets.len() {
        return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Invalid worksheet index" }))));
    }

    let worksheet = &mut sheet.worksheets[req.worksheet_index];
    let validations = worksheet.validations.get_or_insert_with(HashMap::new);

    let rule = ValidationRule {
        validation_type: req.validation_type.clone(),
        operator: req.operator.clone(),
        value1: req.value1.clone(),
        value2: req.value2.clone(),
        allowed_values: req.allowed_values.clone(),
        error_title: Some("Validation Error".to_string()),
        error_message: req.error_message.clone(),
        input_title: None,
        input_message: None,
    };

    for row in req.start_row..=req.end_row {
        for col in req.start_col..=req.end_col {
            let key = format!("{},{}", row, col);
            validations.insert(key, rule.clone());
        }
    }

    sheet.updated_at = Utc::now();
    if let Err(e) = save_sheet_to_drive(&state, &user_id, &sheet).await {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e }))));
    }

    Ok(Json(SaveResponse { id: req.sheet_id, success: true, message: Some("Data validation applied".to_string()) }))
}

pub async fn handle_validate_cell(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ValidateCellRequest>,
) -> Result<Json<ValidationResult>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let sheet = match load_sheet_from_drive(&state, &user_id, &req.sheet_id).await {
        Ok(s) => s,
        Err(e) => return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": e })))),
    };

    if req.worksheet_index >= sheet.worksheets.len() {
        return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Invalid worksheet index" }))));
    }

    let worksheet = &sheet.worksheets[req.worksheet_index];
    let key = format!("{},{}", req.row, req.col);

    let rule = worksheet.validations.as_ref().and_then(|v| v.get(&key));

    if let Some(rule) = rule {
        let (valid, error_msg) = validate_value(&req.value, rule);
        Ok(Json(ValidationResult { valid, error_message: if valid { None } else { error_msg } }))
    } else {
        Ok(Json(ValidationResult { valid: true, error_message: None }))
    }
}

fn validate_value(value: &str, rule: &ValidationRule) -> (bool, Option<String>) {
    let error_msg = rule.error_message.clone().unwrap_or_else(|| "Invalid value".to_string());

    match rule.validation_type.as_str() {
        "list" => {
            if let Some(ref allowed) = rule.allowed_values {
                let valid = allowed.iter().any(|v| v == value);
                (valid, Some(error_msg))
            } else { (true, None) }
        }
        "number" => {
            let num = match value.parse::<f64>() {
                Ok(n) => n,
                Err(_) => return (false, Some("Must be a number".to_string())),
            };
            let op = rule.operator.as_deref().unwrap_or("between");
            let v1 = rule.value1.as_ref().and_then(|v| v.parse::<f64>().ok());
            let v2 = rule.value2.as_ref().and_then(|v| v.parse::<f64>().ok());

            let valid = match op {
                "between" => v1.zip(v2).map(|(a, b)| num >= a && num <= b).unwrap_or(true),
                "notBetween" => v1.zip(v2).map(|(a, b)| num < a || num > b).unwrap_or(true),
                "greaterThan" => v1.map(|a| num > a).unwrap_or(true),
                "lessThan" => v1.map(|a| num < a).unwrap_or(true),
                "greaterThanOrEqual" => v1.map(|a| num >= a).unwrap_or(true),
                "lessThanOrEqual" => v1.map(|a| num <= a).unwrap_or(true),
                "equal" => v1.map(|a| (num - a).abs() < f64::EPSILON).unwrap_or(true),
                "notEqual" => v1.map(|a| (num - a).abs() >= f64::EPSILON).unwrap_or(true),
                _ => true,
            };
            (valid, Some(error_msg))
        }
        "textLength" => {
            let len = value.chars().count();
            let op = rule.operator.as_deref().unwrap_or("between");
            let v1 = rule.value1.as_ref().and_then(|v| v.parse::<usize>().ok());
            let v2 = rule.value2.as_ref().and_then(|v| v.parse::<usize>().ok());

            let valid = match op {
                "between" => v1.zip(v2).map(|(a, b)| len >= a && len <= b).unwrap_or(true),
                "greaterThan" => v1.map(|a| len > a).unwrap_or(true),
                "lessThan" => v1.map(|a| len < a).unwrap_or(true),
                "equal" => v1.map(|a| len == a).unwrap_or(true),
                _ => true,
            };
            (valid, Some(error_msg))
        }
        "date" => {
            let valid = NaiveDate::parse_from_str(value, "%Y-%m-%d").is_ok();
            (valid, Some("Must be a valid date (YYYY-MM-DD)".to_string()))
        }
        "custom" => {
            if let Some(ref formula) = rule.value1 {
                (value == formula, Some(error_msg))
            } else { (true, None) }
        }
        _ => (true, None),
    }
}

pub async fn handle_add_note(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddNoteRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut sheet = match load_sheet_from_drive(&state, &user_id, &req.sheet_id).await {
        Ok(s) => s,
        Err(e) => return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": e })))),
    };

    if req.worksheet_index >= sheet.worksheets.len() {
        return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Invalid worksheet index" }))));
    }

    let worksheet = &mut sheet.worksheets[req.worksheet_index];
    let key = format!("{},{}", req.row, req.col);

    let cell = worksheet.data.entry(key).or_insert_with(|| CellData {
        value: None, formula: None, style: None, format: None, note: None,
    });
    cell.note = if req.note.is_empty() { None } else { Some(req.note.clone()) };

    sheet.updated_at = Utc::now();
    if let Err(e) = save_sheet_to_drive(&state, &user_id, &sheet).await {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e }))));
    }

    Ok(Json(SaveResponse { id: req.sheet_id, success: true, message: Some("Note added".to_string()) }))
}

pub async fn handle_import_sheet(
    State(state): State<Arc<AppState>>,
    body: axum::body::Bytes,
) -> Result<Json<Spreadsheet>, (StatusCode, Json<serde_json::Value>)> {
    let content = String::from_utf8_lossy(&body);
    let user_id = get_current_user_id();

    let mut worksheet_data = HashMap::new();
    for (row_idx, line) in content.lines().enumerate() {
        let cols: Vec<&str> = line.split(',').collect();
        for (col_idx, value) in cols.iter().enumerate() {
            let clean_value = value.trim().trim_matches('"').to_string();
            if !clean_value.is_empty() {
                let key = format!("{},{}", row_idx, col_idx);
                worksheet_data.insert(key, CellData {
                    value: Some(clean_value), formula: None, style: None, format: None, note: None,
                });
            }
        }
    }

    let sheet = Spreadsheet {
        id: Uuid::new_v4().to_string(),
        name: "Imported Spreadsheet".to_string(),
        owner_id: user_id.clone(),
        worksheets: vec![Worksheet {
            name: "Sheet1".to_string(),
            data: worksheet_data,
            column_widths: None, row_heights: None, frozen_rows: None, frozen_cols: None,
            merged_cells: None, filters: None, hidden_rows: None, validations: None,
            conditional_formats: None, charts: None,
        }],
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    if let Err(e) = save_sheet_to_drive(&state, &user_id, &sheet).await {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e }))));
    }

    Ok(Json(sheet))
}

pub async fn handle_get_collaborators(
    Path(sheet_id): Path<String>,
) -> impl IntoResponse {
    let channels = get_collab_channels().read().await;
    let active = channels.contains_key(&sheet_id);
    Json(serde_json::json!({ "sheet_id": sheet_id, "collaborators": [], "active": active }))
}

pub async fn handle_sheet_websocket(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Path(sheet_id): Path<String>,
) -> impl IntoResponse {
    info!("Sheet WebSocket connection request for sheet: {}", sheet_id);
    ws.on_upgrade(move |socket| handle_sheet_connection(socket, state, sheet_id))
}

async fn handle_sheet_connection(socket: WebSocket, _state: Arc<AppState>, sheet_id: String) {
    let (mut sender, mut receiver) = socket.split();

    let channels = get_collab_channels();
    let rx = {
        let mut channels_write = channels.write().await;
        let tx = channels_write.entry(sheet_id.clone()).or_insert_with(|| broadcast::channel(256).0);
        tx.subscribe()
    };

    let user_id = format!("user-{}", &Uuid::new_v4().to_string()[..8]);
    let user_color = get_random_color();

    let welcome = serde_json::json!({
        "type": "connected",
        "sheet_id": sheet_id,
        "user_id": user_id,
        "user_color": user_color,
        "timestamp": Utc::now().to_rfc3339()
    });

    if sender.send(Message::Text(welcome.to_string())).await.is_err() {
        error!("Failed to send welcome message");
        return;
    }

    info!("User {} connected to sheet {}", user_id, sheet_id);
    broadcast_sheet_change(&sheet_id, "userJoined", &user_id, None, None, Some(&user_color)).await;

    let sheet_id_recv = sheet_id.clone();
    let user_id_recv = user_id.clone();
    let user_id_send = user_id.clone();

    let mut rx = rx;
    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if msg.user_id != user_id_send {
                if let Ok(json) = serde_json::to_string(&msg) {
                    if sender.send(Message::Text(json)).await.is_err() {
                        break;
                    }
                }
            }
        }
    });

    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                        let msg_type = parsed.get("type").and_then(|v| v.as_str()).unwrap_or("");
                        match msg_type {
                            "cellChange" => {
                                let row = parsed.get("row").and_then(|v| v.as_u64()).map(|v| v as u32);
                                let col = parsed.get("col").and_then(|v| v.as_u64()).map(|v| v as u32);
                                let value = parsed.get("value").and_then(|v| v.as_str()).map(String::from);
                                broadcast_sheet_change(&sheet_id_recv, "cellChange", &user_id_recv, row, col, value.as_deref()).await;
                            }
                            "cursor" => {
                                let row = parsed.get("row").and_then(|v| v.as_u64()).map(|v| v as u32);
                                let col = parsed.get("col").and_then(|v| v.as_u64()).map(|v| v as u32);
                                broadcast_sheet_change(&sheet_id_recv, "cursor", &user_id_recv, row, col, None).await;
                            }
                            _ => {}
                        }
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }

    broadcast_sheet_change(&sheet_id, "userLeft", &user_id, None, None, None).await;
    info!("User {} disconnected from sheet {}", user_id, sheet_id);
}

async fn broadcast_sheet_change(
    sheet_id: &str,
    msg_type: &str,
    user_id: &str,
    row: Option<u32>,
    col: Option<u32>,
    value: Option<&str>,
) {
    let channels = get_collab_channels().read().await;
    if let Some(tx) = channels.get(sheet_id) {
        let msg = CollabMessage {
            msg_type: msg_type.to_string(),
            sheet_id: sheet_id.to_string(),
            user_id: user_id.to_string(),
            user_name: format!("User {}", &user_id[..8.min(user_id.len())]),
            user_color: get_random_color(),
            row,
            col,
            value: value.map(String::from),
            worksheet_index: None,
            timestamp: Utc::now(),
        };
        let _ = tx.send(msg);
    }
}

fn get_random_color() -> String {
    let colors = [
        "#3b82f6", "#ef4444", "#22c55e", "#f59e0b", "#8b5cf6",
        "#ec4899", "#14b8a6", "#f97316", "#6366f1", "#84cc16",
    ];
    let idx = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as usize % colors.len())
        .unwrap_or(0);
    colors[idx].to_string()
}

pub mod collaboration;
pub mod export;
pub mod formulas;
pub mod handlers;
pub mod storage;
pub mod types;

use crate::shared::state::AppState;
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

pub use collaboration::{handle_get_collaborators, handle_sheet_websocket};
pub use handlers::{
    handle_add_note, handle_clear_filter, handle_conditional_format, handle_create_chart,
    handle_data_validation, handle_delete_chart, handle_delete_sheet, handle_evaluate_formula,
    handle_export_sheet, handle_filter_data, handle_format_cells, handle_freeze_panes,
    handle_get_sheet_by_id, handle_import_sheet, handle_list_sheets, handle_load_from_drive,
    handle_load_sheet, handle_merge_cells, handle_new_sheet, handle_save_sheet, handle_search_sheets,
    handle_share_sheet, handle_sheet_ai, handle_sort_range, handle_unmerge_cells, handle_update_cell,
    handle_validate_cell,
};
pub use types::{
    CellData, CellStyle, ChartConfig, ChartDataset, ChartOptions, ChartPosition, Collaborator,
    CollabMessage, ConditionalFormatRule, FilterConfig, MergedCell, SaveResponse, Spreadsheet,
    SpreadsheetMetadata, ValidationRule, Worksheet,
};

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
        .route("/api/sheet/ai", post(handle_sheet_ai))
        .route("/api/sheet/:id", get(handle_get_sheet_by_id))
        .route("/api/sheet/:id/collaborators", get(handle_get_collaborators))
        .route("/ws/sheet/:sheet_id", get(handle_sheet_websocket))
}

use crate::core::shared::state::AppState;
use axum::Router;
use std::sync::Arc;

pub use botsheet::{
    configure_sheet_routes as crate_configure_sheet_routes,
    ArrayFormula, CellComment, CellData, CellStyle, ChartConfig, ChartDataset, ChartOptions,
    ChartPosition, Collaborator, CollabMessage, CommentReply, ConditionalFormatRule,
    DriveOps as SheetDriveOps, ExternalLink, FilterConfig, MergedCell, NamedRange,
    SaveResponse, SheetState, Spreadsheet, SpreadsheetMetadata, ValidationRule, Worksheet,
};

fn make_sheet_state(app_state: &Arc<AppState>) -> SheetState {
    let drive: Option<Arc<dyn botsheet::DriveOps>> = None;
    SheetState::new(drive)
}

pub fn configure_sheet_routes(app_state: Arc<AppState>) -> Router {
    crate_configure_sheet_routes()
        .with_state(Arc::new(make_sheet_state(&app_state)))
}

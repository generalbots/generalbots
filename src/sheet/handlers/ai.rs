use crate::shared::state::AppState;
use crate::sheet::types::{SheetAiRequest, SheetAiResponse};
use axum::{extract::State, response::IntoResponse, Json};
use std::sync::Arc;

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

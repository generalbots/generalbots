//! MCP UI resource catalog and preview (#1675).

use super::*;
use crate::mcp_ui;

/// `GET /api/sources/mcp/:name/ui` — declared resources with hash and pin
/// state.
pub async fn handle_mcp_ui_catalog(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Query(params): Query<BotQuery>,
) -> impl IntoResponse {
    let bot_id = params.bot_id.unwrap_or_else(|| "default".to_string());
    let work_path = get_work_path_or_default(&state.get_work_path);
    let catalog = mcp_ui::load_catalog(&work_path, &bot_id);
    let entries = mcp_ui::entries_for(&catalog, &name);
    Json(ApiResponse::success(mcp_ui::views(&entries)))
}

/// `GET /api/sources/mcp/:name/ui/:index` — base64 HTML for the preview.
/// The client builds a Blob and renders it in an opaque sandbox; the raw HTML
/// never becomes a suite-origin document.
pub async fn handle_mcp_ui_preview(
    State(state): State<Arc<AppState>>,
    Path((name, index)): Path<(String, usize)>,
    Query(params): Query<BotQuery>,
) -> impl IntoResponse {
    let bot_id = params.bot_id.unwrap_or_else(|| "default".to_string());
    let work_path = get_work_path_or_default(&state.get_work_path);
    let catalog = mcp_ui::load_catalog(&work_path, &bot_id);
    let entries = mcp_ui::entries_for(&catalog, &name);
    let Some(entry) = entries.get(index) else {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<String>::error("UI resource not found")),
        )
            .into_response();
    };
    let (title, html_base64) = mcp_ui::preview_payload(entry);
    Json(ApiResponse::success(serde_json::json!({
        "name": title,
        "mime_type": entry.mime_type,
        "html_base64": html_base64,
        "hash": mcp_ui::hash_html(&entry.html),
    })))
    .into_response()
}

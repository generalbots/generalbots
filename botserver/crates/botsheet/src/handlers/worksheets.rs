use axum::{extract::State, http::StatusCode, response::Html, Json};
use serde::Deserialize;
use std::sync::Arc;

use crate::state::{get_current_user_id, load_sheet_by_id, save_sheet_to_drive, SheetState};
use crate::types::Spreadsheet;
use crate::types::Worksheet;
use crate::ui_fragments::sidebars::render_worksheet_grid_preview;
use chrono::Utc;

#[derive(Debug, Deserialize)]
pub struct WorksheetActionRequest {
    pub id: String,
    #[serde(default)]
    pub index: Option<usize>,
    #[serde(default)]
    pub name: Option<String>,
}

pub async fn add_worksheet(
    State(state): State<Arc<SheetState>>,
    Json(req): Json<WorksheetActionRequest>,
) -> Result<Html<String>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut sheet = match load_sheet_by_id(&state, &user_id, &req.id).await {
        Ok(s) => s,
        Err(e) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": e })),
            ))
        }
    };
    let next_num = sheet.worksheets.len() + 1;
    let name = req
        .name
        .unwrap_or_else(|| format!("Planilha {}", next_num));
    let ws = Worksheet {
        name,
        ..Worksheet::default()
    };
    sheet.worksheets.push(ws);
    sheet.updated_at = Utc::now();
    if let Err(e) = save_sheet_to_drive(&state, &user_id, &sheet).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ));
    }
    Ok(Html(render_tabs(&sheet)))
}

pub async fn delete_worksheet(
    State(state): State<Arc<SheetState>>,
    Json(req): Json<WorksheetActionRequest>,
) -> Result<Html<String>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut sheet = match load_sheet_by_id(&state, &user_id, &req.id).await {
        Ok(s) => s,
        Err(e) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": e })),
            ))
        }
    };
    let idx = req.index.unwrap_or(0);
    if idx < sheet.worksheets.len() && sheet.worksheets.len() > 1 {
        sheet.worksheets.remove(idx);
    }
    sheet.updated_at = Utc::now();
    if let Err(e) = save_sheet_to_drive(&state, &user_id, &sheet).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ));
    }
    let new_idx = idx.min(sheet.worksheets.len().saturating_sub(1));
    Ok(Html(render_grid_for(&sheet, new_idx)))
}

pub async fn switch_worksheet(
    State(state): State<Arc<SheetState>>,
    Json(req): Json<WorksheetActionRequest>,
) -> Result<Html<String>, (StatusCode, Json<serde_json::Value>)> {
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
    let idx = req.index.unwrap_or(0).min(sheet.worksheets.len().saturating_sub(1));
    Ok(Html(render_grid_for(&sheet, idx)))
}

pub async fn rename_worksheet(
    State(state): State<Arc<SheetState>>,
    Json(req): Json<WorksheetActionRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut sheet = match load_sheet_by_id(&state, &user_id, &req.id).await {
        Ok(s) => s,
        Err(e) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": e })),
            ))
        }
    };
    if let (Some(idx), Some(name)) = (req.index, req.name.as_ref()) {
        if idx < sheet.worksheets.len() {
            sheet.worksheets[idx].name = name.clone();
        }
    }
    sheet.updated_at = Utc::now();
    if let Err(e) = save_sheet_to_drive(&state, &user_id, &sheet).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ));
    }
    Ok(Json(serde_json::json!({ "success": true })))
}

fn render_tabs(sheet: &Spreadsheet) -> String {
    let mut html = String::from(
        r##"<div class="ss-tabs" id="worksheet-tabs" data-active-index="0" style="display:flex;gap:2px;padding:8px;border-bottom:1px solid #334155;background:#0f172a;align-items:center;flex-wrap:wrap;">
<button class="ss-tab-add" hx-post="/api/sheet/worksheets/add" hx-vals='{"id":"ID_PLACEHOLDER"}' hx-target="#worksheet-tabs" hx-swap="outerHTML" title="Nova Planilha" style="background:#3b82f6;color:white;border:none;width:28px;height:28px;border-radius:4px;cursor:pointer;flex-shrink:0;">+</button>"##
    );
    html = html.replace("ID_PLACEHOLDER", &sheet.id);
    for (i, ws) in sheet.worksheets.iter().enumerate() {
        let active_style = if i == 0 {
            r##"background:#1e293b;color:#f8fafc;border:1px solid #3b82f6;""##
        } else {
            r##"background:#0f172a;color:#94a3b8;border:1px solid #334155;""##
        };
        let active_class = if i == 0 { " ss-tab-active" } else { "" };
        html.push_str(&format!(
            r##"<div class="ss-tab{active}" data-index="{i}" style="{style}padding:6px 12px;border-radius:4px 4px 0 0;cursor:pointer;display:flex;align-items:center;gap:6px;flex-shrink:0;">
<button type="button" hx-post="/api/sheet/worksheets/switch" hx-vals='{{"id":"{sid}","index":{i}}}' hx-target="#sheet-content" hx-swap="innerHTML" hx-on::after-request="window.dispatchEvent(new CustomEvent('gb-sheet-tab', {{detail:{{index:{i}}}}}))" style="background:none;border:none;color:inherit;font-weight:600;cursor:pointer;padding:0;font-size:13px;">{name}</button>
<button type="button" class="ss-tab-del" hx-post="/api/sheet/worksheets/delete" hx-vals='{{"id":"{sid}","index":{i}}}' hx-target="#sheet-content" hx-swap="innerHTML" hx-confirm="Excluir {name}?" title="Excluir planilha" style="background:none;border:none;color:#94a3b8;cursor:pointer;font-size:14px;line-height:1;padding:0 2px;">×</button>
</div>"##,
            active = active_class,
            style = active_style,
            i = i,
            sid = sheet.id,
            name = html_escape(&ws.name)
        ));
    }
    html.push_str("</div>");
    html
}

fn render_grid_for(sheet: &Spreadsheet, idx: usize) -> String {
    let mut html = String::new();
    html.push_str(&render_tabs(sheet));
    if let Some(ws) = sheet.worksheets.get(idx) {
        html.push_str(&render_worksheet_grid_preview(ws, idx));
    } else if let Some(ws) = sheet.worksheets.first() {
        html.push_str(&render_worksheet_grid_preview(ws, 0));
    }
    html
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

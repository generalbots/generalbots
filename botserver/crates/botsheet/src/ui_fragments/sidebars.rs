use super::{empty_fragment, html_escape, render_metadata_card, render_spreadsheet_summary, Lang, t, tf};
use axum::http::HeaderMap;
use axum::{response::Html, Json};
use botsheet_core::types::{Spreadsheet, SpreadsheetMetadata, Worksheet};

pub async fn handle_sheet_list_sidebar(
    headers: HeaderMap,
    Json(items): Json<Vec<SpreadsheetMetadata>>,
) -> Html<String> {
    let lang = Lang::from_headers(&headers);
    if items.is_empty() {
        return Html(empty_fragment(&t(lang, "sidebar.empty")));
    }
    let mut html = String::from(
        r##"<div class="ss-sidebar" id="sheet-list-sidebar">
<div class="ss-sidebar-header" style="display:flex;justify-content:space-between;align-items:center;padding:12px;border-bottom:1px solid #334155;">
<h3 style="margin:0;color:#f8fafc;font-size:14px;">"##
    );
    html.push_str(&t(lang, "sidebar.title"));
    html.push_str(
        r##"</h3>
<button class="btn-icon" hx-get="/api/sheet/new" hx-ext="json-enc" hx-target="#sheet-content" hx-swap="innerHTML" title=""##,
    );
    html.push_str(&t(lang, "sidebar.new_title"));
    html.push_str(
        r##"" style="background:#10b981;color:white;border:none;padding:4px 10px;border-radius:4px;cursor:pointer;font-size:12px;">"##,
    );
    html.push_str(&t(lang, "sidebar.new"));
    html.push_str(
        r##"</button>
</div>
<div class="ss-sidebar-body" style="padding:8px;display:flex;flex-direction:column;gap:8px;max-height:calc(100vh - 200px);overflow-y:auto;">"##,
    );
    for m in &items {
        html.push_str(&render_metadata_card(m, lang));
    }
    html.push_str("</div></div>");
    Html(html)
}

pub async fn handle_search_sidebar(
    headers: HeaderMap,
    Json(items): Json<Vec<SpreadsheetMetadata>>,
) -> Html<String> {
    let lang = Lang::from_headers(&headers);
    if items.is_empty() {
        return Html(empty_fragment(&t(lang, "search.empty")));
    }
    let mut html = String::from(
        r##"<div class="ss-sidebar" id="search-sidebar">
<div class="ss-sidebar-header" style="padding:12px;border-bottom:1px solid #334155;">
<h3 style="margin:0;color:#f8fafc;font-size:14px;">"##,
    );
    html.push_str(&t(lang, "search.title"));
    html.push_str(
        r##"</h3>
</div>
<div class="ss-sidebar-body" style="padding:8px;display:flex;flex-direction:column;gap:8px;">"##,
    );
    for m in &items {
        html.push_str(&render_metadata_card(m, lang));
    }
    html.push_str(&format!(
        r##"<div style="padding:8px;color:#94a3b8;font-size:12px;text-align:center;">{}</div>"##,
        tf(lang, "search.count", &[("n", &items.len().to_string())])
    ));
    html.push_str("</div></div>");
    Html(html)
}

pub async fn handle_recent_sidebar(
    headers: HeaderMap,
    Json(items): Json<Vec<SpreadsheetMetadata>>,
) -> Html<String> {
    let lang = Lang::from_headers(&headers);
    let recent: Vec<&SpreadsheetMetadata> = items.iter().take(5).collect();
    if recent.is_empty() {
        return Html(empty_fragment(&t(lang, "recent.empty")));
    }
    let mut html = String::from(
        r##"<div class="ss-sidebar" id="recent-sidebar">
<div class="ss-sidebar-header" style="padding:12px;border-bottom:1px solid #334155;">
<h3 style="margin:0;color:#f8fafc;font-size:14px;">"##,
    );
    html.push_str(&t(lang, "recent.title"));
    html.push_str(
        r##"</h3>
</div>
<div class="ss-sidebar-body" style="padding:8px;display:flex;flex-direction:column;gap:8px;">"##,
    );
    for m in recent {
        html.push_str(&render_metadata_card(m, lang));
    }
    html.push_str("</div></div>");
    Html(html)
}

pub async fn handle_worksheet_tabs_sidebar(
    headers: HeaderMap,
    Json(sheet): Json<Spreadsheet>,
) -> Html<String> {
    let lang = Lang::from_headers(&headers);
    let mut html = String::from(
        r##"<div class="ss-tabs" id="worksheet-tabs" data-active-index="0" style="display:flex;gap:2px;padding:8px;border-bottom:1px solid #334155;background:#0f172a;align-items:center;flex-wrap:wrap;">
<button class="ss-tab-add" hx-post="/api/sheet/worksheets/add" hx-vals='{"id":"ID_PLACEHOLDER"}' hx-target="#worksheet-tabs" hx-swap="outerHTML" title=""##,
    );
    html.push_str(&t(lang, "tabs.add_title"));
    html.push_str(
        r##"" style="background:#3b82f6;color:white;border:none;width:28px;height:28px;border-radius:4px;cursor:pointer;flex-shrink:0;">+</button>"##,
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
            r##"<div class="ss-tab{active}" data-index="{i}" style="{style}padding:6px 12px;border-radius:4px 4px 0 0;display:flex;align-items:center;gap:6px;flex-shrink:0;">
<button type="button" hx-post="/api/sheet/worksheets/switch" hx-vals='{{"id":"{sid}","index":{i}}}' hx-target="#sheet-content" hx-swap="innerHTML" hx-on::after-request="window.dispatchEvent(new CustomEvent('gb-sheet-tab',{{detail:{{index:{i}}}}}))" style="background:none;border:none;color:inherit;font-weight:600;cursor:pointer;padding:0;font-size:13px;">{name}</button>
<button type="button" class="ss-tab-del" hx-post="/api/sheet/worksheets/delete" hx-vals='{{"id":"{sid}","index":{i}}}' hx-target="#sheet-content" hx-swap="innerHTML" hx-confirm="{confirm}" title="{del_title}" style="background:none;border:none;color:#94a3b8;cursor:pointer;font-size:14px;line-height:1;padding:0 2px;">×</button>
</div>"##,
            active = active_class,
            style = active_style,
            i = i,
            sid = sheet.id,
            name = html_escape(&ws.name),
            confirm = tf(lang, "tabs.confirm_delete", &[("name", &ws.name)]),
            del_title = t(lang, "tabs.delete_title")
        ));
    }
    html.push_str("</div>");
    html.push_str(&render_spreadsheet_summary(&sheet, lang));
    let ws = sheet.worksheets.first();
    if let Some(ws) = ws {
        html.push_str(&render_worksheet_grid_preview(ws, 0, lang));
    }
    Html(html)
}

pub fn render_worksheet_grid_preview(ws: &Worksheet, idx: usize, lang: Lang) -> String {
    let mut html = format!(
        r##"<div class="ss-grid-preview" id="worksheet-{idx}" data-index="{idx}" style="padding:12px;">
<div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:8px;">
<strong style="color:#f8fafc;">{}</strong>
<span style="color:#94a3b8;font-size:12px;">{}</span>
</div>
<div style="display:grid;grid-template-columns:repeat(8,1fr);gap:2px;background:#334155;border-radius:4px;overflow:hidden;">"##,
        html_escape(&ws.name),
        tf(lang, "preview.cells", &[("n", &ws.data.len().to_string())])
    );
    let entries: Vec<(&String, &botsheet_core::types::CellData)> = ws.data.iter().take(64).collect();
    for (k, v) in entries {
        let display = v.value.as_deref().or(v.formula.as_deref()).unwrap_or("");
        let is_formula = v.formula.is_some();
        let bg = if is_formula { "#1e3a8a" } else { "#0f172a" };
        html.push_str(&format!(
            r##"<div style="background:{bg};padding:4px;font-size:10px;color:#cbd5e1;min-height:20px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-family:monospace;" title="{k}: {display}">{display}</div>"##,
            bg = bg,
            k = html_escape(k),
            display = html_escape(display)
        ));
    }
    html.push_str("</div></div>");
    html
}

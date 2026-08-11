use super::{html_escape, Lang, t, tf};
use axum::http::HeaderMap;
use axum::{response::Html, Json};

pub async fn handle_share_modal(
    headers: HeaderMap,
    Json(payload): Json<serde_json::Value>,
) -> Html<String> {
    let lang = Lang::from_headers(&headers);
    let sheet_id = payload.get("id").and_then(|v| v.as_str()).unwrap_or("");
    let name = payload
        .get("name")
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .unwrap_or_else(|| t(lang, "common.spreadsheet"));
    Html(format!(
        r##"<div class="ss-modal" id="share-modal" style="position:fixed;top:0;left:0;right:0;bottom:0;background:rgba(0,0,0,0.7);z-index:1000;display:flex;align-items:center;justify-content:center;">
<div class="ss-modal-content" style="background:#1e293b;border:1px solid #334155;border-radius:8px;max-width:480px;width:90%;max-height:80vh;overflow-y:auto;">
<div class="ss-modal-header" style="padding:16px;border-bottom:1px solid #334155;display:flex;justify-content:space-between;align-items:center;">
<h3 style="margin:0;color:#f8fafc;">{title}</h3>
<button onclick="this.closest('.ss-modal').remove()" style="background:none;border:none;color:#94a3b8;cursor:pointer;font-size:20px;">×</button>
</div>
<div class="ss-modal-body" hx-get="/suite/sheet/forms/share" hx-vals='{{"id":"{id}"}}' hx-trigger="load" hx-swap="innerHTML"></div>
</div>
</div>"##,
        title = tf(lang, "modal.share_title", &[("name", &html_escape(&name))]),
        id = html_escape(sheet_id)
    ))
}

pub async fn handle_chart_modal(
    headers: HeaderMap,
    Json(payload): Json<serde_json::Value>,
) -> Html<String> {
    let lang = Lang::from_headers(&headers);
    let sheet_id = payload.get("id").and_then(|v| v.as_str()).unwrap_or("");
    Html(format!(
        r##"<div class="ss-modal" id="chart-modal" style="position:fixed;top:0;left:0;right:0;bottom:0;background:rgba(0,0,0,0.7);z-index:1000;display:flex;align-items:center;justify-content:center;">
<div class="ss-modal-content" style="background:#1e293b;border:1px solid #334155;border-radius:8px;max-width:480px;width:90%;">
<div class="ss-modal-header" style="padding:16px;border-bottom:1px solid #334155;display:flex;justify-content:space-between;align-items:center;">
<h3 style="margin:0;color:#f8fafc;">{title}</h3>
<button onclick="this.closest('.ss-modal').remove()" style="background:none;border:none;color:#94a3b8;cursor:pointer;font-size:20px;">×</button>
</div>
<div class="ss-modal-body" hx-get="/suite/sheet/forms/chart" hx-vals='{{"id":"{id}"}}' hx-trigger="load" hx-swap="innerHTML"></div>
</div>
</div>"##,
        title = t(lang, "modal.chart"),
        id = html_escape(sheet_id)
    ))
}

pub async fn handle_find_replace_modal(
    headers: HeaderMap,
    Json(payload): Json<serde_json::Value>,
) -> Html<String> {
    let lang = Lang::from_headers(&headers);
    let sheet_id = payload.get("id").and_then(|v| v.as_str()).unwrap_or("");
    Html(format!(
        r##"<div class="ss-modal" id="find-replace-modal" style="position:fixed;top:0;left:0;right:0;bottom:0;background:rgba(0,0,0,0.7);z-index:1000;display:flex;align-items:center;justify-content:center;">
<div class="ss-modal-content" style="background:#1e293b;border:1px solid #334155;border-radius:8px;max-width:480px;width:90%;">
<div class="ss-modal-header" style="padding:16px;border-bottom:1px solid #334155;display:flex;justify-content:space-between;align-items:center;">
<h3 style="margin:0;color:#f8fafc;">{title}</h3>
<button onclick="this.closest('.ss-modal').remove()" style="background:none;border:none;color:#94a3b8;cursor:pointer;font-size:20px;">×</button>
</div>
<div class="ss-modal-body" hx-get="/suite/sheet/forms/find-replace" hx-vals='{{"id":"{id}"}}' hx-trigger="load" hx-swap="innerHTML"></div>
</div>
</div>"##,
        title = t(lang, "modal.find_replace"),
        id = html_escape(sheet_id)
    ))
}

pub async fn handle_conditional_format_modal(
    headers: HeaderMap,
    Json(payload): Json<serde_json::Value>,
) -> Html<String> {
    let lang = Lang::from_headers(&headers);
    let sheet_id = payload.get("id").and_then(|v| v.as_str()).unwrap_or("");
    Html(format!(
        r##"<div class="ss-modal" id="conditional-format-modal" style="position:fixed;top:0;left:0;right:0;bottom:0;background:rgba(0,0,0,0.7);z-index:1000;display:flex;align-items:center;justify-content:center;">
<div class="ss-modal-content" style="background:#1e293b;border:1px solid #334155;border-radius:8px;max-width:480px;width:90%;">
<div class="ss-modal-header" style="padding:16px;border-bottom:1px solid #334155;display:flex;justify-content:space-between;align-items:center;">
<h3 style="margin:0;color:#f8fafc;">{title}</h3>
<button onclick="this.closest('.ss-modal').remove()" style="background:none;border:none;color:#94a3b8;cursor:pointer;font-size:20px;">×</button>
</div>
<div class="ss-modal-body" hx-get="/suite/sheet/forms/conditional-format" hx-vals='{{"id":"{id}"}}' hx-trigger="load" hx-swap="innerHTML"></div>
</div>
</div>"##,
        title = t(lang, "modal.conditional_format"),
        id = html_escape(sheet_id)
    ))
}

pub async fn handle_data_validation_modal(
    headers: HeaderMap,
    Json(payload): Json<serde_json::Value>,
) -> Html<String> {
    let lang = Lang::from_headers(&headers);
    let sheet_id = payload.get("id").and_then(|v| v.as_str()).unwrap_or("");
    Html(format!(
        r##"<div class="ss-modal" id="data-validation-modal" style="position:fixed;top:0;left:0;right:0;bottom:0;background:rgba(0,0,0,0.7);z-index:1000;display:flex;align-items:center;justify-content:center;">
<div class="ss-modal-content" style="background:#1e293b;border:1px solid #334155;border-radius:8px;max-width:480px;width:90%;">
<div class="ss-modal-header" style="padding:16px;border-bottom:1px solid #334155;display:flex;justify-content:space-between;align-items:center;">
<h3 style="margin:0;color:#f8fafc;">{title}</h3>
<button onclick="this.closest('.ss-modal').remove()" style="background:none;border:none;color:#94a3b8;cursor:pointer;font-size:20px;">×</button>
</div>
<div class="ss-modal-body" hx-get="/suite/sheet/forms/data-validation" hx-vals='{{"id":"{id}"}}' hx-trigger="load" hx-swap="innerHTML"></div>
</div>
</div>"##,
        title = t(lang, "modal.data_validation"),
        id = html_escape(sheet_id)
    ))
}

pub async fn handle_custom_format_modal(
    headers: HeaderMap,
    Json(payload): Json<serde_json::Value>,
) -> Html<String> {
    let lang = Lang::from_headers(&headers);
    let sheet_id = payload.get("id").and_then(|v| v.as_str()).unwrap_or("");
    Html(format!(
        r##"<div class="ss-modal" id="custom-format-modal" style="position:fixed;top:0;left:0;right:0;bottom:0;background:rgba(0,0,0,0.7);z-index:1000;display:flex;align-items:center;justify-content:center;">
<div class="ss-modal-content" style="background:#1e293b;border:1px solid #334155;border-radius:8px;max-width:480px;width:90%;">
<div class="ss-modal-header" style="padding:16px;border-bottom:1px solid #334155;display:flex;justify-content:space-between;align-items:center;">
<h3 style="margin:0;color:#f8fafc;">{title}</h3>
<button onclick="this.closest('.ss-modal').remove()" style="background:none;border:none;color:#94a3b8;cursor:pointer;font-size:20px;">×</button>
</div>
<div class="ss-modal-body" hx-get="/suite/sheet/forms/custom-format" hx-vals='{{"id":"{id}"}}' hx-trigger="load" hx-swap="innerHTML"></div>
</div>
</div>"##,
        title = t(lang, "modal.custom_format"),
        id = html_escape(sheet_id)
    ))
}

pub async fn handle_insert_image_modal(
    headers: HeaderMap,
    Json(payload): Json<serde_json::Value>,
) -> Html<String> {
    let lang = Lang::from_headers(&headers);
    let sheet_id = payload.get("id").and_then(|v| v.as_str()).unwrap_or("");
    Html(format!(
        r##"<div class="ss-modal" id="insert-image-modal" style="position:fixed;top:0;left:0;right:0;bottom:0;background:rgba(0,0,0,0.7);z-index:1000;display:flex;align-items:center;justify-content:center;">
<div class="ss-modal-content" style="background:#1e293b;border:1px solid #334155;border-radius:8px;max-width:480px;width:90%;">
<div class="ss-modal-header" style="padding:16px;border-bottom:1px solid #334155;display:flex;justify-content:space-between;align-items:center;">
<h3 style="margin:0;color:#f8fafc;">{title}</h3>
<button onclick="this.closest('.ss-modal').remove()" style="background:none;border:none;color:#94a3b8;cursor:pointer;font-size:20px;">×</button>
</div>
<div class="ss-modal-body" hx-get="/suite/sheet/forms/insert-image" hx-vals='{{"id":"{id}"}}' hx-trigger="load" hx-swap="innerHTML"></div>
</div>
</div>"##,
        title = t(lang, "modal.insert_image"),
        id = html_escape(sheet_id)
    ))
}

pub async fn handle_print_preview_modal(
    headers: HeaderMap,
    Json(payload): Json<serde_json::Value>,
) -> Html<String> {
    let lang = Lang::from_headers(&headers);
    let sheet_id = payload.get("id").and_then(|v| v.as_str()).unwrap_or("");
    Html(format!(
        r##"<div class="ss-modal" id="print-preview-modal" style="position:fixed;top:0;left:0;right:0;bottom:0;background:rgba(0,0,0,0.7);z-index:1000;display:flex;align-items:center;justify-content:center;">
<div class="ss-modal-content" style="background:#1e293b;border:1px solid #334155;border-radius:8px;max-width:640px;width:90%;">
<div class="ss-modal-header" style="padding:16px;border-bottom:1px solid #334155;display:flex;justify-content:space-between;align-items:center;">
<h3 style="margin:0;color:#f8fafc;">{title}</h3>
<button onclick="this.closest('.ss-modal').remove()" style="background:none;border:none;color:#94a3b8;cursor:pointer;font-size:20px;">×</button>
</div>
<div class="ss-modal-body" hx-get="/suite/sheet/forms/print-preview" hx-vals='{{"id":"{id}"}}' hx-trigger="load" hx-swap="innerHTML"></div>
</div>
</div>"##,
        title = t(lang, "modal.print_preview"),
        id = html_escape(sheet_id)
    ))
}

pub async fn handle_ai_panel(
    headers: HeaderMap,
    Json(payload): Json<serde_json::Value>,
) -> Html<String> {
    let lang = Lang::from_headers(&headers);
    let sheet_id = payload.get("id").and_then(|v| v.as_str()).unwrap_or("");
    let prompt = payload.get("prompt").and_then(|v| v.as_str()).unwrap_or("");
    Html(format!(
        r##"<div class="ss-panel" id="ai-panel" style="padding:16px;background:#1e1b4b;border:1px solid #4338ca;border-radius:8px;">
<div style="display:flex;align-items:center;gap:8px;margin-bottom:12px;">
<span style="font-size:24px;">✨</span>
<h3 style="margin:0;color:#c7d2fe;font-size:16px;">{assistant}</h3>
</div>
<form hx-post="/api/sheet/ai" hx-vals='{{"id":"{id}","prompt":"{prompt}"}}' hx-target="#ai-response" hx-swap="innerHTML" style="display:flex;flex-direction:column;gap:8px;">
<textarea name="question" placeholder="{placeholder}" rows="3" style="padding:8px;background:#0f172a;border:1px solid #4338ca;border-radius:4px;color:#f8fafc;font-family:inherit;resize:vertical;">{prompt}</textarea>
<button type="submit" style="background:#6366f1;color:white;border:none;padding:10px;border-radius:4px;cursor:pointer;font-weight:600;">{submit}</button>
</form>
<div id="ai-response" style="margin-top:12px;padding:8px;background:#0f172a;border-radius:4px;min-height:60px;color:#cbd5e1;font-size:13px;"></div>
</div>"##,
        assistant = t(lang, "modal.ai_assistant"),
        placeholder = t(lang, "modal.ai_placeholder"),
        submit = t(lang, "modal.ai_submit"),
        id = html_escape(sheet_id),
        prompt = html_escape(prompt)
    ))
}

pub async fn handle_advanced_ranges_panel(
    headers: HeaderMap,
    Json(_payload): Json<serde_json::Value>,
) -> Html<String> {
    let lang = Lang::from_headers(&headers);
    Html(format!(
        r##"<div class="ss-panel" id="advanced-ranges-panel" style="padding:16px;">
<h3 style="color:#f8fafc;margin-top:0;">{title}</h3>
<div style="display:grid;grid-template-columns:1fr 1fr;gap:12px;">
<button hx-get="/suite/sheet/modals/conditional-format" hx-target="#modal-container" hx-swap="innerHTML" class="ss-action-btn" style="padding:12px;background:#1e293b;border:1px solid #334155;border-radius:6px;color:#f8fafc;cursor:pointer;text-align:left;">
<strong>{cf_title}</strong>
<div style="font-size:11px;color:#94a3b8;margin-top:4px;">{cf_desc}</div>
</button>
<button hx-get="/suite/sheet/modals/data-validation" hx-target="#modal-container" hx-swap="innerHTML" class="ss-action-btn" style="padding:12px;background:#1e293b;border:1px solid #334155;border-radius:6px;color:#f8fafc;cursor:pointer;text-align:left;">
<strong>{dv_title}</strong>
<div style="font-size:11px;color:#94a3b8;margin-top:4px;">{dv_desc}</div>
</button>
<button hx-get="/suite/sheet/modals/chart" hx-target="#modal-container" hx-swap="innerHTML" class="ss-action-btn" style="padding:12px;background:#1e293b;border:1px solid #334155;border-radius:6px;color:#f8fafc;cursor:pointer;text-align:left;">
<strong>{charts_title}</strong>
<div style="font-size:11px;color:#94a3b8;margin-top:4px;">{charts_desc}</div>
</button>
<button hx-get="/suite/sheet/modals/print-preview" hx-target="#modal-container" hx-swap="innerHTML" class="ss-action-btn" style="padding:12px;background:#1e293b;border:1px solid #334155;border-radius:6px;color:#f8fafc;cursor:pointer;text-align:left;">
<strong>{print_title}</strong>
<div style="font-size:11px;color:#94a3b8;margin-top:4px;">{print_desc}</div>
</button>
<button hx-get="/suite/sheet/modals/ai" hx-target="#modal-container" hx-swap="innerHTML" class="ss-action-btn" style="padding:12px;background:#1e293b;border:1px solid #334155;border-radius:6px;color:#f8fafc;cursor:pointer;text-align:left;">
<strong>{ai_title}</strong>
<div style="font-size:11px;color:#94a3b8;margin-top:4px;">{ai_desc}</div>
</button>
<button hx-get="/suite/sheet/modals/find-replace" hx-target="#modal-container" hx-swap="innerHTML" class="ss-action-btn" style="padding:12px;background:#1e293b;border:1px solid #334155;border-radius:6px;color:#f8fafc;cursor:pointer;text-align:left;">
<strong>{find_title}</strong>
<div style="font-size:11px;color:#94a3b8;margin-top:4px;">{find_desc}</div>
</button>
</div>
</div>"##,
        title = t(lang, "panel.advanced.title"),
        cf_title = t(lang, "modal.conditional_format"),
        cf_desc = t(lang, "panel.advanced.cf_desc"),
        dv_title = t(lang, "modal.data_validation"),
        dv_desc = t(lang, "panel.advanced.dv_desc"),
        charts_title = t(lang, "modal.chart"),
        charts_desc = t(lang, "panel.advanced.charts_desc"),
        print_title = t(lang, "modal.print_preview"),
        print_desc = t(lang, "panel.advanced.print_desc"),
        ai_title = t(lang, "modal.ai_assistant"),
        ai_desc = t(lang, "panel.advanced.ai_desc"),
        find_title = t(lang, "modal.find_replace"),
        find_desc = t(lang, "panel.advanced.find_desc")
    ))
}

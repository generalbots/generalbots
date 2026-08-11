pub mod panels;
pub mod sidebars;
pub mod toolbar;
pub mod modals;
pub mod i18n;

pub use i18n::{Lang, t, tf};

use axum::{routing::{get, post}, Router};
use botsheet_core::types::{Spreadsheet, SpreadsheetMetadata};
use serde::Deserialize;

use panels::{
    handle_named_ranges_panel, handle_charts_panel, handle_validations_panel,
    handle_conditional_formats_panel, handle_external_links_panel, handle_comments_panel,
    handle_protection_panel, handle_array_formulas_panel,
};
use sidebars::{
    handle_sheet_list_sidebar, handle_search_sidebar, handle_recent_sidebar,
    handle_worksheet_tabs_sidebar,
};
use toolbar::{
    handle_share_form, handle_find_replace_form, handle_conditional_format_form,
    handle_data_validation_form, handle_custom_format_form, handle_insert_image_form,
    handle_print_preview_form, handle_chart_form,
};
use modals::{
    handle_share_modal, handle_chart_modal, handle_find_replace_modal,
    handle_conditional_format_modal, handle_data_validation_modal,
    handle_custom_format_modal, handle_insert_image_modal, handle_print_preview_modal,
    handle_ai_panel, handle_advanced_ranges_panel,
};

pub fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

pub fn fmt_money(v: f64) -> String {
    if v.fract() == 0.0 {
        format!("R$ {:.0}", v)
    } else {
        format!("R$ {:.2}", v)
    }
}

pub fn fmt_number(v: f64) -> String {
    format!("{:.2}", v)
}

pub fn fmt_pct(v: f64) -> String {
    format!("{:.1}%", v)
}

pub fn err_fragment(lang: Lang, msg: &str) -> String {
    format!(
        r##"<div class="fragment-error" role="alert" style="padding:12px;border:1px solid #f87171;background:#7f1d1d;color:#fecaca;border-radius:6px;margin:8px 0;">
<strong>{}</strong> {}</div>"##,
        t(lang, "common.error_label"),
        html_escape(msg)
    )
}

pub fn empty_fragment(msg: &str) -> String {
    format!(
        r##"<div class="fragment-empty" style="padding:24px;text-align:center;color:#94a3b8;font-style:italic;">
{}</div>"##,
        html_escape(msg)
    )
}

#[derive(Debug, Deserialize)]
pub struct SheetIdQuery {
    pub id: Option<String>,
}

pub fn configure<S: Clone + Send + Sync + 'static>() -> Router<S> {
    Router::new()
        .route("/suite/sheet/fragments/sheet-list", get(handle_sheet_list_sidebar))
        .route("/suite/sheet/fragments/search", get(handle_search_sidebar))
        .route("/suite/sheet/fragments/recent", get(handle_recent_sidebar))
        .route("/suite/sheet/fragments/tabs", post(handle_worksheet_tabs_sidebar))
        .route("/suite/sheet/fragments/named-ranges", post(handle_named_ranges_panel))
        .route("/suite/sheet/fragments/charts", post(handle_charts_panel))
        .route("/suite/sheet/fragments/validations", post(handle_validations_panel))
        .route("/suite/sheet/fragments/conditional-formats", post(handle_conditional_formats_panel))
        .route("/suite/sheet/fragments/external-links", post(handle_external_links_panel))
        .route("/suite/sheet/fragments/comments", post(handle_comments_panel))
        .route("/suite/sheet/fragments/protection", post(handle_protection_panel))
        .route("/suite/sheet/fragments/array-formulas", post(handle_array_formulas_panel))
        .route("/suite/sheet/fragments/advanced-ranges", post(handle_advanced_ranges_panel))
        .route("/suite/sheet/forms/share", post(handle_share_form))
        .route("/suite/sheet/forms/find-replace", post(handle_find_replace_form))
        .route("/suite/sheet/forms/conditional-format", post(handle_conditional_format_form))
        .route("/suite/sheet/forms/data-validation", post(handle_data_validation_form))
        .route("/suite/sheet/forms/custom-format", post(handle_custom_format_form))
        .route("/suite/sheet/forms/insert-image", post(handle_insert_image_form))
        .route("/suite/sheet/forms/print-preview", post(handle_print_preview_form))
        .route("/suite/sheet/forms/chart", post(handle_chart_form))
        .route("/suite/sheet/modals/share", post(handle_share_modal))
        .route("/suite/sheet/modals/chart", post(handle_chart_modal))
        .route("/suite/sheet/modals/find-replace", post(handle_find_replace_modal))
        .route("/suite/sheet/modals/conditional-format", post(handle_conditional_format_modal))
        .route("/suite/sheet/modals/data-validation", post(handle_data_validation_modal))
        .route("/suite/sheet/modals/custom-format", post(handle_custom_format_modal))
        .route("/suite/sheet/modals/insert-image", post(handle_insert_image_modal))
        .route("/suite/sheet/modals/print-preview", post(handle_print_preview_modal))
        .route("/suite/sheet/modals/ai", post(handle_ai_panel))
        .route("/suite/sheet/health", get(health))
}

async fn health() -> &'static str { "ok" }

pub fn render_metadata_card(m: &SpreadsheetMetadata, lang: Lang) -> String {
    format!(
        r##"<div class="ss-metadata-card" onclick="loadSheet('{id}')" style="padding:12px;border:1px solid #334155;border-radius:6px;background:#1e293b;cursor:pointer;">
<div style="font-weight:600;color:#f8fafc;">{name}</div>
<div style="font-size:12px;color:#94a3b8;margin-top:4px;">{count} • {updated}</div>
</div>"##,
        id = html_escape(&m.id),
        name = html_escape(&m.name),
        count = tf(lang, "metadata.sheets_count", &[("n", &m.worksheet_count.to_string())]),
        updated = tf(lang, "metadata.updated", &[("date", &m.updated_at.format("%d/%m/%Y %H:%M").to_string())])
    )
}

pub fn render_spreadsheet_summary(s: &Spreadsheet, lang: Lang) -> String {
    let ws_count = s.worksheets.len();
    let cell_count: usize = s.worksheets.iter().map(|w| w.data.len()).sum();
    let named_count = s.named_ranges.as_ref().map(|v| v.len()).unwrap_or(0);
    format!(
        r##"<div class="ss-summary" style="display:grid;grid-template-columns:repeat(4,1fr);gap:12px;padding:16px;">
<div class="ss-stat" style="padding:16px;background:#1e293b;border:1px solid #334155;border-radius:8px;">
<div style="font-size:24px;font-weight:600;color:#3b82f6;">{ws_count}</div>
<div style="font-size:12px;color:#94a3b8;">{ws_label}</div>
</div>
<div class="ss-stat" style="padding:16px;background:#1e293b;border:1px solid #334155;border-radius:8px;">
<div style="font-size:24px;font-weight:600;color:#10b981;">{cell_count}</div>
<div style="font-size:12px;color:#94a3b8;">{cells_label}</div>
</div>
<div class="ss-stat" style="padding:16px;background:#1e293b;border:1px solid #334155;border-radius:8px;">
<div style="font-size:24px;font-weight:600;color:#f59e0b;">{named_count}</div>
<div style="font-size:12px;color:#94a3b8;">{named_label}</div>
</div>
<div class="ss-stat" style="padding:16px;background:#1e293b;border:1px solid #334155;border-radius:8px;">
<div style="font-size:24px;font-weight:600;color:#8b5cf6;">{owner}</div>
<div style="font-size:12px;color:#94a3b8;">{owner_label}</div>
</div>
</div>"##,
        ws_label = t(lang, "common.sheets"),
        cells_label = t(lang, "common.filled_cells"),
        named_label = t(lang, "common.named_ranges"),
        owner_label = t(lang, "common.owner"),
        owner = html_escape(&s.owner_id)
    )
}

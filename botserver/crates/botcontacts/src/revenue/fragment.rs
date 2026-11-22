//! #2548 — no-JS fallback for the CRM revenue summary.
//!
//! Labels reuse existing catalog keys only (no new i18n entry is required for
//! this fragment). Values that cannot be computed are rendered as an em dash,
//! never as `0`. The full typed report is embedded as JSON for the enhancing
//! script in `botui/ui/suite/crm/crm-revenue.js`.

use axum::{
    extract::State,
    http::HeaderMap,
    response::Html,
};
use std::sync::Arc;

use super::{evaluate::build_report, store};
use crate::models::html_escape;
use crate::CrateState;

fn unavailable_or<T, F>(value: Option<T>, render: F) -> String
where
    F: FnOnce(T) -> String,
{
    match value {
        Some(item) => render(item),
        None => "\u{2014}".to_string(),
    }
}

pub async fn attribution_fragment(
    State(state): State<Arc<CrateState>>,
    headers: HeaderMap,
) -> Html<String> {
    let input = match store::load_input(&state, &headers, None, None) {
        Ok(value) => value,
        Err(error) => {
            log::error!("CRM revenue fragment unavailable: {error}");
            return Html(
                r#"<section id="crm-revenue-report" class="crm-revenue-report" data-state="error" role="status" aria-live="polite"></section>"#.to_string(),
            );
        }
    };
    let report = build_report(&input);
    let json = match serde_json::to_string(&report) {
        Ok(value) => value,
        Err(error) => {
            log::error!("CRM revenue report could not be serialized: {error}");
            "{}".to_string()
        }
    };
    let args = format!(
        "{{\"sent\":{},\"delivered\":{},\"opened\":{},\"clicked\":{}}}",
        report.touches.send, report.touches.delivery, report.touches.open, report.touches.click
    );
    let revenue = unavailable_or(report.revenue_influenced.clone(), |money| {
        format!("{} {}", format!("{:.2}", money.amount), html_escape(&money.currency))
    });
    let conversion = unavailable_or(report.conversion_rate, |rate| format!("{rate:.1}%"));
    let quality = report
        .data_quality
        .notes
        .iter()
        .map(|note| note.as_str())
        .collect::<Vec<&str>>()
        .join(",");
    // JSON is raw text inside <script>; HTML entities would not be decoded.
    // JSON-unescape the markup characters instead so a currency value from a
    // CRM row can never close the element.
    let safe_json = json
        .replace('&', "\\u0026")
        .replace('<', "\\u003c")
        .replace('>', "\\u003e");

    Html(format!(
        concat!(
            "<section id=\"crm-revenue-report\" class=\"crm-revenue-report\" data-state=\"ok\"",
            " data-unattributed=\"{}\" data-quality=\"{}\" role=\"status\" aria-live=\"polite\">",
            "<p data-i18n=\"crm-campaign-metrics-sent\" data-i18n-args='{}'></p>",
            "<div class=\"crm-revenue-totals\">",
            "<span data-i18n=\"crm-col-revenue\"></span> <strong>{}</strong>",
            "<span data-i18n=\"crm-conversion-rate\"></span> <strong>{}</strong>",
            "</div>",
            "<script type=\"application/json\" id=\"crm-revenue-json\">{}</script>",
            "</section>"
        ),
        report.unattributed_recipients,
        html_escape(&quality),
        html_escape(&args),
        revenue,
        conversion,
        safe_json
    ))
}

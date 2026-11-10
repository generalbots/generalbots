//! HTML rendering for the data and chart answer modes.

use uuid::Uuid;

pub fn format_results_as_html_table(results: &[serde_json::Value]) -> String {
    if results.is_empty() {
        return "<p><em>No results found.</em></p>".to_string();
    }

    let first = &results[0];
    let keys: Vec<String> = match first {
        serde_json::Value::Object(map) => map.keys().cloned().collect(),
        _ => return format!("<p>{}</p>", escape_html(&first.to_string())),
    };

    let mut html = String::from(
        "<div style=\"overflow-x:auto;margin:12px 0;\"><table style=\"\
            border-collapse:collapse;width:100%;font-family:system-ui,sans-serif;\
            font-size:14px;box-shadow:0 1px 3px rgba(0,0,0,0.1);border-radius:8px;\
            overflow:hidden;\">",
    );

    html.push_str("<thead style=\"background:#f8f9fa;\"><tr>");
    for key in &keys {
        html.push_str(&format!(
            "<th style=\"padding:10px 12px;text-align:left;font-weight:600;\
             border-bottom:2px solid #dee2e6;white-space:nowrap;\">{}</th>",
            escape_html(key)
        ));
    }
    html.push_str("</tr></thead><tbody>");

    for (i, row) in results.iter().enumerate() {
        let bg = if i % 2 == 0 { "#ffffff" } else { "#f8f9fa" };
        html.push_str(&format!("<tr style=\"background:{bg};\">"));
        if let serde_json::Value::Object(map) = row {
            for key in &keys {
                let val = map
                    .get(key)
                    .map(|v| match v {
                        serde_json::Value::Null => String::from("<em>null</em>"),
                        serde_json::Value::String(s) => escape_html(s),
                        other => escape_html(&other.to_string()),
                    })
                    .unwrap_or_default();
                html.push_str(&format!(
                    "<td style=\"padding:8px 12px;border-bottom:1px solid #eee;\">{val}</td>"
                ));
            }
        }
        html.push_str("</tr>");
    }

    html.push_str("</tbody></table></div>");
    html.push_str(&format!(
        "<p style=\"font-size:13px;color:#666;margin-top:8px;\">{} row(s) returned</p>",
        results.len()
    ));
    html
}

pub fn format_chart_html(chart_config: &serde_json::Value) -> Result<String, String> {
    let chart_id = format!(
        "chart_{}",
        Uuid::new_v4()
            .to_string()
            .split('-')
            .next()
            .unwrap_or("main")
    );
    let config_json = serde_json::to_string_pretty(chart_config)
        .map_err(|e| format!("Failed to serialize chart config: {e}"))?;

    let html = format!(
        r#"<div style="margin:16px 0;padding:16px;background:#fff;border-radius:12px;box-shadow:0 2px 8px rgba(0,0,0,0.08);">
<canvas id="{chart_id}" style="max-width:100%;height:auto;"></canvas>
</div>
<script>
(function() {{
function renderChart_{chart_id}() {{
var ctx = document.getElementById('{chart_id}');
if (!ctx) return;
var config = {config_json};
if (typeof Chart !== 'undefined') {{
new Chart(ctx, config);
}} else {{
var s = document.createElement('script');
s.src = '/suite/js/vendor/chart.umd.min.js';
s.onload = function() {{ new Chart(ctx, config); }};
document.head.appendChild(s);
}}
}}
renderChart_{chart_id}();
}})();
</script>"#
    );
    Ok(html)
}

pub(super) fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

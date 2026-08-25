//! HTMX-friendly consent card markup.
//!
//! Cards use the `gb-consent-` class prefix. Prompt buttons POST to
//! `/api/consent/resolve` and swap the whole card (`hx-target="closest
//! .consent-card"`, `hx-swap="outerHTML"`), so the server answer replaces the
//! prompt with the outcome summary.

use crate::enforce::PendingRequest;

pub fn escape_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

fn detail_lines(detail: &serde_json::Value) -> String {
    match detail {
        serde_json::Value::Object(map) => map
            .iter()
            .map(|(k, v)| {
                let value = if v.is_string() {
                    v.as_str().unwrap_or_default().to_string()
                } else {
                    v.to_string()
                };
                format!(
                    "<div class=\"gb-consent-detail-row\"><span class=\"gb-consent-detail-key\">{}</span> <span class=\"gb-consent-detail-value\">{}</span></div>",
                    escape_html(k),
                    escape_html(&value),
                )
            })
            .collect::<Vec<_>>()
            .join(""),
        serde_json::Value::Null => String::new(),
        other => format!(
            "<div class=\"gb-consent-detail-row\"><span class=\"gb-consent-detail-value\">{}</span></div>",
            escape_html(&other.to_string()),
        ),
    }
}

/// Consent prompt card rendered into chat when an operation awaits approval.
pub fn prompt_card_html(req: &PendingRequest) -> String {
    let app = escape_html(&req.app_id);
    let class = escape_html(&req.action_class);
    let request_id = escape_html(&req.request_id);
    format!(
        r#"<div class="consent-card gb-consent-prompt" data-request-id="{request_id}">
  <div class="gb-consent-title">Consent required</div>
  <div class="gb-consent-subtitle">Application <strong>{app}</strong> requests permission to perform a <strong>{class}</strong> action on your behalf.</div>
  <div class="gb-consent-details">{details}</div>
  <div class="gb-consent-actions">
    <button type="button" class="gb-consent-btn gb-consent-btn-allow-once"
      hx-post="/api/consent/resolve"
      hx-vals='{{"request_id":"{request_id}","decision":"allow_once"}}'
      hx-target="closest .consent-card" hx-swap="outerHTML">Allow once</button>
    <button type="button" class="gb-consent-btn gb-consent-btn-always"
      hx-post="/api/consent/resolve"
      hx-vals='{{"request_id":"{request_id}","decision":"always"}}'
      hx-target="closest .consent-card" hx-swap="outerHTML">Always allow</button>
    <button type="button" class="gb-consent-btn gb-consent-btn-deny"
      hx-post="/api/consent/resolve"
      hx-vals='{{"request_id":"{request_id}","decision":"deny"}}'
      hx-target="closest .consent-card" hx-swap="outerHTML">Deny</button>
  </div>
</div>"#,
        details = detail_lines(&req.detail),
        request_id = request_id,
        app = app,
        class = class,
    )
}

/// Outcome summary replacing the prompt after the user decides.
/// `outcome` is one of `allow_once`, `always`, `deny`, `expired` or `error`.
pub fn summary_card_html(outcome: &str) -> String {
    let (label, css) = match outcome {
        "always" | "allow_once" => ("Access granted", "gb-consent-ok"),
        "deny" => ("Access denied", "gb-consent-denied"),
        "expired" => ("This request expired. Please try again.", "gb-consent-expired"),
        _ => ("Could not process this request.", "gb-consent-error"),
    };
    format!(
        r#"<div class="consent-card gb-consent-summary {css}">
  <div class="gb-consent-title">{label}</div>
  <div class="gb-consent-subtitle">{outcome}</div>
</div>"#,
        label = escape_html(label),
        css = css,
        outcome = escape_html(outcome),
    )
}

/// Server-rendered permissions table used by the settings fragment endpoint.
pub fn permissions_table_html(rows: &[crate::models::AppPermissionRow]) -> String {
    let body: String = rows
        .iter()
        .map(|r| {
            let expires = r
                .expires_at
                .map(|e| e.to_rfc3339())
                .unwrap_or_else(|| "never".to_string());
            format!(
                r#"<tr><td>{app}</td><td>{class}</td><td>{via}</td><td>{expires}</td>
<td><button type="button" class="gb-consent-btn gb-consent-btn-revoke"
  hx-delete="/api/consent/permissions/{id}" hx-target="closest tr" hx-swap="outerHTML">Revoke</button></td></tr>"#,
                app = escape_html(&r.app_id),
                class = escape_html(&r.action_class),
                via = escape_html(&r.granted_via),
                expires = escape_html(&expires),
                id = r.id,
            )
        })
        .collect();
    if rows.is_empty() {
        return "<p class=\"gb-consent-empty\">No application permissions granted.</p>".to_string();
    }
    format!(
        "<table class=\"gb-consent-table\"><thead><tr><th>Application</th><th>Action class</th><th>Granted via</th><th>Expires at</th><th></th></tr></thead><tbody>{body}</tbody></table>",
        body = body,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn escapes_markup_in_values() {
        assert_eq!(escape_html("<script>&\"'"), "&lt;script&gt;&amp;&quot;&#39;");
    }

    #[test]
    fn prompt_card_targets_closest_consent_card() {
        let req = PendingRequest {
            request_id: Uuid::new_v4().to_string(),
            user_id: Uuid::nil(),
            app_id: "crm".to_string(),
            action_class: "payment".to_string(),
            detail: serde_json::json!({ "amount": "R$ 10,00", "note": "<b>" }),
        };
        let html = prompt_card_html(&req);
        assert!(html.contains("hx-post=\"/api/consent/resolve\""));
        assert!(html.contains("hx-target=\"closest .consent-card\""));
        assert!(html.contains("hx-swap=\"outerHTML\""));
        assert!(html.contains("&lt;b&gt;"));
        assert!(html.contains("\"decision\":\"always\""));
    }

    #[test]
    fn summary_variants_render_distinct_states() {
        assert!(summary_card_html("always").contains("gb-consent-ok"));
        assert!(summary_card_html("deny").contains("gb-consent-denied"));
        assert!(summary_card_html("expired").contains("gb-consent-expired"));
    }
}

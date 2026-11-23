//! Bulk campaign email fan-out with the #2536 compliance gate applied per
//! recipient. Split from `email.rs` to keep both files under the size limit.
//! Also hosts the shared #2889 template renderer (`render_personalized` /
//! `render_plain`) used by every campaign send path.

use diesel::{prelude::*, OptionalExtension};
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;

use crate::campaigns::CrmCampaign;
use crate::compliance::suppression::hydrate_suppressions;
use crate::compliance::{evaluate_send_gate, MarketingChannel};
use crate::email::{send_campaign_email, EmailCampaignPayload};
use crate::schema::{marketing_campaigns, marketing_contacts, marketing_recipients};
use crate::state::AppState;

/// HTML-escapes a caller/DB-sourced value for interpolation into a campaign
/// template (#2889). Quotes are escaped too, so the same function is safe in a
/// text node and inside a quoted attribute value. This is the local equivalent
/// of `html_escape::encode_text`; keeping it here avoids a new dependency.
pub(crate) fn escape_html(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for c in value.chars() {
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

/// HTML-safe value for a URL-bearing attribute (`href`, `src`, …), or `None`
/// when the value would hand the client an executable scheme. Whitespace and
/// control characters are ignored when checking the scheme so
/// `java\nscript:` and leading-space variants cannot slip through.
pub(crate) fn safe_url(value: &str) -> Option<String> {
    let compact: String = value
        .chars()
        .filter(|c| !c.is_ascii_control() && !c.is_ascii_whitespace())
        .collect();
    let lower = compact.to_ascii_lowercase();
    if lower.starts_with("javascript:")
        || lower.starts_with("vbscript:")
        || lower.starts_with("data:text/html")
        || lower.starts_with("data:application/javascript")
    {
        return None;
    }
    Some(escape_html(value))
}

/// Strips CR/LF, other control characters and Unicode line separators from a
/// value used as a WhatsApp template parameter or plain-text substitution
/// (#2889). A contact name must not be able to inject extra lines/parameters.
pub(crate) fn plain_param(value: &str) -> String {
    value
        .chars()
        .filter(|c| !c.is_control() && *c != '\u{2028}' && *c != '\u{2029}')
        .collect()
}

/// True when `pos` sits inside a tag whose current attribute is a URL target.
/// Only the template (trusted campaign editor output) is inspected here; the
/// untrusted value itself is always escaped/sanitized before being inserted.
fn url_attribute_before(template: &str, pos: usize) -> bool {
    let before = &template[..pos];
    let Some(tag_start) = before.rfind('<') else {
        return false;
    };
    let tag = &template[tag_start + 1..pos];
    if tag.contains('>') {
        return false;
    }
    let Some(eq) = tag.rfind('=') else {
        return false;
    };
    let attr = tag[..eq]
        .trim_end_matches(|c: char| c.is_ascii_whitespace() || c == '"' || c == '\'')
        .rsplit(|c: char| c.is_ascii_whitespace() || c == '"' || c == '\'')
        .next();
    match attr {
        Some(attr) => matches!(
            attr.to_ascii_lowercase().as_str(),
            "href" | "src" | "action" | "formaction" | "xlink:href" | "poster" | "background"
                | "data"
        ),
        None => false,
    }
}

/// Single-pass placeholder expansion. Values are written straight to the
/// output buffer and never rescanned, so a contact value containing
/// `{{email}}` cannot be expanded a second time. Unknown placeholders are
/// copied verbatim. `sanitize` protects the value for the target context;
/// URL-bearing attributes additionally reject executable schemes.
fn render_with(
    template: &str,
    name: &str,
    email: &str,
    sanitize: fn(&str) -> String,
    guard_urls: bool,
) -> String {
    let mut out = String::with_capacity(template.len() + name.len() + email.len());
    let mut cursor = 0usize;
    while let Some(offset) = template[cursor..].find("{{") {
        let start = cursor + offset;
        let after_open = start + 2;
        let Some(rest) = template.get(after_open..) else {
            break;
        };
        let Some(close_offset) = rest.find("}}") else {
            break;
        };
        let end = after_open + close_offset + 2;
        match &rest[..close_offset] {
            key @ ("name" | "email") => {
                let value = if key == "name" { name } else { email };
                let rendered = if guard_urls && url_attribute_before(template, start) {
                    match safe_url(value) {
                        Some(safe) => safe,
                        None => String::new(),
                    }
                } else {
                    sanitize(value)
                };
                out.push_str(&template[cursor..start]);
                out.push_str(&rendered);
            }
            _ => out.push_str(&template[cursor..end]),
        }
        cursor = end;
    }
    out.push_str(&template[cursor..]);
    out
}

/// HTML e-mail renderer: escapes every substituted value and rejects
/// `javascript:`/`vbscript:` in URL attributes.
pub(crate) fn render_personalized(template: &str, name: &str, email: &str) -> String {
    render_with(template, name, email, escape_html, true)
}

/// WhatsApp/plain-text renderer: substituted values are control-character
/// clean so they cannot inject extra lines or template parameters.
pub(crate) fn render_plain(template: &str, name: &str, email: &str) -> String {
    render_with(template, name, email, plain_param, false)
}

/// Loads the persisted gate inputs for one bulk recipient. `recipient_id` in
/// this path is a `marketing_recipients.id`; consent is reached through its
/// `contact_id`. A missing row fails closed (no consent → skipped).
fn bulk_gate_inputs(
    conn: &mut diesel::PgConnection,
    recipient_id: Uuid,
) -> Result<(Option<String>, Option<bool>, Option<Value>), String> {
    let row = marketing_recipients::table
        .filter(marketing_recipients::id.eq(recipient_id))
        .select((
            marketing_recipients::status,
            marketing_recipients::contact_id,
        ))
        .first::<(Option<String>, Option<Uuid>)>(conn)
        .optional()
        .map_err(|e| format!("Recipient gate query: {e}"))?;
    let Some((status, Some(contact_id))) = row else {
        return Ok((None, None, None));
    };
    let consent = marketing_contacts::table
        .filter(marketing_contacts::id.eq(contact_id))
        .select((marketing_contacts::subscribed, marketing_contacts::metadata))
        .first::<(Option<bool>, Option<Value>)>(conn)
        .optional()
        .map_err(|e| format!("Contact consent query: {e}"))?;
    let (subscribed, metadata) = consent.unwrap_or((None, None));
    Ok((status, subscribed, metadata))
}

pub async fn send_bulk_campaign_emails(
    state: &Arc<AppState>,
    campaign_id: Uuid,
    contacts: Vec<(Uuid, String, String)>,
) -> Result<(i32, i32), String> {
    let mut sent = 0;
    let mut failed = 0;

    let (_, _, bot_id) = state.get_scope();
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {e}"))?;

    // #2536 — hydrate the durable do-not-contact list before gating recipients.
    if let Err(error) = hydrate_suppressions(&mut conn) {
        log::error!("bulk email could not hydrate suppressions: {error}");
    }

    let campaign: CrmCampaign = marketing_campaigns::table
        .filter(marketing_campaigns::id.eq(campaign_id))
        .first(&mut conn)
        .map_err(|_| "Campaign not found")?;

    let default_metrics = serde_json::json!({});
    let template_data = campaign.metrics.as_ref().unwrap_or(&default_metrics);
    let subject = template_data
        .get("subject")
        .and_then(|s| s.as_str())
        .unwrap_or("Newsletter")
        .to_string();

    let body_html = template_data
        .get("body")
        .and_then(|b| b.as_str())
        .map(String::from);

    for (contact_id, email, name) in contacts {
        // #2536 — this bulk path must not bypass the gate used by the worker
        // and the publish fan-out. Recipients without consent are skipped and
        // never handed to the email provider.
        let (recipient_status, subscribed, metadata) =
            bulk_gate_inputs(&mut conn, contact_id)?;
        let decision = evaluate_send_gate(
            campaign.branch_id,
            MarketingChannel::Email,
            &email,
            recipient_status.as_deref(),
            subscribed,
            metadata.as_ref(),
        );
        if !decision.is_allowed() {
            log::info!("bulk email skipped for {email}: {}", decision.status());
            continue;
        }

        let personalized_body = body_html
            .as_ref()
            .map(|html| render_personalized(html, &name, &email));

        let payload = EmailCampaignPayload {
            to: email,
            subject: subject.clone(),
            body_html: personalized_body,
            body_text: None,
            campaign_id: Some(campaign_id),
            recipient_id: Some(contact_id),
        };

        match send_campaign_email(state, bot_id, payload).await {
            Ok(result) => {
                if result.success {
                    sent += 1;
                } else {
                    failed += 1;
                    log::error!("Email send failed: {:?}", result.error);
                }
            }
            Err(e) => {
                failed += 1;
                log::error!("Email error: {}", e);
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }

    Ok((sent, failed))
}

#[cfg(test)]
mod template_tests {
    use super::{escape_html, plain_param, render_personalized, render_plain};

    #[test]
    fn img_onerror_payload_is_escaped() {
        let name = "Alice</p><img src=x onerror=alert(1)>";
        let out = render_personalized("Hi {{name}}", name, "a@example.com");
        assert_eq!(out, "Hi Alice&lt;/p&gt;&lt;img src=x onerror=alert(1)&gt;");
        assert!(!out.contains("<img"));
    }

    #[test]
    fn quote_breakout_in_attribute_is_escaped() {
        let name = "\" onmouseover=\"alert(1)";
        let out = render_personalized(
            r#"<span title="{{name}}">Hi {{email}}</span>"#,
            name,
            "a\"@example.com",
        );
        assert_eq!(
            out,
            r#"<span title="&quot; onmouseover=&quot;alert(1)">Hi a&quot;@example.com</span>"#
        );
    }

    #[test]
    fn javascript_url_in_href_is_rejected() {
        for payload in ["javascript:alert(1)", "  JaVa\nScRiPt:alert(1)", "vbscript:msgbox(1)"] {
            let out = render_personalized(r#"<a href="{{name}}">x</a>"#, payload, "a@example.com");
            assert_eq!(out, r#"<a href="">x</a>"#, "payload: {payload}");
        }
        let safe = render_personalized(
            r#"<a href="{{email}}">x</a>"#,
            "n",
            "https://x.example/?a=1&b=2",
        );
        assert_eq!(safe, r#"<a href="https://x.example/?a=1&amp;b=2">x</a>"#);
    }

    #[test]
    fn single_pass_prevents_placeholder_injection() {
        let out = render_personalized("{{name}}", "{{email}}", "real@example.com");
        assert_eq!(out, "{{email}}");
        assert_eq!(
            render_personalized("{{company}} {{name}}", "Ada", "a@example.com"),
            "{{company}} Ada"
        );
    }

    #[test]
    fn html_special_characters_and_ampersands_are_escaped() {
        assert_eq!(escape_html("&<>\"'"), "&amp;&lt;&gt;&quot;&#39;");
    }

    #[test]
    fn plain_renderer_strips_line_and_parameter_injection() {
        let out = render_plain("Hi {{name}}", "A\nB\rC\u{2028}D\u{0000}E", "a@example.com");
        assert_eq!(out, "Hi ABCDE");
        assert_eq!(plain_param("\t x \n"), " x ");
    }
}

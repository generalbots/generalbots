//! Bulk campaign email fan-out with the #2536 compliance gate applied per
//! recipient. Split from `email.rs` to keep both files under the size limit.

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

        let personalized_body = body_html.as_ref().map(|html| {
            html.replace("{{name}}", &name)
                .replace("{{email}}", &email)
        });

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

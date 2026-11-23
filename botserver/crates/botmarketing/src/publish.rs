//! Multi-channel campaign publish (Campaign Studio fan-out).
//!
//! One designed campaign (metrics.subject/body/images jsonb) distributes to
//! email, WhatsApp and Instagram recipients from a chosen list. Recipient
//! tracking rows are written first so the unified monitor reflects every
//! per-recipient status (sent/failed/delivered/opened/clicked).

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use diesel::prelude::*;
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;

use crate::compliance::{
    all_channels_failed_body, audience_has_consent, evaluate_send_gate, ChannelError,
    MarketingChannel,
};
use crate::email::EmailCampaignPayload;
use crate::email_bulk::render_personalized;
use crate::publish_support::{
    channel_failure, hydrate_gate, log_state_result, merge_metrics, plain_text, publish_instagram,
    record_skip_event, set_recipient_state,
};
use crate::schema::{marketing_campaigns, marketing_contacts, marketing_recipients};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct PublishRequest {
    pub list_id: Option<Uuid>,
    pub channels: Option<Vec<String>>,
}

pub async fn publish_campaign(
    State(state): State<Arc<AppState>>,
    Path(campaign_id): Path<Uuid>,
    Json(req): Json<PublishRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}")))?;

    // #2536 — the durable list is the gate source of truth after a restart.
    hydrate_gate(&mut conn);

    let campaign: crate::campaigns::CrmCampaign = marketing_campaigns::table
        .filter(marketing_campaigns::id.eq(campaign_id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Campaign not found".to_string()))?;

    let (_, _, bot_id) = state.get_scope();
    let metrics = campaign.metrics.clone().unwrap_or_else(|| serde_json::json!({}));
    let subject = metrics
        .get("subject")
        .and_then(|s| s.as_str())
        .unwrap_or("Newsletter")
        .to_string();
    let body_html = metrics.get("body").and_then(|b| b.as_str()).unwrap_or("").to_string();
    let images = metrics.get("images").cloned().unwrap_or_else(|| serde_json::json!({}));

    let list_id = req.list_id.ok_or_else(|| {
        (StatusCode::BAD_REQUEST, "list_id is required — choose a marketing list".to_string())
    })?;

    // Resolve recipients from the list's marketing_contacts rows. #2537:
    // consent and CRM suppression are applied here, at send time, so stale
    // membership rows can never resurrect an opted-out recipient.
    let contact_rows: Vec<(
        Uuid,
        String,
        Option<String>,
        Option<String>,
        Option<bool>,
        Option<Value>,
    )> = marketing_contacts::table
        .filter(marketing_contacts::list_id.eq(list_id))
        .select((
            marketing_contacts::id,
            marketing_contacts::email,
            marketing_contacts::name,
            marketing_contacts::phone,
            marketing_contacts::subscribed,
            marketing_contacts::metadata,
        ))
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Recipients query: {e}")))?;

    let suppressed = crate::segment::suppressed_crm_emails(&mut conn, campaign.branch_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Suppression query: {e}")))?;
    // #2536 — keep the consent columns on the contact tuple: the per-channel
    // gate runs again immediately before each send, not just at list refresh.
    let contacts: Vec<(
        Uuid,
        String,
        Option<String>,
        Option<String>,
        Option<bool>,
        Option<Value>,
    )> = contact_rows
        .into_iter()
        .filter(|(_, email, _, _, _, _)| !suppressed.contains(&email.trim().to_lowercase()))
        .collect();

    if contacts.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "No recipients resolved for this list".to_string()));
    }

    let channels: Vec<String> = match req.channels {
        Some(c) if !c.is_empty() => c,
        _ => match campaign.campaign_type.as_str() {
            "multi" => vec!["email".into(), "whatsapp".into(), "instagram".into()],
            "whatsapp" => vec!["whatsapp".into()],
            "instagram" => vec!["instagram".into()],
            _ => vec!["email".into()],
        },
    };

    // Rewrite tracking rows for this send: fresh queued state per recipient.
    diesel::delete(
        marketing_recipients::table.filter(marketing_recipients::campaign_id.eq(campaign_id)),
    )
    .execute(&mut conn)
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Reset recipients: {e}")))?;

    let now = chrono::Utc::now();
    for (cid, email, name, _phone, _, _) in &contacts {
        diesel::insert_into(marketing_recipients::table)
            .values((
                marketing_recipients::id.eq(Uuid::new_v4()),
                marketing_recipients::branch_id.eq(campaign.branch_id),
                marketing_recipients::campaign_id.eq(campaign_id),
                marketing_recipients::list_id.eq(list_id),
                marketing_recipients::contact_id.eq(*cid),
                marketing_recipients::email.eq(email.clone()),
                marketing_recipients::name.eq(name.clone()),
                marketing_recipients::status.eq("queued"),
                marketing_recipients::created_at.eq(now),
            ))
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Queue recipients: {e}")))?;
    }

    let mut totals = serde_json::Map::new();
    // #2536 — typed per-channel failures. A channel with no provider must
    // never be reported as a successful send.
    let mut channel_errors = serde_json::Map::new();

    for channel in &channels {
        let mut sent = 0i64;
        let mut failed = 0i64;
        let mut skipped = 0i64;
        match channel.as_str() {
            "email" => {
                for (cid, email, name, _phone, subscribed, metadata) in &contacts {
                    if email.trim().is_empty() {
                        skipped += 1;
                        continue;
                    }
                    let decision = evaluate_send_gate(
                        campaign.branch_id,
                        MarketingChannel::Email,
                        email,
                        None,
                        *subscribed,
                        metadata.as_ref(),
                    );
                    if !decision.is_allowed() {
                        skipped += 1;
                        log::info!("email publish skipped for {email}: {}", decision.status());
                        record_skip_event(
                            &mut conn,
                            campaign.branch_id,
                            campaign_id,
                            "email",
                            Some(email.as_str()),
                            decision.status(),
                        );
                        if let Err(error) =
                            set_recipient_state(&mut conn, campaign_id, *cid, decision.status(), None)
                        {
                            log::error!("email skip state not persisted: {error}");
                        }
                        continue;
                    }
                    let personalized =
                        render_personalized(&body_html, name.as_deref().unwrap_or(""), email);
                    let result = crate::email::send_campaign_email(
                        &state,
                        bot_id,
                        EmailCampaignPayload {
                            to: email.clone(),
                            subject: subject.clone(),
                            body_html: Some(personalized),
                            body_text: None,
                            campaign_id: Some(campaign_id),
                            recipient_id: Some(*cid),
                        },
                    )
                    .await;
                    match result {
                        Ok(r) if r.success => {
                            sent += 1;
                            log_state_result(
                                set_recipient_state(&mut conn, campaign_id, *cid, "sent", None),
                                "email sent",
                            );
                        }
                        Ok(_) => {
                            failed += 1;
                            log_state_result(
                                set_recipient_state(&mut conn, campaign_id, *cid, "failed", None),
                                "email failed",
                            );
                        }
                        Err(e) => {
                            failed += 1;
                            log::warn!("email send failed for {email}: {e}");
                            log_state_result(
                                set_recipient_state(
                                    &mut conn,
                                    campaign_id,
                                    *cid,
                                    "failed",
                                    Some(&e),
                                ),
                                "email failed",
                            );
                        }
                    }
                }
            }
            "whatsapp" => {
                for (cid, _email, name, phone, subscribed, metadata) in &contacts {
                    let number = phone.as_deref().unwrap_or("").trim();
                    if number.is_empty() {
                        skipped += 1;
                        continue;
                    }
                    let decision = evaluate_send_gate(
                        campaign.branch_id,
                        MarketingChannel::Whatsapp,
                        number,
                        None,
                        *subscribed,
                        metadata.as_ref(),
                    );
                    if !decision.is_allowed() {
                        skipped += 1;
                        log::info!("whatsapp publish skipped for {number}: {}", decision.status());
                        record_skip_event(
                            &mut conn,
                            campaign.branch_id,
                            campaign_id,
                            "whatsapp",
                            Some(number),
                            decision.status(),
                        );
                        if let Err(error) =
                            set_recipient_state(&mut conn, campaign_id, *cid, decision.status(), None)
                        {
                            log::error!("whatsapp skip state not persisted: {error}");
                        }
                        continue;
                    }
                    let message = plain_text(&body_html);
                    match (state.send_whatsapp)(bot_id, number, &message, name.as_deref(), None) {
                        Ok(_) => {
                            sent += 1;
                            log_state_result(
                                set_recipient_state(&mut conn, campaign_id, *cid, "sent", None),
                                "whatsapp sent",
                            );
                        }
                        Err(e) => {
                            failed += 1;
                            log::warn!("whatsapp send failed for {number}: {e}");
                            log_state_result(
                                set_recipient_state(
                                    &mut conn,
                                    campaign_id,
                                    *cid,
                                    "failed",
                                    Some(&e),
                                ),
                                "whatsapp failed",
                            );
                        }
                    }
                }
            }
            "instagram" => {
                // #2536 — Instagram publishes to the brand account, not to
                // individual recipients. Gate the broadcast on its audience:
                // no consented contact on this branch/channel means no post.
                let eligible = audience_has_consent(
                    campaign.branch_id,
                    MarketingChannel::Instagram,
                    contacts.iter().map(|(_, email, _, phone, subscribed, metadata)| {
                        (
                            email.as_str(),
                            phone.as_deref(),
                            *subscribed,
                            metadata.as_ref(),
                        )
                    }),
                );
                if !eligible {
                    let error = ChannelError::NoConsentedRecipients {
                        channel: "instagram".to_string(),
                    };
                    channel_errors.insert("instagram".to_string(), error.to_value());
                    record_skip_event(
                        &mut conn,
                        campaign.branch_id,
                        campaign_id,
                        "instagram",
                        None,
                        error.code(),
                    );
                }
                let image_url = if eligible {
                    images
                        .get("instagram")
                        .and_then(|v| v.as_array())
                        .and_then(|a| a.first())
                        .and_then(|v| v.as_str())
                        .or_else(|| {
                            images
                                .get("email")
                                .and_then(|v| v.as_array())
                                .and_then(|a| a.first())
                                .and_then(|v| v.as_str())
                        })
                        .map(String::from)
                } else {
                    None
                };
                let caption = plain_text(&body_html);
                match image_url {
                    Some(url) => {
                        let result = publish_instagram(&state, &bot_id, &url, &caption).await;
                        match result {
                            Ok(_) => {
                                sent += 1;
                                if let Err(error) = set_recipient_state(
                                    &mut conn,
                                    campaign_id,
                                    Uuid::nil(),
                                    "sent",
                                    None,
                                ) {
                                    log::error!("instagram recipient state not persisted: {error}");
                                }
                            }
                            Err(e) => {
                                failed += 1;
                                log::warn!("instagram publish failed: {e}");
                                if let Err(error) = set_recipient_state(
                                    &mut conn,
                                    campaign_id,
                                    Uuid::nil(),
                                    "failed",
                                    Some(&e),
                                ) {
                                    log::error!("instagram recipient state not persisted: {error}");
                                }
                            }
                        }
                    }
                    None => {
                        skipped += 1;
                        if eligible {
                            // Audience passed the gate but there is no image to
                            // post: record the skip so it is auditable too.
                            record_skip_event(
                                &mut conn,
                                campaign.branch_id,
                                campaign_id,
                                "instagram",
                                None,
                                "no_content",
                            );
                            log::warn!(
                                "instagram publish skipped: no generated image in metrics.images"
                            );
                        } else {
                            log::info!(
                                "instagram publish skipped: no consented recipients on list {list_id}"
                            );
                        }
                    }
                }
            }
            "sms" => {
                // #2536 — no SMS provider is wired. Return a typed error rather
                // than counting this as a successful delivery.
                channel_errors.insert(
                    "sms".to_string(),
                    channel_failure(&mut conn, campaign.branch_id, campaign_id, "sms"),
                );
                skipped += 1;
                log::warn!("sms publish rejected: no provider configured");
            }
            _ => {
                // #2536 — an unrecognised channel is never a silent skip.
                channel_errors.insert(
                    channel.clone(),
                    channel_failure(&mut conn, campaign.branch_id, campaign_id, channel),
                );
                skipped += 1;
                log::warn!("publish rejected unsupported channel {channel}: no provider configured");
            }
        }
        totals.insert(
            channel.clone(),
            serde_json::json!({ "sent": sent, "failed": failed, "skipped": skipped }),
        );
    }

    // Merge fan-out totals into the campaign metrics (counters preserved).
    merge_metrics(&mut conn, campaign_id, &totals).map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Metrics update: {e}"))
    })?;

    let total_sent: i64 = totals.values().map(|v| v["sent"].as_i64().unwrap_or(0)).sum();
    let total_failed: i64 = totals.values().map(|v| v["failed"].as_i64().unwrap_or(0)).sum();

    // A request whose channels all failed the compliance gate or have no
    // provider is an error, never a successful publish. Mixed requests keep
    // their per-channel typed errors in the success payload below.
    if total_sent == 0 && !channel_errors.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            all_channels_failed_body(&channel_errors),
        ));
    }

    Ok(Json(serde_json::json!({
        "success": channel_errors.is_empty(),
        "sent": total_sent,
        "failed": total_failed,
        "channels": totals,
        "channel_errors": channel_errors,
    })))
}

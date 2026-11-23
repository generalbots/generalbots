use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use chrono::{DateTime, Utc};
use diesel::{prelude::*, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;

use crate::click_sign::{self, TrackingRecord};
use crate::compliance::{
    build_unsubscribe_url, evaluate_send_gate, hydrate_suppressions, unsubscribe_base_url,
    unsubscribe_headers, MarketingChannel, UnsubscribeClaims,
};
use crate::schema::{
    crm_contacts, email_tracking, marketing_campaigns, marketing_contacts, marketing_recipients,
};
use crate::segment::SUPPRESSED_STATUSES;
use crate::state::AppState;
use crate::whatsapp::direct_send_scope;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailCampaignPayload {
    pub to: String,
    pub subject: String,
    pub body_html: Option<String>,
    pub body_text: Option<String>,
    pub campaign_id: Option<Uuid>,
    pub recipient_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailSendResult {
    pub success: bool,
    pub message_id: Option<String>,
    pub tracking_id: Option<Uuid>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignMetrics {
    pub total_sent: i64,
    pub total_delivered: i64,
    pub total_failed: i64,
    pub total_opened: i64,
    pub total_clicked: i64,
    pub open_rate: f64,
    pub click_rate: f64,
    pub bounce_rate: f64,
}

fn inject_tracking_pixel(html: &str, token: Uuid, base_url: &str) -> String {
    let pixel_url = format!("{}/api/marketing/track/open/{}", base_url, token);
    let pixel = format!(
        r#"<img src="{}" width="1" height="1" alt="" style="display:none;visibility:hidden;border:0;" />"#,
        pixel_url
    );

    if html.to_lowercase().contains("</body>") {
        html.replace("</body>", &format!("{}</body>", pixel))
            .replace("</BODY>", &format!("{}</BODY>", pixel))
    } else {
        format!("{}{}", html, pixel)
    }
}

/// Rewrites every `href` in the email body to a signed tracking link.
///
/// #2139 — the destination travels as a signed query parameter (`u`, `e`, `s`),
/// not as a path segment, so the click endpoint can prove the link was issued
/// here. Links that cannot be signed (anchors, `mailto:`, `tel:`, already
/// tracked, or a destination the endpoint would refuse) are left untouched
/// rather than wrapped into a link that can only answer `400`.
fn wrap_tracking_links(
    html: &str,
    record: &TrackingRecord,
    base_url: &str,
    expires_at: i64,
    key: &str,
) -> String {
    const MARKERS: [(&str, char); 2] = [("href=\"", '"'), ("href='", '\'')];

    let mut out = String::with_capacity(html.len());
    let mut cursor = 0usize;
    while cursor < html.len() {
        let Some((offset, quote)) = MARKERS
            .iter()
            .filter_map(|&(marker, quote)| html[cursor..].find(marker).map(|i| (i, quote)))
            .min_by_key(|(i, _)| *i)
        else {
            break;
        };
        // `href="` and `href='` are both six bytes long.
        let value_start = cursor + offset + 6;
        let Some(close) = html[value_start..].find(quote) else {
            break;
        };
        let value_end = value_start + close;
        let original = &html[value_start..value_end];

        out.push_str(&html[cursor..value_start]);
        match tracked_destination(original, record, base_url, expires_at, key) {
            Some(replacement) => out.push_str(&replacement),
            None => out.push_str(original),
        }
        cursor = value_end;
    }
    out.push_str(&html[cursor..]);
    out
}

fn tracked_destination(
    original: &str,
    record: &TrackingRecord,
    base_url: &str,
    expires_at: i64,
    key: &str,
) -> Option<String> {
    let destination = original.trim();
    if destination.is_empty() || destination.starts_with('#') {
        return None;
    }
    let lower = destination.to_ascii_lowercase();
    if lower.starts_with("mailto:")
        || lower.starts_with("tel:")
        || lower.starts_with("cid:")
        || lower.contains("/api/marketing/track/")
    {
        return None;
    }
    // An `href` attribute escapes `&` as `&amp;`; the redirect must carry the
    // URL the browser would have navigated to, not the HTML-escaped spelling.
    let destination = destination.replace("&amp;", "&");
    click_sign::build_click_url(base_url, record, &destination, expires_at, key)
}

/// Loads the persisted gate inputs for a one-off address. A contact without a
/// consent record fails closed, exactly as the bulk/publish paths do (#2891).
pub(crate) fn direct_gate_inputs(
    conn: &mut diesel::PgConnection,
    branch_id: Uuid,
    channel: MarketingChannel,
    address: &str,
) -> Result<(Option<String>, Option<bool>, Option<Value>), String> {
    let normalized = address.trim().to_ascii_lowercase();
    let addr = normalized.as_str();
    let contact = marketing_contacts::table
        .filter(marketing_contacts::branch_id.eq(branch_id))
        .filter(marketing_contacts::email.eq(addr).or(marketing_contacts::phone.eq(addr)))
        .select((marketing_contacts::id, marketing_contacts::subscribed, marketing_contacts::metadata))
        .first::<(Uuid, Option<bool>, Option<Value>)>(conn)
        .optional()
        .map_err(|e| format!("Consent query: {e}"))?;

    let recipient_status = match contact {
        Some((contact_id, _, _)) => marketing_recipients::table
            .filter(marketing_recipients::branch_id.eq(branch_id))
            .filter(marketing_recipients::contact_id.eq(contact_id))
            .select(marketing_recipients::status)
            .order(marketing_recipients::created_at.desc())
            .first::<Option<String>>(conn)
            .optional()
            .map_err(|e| format!("Recipient status query: {e}"))?
            .flatten(),
        None => None,
    };

    // A CRM opt-out suppresses delivery even when a stale contact row says
    // opted in (#2536, #2885).
    let crm_opted_out = crm_contacts::table
        .filter(crm_contacts::branch_id.eq(branch_id))
        .filter(crm_contacts::status.eq_any(SUPPRESSED_STATUSES.to_vec()))
        .select((crm_contacts::email, crm_contacts::phone))
        .load::<(Option<String>, Option<String>)>(conn)
        .map_err(|e| format!("CRM suppression query: {e}"))?
        .iter()
        .any(|(email, phone)| {
            [email.as_deref(), phone.as_deref()]
                .into_iter()
                .flatten()
                .any(|v| v.trim().eq_ignore_ascii_case(&normalized))
        });

    let (subscribed, metadata) = contact.map(|(_, s, m)| (s, m)).unwrap_or((None, None));
    let subscribed = if crm_opted_out { Some(false) } else { subscribed };
    Ok((recipient_status, subscribed, metadata))
}

pub async fn send_campaign_email(
    state: &Arc<AppState>,
    bot_id: Uuid,
    payload: EmailCampaignPayload,
) -> Result<EmailSendResult, String> {
    send_campaign_email_scoped(state, bot_id, payload, None).await
}

/// Same send path, but lets a verified caller branch scope the unsubscribe
/// claim. Direct sends have no campaign row from which to derive that branch,
/// so their claim must otherwise match the gate branch (#2891).
async fn send_campaign_email_scoped(
    state: &Arc<AppState>,
    bot_id: Uuid,
    payload: EmailCampaignPayload,
    caller_branch: Option<Uuid>,
) -> Result<EmailSendResult, String> {
    let open_token = Uuid::new_v4();
    let tracking_id = Uuid::new_v4();

    let base_url = (state.get_config)(&bot_id, "server-url", Some(""))
        .unwrap_or_else(|_| "".to_string());
    // Resolve the default-bot scope before checking out a pooled connection:
    // `get_scope` needs a connection of its own and the pool must never be
    // held across that nested acquisition.
    let scope_branch_id = state.get_scope().1;

    // #2139 — links carry a signed destination instead of a caller-supplied
    // path, so the anonymous click endpoint can never redirect anywhere this
    // server did not put into the email.
    let record = click_sign::TrackingRecord {
        id: tracking_id,
        campaign_id: payload.campaign_id,
        recipient_id: payload.recipient_id,
    };
    let signing_key = click_sign::signing_key();
    let link_expires_at = Utc::now().timestamp() + click_sign::CLICK_LINK_TTL_SECS;

    let body_html = payload
        .body_html
        .map(|html| {
            wrap_tracking_links(&html, &record, &base_url, link_expires_at, signing_key)
        })
        .map(|html| inject_tracking_pixel(&html, open_token, &base_url));

    let mut conn = state.conn.get().map_err(|e| format!("DB connection failed: {}", e))?;

    // `email_tracking` has no `branch_id`/`email`/`event_type` columns in the
    // database (#1517); the recipient is keyed by id and `open_token` is a UUID.
    diesel::insert_into(email_tracking::table)
        .values((
            email_tracking::id.eq(tracking_id),
            email_tracking::recipient_id.eq(payload.recipient_id),
            email_tracking::campaign_id.eq(payload.campaign_id),
            email_tracking::open_token.eq(Some(open_token)),
            email_tracking::open_tracking_enabled.eq(true),
            email_tracking::opened.eq(false),
            email_tracking::clicked.eq(false),
            email_tracking::created_at.eq(Utc::now()),
        ))
        .execute(&mut conn)
        .map_err(|e| format!("Failed to create tracking record: {}", e))?;

    let body = body_html.unwrap_or_else(|| payload.body_text.unwrap_or_default());

    // #2536 — every marketing email carries a signed one-click unsubscribe
    // header. A safe first-party base URL is used when `server-url` is absent;
    // if no signing key is configured the send is refused rather than emitting
    // an email whose unsubscribe link cannot be honoured.
    let header_base = unsubscribe_base_url(&base_url);
    // The token must carry the gate branch so an unsubscribe feeds back into
    // the same suppression key. Direct sends supply the verified caller scope;
    // campaign sends derive it from their campaign row.
    let branch_id = caller_branch.unwrap_or(match payload.campaign_id {
        Some(campaign_id) => marketing_campaigns::table
            .filter(marketing_campaigns::id.eq(campaign_id))
            .select(marketing_campaigns::branch_id)
            .first::<Uuid>(&mut conn)
            .unwrap_or(scope_branch_id),
        None => scope_branch_id,
    });
    let claims = UnsubscribeClaims {
        branch_id,
        channel: MarketingChannel::Email,
        address: payload.to.clone(),
    };
    let headers = match build_unsubscribe_url(&header_base, &claims) {
        Some(url) => unsubscribe_headers(&url),
        None => {
            log::error!("marketing email refused: one-click unsubscribe link unavailable");
            return Err("one-click unsubscribe link unavailable".to_string());
        }
    };

    match (state.send_email)(
        &payload.to,
        &payload.subject,
        &body,
        bot_id,
        Some(headers.as_str()),
    ) {
        Ok(msg_id) => {
            if let Err(error) =
                diesel::update(email_tracking::table.filter(email_tracking::id.eq(tracking_id)))
                    .set(email_tracking::message_id.eq(Some(msg_id.clone())))
                    .execute(&mut conn)
            {
                log::error!("tracking message id not persisted for {tracking_id}: {error}");
            }

            if let Some(recipient_id) = payload.recipient_id {
                if let Err(error) = diesel::update(
                    marketing_recipients::table
                        .filter(marketing_recipients::id.eq(recipient_id)),
                )
                .set((
                    marketing_recipients::status.eq("sent"),
                    marketing_recipients::sent_at.eq(Some(Utc::now())),
                ))
                .execute(&mut conn)
                {
                    log::error!("recipient {recipient_id} sent status not persisted: {error}");
                }
            }

            Ok(EmailSendResult {
                success: true,
                message_id: Some(msg_id),
                tracking_id: Some(tracking_id),
                error: None,
            })
        }
        Err(e) => {
            if let Some(recipient_id) = payload.recipient_id {
                if let Err(error) = diesel::update(
                    marketing_recipients::table
                        .filter(marketing_recipients::id.eq(recipient_id)),
                )
                .set((
                    marketing_recipients::status.eq("failed"),
                    marketing_recipients::failed_at.eq(Some(Utc::now())),
                    marketing_recipients::error_message.eq(Some(e.clone())),
                ))
                .execute(&mut conn)
                {
                    log::error!("recipient {recipient_id} failed status not persisted: {error}");
                }
            }

            Ok(EmailSendResult {
                success: false,
                message_id: None,
                tracking_id: Some(tracking_id),
                error: Some(e),
            })
        }
    }
}

pub async fn get_campaign_email_metrics(
    state: &Arc<AppState>,
    campaign_id: Uuid,
) -> Result<CampaignMetrics, String> {
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {}", e))?;

    let results: Vec<(Option<bool>, Option<bool>)> = email_tracking::table
        .filter(email_tracking::campaign_id.eq(campaign_id))
        .select((email_tracking::opened, email_tracking::clicked))
        .load(&mut conn)
        .map_err(|e| format!("Query error: {}", e))?;

    let total = results.len() as i64;
    let opened = results.iter().filter(|pair| pair.0.unwrap_or(false)).count() as i64;
    let clicked = results.iter().filter(|pair| pair.1.unwrap_or(false)).count() as i64;

    let recipients: Vec<(Option<String>, Option<DateTime<Utc>>)> = marketing_recipients::table
        .filter(marketing_recipients::campaign_id.eq(campaign_id))
        .filter(marketing_recipients::channel.eq("email"))
        .select((marketing_recipients::status, marketing_recipients::sent_at))
        .load(&mut conn)
        .map_err(|e| format!("Query error: {}", e))?;

    let sent = recipients.iter().filter(|(s, _)| s.as_deref() == Some("sent")).count() as i64;
    let failed = recipients.iter().filter(|(s, _)| s.as_deref() == Some("failed")).count() as i64;
    let delivered = sent;

    Ok(CampaignMetrics {
        total_sent: total,
        total_delivered: delivered,
        total_failed: failed,
        total_opened: opened,
        total_clicked: clicked,
        open_rate: if delivered > 0 { (opened as f64 / delivered as f64) * 100.0 } else { 0.0 },
        click_rate: if delivered > 0 { (clicked as f64 / delivered as f64) * 100.0 } else { 0.0 },
        bounce_rate: if sent > 0 { (failed as f64 / sent as f64) * 100.0 } else { 0.0 },
    })
}

// #2536 — bulk campaign sends live in their own module and apply the same
// per-recipient compliance gate as the worker and publish paths.
pub use crate::email_bulk::send_bulk_campaign_emails;

#[derive(Debug, Deserialize)]
pub struct SendEmailRequest {
    pub to: String,
    pub subject: String,
    pub body_html: Option<String>,
    pub body_text: Option<String>,
}

pub async fn send_email_api(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<SendEmailRequest>,
) -> Result<Json<EmailSendResult>, (StatusCode, String)> {
    let (_, branch_id, bot_id) = direct_send_scope(&state, &headers);
    if branch_id.is_nil() {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, "no tenant scope".to_string()));
    }

    // #2891 — the one-off API must pass the same consent/suppression gate as
    // bulk/publish/worker before the provider sees the address.
    let mut conn = state.conn.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}")))?;
    if let Err(error) = hydrate_suppressions(&mut conn) {
        log::error!("direct email could not hydrate suppressions: {error}");
    }
    let (status, subscribed, metadata) =
        direct_gate_inputs(&mut conn, branch_id, MarketingChannel::Email, &req.to)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let decision = evaluate_send_gate(branch_id, MarketingChannel::Email, &req.to,
        status.as_deref(), subscribed, metadata.as_ref());
    if !decision.is_allowed() {
        log::info!("direct email refused for {}: {}", req.to, decision.status());
        return Err((StatusCode::FORBIDDEN, format!("recipient refused: {}", decision.status())));
    }
    drop(conn);

    let payload = EmailCampaignPayload {
        to: req.to,
        subject: req.subject,
        body_html: req.body_html,
        body_text: req.body_text,
        campaign_id: None,
        recipient_id: None,
    };

    match send_campaign_email_scoped(&state, bot_id, payload, Some(branch_id)).await {
        Ok(result) => Ok(Json(result)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

#[cfg(test)]
#[path = "email_tracking_link_tests.rs"]
mod tracking_link_tests;
